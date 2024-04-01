//! Standard APIs for setting simulator options.

use crate::simulation::{SimulationContext, Simulator};
use rust_decimal::Decimal;
use std::ops::{Deref, DerefMut};

/// An option for a simulator.
pub trait SimOption<S: Simulator> {
    /// Modifies the simulator's options to enable this option.
    fn set_option(self, opts: &mut <S as Simulator>::Options, ctx: &SimulationContext<S>);
}

/// A temperature to use in simulation.
pub struct Temperature(Decimal);

impl Deref for Temperature {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Temperature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Decimal> for Temperature {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

/// Initial conditions.
pub mod ic {
    use crate::io::schematic::{NestedNode, NestedTerminal, NodePath, TerminalPath};
    use crate::simulation::{SimulationContext, Simulator};
    use rust_decimal::Decimal;
    use std::ops::{Deref, DerefMut};
    use substrate::simulation::options::SimOption;
    use type_dispatch::impl_dispatch;

    /// An initial condition.
    pub struct InitialCondition<K, V> {
        /// A path referring to the item whose initial condition needs to be set.
        pub path: K,
        /// An initial condition that should be set at the above path.
        pub value: V,
    }

    /// An initial voltage value.
    pub struct Voltage(pub Decimal);

    impl Deref for Voltage {
        type Target = Decimal;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for Voltage {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[impl_dispatch({NestedNode; &NestedNode})]
    impl<N, V, S: Simulator> SimOption<S> for InitialCondition<N, V>
    where
        InitialCondition<NodePath, V>: SimOption<S>,
    {
        fn set_option(self, opts: &mut <S as Simulator>::Options, ctx: &SimulationContext<S>) {
            InitialCondition {
                path: self.path.path(),
                value: self.value,
            }
            .set_option(opts, ctx)
        }
    }

    #[impl_dispatch({TerminalPath; &TerminalPath})]
    impl<N, V, S: Simulator> SimOption<S> for InitialCondition<N, V>
    where
        for<'a> InitialCondition<&'a NodePath, V>: SimOption<S>,
    {
        fn set_option(self, opts: &mut <S as Simulator>::Options, ctx: &SimulationContext<S>) {
            InitialCondition {
                path: self.path.as_ref(),
                value: self.value,
            }
            .set_option(opts, ctx)
        }
    }

    #[impl_dispatch({NestedTerminal; &NestedTerminal})]
    impl<T, V, S: Simulator> SimOption<S> for InitialCondition<T, V>
    where
        InitialCondition<TerminalPath, V>: SimOption<S>,
    {
        fn set_option(self, opts: &mut <S as Simulator>::Options, ctx: &SimulationContext<S>) {
            InitialCondition {
                path: self.path.path(),
                value: self.value,
            }
            .set_option(opts, ctx)
        }
    }
}
