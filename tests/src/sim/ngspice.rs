use approx::relative_eq;
use ngspice::blocks::Vsource;
use ngspice::tran::Tran;
use ngspice::{Ngspice, Options};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::{Signal, TestbenchIo};
use substrate::schematic::primitives::Resistor;
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Instance, NestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::{SimController, SimulationContext, Simulator, Testbench};
use test_log::test;

use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;

#[test]
fn ngspice_can_save_voltages_and_currents() {
    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct ResistorTb;

    #[derive(NestedData)]
    struct ResistorTbData {
        r1: Instance<Resistor>,
        r2: Instance<Resistor>,
        r3: Instance<Resistor>,
    }

    impl ExportsNestedData for ResistorTb {
        type NestedData = ResistorTbData;
    }

    impl Schematic<Ngspice> for ResistorTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Ngspice>,
        ) -> substrate::error::Result<Self::NestedData> {
            let vdd = cell.signal("vdd", Signal);
            let r1 = cell.instantiate(Resistor::new(dec!(100)));
            let r2 = cell.instantiate(Resistor::new(dec!(100)));
            let r3 = cell.instantiate(Resistor::new(dec!(100)));

            cell.connect(r1.io().p, vdd);
            cell.connect(r1.io().n, r2.io().p);
            cell.connect(r2.io().n, io.vss);
            cell.connect(r1.io().n, r3.io().p);
            cell.connect(r3.io().n, io.vss);

            let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(ResistorTbData { r1, r2, r3 })
        }
    }

    #[derive(FromSaved, Serialize, Deserialize)]
    struct ResistorTbOutput {
        r1: tran::Current,
        r2: tran::Current,
        r3: tran::Current,
        vout: tran::Voltage,
        r3_terminal: tran::Current,
    }

    impl SaveTb<Ngspice, Tran, ResistorTbOutput> for ResistorTb {
        fn save_tb(
            ctx: &SimulationContext<Ngspice>,
            to_save: &Cell<Self>,
            opts: &mut <Ngspice as Simulator>::Options,
        ) -> <ResistorTbOutput as FromSaved<Ngspice, Tran>>::SavedKey {
            ResistorTbOutputSavedKey {
                r1: tran::Current::save(ctx, &to_save.r1, opts),
                r2: tran::Current::save(ctx, &to_save.r2, opts),
                r3: tran::Current::save(ctx, &to_save.r3, opts),
                vout: tran::Voltage::save(ctx, to_save.data().r1.io().n, opts),
                r3_terminal: tran::Current::save(ctx, to_save.data().r3.io().p, opts),
            }
        }
    }

    impl Testbench<Ngspice> for ResistorTb {
        type Output = ResistorTbOutput;

        fn run(&self, sim: SimController<Ngspice, Self>) -> Self::Output {
            sim.simulate(
                Options::default(),
                Tran {
                    step: dec!(2e-10),
                    stop: dec!(2e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation")
        }
    }

    let test_name = "ngspice_can_save_voltages_and_currents";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_open_ctx();
    let ResistorTbOutput {
        r1,
        r2,
        r3,
        vout,
        r3_terminal,
    } = ctx.simulate(ResistorTb, sim_dir).unwrap();

    for (actual, expected) in [
        (&*r1, 1.8 / 150.),
        (&*r2, 1.8 / 300.),
        (&*r3, 1.8 / 300.),
        (&*vout, 1.8 / 3.),
        (&*r3_terminal, 1.8 / 300.),
    ] {
        actual.iter().copied().for_each(|val| {
            assert!(
                relative_eq!(val, expected),
                "found {val}, expected {expected}"
            )
        });
    }
}
