//! In-memory caching utilities.

use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
    thread,
};

use once_cell::sync::OnceCell;

use crate::{
    error::{ArcResult, Error},
    CacheHandle, Cacheable, CacheableWithState,
};

/// An abstraction for generating values in the background and caching them
/// based on hashable types in memory.
#[derive(Default, Debug, Clone)]
pub struct Cache {
    /// A map from key type to another map from key to value handle.
    ///
    /// Effectively, the type of this map is `TypeId::of::<K>() -> HashMap<Arc<K>, Arc<OnceCell<Result<V, E>>>`.
    cells: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
    /// A map from key and state types to another map from key to value handle.
    ///
    /// Effectively, the type of this map is
    /// `(TypeId::of::<K>(), TypeId::of::<S>()) -> HashMap<Arc<K>, Arc<OnceCell<Result<V, E>>>`.
    cells_with_state: HashMap<(TypeId, TypeId), Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl Cache {
    /// Creates a new cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`Cache::get`].
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
    /// use cache::{mem::Cache, error::Error, CacheableWithState};
    ///
    /// let mut cache = Cache::new();
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate(
    ///     (5, 6),
    ///     generate_fn,
    /// );
    ///
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate(
    ///     (5, 6),
    ///     generate_fn,
    /// );
    ///
    /// assert_eq!(*handle.get(), 11);
    /// ```
    pub fn generate<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + Any,
    ) -> CacheHandle<V> {
        let key = Arc::new(key);

        let entry = self
            .cells
            .entry(TypeId::of::<K>())
            .or_insert(Arc::new(Mutex::<
                HashMap<Arc<K>, Arc<OnceCell<ArcResult<V>>>>,
            >::default()));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<Arc<K>, Arc<OnceCell<ArcResult<V>>>>>()
            .unwrap()
            .entry(key.clone());

        CacheHandle(match entry {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let cell = v.insert(Arc::new(OnceCell::new()));

                let cell2 = cell.clone();

                thread::spawn(move || {
                    let cell3 = cell2.clone();
                    let handle = thread::spawn(move || {
                        let value = generate_fn(key.as_ref());
                        if cell3.set(Ok(value)).is_err() {
                            panic!("failed to set cell value");
                        }
                    });
                    if handle.join().is_err() && cell2.set(Err(Arc::new(Error::Panic))).is_err() {
                        panic!("failed to set cell value on panic");
                    }
                });

                cell.clone()
            }
        })
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// A more general counterpart to [`Cache::get_with_state`].
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
    /// use cache::{mem::Cache, error::Error, CacheableWithState};
    ///
    /// #[derive(Clone)]
    /// pub struct Log(Arc<Mutex<Vec<(u64, u64)>>>);
    ///
    /// let mut cache = Cache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// fn generate_fn(tuple: &(u64, u64), state: Log) -> u64 {
    ///     println!("Logging parameters...");
    ///     state.0.lock().unwrap().push(*tuple);
    ///
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate_with_state(
    ///     (5, 6),
    ///     log.clone(),
    ///     generate_fn,
    /// );
    ///
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let handle = cache.generate_with_state(
    ///     (5, 6),
    ///     log.clone(),
    ///     generate_fn,
    /// );
    ///
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
        generate_fn: impl FnOnce(&K, S) -> V + Send + Any,
    ) -> CacheHandle<V> {
        let key = Arc::new(key);

        let entry = self
            .cells_with_state
            .entry((TypeId::of::<K>(), TypeId::of::<S>()))
            .or_insert(Arc::new(Mutex::<
                HashMap<Arc<K>, Arc<OnceCell<ArcResult<V>>>>,
            >::default()));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<Arc<K>, Arc<OnceCell<ArcResult<V>>>>>()
            .unwrap()
            .entry(key.clone());

        CacheHandle(match entry {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let cell = v.insert(Arc::new(OnceCell::new()));

                let cell2 = cell.clone();

                thread::spawn(move || {
                    let cell3 = cell2.clone();
                    let handle = thread::spawn(move || {
                        let value = generate_fn(key.as_ref(), state);
                        if cell3.set(Ok(value)).is_err() {
                            panic!("failed to set cell value");
                        }
                    });
                    if handle.join().is_err() && cell2.set(Err(Arc::new(Error::Panic))).is_err() {
                        panic!("failed to set cell value on panic");
                    }
                });

                cell.clone()
            }
        })
    }

    /// Gets a handle to a cacheable object from the cache, generating the object in the background
    /// if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use cache::{mem::Cache, error::Error, Cacheable};
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
    ///
    ///         Ok(2 * self.param1)
    ///     }
    /// }
    ///
    /// let mut cache = Cache::new();
    ///
    /// let handle = cache.get(Params {
    ///     param1: 50,
    ///     param2: "cache".to_string(),
    /// });
    ///
    /// assert_eq!(*handle.unwrap_inner(), 100);
    ///
    /// let handle = cache.get(Params {
    ///     param1: 5,
    ///     param2: "cache".to_string(),
    /// });
    ///
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get(Params {
    ///     param1: 50,
    ///     param2: "panic".to_string(),
    /// });
    ///
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
    /// use cache::{mem::Cache, error::Error, CacheableWithState};
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
    /// let mut cache = Cache::new();
    /// let log = Log(Arc::new(Mutex::new(Vec::new())));
    ///
    /// let handle = cache.get_with_state(
    ///     Params(0),
    ///     log.clone(),
    /// );
    ///
    /// assert_eq!(*handle.unwrap_inner(), 0);
    ///
    /// let handle = cache.get_with_state(
    ///     Params(5),
    ///     log.clone(),
    /// );
    ///
    /// assert_eq!(format!("{}", handle.unwrap_err_inner().root_cause()), "invalid param");
    ///
    /// let handle = cache.get_with_state(
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
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_with_state(key, state, |key, state| key.generate_with_state(state))
    }
}
