//! A library for assigning unique names.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

/// A set of unique names.
///
/// Each key of type `K` is assigned a unique name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Names<K: Hash + Eq> {
    names: HashSet<ArcStr>,
    assignments: HashMap<K, ArcStr>,
}

impl<K: Hash + Eq> Default for Names<K> {
    fn default() -> Self {
        Self {
            names: HashSet::new(),
            assignments: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq> Names<K> {
    /// Creates a new, empty name set.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new, empty name set with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            names: HashSet::with_capacity(capacity),
            assignments: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the name associated with this key, if it exists.
    #[inline]
    pub fn name(&self, id: &K) -> Option<ArcStr> {
        self.assignments.get(id).cloned()
    }

    /// Attempts to assign the given name to key `id`.
    ///
    /// If the name is not currently in use, it is assigned to the key `id` and `true` is returned.
    /// If the name is already in use, `false` is returned and no changes are made.
    pub fn reserve_name(&mut self, id: K, name: impl Into<ArcStr>) -> bool {
        let name = name.into();
        if !self.names.insert(name.clone()) {
            false
        } else {
            self.assignments.insert(id, name);
            true
        }
    }

    /// Allocates a new, unique name associated with the given ID.
    ///
    /// The name will be based on the given `base_name`.
    pub fn assign_name(&mut self, id: K, base_name: &str) -> ArcStr {
        let name = if self.names.contains(base_name) {
            let mut i = 1;
            loop {
                let new_name = arcstr::format!("{}_{}", base_name, i);
                if !self.names.contains(&new_name) {
                    break new_name;
                }
                i += 1;
            }
        } else {
            base_name.into()
        };

        self.names.insert(name.clone());
        self.assignments.insert(id, name.clone());
        name
    }
}
