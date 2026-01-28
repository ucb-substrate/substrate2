use std::path::PathBuf;

// begin-code-snippet imports
use sky130::Sky130;
use sky130::mos::{Nfet01v8, Pfet01v8};
use substrate::block::Block;
use substrate::context::Context;
use substrate::error::Result;
use substrate::schematic::{CellBuilder, Schematic};
use substrate::types::schematic::IoNodeBundle;
use substrate::types::{InOut, Input, Io, Output, Signal};
// end-code-snippet imports

pub mod atoll;
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
}
// end-code-snippet inverter-struct

// begin-code-snippet inverter-schematic
impl Schematic for Inverter {
    type Schema = Sky130;
    type NestedData = ();
    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> Result<Self::NestedData> {
        let nmos = cell.instantiate(Nfet01v8::new((self.nw, 150)));
        cell.connect(io.dout, nmos.io().d);
        cell.connect(io.din, nmos.io().g);
        cell.connect(io.vss, nmos.io().s);
        cell.connect(io.vss, nmos.io().b);

        let pmos = cell.instantiate(Pfet01v8::new((self.pw, 150)));
        cell.connect(io.dout, pmos.io().d);
        cell.connect(io.din, pmos.io().g);
        cell.connect(io.vdd, pmos.io().s);
        cell.connect(io.vdd, pmos.io().b);

        Ok(())
    }
}
// end-code-snippet inverter-schematic

// begin-code-snippet open-constants
pub fn open_pdks_root() -> PathBuf {
    PathBuf::from(
        std::env::var("OPEN_PDKS_ROOT")
            .expect("OPEN_PDKS_ROOT environment variable must be defined"),
    )
}
pub fn sky130_magic_tech_file() -> PathBuf {
    open_pdks_root().join("sky130/magic/sky130.tech")
}
pub fn sky130_netgen_setup_file() -> PathBuf {
    open_pdks_root().join("sky130/netgen/sky130_setup.tcl")
}
// end-code-snippet open-constants

// begin-code-snippet cds-constants
pub fn sky130_cds_pdk_root() -> PathBuf {
    PathBuf::from(
        std::env::var("SKY130_CDS_PDK_ROOT")
            .expect("SKY130_CDS_PDK_ROOT environment variable must be defined"),
    )
}
pub fn sky130_drc() -> PathBuf {
    sky130_cds_pdk_root().join("Sky130_DRC")
}
pub fn sky130_drc_rules_path() -> PathBuf {
    sky130_drc().join("sky130_rev_0.0_1.0.drc.pvl")
}
pub fn sky130_lvs() -> PathBuf {
    sky130_cds_pdk_root().join("Sky130_LVS")
}
pub fn sky130_lvs_rules_path() -> PathBuf {
    sky130_lvs().join("sky130.lvs.pvl")
}
pub fn sky130_technology_dir() -> PathBuf {
    sky130_cds_pdk_root().join("quantus/extraction/typical")
}
// end-code-snippet cds-constants

// begin-code-snippet sky130-open-ctx
/// Create a new Substrate context for the SKY130 open PDK.
///
/// Sets the PDK root to the value of the `SKY130_OPEN_PDK_ROOT`
/// environment variable and installs ngspice with default configuration.
///
/// # Panics
///
/// Panics if the `SKY130_OPEN_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_open_ctx() -> Context {
    let pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("SKY130_OPEN_PDK_ROOT environment variable must be defined");
    Context::builder()
        .install(ngspice::Ngspice::default())
        .install(Sky130::open(pdk_root))
        .build()
}
// end-code-snippet sky130-open-ctx

// begin-code-snippet sky130-cds-ctx
/// Create a new Substrate context for the SKY130 Cadence PDK.
///
/// Sets the PDK root to the value of the `SKY130_CDS_PDK_ROOT`
/// environment variable and installs Spectre with default configuration.
///
/// # Panics
///
/// Panics if the `SKY130_CDS_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_cds_ctx() -> Context {
    let pdk_root = sky130_cds_pdk_root();
    Context::builder()
        .install(spectre::Spectre::default())
        .install(Sky130::cds_only(pdk_root))
        .build()
}
// end-code-snippet sky130-cds-ctx
