use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::{assert_relative_eq, relative_eq};
use cache::multi::MultiCache;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use spectre::analysis::tran::Tran;
use spectre::blocks::Vsource;
use spectre::{Options, Primitive, Spectre};
use spice::{BlackboxContents, BlackboxElement, Spice};
use substrate::block::Block;
use substrate::cache::Cache;
use substrate::context::Context;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::io::schematic::HardwareType;
use substrate::io::{InOut, Signal, TestbenchIo};
use substrate::io::{Io, TwoTerminalIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::{
    Cell, CellBuilder, ExportsNestedData, Instance, PrimitiveBinding, Schematic,
};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::{SimController, SimulationContext, Simulator, Testbench};
use test_log::test;

use crate::paths::test_data;
use crate::shared::inverter::tb::InverterTb;
use crate::shared::inverter::Inverter;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::{VdividerArrayTb, VdividerDuplicateSubcktTb};
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};
use substrate::schematic::primitives::{RawInstance, Resistor};

#[test]
fn vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir).unwrap();

    for (actual, expected) in [
        (&*output.current, 1.8 / 40.),
        (&*output.iprobe, 1.8 / 40.),
        (&*output.vdd, 1.8),
        (&*output.out, 0.9),
    ] {
        assert!(actual
            .iter()
            .cloned()
            .all(|val| relative_eq!(val, expected)));
    }
}

#[test]
fn vdivider_duplicate_subckt() {
    let test_name = "spectre_vdivider_duplicate_subckt";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerDuplicateSubcktTb, sim_dir).unwrap();

    // There are 2 subcircuits with the name `resistor`.
    // The first has a value of 100; the second has a value of 200.
    // We expect the second one to be used.
    let expected = 1.8 * 200.0 / (200.0 + 600.0);
    assert!(output.out.iter().all(|&val| relative_eq!(val, expected)));
}

#[test]
fn vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerArrayTb, sim_dir).unwrap();

    let cell = ctx.generate_schematic(crate::shared::vdivider::tb::VdividerArrayTb);

    for (expected, (out, out_nested)) in cell
        .cell()
        .iter()
        .map(|inst| {
            (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                .to_f64()
                .unwrap()
                * 1.8f64
        })
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(*val, expected)));
        assert!(out_nested.iter().all(|val| relative_eq!(*val, expected)));
    }

    assert!(output.vdd.iter().all(|val| relative_eq!(*val, 1.8)));
}

#[test]
fn flattened_vdivider_array_tran() {
    let test_name = "flattened_spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx
        .simulate(
            crate::shared::vdivider::tb::FlattenedVdividerArrayTb,
            sim_dir,
        )
        .unwrap();

    let cell = ctx.generate_schematic(crate::shared::vdivider::tb::FlattenedVdividerArrayTb);

    for (expected, (out, out_nested)) in cell
        .cell()
        .iter()
        .map(|inst| {
            (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                .to_f64()
                .unwrap()
                * 1.8f64
        })
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(*val, expected)));
        assert!(out_nested.iter().all(|val| relative_eq!(*val, expected)));
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
    )
    .unwrap();
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

    let ctx = Context::builder()
        .install(Spectre::default())
        .cache(Cache::new(MultiCache::builder().build()))
        .executor(executor)
        .build();

    ctx.simulate(VdividerTb, &sim_dir).unwrap();
    ctx.simulate(VdividerTb, &sim_dir).unwrap();

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

    impl ExportsNestedData for LibIncludeResistor {
        type NestedData = ();
    }

    impl Schematic<Spectre> for LibIncludeResistor {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            let mut prim = PrimitiveBinding::new(Primitive::BlackboxInstance {
                contents: BlackboxContents {
                    elems: vec![
                        BlackboxElement::InstanceName,
                        " ( ".into(),
                        BlackboxElement::Port("p".into()),
                        " ".into(),
                        BlackboxElement::Port("n".into()),
                        " ) example_resistor".into(),
                    ],
                },
            });
            prim.connect("p", io.p);
            prim.connect("n", io.n);
            cell.set_primitive(prim);
            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct LibIncludeTb(String);

    impl ExportsNestedData for LibIncludeTb {
        type NestedData = Instance<LibIncludeResistor>;
    }

    impl Schematic<Spectre> for LibIncludeTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            let vdd = cell.signal("vdd", Signal);
            let dut = cell.instantiate(LibIncludeResistor);
            let res = cell.instantiate(Resistor::new(1000));

            cell.connect(dut.io().p, vdd);
            cell.connect(dut.io().n, res.io().p);
            cell.connect(io.vss, res.io().n);

            let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(dut)
        }
    }

    impl SaveTb<Spectre, Tran, tran::Voltage> for LibIncludeTb {
        fn save_tb(
            ctx: &SimulationContext<Spectre>,
            cell: &Cell<Self>,
            opts: &mut <Spectre as Simulator>::Options,
        ) -> <tran::Voltage as FromSaved<Spectre, Tran>>::SavedKey {
            tran::Voltage::save(ctx, cell.data().io().n, opts)
        }
    }

    impl Testbench<Spectre> for LibIncludeTb {
        type Output = f64;

        fn run(&self, sim: SimController<Spectre, Self>) -> Self::Output {
            let mut opts = Options::default();
            opts.include_section(test_data("spectre/example_lib.scs"), &self.0);
            let vout: tran::Voltage = sim
                .simulate(
                    opts,
                    Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(spectre::ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            *vout.first().unwrap()
        }
    }

    let test_name = "spectre_can_include_sections";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();

    let output_tt = ctx
        .simulate(LibIncludeTb("section_a".to_string()), &sim_dir)
        .unwrap();
    let output_ss = ctx
        .simulate(LibIncludeTb("section_b".to_string()), sim_dir)
        .unwrap();

    assert_relative_eq!(output_tt, 0.9);
    assert_relative_eq!(output_ss, 1.2);
}

#[test]
fn spectre_can_save_paths_with_flattened_instances() {
    #[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct ScirResistor;

    impl ExportsNestedData for ScirResistor {
        type NestedData = ();
    }

    impl Schematic<Spectre> for ScirResistor {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            let mut scir = Spice::scir_cell_from_str(
                r#"
            .subckt res p n
            R0 p n 100
            .ends
            "#,
                "res",
            )
            .convert_schema::<Spectre>()?;

            scir.connect("p", io.p);
            scir.connect("n", io.n);

            cell.set_scir(scir);
            Ok(())
        }
    }

    #[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct VirtualResistor;

    impl ExportsNestedData for VirtualResistor {
        type NestedData = ();
    }

    impl Schematic<Spectre> for VirtualResistor {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            cell.instantiate_connected(ScirResistor, io);
            cell.instantiate_connected(Resistor::new(dec!(200)), io);
            let raw_res = cell.instantiate(RawInstance::with_params(
                arcstr::literal!("resistor"),
                vec![arcstr::literal!("pos"), arcstr::literal!("neg")],
                HashMap::from_iter([(arcstr::literal!("r"), dec!(300).into())]),
            ));
            cell.connect(raw_res.io()[0], io.p);
            cell.connect(raw_res.io()[1], io.n);

            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct VirtualResistorTb;

    impl ExportsNestedData for VirtualResistorTb {
        type NestedData = Instance<VirtualResistor>;
    }

    impl Schematic<Spectre> for VirtualResistorTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            let vdd = cell.signal("vdd", Signal);
            let dut = cell.instantiate(VirtualResistor);

            cell.connect(dut.io().p, vdd);
            cell.connect(dut.io().n, io.vss);

            let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(dut)
        }
    }

    impl SaveTb<Spectre, Tran, tran::Current> for VirtualResistorTb {
        fn save_tb(
            ctx: &SimulationContext<Spectre>,
            cell: &Cell<Self>,
            opts: &mut <Spectre as Simulator>::Options,
        ) -> <tran::Current as FromSaved<Spectre, Tran>>::SavedKey {
            tran::Current::save(ctx, cell.data().io().p, opts)
        }
    }

    impl Testbench<Spectre> for VirtualResistorTb {
        type Output = tran::Current;

        fn run(&self, sim: SimController<Spectre, Self>) -> Self::Output {
            sim.simulate(
                Options::default(),
                Tran {
                    stop: dec!(2e-9),
                    errpreset: Some(spectre::ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation")
        }
    }

    let test_name = "spectre_can_save_paths_with_flattened_instances";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let current_draw = ctx.simulate(VirtualResistorTb, sim_dir).unwrap();

    assert!(current_draw
        .iter()
        .cloned()
        .all(|val| relative_eq!(val, 1.8 * (1. / 100. + 1. / 200. + 1. / 300.))));
}

#[test]
fn spectre_initial_condition() {
    let test_name = "spectre_initial_condition";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();

    let (first, _, _) = ctx
        .simulate(crate::shared::rc::RcTb::new(dec!(1.4)), &sim_dir)
        .unwrap();
    assert_relative_eq!(first, 1.4);

    let (first, _, _) = ctx
        .simulate(crate::shared::rc::RcTb::new(dec!(2.1)), sim_dir)
        .unwrap();
    assert_relative_eq!(first, 2.1);
}

#[test]
fn spectre_rc_zin_ac() {
    let test_name = "spectre_rc_zin_ac";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();

    let (_, _, z) = ctx
        .simulate(crate::shared::rc::RcTb::new(dec!(0)), sim_dir)
        .unwrap();
    assert_relative_eq!(z.re, -17.286407017773225);
    assert_relative_eq!(z.im, 130.3364383055986);
}
