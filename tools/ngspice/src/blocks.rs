//! ngspice-specific blocks for use in testbenches.

use crate::{Ngspice, Primitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::TwoTerminalIo;
use substrate::schematic::primitives::DcVsource;
use substrate::schematic::{CellBuilder, ExportsNestedData, PrimitiveBinding, Schematic};

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

impl ExportsNestedData for Vsource {
    type NestedData = ();
}

impl Schematic<Ngspice> for Vsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Ngspice>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::Vsource(*self));
        prim.connect("P", io.p);
        prim.connect("N", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

impl Schematic<Ngspice> for DcVsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Ngspice>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        cell.instantiate_connected(Vsource::dc(self.value()), io);
        Ok(())
    }
}
