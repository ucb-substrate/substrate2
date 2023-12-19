use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::analysis::montecarlo;
use spectre::analysis::montecarlo::{MonteCarlo, Variations};
use spectre::analysis::tran::Tran;
use spectre::blocks::{Iprobe, Vsource};
use spectre::{Options, Spectre};
use spice::Spice;
use substrate::block::Block;
use substrate::io::TestbenchIo;
use substrate::io::{SchematicType, Signal};
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Instance, NestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb, SavedKey};
use substrate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy, Testbench};

use crate::hard_macro::VdividerDuplicateSubckt;
use crate::shared::vdivider::{Resistor, Vdivider, VdividerArray};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerTb;

#[derive(NestedData)]
pub struct VdividerTbData {
    iprobe: Instance<Iprobe>,
    dut: Instance<Vdivider>,
}

impl ExportsNestedData for VdividerTb {
    type NestedData = VdividerTbData;
}

impl Schematic<Spectre> for VdividerTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
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

        Ok(VdividerTbData { iprobe, dut })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerDuplicateSubcktTb;

impl ExportsNestedData for VdividerDuplicateSubcktTb {
    type NestedData = Instance<VdividerDuplicateSubckt>;
}

impl Schematic<Spectre> for VdividerDuplicateSubcktTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell
            .sub_builder::<Spice>()
            .instantiate(VdividerDuplicateSubckt);

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        cell.connect(dut.io().out, out);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);
        Ok(dut)
    }
}

#[derive(Debug, Clone, FromSaved, Serialize, Deserialize)]
pub struct VdividerDuplicateSubcktTbOutput {
    pub vdd: tran::Voltage,
    pub out: tran::Voltage,
}

impl SaveTb<Spectre, Tran, VdividerDuplicateSubcktTbOutput> for VdividerDuplicateSubcktTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        to_save: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <VdividerDuplicateSubcktTbOutput as FromSaved<Spectre, Tran>>::SavedKey {
        VdividerDuplicateSubcktTbOutputSavedKey {
            vdd: tran::Voltage::save(ctx, &to_save.data().io().vdd, opts),
            out: tran::Voltage::save(ctx, &to_save.data().io().out, opts),
        }
    }
}

impl Testbench<Spectre> for VdividerDuplicateSubcktTb {
    type Output = VdividerDuplicateSubcktTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        sim.simulate(
            Options::default(),
            Tran {
                stop: dec!(1e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerTbOutput {
    pub current: tran::Current,
    pub iprobe: tran::Current,
    pub vdd: tran::Voltage,
    pub out: tran::Voltage,
    pub mc_out: montecarlo::Output<tran::Voltage>,
}

pub struct VdividerTbOutputSavedKey {
    pub current: SavedKey<Spectre, Tran, tran::Current>,
    pub iprobe: SavedKey<Spectre, Tran, tran::Current>,
    pub vdd: SavedKey<Spectre, Tran, tran::Voltage>,
    pub out: SavedKey<Spectre, Tran, tran::Voltage>,
    pub mc_out: SavedKey<Spectre, MonteCarlo<Tran>, montecarlo::Output<tran::Voltage>>,
}

pub struct VdividerAnalysis {
    tran: Tran,
    mc: MonteCarlo<Tran>,
}

pub struct VdividerAnalysisOutput {
    tran: spectre::analysis::tran::Output,
    mc: Vec<spectre::analysis::tran::Output>,
}
impl Analysis for VdividerAnalysis {
    type Output = VdividerAnalysisOutput;
}

impl SupportedBy<Spectre> for VdividerAnalysis {
    fn into_input(self, inputs: &mut Vec<<Spectre as Simulator>::Input>) {
        self.tran.into_input(inputs);
        self.mc.into_input(inputs);
    }

    fn from_output(
        outputs: &mut impl Iterator<Item = <Spectre as Simulator>::Output>,
    ) -> Self::Output {
        VdividerAnalysisOutput {
            tran: outputs.next().unwrap().try_into().unwrap(),
            mc: outputs.next().unwrap().try_into().unwrap(),
        }
    }
}

impl FromSaved<Spectre, VdividerAnalysis> for VdividerTbOutput {
    type SavedKey = VdividerTbOutputSavedKey;

    fn from_saved(output: &<VdividerAnalysis as Analysis>::Output, key: &Self::SavedKey) -> Self {
        Self {
            current: tran::Current::from_saved(&output.tran, &key.current),
            iprobe: tran::Current::from_saved(&output.tran, &key.iprobe),
            vdd: tran::Voltage::from_saved(&output.tran, &key.vdd),
            out: tran::Voltage::from_saved(&output.tran, &key.out),
            mc_out: montecarlo::Output::<_>::from_saved(&output.mc, &key.mc_out),
        }
    }
}

impl SaveTb<Spectre, VdividerAnalysis, VdividerTbOutput> for VdividerTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        to_save: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> SavedKey<Spectre, VdividerAnalysis, VdividerTbOutput> {
        VdividerTbOutputSavedKey {
            current: tran::Current::save(ctx, to_save.dut.io().pwr.vdd, opts),
            iprobe: tran::Current::save(ctx, to_save.iprobe.io().p, opts),
            vdd: tran::Voltage::save(ctx, to_save.dut.io().pwr.vdd, opts),
            out: tran::Voltage::save(ctx, to_save.dut.io().out, opts),
            mc_out: tran::Voltage::save(ctx, to_save.dut.io().out, opts),
        }
    }
}

impl Testbench<Spectre> for VdividerTb {
    type Output = VdividerTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        sim.simulate(
            Options::default(),
            VdividerAnalysis {
                tran: Tran {
                    stop: dec!(1e-9),
                    ..Default::default()
                },
                mc: MonteCarlo {
                    variations: Variations::All,
                    numruns: 2,
                    seed: None,
                    firstrun: None,
                    analysis: Tran {
                        stop: dec!(1e-9),
                        ..Default::default()
                    },
                },
            },
        )
        .expect("failed to run simulation")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerArrayTb;

impl ExportsNestedData for VdividerArrayTb {
    type NestedData = Instance<VdividerArray>;
}

impl Schematic<Spectre> for VdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
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
        Ok(dut)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct FlattenedVdividerArrayTb;

impl ExportsNestedData for FlattenedVdividerArrayTb {
    type NestedData = Instance<super::flattened::VdividerArray>;
}

impl Schematic<Spectre> for FlattenedVdividerArrayTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
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
        Ok(dut)
    }
}
#[derive(Debug, Clone, FromSaved, Serialize, Deserialize)]
pub struct VdividerArrayTbOutput {
    pub out: Vec<tran::Voltage>,
    pub out_nested: Vec<tran::Voltage>,
    pub vdd: tran::Voltage,
}

impl SaveTb<Spectre, Tran, VdividerArrayTbOutput> for VdividerArrayTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <VdividerArrayTbOutput as FromSaved<Spectre, Tran>>::SavedKey {
        VdividerArrayTbOutputSavedKey {
            out: cell
                .iter()
                .map(|inst| tran::Voltage::save(ctx, inst.io().out, opts))
                .collect(),
            out_nested: cell
                .iter()
                .map(|inst| tran::Voltage::save(ctx, inst.r1.io().n, opts))
                .collect(),
            vdd: tran::Voltage::save(ctx, &cell.data().io().elements[0].vdd, opts),
        }
    }
}

impl SaveTb<Spectre, Tran, VdividerArrayTbOutput> for FlattenedVdividerArrayTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <VdividerArrayTbOutput as FromSaved<Spectre, Tran>>::SavedKey {
        VdividerArrayTbOutputSavedKey {
            out: cell
                .iter()
                .map(|inst| tran::Voltage::save(ctx, inst.io().out, opts))
                .collect(),
            out_nested: cell
                .iter()
                .map(|inst| tran::Voltage::save(ctx, inst.r1.io().n, opts))
                .collect(),
            vdd: tran::Voltage::save(ctx, &cell.data().io().elements[0].vdd, opts),
        }
    }
}

impl Testbench<Spectre> for VdividerArrayTb {
    type Output = VdividerArrayTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        sim.simulate(
            Options::default(),
            Tran {
                stop: dec!(1e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}

impl Testbench<Spectre> for FlattenedVdividerArrayTb {
    type Output = VdividerArrayTbOutput;
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        sim.simulate(
            Options::default(),
            Tran {
                stop: dec!(1e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}
