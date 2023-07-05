use serde::{Deserialize, Serialize};
use sky130pdk::mos::{Nfet01v8, Pfet01v8};
use sky130pdk::Sky130Pdk;
use substrate::block::Block;
use substrate::io::{InOut, Input, Output, Signal};
use substrate::schematic::{HasSchematic, HasSchematicImpl};
use substrate::Io;

pub mod tb;

#[derive(Io, Clone, Default, Debug)]
pub struct InverterIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    /// NMOS width.
    pub nw: i64,
    /// PMOS width.
    pub pw: i64,
    /// Channel length.
    pub lch: i64,
}

impl Block for Inverter {
    type Io = InverterIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("sky130_inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("sky130_inverter")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Inverter {
    type Data = ();
}

impl HasSchematicImpl<Sky130Pdk> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
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
