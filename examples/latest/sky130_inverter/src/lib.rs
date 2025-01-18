// begin-code-snippet imports
use sky130pdk::mos::{Nfet01v8, Pfet01v8};
use sky130pdk::Sky130Pdk;
use substrate::block::Block;
use substrate::schematic::{CellBuilder, Schematic};
use substrate::types::{InOut, Input, Io, Output, Signal};
// end-code-snippet imports

pub mod layout;
pub mod tb;

// begin-code-snippet inverter-io
#[derive(Io, Clone, Default, Debug)]
pub struct InverterIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
}
// end-code-snippet inverter-io

// begin-code-snippet inverter-struct
#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "InverterIo")]
pub struct Inverter {
    /// NMOS width.
    pub nw: i64,
    /// PMOS width.
    pub pw: i64,
    /// Channel length.
    pub lch: i64,
}
// end-code-snippet inverter-struct

// begin-code-snippet inverter-schematic
impl Schematic for Inverter {
    type Schema = Sky130Pdk;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
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
// end-code-snippet inverter-schematic

pub const SKY130_MAGIC_TECH_FILE: &str =
    concat!(env!("OPEN_PDKS_ROOT"), "/sky130/magic/sky130.tech");
pub const SKY130_NETGEN_SETUP_FILE: &str =
    concat!(env!("OPEN_PDKS_ROOT"), "/sky130/netgen/sky130_setup.tcl");
