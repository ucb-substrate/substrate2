//! Interfaces for interacting with simulation data.

use std::{any::Any, ops::Deref, sync::Arc};

use scir::NodePath;

use super::Simulator;

/// A simulation artifact with node data `V` that can be indexed by key `K`.
pub trait HasNodeData<K: ?Sized, V> {
    /// Gets data for key `k`.
    fn get_data(&self, k: &K) -> Option<&V>;
}
