//! In-memory caching utilities.

use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
    thread,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::ArcResult, run_generator, CacheHandle, Cacheable, CacheableWithState, GenerateFn,
    GenerateResultFn, GenerateResultWithStateFn, GenerateWithStateFn,
};

#[derive(Debug, Clone)]
struct TypeCacheMap<T> {
    /// Effectively a map from `T -> HashMap<Arc<K>, CacheHandle<V>>`.
    entries: HashMap<T, Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl<T> Default for TypeCacheMap<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl<T: Hash + PartialEq + Eq> TypeCacheMap<T> {
    fn generate<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        namespace: T,
        key: K,
        generate_fn: impl GenerateFn<K, V>,
    ) -> CacheHandle<V> {
        let key = Arc::new(key);

        let entry = self
            .entries
            .entry(namespace)
            .or_insert(Arc::new(Mutex::<HashMap<Arc<K>, CacheHandle<V>>>::default()));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<Arc<K>, CacheHandle<V>>>()
            .unwrap()
            .entry(key.clone());

        match entry {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => v
                .insert(CacheHandle::new(move || generate_fn(key.as_ref())))
                .clone(),
        }
    }
}

/// An in-memory cache based on hashable types.
#[derive(Default, Debug, Clone)]
pub struct TypeCache {
    /// A map from key type to another map from key to value handle.
    ///
    /// Effectively, the type of this map is `TypeId::of::<K>() -> HashMap<Arc<K>, Arc<OnceCell<Result<V, E>>>`.
    cells: TypeCacheMap<TypeId>,
    /// A map from key and state types to another map from key to value handle.
    ///
    /// Effectively, the type of this map is
    /// `(TypeId::of::<K>(), TypeId::of::<S>()) -> HashMap<Arc<K>, Arc<OnceCell<Result<V, E>>>`.
    cells_with_state: TypeCacheMap<(TypeId, TypeId)>,
}

impl TypeCache {
    /// Creates a new cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`TypeCache::get`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Panics
    ///
    /// Panics if a different type `V` or `E` is already associated with type `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::TypeCache, error::Error, CacheableWithState};
    ///
    /// let mut cache = TypeCache::new();
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate((5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate((5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    /// ```
    pub fn generate<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        key: K,
        generate_fn: impl GenerateFn<K, V>,
    ) -> CacheHandle<V> {
        self.cells.generate(TypeId::of::<K>(), key, generate_fn)
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`TypeCache::get_with_state`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Panics
    ///
    /// Panics if a different type `V` or `E` is already associated with type `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::TypeCache, error::Error, CacheableWithState};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let mut cache = TypeCache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// fn generate_fn(tuple: &(u64, u64), state: Log) -> u64 {
    ///     println!("Logging parameters...");
    ///     state.0.lock().unwrap().push(*tuple);
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate_with_state((5, 6), log.clone(), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate_with_state((5, 6), log.clone(), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![(5, 6)]);
    /// ```
    pub fn generate_with_state<
        K: Hash + Eq + Any + Send + Sync,
        V: Send + Sync + Any,
        S: Send + Sync + Any,
    >(
        &mut self,
        key: K,
        state: S,
        generate_fn: impl GenerateWithStateFn<K, S, V>,
    ) -> CacheHandle<V> {
        self.cells_with_state
            .generate((TypeId::of::<K>(), TypeId::of::<S>()), key, |key| {
                generate_fn(key, state)
            })
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::TypeCache, error::Error, Cacheable};
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
    ///         if self.param1 == 5 {
    ///             anyhow::bail!("invalid param");
    ///         } else if &self.param2 == "panic" {
    ///             panic!("unrecoverable param");
    ///         }
    ///         Ok(2 * self.param1)
    ///     }
    /// }
    ///
    /// let mut cache = TypeCache::new();
    ///
    /// let handle = cache.get(Params { param1: 50, param2: "cache".to_string() });
    /// assert_eq!(*handle.unwrap_inner(), 100);
    ///
    /// let handle = cache.get(Params { param1: 5, param2: "cache".to_string() });
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get(Params { param1: 50, param2: "panic".to_string() });
    /// assert!(matches!(handle.get_err().as_ref(), Error::Panic));
    /// ```
    pub fn get<K: Cacheable>(
        &mut self,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate(key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// **Note:** The state is not used to determine whether the object should be regenerated. As
    /// such, it should not impact the output of this function but rather should only be used to
    /// store collateral or reuse computation from other function calls.
    ///
    /// However, the entries generated with different state types are not interchangeable. That is,
    /// getting the same key with different states will regenerate the key several times, once for
    /// each state type `S`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::TypeCache, error::Error, CacheableWithState};
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
    ///
    ///         if self.0 == 5 {
    ///             anyhow::bail!("invalid param");
    ///         } else if self.0 == 8 {
    ///             panic!("unrecoverable param");
    ///         }
    ///         Ok(2 * self.0)
    ///     }
    /// }
    ///
    /// let mut cache = TypeCache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// let handle = cache.get_with_state(Params(0), log.clone());
    /// assert_eq!(*handle.unwrap_inner(), 0);
    ///
    /// let handle = cache.get_with_state(Params(5), log.clone());
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get_with_state(Params(8), log.clone());
    /// assert!(matches!(handle.get_err().as_ref(), Error::Panic));
    ///
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![Params(0), Params(5), Params(8)]);
    /// ```
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &mut self,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_with_state(key, state, |key, state| key.generate_with_state(state))
    }
}

/// Maps from a key to a handle on a value that may be set to [`None`] if the generator returns an
/// uncacheable result. In this case, the result must be regenerated each time.
type NamespaceEntryMap = HashMap<Vec<u8>, CacheHandle<Option<Vec<u8>>>>;

/// Serializes the provided value to bytes, returning [`None`] if the value should not be cached.
trait SerializeValueFn<V>: FnOnce(&V) -> Option<Vec<u8>> + Send + Any {}
impl<V, T: FnOnce(&V) -> Option<Vec<u8>> + Send + Any> SerializeValueFn<V> for T {}

/// Deserializes desired value from bytes stored in the cache. If `V` is a result, would need to
/// wrap the bytes from the cache with an `Ok` since `Err` results are not stored in the cache.
trait DeserializeValueFn<V>: FnOnce(&[u8]) -> ArcResult<V> + Send + Any {}
impl<V, T: FnOnce(&[u8]) -> ArcResult<V> + Send + Any> DeserializeValueFn<V> for T {}

/// An in-memory cache based on namespace strings and types that implement [`Serialize`] and
/// [`Deserialize`](serde::Deserialize).
///
/// Unlike a [`TypeCache`], a [`NamespaceCache`] works by serializing and deserializing keys and
/// values. As such, an entry can be accessed with several generic types as long as all of the
/// types serialize/deserialize to/from the same bytes.
#[derive(Default, Debug, Clone)]
pub struct NamespaceCache {
    /// A map from namespace to another map from key to value handle.
    entries: HashMap<String, NamespaceEntryMap>,
}

impl NamespaceCache {
    /// Creates a new cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`NamespaceCache::get`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Panics
    ///
    /// Panics if a different type `V` or `E` is already associated with type `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::NamespaceCache, error::Error, CacheableWithState};
    ///
    /// let mut cache = NamespaceCache::new();
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// fn generate_fn2(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 * tuple.1
    /// }
    ///
    /// let handle = cache.generate("example.namespace", (5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate("example.namespace", (5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Calls the new `generate_fn2` as the namespace is different,
    /// // even though the key is the same.
    /// let handle = cache.generate("example.namespace2", (5, 6), generate_fn2);
    /// assert_eq!(*handle.get(), 30);
    /// ```
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
            |value| Some(flexbuffers::to_vec(value).unwrap()),
            |value| flexbuffers::from_slice(value).map_err(|e| Arc::new(e.into())),
        )
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`NamespaceCache::get_with_state`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Panics
    ///
    /// Panics if a different type `V` or `E` is already associated with type `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::NamespaceCache, error::Error, CacheableWithState};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let mut cache = NamespaceCache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// fn generate_fn(tuple: &(u64, u64), state: Log) -> u64 {
    ///     println!("Logging parameters...");
    ///     state.0.lock().unwrap().push(*tuple);
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate_with_state("example.namespace", (5, 6), log.clone(), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate_with_state("example.namespace", (5, 6), log.clone(), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![(5, 6)]);
    /// ```
    pub fn generate_with_state<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        S: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl GenerateWithStateFn<K, S, V>,
    ) -> CacheHandle<V> {
        self.generate(namespace.into(), key, |key| generate_fn(key, state))
    }

    /// Ensures that a result corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`NamespaceCache::generate`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::NamespaceCache, error::Error, Cacheable};
    ///
    /// let mut cache = NamespaceCache::new();
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> anyhow::Result<u64> {
    ///     if *tuple == (5, 5) {
    ///         Err(anyhow::anyhow!("invalid tuple"))
    ///     } else {
    ///         Ok(tuple.0 + tuple.1)
    ///     }
    /// }
    ///
    /// let handle = cache.generate_result("example.namespace", (5, 5), generate_fn);
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    ///
    /// // Calls `generate_fn` again as the error was not cached.
    /// let handle = cache.generate_result("example.namespace", (5, 5), generate_fn);
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    /// ```
    pub fn generate_result<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl GenerateResultFn<K, V, E>,
    ) -> CacheHandle<std::result::Result<V, E>> {
        self.generate_inner(
            namespace,
            key,
            generate_fn,
            |value| {
                value
                    .as_ref()
                    .ok()
                    .map(|value| flexbuffers::to_vec(value).unwrap())
            },
            |value| {
                flexbuffers::from_slice(value)
                    .map(|value| Ok(value))
                    .map_err(|e| Arc::new(e.into()))
            },
        )
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Does not cache on failure as errors are not constrained to be serializable/deserializable.
    /// As such, failures should happen quickly, or should be serializable and stored as part of
    /// cached value using [`NamespaceCache::generate_with_state`].
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::NamespaceCache, error::Error, Cacheable};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let mut cache = NamespaceCache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
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
    /// let handle = cache.generate_result_with_state(
    ///     "example.namespace", (5, 5), log.clone(), generate_fn
    /// );
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    ///
    /// // Calls `generate_fn` again as the error was not cached.
    /// let handle = cache.generate_result_with_state(
    ///     "example.namespace", (5, 5), log.clone(), generate_fn
    /// );
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid tuple");
    ///
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![(5, 5), (5, 5)]);
    /// ```
    pub fn generate_result_with_state<
        K: Serialize + Send + Sync + Any,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
        S: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
        generate_fn: impl GenerateResultWithStateFn<K, S, V, E>,
    ) -> CacheHandle<std::result::Result<V, E>> {
        self.generate_result(namespace, key, move |k| generate_fn(k, state))
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::NamespaceCache, error::Error, Cacheable};
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
    ///         if self.param1 == 5 {
    ///             anyhow::bail!("invalid param");
    ///         } else if &self.param2 == "panic" {
    ///             panic!("unrecoverable param");
    ///         }
    ///         Ok(2 * self.param1)
    ///     }
    /// }
    ///
    /// let mut cache = NamespaceCache::new();
    ///
    /// let handle = cache.get(
    ///     "example.namespace", Params { param1: 50, param2: "cache".to_string() }
    /// );
    /// assert_eq!(*handle.unwrap_inner(), 100);
    ///
    /// let handle = cache.get(
    ///     "example.namespace", Params { param1: 5, param2: "cache".to_string() }
    /// );
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get(
    ///     "example.namespace",Params { param1: 50, param2: "panic".to_string() }
    /// );
    /// assert!(matches!(handle.get_err().as_ref(), Error::Panic));
    /// ```
    pub fn get<K: Cacheable>(
        &mut self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_result(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, caching failures as well.
    ///
    /// Generates the object in the background if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::NamespaceCache, error::Error, Cacheable};
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
    ///     type Error = bool;
    ///
    ///     fn generate(&self) -> Result<Self::Output, Self::Error> {
    ///         if &self.param2 == "panic" {
    ///             panic!("unrecoverable param");
    ///         }
    ///         // Expensive computation...
    ///         # let computation_result = 5;
    ///         if computation_result == 5 {
    ///             return Err(false);
    ///         }
    ///         Ok(2 * self.param1)
    ///     }
    /// }
    ///
    /// let mut cache = NamespaceCache::new();
    ///
    /// let handle = cache.get_with_err(
    ///     "example.namespace", Params { param1: 5, param2: "cache".to_string() }
    /// );
    /// assert_eq!(*handle.unwrap_err_inner(), false);
    ///
    /// // Does not need to carry out the expensive computation again as the error is cached.
    /// let handle = cache.get_with_err(
    ///     "example.namespace", Params { param1: 5, param2: "cache".to_string() }
    /// );
    /// assert_eq!(*handle.unwrap_err_inner(), false);
    /// ```
    pub fn get_with_err<
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: Cacheable<Error = E>,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate(namespace, key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// **Note:** The state is not used to determine whether the object should be regenerated. As
    /// such, it should not impact the output of this function but rather should only be used to
    /// store collateral or reuse computation from other function calls.
    ///
    /// However, the entries generated with different state types are not interchangeable. That is,
    /// getting the same key with different states will regenerate the key several times, once for
    /// each state type `S`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::NamespaceCache, error::Error, CacheableWithState};
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
    ///
    ///         if self.0 == 5 {
    ///             anyhow::bail!("invalid param");
    ///         } else if self.0 == 8 {
    ///             panic!("unrecoverable param");
    ///         }
    ///
    ///         Ok(2 * self.0)
    ///     }
    /// }
    ///
    /// let mut cache = NamespaceCache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// let handle = cache.get_with_state(
    ///     "example.namespace",
    ///     Params(0),
    ///     log.clone(),
    /// );
    ///
    /// assert_eq!(*handle.unwrap_inner(), 0);
    ///
    /// let handle = cache.get_with_state(
    ///     "example.namespace",
    ///     Params(5),
    ///     log.clone(),
    /// );
    ///
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get_with_state(
    ///     "example.namespace",
    ///     Params(8),
    ///     log.clone(),
    /// );
    ///
    /// assert!(matches!(handle.get_err().as_ref(), Error::Panic));
    ///
    /// assert_eq!(log.0.lock().unwrap().clone(), vec![Params(0), Params(5), Params(8)]);
    /// ```
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &mut self,
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
    /// See [`NamespaceCache::get_with_err`] and [`NamespaceCache::get_with_state`] for related examples.
    pub fn get_with_state_and_err<
        S: Send + Sync + Any,
        E: Send + Sync + Serialize + DeserializeOwned + Any,
        K: CacheableWithState<S, Error = E>,
    >(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_with_state(namespace, key, state, |key, state| {
            key.generate_with_state(state)
        })
    }

    fn generate_inner<K: Serialize + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        namespace: impl Into<String>,
        key: K,
        generate_fn: impl GenerateFn<K, V>,
        serialize_value: impl SerializeValueFn<V>,
        deserialize_value: impl DeserializeValueFn<V>,
    ) -> CacheHandle<V> {
        let handle = CacheHandle::empty();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());

        let (in_progress, entry) = match self
            .entries
            .entry(namespace.into())
            .or_insert(HashMap::new())
            .entry(hash)
        {
            Entry::Vacant(v) => (false, v.insert(CacheHandle::empty()).clone()),
            Entry::Occupied(o) => (true, o.get().clone()),
        }
        .clone();

        let entry2 = entry.clone();
        let handle_clone = handle.clone();
        thread::spawn(move || {
            let value = if in_progress {
                match entry.try_get() {
                    Ok(Some(value)) => Some(Ok(value)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            } else {
                None
            };
            if let Some(value) = value {
                handle_clone.set(value.and_then(|value| deserialize_value(value)));
            } else {
                let v = run_generator(move || generate_fn(&key));
                if !in_progress {
                    entry2.set(v.as_ref().map(serialize_value).map_err(|e| e.clone()));
                }
                handle_clone.set(v);
            }
        });

        handle
    }
}
