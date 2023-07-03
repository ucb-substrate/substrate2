//! Substrate's simulation API.

use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use impl_trait_for_tuples::impl_for_tuples;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::io::SchematicType;
use crate::pdk::Pdk;
use crate::schematic::{CellBuilder, HasSchematic};

/// A single simulator analysis.
pub trait Analysis {
    /// The output produced by this analysis.
    type Output;
}

/// A circuit simulator.
pub trait Simulator: Any + Send + Sync {
    /// The input type this simulator accepts.
    type Input;
    /// Options shared across all analyses for a given simulator run.
    type Options;
    /// The output type produced by this simulator.
    type Output;
    /// The error type returned by the simulator.
    type Error;

    /// Simulates the given set of analyses.
    fn simulate_inputs(
        &self,
        config: &SimulationConfig,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>, Self::Error>;

    /// Simulates the given, possibly composite, analysis.
    fn simulate<A>(
        &self,
        config: &SimulationConfig,
        options: Self::Options,
        input: A,
    ) -> Result<A::Output, Self::Error>
    where
        A: Analysis + SupportedBy<Self>,
        Self: Sized,
    {
        let mut inputs = Vec::new();
        input.into_input(&mut inputs);
        let output = self.simulate_inputs(config, options, inputs)?;
        let mut output = output.into_iter();
        Ok(A::from_output(&mut output))
    }
}

/// Substrate-defined simulation configuration.
pub struct SimulationConfig {
    /// The simulator's intended working directory.
    pub work_dir: PathBuf,
    /// The SCIR library to simulate.
    pub lib: scir::Library,
}

/// Indicates that a simulator supports a certain analysis.
pub trait Supports<A: Analysis>: Simulator {
    /// Convert this analysis into inputs accepted by the simulator.
    fn into_input(a: A, inputs: &mut Vec<Self::Input>);
    /// Convert the simulator outputs to this analysis's output.
    fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> A::Output;
}

/// Indicates that a particular analysis is supported by a simulator.
///
/// Where possible, prefer implementing [`Supports`].
pub trait SupportedBy<S: Simulator>: Analysis {
    /// Convert the analysis into inputs accepted by this simulator.
    fn into_input(self, inputs: &mut Vec<S::Input>);
    /// Convert this simulator's outputs to the analysis's expected output.
    fn from_output(outputs: &mut impl Iterator<Item = S::Output>) -> Self::Output;
}

impl<S, A> SupportedBy<S> for A
where
    A: Analysis,
    S: Supports<A>,
{
    fn into_input(self, inputs: &mut Vec<<S as Simulator>::Input>) {
        S::into_input(self, inputs);
    }
    fn from_output(outputs: &mut impl Iterator<Item = <S as Simulator>::Output>) -> Self::Output {
        S::from_output(outputs)
    }
}

/// Controls simulation options.
pub struct SimController<S> {
    pub(crate) simulator: Arc<S>,
    pub(crate) config: SimulationConfig,
}

impl<S: Simulator> SimController<S> {
    /// Run the given analysis, consuming this simulation controller.
    pub fn simulate<A: Analysis + SupportedBy<S>>(
        self,
        options: S::Options,
        input: A,
    ) -> Result<A::Output, S::Error> {
        self.simulator.simulate(&self.config, options, input)
    }
}

/// A testbench that can be simulated.
pub trait Testbench<PDK: Pdk, S: Simulator>: HasTestbenchSchematicImpl<PDK, S> {
    /// The output produced by this testbench.
    type Output: Any + Serialize + Deserialize<'static>;
    /// Run the testbench using the given simulation controller.
    fn run(&self, sim: SimController<S>) -> Self::Output;
}

/// A testbench block that has a schematic compatible with the given PDK and simulator.
pub trait HasTestbenchSchematicImpl<PDK: Pdk, S: Simulator>: Block + HasSchematic {
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Data,
        simulator: &S,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> crate::error::Result<Self::Data>;
}

#[impl_for_tuples(32)]
impl Analysis for Tuple {
    for_tuples!( type Output = ( #( Tuple::Output ),* ); );
}

macro_rules! support_tuple {
    ( $( ($t:ident, $idx:tt), )* ) => {
        impl<S, $( $t ),* > Supports<( $($t,)* )> for S
            where S: Simulator,
                  $( $t: Analysis + SupportedBy<S> ),*
        {
            #[allow(unused_variables)]
            fn into_input(a: ($($t,)*), inputs: &mut Vec<S::Input>) {
                $(
                    a.$idx.into_input(inputs);
                )*
            }
            #[allow(unused_variables)]
            fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> <($($t,)*) as Analysis>::Output {
                ($(
                    $t::from_output(outputs),
                )*)
            }
        }
    }
}

support_tuple!((T0, 0),);
support_tuple!((T0, 0), (T1, 1),);
support_tuple!((T0, 0), (T1, 1), (T2, 2),);
support_tuple!((T0, 0), (T1, 1), (T2, 2), (T3, 3),);
support_tuple!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4),);
support_tuple!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5),);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
    (T27, 27),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
    (T27, 27),
    (T28, 28),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
    (T27, 27),
    (T28, 28),
    (T29, 29),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
    (T27, 27),
    (T28, 28),
    (T29, 29),
    (T30, 30),
);
support_tuple!(
    (T0, 0),
    (T1, 1),
    (T2, 2),
    (T3, 3),
    (T4, 4),
    (T5, 5),
    (T6, 6),
    (T7, 7),
    (T8, 8),
    (T9, 9),
    (T10, 10),
    (T11, 11),
    (T12, 12),
    (T13, 13),
    (T14, 14),
    (T15, 15),
    (T16, 16),
    (T17, 17),
    (T18, 18),
    (T19, 19),
    (T20, 20),
    (T21, 21),
    (T22, 22),
    (T23, 23),
    (T24, 24),
    (T25, 25),
    (T26, 26),
    (T27, 27),
    (T28, 28),
    (T29, 29),
    (T30, 30),
    (T31, 31),
);
