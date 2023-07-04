//! Interfaces for interacting with simulation data.

/// A simulation artifact with node data `V` that can be indexed by key `K`.
pub trait HasNodeData<K: ?Sized, V> {
    /// Gets data for key `k`.
    fn get_data(&self, k: &K) -> Option<&V>;
}
