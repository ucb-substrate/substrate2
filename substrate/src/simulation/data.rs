//! Interfaces for interacting with simulation data.

use crate::io::{FlatLen, Flatten, NestedNode, NodePath};
use crate::simulation::{Analysis, SimController, SimulationContext, Simulator};
use serde::{Deserialize, Serialize};

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

pub trait HasSaveKey {
    type SaveKey;
}

pub trait FromSaved<S: Simulator, A: Analysis>: HasSaveKey {
    fn from_saved(output: &mut A::Output, key: Self::SaveKey) -> Self;
}

pub trait Save<S: Simulator, A: Analysis, T>: FromSaved<S, A> {
    fn save(ctx: &SimulationContext, to_save: T, opts: &mut S::Options) -> Self::SaveKey;
}
