//! Interfaces for interacting with simulation data.

pub use codegen::FromSaved;
use type_dispatch::impl_dispatch;

use crate::io::{NestedNode, NestedTerminal, NodePath, TerminalPath};
use crate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};

/// A simulation output that can be recovered from the output of a particular analysis.
pub trait FromSaved<S: Simulator, A: Analysis> {
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type Key;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(output: &<A as Analysis>::Output, key: Self::Key) -> Self;
}

impl<S: Simulator, A: Analysis, T: FromSaved<S, A>> FromSaved<S, A> for Vec<T> {
    type Key = Vec<<T as FromSaved<S, A>>::Key>;

    fn from_saved(output: &<A as Analysis>::Output, key: Self::Key) -> Self {
        key.into_iter()
            .map(|key| T::from_saved(output, key))
            .collect()
    }
}

/// A simulation output that can be saved in an analysis within a given simulator.
///
/// `T` is any type that can be used as arguments for deciding what should be saved in
/// this simulation output.
pub trait Save<S: Simulator, A: Analysis + SupportedBy<S>, T>: FromSaved<S, A> {
    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(
        ctx: &SimulationContext<S>,
        to_save: T,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key;
}

#[impl_dispatch({NestedNode; &NestedNode})]
impl<N, S: Simulator, A: Analysis + SupportedBy<S>, T: Save<S, A, NodePath>> Save<S, A, N> for T {
    fn save(
        ctx: &SimulationContext<S>,
        to_save: N,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, to_save.path(), opts)
    }
}

#[impl_dispatch({TerminalPath; &TerminalPath})]
impl<N, S: Simulator, A: Analysis + SupportedBy<S>, T: for<'a> Save<S, A, &'a NodePath>>
    Save<S, A, N> for T
{
    fn save(
        ctx: &SimulationContext<S>,
        to_save: N,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, to_save.as_ref(), opts)
    }
}

#[impl_dispatch({NestedTerminal; &NestedTerminal})]
impl<N, S: Simulator, A: Analysis + SupportedBy<S>, T: Save<S, A, TerminalPath>> Save<S, A, N>
    for T
{
    fn save(
        ctx: &SimulationContext<S>,
        to_save: N,
        opts: &mut <S as Simulator>::Options,
    ) -> Self::Key {
        T::save(ctx, to_save.path(), opts)
    }
}

/// Transient data definitions.
pub mod tran {
    use serde::{Deserialize, Serialize};
    use std::ops::Deref;
    use std::sync::Arc;

    /// A time-series of voltage measurements from a transient simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Voltage(pub Arc<Vec<f64>>);

    impl Deref for Voltage {
        type Target = Vec<f64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// A time-series of current measurements from a transient simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Current(pub Arc<Vec<f64>>);

    impl Deref for Current {
        type Target = Vec<f64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// The time points associated with a transient simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Time(pub Arc<Vec<f64>>);

    impl Deref for Time {
        type Target = Vec<f64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
