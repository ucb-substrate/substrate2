//! ngspice-specific blocks for use in testbenches.

use crate::{Ngspice, Primitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::schematic::{CellBuilder, PrimitiveBinding, Schematic};
use substrate::types::TwoTerminalIo;

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
#[substrate(io = "TwoTerminalIo")]
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

impl Schematic for Vsource {
    type Schema = Ngspice;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::Vsource(*self));
        prim.connect("P", io.p);
        prim.connect("N", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// A current source.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "TwoTerminalIo")]
pub enum Isource {
    /// A dc current source.
    Dc(Decimal),
    /// A pulse current source.
    Pulse(Pulse),
}

impl Isource {
    /// Creates a new DC current source.
    pub fn dc(value: Decimal) -> Self {
        Self::Dc(value)
    }

    /// Creates a new pulse current source.
    pub fn pulse(value: Pulse) -> Self {
        Self::Pulse(value)
    }
}

impl Schematic for Isource {
    type Schema = Ngspice;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::Isource(*self));
        prim.connect("P", io.p);
        prim.connect("N", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}
