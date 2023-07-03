use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::{Opts, Spectre, Tran, TranOutput};
use substrate::block::Block;
use substrate::io::Signal;
use substrate::ios::TestbenchIo;
use substrate::pdk::Pdk;
use substrate::schematic::{Cell, HasSchematic, Instance};
use substrate::simulation::data::HasNodeData;
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
    type Data = Instance<Vdivider>;
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
        Ok(dut)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VDividerTbData {
    pub vdd: Vec<f64>,
    pub out: Vec<f64>,
}

impl<PDK: Pdk> Testbench<PDK, Spectre> for VdividerTb {
    type Output = VDividerTbData;
    fn run(
        &self,
        cell: &Cell<VdividerTb>,
        sim: substrate::simulation::SimController<Spectre>,
    ) -> Self::Output {
        let output = sim.simulate(
            Opts {},
            Tran {
                stop: dec!(1e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation");

        VDividerTbData {
            vdd: output.get_data(&cell.data().io().pwr.vdd.path()).unwrap().clone(),
            out: output.get_data(&cell.data().io().out.path()).unwrap().clone(),
        }
    }
}
