use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::blocks::Vsource;
use spectre::{Options, Spectre, Tran};
use substrate::block::Block;
use substrate::io::Signal;
use substrate::io::TestbenchIo;
use substrate::pdk::Pdk;
use substrate::schematic::{Cell, HasSchematic, Instance};
use substrate::simulation::data::HasNodeData;
use substrate::simulation::{HasTestbenchSchematicImpl, Testbench};

use crate::shared::vdivider::{Resistor, Vdivider, VdividerArray};

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
        cell: &mut substrate::schematic::TestbenchCellBuilder<PDK, Spectre, Self>,
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

        let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerTbData {
    pub vdd: Vec<f64>,
    pub out: Vec<f64>,
}

impl<PDK: Pdk> Testbench<PDK, Spectre> for VdividerTb {
    type Output = VdividerTbData;
    fn run(
        &self,
        cell: &Cell<VdividerTb>,
        sim: substrate::simulation::SimController<PDK, Spectre>,
    ) -> Self::Output {
        let output = sim
            .simulate(
                Options::default(),
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        VdividerTbData {
            vdd: output.get_data(&cell.data().io().pwr.vdd).unwrap().clone(),
            out: output.get_data(&cell.data().io().out).unwrap().clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VdividerArrayTb;

impl Block for VdividerArrayTb {
    type Io = TestbenchIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("vdivider_array_tb")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("vdivider_array_tb")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for VdividerArrayTb {
    type Data = Instance<VdividerArray>;
}

impl<PDK: Pdk> HasTestbenchSchematicImpl<PDK, Spectre> for VdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::TestbenchCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let vdd = cell.signal("vdd", Signal);
        // TODO: Use other resistor sizings once primitive parametrization is implemented.
        let dut = cell.instantiate(VdividerArray {
            vdividers: vec![
                Vdivider::new(300, 300),
                Vdivider::new(600, 600),
                Vdivider::new(800, 800),
            ],
        });

        for i in 0..3 {
            cell.connect(dut.io()[i].vdd, vdd);
            cell.connect(dut.io()[i].vss, io.vss);
        }

        let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerArrayTbData {
    pub expected: Vec<f64>,
    pub out: Vec<Vec<f64>>,
    pub out_nested: Vec<Vec<f64>>,
    pub vdd: Vec<f64>,
}

impl<PDK: Pdk> Testbench<PDK, Spectre> for VdividerArrayTb {
    type Output = VdividerArrayTbData;
    fn run(
        &self,
        cell: &Cell<VdividerArrayTb>,
        sim: substrate::simulation::SimController<PDK, Spectre>,
    ) -> Self::Output {
        let output = sim
            .simulate(
                Options::default(),
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let expected: Vec<_> = cell
            .data()
            .data()
            .into_iter()
            .map(|inst| {
                inst.block().r1.r as f64 / (inst.block().r1.r + inst.block().r2.r) as f64 * 1.8
            })
            .collect();

        let out = cell
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.io().out).unwrap().clone())
            .collect();

        let out_nested = cell
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.data().r1.io().n).unwrap().clone())
            .collect();

        let vdd = output
            .get_data(&cell.data().cell().io()[0].vdd)
            .unwrap()
            .clone();

        VdividerArrayTbData {
            expected,
            out,
            out_nested,
            vdd,
        }
    }
}
