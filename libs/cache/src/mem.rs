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
    error::{Error, Result},
    CacheHandle, Cacheable, CacheableWithState,
};

/// An abstraction for generating values in the background and caching them
/// based on hashable types in memory.
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
/// assert!(matches!(handle.get_err(), Error::Panic));
/// ```
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
/// assert!(matches!(handle.get_err(), Error::Panic));
///
/// assert_eq!(log.0.lock().unwrap().clone(), vec![Params(0), Params(5), Params(8)]);
/// ```
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

    /// Gets a handle to a cacheable object from the cache.
    pub fn get<K: Cacheable>(
        &mut self,
        key: K,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate(key, |key| key.generate())
    }

    /// Gets a handle to a cacheable object from the cache.
    ///
    /// **Note:** The state is not used to determine whether the object should be regenerated. As
    /// such, it should not impact the output of this function but rather should only be used to
    /// store collateral or reuse computation from other function calls.
    ///
    /// However, the entries generated with different state types are not interchangeable. That is,
    /// getting the same key with different states will regenerate the key several times, once for
    /// each state type `S`.
    pub fn get_with_state<S: Send + Sync + Any, K: CacheableWithState<S>>(
        &mut self,
        key: K,
        state: S,
    ) -> CacheHandle<std::result::Result<K::Output, K::Error>> {
        self.generate_with_state(key, state, |key, state| key.generate_with_state(state))
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
    pub fn generate<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + 'static>(
        &mut self,
        key: K,
        generate_fn: impl FnOnce(&K) -> V + Send + 'static,
    ) -> CacheHandle<V> {
        let key = Arc::new(key);

        let entry = self
            .cells
            .entry(TypeId::of::<K>())
            .or_insert(Arc::new(
                Mutex::<HashMap<Arc<K>, Arc<OnceCell<Result<V>>>>>::default(),
            ));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<Arc<K>, Arc<OnceCell<Result<V>>>>>()
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
                    if handle.join().is_err() && cell2.set(Err(Error::Panic)).is_err() {
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
    pub fn generate_with_state<
        K: Hash + Eq + Any + Send + Sync,
        V: Send + Sync + 'static,
        S: Send + Sync + Any,
    >(
        &mut self,
        key: K,
        state: S,
        generate_fn: impl FnOnce(&K, S) -> V + Send + 'static,
    ) -> CacheHandle<V> {
        let key = Arc::new(key);

        let entry = self
            .cells_with_state
            .entry((TypeId::of::<K>(), TypeId::of::<S>()))
            .or_insert(Arc::new(
                Mutex::<HashMap<Arc<K>, Arc<OnceCell<Result<V>>>>>::default(),
            ));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<Arc<K>, Arc<OnceCell<Result<V>>>>>()
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
                    if handle.join().is_err() && cell2.set(Err(Error::Panic)).is_err() {
                        panic!("failed to set cell value on panic");
                    }
                });

                cell.clone()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use anyhow::bail;
    use crossbeam_channel::unbounded;
    use serde::{Deserialize, Serialize};

    use crate::{
        error::Error,
        mem::{Cacheable, CacheableWithState},
    };

    use super::Cache;

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    struct Params1 {
        value: usize,
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    struct Params2 {
        variety: String,
        arc: Arc<Params1>,
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    struct Value<T> {
        inner: Arc<T>,
        extra: usize,
    }

    #[test]
    fn generates_in_background_and_caches_values() {
        let mut cache = Cache::new();
        let num_gen = Arc::new(Mutex::new(0));
        let (s, r) = unbounded();

        let num_gen_clone = num_gen.clone();

        let params1_func = move |params: &Params1| {
            *num_gen_clone.lock().unwrap() += 1;
            r.recv().unwrap();
            Value {
                inner: Arc::new("substrate".to_string()),
                extra: params.value,
            }
        };

        let p1 = Params1 { value: 5 };

        let expected1 = Value {
            inner: Arc::new("substrate".to_string()),
            extra: 5,
        };

        let handle1 = cache.generate(p1, params1_func.clone());

        // Should not use call the generator as the corresponding block is already being generated.
        let handle2 = cache.generate(p1, params1_func.clone());

        assert!(handle1.poll().is_none());
        assert!(handle2.poll().is_none());

        s.send(()).unwrap();

        assert_eq!(handle1.get(), &expected1);

        // Should reference the same cell as `handle1`.
        assert_eq!(handle2.get(), &expected1);

        // Should immediately return a filled cell as this has already been generated.
        let num_gen_clone = num_gen.clone();
        let handle3 = cache.generate(Params1 { value: 5 }, move |_| {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("circuit".to_string()),
                extra: 50,
            }
        });

        assert_eq!(handle3.get(), &expected1);

        // Should generate a new block as it has not been generated with the provided parameters
        // yet.
        let handle4 = cache.generate(Params1 { value: 10 }, params1_func);

        s.send(()).unwrap();

        assert_eq!(
            handle4.get(),
            &Value {
                inner: Arc::new("substrate".to_string()),
                extra: 10,
            }
        );

        assert_eq!(*num_gen.lock().unwrap(), 2);
    }

    #[test]
    fn caches_cacheable_keys() {
        #[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
        pub struct Key(u64);
        impl Cacheable for Key {
            type Output = u64;
            type Error = anyhow::Error;

            fn generate(&self) -> Result<Self::Output, Self::Error> {
                if self.0 == 5 {
                    bail!("invalid key");
                } else if self.0 == 8 {
                    panic!("panic during generation");
                }
                Ok(self.0)
            }
        }

        impl CacheableWithState<Arc<Mutex<Vec<u64>>>> for Key {
            type Output = u64;
            type Error = anyhow::Error;

            fn generate_with_state(
                &self,
                state: Arc<Mutex<Vec<u64>>>,
            ) -> Result<Self::Output, Self::Error> {
                let out = self.generate()?;
                println!("generating");
                state.lock().unwrap().push(out);
                Ok(out)
            }
        }

        let mut cache = Cache::new();
        let handle1 = cache.get(Key(0));
        let handle2 = cache.get(Key(5));
        let handle3 = cache.get(Key(8));

        assert_eq!(*handle1.unwrap_inner(), 0);
        assert_eq!(
            format!("{}", handle2.unwrap_err_inner().root_cause()),
            "invalid key"
        );
        assert!(matches!(handle3.get_err(), crate::error::Error::Panic));

        let state = Arc::new(Mutex::new(Vec::new()));
        let handle1 = cache.get_with_state(Key(0), state.clone());
        let handle2 = cache.get_with_state(Key(5), state.clone());
        let handle3 = cache.get_with_state(Key(8), state.clone());

        assert_eq!(*handle1.unwrap_inner(), 0);
        assert_eq!(
            format!("{}", handle2.unwrap_err_inner().root_cause()),
            "invalid key"
        );
        assert!(matches!(handle3.get_err(), crate::error::Error::Panic));

        assert_eq!(state.lock().unwrap().clone(), vec![0]);
    }

    #[test]
    fn can_cache_multiple_types() {
        let mut cache = Cache::new();
        let num_gen = Arc::new(Mutex::new(0));

        let num_gen_clone = num_gen.clone();
        let handle1 = cache.generate(Params1 { value: 5 }, move |_| {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            }
        });

        let handle2 = cache.generate(
            Params2 {
                variety: "block".to_string(),
                arc: Arc::new(Params1 { value: 20 }),
            },
            move |_| {
                *num_gen.lock().unwrap() += 1;
                Value {
                    inner: Arc::new(5),
                    extra: 50,
                }
            },
        );

        assert_eq!(
            handle1.get(),
            &Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20
            }
        );

        assert_eq!(
            handle2.get(),
            &Value {
                inner: Arc::new(5),
                extra: 50
            }
        );
    }

    #[test]
    #[should_panic]
    fn panics_on_mismatched_types() {
        let mut cache = Cache::new();

        let _ = cache.generate(Params1 { value: 5 }, |_| "cell".to_string());
        let _ = cache.generate(Params1 { value: 10 }, |_| 5);
    }

    #[test]
    fn cache_should_not_hang_on_panic() {
        let mut cache = Cache::new();

        let handle =
            cache.generate::<_, usize>(Params1 { value: 5 }, |_| panic!("panic during generation"));

        assert!(matches!(handle.get_err(), Error::Panic));
    }
}
