use rust_decimal::prelude::ToPrimitive;
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
use substrate::Block;

use crate::shared::vdivider::{Resistor, Vdivider, VdividerArray};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerTb;

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
            r1: Resistor::new(20),
            r2: Resistor::new(20),
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
            .simulate_without_corner(
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerArrayTb;

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
        let dut = cell.instantiate(VdividerArray {
            vdividers: vec![
                Vdivider::new(300, 300),
                Vdivider::new(600, 800),
                Vdivider::new(3600, 1600),
            ],
        });

        for i in 0..3 {
            cell.connect(dut.io().elements[i].vdd, vdd);
            cell.connect(dut.io().elements[i].vss, io.vss);
        }

        let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct FlattenedVdividerArrayTb;

impl HasSchematic for FlattenedVdividerArrayTb {
    type Data = Instance<super::flattened::VdividerArray>;
}

impl<PDK: Pdk> HasTestbenchSchematicImpl<PDK, Spectre> for FlattenedVdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::TestbenchCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let vdd = cell.signal("vdd", Signal);
        let dut = cell.instantiate(super::flattened::VdividerArray {
            vdividers: vec![
                super::flattened::Vdivider::new(300, 300),
                super::flattened::Vdivider::new(600, 800),
                super::flattened::Vdivider::new(3600, 1600),
            ],
        });

        for i in 0..3 {
            cell.connect(dut.io().elements[i].vdd, vdd);
            cell.connect(dut.io().elements[i].vss, io.vss);
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
            .simulate_without_corner(
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
                (inst.block().r2.value / (inst.block().r1.value + inst.block().r2.value))
                    .to_f64()
                    .unwrap()
                    * 1.8f64
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
            .get_data(&cell.data().cell().io().elements[0].vdd)
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

impl<PDK: Pdk> Testbench<PDK, Spectre> for FlattenedVdividerArrayTb {
    type Output = VdividerArrayTbData;
    fn run(
        &self,
        cell: &Cell<FlattenedVdividerArrayTb>,
        sim: substrate::simulation::SimController<PDK, Spectre>,
    ) -> Self::Output {
        let output = sim
            .simulate_without_corner(
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
                (inst.block().r2.value / (inst.block().r1.value + inst.block().r2.value))
                    .to_f64()
                    .unwrap()
                    * 1.8f64
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
            .get_data(&cell.data().cell().io().elements[0].vdd)
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
