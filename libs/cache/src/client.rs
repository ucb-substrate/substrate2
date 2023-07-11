//! A client for interacting with a cache server.

use std::{
    any::Any,
    path::PathBuf,
    sync::{
        mpsc::{channel, RecvTimeoutError},
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
    rpc::cache::{
        get_reply, remote_cache_client, GetReply, GetRequest, HeartbeatRequest, SetRequest,
    },
    server::{ConfigManifest, CONFIG_MANIFEST_NAME, HEARTBEAT_INTERVAL_SECS},
    CacheHandle,
};

/// The interval between polling the cache server on whether a value has finished loading.
pub const POLL_INTERVAL_MS: u64 = 100;

/// The timeout for connecting to the cache server.
pub const CONNECTION_TIMEOUT_MS: u64 = 1000;

/// The timeout for making a request to the cache server.
pub const REQUEST_TIMEOUT_MS: u64 = 1000;

/// A gRPC cache client.
#[derive(Clone, Debug)]
pub struct CacheClient {
    root: Arc<PathBuf>,
    runtime: Arc<Runtime>,
}

impl CacheClient {
    /// Creates a new gRPC cache client
    ///
    /// Starts a [`tokio::runtime::Runtime`] for making asynchronous gRPC requests.
    pub fn new(root: PathBuf) -> Self {
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

    async fn connect(&self) -> Result<remote_cache_client::RemoteCacheClient<Channel>> {
        let manifest = self.read_config_manifest().await?;
        let endpoint = Endpoint::from_shared(format!("http://{}", manifest.addr))?
            .timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
            .connect_timeout(Duration::from_millis(CONNECTION_TIMEOUT_MS));
        let test = remote_cache_client::RemoteCacheClient::connect(endpoint).await;
        Ok(test?)
    }

    fn get_rpc(&self, namespace: String, key: Vec<u8>) -> Result<get_reply::EntryStatus> {
        let out = self.runtime.block_on(async {
            let mut client = self.connect().await?;
            let out: Result<GetReply> = Ok(client
                .get(GetRequest {
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
        self.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.heartbeat(HeartbeatRequest { id }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
    }

    fn set_rpc(&self, id: u64, value: Vec<u8>) -> Result<()> {
        self.runtime.block_on(async {
            let mut client = self.connect().await?;
            client.set(SetRequest { id, value }).await?;
            let out: Result<()> = Ok(());
            out
        })?;
        Ok(())
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    pub fn generate<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + 'static,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + 'static,
    ) -> CacheHandle<V> {
        let namespace = namespace.into();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());

        let handle = CacheHandle(Arc::new(OnceCell::new()));

        let handle2 = handle.clone();
        let self_clone = self.clone();
        thread::spawn(move || {
            let inner = || -> Result<()> {
                let handle3 = handle2.clone();
                let run_generation = || {
                    let handle = thread::spawn(move || {
                        let value = generate_fn(&key);
                        if handle3.0.set(Ok(value)).is_err() {
                            panic!("failed to set cell value");
                        }
                    });
                    if handle.join().is_err() && handle2.0.set(Err(Error::Panic)).is_err() {
                        panic!("failed to set cell value on panic");
                    }
                };
                loop {
                    let status = self_clone.get_rpc(namespace.clone(), hash.clone())?;
                    match status {
                        get_reply::EntryStatus::Unassigned(_) => {
                            run_generation();
                            break;
                        }
                        get_reply::EntryStatus::Assign(id) => {
                            let (s_heartbeat_stop, r_heartbeat_stop) = channel();
                            let (s_heartbeat_stopped, r_heartbeat_stopped) = channel();
                            let self_clone2 = self_clone.clone();
                            thread::spawn(move || {
                                loop {
                                    match r_heartbeat_stop
                                        .recv_timeout(Duration::from_secs(HEARTBEAT_INTERVAL_SECS))
                                    {
                                        Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                            break;
                                        }
                                        Err(RecvTimeoutError::Timeout) => {
                                            if self_clone2.heartbeat_rpc(id).is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                                let _ = s_heartbeat_stopped.send(());
                            });
                            run_generation();
                            if let Ok(data) = handle2.try_get() {
                                let _ = s_heartbeat_stop.send(());
                                let _ = r_heartbeat_stopped.recv();
                                self_clone.set_rpc(id, flexbuffers::to_vec(data).unwrap())?;
                            }
                            break;
                        }
                        get_reply::EntryStatus::Loading(_) => {
                            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                        }
                        get_reply::EntryStatus::Ready(data) => {
                            if handle2
                                .0
                                .set(Ok(flexbuffers::from_slice::<V>(&data)?))
                                .is_err()
                            {
                                panic!("failed to set cell value");
                            }
                            break;
                        }
                    }
                }
                Ok(())
            };
            if let Err(e) = inner() {
                let _ = handle2.0.set(Err(e));
            }
        });

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
        server::{CacheServer, HEARTBEAT_TIMEOUT_SECS},
    };

    use super::CacheClient;

    const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

    #[test]
    fn client_retrieves_persistently_cached_values() -> Result<()> {
        let root = PathBuf::from(BUILD_DIR).join("client_retrieves_persistently_cached_values");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let server = CacheServer::new(root.clone(), "0.0.0.0:28055".parse().unwrap());
        let handle = runtime.spawn(async move { server.start().await });

        std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

        let client = CacheClient::new(root.clone());
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

        let server = CacheServer::new(root, "0.0.0.0:28056".parse().unwrap());
        drop(runtime.spawn(async move { server.start().await }));

        std::thread::sleep(Duration::from_millis(100)); // Wait until server starts.

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
    fn client_runs_long_running_tasks_once() -> Result<()> {
        let root = PathBuf::from(BUILD_DIR).join("client_runs_long_running_tasks_once");
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let server = CacheServer::new(root.clone(), "0.0.0.0:28055".parse().unwrap());
        drop(runtime.spawn(async move { server.start().await }));

        std::thread::sleep(Duration::from_millis(100)); // Wait until server starts.

        let client = CacheClient::new(root);
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
