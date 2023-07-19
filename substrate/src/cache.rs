//! Caching utilities.

use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use cache::{mem::TypeCache, multi::MultiCache, CacheHandle, Cacheable, CacheableWithState};
use serde::{de::DeserializeOwned, Serialize};

/// A cache with APIs for in-memory and persistent caching.
#[derive(Default, Debug, Clone)]
pub struct Cache {
    inner: Arc<Mutex<CacheInner>>,
}

#[derive(Default, Debug)]
struct CacheInner {
    type_cache: TypeCache,
    cache: MultiCache,
}

impl Cache {
    /// Creates a new [`Cache`] with the provided [`MultiCache`] configuration.
    pub fn new(cache: MultiCache) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CacheInner {
                type_cache: TypeCache::new(),
                cache,
            })),
        }
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// If configured, persists data to disk.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`Cache::get_with_err`].
    pub fn get<K: Cacheable>(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.get(namespace, key)
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed and persists data to disk according to
    /// configuration.
    pub fn get_with_err<
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: Cacheable<Error = E>,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.get_with_err(namespace, key)
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// If configured, persists data to disk.
    ///
    /// Does not cache errors, so any errors thrown should be thrown quickly. Any errors that need
    /// to be cached should be included in the cached output or should be cached using
    /// [`Cache::get_with_state_and_err`].
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.get_with_state(namespace, key, state)
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed and persists data to disk according to
    /// configuration.
    ///
    /// See [`Cache::get_with_err`] and [`Cache::get_with_state`] for related examples.
    pub fn get_with_state_and_err<
        S: Send + Sync + Any,
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: CacheableWithState<S, Error = E>,
    >(
        &self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.get_with_state_and_err(namespace, key, state)
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Only caches data in memory.
    pub fn type_get<K: Cacheable>(
        &self,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner.type_cache.generate(key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// Only caches data in memory.
    ///
    /// **Note:** The state is not used to determine whether the object should be regenerated. As
    /// such, it should not impact the output of this function but rather should only be used to
    /// store collateral or reuse computation from other function calls.
    ///
    /// However, the entries generated with different state types are not interchangeable. That is,
    /// getting the same key with different states will regenerate the key several times, once for
    /// each state type `S`.
    pub fn type_get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &self,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        let mut inner = self.inner.lock().unwrap();
        inner
            .type_cache
            .generate_with_state(key, state, |key, state| key.generate_with_state(state))
    }
}
