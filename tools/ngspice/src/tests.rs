use std::path::PathBuf;

use crate::blocks::Vsource;
use crate::tran::Tran;
use crate::{Ngspice, Options};
use approx::relative_eq;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spice::Resistor;
use substrate::block::Block;
use substrate::context::Context;
use substrate::schematic::{CellBuilder, ConvertSchema, NestedData, Schematic};
use substrate::types::schematic::Terminal;
use substrate::types::{Signal, TestbenchIo};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

#[inline]
fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

fn ngspice_ctx() -> Context {
    Context::builder().install(Ngspice::default()).build()
}

#[test]
fn ngspice_can_save_voltages_and_currents() {
    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct ResistorTb;

    #[derive(NestedData)]
    struct ResistorTbData {
        r1: Terminal,
        r2: Terminal,
        r3: Terminal,
    }

    impl Schematic for ResistorTb {
        type Schema = Ngspice;
        type NestedData = ResistorTbData;
        fn schematic(
            &self,
            io: &substrate::types::schematic::IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            let vdd = cell.signal("vdd", Signal);
            let r1 = cell.instantiate(ConvertSchema::<_, Ngspice>::new(Resistor::new(dec!(100))));
            let r2 = cell.instantiate(ConvertSchema::<_, Ngspice>::new(Resistor::new(dec!(100))));
            let r3 = cell.instantiate(ConvertSchema::<_, Ngspice>::new(Resistor::new(dec!(100))));

            cell.connect(r1.io().p, vdd);
            cell.connect(r1.io().n, r2.io().p);
            cell.connect(r2.io().n, io.vss);
            cell.connect(r1.io().n, r3.io().p);
            cell.connect(r3.io().n, io.vss);

            let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(ResistorTbData {
                r1: r1.io().p,
                r2: r2.io().p,
                r3: r3.io().p,
            })
        }
    }

    let test_name = "ngspice_can_save_voltages_and_currents";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = ngspice_ctx();
    let sim = ctx
        .get_sim_controller(ResistorTb, sim_dir)
        .expect("failed to get sim controller");

    let output = sim
        .simulate(
            Options::default(),
            Tran {
                step: dec!(2e-10),
                stop: dec!(2e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation");

    for (actual, expected) in [
        (&*output.r1.i, 1.8 / 150.),
        (&*output.r2.i, 1.8 / 300.),
        (&*output.r3.i, 1.8 / 300.),
        (&*output.r2.v, 1.8 / 3.),
    ] {
        actual.iter().copied().for_each(|val| {
            assert!(
                relative_eq!(val, expected),
                "found {val}, expected {expected}"
            )
        });
    }
}
