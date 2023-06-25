//! Utilities for generating values in the background and caching them based on hashable keys.

use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    sync::{Arc, Mutex},
    thread,
};

use once_cell::sync::OnceCell;

use crate::error::Error;

/// An abstraction for generating values in the background and caching them
/// based on hashable keys.
#[derive(Default, Debug, Clone)]
pub(crate) struct Generator {
    /// A map from key type to another map from key to value handle.
    ///
    /// Effectively, the type of this map is `TypeId::of::<K>() -> HashMap<K, Arc<OnceCell<V>>`.
    cells: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl Generator {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Ensures that a value corresponding to `key` is generated, using `generate_fn`
    /// to generate it if it has not already been generated.
    ///
    /// Returns a handle to the value. If the value is not yet generated, it is generated
    /// in the background.
    ///
    /// # Panics
    ///
    /// Panics if a different type `V` is already associated with type `K`.
    pub(crate) fn generate<K: Hash + Eq + Any + Send + Sync, V: Send + Sync + Any>(
        &mut self,
        key: K,
        generate_fn: impl FnOnce() -> V + Send + Any,
    ) -> Arc<OnceCell<V>> {
        let entry = self
            .cells
            .entry(TypeId::of::<K>())
            .or_insert(Arc::new(Mutex::<HashMap<K, Arc<OnceCell<V>>>>::default()));

        let mut entry_locked = entry.lock().unwrap();

        let entry = entry_locked
            .downcast_mut::<HashMap<K, Arc<OnceCell<V>>>>()
            .unwrap()
            .entry(key);

        match entry {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let cell = v.insert(Arc::new(OnceCell::new()));

                let cell2 = cell.clone();

                thread::spawn(move || {
                    let value = generate_fn();
                    cell2.set(value).map_err(|_| Error::Internal).unwrap()
                });

                cell.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::unbounded;

    use super::Generator;

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct Params1 {
        value: usize,
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct Params2 {
        variety: String,
        arc: Arc<Params1>,
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct Value<T> {
        inner: Arc<T>,
        extra: usize,
    }

    #[test]
    fn generator_generates_in_background_and_caches_values() {
        let mut generator = Generator::new();
        let num_gen = Arc::new(Mutex::new(0));
        let (s, r) = unbounded();

        let num_gen_clone = num_gen.clone();
        let handle1 = generator.generate(Params1 { value: 5 }, move || {
            *num_gen_clone.lock().unwrap() += 1;
            r.recv().unwrap();
            Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            }
        });

        // Should not use this generation function as the corresponding block is already being generated.
        let num_gen_clone = num_gen.clone();
        let handle2 = generator.generate(Params1 { value: 5 }, move || {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("circuit".to_string()),
                extra: 50,
            }
        });

        assert_eq!(handle1.get(), None);
        assert_eq!(handle2.get(), None);

        s.send(()).unwrap();

        assert_eq!(
            handle1.wait(),
            &Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            }
        );

        // Should reference the same cell as `handle1`.
        assert_eq!(
            handle2.get(),
            Some(&Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            })
        );

        // Should immediately return a filled cell as this has already been generated.
        let num_gen_clone = num_gen.clone();
        let handle3 = generator.generate(Params1 { value: 5 }, move || {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("circuit".to_string()),
                extra: 50,
            }
        });

        assert_eq!(
            handle3.get(),
            Some(&Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            })
        );

        // Should generate a new block as it has not been generated with the provided parameters
        // yet.
        let num_gen_clone = num_gen.clone();
        let handle4 = generator.generate(Params1 { value: 10 }, move || {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("circuit".to_string()),
                extra: 50,
            }
        });

        assert_eq!(
            handle4.wait(),
            &Value {
                inner: Arc::new("circuit".to_string()),
                extra: 50,
            }
        );

        assert_eq!(*num_gen.lock().unwrap(), 2);
    }

    #[test]
    fn generator_can_cache_multiple_types() {
        let mut generator = Generator::new();
        let num_gen = Arc::new(Mutex::new(0));

        let num_gen_clone = num_gen.clone();
        let handle1 = generator.generate(Params1 { value: 5 }, move || {
            *num_gen_clone.lock().unwrap() += 1;
            Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20,
            }
        });

        let handle2 = generator.generate(
            Params2 {
                variety: "block".to_string(),
                arc: Arc::new(Params1 { value: 20 }),
            },
            move || {
                *num_gen.lock().unwrap() += 1;
                Value {
                    inner: Arc::new(5),
                    extra: 50,
                }
            },
        );

        assert_eq!(
            handle1.wait(),
            &Value {
                inner: Arc::new("substrate".to_string()),
                extra: 20
            }
        );

        assert_eq!(
            handle2.wait(),
            &Value {
                inner: Arc::new(5),
                extra: 50
            }
        );
    }

    #[test]
    #[should_panic]
    fn generator_panics_on_mismatched_types() {
        let mut generator = Generator::new();

        let _ = generator.generate(Params1 { value: 5 }, || "cell".to_string());
        let _ = generator.generate(Params1 { value: 10 }, || 5);
    }
}
