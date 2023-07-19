//! A cache with multiple providers.

use std::{
    any::Any,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

use crate::{
    error::ArcResult, mem::NamespaceCache, persistent::client::Client, run_generator, CacheHandle,
    Cacheable, CacheableWithState, GenerateFn, GenerateResultFn, GenerateResultWithStateFn,
    GenerateWithStateFn,
};

use serde::{de::DeserializeOwned, Serialize};

/// A cache with multiple providers.
///
/// Exposes a unified API for accessing an in-memory [`NamespaceCache`] as well as persistent
/// cache [`Client`]s.
#[derive(Debug, Clone)]
pub struct MultiCache {
    namespace_cache: Option<NamespaceCache>,
    clients: Vec<Client>,
}

impl Default for MultiCache {
    fn default() -> Self {
        Self {
            namespace_cache: Some(NamespaceCache::new()),
            clients: Vec::new(),
        }
    }
}

/// A builder for a [`MultiCache`].
#[derive(Default, Debug, Clone)]
pub struct MultiCacheBuilder {
    skip_memory: bool,
    clients: Vec<Client>,
}

type OptionGenerateHandle<V> = GenerateHandle<V, Option<V>>;

struct GenerateHandle<V, R> {
    has_value_r: Receiver<Option<CacheHandle<V>>>,
    value_s: Sender<R>,
    handle: CacheHandle<V>,
}

/// A generate function dispatched to cache provider `C` in order to retrieve a cache handle to a
/// value that the cache may or may not have, sent over the provided [`Sender`].
///
/// The receiver can then be used to recover value that the [`MultiCache`] gets, potentially from
/// other caches.
trait MultiGenerateFn<C, K, V, R>:
    Fn(&mut C, String, Arc<K>, Sender<Option<CacheHandle<V>>>, Receiver<R>) -> CacheHandle<V>
{
}
impl<
        C,
        K,
        V,
        R,
        T: Fn(&mut C, String, Arc<K>, Sender<Option<CacheHandle<V>>>, Receiver<R>) -> CacheHandle<V>,
    > MultiGenerateFn<C, K, V, R> for T
{
}

impl MultiCacheBuilder {
    /// Creates a new [`MultiCacheBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Skips caching results in memory.
    ///
    /// With this flag enabled, all cache accesses must go through a cache provider even if key in
    /// question was accessed earlier by the same process.
    pub fn skip_memory(&mut self) -> &mut Self {
        self.skip_memory = true;
        self
    }

    /// Adds a new provider to the cache.
    pub fn with_provider(&mut self, client: Client) -> &mut Self {
        self.clients.push(client);
        self
    }

    /// Builds a [`MultiCache`] from the configured parameters.
    pub fn build(&mut self) -> MultiCache {
        MultiCache {
            namespace_cache: if self.skip_memory {
                None
            } else {
                Some(NamespaceCache::new())
            },
            clients: self.clients.clone(),
        }
    }
}

impl MultiCache {
    /// Creates a [`MultiCache`] with only in-memory providers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`MultiCacheBuilder`].
    pub fn builder() -> MultiCacheBuilder {
        MultiCacheBuilder::new()
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// See [`Client::generate`] and [`NamespaceCache::generate`] for related examples.
    pub fn generate<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl GenerateFn<K, V>,
    ) -> CacheHandle<V> {
        self.generate_inner(
            namespace,
            key,
            generate_fn,
            |cache, namespace, key, has_value_s, value_r| {
                cache.generate(namespace, key, move |_| {
                    let _ = has_value_s.send(None);
                    value_r.recv().unwrap()
                })
            },
            |cache, namespace, key, has_value_s, value_r| {
                cache.generate(namespace, key, move |_| {
                    let _ = has_value_s.send(None);
                    // Panics if no value is provided. Clients do not cache generator panics.
                    value_r.recv().unwrap().unwrap()
                })
            },
            MultiCache::recover_value,
            MultiCache::send_value_to_providers,
        )
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// See [`Client::generate_with_state`] and [`NamespaceCache::generate_with_state`] for related examples.
    pub fn generate_with_state<
        K: Serialize + Send + Sync + Any,
        S: Send + Sync + Any,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl GenerateWithStateFn<K, S, V>,
    ) -> CacheHandle<V> {
        self.generate(namespace, key, move |k| generate_fn(k, state))
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`MultiCache::generate`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    ///
    /// See [`Client::generate_result`] and [`NamespaceCache::generate_result`] for related examples.
    pub fn generate_result<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl GenerateResultFn<K, V, E>,
    ) -> CacheHandle<Result<V, E>> {
        self.generate_inner(
            namespace,
            key,
            generate_fn,
            |cache, namespace, key, has_value_s, value_r| {
                cache.generate_result(namespace, key, move |_| {
                    let _ = has_value_s.send(None);
                    value_r.recv().unwrap()
                })
            },
            |cache, namespace, key, has_value_s, value_r| {
                cache.generate_result(namespace, key, move |_| {
                    let _ = has_value_s.send(None);
                    // Panics if no value is provided. Clients do not cache generator panics.
                    value_r.recv().unwrap().unwrap()
                })
            },
            MultiCache::recover_result,
            MultiCache::send_result_to_providers,
        )
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`MultiCache::generate_with_state`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// See [`Client::generate_result_with_state`] and
    /// [`NamespaceCache::generate_result_with_state`] for related examples.
    pub fn generate_result_with_state<
        K: Serialize + Send + Sync + Any,
        S: Send + Sync + Any,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl GenerateResultWithStateFn<K, S, V, E>,
    ) -> CacheHandle<Result<V, E>> {
        self.generate_result(namespace, key, move |k| generate_fn(k, state))
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`MultiCache::get_with_err`].
    ///
    /// See [`Client::get`] and [`NamespaceCache::get`] for related examples.
    pub fn get<K: Cacheable>(
        &mut self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        self.generate_result(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed.
    ///
    /// See [`Client::get_with_err`] and [`NamespaceCache::get_with_err`] for related examples.
    pub fn get_with_err<
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: Cacheable<Error = E>,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        self.generate(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`MultiCache::get_with_state_and_err`].
    ///
    /// See [`Client::get_with_state`] and [`NamespaceCache::get_with_state`] for related examples.
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        self.generate_result_with_state(namespace, key, state, |key, state| {
            key.generate_with_state(state)
        })
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed.
    ///
    /// See [`MultiCache::get_with_err`] and [`MultiCache::get_with_state`] for related examples.
    pub fn get_with_state_and_err<
        S: Send + Sync + Any,
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: CacheableWithState<S, Error = E>,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        self.generate_with_state(namespace, key, state, |key, state| {
            key.generate_with_state(state)
        })
    }

    /// Dispatches the provided generate_fn to a cache provider, attempting to recover the cached value in
    /// the background.
    fn start_generate<C, K, V: Send + Sync + Any, R>(
        cache: &mut C,
        namespace: String,
        key: Arc<K>,
        generate_fn: impl MultiGenerateFn<C, K, V, R>,
    ) -> GenerateHandle<V, R> {
        let (has_value_s, has_value_r) = channel();
        let (value_s, value_r) = channel();

        let handle = generate_fn(cache, namespace, key, has_value_s.clone(), value_r);

        let handle2 = handle.clone();
        std::thread::spawn(move || {
            let _ = handle2.try_get();
            let _ = has_value_s.send(Some(handle2));
        });

        GenerateHandle {
            has_value_r,
            value_s,
            handle,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_inner<K: Send + Sync + Any, V: Send + Sync + Any>(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl GenerateFn<K, V>,
        namespace_generate: impl MultiGenerateFn<NamespaceCache, K, V, V>,
        client_generate: impl MultiGenerateFn<Client, K, V, Option<V>>,
        recover_value: impl FnOnce(ArcResult<&V>) -> Option<V> + Send + Any,
        send_value_to_providers: impl Fn(&V, &mut [GenerateHandle<V, Option<V>>]) + Send + Any,
    ) -> CacheHandle<V> {
        let namespace = namespace.into();
        let key = Arc::new(key);

        let mut handle = CacheHandle::empty();
        let mut mem_handle = None;
        let mut client_handles = Vec::new();

        if let Some(cache) = &mut self.namespace_cache {
            tracing::debug!("dispatching request to in-memory cache");
            let (namespace, key) = (namespace.clone(), key.clone());
            let generate_handle =
                MultiCache::start_generate(cache, namespace, key, namespace_generate);
            handle = generate_handle.handle.clone();
            mem_handle = Some(generate_handle);
        }

        for (i, client) in self.clients.iter_mut().enumerate() {
            tracing::debug!("dispatching request to persistent client {}", i);
            let (namespace, key) = (namespace.clone(), key.clone());
            client_handles.push(MultiCache::start_generate(
                client,
                namespace,
                key,
                &client_generate,
            ));
        }

        let handle2 = handle.clone();

        tracing::debug!("spawning thread to aggregate results");
        std::thread::spawn(move || {
            let mut retrieved_value: Option<V> = None;
            for (i, has_value_r) in mem_handle
                .iter()
                .map(|x| &x.has_value_r)
                .chain(client_handles.iter().map(|x| &x.has_value_r))
                .enumerate()
            {
                tracing::debug!("waiting on generate handle {}", i);
                if let Some(value_handle) = has_value_r.recv().unwrap() {
                    tracing::debug!("received value from generate handle {}", i);
                    retrieved_value = recover_value(value_handle.try_get());
                    break;
                }
                tracing::debug!(
                    "did not receive value from generate handle {}, trying next handle",
                    i
                );
            }

            let value = retrieved_value.map(Ok).unwrap_or_else(|| {
                tracing::debug!("did not receive a value, generating now");
                run_generator(move || generate_fn(key.as_ref()))
            });

            if let Ok(value) = value.as_ref() {
                tracing::debug!("sending generated value to all clients");
                send_value_to_providers(value, &mut client_handles);
            }

            // Block until all clients have finished handling the received values.
            for (i, GenerateHandle { handle, .. }) in client_handles.iter().enumerate() {
                tracing::debug!("blocking on client {}", i);
                let _ = handle.try_get();
            }

            match value {
                Ok(value) => {
                    if let Some(mem_handle) = mem_handle {
                        let _ = mem_handle.value_s.send(value);
                    } else {
                        handle2.set(Ok(value));
                    }
                }
                e @ Err(_) => handle2.set(e),
            }
        });

        handle
    }

    fn recover_value<V: Serialize + DeserializeOwned>(
        retrieved_result: ArcResult<&V>,
    ) -> Option<V> {
        if let Ok(value) = retrieved_result {
            Some(flexbuffers::from_slice(&flexbuffers::to_vec(value).unwrap()).unwrap())
        } else {
            None
        }
    }

    fn recover_result<V: Serialize + DeserializeOwned, E>(
        retrieved_result: ArcResult<&Result<V, E>>,
    ) -> Option<Result<V, E>> {
        if let Ok(Ok(value)) = retrieved_result {
            Some(Ok(flexbuffers::from_slice(
                &flexbuffers::to_vec(value).unwrap(),
            )
            .unwrap()))
        } else {
            None
        }
    }

    fn send_value_to_providers<V: Serialize + DeserializeOwned>(
        value: &V,
        client_handles: &mut [OptionGenerateHandle<V>],
    ) {
        for GenerateHandle { value_s, .. } in client_handles.iter_mut() {
            let _ = value_s.send(Some(
                flexbuffers::from_slice(&flexbuffers::to_vec(value).unwrap()).unwrap(),
            ));
        }
    }

    fn send_result_to_providers<V: Serialize + DeserializeOwned, E>(
        value: &Result<V, E>,
        client_handles: &mut [OptionGenerateHandle<Result<V, E>>],
    ) {
        for GenerateHandle { value_s, .. } in client_handles.iter_mut() {
            if let Ok(value) = value {
                let _ = value_s.send(Some(Ok(flexbuffers::from_slice(
                    &flexbuffers::to_vec(value).unwrap(),
                )
                .unwrap())));
            } else {
                let _ = value_s.send(None);
            }
        }
    }
}
