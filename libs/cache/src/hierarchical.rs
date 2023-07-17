use std::any::Any;

use crate::{
    mem::{NamespaceCache, TypeCache},
    persistent::client::Client,
    CacheHandle,
};

use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct HierarchicalCache {
    type_cache: TypeCache,
    namespace_cache: Option<NamespaceCache>,
    clients: Vec<Client>,
}

impl Default for HierarchicalCache {
    fn default() -> Self {
        Self {
            type_cache: TypeCache::new(),
            namespace_cache: Some(NamespaceCache::new()),
            clients: Vec::new(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct HierarchicalCacheBuilder {
    skip_memory: bool,
    clients: Vec<Client>,
}

impl HierarchicalCacheBuilder {
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
    ///
    /// Providers are accessed in the order in which they are added. As such, faster providers
    /// (i.e. local/private providers) should be added first.
    pub fn with_provider(&mut self, client: Client) -> &mut Self {
        self.clients.push(client);
        self
    }

    pub fn build(&mut self) -> HierarchicalCache {
        HierarchicalCache {
            type_cache: TypeCache::new(),
            namespace_cache: if self.skip_memory {
                None
            } else {
                Some(NamespaceCache::new())
            },
            clients: self.clients.clone(),
        }
    }
}

impl HierarchicalCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> HierarchicalCacheBuilder {
        HierarchicalCacheBuilder::new()
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
}
