use crate::Sky130Pdk;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use spice::Spice;
use std::path::PathBuf;
use substrate::block::Block;
use substrate::io::{InOut, Input, Io, Output, SchematicType, Signal};
use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

impl Sky130Pdk {
    pub(crate) fn stdcell_path(&self, lib: &str, name: &str) -> PathBuf {
        self.open_root_dir
            .as_ref()
            .expect("Requires Sky130 open PDK root directory to be specified")
            .join(format!("libraries/{lib}/latest/cells/{name}"))
    }
}

#[derive(Default, Debug, Clone, Copy, Io)]
pub struct PowerIo {
    pub vgnd: InOut<Signal>,
    pub vpwr: InOut<Signal>,
    pub vnb: InOut<Signal>,
    pub vpb: InOut<Signal>,
}

impl PowerIoSchematic {
    pub fn with_bodies_tied_to_rails(pwr: substrate::io::PowerIoSchematic) -> Self {
        Self {
            vgnd: pwr.vss,
            vnb: pwr.vss,
            vpb: pwr.vdd,
            vpwr: pwr.vdd,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Io)]
pub struct And2Io {
    pub pwr: PowerIo,
    pub a: Input<Signal>,
    pub b: Input<Signal>,
    pub x: Output<Signal>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum And2 {
    S0,
    S1,
    S2,
    S4,
}

impl And2 {
    pub fn strength(&self) -> i64 {
        match self {
            And2::S0 => 0,
            And2::S1 => 1,
            And2::S2 => 2,
            And2::S4 => 4,
        }
    }
}

impl Block for And2 {
    type Io = And2Io;

    fn id() -> ArcStr {
        arcstr::literal!("and2")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("and2_{}", self.strength())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for And2 {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for And2 {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let pdk = cell
            .ctx()
            .get_installation::<Sky130Pdk>()
            .expect("Requires Sky130 PDK installation");

        let lib = "sky130_fd_sc_hd";
        let name = "and2";
        let cell_name = format!("{lib}__{name}_{}", self.strength());
        println!(
            "{:?}",
            pdk.stdcell_path(lib, name)
                .join(format!("{}.spice", cell_name))
        );
        let mut scir = Spice::scir_cell_from_file(
            pdk.stdcell_path(lib, name)
                .join(format!("{}.spice", cell_name)),
            &cell_name,
        )
        .convert_schema::<Sky130Pdk>()?;

        scir.connect("A", io.a);
        scir.connect("B", io.b);
        scir.connect("VGND", io.pwr.vgnd);
        scir.connect("VNB", io.pwr.vnb);
        scir.connect("VPB", io.pwr.vpb);
        scir.connect("VPWR", io.pwr.vpwr);
        scir.connect("X", io.x);

        cell.set_scir(scir);
        Ok(())
    }
}
