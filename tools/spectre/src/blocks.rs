//! Spectre-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::ios::VsourceIo;
use substrate::pdk::Pdk;
use substrate::schematic::HasSchematic;
use substrate::simulation::HasTestbenchSchematicImpl;

use crate::Spectre;

/// A voltage source.
///
/// Currently only spuports DC values.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Vsource {
    dc: Decimal,
}

impl Vsource {
    /// Creates a new DC voltage source.
    pub fn dc(value: Decimal) -> Self {
        Self { dc: value }
    }
}

impl Block for Vsource {
    type Io = VsourceIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("vsource")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("vsource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Vsource {
    type Data = ();
}

impl<PDK: Pdk> HasTestbenchSchematicImpl<PDK, Spectre> for Vsource {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::TestbenchCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.set_blackbox(arcstr::format!("V0 ( io_p io_n ) vsource dc={}", self.dc));
        Ok(())
    }
}
