//! In-memory caching utilities.

use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::Arc,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::ArcResult, run_generator, CacheHandle, CacheHandleInner, CacheValueHolder, GenerateFn,
    GenerateResultFn, Namespace,
};

#[derive(Debug)]
struct TypeMapInner<T> {
    /// Effectively a map from `T -> HashMap<K, V>`.
    entries: HashMap<T, Box<dyn Any + Send + Sync>>,
}

impl<T> Default for TypeMapInner<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl<T: Hash + PartialEq + Eq> TypeMapInner<T> {
    fn get_or_insert<K: Hash + Eq + Any + Send + Sync, V: Any + Send + Sync>(
        &mut self,
        namespace: T,
        key: K,
        f: impl FnOnce() -> V,
    ) -> &V {
        let entry = self
            .entries
            .entry(namespace)
            .or_insert_with(|| Box::<HashMap<K, V>>::default());

        entry
            .downcast_mut::<HashMap<K, V>>()
            .unwrap()
            .entry(key)
            .or_insert_with(f)
    }
}

/// An in-memory cache based on hashable types.
#[derive(Default, Debug)]
pub struct TypeCache {
    /// A map from key type to another map from key to value handle.
    ///
    /// Effectively, the type of this map is `TypeId::of::<K>() -> HashMap<Arc<K>, Arc<OnceCell<Result<V, E>>>`.
    map: TypeMapInner<TypeId>,
}

impl TypeCache {
    /// Creates a new cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// The blocking equivalent of [`TypeCache::generate`].
    pub fn generate_blocking<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        key: K,
        generate_fn: impl GenerateFn<V>,
    ) -> &V {
        self.map
            .get_or_insert(TypeId::of::<K>(), key, move || {
                CacheHandle::new_blocking(generate_fn)
            })
            .get()
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
        generate_fn: impl GenerateFn<V>,
    ) -> CacheHandle<V> {
        self.map
            .get_or_insert(TypeId::of::<K>(), key, move || {
                CacheHandle::new(generate_fn)
            })
            .clone()
    }
}

/// Maps from a key to a handle on a value that may be set to [`None`] if the generator returns an
/// uncacheable result. In this case, the result must be regenerated each time.
type NamespaceEntryMap = HashMap<Vec<u8>, CacheHandleInner<Option<Vec<u8>>>>;

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
    entries: HashMap<Namespace, NamespaceEntryMap>,
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
        namespace: impl Into<Namespace>,
        key: K,
        generate_fn: impl GenerateFn<V>,
    ) -> CacheHandle<V> {
        CacheHandle::from_inner(Arc::new(self.generate_inner(namespace, key, generate_fn)))
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
        namespace: impl Into<Namespace>,
        key: K,
        generate_fn: impl GenerateResultFn<V, E>,
    ) -> CacheHandle<std::result::Result<V, E>> {
        CacheHandle::from_inner(Arc::new(self.generate_result_inner(
            namespace,
            key,
            generate_fn,
        )))
    }

    pub(crate) fn generate_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<Namespace>,
        key: K,
        generate_fn: impl GenerateFn<V>,
    ) -> CacheHandleInner<V> {
        let namespace = namespace.into();
        self.generate_inner_dispatch(
            namespace,
            key,
            generate_fn,
            |value| Some(flexbuffers::to_vec(value).unwrap()),
            |value| flexbuffers::from_slice(value).map_err(|e| Arc::new(e.into())),
        )
    }
    pub(crate) fn generate_result_inner<
        K: Serialize + Any + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync + Any,
        E: Send + Sync + Any,
    >(
        &mut self,
        namespace: impl Into<Namespace>,
        key: K,
        generate_fn: impl GenerateResultFn<V, E>,
    ) -> CacheHandleInner<std::result::Result<V, E>> {
        let namespace = namespace.into();
        self.generate_inner_dispatch(
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

    fn generate_inner_dispatch<K: Serialize + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        namespace: Namespace,
        key: K,
        generate_fn: impl GenerateFn<V>,
        serialize_value: impl SerializeValueFn<V>,
        deserialize_value: impl DeserializeValueFn<V>,
    ) -> CacheHandleInner<V> {
        let handle = CacheHandleInner::default();
        let hash = crate::hash(&flexbuffers::to_vec(&key).unwrap());

        let (in_progress, entry) = match self.entries.entry(namespace).or_default().entry(hash) {
            Entry::Vacant(v) => (false, v.insert(CacheHandleInner::default()).clone()),
            Entry::Occupied(o) => (true, o.get().clone()),
        }
        .clone();

        let entry2 = entry.clone();
        let handle_clone = handle.clone();
        rayon::spawn(move || {
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
                let v = run_generator(generate_fn);
                if !in_progress {
                    entry2.set(v.as_ref().map(serialize_value).map_err(|e| e.clone()));
                }
                handle_clone.set(v);
            }
        });

        handle
    }
}
