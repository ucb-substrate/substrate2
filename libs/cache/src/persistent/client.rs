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

use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};
use tonic::transport::{Channel, Endpoint};

use crate::{
    error::{Error, Result},
    persistent::server::{ConfigManifest, CONFIG_MANIFEST_NAME, HEARTBEAT_INTERVAL_SECS},
    rpc::{
        local::{self, local_cache_client},
        remote::{self, remote_cache_client},
    },
    CacheHandle, Cacheable,
};

/// The interval between polling the cache server on whether a value has finished loading.
pub const POLL_INTERVAL_MS: u64 = 100;

/// The timeout for connecting to the cache server.
pub const CONNECTION_TIMEOUT_MS: u64 = 1000;

/// The timeout for making a request to the cache server.
pub const REQUEST_TIMEOUT_MS: u64 = 1000;

#[derive(Clone, Debug)]
struct CacheClient {
    root: Arc<PathBuf>,
    runtime: Arc<Runtime>,
}

impl CacheClient {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    fn new(root: PathBuf) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        Self {
            root: Arc::new(root),
            runtime: Arc::new(runtime),
        }
    }

    async fn read_config_manifest(&self) -> Result<ConfigManifest> {
        let mut buf = String::new();
        File::open(self.root.join(CONFIG_MANIFEST_NAME))
            .await?
            .read_to_string(&mut buf)
            .await?;
        Ok(toml::from_str(&buf)?)
    }

    fn run_generation<K: Any + Send + Sync, V: Any + Send + Sync>(
        handle: CacheHandle<V>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) {
        let handle2 = handle.clone();
        let join_handle = thread::spawn(move || {
            let value = generate_fn(&key);
            if handle2.0.set(Ok(value)).is_err() {
                panic!("failed to set cell value");
            }
        });
        if join_handle.join().is_err() && handle.0.set(Err(Arc::new(Error::Panic))).is_err() {
            panic!("failed to set cell value on panic");
        }
    }

    fn start_heartbeats(
        send_heartbeat: impl Fn() -> Result<()> + Send + Any,
    ) -> (Sender<()>, Receiver<()>) {
        let (s_heartbeat_stop, r_heartbeat_stop) = channel();
        let (s_heartbeat_stopped, r_heartbeat_stopped) = channel();
        thread::spawn(move || {
            loop {
                match r_heartbeat_stop.recv_timeout(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)) {
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

    fn set_handle<V: DeserializeOwned>(handle: CacheHandle<V>, data: &[u8]) -> Result<()> {
        if handle
            .0
            .set(Ok(flexbuffers::from_slice::<V>(data)?))
            .is_err()
        {
            panic!("failed to set cell value");
        }
        Ok(())
    }
}

/// A gRPC cache client that does not have a shared filesystem with the cache server.
#[derive(Clone, Debug)]
pub struct RemoteCacheClient {
    inner: CacheClient,
}

impl RemoteCacheClient {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    pub fn new(root: PathBuf) -> Self {
        Self {
            inner: CacheClient::new(root),
        }
    }

    async fn connect(&self) -> Result<remote_cache_client::RemoteCacheClient<Channel>> {
        let manifest = self.inner.read_config_manifest().await?;
        let endpoint = Endpoint::from_shared(format!(
            "http://{}",
            manifest.remote_addr.ok_or(Error::Connection)?
        ))?
        .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
        .connect_timeout(Duration::from_millis(CONNECTION_TIMEOUT_MS));
        let test = remote_cache_client::RemoteCacheClient::connect(endpoint).await;
        Ok(test?)
    }

    fn get_rpc(&self, namespace: String, key: Vec<u8>) -> Result<remote::get_reply::EntryStatus> {
        let out = self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            let out: Result<remote::GetReply> = Ok(client
                .get(remote::GetRequest {
                    namespace,
                    key,
                    assign: true,
                })
                .await?
                .into_inner());
            out
        })?;
        Ok(out.entry_status.unwrap())
    }

    fn heartbeat_rpc(&self, id: u64) -> Result<()> {
        self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.heartbeat(remote::HeartbeatRequest { id }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
    }

    fn set_rpc(&self, id: u64, value: Vec<u8>) -> Result<()> {
        self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.set(remote::SetRequest { id, value }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
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
        thread::spawn(move || {
            let inner = || -> Result<()> {
                loop {
                    let status = self.get_rpc(namespace.clone(), hash.clone())?;
                    match status {
                        remote::get_reply::EntryStatus::Unassigned(_) => {
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            break;
                        }
                        remote::get_reply::EntryStatus::Assign(id) => {
                            let self_clone = self.clone();
                            let (s_heartbeat_stop, r_heartbeat_stopped) =
                                CacheClient::start_heartbeats(move || -> Result<()> {
                                    self_clone.heartbeat_rpc(id)
                                });
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            if let Ok(data) = handle.try_get() {
                                let _ = s_heartbeat_stop.send(());
                                let _ = r_heartbeat_stopped.recv();
                                self.set_rpc(id, flexbuffers::to_vec(data).unwrap())?;
                            }
                            break;
                        }
                        remote::get_reply::EntryStatus::Loading(_) => {
                            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                        }
                        remote::get_reply::EntryStatus::Ready(data) => {
                            CacheClient::set_handle(handle.clone(), &data)?;
                            break;
                        }
                    }
                }
                Ok(())
            };
            if let Err(e) = inner() {
                let _ = handle.0.set(Err(Arc::new(e)));
            }
        });
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
        let namespace = namespace.into();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());
        let handle = CacheHandle(Arc::new(OnceCell::new()));

        self.clone()
            .generate_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }
}

/// A gRPC cache client that has a shared filesystem with the cache server.
#[derive(Clone, Debug)]
pub struct LocalCacheClient {
    inner: CacheClient,
}

impl LocalCacheClient {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    pub fn new(root: PathBuf) -> Self {
        Self {
            inner: CacheClient::new(root),
        }
    }

    async fn connect(&self) -> Result<local_cache_client::LocalCacheClient<Channel>> {
        let manifest = self.inner.read_config_manifest().await?;
        let endpoint = Endpoint::from_shared(format!(
            "http://{}",
            manifest.local_addr.ok_or(Error::Connection)?
        ))?
        .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
        .connect_timeout(Duration::from_millis(CONNECTION_TIMEOUT_MS));
        let test = local_cache_client::LocalCacheClient::connect(endpoint).await;
        Ok(test?)
    }

    fn get_rpc(&self, namespace: String, key: Vec<u8>) -> Result<local::get_reply::EntryStatus> {
        let out = self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            let out: Result<local::GetReply> = Ok(client
                .get(local::GetRequest {
                    namespace,
                    key,
                    assign: true,
                })
                .await?
                .into_inner());
            out
        })?;
        Ok(out.entry_status.unwrap())
    }

    fn heartbeat_rpc(&self, id: u64) -> Result<()> {
        self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.heartbeat(local::HeartbeatRequest { id }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
    }

    fn done_rpc(&self, id: u64) -> Result<()> {
        self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.done(local::DoneRequest { id }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
    }

    fn drop_rpc(&self, id: u64) -> Result<()> {
        self.inner.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.drop(local::DropRequest { id }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
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
        thread::spawn(move || {
            let inner = || -> Result<()> {
                loop {
                    let status = self.get_rpc(namespace.clone(), hash.clone())?;
                    match status {
                        local::get_reply::EntryStatus::Unassigned(_) => {
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            break;
                        }
                        local::get_reply::EntryStatus::Assign(local::IdPath { id, path }) => {
                            let self_clone = self.clone();
                            let (s_heartbeat_stop, r_heartbeat_stopped) =
                                CacheClient::start_heartbeats(move || -> Result<()> {
                                    self_clone.heartbeat_rpc(id)
                                });
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            if let Ok(data) = handle.try_get() {
                                let _ = s_heartbeat_stop.send(());
                                let _ = r_heartbeat_stopped.recv();
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
                                self.done_rpc(id)?;
                            }
                            break;
                        }
                        local::get_reply::EntryStatus::Loading(_) => {
                            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                        }
                        local::get_reply::EntryStatus::Ready(local::IdPath { id, path }) => {
                            let mut file = std::fs::File::open(path)?;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf)?;
                            CacheClient::set_handle(handle.clone(), &buf)?;
                            self.drop_rpc(id)?;
                            break;
                        }
                    }
                }
                Ok(())
            };
            if let Err(e) = inner() {
                let _ = handle.0.set(Err(Arc::new(e)));
            }
        });
    }

    fn generate_inner_result<
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
        thread::spawn(move || {
            let inner = || -> Result<()> {
                loop {
                    let status = self.get_rpc(namespace.clone(), hash.clone())?;
                    match status {
                        local::get_reply::EntryStatus::Unassigned(_) => {
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            break;
                        }
                        local::get_reply::EntryStatus::Assign(local::IdPath { id, path }) => {
                            let self_clone = self.clone();
                            let (s_heartbeat_stop, r_heartbeat_stopped) =
                                CacheClient::start_heartbeats(move || -> Result<()> {
                                    self_clone.heartbeat_rpc(id)
                                });
                            CacheClient::run_generation(handle.clone(), key, generate_fn);
                            if let Ok(data) = handle.try_inner() {
                                let _ = s_heartbeat_stop.send(());
                                let _ = r_heartbeat_stopped.recv();
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
                                self.done_rpc(id)?;
                            }
                            break;
                        }
                        local::get_reply::EntryStatus::Loading(_) => {
                            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                        }
                        local::get_reply::EntryStatus::Ready(local::IdPath { id, path }) => {
                            let mut file = std::fs::File::open(path)?;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf)?;
                            if handle
                                .0
                                .set(Ok(Ok(flexbuffers::from_slice::<V>(&buf)?)))
                                .is_err()
                            {
                                panic!("failed to set cell value");
                            }
                            self.drop_rpc(id)?;
                            break;
                        }
                    }
                }
                Ok(())
            };
            if let Err(e) = inner() {
                let _ = handle.0.set(Err(Arc::new(e)));
            }
        });
    }

    /// Gets a handle to a cacheable object from the cache.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`LocalCacheClient::get_with_err`].
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
        let namespace = namespace.into();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());
        let handle = CacheHandle(Arc::new(OnceCell::new()));

        self.clone()
            .generate_inner(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`LocalCacheClient::generate`].
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
        let namespace = namespace.into();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());
        let handle = CacheHandle(Arc::new(OnceCell::new()));

        self.clone()
            .generate_inner_result(handle.clone(), namespace, hash, key, generate_fn);

        handle
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        error::Result,
        persistent::{
            client::LocalCacheClient,
            server::{CacheServer, HEARTBEAT_TIMEOUT_SECS},
        },
    };

    use super::RemoteCacheClient;

    const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

    #[test]
    fn remote_client_retrieves_persistently_cached_values() -> Result<()> {
        let root =
            PathBuf::from(BUILD_DIR).join("remote_client_retrieves_persistently_cached_values");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let server = CacheServer::new(root.clone()).with_remote("0.0.0.0:28055".parse().unwrap());
        let handle = runtime.spawn(async move { server.start().await });

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let client = RemoteCacheClient::new(root.clone());
        let count = Arc::new(Mutex::new(0));
        let count1 = count.clone();
        let count2 = count.clone();
        let handle1 = client.generate("test", (1, 2, 3), move |k| {
            *count1.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });
        let handle2 = client.generate("test", (1, 2, 3), move |k| {
            *count2.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });

        assert_eq!(handle1.get(), handle2.get());

        handle.abort();

        let server = CacheServer::new(root).with_remote("0.0.0.0:28056".parse().unwrap());
        drop(runtime.spawn(async move { server.start().await }));

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let count2 = count.clone();
        let handle2 = client.generate("test", (1, 2, 3), move |k| {
            *count2.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });

        assert_eq!(handle1.get(), handle2.get());
        assert_eq!(*count.lock().unwrap(), 1);

        Ok(())
    }

    #[test]
    fn local_client_retrieves_persistently_cached_values() -> Result<()> {
        let root =
            PathBuf::from(BUILD_DIR).join("local_client_retrieves_persistently_cached_values");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let server = CacheServer::new(root.clone()).with_local("0.0.0.0:28057".parse().unwrap());
        let handle = runtime.spawn(async move { server.start().await });

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let client = LocalCacheClient::new(root.clone());
        let count = Arc::new(Mutex::new(0));
        let count1 = count.clone();
        let count2 = count.clone();
        let handle1 = client.generate("test", (1, 2, 3), move |k| {
            *count1.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });
        let handle2 = client.generate("test", (1, 2, 3), move |k| {
            *count2.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });

        assert_eq!(handle1.get(), handle2.get());

        handle.abort();

        let server = CacheServer::new(root).with_local("0.0.0.0:28058".parse().unwrap());
        drop(runtime.spawn(async move { server.start().await }));

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let count2 = count.clone();
        let handle2 = client.generate("test", (1, 2, 3), move |k| {
            *count2.lock().unwrap() += 1;
            k.0 + k.1 + k.2
        });

        assert_eq!(handle1.get(), handle2.get());
        assert_eq!(*count.lock().unwrap(), 1);

        Ok(())
    }

    #[test]
    #[ignore = "long"]
    fn remote_client_runs_long_running_tasks_once() -> Result<()> {
        let root = PathBuf::from(BUILD_DIR).join("remote_client_runs_long_running_tasks_once");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let server = CacheServer::new(root.clone()).with_remote("0.0.0.0:28059".parse().unwrap());
        drop(runtime.spawn(async move { server.start().await }));

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let client = RemoteCacheClient::new(root);
        let count = Arc::new(Mutex::new(0));
        let count1 = count.clone();
        let count2 = count.clone();
        let handle1 = client.generate("test", (1, 2, 3), move |k| {
            *count1.lock().unwrap() += 1;
            std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS + 1));
            k.0 + k.1 + k.2
        });
        let handle2 = client.generate("test", (1, 2, 3), move |k| {
            *count2.lock().unwrap() += 1;
            std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS + 1));
            k.0 + k.1 + k.2
        });

        assert_eq!(handle1.get(), handle2.get());
        assert_eq!(*count.lock().unwrap(), 1);

        Ok(())
    }
}
