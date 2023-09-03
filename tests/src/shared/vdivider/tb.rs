use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::blocks::{Iprobe, Vsource};
use spectre::tran::{Tran, TranCurrent, TranVoltage};
use spectre::{Options, Spectre, SpectrePrimitive};
use substrate::block::{self, Block};
use substrate::io::TestbenchIo;
use substrate::io::{SchematicType, Signal};
use substrate::pdk::corner::SupportsSimulator;
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{
    Cell, CellBuilder, ExportsNestedData, Instance, InstanceData, Schematic, SchematicData,
};
use substrate::simulation::data::{FromSaved, HasSimData, Save};
use substrate::simulation::{SimulationContext, Simulator, Testbench};

use crate::hard_macro::VdividerDuplicateSubckt;
use crate::shared::vdivider::{Resistor, Vdivider, VdividerArray};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "block::Cell")]
pub struct VdividerTb;

#[derive(SchematicData)]
pub struct VdividerTbData {
    iprobe: InstanceData<Iprobe>,
    dut: InstanceData<Vdivider>,
}

impl ExportsNestedData for VdividerTb {
    type NestedData = VdividerTbData;
}

impl<PDK: Pdk> Schematic<PDK, Spectre> for VdividerTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vdd_a = cell.signal("vdd_a", Signal);
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell.instantiate(Vdivider {
            r1: Resistor::new(20),
            r2: Resistor::new(20),
        });

        cell.connect(dut.io().pwr.vdd, vdd);
        cell.connect(dut.io().pwr.vss, io.vss);
        cell.connect(dut.io().out, out);

        let iprobe = cell.instantiate(Iprobe);
        cell.connect(iprobe.io().p, vdd_a);
        cell.connect(iprobe.io().n, vdd);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd_a);
        cell.connect(vsource.io().n, io.vss);

        Ok(VdividerTbData {
            iprobe: iprobe.data(),
            dut: dut.data(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "block::Cell")]
pub struct VdividerDuplicateSubcktTb;

impl ExportsNestedData for VdividerDuplicateSubcktTb {
    type NestedData = InstanceData<VdividerDuplicateSubckt>;
}

impl<PDK> Schematic<PDK, Spectre> for VdividerDuplicateSubcktTb
where
    PDK: Pdk,
    VdividerDuplicateSubckt: Schematic<PDK, Spectre>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell.instantiate(VdividerDuplicateSubckt);

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        cell.connect(dut.io().out, out);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut.data())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerDuplicateSubcktTbOutput {
    pub vdd: Vec<f64>,
    pub out: Vec<f64>,
}

impl<PDK> Testbench<PDK, Spectre> for VdividerDuplicateSubcktTb
where
    PDK: Pdk + SupportsSimulator<Spectre>,
    VdividerDuplicateSubckt: Schematic<PDK, Spectre>,
{
    type Output = VdividerDuplicateSubcktTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<PDK, Spectre, Self>) -> Self::Output {
        let output = sim
            .simulate_default(
                Options::default(),
                None,
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        VdividerDuplicateSubcktTbOutput {
            vdd: output
                .get_data(&sim.tb.data().terminals().vdd)
                .unwrap()
                .clone(),
            out: output
                .get_data(&sim.tb.data().terminals().out)
                .unwrap()
                .clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerTbOutput {
    pub tran: VdividerTbTranOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromSaved)]
pub struct VdividerTbTranOutput {
    pub current: TranCurrent,
    pub iprobe: TranCurrent,
    pub vdd: TranVoltage,
    pub out: TranVoltage,
}

impl Save<Spectre, Tran, &Cell<SpectrePrimitive, VdividerTb>> for VdividerTbTranOutput {
    fn save(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<SpectrePrimitive, VdividerTb>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::Key {
        Self::Key {
            current: TranCurrent::save(ctx, cell.nodes().dut.terminals().pwr.vdd, opts),
            iprobe: TranCurrent::save(ctx, cell.nodes().iprobe.terminals().p, opts),
            vdd: TranVoltage::save(ctx, cell.nodes().dut.terminals().pwr.vdd, opts),
            out: TranVoltage::save(ctx, cell.nodes().dut.terminals().out, opts),
        }
    }
}

impl<PDK: Pdk + SupportsSimulator<Spectre>> Testbench<PDK, Spectre> for VdividerTb {
    type Output = VdividerTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<PDK, Spectre, Self>) -> Self::Output {
        let tran: VdividerTbTranOutput = sim
            .simulate(
                Options::default(),
                None,
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        VdividerTbOutput { tran }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "block::Cell")]
pub struct VdividerArrayTb;

impl ExportsNestedData for VdividerArrayTb {
    type NestedData = InstanceData<VdividerArray>;
}

impl<PDK: Pdk + SupportsSimulator<Spectre>> Schematic<PDK, Spectre> for VdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
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

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut.data())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "block::Cell")]
pub struct FlattenedVdividerArrayTb;

impl ExportsNestedData for FlattenedVdividerArrayTb {
    type NestedData = InstanceData<super::flattened::VdividerArray>;
}

impl<PDK: Pdk + SupportsSimulator<Spectre>> Schematic<PDK, Spectre> for FlattenedVdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vdd = cell.signal("vdd", Signal);
        let dut = cell.instantiate(super::flattened::VdividerArray {
            vdividers: vec![
                super::flattened::Vdivider::new(32000, 12000),
                super::flattened::Vdivider::new(10, 10),
                super::flattened::Vdivider::new(680, 970),
            ],
        });

        for i in 0..3 {
            cell.connect(dut.io().elements[i].vdd, vdd);
            cell.connect(dut.io().elements[i].vss, io.vss);
        }

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut.data())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerArrayTbData {
    pub expected: Vec<f64>,
    pub out: Vec<Vec<f64>>,
    pub out_nested: Vec<Vec<f64>>,
    pub vdd: Vec<f64>,
}

impl<PDK: Pdk + SupportsSimulator<Spectre>> Testbench<PDK, Spectre> for VdividerArrayTb {
    type Output = VdividerArrayTbData;
    fn run(&self, sim: substrate::simulation::SimController<PDK, Spectre, Self>) -> Self::Output {
        let output = sim
            .simulate_default(
                Options::default(),
                None,
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let expected: Vec<_> = sim
            .tb
            .nodes()
            .nodes()
            .into_iter()
            .map(|inst| {
                (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                    .to_f64()
                    .unwrap()
                    * 1.8f64
            })
            .collect();

        let out = sim
            .tb
            .nodes()
            .nodes()
            .iter()
            .map(|inst| output.get_data(&inst.terminals().out).unwrap().clone())
            .collect();

        let out_nested = sim
            .tb
            .nodes()
            .nodes()
            .iter()
            .map(|inst| {
                output
                    .get_data(&inst.nodes().r1.terminals().n)
                    .unwrap()
                    .clone()
            })
            .collect();

        let vdd = output
            .get_data(&sim.tb.nodes().terminals().elements[0].vdd)
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

impl<PDK: Pdk + SupportsSimulator<Spectre>> Testbench<PDK, Spectre> for FlattenedVdividerArrayTb {
    type Output = VdividerArrayTbData;
    fn run(&self, sim: substrate::simulation::SimController<PDK, Spectre, Self>) -> Self::Output {
        let output = sim
            .simulate_default(
                Options::default(),
                None,
                Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let expected: Vec<_> = sim
            .tb
            .nodes()
            .nodes()
            .into_iter()
            .map(|inst| {
                (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                    .to_f64()
                    .unwrap()
                    * 1.8f64
            })
            .collect();

        let out = sim
            .tb
            .nodes()
            .nodes()
            .iter()
            .map(|inst| output.get_data(&inst.terminals().out).unwrap().clone())
            .collect();

        let out_nested = sim
            .tb
            .nodes()
            .nodes()
            .iter()
            .map(|inst| {
                output
                    .get_data(&inst.nodes().r1.terminals().n)
                    .unwrap()
                    .clone()
            })
            .collect();

        let vdd = output
            .get_data(&sim.tb.nodes().terminals().elements[0].vdd)
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
