//! Interfaces for interacting with simulation data.

pub use codegen::FromSaved;
use substrate::schematic::ExportsNestedData;

use crate::schematic::Cell;
use crate::simulation::{Analysis, SimulationContext, Simulator};

/// A simulation output that can be recovered from the output of a particular analysis.
pub trait FromSaved<S: Simulator, A: Analysis> {
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type SavedKey;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(output: &<A as Analysis>::Output, key: &Self::SavedKey) -> Self;
}

impl<S, A1, A2, O1, O2> FromSaved<S, (A1, A2)> for (O1, O2)
where
    S: Simulator,
    A1: Analysis,
    A2: Analysis,
    O1: FromSaved<S, A1>,
    O2: FromSaved<S, A2>,
    (A1, A2): Analysis<Output = (A1::Output, A2::Output)>,
{
    type SavedKey = (
        <O1 as FromSaved<S, A1>>::SavedKey,
        <O2 as FromSaved<S, A2>>::SavedKey,
    );

    fn from_saved(output: &<(A1, A2) as Analysis>::Output, key: &Self::SavedKey) -> Self {
        let o1 = <O1 as FromSaved<S, A1>>::from_saved(&output.0, &key.0);
        let o2 = <O2 as FromSaved<S, A2>>::from_saved(&output.1, &key.1);
        (o1, o2)
    }
}

/// Gets the [`FromSaved::SavedKey`] corresponding to type `T`.
pub type SavedKey<S, A, T> = <T as FromSaved<S, A>>::SavedKey;

impl<S: Simulator, A: Analysis, T: FromSaved<S, A>> FromSaved<S, A> for Vec<T> {
    type SavedKey = Vec<<T as FromSaved<S, A>>::SavedKey>;

    fn from_saved(output: &<A as Analysis>::Output, key: &Self::SavedKey) -> Self {
        key.iter().map(|key| T::from_saved(output, key)).collect()
    }
}

/// A simulation output that can be saved in an analysis within a given simulator.
///
/// `T` is any type that can be used as arguments for deciding what should be saved in
/// this simulation output.
pub trait Save<S: Simulator, A: Analysis, T>: FromSaved<S, A> {
    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(
        ctx: &SimulationContext<S>,
        to_save: T,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as FromSaved<S, A>>::SavedKey;
}

/// A testbench that can save data of type `T`.
pub trait SaveTb<S: Simulator, A: Analysis, T: FromSaved<S, A>>: ExportsNestedData {
    /// Saves data `T` from cell `cell`.
    fn save_tb(
        ctx: &SimulationContext<S>,
        cell: &Cell<Self>,
        opts: &mut <S as Simulator>::Options,
    ) -> <T as FromSaved<S, A>>::SavedKey;
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

/// AC data definitions.
pub mod ac {
    use num::complex::Complex64;
    use serde::{Deserialize, Serialize};
    use std::ops::Deref;
    use std::sync::Arc;

    /// A series of voltage vs frequency measurements from an AC simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Voltage(pub Arc<Vec<Complex64>>);

    impl Deref for Voltage {
        type Target = Vec<Complex64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// A series of current vs frequency measurements from an AC simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Current(pub Arc<Vec<Complex64>>);

    impl Deref for Current {
        type Target = Vec<Complex64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// The frequency points associated with an AC simulation.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Freq(pub Arc<Vec<f64>>);

    impl Deref for Freq {
        type Target = Vec<f64>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}
