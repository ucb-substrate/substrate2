//! A persistent cache gRPC server.

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
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_rusqlite::Connection;
use tonic::Response;

use crate::error::Result;
use crate::rpc::local::{
    self,
    local_cache_server::{LocalCache, LocalCacheServer},
};
use crate::rpc::remote::{
    self,
    remote_cache_server::{RemoteCache, RemoteCacheServer},
};
use crate::Namespace;

/// The name of the config manifest TOML file.
pub const CONFIG_MANIFEST_NAME: &str = "Cache.toml";

/// The name of the main manifest database.
pub const MANIFEST_DB_NAME: &str = "cache.sqlite";

/// The expected interval between heartbeats.
pub const HEARTBEAT_INTERVAL_SECS_DEFAULT: u64 = 2;

/// The timeout before an assigned task is assumed to have failed.
pub const HEARTBEAT_TIMEOUT_SECS_DEFAULT: u64 = HEARTBEAT_INTERVAL_SECS_DEFAULT + 2;

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
#[derive(Debug)]
pub struct Server {
    root: Arc<PathBuf>,
    local: Option<TcpListener>,
    remote: Option<TcpListener>,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
}

/// A builder for a gRPC cache server.
#[derive(Default, Debug)]
pub struct ServerBuilder {
    root: Option<Arc<PathBuf>>,
    local: Option<TcpListener>,
    remote: Option<TcpListener>,
    heartbeat_interval: Option<Duration>,
    heartbeat_timeout: Option<Duration>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub(crate) struct ConfigManifest {
    pub(crate) local_addr: Option<SocketAddr>,
    pub(crate) remote_addr: Option<SocketAddr>,
    pub(crate) heartbeat_interval: Duration,
    pub(crate) heartbeat_timeout: Duration,
}

impl ServerBuilder {
    /// Creates a new [`ServerBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the root directory of the cache server.
    pub fn root(mut self, path: PathBuf) -> Self {
        self.root = Some(Arc::new(path));
        self
    }

    /// Configures the local cache gRPC server.
    ///
    /// Returns an error if the provided address cannot be bound.
    pub async fn local(mut self, addr: SocketAddr) -> std::io::Result<Self> {
        self.local = Some(TcpListener::bind(addr).await?);
        Ok(self)
    }

    /// Configures the remote cache gRPC server.
    ///
    /// Returns an error if the provided address cannot be bound.
    pub async fn remote(mut self, addr: SocketAddr) -> std::io::Result<Self> {
        self.remote = Some(TcpListener::bind(addr).await?);
        Ok(self)
    }

    /// Configures the local cache gRPC server to use the provided [`TcpListener`].
    pub fn local_with_incoming(mut self, incoming: TcpListener) -> Self {
        self.local = Some(incoming);
        self
    }

    /// Configures the remote cache gRPC server to use the provided [`TcpListener`].
    pub fn remote_with_incoming(mut self, incoming: TcpListener) -> Self {
        self.remote = Some(incoming);
        self
    }

    /// Sets the expected interval between hearbeats.
    ///
    /// Defaults to [`HEARTBEAT_INTERVAL_SECS_DEFAULT`].
    pub fn heartbeat_interval(mut self, duration: Duration) -> Self {
        self.heartbeat_interval = Some(duration);
        self
    }

    /// Sets the timeout before an assigned task is marked for reassignment.
    ///
    /// Defaults to [`HEARTBEAT_TIMEOUT_SECS_DEFAULT`].
    pub fn heartbeat_timeout(mut self, duration: Duration) -> Self {
        self.heartbeat_timeout = Some(duration);
        self
    }

    /// Builds a [`Server`] from the configured options.
    pub fn build(self) -> Server {
        let server = Server {
            root: self.root.clone().unwrap(),
            local: self.local,
            remote: self.remote,
            heartbeat_interval: self
                .heartbeat_interval
                .unwrap_or(Duration::from_secs(HEARTBEAT_INTERVAL_SECS_DEFAULT)),
            heartbeat_timeout: self
                .heartbeat_timeout
                .unwrap_or(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS_DEFAULT)),
        };

        assert!(
            server.heartbeat_interval < server.heartbeat_timeout,
            "heartbeat interval must be less than the heartbeat interval"
        );

        assert_eq!(
            server.heartbeat_interval.subsec_micros() % 1000,
            0,
            "heartbeat interval cannot have finer than millisecond resolution"
        );

        server
    }
}

impl Server {
    /// Creates a new [`ServerBuilder`] object.
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    /// Starts the gRPC server, listening on the configured address.
    pub async fn start(self) -> Result<()> {
        if let (None, None) = (&self.local, &self.remote) {
            tracing::warn!("no local or remote listener specified so no server is being run");
            return Ok(());
        }

        // Write configuration options to the config manifest.
        let mut config_manifest = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.root.join(CONFIG_MANIFEST_NAME))
            .await?;
        config_manifest.try_lock_exclusive()?;
        config_manifest
            .write_all(
                &toml::to_string(&ConfigManifest {
                    local_addr: self
                        .local
                        .as_ref()
                        .map(|value| value.local_addr())
                        .map_or(Ok(None), |v| v.map(Some))?,
                    remote_addr: self
                        .remote
                        .as_ref()
                        .map(|value| value.local_addr())
                        .map_or(Ok(None), |v| v.map(Some))?,
                    heartbeat_interval: self.heartbeat_interval,
                    heartbeat_timeout: self.heartbeat_timeout,
                })
                .unwrap()
                .into_bytes(),
            )
            .await?;

        let db_path = self.root.join(MANIFEST_DB_NAME);
        let inner = Arc::new(Mutex::new(CacheInner::new(&db_path).await?));

        let imp = CacheImpl::new(
            self.root.clone(),
            self.heartbeat_interval,
            self.heartbeat_timeout,
            inner,
        );

        let Server { local, remote, .. } = self;

        let local_handle = if let Some(local) = local {
            tracing::debug!("local server listening on address {}", local.local_addr()?);
            let local_svc = LocalCacheServer::new(imp.clone());
            Some(tokio::spawn(
                tonic::transport::Server::builder()
                    .add_service(local_svc)
                    .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(local)),
            ))
        } else {
            None
        };
        let remote_handle = if let Some(remote) = remote {
            tracing::debug!(
                "remote server listening on address {}",
                remote.local_addr()?
            );
            let remote_svc = RemoteCacheServer::new(imp);
            Some(tokio::spawn(
                tonic::transport::Server::builder()
                    .add_service(remote_svc)
                    .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(remote)),
            ))
        } else {
            None
        };

        if let Some(local_handle) = local_handle {
            local_handle.await??;
        }

        if let Some(remote_handle) = remote_handle {
            remote_handle.await??;
        }

        // Hold file lock until server terminates.
        drop(config_manifest);

        Ok(())
    }
}

/// Cache state.
#[derive(Clone, Debug)]
struct CacheInner {
    next_assignment_id: AssignmentId,
    next_handle_id: HandleId,
    /// Status of entries currently in the cache.
    entry_status: HashMap<Arc<EntryKey>, EntryStatus>,
    /// Status of entries that are currently loading.
    loading: HashMap<AssignmentId, LoadingData>,
    /// Status of entries that have active handles.
    handles: HashMap<HandleId, Arc<EntryKey>>,
    /// A wrapper around a [`tokio_rusqlite::Connection`].
    conn: CacheInnerConn,
}

impl CacheInner {
    async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        tracing::debug!("connecting to manifest database");
        // Set up the manifest database.
        let conn = Connection::open(db_path.as_ref()).await?;
        conn.call(|conn| {
            let tx = conn.transaction()?;
            tx.execute(CREATE_MANIFEST_TABLE_STMT, ())?;
            tx.commit()?;
            tracing::debug!("ensured that manifest table has been created");
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

        // Load persisted state.
        cache.load_from_disk().await?;

        Ok(cache)
    }

    async fn load_from_disk(&mut self) -> Result<()> {
        tracing::debug!("loading cache state from disk");
        let rows = self
            .conn
            .0
            .call(|conn| {
                let tx = conn.transaction()?;

                // Delete loading entries as we cannot recover assignment IDs on restart.
                tracing::debug!("deleting loading entries from database");
                let mut stmt = tx.prepare(DELETE_ENTRIES_WITH_STATUS_STMT)?;
                stmt.execute([DbEntryStatus::Loading.to_int()])?;
                drop(stmt);

                // Read remaining rows from the manifest, converting them into tuples mapping
                // `EntryKey` to a `DbEntryStatus`.
                tracing::debug!("reading remaining entries from database");
                let mut stmt = tx.prepare(READ_MANIFEST_STMT)?;
                let rows = stmt.query_map(
                    [],
                    |row| -> rusqlite::Result<(Arc<EntryKey>, DbEntryStatus)> {
                        Ok((
                            Arc::new(EntryKey {
                                namespace: Namespace::new(row.get::<_, String>(0)?),
                                key: row.get(1)?,
                            }),
                            DbEntryStatus::from_int(row.get(2)?).unwrap(),
                        ))
                    },
                )?;
                let res = Ok(rows.collect::<Vec<_>>());
                drop(stmt);

                tx.commit()?;
                res
            })
            .await?
            .into_iter()
            .map(|res| res.map_err(|e| e.into()))
            .collect::<std::result::Result<Vec<_>, tokio_rusqlite::Error>>()?;

        // Map database entries into in-memory cache state.
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
                stmt.execute((
                    key.namespace.clone().into_inner(),
                    key.key.clone(),
                    status.to_int(),
                ))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn update_status(&self, key: Arc<EntryKey>, status: DbEntryStatus) -> Result<()> {
        self.0
            .call(move |conn| {
                let mut stmt = conn.prepare(UPDATE_STATUS_STMT)?;
                stmt.execute((
                    status.to_int(),
                    key.namespace.clone().into_inner(),
                    key.key.clone(),
                ))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn delete_status(&self, key: Arc<EntryKey>) -> Result<()> {
        self.0
            .call(move |conn| {
                let mut stmt = conn.prepare(DELETE_STATUS_STMT)?;
                stmt.execute((key.namespace.clone().into_inner(), key.key.clone()))?;
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
    namespace: Namespace,
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
    /// An entry that is marked for eviction.
    ///
    /// Currently unused.
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
    Assign(AssignmentId, Duration),
    Loading,
    ReadyRemote(Vec<u8>),
    ReadyLocal(HandleId),
}

impl GetReplyStatus {
    fn into_local(self, path: String) -> local::get_reply::EntryStatus {
        match self {
            Self::Unassigned => local::get_reply::EntryStatus::Unassigned(()),
            Self::Assign(id, heartbeat_interval) => {
                local::get_reply::EntryStatus::Assign(local::AssignReply {
                    id: id.0,
                    path,
                    heartbeat_interval_ms: heartbeat_interval.as_millis() as u64,
                })
            }
            Self::Loading => local::get_reply::EntryStatus::Loading(()),
            Self::ReadyLocal(id) => {
                local::get_reply::EntryStatus::Ready(local::ReadyReply { id: id.0, path })
            }
            Self::ReadyRemote(_) => panic!("cannot convert remote statuses to local statuses"),
        }
    }
    fn into_remote(self) -> remote::get_reply::EntryStatus {
        match self {
            Self::Unassigned => remote::get_reply::EntryStatus::Unassigned(()),
            Self::Assign(id, heartbeat_interval) => {
                remote::get_reply::EntryStatus::Assign(remote::AssignReply {
                    id: id.0,
                    heartbeat_interval_ms: heartbeat_interval.as_millis() as u64,
                })
            }
            Self::Loading => remote::get_reply::EntryStatus::Loading(()),
            Self::ReadyRemote(val) => remote::get_reply::EntryStatus::Ready(val),
            Self::ReadyLocal(_) => panic!("cannot convert local statuses to remote statuses"),
        }
    }
}

#[derive(Clone, Debug)]
struct CacheImpl {
    root: Arc<PathBuf>,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
    inner: Arc<Mutex<CacheInner>>,
}

impl CacheImpl {
    fn new(
        root: Arc<PathBuf>,
        heartbeat_interval: Duration,
        heartbeat_timeout: Duration,
        inner: Arc<Mutex<CacheInner>>,
    ) -> Self {
        Self {
            root,
            heartbeat_interval,
            heartbeat_timeout,
            inner,
        }
    }

    /// Responds to a `Get` RPC request for the given entry key, assigning unassigned tasks if
    /// `assign` is `true`.
    ///
    /// If `local` is `true`, getting an existing key in the cache requires assigning a new entry
    /// handle, which must be dropped by the client to allow the key to be evicted.
    async fn get_impl(
        &self,
        entry_key: Arc<EntryKey>,
        assign: bool,
        local: bool,
    ) -> std::result::Result<GetReplyStatus, tonic::Status> {
        tracing::debug!("received get request");
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

                        // If the entry is loading but hasn't received a heartbeat recently,
                        // reassign it to be loaded by the new requester.
                        //
                        // Otherwise, notify the requester that the entry is currently loading.
                        if Instant::now().duration_since(data.last_heartbeat)
                            > self.heartbeat_timeout
                        {
                            tracing::debug!("assigned worker has not sent a heartbeat recently, entry is no longer loading");
                            if assign {
                                loading.remove(id);
                                next_assignment_id.increment();
                                *id = *next_assignment_id;
                                tracing::debug!("assigning task with id {:?}", id);
                                loading.insert(
                                    *id,
                                    LoadingData {
                                        last_heartbeat: Instant::now(),
                                        key: entry_key,
                                    },
                                );
                                GetReplyStatus::Assign(*id, self.heartbeat_interval)
                            } else {
                                conn.delete_status(entry_key.clone()).await.map_err(|_| {
                                    tonic::Status::internal("unable to persist changes")
                                })?;
                                o.remove_entry();
                                GetReplyStatus::Unassigned
                            }
                        } else {
                            tracing::debug!("entry is currently loading");
                            GetReplyStatus::Loading
                        }
                    }
                    EntryStatus::Ready(in_use) => {
                        tracing::debug!("entry is ready, sending relevant data to client");
                        if local {
                            // If the requested entry is ready, assign a new handle to the entry.
                            *in_use += 1;
                            next_handle_id.increment();
                            handles.insert(*next_handle_id, entry_key);
                            GetReplyStatus::ReadyLocal(*next_handle_id)
                        } else {
                            // If the requested entry is ready, read it from disk and send it back
                            // to the requester.
                            let mut file = File::open(path).await?;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf).await?;
                            GetReplyStatus::ReadyRemote(buf)
                        }
                    }
                    // If the entry is currently being evicted, do not assign it.
                    //
                    // The client is free to generate on their own, but the cache will not accept a
                    // new value for the entry.
                    EntryStatus::Evicting => {
                        tracing::debug!("entry is currently being evicted");
                        GetReplyStatus::Unassigned
                    }
                }
            }
            Entry::Vacant(v) => {
                // If the entry doesn't exist, assign it to be loaded if needed.
                tracing::debug!("entry does not exist, creating a new entry");
                if assign {
                    next_assignment_id.increment();
                    conn.insert_status(entry_key.clone(), DbEntryStatus::Loading)
                        .await
                        .map_err(|_| tonic::Status::internal("unable to persist changes"))?;
                    v.insert(EntryStatus::Loading(*next_assignment_id));
                    tracing::debug!("assigning task with id {:?}", next_assignment_id);
                    loading.insert(
                        *next_assignment_id,
                        LoadingData {
                            last_heartbeat: Instant::now(),
                            key: entry_key,
                        },
                    );
                    GetReplyStatus::Assign(*next_assignment_id, self.heartbeat_interval)
                } else {
                    GetReplyStatus::Unassigned
                }
            }
        })
    }

    async fn heartbeat_impl(&self, id: AssignmentId) -> std::result::Result<(), tonic::Status> {
        tracing::debug!("received heartbeat request for id {:?}", id);
        let mut inner = self.inner.lock().await;
        match inner.loading.entry(id) {
            Entry::Vacant(_) => {
                tracing::error!(
                    "received heartbeat request for invalid assignment id {:?}",
                    id
                );
                return Err(tonic::Status::invalid_argument("invalid assignment id"));
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
        tracing::debug!("received set request for id {:?}", id);
        let mut inner = self.inner.lock().await;
        let data = inner.loading.get(&id).ok_or_else(|| {
            tracing::error!("received set request for invalid id {:?}", id);
            tonic::Status::invalid_argument("invalid assignment id")
        })?;

        let key = data.key.clone();

        // If there is a value to write to disk, write it to the appropriate file.
        if let Some(value) = value {
            let path = get_file(self.root.as_ref(), &key);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await?;
            f.write_all(&value).await?;
        }

        // Mark the entry as ready in the database and in memory.
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
impl LocalCache for CacheImpl {
    async fn get(
        &self,
        request: tonic::Request<local::GetRequest>,
    ) -> std::result::Result<tonic::Response<local::GetReply>, tonic::Status> {
        let request = request.into_inner();

        if !Namespace::validate(&request.namespace) {
            return Err(tonic::Status::invalid_argument("invalid namespace"));
        }

        let entry_key = Arc::new(EntryKey {
            namespace: Namespace::new(request.namespace),
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

    // TODO: Untested since eviction is not yet implemented.
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

#[tonic::async_trait]
impl RemoteCache for CacheImpl {
    async fn get(
        &self,
        request: tonic::Request<remote::GetRequest>,
    ) -> std::result::Result<tonic::Response<remote::GetReply>, tonic::Status> {
        let request = request.into_inner();

        if !Namespace::validate(&request.namespace) {
            return Err(tonic::Status::invalid_argument("invalid namespace"));
        }

        let entry_key = Arc::new(EntryKey {
            namespace: Namespace::new(request.namespace),
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

fn get_file(root: impl AsRef<Path>, key: impl AsRef<EntryKey>) -> PathBuf {
    let root = root.as_ref();
    let key = key.as_ref();
    // TODO: Require namespace to be filesystem compatible so that cache folder names don't need to
    // be hashed.
    root.join(key.namespace.as_ref())
        .join(hex::encode(crate::hash(&key.key)))
}
