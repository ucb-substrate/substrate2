//! Interfaces for interacting with simulation data.

use crate::io::{NestedNode, NodePath};
use crate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};

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

/// A simulation output that can be recovered from the output of a particular analysis.
pub trait FromSaved<S: Simulator, A: Analysis> {
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type Key;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(output: &A::Output, key: Self::Key) -> Self;
}

/// A simulation output that can be saved in an analysis within a given simulator.
///
/// `T` is any type that can be used as arguments for deciding what should be saved in
/// this simulation output.
pub trait Save<S: Simulator, A: Analysis + SupportedBy<S>, T>: FromSaved<S, A> {
    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(ctx: &SimulationContext, to_save: T, opts: &mut S::Options) -> Self::Key;
}
