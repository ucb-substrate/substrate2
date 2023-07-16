//! A client for interacting with a cache server.

use std::{
    any::Any,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use backoff::ExponentialBackoff;
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use tokio::runtime::{Handle, Runtime};
use tonic::transport::{Channel, Endpoint};

use crate::{
    error::{ArcResult, Error, Result},
    rpc::{
        local::{self, local_cache_client},
        remote::{self, remote_cache_client},
    },
    CacheHandle, Cacheable,
};

/// The timeout for connecting to the cache server.
pub const CONNECTION_TIMEOUT_MS_DEFAULT: u64 = 1000;

/// The timeout for making a request to the cache server.
pub const REQUEST_TIMEOUT_MS_DEFAULT: u64 = 1000;

/// Configuration for a cache client.
#[derive(Debug)]
pub struct ClientConfig {
    url: String,
    poll_backoff: ExponentialBackoff,
    connection_timeout: Duration,
    request_timeout: Duration,
    handle: Handle,
    // Only used to own the runtime.
    #[allow(dead_code)]
    runtime: Option<Runtime>,
}

/// A builder for a cache client configuration.
#[derive(Default, Clone, Debug)]
pub struct ClientConfigBuilder {
    url: Option<String>,
    poll_backoff: Option<ExponentialBackoff>,
    connection_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    handle: Option<Handle>,
}

impl ClientConfig {
    /// Creates a [`ClientConfigBuilder`].
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }
}

impl ClientConfigBuilder {
    /// Creates a new [`ClientConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the configured server URL.
    pub fn url(&mut self, url: String) -> &mut Self {
        self.url = Some(url);
        self
    }

    /// Configures the exponential backoff used when polling the server for cache entry
    /// statuses.
    pub fn poll_backoff(&mut self, backoff: ExponentialBackoff) -> &mut Self {
        self.poll_backoff = Some(backoff);
        self
    }

    /// Sets the timeout for connecting to the server.
    pub fn connection_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// Sets the timeout for receiving a reply from the server.
    pub fn request_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Configures a [`Handle`] for making asynchronous gRPC requests.
    pub fn runtime_handle(&mut self, handle: Handle) -> &mut Self {
        self.handle = Some(handle);
        self
    }

    /// Builds a [`ClientConfig`] object with the configured parameters.
    pub fn build(&mut self) -> ClientConfig {
        let (handle, runtime) = match self.handle.clone() {
            Some(handle) => (handle, None),
            None => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                (runtime.handle().clone(), Some(runtime))
            }
        };
        ClientConfig {
            url: self.url.clone().expect("must specify server URL"),
            poll_backoff: self.poll_backoff.clone().unwrap_or_default(),
            connection_timeout: self
                .connection_timeout
                .unwrap_or(Duration::from_millis(CONNECTION_TIMEOUT_MS_DEFAULT)),
            request_timeout: self
                .request_timeout
                .unwrap_or(Duration::from_millis(REQUEST_TIMEOUT_MS_DEFAULT)),
            handle,
            runtime,
        }
    }
}

#[derive(Clone, Debug)]
struct Client {
    config: Arc<ClientConfig>,
}

impl Client {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    fn new(config: ClientConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Sets up the necessary objects to be passed in to [`Client::generate_inner`].
    fn setup_generate<K: Serialize, V>(
        namespace: impl Into<String>,
        key: &K,
    ) -> (String, Vec<u8>, CacheHandle<V>) {
        (
            namespace.into(),
            crate::hash(&flexbuffers::to_vec(key).unwrap()),
            CacheHandle(Arc::new(OnceCell::new())),
        )
    }

    /// Spawns a new thread to generate the desired value asynchronously.
    fn generate_inner<V: Send + Sync + Any>(
        self,
        handle: CacheHandle<V>,
        generate_loop: impl FnOnce() -> Result<()> + Send + Any,
    ) {
        thread::spawn(move || {
            if let Err(e) = generate_loop() {
                let _ = handle.0.set(Err(Arc::new(e)));
            }
        });
    }

    /// Deserializes a cached value into a [`Result`] that can be stored in a [`CacheHandle`].
    fn deserialize_cache_value<V: DeserializeOwned>(data: &[u8]) -> Result<V> {
        let data = flexbuffers::from_slice(data)?;
        Ok(data)
    }

    /// Deserializes a cached value into a containing result with the appropriate error type.
    fn deserialize_cache_result<V: DeserializeOwned, E>(
        data: &[u8],
    ) -> Result<std::result::Result<V, E>> {
        let data = flexbuffers::from_slice(data)?;
        Ok(Ok(data))
    }

    /// Runs the provided generation function in a new thread, returning the result.
    fn run_generation<K: Any + Send + Sync, V: Any + Send + Sync>(
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) -> ArcResult<V> {
        let join_handle = thread::spawn(move || generate_fn(&key));
        join_handle.join().map_err(|_| Arc::new(Error::Panic))
    }

    /// Starts sending heartbeats to the server in a new thread .
    ///
    /// Returns a sender for telling the spawned thread to stop sending heartbeats and
    /// a receiver for waiting for heartbeats to terminate.
    fn start_heartbeats(
        &self,
        heartbeat_interval: Duration,
        send_heartbeat: impl Fn() -> Result<()> + Send + Any,
    ) -> (Sender<()>, Receiver<()>) {
        let (s_heartbeat_stop, r_heartbeat_stop) = channel();
        let (s_heartbeat_stopped, r_heartbeat_stopped) = channel();
        thread::spawn(move || {
            loop {
                match r_heartbeat_stop.recv_timeout(heartbeat_interval) {
                    Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if send_heartbeat().is_err() {
                            break;
                        }
                    }
                }
            }
            let _ = s_heartbeat_stopped.send(());
        });
        (s_heartbeat_stop, r_heartbeat_stopped)
    }

    /// Sets the provided handle to deserialized data, panicking if unable to.
    fn set_handle<V>(handle: &CacheHandle<V>, data: V) {
        if handle.0.set(Ok(data)).is_err() {
            panic!("failed to set cell value");
        }
    }

    /// Connects to a remote cache gRPC server.
    async fn connect_remote(&self) -> Result<remote_cache_client::RemoteCacheClient<Channel>> {
        let endpoint = Endpoint::from_shared(self.config.url.clone())?
            .timeout(self.config.request_timeout)
            .connect_timeout(self.config.connection_timeout);
        Ok(remote_cache_client::RemoteCacheClient::connect(endpoint).await?)
    }

    /// Issues a `Get` RPC to a remote cache gRPC server.
    fn get_rpc_remote(
        &self,
        namespace: String,
        key: Vec<u8>,
        assign: bool,
    ) -> Result<remote::get_reply::EntryStatus> {
        let out: Result<remote::GetReply> = self.config.handle.block_on(async {
            let mut client = self.connect_remote().await?;
            Ok(client
                .get(remote::GetRequest {
                    namespace,
                    key,
                    assign,
                })
                .await?
                .into_inner())
        });
        Ok(out?.entry_status.unwrap())
    }

    /// Issues a `Heartbeat` RPC to a remote cache gRPC server.
    fn heartbeat_rpc_remote(&self, id: u64) -> Result<()> {
        self.config.handle.block_on(async {
            let mut client = self.connect_remote().await?;
            client.heartbeat(remote::HeartbeatRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Set` RPC to a remote cache gRPC server.
    fn set_rpc_remote(&self, id: u64, value: Vec<u8>) -> Result<()> {
        self.config.handle.block_on(async {
            let mut client = self.connect_remote().await?;
            client.set(remote::SetRequest { id, value }).await?;
            Ok(())
        })
    }

    /// Writes a generated value to a remote cache via the `Set` RPC.
    fn write_generated_value_remote<V: Serialize>(
        &self,
        id: u64,
        value: &ArcResult<V>,
    ) -> Result<()> {
        if let Ok(data) = value {
            self.set_rpc_remote(id, flexbuffers::to_vec(data).unwrap())?;
        }
        Ok(())
    }

    /// Writes data contained in a generated result to a remote cache via the `Set` RPC.
    ///
    /// Does not write to the cache if the generated result is an [`Err`].
    fn write_generated_result_remote<V: Serialize, E>(
        &self,
        id: u64,
        value: &ArcResult<std::result::Result<V, E>>,
    ) -> Result<()> {
        if let Ok(Ok(data)) = value {
            self.set_rpc_remote(id, flexbuffers::to_vec(data).unwrap())?;
        }
        Ok(())
    }

    /// Runs the generate loop for the remote cache protocol, checking whether the desired entry is
    /// loaded and generating it if needed.
    #[allow(clippy::too_many_arguments)]
    fn generate_loop_remote<K: Send + Sync + Any, V: Send + Sync + Any>(
        &self,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
        write_generated_value: impl FnOnce(&Self, u64, &ArcResult<V>) -> Result<()> + Send + Any,
        deserialize_cache_data: impl FnOnce(&[u8]) -> Result<V> + Send + Any,
        handle: CacheHandle<V>,
    ) -> Result<()> {
        let status = backoff::retry(self.config.poll_backoff.clone(), move || {
            let inner = || -> Result<(remote::get_reply::EntryStatus, bool)> {
                let status = self.get_rpc_remote(namespace.clone(), hash.clone(), true)?;
                let retry = matches!(status, remote::get_reply::EntryStatus::Loading(_));

                Ok((status, retry))
            };
            inner()
                .map_err(backoff::Error::Permanent)
                .and_then(|(status, retry)| {
                    if retry {
                        Err(backoff::Error::transient(Error::EntryLoading))
                    } else {
                        Ok(status)
                    }
                })
        })
        .map_err(Box::new)?;
        match status {
            remote::get_reply::EntryStatus::Unassigned(_) => {
                let v = Client::run_generation(key, generate_fn);
                let _ = handle.0.set(v);
            }
            remote::get_reply::EntryStatus::Assign(remote::AssignReply {
                id,
                heartbeat_interval_ms,
            }) => {
                let self_clone = self.clone();
                let (s_heartbeat_stop, r_heartbeat_stopped) = self.start_heartbeats(
                    Duration::from_millis(heartbeat_interval_ms),
                    move || -> Result<()> { self_clone.heartbeat_rpc_remote(id) },
                );
                let v = Client::run_generation(key, generate_fn);
                let _ = s_heartbeat_stop.send(());
                let _ = r_heartbeat_stopped.recv();
                write_generated_value(self, id, &v)?;
                let _ = handle.0.set(v);
            }
            remote::get_reply::EntryStatus::Loading(_) => unreachable!(),
            remote::get_reply::EntryStatus::Ready(data) => {
                Client::set_handle(&handle, deserialize_cache_data(&data)?);
            }
        }
        Ok(())
    }

    /// Connects to a local cache gRPC server.
    async fn connect_local(&self) -> Result<local_cache_client::LocalCacheClient<Channel>> {
        let endpoint = Endpoint::from_shared(self.config.url.clone())?
            .timeout(self.config.request_timeout)
            .connect_timeout(self.config.connection_timeout);
        let test = local_cache_client::LocalCacheClient::connect(endpoint).await;
        Ok(test?)
    }

    /// Issues a `Get` RPC to a local cache gRPC server.
    fn get_rpc_local(
        &self,
        namespace: String,
        key: Vec<u8>,
        assign: bool,
    ) -> Result<local::get_reply::EntryStatus> {
        let out: Result<local::GetReply> = self.config.handle.block_on(async {
            let mut client = self.connect_local().await?;
            Ok(client
                .get(local::GetRequest {
                    namespace,
                    key,
                    assign,
                })
                .await?
                .into_inner())
        });
        Ok(out?.entry_status.unwrap())
    }

    /// Issues a `Heartbeat` RPC to a local cache gRPC server.
    fn heartbeat_rpc_local(&self, id: u64) -> Result<()> {
        self.config.handle.block_on(async {
            let mut client = self.connect_local().await?;
            client.heartbeat(local::HeartbeatRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Done` RPC to a local cache gRPC server.
    fn done_rpc_local(&self, id: u64) -> Result<()> {
        self.config.handle.block_on(async {
            let mut client = self.connect_local().await?;
            client.done(local::DoneRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Drop` RPC to a local cache gRPC server.
    fn drop_rpc_local(&self, id: u64) -> Result<()> {
        self.config.handle.block_on(async {
            let mut client = self.connect_local().await?;
            client.drop(local::DropRequest { id }).await?;
            Ok(())
        })
    }

    fn write_generated_data_to_disk<V: Serialize>(
        &self,
        id: u64,
        path: String,
        data: &V,
    ) -> Result<()> {
        let path = PathBuf::from(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        f.write_all(&flexbuffers::to_vec(data).unwrap())?;
        self.done_rpc_local(id)?;

        Ok(())
    }

    /// Writes a generated value to a local cache via the `Set` RPC.
    fn write_generated_value_local<V: Serialize>(
        &self,
        id: u64,
        path: String,
        value: &ArcResult<V>,
    ) -> Result<()> {
        if let Ok(data) = value {
            self.write_generated_data_to_disk(id, path, data)?;
        }
        Ok(())
    }

    /// Writes data contained in a generated result to a local cache via the `Set` RPC.
    ///
    /// Does not write to the cache if the generated result is an [`Err`].
    fn write_generated_result_local<V: Serialize, E>(
        &self,
        id: u64,
        path: String,
        value: &ArcResult<std::result::Result<V, E>>,
    ) -> Result<()> {
        if let Ok(Ok(data)) = value {
            self.write_generated_data_to_disk(id, path, data)?;
        }
        Ok(())
    }

    /// Runs the generate loop for the local cache protocol, checking whether the desired entry is
    /// loaded and generating it if needed.
    #[allow(clippy::too_many_arguments)]
    fn generate_loop_local<K: Send + Sync + Any, V: Send + Sync + Any>(
        &self,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
        write_generated_value: impl FnOnce(&Self, u64, String, &ArcResult<V>) -> Result<()> + Send + Any,
        deserialize_cache_data: impl FnOnce(&[u8]) -> Result<V> + Send + Any,
        handle: CacheHandle<V>,
    ) -> Result<()> {
        let status = backoff::retry(self.config.poll_backoff.clone(), move || {
            let inner = || -> Result<(local::get_reply::EntryStatus, bool)> {
                let status = self.get_rpc_local(namespace.clone(), hash.clone(), true)?;
                let retry = matches!(status, local::get_reply::EntryStatus::Loading(_));

                Ok((status, retry))
            };
            inner()
                .map_err(backoff::Error::Permanent)
                .and_then(|(status, retry)| {
                    if retry {
                        Err(backoff::Error::transient(Error::EntryLoading))
                    } else {
                        Ok(status)
                    }
                })
        })
        .map_err(Box::new)?;
        match status {
            local::get_reply::EntryStatus::Unassigned(_) => {
                let v = Client::run_generation(key, generate_fn);
                let _ = handle.0.set(v);
            }
            local::get_reply::EntryStatus::Assign(local::AssignReply {
                id,
                path,
                heartbeat_interval_ms,
            }) => {
                let self_clone = self.clone();
                let (s_heartbeat_stop, r_heartbeat_stopped) = self.start_heartbeats(
                    Duration::from_millis(heartbeat_interval_ms),
                    move || -> Result<()> { self_clone.heartbeat_rpc_local(id) },
                );
                let v = Client::run_generation(key, generate_fn);
                let _ = s_heartbeat_stop.send(());
                let _ = r_heartbeat_stopped.recv();
                write_generated_value(self, id, path, &v)?;
                let _ = handle.0.set(v);
            }
            local::get_reply::EntryStatus::Loading(_) => unreachable!(),
            local::get_reply::EntryStatus::Ready(local::ReadyReply { id, path }) => {
                let mut file = std::fs::File::open(path)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                self.drop_rpc_local(id)?;
                Client::set_handle(&handle, deserialize_cache_data(&buf)?);
            }
        }
        Ok(())
    }
}

/// A gRPC cache client that does not have a shared filesystem with the cache server.
#[derive(Clone, Debug)]
pub struct RemoteClient {
    inner: Client,
}

impl RemoteClient {
    /// Creates a new gRPC cache client, querying the remote cache server for server configuration.
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    pub fn new(config: ClientConfig) -> Self {
        Self {
            inner: Client::new(config),
        }
    }

    fn generate_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        self,
        handle: CacheHandle<V>,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) {
        self.inner.clone().generate_inner(handle.clone(), move || {
            self.inner.generate_loop_remote(
                namespace,
                hash,
                key,
                generate_fn,
                Client::write_generated_value_remote,
                Client::deserialize_cache_value,
                handle,
            )
        });
    }

    fn generate_result_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        self,
        handle: CacheHandle<std::result::Result<V, E>>,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> std::result::Result<V, E> + Send + Any,
    ) {
        self.inner.clone().generate_inner(handle.clone(), move || {
            self.inner.generate_loop_remote(
                namespace,
                hash,
                key,
                generate_fn,
                Client::write_generated_result_remote,
                Client::deserialize_cache_result,
                handle,
            )
        });
    }

    /// Gets a handle to a cacheable object from the cache.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`LocalClient::get_with_err`].
    pub fn get<K: Cacheable>(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_result(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    pub fn get_with_err<
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: Cacheable<Error = E>,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate(namespace, key, |key| key.generate())
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    pub fn generate<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) -> CacheHandle<V> {
        let (namespace, hash, handle) = Client::setup_generate(namespace, &key);

        self.clone()
            .generate_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`RemoteClient::generate`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    pub fn generate_result<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> std::result::Result<V, E> + Send + Any,
    ) -> CacheHandle<std::result::Result<V, E>> {
        let (namespace, hash, handle) = Client::setup_generate(namespace, &key);

        self.clone()
            .generate_result_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }
}

/// A gRPC cache client that has a shared filesystem with the cache server.
#[derive(Clone, Debug)]
pub struct LocalClient {
    inner: Client,
}

impl LocalClient {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    pub fn new(config: ClientConfig) -> Self {
        Self {
            inner: Client::new(config),
        }
    }

    fn generate_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        self,
        handle: CacheHandle<V>,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) {
        self.inner.clone().generate_inner(handle.clone(), move || {
            self.inner.generate_loop_local(
                namespace,
                hash,
                key,
                generate_fn,
                Client::write_generated_value_local,
                Client::deserialize_cache_value,
                handle,
            )
        });
    }

    fn generate_result_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        self,
        handle: CacheHandle<std::result::Result<V, E>>,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> std::result::Result<V, E> + Send + Any,
    ) {
        self.inner.clone().generate_inner(handle.clone(), move || {
            self.inner.generate_loop_local(
                namespace,
                hash,
                key,
                generate_fn,
                Client::write_generated_result_local,
                Client::deserialize_cache_result,
                handle,
            )
        });
    }

    /// Gets a handle to a cacheable object from the cache.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`LocalClient::get_with_err`].
    pub fn get<K: Cacheable>(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_result(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    pub fn get_with_err<
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: Cacheable<Error = E>,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate(namespace, key, |key| key.generate())
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    pub fn generate<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) -> CacheHandle<V> {
        let (namespace, hash, handle) = Client::setup_generate(namespace, &key);

        self.clone()
            .generate_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`LocalClient::generate`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    pub fn generate_result<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> std::result::Result<V, E> + Send + Any,
    ) -> CacheHandle<std::result::Result<V, E>> {
        let (namespace, hash, handle) = Client::setup_generate(namespace, &key);

        self.clone()
            .generate_result_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }
}
