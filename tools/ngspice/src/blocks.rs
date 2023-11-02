//! ngspice-specific blocks for use in testbenches.

use crate::{Ngspice, Primitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::{SchematicType, TwoTerminalIo};
use substrate::schematic::{PrimitiveBinding, PrimitiveSchematic};

/// Data associated with a pulse [`Vsource`].
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Pulse {
    /// The zero value of the pulse.
    pub val0: Decimal,
    /// The one value of the pulse.
    pub val1: Decimal,
    /// The period of the pulse.
    pub period: Option<Decimal>,
    /// Rise time.
    pub rise: Option<Decimal>,
    /// Fall time.
    pub fall: Option<Decimal>,
    /// The pulse width.
    pub width: Option<Decimal>,
    /// Waveform delay.
    pub delay: Option<Decimal>,
    /// Number of pulses.
    pub num_pulses: Option<Decimal>,
}

/// A voltage source.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "TwoTerminalIo", kind = "Primitive")]
pub enum Vsource {
    /// A dc voltage source.
    Dc(Decimal),
    /// A pulse voltage source.
    Pulse(Pulse),
}

impl Vsource {
    /// Creates a new DC voltage source.
    pub fn dc(value: Decimal) -> Self {
        Self::Dc(value)
    }

    /// Creates a new pulse voltage source.
    pub fn pulse(value: Pulse) -> Self {
        Self::Pulse(value)
    }
}

impl PrimitiveSchematic<Ngspice> for Vsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> PrimitiveBinding<Ngspice> {
        let mut prim = PrimitiveBinding::new(Primitive::Vsource(*self));
        prim.connect("P", io.p);
        prim.connect("N", io.n);
        prim
    }
}
