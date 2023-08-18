use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::{assert_relative_eq, relative_eq};
use cache::multi::MultiCache;
use indexmap::IndexMap;
use rust_decimal_macros::dec;
use scir::Expr::NumericLiteral;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130CommercialPdk;
use spectre::blocks::Vsource;
use spectre::tran::{Tran, TranCurrent};
use spectre::{Options, Spectre};
use substrate::block::Block;
use substrate::cache::Cache;
use substrate::context::Context;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::io::{InOut, SchematicType, Signal, TestbenchIo};
use substrate::io::{Io, TwoTerminalIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::{
    Cell, ExportsSchematicData, Instance, PrimitiveDevice, PrimitiveDeviceKind, PrimitiveNode,
    Schematic, SimCellBuilder,
};
use substrate::simulation::data::{FromSaved, HasSimData, Save};
use substrate::simulation::{
    HasSimSchematic, SimController, SimulationContext, Simulator, Testbench,
};
use test_log::test;

use crate::paths::test_data;
use crate::shared::inverter::tb::InverterTb;
use crate::shared::inverter::Inverter;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::{VdividerArrayTb, VdividerDuplicateSubcktTb};
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};
use substrate::schematic::primitives::Resistor;

#[test]
fn vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir).unwrap();

    for (actual, expected) in [
        (&*output.tran.current, 1.8 / 40.),
        (&*output.tran.iprobe, 1.8 / 40.),
        (&*output.tran.vdd, 1.8),
        (&*output.tran.out, 0.9),
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

    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    let ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new(pdk_root))
        .with_simulator(Spectre::default())
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

    impl ExportsSchematicData for LibIncludeResistor {
        type Data = ();
    }

    impl HasSimSchematic<Sky130CommercialPdk, Spectre> for LibIncludeResistor {
        fn schematic(
            &self,
            _io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut SimCellBuilder<Sky130CommercialPdk, Spectre, Self>,
        ) -> substrate::error::Result<Self::Data> {
            cell.set_blackbox("res0 (p n) example_resistor");

            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct LibIncludeTb(String);

    impl ExportsSchematicData for LibIncludeTb {
        type Data = Instance<LibIncludeResistor>;
    }

    impl HasSimSchematic<Sky130CommercialPdk, Spectre> for LibIncludeTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut SimCellBuilder<Sky130CommercialPdk, Spectre, Self>,
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

        fn run(&self, sim: SimController<Sky130CommercialPdk, Spectre, Self>) -> Self::Output {
            let mut opts = Options::default();
            opts.include_section(test_data("spectre/example_lib.scs"), &self.0);
            let output = sim
                .simulate_default(
                    opts,
                    Some(&Sky130Corner::Tt),
                    Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(spectre::ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");
            *output
                .get_data(&sim.tb.data().terminals().n)
                .unwrap()
                .first()
                .unwrap()
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
    #[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Schematic)]
    #[substrate(io = "TwoTerminalIo", flatten)]
    #[substrate(schematic(
        source = "r#\"\
            .subckt res p n
            R0 p n 100
            .ends
        \"#",
        name = "res",
        fmt = "inline-spice",
        pdk = "Sky130CommercialPdk"
    ))]
    pub struct ScirResistor;

    #[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo", flatten)]
    pub struct VirtualResistor;

    impl ExportsSchematicData for VirtualResistor {
        type Data = ();
    }

    impl HasSimSchematic<Sky130CommercialPdk, Spectre> for VirtualResistor {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut SimCellBuilder<Sky130CommercialPdk, Spectre, Self>,
        ) -> substrate::error::Result<Self::Data> {
            let res1 = cell.instantiate(ScirResistor);
            cell.connect(res1.io().p, io.p);
            cell.connect(res1.io().n, io.n);
            cell.add_primitive(
                PrimitiveDeviceKind::Res2 {
                    pos: PrimitiveNode::new("1", io.p),
                    neg: PrimitiveNode::new("2", io.n),
                    value: dec!(200),
                }
                .into(),
            );
            cell.add_primitive(PrimitiveDevice::from_params(
                PrimitiveDeviceKind::RawInstance {
                    ports: vec![PrimitiveNode::new("1", io.p), PrimitiveNode::new("2", io.n)],
                    cell: arcstr::literal!("resistor"),
                },
                IndexMap::from_iter([(arcstr::literal!("r"), NumericLiteral(dec!(300)))]),
            ));
            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
    #[substrate(io = "TestbenchIo")]
    struct VirtualResistorTb;

    impl ExportsSchematicData for VirtualResistorTb {
        type Data = Instance<VirtualResistor>;
    }

    impl HasSimSchematic<Sky130CommercialPdk, Spectre> for VirtualResistorTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut SimCellBuilder<Sky130CommercialPdk, Spectre, Self>,
        ) -> substrate::error::Result<Self::Data> {
            let vdd = cell.signal("vdd", Signal);
            let dut = cell.instantiate_tb(VirtualResistor);

            cell.connect(dut.io().p, vdd);
            cell.connect(dut.io().n, io.vss);

            let vsource = cell.instantiate_tb(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(dut)
        }
    }

    #[derive(FromSaved, Serialize, Deserialize)]
    struct VirtualResistorOutput {
        current_draw: TranCurrent,
    }

    impl Save<Spectre, Tran, &Cell<VirtualResistorTb>> for VirtualResistorOutput {
        fn save(
            ctx: &SimulationContext,
            to_save: &Cell<VirtualResistorTb>,
            opts: &mut <Spectre as Simulator>::Options,
        ) -> Self::Key {
            Self::Key {
                current_draw: TranCurrent::save(ctx, to_save.data().terminals().p, opts),
            }
        }
    }

    impl Testbench<Sky130CommercialPdk, Spectre> for VirtualResistorTb {
        type Output = VirtualResistorOutput;

        fn run(&self, sim: SimController<Sky130CommercialPdk, Spectre, Self>) -> Self::Output {
            sim.simulate(
                Options::default(),
                Some(&Sky130Corner::Tt),
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
    let VirtualResistorOutput { current_draw } = ctx.simulate(VirtualResistorTb, sim_dir).unwrap();

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

    let (first, _) = ctx
        .simulate(crate::shared::rc::RcTb::new(dec!(1.4)), &sim_dir)
        .unwrap();
    assert_relative_eq!(first, 1.4);

    let (first, _) = ctx
        .simulate(crate::shared::rc::RcTb::new(dec!(2.1)), sim_dir)
        .unwrap();
    assert_relative_eq!(first, 2.1);
}
