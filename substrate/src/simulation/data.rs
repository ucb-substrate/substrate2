//! Interfaces for interacting with simulation data.

pub use codegen::FromSaved;

use crate::schematic::Schematic;
use crate::simulation::{Analysis, SimulationContext, Simulator};

/// Gets the [`Save::SaveKey`] corresponding to type `T`.
pub type SaveKey<T, S, A> = <T as Save<S, A>>::SaveKey;

/// A simulation output that can be saved in an analysis within a given simulator.
///
/// `T` is any type that can be used as arguments for deciding what should be saved in
/// this simulation output.
pub trait Save<S: Simulator, A: Analysis> {
    type Save;
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type SaveKey;

    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Save;
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
