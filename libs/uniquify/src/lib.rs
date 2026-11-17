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
    /// The suffix at which [`Names::assign_name`] starts searching for a free name,
    /// per base name.
    ///
    /// Every lower suffix is in use. This only speeds up the search, so it is
    /// neither serialized nor compared.
    #[serde(skip)]
    next_suffix: HashMap<ArcStr, usize>,
}

impl<K: Hash + Eq> Default for Names<K> {
    fn default() -> Self {
        Self {
            names: HashSet::new(),
            assignments: HashMap::new(),
            next_suffix: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq> PartialEq for Names<K> {
    fn eq(&self, other: &Self) -> bool {
        self.names == other.names && self.assignments == other.assignments
    }
}

impl<K: Hash + Eq> Eq for Names<K> {}

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
            next_suffix: HashMap::new(),
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
            // Resume where the previous search for this base name stopped. Scanning from 1
            // every time is quadratic in the number of keys sharing a base name.
            let i = self.next_suffix.entry(base_name.into()).or_insert(1);
            loop {
                let new_name = arcstr::format!("{}_{}", base_name, i);
                *i += 1;
                if !self.names.contains(&new_name) {
                    break new_name;
                }
            }
        } else {
            base_name.into()
        };

        self.names.insert(name.clone());
        self.assignments.insert(id, name.clone());
        name
    }

    /// Unassigns the name associated to `id`.
    ///
    /// Returns true if `id` was unbound.
    /// Returns false if `id` did not have an assigned name to unbind.
    pub fn unassign(&mut self, id: &K) -> bool {
        if let Some(name) = self.assignments.remove(id) {
            self.names.remove(&name);
            // If `name` has the form `{base}_{i}`, suffix `i` is free again for `base`.
            if let Some((base, suffix)) = name.rsplit_once('_')
                && let (Some(next), Ok(i)) = (self.next_suffix.get_mut(base), suffix.parse())
            {
                *next = std::cmp::min(*next, i);
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_name_reuses_unassigned_suffixes() {
        let mut names = Names::new();
        assert_eq!(names.assign_name(0, "a"), "a");
        assert_eq!(names.assign_name(1, "a"), "a_1");
        assert_eq!(names.assign_name(2, "a"), "a_2");
        assert!(names.reserve_name(3, "a_3"));
        assert_eq!(names.assign_name(4, "a"), "a_4");
        assert!(names.unassign(&1));
        assert_eq!(names.assign_name(5, "a"), "a_1");
        assert_eq!(names.assign_name(6, "a"), "a_5");
    }
}
