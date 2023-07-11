//! A gRPC persistent cache server.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{net::SocketAddr, path::PathBuf};

use fs4::tokio::AsyncFileExt;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_rusqlite::Connection;
use tonic::transport::Server;
use tonic::Response;
use tracing::Level;

use crate::error::Result;
use crate::rpc::local::{
    self,
    local_cache_server::{LocalCache, LocalCacheServer},
};
use crate::rpc::remote::{
    self,
    remote_cache_server::{RemoteCache, RemoteCacheServer},
};

/// The name of the config manifest TOML file.
pub const CONFIG_MANIFEST_NAME: &str = "Cache.toml";

/// The name of the main manifest database.
pub const MANIFEST_DB_NAME: &str = "cache.sqlite";

/// The expected interval between heartbeats.
pub const HEARTBEAT_INTERVAL_SECS: u64 = 2;

/// The timeout before an assigned task is assumed to have failed.
pub const HEARTBEAT_TIMEOUT_SECS: u64 = HEARTBEAT_INTERVAL_SECS + 2;

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

/// A gRPC cache server.
pub struct CacheServer {
    root: Arc<PathBuf>,
    remote_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub(crate) struct ConfigManifest {
    pub(crate) remote_addr: Option<SocketAddr>,
    pub(crate) local_addr: Option<SocketAddr>,
}

impl CacheServer {
    /// Creates a new [`CacheServer`] object without setting up any gRPC servers.
    ///
    /// Use [`CacheServer::with_remote`] and [`CacheServer::with_local`] to configure the desired
    /// servers.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: Arc::new(root),
            remote_addr: None,
            local_addr: None,
        }
    }

    /// Set up the remote cache gRPC server.
    pub fn with_remote(mut self, addr: SocketAddr) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    /// Set up the local cache gRPC server.
    pub fn with_local(mut self, addr: SocketAddr) -> Self {
        self.local_addr = Some(addr);
        self
    }

    /// Starts the gRPC server, listening on the configured address.
    pub async fn start(&self) -> Result<()> {
        if let (None, None) = (self.local_addr, self.remote_addr) {
            tracing::event!(
                Level::WARN,
                "no local or remote address specified so no server is being run"
            );
        }
        let mut config_manifest = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.root.join(CONFIG_MANIFEST_NAME))
            .await?;
        config_manifest.try_lock_exclusive()?;
        config_manifest
            .write_all(
                &toml::to_string(&ConfigManifest {
                    remote_addr: self.remote_addr,
                    local_addr: self.local_addr,
                })
                .unwrap()
                .into_bytes(),
            )
            .await?;

        let db_path = self.root.join(MANIFEST_DB_NAME);
        let inner = Arc::new(Mutex::new(CacheInner::new(&db_path).await?));
        let imp = CacheImpl::new(self.root.clone(), inner);
        let remote_svc = RemoteCacheServer::new(imp.clone());
        let local_svc = LocalCacheServer::new(imp);
        let mut handle = None;
        if let Some(addr) = self.remote_addr {
            handle = Some(tokio::spawn(
                Server::builder().add_service(remote_svc).serve(addr),
            ));
        }
        if let Some(addr) = self.local_addr {
            handle = Some(tokio::spawn(
                Server::builder().add_service(local_svc).serve(addr),
            ));
        }

        if let Some(handle) = handle {
            handle.await??;
        }
        drop(config_manifest);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct CacheInner {
    next_assignment_id: AssignmentId,
    next_handle_id: HandleId,
    entry_status: HashMap<Arc<EntryKey>, EntryStatus>,
    loading: HashMap<AssignmentId, LoadingData>,
    handles: HashMap<HandleId, Arc<EntryKey>>,
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
            next_assignment_id: AssignmentId(0),
            next_handle_id: HandleId(0),
            entry_status: HashMap::new(),
            loading: HashMap::new(),
            handles: HashMap::new(),
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
                    DbEntryStatus::Ready => Some(EntryStatus::Ready(0)),
                    DbEntryStatus::Evicting => Some(EntryStatus::Evicting),
                }?,
            ))
        }));
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct CacheInnerConn(Connection);

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

/// An ID corresponding to a client assigned to generate a certain value.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
struct AssignmentId(u64);

impl AssignmentId {
    fn increment(&mut self) {
        self.0 += 1
    }
}

/// An ID corresponding to a client that currently has a handle to a ready entry.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
struct HandleId(u64);

impl HandleId {
    fn increment(&mut self) {
        self.0 += 1
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct EntryKey {
    namespace: String,
    key: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
enum EntryStatus {
    Loading(AssignmentId),
    /// Number of local requests that are using this entry.
    Ready(u64),
    Evicting,
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
struct LoadingData {
    last_heartbeat: Instant,
    key: Arc<EntryKey>,
}

#[derive(Clone, Debug)]
enum GetReplyStatus {
    Unassigned,
    Assign(AssignmentId),
    Loading,
    Ready(Vec<u8>),
    ReadyLocal(HandleId),
}

impl GetReplyStatus {
    fn into_remote(self) -> remote::get_reply::EntryStatus {
        match self {
            Self::Unassigned => remote::get_reply::EntryStatus::Unassigned(()),
            Self::Assign(id) => remote::get_reply::EntryStatus::Assign(id.0),
            Self::Loading => remote::get_reply::EntryStatus::Loading(()),
            Self::Ready(val) => remote::get_reply::EntryStatus::Ready(val),
            Self::ReadyLocal(_) => panic!("cannot convert local statuses to remote statuses"),
        }
    }
    fn into_local(self, path: String) -> local::get_reply::EntryStatus {
        match self {
            Self::Unassigned => local::get_reply::EntryStatus::Unassigned(()),
            Self::Assign(id) => {
                local::get_reply::EntryStatus::Assign(local::IdPath { id: id.0, path })
            }
            Self::Loading => local::get_reply::EntryStatus::Loading(()),
            Self::ReadyLocal(id) => {
                local::get_reply::EntryStatus::Ready(local::IdPath { id: id.0, path })
            }
            Self::Ready(_) => panic!("cannot convert remote statuses to local statuses"),
        }
    }
}

#[derive(Clone, Debug)]
struct CacheImpl {
    root: Arc<PathBuf>,
    inner: Arc<Mutex<CacheInner>>,
}

impl CacheImpl {
    pub fn new(root: Arc<PathBuf>, inner: Arc<Mutex<CacheInner>>) -> Self {
        Self { root, inner }
    }

    async fn get_impl(
        &self,
        entry_key: Arc<EntryKey>,
        assign: bool,
        local: bool,
    ) -> std::result::Result<GetReplyStatus, tonic::Status> {
        let mut inner = self.inner.lock().await;

        let CacheInner {
            next_assignment_id,
            next_handle_id,
            entry_status,
            loading,
            handles,
            conn,
            ..
        } = &mut *inner;
        let path = get_file(self.root.as_ref(), &entry_key);
        Ok(match entry_status.entry(entry_key.clone()) {
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
                            if assign {
                                loading.remove(id);
                                next_assignment_id.increment();
                                *id = *next_assignment_id;
                                loading.insert(
                                    *id,
                                    LoadingData {
                                        last_heartbeat: Instant::now(),
                                        key: entry_key,
                                    },
                                );
                                GetReplyStatus::Assign(*id)
                            } else {
                                conn.delete_status(entry_key.clone()).await.map_err(|_| {
                                    tonic::Status::internal("unable to persist changes")
                                })?;
                                o.remove_entry();
                                GetReplyStatus::Unassigned
                            }
                        } else {
                            GetReplyStatus::Loading
                        }
                    }
                    EntryStatus::Ready(in_use) => {
                        if local {
                            *in_use += 1;
                            next_handle_id.increment();
                            handles.insert(*next_handle_id, entry_key);
                            GetReplyStatus::ReadyLocal(*next_handle_id)
                        } else {
                            let mut file = File::open(path).await?;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf).await?;
                            GetReplyStatus::Ready(buf)
                        }
                    }
                    EntryStatus::Evicting => GetReplyStatus::Unassigned,
                }
            }
            Entry::Vacant(v) => {
                if assign {
                    next_assignment_id.increment();
                    conn.insert_status(entry_key.clone(), DbEntryStatus::Loading)
                        .await
                        .map_err(|_| tonic::Status::internal("unable to persist changes"))?;
                    v.insert(EntryStatus::Loading(*next_assignment_id));
                    loading.insert(
                        *next_assignment_id,
                        LoadingData {
                            last_heartbeat: Instant::now(),
                            key: entry_key,
                        },
                    );
                    GetReplyStatus::Assign(*next_assignment_id)
                } else {
                    GetReplyStatus::Unassigned
                }
            }
        })
    }

    async fn heartbeat_impl(&self, id: AssignmentId) -> std::result::Result<(), tonic::Status> {
        let mut inner = self.inner.lock().await;
        match inner.loading.entry(id) {
            Entry::Vacant(_) => {
                return Err(tonic::Status::invalid_argument("invalid assignment id"))
            }
            Entry::Occupied(o) => {
                o.into_mut().last_heartbeat = Instant::now();
            }
        }
        Ok(())
    }

    async fn set_impl(
        &self,
        id: AssignmentId,
        value: Option<Vec<u8>>,
    ) -> std::result::Result<(), tonic::Status> {
        let mut inner = self.inner.lock().await;
        let data = inner
            .loading
            .get(&id)
            .ok_or(tonic::Status::invalid_argument("invalid assignment id"))?;

        let key = data.key.clone();
        if let Some(value) = value {
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
            f.write_all(&value).await?;
        }

        inner
            .conn
            .update_status(key.clone(), DbEntryStatus::Ready)
            .await
            .map_err(|_| tonic::Status::internal("unable to persist changes"))?;

        let status = inner
            .entry_status
            .get_mut(&key)
            .ok_or(tonic::Status::internal("unable to retrieve status of key"))?;
        *status = EntryStatus::Ready(0);
        Ok(())
    }
}

#[tonic::async_trait]
impl RemoteCache for CacheImpl {
    async fn get(
        &self,
        request: tonic::Request<remote::GetRequest>,
    ) -> std::result::Result<tonic::Response<remote::GetReply>, tonic::Status> {
        let request = request.into_inner();

        let entry_key = Arc::new(EntryKey {
            namespace: request.namespace,
            key: request.key,
        });

        let entry_status = self
            .get_impl(entry_key, request.assign, false)
            .await?
            .into_remote();

        Ok(Response::new(remote::GetReply {
            entry_status: Some(entry_status),
        }))
    }

    async fn heartbeat(
        &self,
        request: tonic::Request<remote::HeartbeatRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.heartbeat_impl(AssignmentId(request.into_inner().id))
            .await?;
        Ok(Response::new(()))
    }

    async fn set(
        &self,
        request: tonic::Request<remote::SetRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();
        self.set_impl(AssignmentId(request.id), Some(request.value))
            .await?;
        Ok(Response::new(()))
    }
}

#[tonic::async_trait]
impl LocalCache for CacheImpl {
    async fn get(
        &self,
        request: tonic::Request<local::GetRequest>,
    ) -> std::result::Result<tonic::Response<local::GetReply>, tonic::Status> {
        let request = request.into_inner();

        let entry_key = Arc::new(EntryKey {
            namespace: request.namespace,
            key: request.key,
        });

        let path = get_file(self.root.as_ref(), &entry_key)
            .absolutize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let entry_status = self
            .get_impl(entry_key, request.assign, true)
            .await?
            .into_local(path);

        Ok(Response::new(local::GetReply {
            entry_status: Some(entry_status),
        }))
    }

    async fn heartbeat(
        &self,
        request: tonic::Request<local::HeartbeatRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.heartbeat_impl(AssignmentId(request.into_inner().id))
            .await?;
        Ok(Response::new(()))
    }

    async fn done(
        &self,
        request: tonic::Request<local::DoneRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();
        self.set_impl(AssignmentId(request.id), None).await?;
        Ok(Response::new(()))
    }

    async fn drop(
        &self,
        request: tonic::Request<local::DropRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request = request.into_inner();
        let mut inner = self.inner.lock().await;

        let CacheInner {
            handles,
            entry_status,
            ..
        } = &mut *inner;

        let handle_id = HandleId(request.id);
        let entry_key = handles
            .get(&handle_id)
            .ok_or(tonic::Status::invalid_argument("invalid handle id"))?;
        let entry_status = entry_status
            .get_mut(entry_key)
            .ok_or(tonic::Status::internal("unable to retrieve status of key"))?;
        if let EntryStatus::Ready(in_use) = entry_status {
            *in_use -= 1;
            handles.remove(&handle_id);
        } else {
            return Err(tonic::Status::internal("inconsistent internal state"));
        }
        Ok(Response::new(()))
    }
}

fn get_file(root: impl AsRef<Path>, key: impl AsRef<EntryKey>) -> PathBuf {
    let root = root.as_ref();
    let key = key.as_ref();
    root.join(hex::encode(crate::hash(key.namespace.as_bytes())))
        .join(hex::encode(crate::hash(&key.key)))
}
