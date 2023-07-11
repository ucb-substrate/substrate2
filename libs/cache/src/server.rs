use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{net::SocketAddr, path::PathBuf};

use crate::error::Result;
use crate::rpc::cache::{remote_cache_server::RemoteCacheServer, *};
use fs4::tokio::AsyncFileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_rusqlite::Connection;
use tonic::transport::Server;
use tonic::Response;

pub const CONFIG_MANIFEST_NAME: &str = "Cache.toml";
pub const MANIFEST_DB_NAME: &str = "cache.sqlite";
pub const HEARTBEAT_INTERVAL_SECS: u64 = 2;
pub const HEARTBEAT_TIMEOUT_SECS: u64 = HEARTBEAT_INTERVAL_SECS + 5;

const CREATE_MANIFEST_TABLE_STMT: &str = r#"
    CREATE TABLE IF NOT EXISTS manifest (
        namespace STRING, 
        key BLOB NOT NULL,
        status INTEGER, 
        PRIMARY KEY (namespace, key)
    );
"#;

const READ_MANIFEST_STMT: &str = r#"
    SELECT namespace, key, status FROM manifest;
"#;

const DELETE_ENTRIES_WITH_STATUS_STMT: &str = r#"
    DELETE FROM manifest WHERE status = ?;
"#;

const INSERT_STATUS_STMT: &str = r#"
    INSERT INTO manifest (namespace, key, status) VALUES (?, ?, ?);
"#;

const UPDATE_STATUS_STMT: &str = r#"
    UPDATE manifest SET status = ? WHERE namespace = ? AND key = ?;
"#;

const DELETE_STATUS_STMT: &str = r#"
    DELETE FROM manifest WHERE namespace = ? AND key = ?;
"#;

pub struct CacheServer {
    root: Arc<PathBuf>,
    addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ConfigManifest {
    pub addr: SocketAddr,
}

impl CacheServer {
    pub fn new(root: PathBuf, addr: SocketAddr) -> Self {
        Self {
            root: Arc::new(root),
            addr,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut config_manifest = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.root.join(CONFIG_MANIFEST_NAME))
            .await?;
        config_manifest.try_lock_exclusive()?;
        config_manifest
            .write_all(
                &toml::to_string(&ConfigManifest { addr: self.addr })
                    .unwrap()
                    .into_bytes(),
            )
            .await?;

        let db_path = self.root.join(MANIFEST_DB_NAME);
        let inner = Arc::new(Mutex::new(CacheInner::new(&db_path).await?));
        let svc = RemoteCacheServer::new(RemoteCache::new(self.root.clone(), inner).await?);
        Server::builder().add_service(svc).serve(self.addr).await?;
        drop(config_manifest);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct RemoteCache {
    root: Arc<PathBuf>,
    inner: Arc<Mutex<CacheInner>>,
}

#[derive(Clone, Debug)]
struct CacheInner {
    next_id: AssignmentId,
    entry_status: HashMap<Arc<EntryKey>, EntryStatus>,
    loading: HashMap<AssignmentId, LoadingData>,
    conn: CacheInnerConn,
}

impl CacheInner {
    async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref()).await?;
        conn.call(|conn| {
            let tx = conn.transaction()?;
            tx.execute(CREATE_MANIFEST_TABLE_STMT, ())?;
            tx.commit()?;
            Ok(())
        })
        .await?;
        let mut cache = Self {
            next_id: AssignmentId(0),
            entry_status: HashMap::new(),
            loading: HashMap::new(),
            conn: CacheInnerConn(conn),
        };
        cache.load_from_disk().await?;
        Ok(cache)
    }

    async fn load_from_disk(&mut self) -> Result<()> {
        let rows = self
            .conn
            .0
            .call(|conn| {
                let tx = conn.transaction()?;
                let mut stmt = tx.prepare(DELETE_ENTRIES_WITH_STATUS_STMT)?;
                stmt.execute([DbEntryStatus::Loading.to_int()])?;
                let mut stmt = tx.prepare(READ_MANIFEST_STMT)?;
                let rows = stmt.query_map(
                    [],
                    |row| -> rusqlite::Result<(Arc<EntryKey>, DbEntryStatus)> {
                        Ok((
                            Arc::new(EntryKey {
                                namespace: row.get(0)?,
                                key: row.get(1)?,
                            }),
                            DbEntryStatus::from_int(row.get(2)?).unwrap(),
                        ))
                    },
                )?;
                Ok(rows.collect::<Vec<_>>())
            })
            .await?
            .into_iter()
            .map(|res| res.map_err(|e| e.into()))
            .collect::<std::result::Result<Vec<_>, tokio_rusqlite::Error>>()?;
        self.entry_status = HashMap::from_iter(rows.into_iter().filter_map(|v| {
            Some((
                v.0,
                match v.1 {
                    DbEntryStatus::Loading => None,
                    DbEntryStatus::Ready => Some(EntryStatus::Ready),
                    DbEntryStatus::Evicting => Some(EntryStatus::Evicting),
                }?,
            ))
        }));
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CacheInnerConn(Connection);

impl CacheInnerConn {
    async fn insert_status(&self, key: Arc<EntryKey>, status: DbEntryStatus) -> Result<()> {
        self.0
            .call(move |conn| {
                let mut stmt = conn.prepare(INSERT_STATUS_STMT)?;
                stmt.execute((key.namespace.clone(), key.key.clone(), status.to_int()))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn update_status(&self, key: Arc<EntryKey>, status: DbEntryStatus) -> Result<()> {
        self.0
            .call(move |conn| {
                let mut stmt = conn.prepare(UPDATE_STATUS_STMT)?;
                stmt.execute((status.to_int(), key.namespace.clone(), key.key.clone()))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn delete_status(&self, key: Arc<EntryKey>) -> Result<()> {
        self.0
            .call(move |conn| {
                let mut stmt = conn.prepare(DELETE_STATUS_STMT)?;
                stmt.execute((key.namespace.clone(), key.key.clone()))?;
                Ok(())
            })
            .await?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct AssignmentId(u64);

impl AssignmentId {
    fn increment(&mut self) {
        self.0 += 1
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EntryKey {
    namespace: String,
    key: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
enum EntryStatus {
    Loading(AssignmentId),
    Ready,
    Evicting,
}

impl EntryStatus {
    fn to_db(self) -> DbEntryStatus {
        match self {
            Self::Loading { .. } => DbEntryStatus::Loading,
            Self::Ready => DbEntryStatus::Ready,
            Self::Evicting => DbEntryStatus::Evicting,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum DbEntryStatus {
    Loading,
    Ready,
    Evicting,
}

impl DbEntryStatus {
    fn to_int(self) -> u64 {
        match self {
            Self::Loading => 0,
            Self::Ready => 1,
            Self::Evicting => 2,
        }
    }

    fn from_int(val: u64) -> Option<Self> {
        match val {
            0 => Some(Self::Loading),
            1 => Some(Self::Ready),
            2 => Some(Self::Evicting),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoadingData {
    last_heartbeat: Instant,
    key: Arc<EntryKey>,
}

impl RemoteCache {
    pub async fn new(root: Arc<PathBuf>, inner: Arc<Mutex<CacheInner>>) -> Result<Self> {
        Ok(Self { root, inner })
    }
}

#[tonic::async_trait]
impl remote_cache_server::RemoteCache for RemoteCache {
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> std::result::Result<tonic::Response<GetReply>, tonic::Status> {
        let request = request.into_inner();

        let mut inner = self.inner.lock().await;

        let CacheInner {
            next_id,
            entry_status,
            loading,
            conn,
            ..
        } = &mut *inner;
        let entry_key = Arc::new(EntryKey {
            namespace: request.namespace,
            key: request.key,
        });
        let path = get_file(self.root.as_ref(), &entry_key);
        let entry_status = match entry_status.entry(entry_key.clone()) {
            Entry::Occupied(mut o) => {
                let v = o.get_mut();
                match v {
                    EntryStatus::Loading(id) => {
                        let data = loading
                            .get(id)
                            .ok_or(tonic::Status::internal("unable to retrieve status of key"))?;
                        if Instant::now().duration_since(data.last_heartbeat)
                            > Duration::from_secs(HEARTBEAT_TIMEOUT_SECS)
                        {
                            if request.assign {
                                loading.remove(id);
                                next_id.increment();
                                *id = *next_id;
                                loading.insert(
                                    *id,
                                    LoadingData {
                                        last_heartbeat: Instant::now(),
                                        key: entry_key,
                                    },
                                );
                                get_reply::EntryStatus::Assign(next_id.0)
                            } else {
                                conn.delete_status(entry_key.clone()).await.map_err(|e| {
                                    tonic::Status::internal("unable to persist changes")
                                })?;
                                o.remove_entry();
                                get_reply::EntryStatus::Unassigned(())
                            }
                        } else {
                            get_reply::EntryStatus::Loading(())
                        }
                    }
                    EntryStatus::Ready => {
                        let mut file = File::open(path).await?;
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).await?;
                        get_reply::EntryStatus::Ready(buf)
                    }
                    EntryStatus::Evicting => get_reply::EntryStatus::Unassigned(()),
                }
            }
            Entry::Vacant(v) => {
                if request.assign {
                    next_id.increment();
                    conn.insert_status(entry_key.clone(), DbEntryStatus::Loading)
                        .await
                        .map_err(|e| tonic::Status::internal("unable to persist changes"))?;
                    v.insert(EntryStatus::Loading(*next_id));
                    loading.insert(
                        *next_id,
                        LoadingData {
                            last_heartbeat: Instant::now(),
                            key: entry_key,
                        },
                    );
                    get_reply::EntryStatus::Assign(next_id.0)
                } else {
                    get_reply::EntryStatus::Unassigned(())
                }
            }
        };

        Ok(Response::new(GetReply {
            entry_status: Some(entry_status),
        }))
    }
    async fn heartbeat(
        &self,
        request: tonic::Request<HeartbeatRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let mut inner = self.inner.lock().await;
        match inner.loading.entry(AssignmentId(request.into_inner().id)) {
            Entry::Vacant(_) => {
                return Err(tonic::Status::invalid_argument("invalid assignment id"))
            }
            Entry::Occupied(o) => {
                o.into_mut().last_heartbeat = Instant::now();
            }
        }
        Ok(Response::new(()))
    }
    async fn set(
        &self,
        request: tonic::Request<SetRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let mut inner = self.inner.lock().await;
        let request = request.into_inner();
        let data = inner
            .loading
            .get(&AssignmentId(request.id))
            .ok_or(tonic::Status::invalid_argument("invalid assignment id"))?;

        let key = data.key.clone();
        let path = get_file(self.root.as_ref(), &key);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await?;
        f.write_all(&request.value).await?;

        inner
            .conn
            .update_status(key.clone(), DbEntryStatus::Ready)
            .await
            .map_err(|e| tonic::Status::internal("unable to persist changes"))?;

        let status = inner
            .entry_status
            .get_mut(&key)
            .ok_or(tonic::Status::internal("unable to retrieve status of key"))?;
        *status = EntryStatus::Ready;
        Ok(Response::new(()))
    }
}

fn get_file(root: impl AsRef<Path>, key: impl AsRef<EntryKey>) -> PathBuf {
    let root = root.as_ref();
    let key = key.as_ref();
    root.join(hex::encode(crate::hash(key.namespace.as_bytes())))
        .join(hex::encode(crate::hash(&key.key)))
}
