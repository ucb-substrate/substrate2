//! Interfaces for interacting with simulation data.

pub use codegen::FromSaved;
use type_dispatch::impl_dispatch;

use crate::io::{NestedNode, NodePath, Terminal, TerminalPath};
use crate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};

/// A simulation artifact with node data `V` that can be indexed by key `K`.
pub trait HasSimData<K: ?Sized, V> {
    /// Gets data for key `k`.
    fn get_data(&self, k: &K) -> Option<&V>;
}

impl<D, T: HasSimData<NodePath, D>> HasSimData<NestedNode, D> for T {
    fn get_data(&self, k: &NestedNode) -> Option<&D> {
        self.get_data(&k.path())
    }
}

#[impl_dispatch({NodePath, TerminalPath; NestedNode, Terminal})]
impl<N1, N2, D, T: HasSimData<N1, D>> HasSimData<N2, D> for T {
    fn get_data(&self, k: &N2) -> Option<&D> {
        self.get_data(k.as_ref())
    }
}

/// A simulation output that can be recovered from the output of a particular analysis.
pub trait FromSaved<S: Simulator, A: Analysis> {
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type Key;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(output: &<A as Analysis>::Output, key: Self::Key) -> Self;
}

/// A simulation output that can be saved in an analysis within a given simulator.
///
/// `T` is any type that can be used as arguments for deciding what should be saved in
/// this simulation output.
pub trait Save<S: Simulator, A: Analysis + SupportedBy<S>, T>: FromSaved<S, A> {
    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(ctx: &SimulationContext, to_save: T, opts: &mut <S as Simulator>::Options)
        -> Self::Key;
}

#[impl_dispatch({NestedNode; &NestedNode})]
impl<N, S: Simulator, A: Analysis + SupportedBy<S>, T: Save<S, A, NodePath>> Save<S, A, N> for T {
    fn save(
        ctx: &SimulationContext,
        to_save: N,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, to_save.path(), opts)
    }
}

#[impl_dispatch(&'a NodePath, {TerminalPath; &TerminalPath})]
impl<N1, N2, S: Simulator, A: Analysis + SupportedBy<S>, T: for<'a> Save<S, A, N1>> Save<S, A, N2>
    for T
{
    fn save(
        ctx: &SimulationContext,
        to_save: N2,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, to_save.as_ref(), opts)
    }
}

#[impl_dispatch(&'a TerminalPath, {Terminal; &Terminal})]
impl<N1, N2, S: Simulator, A: Analysis + SupportedBy<S>, T: for<'a> Save<S, A, N1>> Save<S, A, N2>
    for T
{
    fn save(
        ctx: &SimulationContext,
        to_save: N2,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, &to_save.path(), opts)
    }
}
