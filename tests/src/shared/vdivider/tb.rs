use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::{Opts, Spectre, Tran};
use substrate::block::Block;
use substrate::io::Signal;
use substrate::ios::TestbenchIo;
use substrate::pdk::Pdk;
use substrate::schematic::HasSchematic;
use substrate::simulation::{HasTestbenchSchematicImpl, Testbench};

use crate::shared::vdivider::{Resistor, Vdivider};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VdividerTb;

impl Block for VdividerTb {
    type Io = TestbenchIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("vdivider_tb")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("vdivider_tb")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for VdividerTb {
    type Data = ();
}

impl<PDK: Pdk> HasTestbenchSchematicImpl<PDK, Spectre> for VdividerTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        _simulator: &Spectre,
        cell: &mut substrate::schematic::CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell.instantiate(Vdivider {
            r1: Resistor { r: 20 },
            r2: Resistor { r: 20 },
        });
        cell.connect(dut.io().pwr.vdd, vdd);
        cell.connect(dut.io().pwr.vss, io.vss);
        cell.connect(dut.io().out, out);
        Ok(())
    }
}

impl<PDK: Pdk> Testbench<PDK, Spectre> for VdividerTb {
    type Output = ();
    fn run(&self, sim: substrate::simulation::SimController<Spectre>) -> Self::Output {
        sim.simulate(
            Opts {},
            Tran {
                stop: dec!(1e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation");
    }
}
