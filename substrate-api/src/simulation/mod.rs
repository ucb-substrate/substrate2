use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use impl_trait_for_tuples::impl_for_tuples;
use serde::{Deserialize, Serialize};

pub trait Analysis {
    type Output;
}

pub trait Simulator: Send + Sync {
    type Input;
    type Options;
    type Output;
    fn simulate_inputs(
        &self,
        config: &SimulationConfig,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Vec<Self::Output>;

    fn simulate<A>(&self, config: &SimulationConfig, options: Self::Options, input: A) -> A::Output
    where
        A: Analysis + SupportedBy<Self>,
        Self: Sized,
    {
        let mut inputs = Vec::new();
        input.into_input(&mut inputs);
        let output = self.simulate_inputs(config, options, inputs);
        let mut output = output.into_iter();
        A::from_output(&mut output)
    }
}

pub struct SimulationConfig {
    pub work_dir: PathBuf,
    // TODO: SCIR Library
}

pub trait Supports<A: Analysis>: Simulator {
    fn into_input(a: A, inputs: &mut Vec<Self::Input>);
    fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> A::Output;
}

pub trait SupportedBy<S: Simulator>: Analysis {
    fn into_input(self, inputs: &mut Vec<S::Input>);
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

pub struct SimController<S> {
    simulator: Arc<S>,
    config: SimulationConfig,
}

impl<S: Simulator> SimController<S> {
    pub fn simulate<A: Analysis + SupportedBy<S>>(
        self,
        options: S::Options,
        input: A,
    ) -> A::Output {
        self.simulator.simulate(&self.config, options, input)
    }
}

pub trait Testbench<PDK, S: Simulator> {
    type Output: Any + Serialize + Deserialize<'static>;
    fn run(&self, sim: SimController<S>) -> Self::Output;
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
