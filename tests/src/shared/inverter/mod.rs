use serde::{Deserialize, Serialize};
use sky130pdk::mos::{Nfet01v8, Pfet01v8};
use sky130pdk::Sky130Pdk;
use substrate::block::{self, Block};
use substrate::io::{InOut, Input, Io, Output, SchematicType, Signal};
use substrate::pdk::{Pdk, PdkSchematic, ToSchema};
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, ExportsNestedNodes, PdkCellBuilder, Schematic};

pub mod tb;
#[derive(Io, Clone, Default, Debug)]
pub struct InverterIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
}

#[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "InverterIo", kind = "block::PdkCell")]
pub struct Inverter {
    /// NMOS width.
    pub nw: i64,
    /// PMOS width.
    pub pw: i64,
    /// Channel length.
    pub lch: i64,
}

impl ExportsNestedNodes for Inverter {
    type NestedNodes = ();
}

impl PdkSchematic<Sky130Pdk> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedNodes> {
        let nmos = cell.instantiate(Nfet01v8::new((self.nw, self.lch)));
        cell.connect(io.dout, nmos.io().d);
        cell.connect(io.din, nmos.io().g);
        cell.connect(io.vss, nmos.io().s);
        cell.connect(io.vss, nmos.io().b);

        let pmos = cell.instantiate(Pfet01v8::new((self.pw, self.lch)));
        cell.connect(io.dout, pmos.io().d);
        cell.connect(io.din, pmos.io().g);
        cell.connect(io.vdd, pmos.io().s);
        cell.connect(io.vdd, pmos.io().b);

        Ok(())
    }
}
