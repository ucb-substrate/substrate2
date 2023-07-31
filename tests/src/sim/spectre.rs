use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::{assert_relative_eq, relative_eq};
use cache::multi::MultiCache;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130CommercialPdk;
use spectre::blocks::Vsource;
use spectre::{Options, Spectre, Tran};
use substrate::block::Block;
use substrate::cache::Cache;
use substrate::context::Context;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::io::{InOut, SchematicType, Signal, TestbenchIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::{Cell, HasSchematic, Instance, TestbenchCellBuilder};
use substrate::simulation::data::HasNodeData;
use substrate::simulation::{HasTestbenchSchematicImpl, SimController, Testbench};
use substrate::{Block, Io};
use test_log::test;

use crate::paths::test_data;
use crate::shared::inverter::tb::InverterTb;
use crate::shared::inverter::Inverter;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::VdividerArrayTb;
use crate::shared::vdivider::Resistor;
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir);

    println!("{:?}", output.vdd);
    println!("{:?}", output.out);
}

#[test]
fn spectre_vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerArrayTb, sim_dir);

    for (expected, (out, out_nested)) in output
        .expected
        .iter()
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(val, expected)));
        assert_eq!(out, out_nested);
    }

    assert!(output.vdd.iter().all(|val| relative_eq!(*val, 1.8)));
}

#[test]
fn spectre_flattened_vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(
        crate::shared::vdivider::tb::FlattenedVdividerArrayTb,
        sim_dir,
    );

    for (expected, (out, out_nested)) in output
        .expected
        .iter()
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(val, expected)));
        assert_eq!(out, out_nested);
    }

    assert!(output.vdd.iter().all(|val| relative_eq!(*val, 1.8)));
}

#[test]
fn inv_tb() {
    let test_name = "inv_tb";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    ctx.simulate(
        InverterTb::new(
            Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25)),
            Inverter {
                nw: 1_200,
                pw: 2_000,
                lch: 150,
            },
        ),
        sim_dir,
    );
}

#[test]
fn spectre_caches_simulations() {
    #[derive(Clone, Debug, Default)]
    struct CountExecutor {
        executor: LocalExecutor,
        count: Arc<Mutex<u64>>,
    }

    impl Executor for CountExecutor {
        fn execute(&self, command: Command, opts: ExecOpts) -> Result<(), substrate::error::Error> {
            *self.count.lock().unwrap() += 1;
            self.executor.execute(command, opts)
        }
    }

    let test_name = "spectre_caches_simulations";
    let sim_dir = get_path(test_name, "sim/");
    let executor = CountExecutor::default();
    let count = executor.count.clone();

    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    let ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new(pdk_root))
        .with_simulator(Spectre::default())
        .cache(Cache::new(MultiCache::builder().build()))
        .executor(executor)
        .build();

    ctx.simulate(VdividerTb, &sim_dir);
    ctx.simulate(VdividerTb, &sim_dir);

    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn spectre_can_include_sections() {
    #[derive(Default, Clone, Io)]
    struct LibIncludeResistorIo {
        p: InOut<Signal>,
        n: InOut<Signal>,
    }

    #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "LibIncludeResistorIo")]
    struct LibIncludeResistor;

    impl HasSchematic for LibIncludeResistor {
        type Data = ();
    }

    impl HasTestbenchSchematicImpl<Sky130CommercialPdk, Spectre> for LibIncludeResistor {
        fn schematic(
            &self,
            _io: &<<Self as Block>::Io as SchematicType>::Data,
            cell: &mut TestbenchCellBuilder<Sky130CommercialPdk, Spectre, Self>,
        ) -> substrate::error::Result<Self::Data> {
            cell.set_blackbox("res0 (p n) example_resistor");

            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct LibIncludeTb(String);

    impl HasSchematic for LibIncludeTb {
        type Data = Instance<LibIncludeResistor>;
    }

    impl HasTestbenchSchematicImpl<Sky130CommercialPdk, Spectre> for LibIncludeTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Data,
            cell: &mut TestbenchCellBuilder<Sky130CommercialPdk, Spectre, Self>,
        ) -> substrate::error::Result<Self::Data> {
            let vdd = cell.signal("vdd", Signal);
            let dut = cell.instantiate_tb(LibIncludeResistor);
            let res = cell.instantiate(Resistor::new(1000));

            cell.connect(dut.io().p, vdd);
            cell.connect(dut.io().n, res.io().p);
            cell.connect(io.vss, res.io().n);

            let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(dut)
        }
    }

    impl Testbench<Sky130CommercialPdk, Spectre> for LibIncludeTb {
        type Output = f64;

        fn run(
            &self,
            cell: &Cell<Self>,
            sim: SimController<Sky130CommercialPdk, Spectre>,
        ) -> Self::Output {
            let mut opts = Options::default();
            opts.include_section(test_data("spectre/example_lib.scs"), &self.0);
            let output = sim
                .simulate(
                    opts,
                    Sky130Corner::Tt,
                    Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(spectre::ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");
            *output
                .get_data(&cell.data().io().n)
                .unwrap()
                .first()
                .unwrap()
        }
    }

    let test_name = "spectre_can_include_sections";
    let sim_dir = get_path(test_name, "sim/");

    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    let ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new(pdk_root))
        .with_simulator(Spectre::default())
        .build();
    let output_tt = ctx.simulate(LibIncludeTb("section_a".to_string()), &sim_dir);
    let output_ss = ctx.simulate(LibIncludeTb("section_b".to_string()), sim_dir);

    assert_relative_eq!(output_tt, 0.9);
    assert_relative_eq!(output_ss, 1.2);
}
