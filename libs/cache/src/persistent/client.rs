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
    CacheHandle, Cacheable, CacheableWithState,
};

/// The timeout for connecting to the cache server.
pub const CONNECTION_TIMEOUT_MS_DEFAULT: u64 = 1000;

/// The timeout for making a request to the cache server.
pub const REQUEST_TIMEOUT_MS_DEFAULT: u64 = 1000;

/// An enumeration of client kinds.
///
/// Each interacts with a different cache server API, depending on the desired functionality.
#[derive(Debug, Clone, Copy)]
pub enum ClientKind {
    /// A client that shares a filesystem with the server.
    ///
    /// Enables storing data in the cache via the filesystem without sending large bytestreams over gRPC.
    Local,
    /// A client that does not share a filseystem with the server.
    ///
    /// Sends data to the cache server via gRPC.
    Remote,
}

#[derive(Debug)]
struct ClientInner {
    kind: ClientKind,
    url: String,
    poll_backoff: ExponentialBackoff,
    connection_timeout: Duration,
    request_timeout: Duration,
    handle: Handle,
    // Only used to own a runtime created by the builder.
    #[allow(dead_code)]
    runtime: Option<Runtime>,
}

/// A gRPC cache client.
///
/// The semantics of the [`Client`] API are the same as those of the
/// [`NamespaceCache`](crate::mem::NamespaceCache) API.
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

/// A builder for a [`Client`].
#[derive(Default, Clone, Debug)]
pub struct ClientBuilder {
    kind: Option<ClientKind>,
    url: Option<String>,
    poll_backoff: Option<ExponentialBackoff>,
    connection_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    handle: Option<Handle>,
}

struct GenerateKey<K> {
    namespace: String,
    hash: Vec<u8>,
    key: K,
}

impl ClientBuilder {
    /// Creates a new [`ClientBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the configured server URL.
    pub fn url(&mut self, url: impl Into<String>) -> &mut Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the configured client type.
    pub fn kind(&mut self, kind: ClientKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }
    /// Creates a new [`ClientBuilder`] with configured client type [`ClientKind::Local`] and a
    /// server URL `url`.
    pub fn local(url: impl Into<String>) -> Self {
        let mut builder = Self::new();
        builder.kind(ClientKind::Local).url(url);
        builder
    }

    /// Creates a new [`ClientBuilder`] with configured client type [`ClientKind::Remote`] and a
    /// server URL `url`.
    pub fn remote(url: impl Into<String>) -> Self {
        let mut builder = Self::new();
        builder.kind(ClientKind::Remote).url(url);
        builder
    }

    /// Configures the exponential backoff used when polling the server for cache entry
    /// statuses.
    ///
    /// Defaults to [`ExponentialBackoff::default`].
    pub fn poll_backoff(&mut self, backoff: ExponentialBackoff) -> &mut Self {
        self.poll_backoff = Some(backoff);
        self
    }

    /// Sets the timeout for connecting to the server.
    ///
    /// Defaults to [`CONNECTION_TIMEOUT_MS_DEFAULT`].
    pub fn connection_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// Sets the timeout for receiving a reply from the server.
    ///
    /// Defaults to [`REQUEST_TIMEOUT_MS_DEFAULT`].
    pub fn request_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Configures a [`Handle`] for making asynchronous gRPC requests.
    ///
    /// If no handle is specified, starts a new [`tokio::runtime::Runtime`] upon building the
    /// [`Client`] object.
    pub fn runtime_handle(&mut self, handle: Handle) -> &mut Self {
        self.handle = Some(handle);
        self
    }

    /// Builds a [`Client`] object with the configured parameters.
    pub fn build(&mut self) -> Client {
        let (handle, runtime) = match self.handle.clone() {
            Some(handle) => (handle, None),
            None => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .enable_all()
                    .build()
                    .unwrap();
                (runtime.handle().clone(), Some(runtime))
            }
        };
        Client {
            inner: Arc::new(ClientInner {
                kind: self.kind.expect("must specify client kind"),
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
            }),
        }
    }
}

impl Client {
    /// Creates a new gRPC cache client for a server at `url` with default configuration values.
    pub fn with_default_config(kind: ClientKind, url: impl Into<String>) -> Self {
        Self::builder().kind(kind).url(url).build()
    }

    /// Creates a new gRPC cache client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Creates a new local gRPC cache client.
    ///
    /// See [`ClientKind`] for an explanation of the different kinds of clients.
    pub fn local(url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::local(url)
    }

    /// Creates a new remote gRPC cache client.
    ///
    /// See [`ClientKind`] for an explanation of the different kinds of clients.
    pub fn remote(url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::remote(url)
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::generate`](crate::mem::NamespaceCache::generate).
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_generate");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = client.generate("example.namespace", (5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    /// ```
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

        match self.inner.kind {
            ClientKind::Local => {
                self.clone()
                    .generate_inner_local(handle.clone(), namespace, hash, key, generate_fn)
            }
            ClientKind::Remote => self.clone().generate_inner_remote(
                handle.clone(),
                namespace,
                hash,
                key,
                generate_fn,
            ),
        }

        handle
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::generate_with_state`](crate::mem::NamespaceCache::generate_with_state).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_generate_with_state");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// fn generate_fn(tuple: &(u64, u64), state: Log) -> u64 {
    ///     println!("Logging parameters...");
    ///     state.0.lock().unwrap().push(*tuple);
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = client.generate_with_state(
    ///     "example.namespace", (5, 6), log.clone(), generate_fn
    /// );
    /// assert_eq!(*handle.get(), 11);
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![(5, 6)]);
    /// ```
    pub fn generate_with_state<
        K: Serialize + Send + Sync + Any,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        S: Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl FnOnce(&K, S) -> V + Send + Any,
    ) -> CacheHandle<V> {
        self.generate(namespace, key, move |k| generate_fn(k, state))
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`Client::generate`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated in the
    /// background.
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::generate_result`](crate::mem::NamespaceCache::generate_result).
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_generate_result");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> anyhow::Result<u64> {
    ///     if *tuple == (5, 5) {
    ///         Err(anyhow::anyhow!("invalid tuple"))
    ///     } else {
    ///         Ok(tuple.0 + tuple.1)
    ///     }
    /// }
    ///
    /// let handle = client.generate_result("example.namespace", (5, 5), generate_fn);
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    /// ```
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

        match self.inner.kind {
            ClientKind::Local => {
                self.clone().generate_result_inner_local(
                    handle.clone(),
                    namespace,
                    hash,
                    key,
                    generate_fn,
                );
            }
            ClientKind::Remote => {
                self.clone().generate_result_inner_remote(
                    handle.clone(),
                    namespace,
                    hash,
                    key,
                    generate_fn,
                );
            }
        }

        handle
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`Client::generate_with_state`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::generate_result_with_state`](crate::mem::NamespaceCache::generate_result_with_state).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root =
    /// # PathBuf::from(BUILD_DIR).join("persistent_client_Client_generate_result_with_state");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// fn generate_fn(tuple: &(u64, u64), state: Log) -> anyhow::Result<u64> {
    ///     println!("Logging parameters...");
    ///     state.0.lock().unwrap().push(*tuple);
    ///
    ///     if *tuple == (5, 5) {
    ///         Err(anyhow::anyhow!("invalid tuple"))
    ///     } else {
    ///         Ok(tuple.0 + tuple.1)
    ///     }
    /// }
    ///
    /// let handle = client.generate_result_with_state(
    ///     "example.namespace", (5, 5), log.clone(), generate_fn,
    /// );
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![(5, 5)]);
    /// ```
    pub fn generate_result_with_state<
        K: Serialize + Send + Sync + Any,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
        S: Send + Sync + Any,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl FnOnce(&K, S) -> std::result::Result<V, E> + Send + Any,
    ) -> CacheHandle<std::result::Result<V, E>> {
        self.generate_result(namespace, key, move |k| generate_fn(k, state))
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`Client::get_with_err`].
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::get`](crate::mem::NamespaceCache::get).
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize, Hash, Eq, PartialEq)]
    /// pub struct Params {
    ///     param1: u64,
    ///     param2: String,
    /// };
    ///
    /// impl Cacheable for Params {
    ///     type Output = u64;
    ///     type Error = anyhow::Error;
    ///
    ///     fn generate(&self) -> anyhow::Result<u64> {
    ///         Ok(2 * self.param1)
    ///     }
    /// }
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_get");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// let handle = client.get(
    ///     "example.namespace", Params { param1: 50, param2: "cache".to_string() }
    /// );
    /// assert_eq!(*handle.unwrap_inner(), 100);
    /// ```
    pub fn get<K: Cacheable>(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_result(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed.
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::get_with_err`](crate::mem::NamespaceCache::get_with_err).
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize, Hash, Eq, PartialEq)]
    /// pub struct Params {
    ///     param1: u64,
    ///     param2: String,
    /// };
    ///
    /// impl Cacheable for Params {
    ///     type Output = u64;
    ///     type Error = String;
    ///
    ///     fn generate(&self) -> Result<Self::Output, Self::Error> {
    ///         if self.param1 == 5 {
    ///             Err("invalid param".to_string())
    ///         } else {
    ///             Ok(2 * self.param1)
    ///         }
    ///     }
    /// }
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_get_with_err");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// let handle = client.get_with_err(
    ///     "example.namespace", Params { param1: 5, param2: "cache".to_string() }
    /// );
    /// assert_eq!(handle.unwrap_err_inner(), "invalid param");
    /// ```
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

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`Client::get_with_state_and_err`].
    ///
    /// For more detailed examples, refer to
    /// [`NamespaceCache::get_with_state`](crate::mem::NamespaceCache::get_with_state).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{persistent::client::{Client, ClientKind}, error::Error, CacheableWithState};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
    /// pub struct Params(u64);
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<Params>>>);
    ///
    /// impl CacheableWithState<Log> for Params {
    ///     type Output = u64;
    ///     type Error = anyhow::Error;
    ///
    ///     fn generate_with_state(&self, state: Log) -> anyhow::Result<u64> {
    ///         println!("Logging parameters...");
    ///         state.0.lock().unwrap().push(self.clone());
    ///         Ok(2 * self.0)
    ///     }
    /// }
    ///
    /// let client = Client::with_default_config(ClientKind::Local, "http://0.0.0.0:28055");
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    /// # use std::path::PathBuf;
    /// # use std::time::Duration;
    /// # use cache::persistent::server::Server;
    /// # const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    /// # let port = portpicker::pick_unused_port().expect("no open ports");
    /// # let runtime = tokio::runtime::Builder::new_multi_thread()
    /// #     .enable_all()
    /// #     .build()
    /// #     .unwrap();
    /// # let root = PathBuf::from(BUILD_DIR).join("persistent_client_Client_get_with_state");
    /// # if root.exists() {
    /// #     std::fs::remove_dir_all(&root).unwrap();
    /// # }
    /// # std::fs::create_dir_all(&root).unwrap();
    /// # let server = Server::builder()
    /// #     .root(root)
    /// #     .local(format!("0.0.0.0:{port}").parse().unwrap())
    /// #     .build();
    /// # drop(runtime.spawn(async move { server.start().await }));
    /// # std::thread::sleep(Duration::from_millis(500)); // Wait until server starts.
    /// # let client = Client::with_default_config(ClientKind::Local, format!("http://0.0.0.0:{port}"));
    ///
    /// let handle = client.get_with_state(
    ///     "example.namespace",
    ///     Params(0),
    ///     log.clone(),
    /// );
    /// assert_eq!(*handle.unwrap_inner(), 0);
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![Params(0)]);
    /// ```
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_result_with_state(namespace, key, state, |key, state| {
            key.generate_with_state(state)
        })
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed.
    ///
    /// See [`Client::get_with_err`] and [`Client::get_with_state`] for related examples.
    pub fn get_with_state_and_err<
        S: Send + Sync + Any,
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: CacheableWithState<S, Error = E>,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_with_state(namespace, key, state, |key, state| {
            key.generate_with_state(state)
        })
    }

    /// Sets up the necessary objects to be passed in to [`Client::spawn_handler`].
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
    fn spawn_handler<V: Send + Sync + Any>(
        self,
        handle: CacheHandle<V>,
        handler: impl FnOnce() -> Result<()> + Send + Any,
    ) {
        thread::spawn(move || {
            if let Err(e) = handler() {
                tracing::event!(
                    Level::ERROR,
                    "encountered error while executing handler: {}",
                    e,
                );
                handle.set(Err(Arc::new(e)));
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

    /// Runs the provided generator in a new thread, returning the result.
    fn run_generator<K: Any + Send + Sync, V: Any + Send + Sync>(
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
        tracing::debug!("starting heartbeats");
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

    /// Connects to a local cache gRPC server.
    async fn connect_local(&self) -> Result<local_cache_client::LocalCacheClient<Channel>> {
        let endpoint = Endpoint::from_shared(self.inner.url.clone())?
            .timeout(self.inner.request_timeout)
            .connect_timeout(self.inner.connection_timeout);
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
        let out: Result<local::GetReply> = self.inner.handle.block_on(async {
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
        self.inner.handle.block_on(async {
            let mut client = self.connect_local().await?;
            client.heartbeat(local::HeartbeatRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Done` RPC to a local cache gRPC server.
    fn done_rpc_local(&self, id: u64) -> Result<()> {
        self.inner.handle.block_on(async {
            let mut client = self.connect_local().await?;
            client.done(local::DoneRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Drop` RPC to a local cache gRPC server.
    fn drop_rpc_local(&self, id: u64) -> Result<()> {
        self.inner.handle.block_on(async {
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
    fn generate_loop_local<K: Send + Sync + Any, V: Send + Sync + Any>(
        &self,
        handle: CacheHandle<V>,
        key: GenerateKey<K>,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
        write_generated_value: impl FnOnce(&Self, u64, String, &ArcResult<V>) -> Result<()> + Send + Any,
        deserialize_cache_data: impl FnOnce(&[u8]) -> Result<V> + Send + Any,
    ) -> Result<()> {
        let GenerateKey {
            namespace,
            hash,
            key,
        } = key;

        let status = backoff::retry(self.inner.poll_backoff.clone(), move || {
            let inner = || -> Result<(local::get_reply::EntryStatus, bool)> {
                tracing::debug!("attempting local get request to retrieve entry status");
                let status = self.get_rpc_local(namespace.clone(), hash.clone(), true)?;
                let retry = matches!(status, local::get_reply::EntryStatus::Loading(_));

                Ok((status, retry))
            };
            inner()
                .map_err(backoff::Error::Permanent)
                .and_then(|(status, retry)| {
                    if retry {
                        tracing::debug!("entry is currently loading, retrying later");
                        Err(backoff::Error::transient(Error::EntryLoading))
                    } else {
                        Ok(status)
                    }
                })
        })
        .map_err(Box::new)?;

        match status {
            local::get_reply::EntryStatus::Unassigned(_) => {
                tracing::debug!("entry is unassigned, generating locally");
                let v = Client::run_generator(key, generate_fn);
                handle.set(v);
            }
            local::get_reply::EntryStatus::Assign(local::AssignReply {
                id,
                path,
                heartbeat_interval_ms,
            }) => {
                tracing::debug!("entry has been assigned to the client, generating locally");
                let self_clone = self.clone();
                let (s_heartbeat_stop, r_heartbeat_stopped) = self.start_heartbeats(
                    Duration::from_millis(heartbeat_interval_ms),
                    move || -> Result<()> { self_clone.heartbeat_rpc_local(id) },
                );
                let v = Client::run_generator(key, generate_fn);
                let _ = s_heartbeat_stop.send(());
                let _ = r_heartbeat_stopped.recv();
                tracing::debug!("finished generating, writing value to cache");
                write_generated_value(self, id, path, &v)?;
                handle.set(v);
            }
            local::get_reply::EntryStatus::Loading(_) => unreachable!(),
            local::get_reply::EntryStatus::Ready(local::ReadyReply { id, path }) => {
                tracing::debug!("entry is ready, reading from cache");
                let mut file = std::fs::File::open(path)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                self.drop_rpc_local(id)?;
                tracing::debug!("finished reading entry from disk");
                handle.set(Ok(deserialize_cache_data(&buf)?));
            }
        }
        Ok(())
    }

    fn generate_inner_local<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        self,
        handle: CacheHandle<V>,
        key: GenerateKey<K>,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) {
        tracing::debug!("generating using local cache API");
        self.clone().spawn_handler(handle.clone(), move || {
            self.generate_loop_local(
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

    fn generate_result_inner_local<
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
        self.clone().spawn_handler(handle.clone(), move || {
            self.generate_loop_local(
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

    /// Connects to a remote cache gRPC server.
    async fn connect_remote(&self) -> Result<remote_cache_client::RemoteCacheClient<Channel>> {
        let endpoint = Endpoint::from_shared(self.inner.url.clone())?
            .timeout(self.inner.request_timeout)
            .connect_timeout(self.inner.connection_timeout);
        Ok(remote_cache_client::RemoteCacheClient::connect(endpoint).await?)
    }

    /// Issues a `Get` RPC to a remote cache gRPC server.
    fn get_rpc_remote(
        &self,
        namespace: String,
        key: Vec<u8>,
        assign: bool,
    ) -> Result<remote::get_reply::EntryStatus> {
        let out: Result<remote::GetReply> = self.inner.handle.block_on(async {
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
        self.inner.handle.block_on(async {
            let mut client = self.connect_remote().await?;
            client.heartbeat(remote::HeartbeatRequest { id }).await?;
            Ok(())
        })
    }

    /// Issues a `Set` RPC to a remote cache gRPC server.
    fn set_rpc_remote(&self, id: u64, value: Vec<u8>) -> Result<()> {
        self.inner.handle.block_on(async {
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
        handle: CacheHandle<V>,
        namespace: String,
        hash: Vec<u8>,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
        write_generated_value: impl FnOnce(&Self, u64, &ArcResult<V>) -> Result<()> + Send + Any,
        deserialize_cache_data: impl FnOnce(&[u8]) -> Result<V> + Send + Any,
    ) -> Result<()> {
        let status = backoff::retry(self.inner.poll_backoff.clone(), move || {
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
                let v = Client::run_generator(key, generate_fn);
                Client::set_handle(&handle, v);
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
                let v = Client::run_generator(key, generate_fn);
                let _ = s_heartbeat_stop.send(());
                let _ = r_heartbeat_stopped.recv();
                write_generated_value(self, id, &v)?;
                Client::set_handle(&handle, v);
            }
            remote::get_reply::EntryStatus::Loading(_) => unreachable!(),
            remote::get_reply::EntryStatus::Ready(data) => {
                Client::set_handle(&handle, Ok(deserialize_cache_data(&data)?));
            }
        }
        Ok(())
    }

    fn generate_inner_remote<
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
        tracing::event!(Level::DEBUG, "generating using remote cache API");
        self.clone().spawn_handler(handle.clone(), move || {
            self.generate_loop_remote(
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

    fn generate_result_inner_remote<
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
        self.clone().spawn_handler(handle.clone(), move || {
            self.generate_loop_remote(
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
}
