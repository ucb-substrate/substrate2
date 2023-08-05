use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::blocks::{Iprobe, Vsource};
use spectre::{Options, Spectre, Tran, TranCurrent, TranVoltage};
use substrate::block::Block;
use substrate::io::Signal;
use substrate::io::TestbenchIo;
use substrate::pdk::corner::InstallCorner;
use substrate::pdk::Pdk;
use substrate::schematic::{Cell, HasSchematic, HasSchematicData, Instance};
use substrate::simulation::data::{FromSaved, HasNodeData, Save};
use substrate::simulation::{Analysis, HasSimSchematic, SimulationContext, Simulator, Testbench};
use substrate::{Block, SchematicData};

use crate::hard_macro::VdividerDuplicateSubckt;
use crate::shared::vdivider::{Resistor, Vdivider, VdividerArray};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerTb;

#[derive(SchematicData)]
pub struct VdividerTbData {
    #[substrate(nested)]
    iprobe: Instance<Iprobe>,
    #[substrate(nested)]
    dut: Instance<Vdivider>,
}

impl HasSchematicData for VdividerTb {
    type Data = VdividerTbData;
}

impl<PDK: Pdk + InstallCorner<Spectre>> HasSimSchematic<PDK, Spectre> for VdividerTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
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

        let iprobe = cell.instantiate_tb(Iprobe);
        cell.connect(iprobe.io().p, vdd_a);
        cell.connect(iprobe.io().n, vdd);

        let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd_a);
        cell.connect(vsource.io().n, io.vss);

        Ok(VdividerTbData { iprobe, dut })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerDuplicateSubcktTb;

impl HasSchematicData for VdividerDuplicateSubcktTb {
    type Data = Instance<VdividerDuplicateSubckt>;
}

impl<PDK> HasSimSchematic<PDK, Spectre> for VdividerDuplicateSubcktTb
where
    PDK: Pdk,
    VdividerDuplicateSubckt: HasSchematic<PDK>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell.instantiate(VdividerDuplicateSubckt);

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        cell.connect(dut.io().out, out);

        let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerDuplicateSubcktTbOutput {
    pub vdd: Vec<f64>,
    pub out: Vec<f64>,
}

impl<PDK> Testbench<PDK, Spectre> for VdividerDuplicateSubcktTb
where
    PDK: Pdk + InstallCorner<Spectre>,
    VdividerDuplicateSubckt: HasSchematic<PDK>,
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
            vdd: output.get_data(&sim.tb.data().io().vdd).unwrap().clone(),
            out: output.get_data(&sim.tb.data().io().out).unwrap().clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerTbOutput {
    pub tran: VdividerTbTranOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, substrate::FromSaved)]
pub struct VdividerTbTranOutput {
    pub current: TranCurrent,
    pub iprobe: TranCurrent,
    pub vdd: TranVoltage,
    pub out: TranVoltage,
}

pub struct VdividerTbTranOutputKey<S: Simulator, A: Analysis>
where
    TranCurrent: FromSaved<S, A>,
    TranCurrent: FromSaved<S, A>,
    TranVoltage: FromSaved<S, A>,
    TranVoltage: FromSaved<S, A>,
{
    pub current: <TranCurrent as FromSaved<S, A>>::Key,
    pub iprobe: <TranCurrent as FromSaved<S, A>>::Key,
    pub vdd: <TranVoltage as FromSaved<S, A>>::Key,
    pub out: <TranVoltage as FromSaved<S, A>>::Key,
}

#[derive(Debug, Clone, Serialize, Deserialize, substrate::FromSaved)]
pub enum TestOutput {
    Voltage {
        vout: TranVoltage,
    },
    VoltageAndCurrent {
        vout: TranVoltage,
        iout: TranCurrent,
    },
}

impl Save<Spectre, Tran, &Cell<VdividerTb>> for VdividerTbTranOutput {
    fn save(
        ctx: &SimulationContext,
        cell: &Cell<VdividerTb>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::Key {
        Self::Key {
            current: TranCurrent::save(ctx, cell.data().dut.terminals().pwr.vdd, opts),
            iprobe: TranCurrent::save(ctx, cell.data().iprobe.terminals().p, opts),
            vdd: TranVoltage::save(ctx, cell.data().dut.io().pwr.vdd, opts),
            out: TranVoltage::save(ctx, cell.data().dut.io().out, opts),
        }
    }
}

impl<PDK: Pdk + InstallCorner<Spectre>> Testbench<PDK, Spectre> for VdividerTb {
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
#[substrate(io = "TestbenchIo")]
pub struct VdividerArrayTb;

impl HasSchematicData for VdividerArrayTb {
    type Data = Instance<VdividerArray>;
}

impl<PDK: Pdk + InstallCorner<Spectre>> HasSimSchematic<PDK, Spectre> for VdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Spectre, Self>,
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

impl HasSchematicData for FlattenedVdividerArrayTb {
    type Data = Instance<super::flattened::VdividerArray>;
}

impl<PDK: Pdk + InstallCorner<Spectre>> HasSimSchematic<PDK, Spectre> for FlattenedVdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
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

impl<PDK: Pdk + InstallCorner<Spectre>> Testbench<PDK, Spectre> for VdividerArrayTb {
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

        let out = sim
            .tb
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.io().out).unwrap().clone())
            .collect();

        let out_nested = sim
            .tb
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.data().r1.io().n).unwrap().clone())
            .collect();

        let vdd = output
            .get_data(&sim.tb.data().io().elements[0].vdd)
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

impl<PDK: Pdk + InstallCorner<Spectre>> Testbench<PDK, Spectre> for FlattenedVdividerArrayTb {
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

        let out = sim
            .tb
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.io().out).unwrap().clone())
            .collect();

        let out_nested = sim
            .tb
            .data()
            .data()
            .iter()
            .map(|inst| output.get_data(&inst.data().r1.io().n).unwrap().clone())
            .collect();

        let vdd = output
            .get_data(&sim.tb.data().io().elements[0].vdd)
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
