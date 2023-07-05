//! Interfaces for interacting with simulation data.

use crate::io::{NestedNode, NodePath};

/// A simulation artifact with node data `V` that can be indexed by key `K`.
pub trait HasNodeData<K: ?Sized, V> {
    /// Gets data for key `k`.
    fn get_data(&self, k: &K) -> Option<&V>;
}

impl<D, T: HasNodeData<NodePath, D>> HasNodeData<NestedNode, D> for T {
    fn get_data(&self, k: &NestedNode) -> Option<&D> {
        self.get_data(&k.path())
    }
}
