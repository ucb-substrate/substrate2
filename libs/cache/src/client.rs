//! A client for interacting with a cache server.

use std::{any::Any, net::SocketAddr, path::PathBuf, sync::Arc, thread, time::Duration};

use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    runtime::{Handle, Runtime},
};
use tonic::transport::Channel;

use crate::{
    error::Result,
    mem::CacheHandle,
    rpc::cache::{
        get_reply, remote_cache_client, GetReply, GetRequest, HeartbeatRequest, SetRequest,
    },
    server::{ConfigManifest, CONFIG_MANIFEST_NAME},
};

pub const POLL_INTERVAL_MS: u64 = 100;

#[derive(Clone, Debug)]
pub struct CacheClient {
    root: Arc<PathBuf>,
    runtime: Arc<Runtime>,
}

impl CacheClient {
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
        Ok(
            remote_cache_client::RemoteCacheClient::connect(format!("http://{}", manifest.addr))
                .await?,
        )
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

    pub fn generate<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + 'static,
        E: Send + Sync + 'static,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl FnOnce(&K) -> std::result::Result<V, E> + Send + 'static,
        panic_error: E,
    ) -> CacheHandle<V, E> {
        let namespace = namespace.into();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());

        let cell = Arc::new(OnceCell::new());

        let cell2 = cell.clone();
        let self_clone = self.clone();
        thread::spawn(move || -> Result<()> {
            let cell3 = cell2.clone();
            let run_generation = || {
                let handle = thread::spawn(move || {
                    let value = generate_fn(&key);
                    if cell3.set(value).is_err() {
                        panic!("failed to set cell value");
                    }
                });
                if handle.join().is_err() && cell2.set(Err(panic_error)).is_err() {
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
                        run_generation();
                        // TODO: Add heartbeats.
                        if let Ok(data) = cell2.wait() {
                            self_clone.set_rpc(id, flexbuffers::to_vec(data).unwrap())?;
                        }
                        break;
                    }
                    get_reply::EntryStatus::Loading(_) => {
                        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                    }
                    get_reply::EntryStatus::Ready(data) => {
                        if cell2.set(Ok(flexbuffers::from_slice::<V>(&data)?)).is_err() {
                            panic!("failed to set cell value");
                        }
                        break;
                    }
                }
            }
            Ok(())
        });

        CacheHandle(cell)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::anyhow;

    use crate::error::Result;

    use super::CacheClient;

    const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

    #[test]
    fn client_works() -> Result<()> {
        let client = CacheClient::new(PathBuf::from(BUILD_DIR));
        let handle1 = client.generate(
            "test",
            (1, 2, 3),
            |k| {
                std::thread::sleep_ms(3000);
                Ok(k.0 + k.1 + k.2)
            },
            anyhow!("panic during generation"),
        );
        let handle2 = client.generate(
            "test",
            (1, 2, 3),
            |k| {
                std::thread::sleep_ms(3000);
                Ok(k.0 + k.1 + k.2)
            },
            anyhow!("panic during generation"),
        );

        assert_eq!(handle1.get(), handle2.get());

        Ok(())
    }
}
