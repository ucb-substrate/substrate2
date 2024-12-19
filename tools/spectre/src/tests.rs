use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::*;
use spectre::Spectre;
use spice::netlist::{NetlistKind, NetlistOptions, NetlisterInstance};
use spice::{BlackboxContents, BlackboxElement, ComponentValue, Spice};
use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::schema::Schema;

use approx::assert_relative_eq;
use rust_decimal_macros::dec;
use spice::{BlackboxContents, BlackboxElement};
use substrate::{
    block::Block,
    context::Context,
    schematic::{CellBuilder, NestedData, PrimitiveBinding, Schematic},
    simulation::{data::Save, Analysis, SimController, Simulator},
    types::{
        schematic::{IoNodeBundle, NestedNode, Node},
        InOut, Io, Signal, TestbenchIo,
    },
};

use crate::{
    analysis::tran::Tran,
    blocks::{Resistor, Vsource},
    ErrPreset, Options, Primitive, Spectre,
};

pub const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
pub const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

#[test]
fn spectre_can_include_sections() {
    #[derive(Default, Clone, Io)]
    struct LibIncludeResistorIo {
        p: InOut<Signal>,
        n: InOut<Signal>,
    }

    #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Block)]
    #[substrate(io = "LibIncludeResistorIo")]
    struct LibIncludeResistor;

    impl Schematic for LibIncludeResistor {
        type Schema = Spectre;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
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

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Block)]
    #[substrate(io = "TestbenchIo")]
    struct LibIncludeTb(String);

    #[derive(Debug, Clone, Copy, NestedData)]
    struct LibIncludeTbData {
        n: Node,
    }

    impl Schematic for LibIncludeTb {
        type Schema = Spectre;
        type NestedData = LibIncludeTbData;
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
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

            Ok(LibIncludeTbData { n: *dut.io().n })
        }
    }

    fn run(sim: SimController<Spectre, LibIncludeTb>) -> f64 {
        let mut opts = Options::default();
        opts.include_section(
            PathBuf::from(TEST_DATA_DIR).join("spectre/example_lib.scs"),
            &sim.tb.block().0,
        );
        let vout = sim
            .simulate(
                opts,
                Tran {
                    stop: dec!(2e-9),
                    errpreset: Some(ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        *vout.n.first().unwrap()
    }

    let test_name = "spectre_can_include_sections";
    let sim_dir = PathBuf::from(BUILD_DIR).join(test_name).join("sim/");
    let ctx = Context::builder().install(Spectre::default()).build();

    let output_tt = run(ctx
        .get_sim_controller(LibIncludeTb("section_a".to_string()), &sim_dir)
        .unwrap());
    let output_ss = run(ctx
        .get_sim_controller(LibIncludeTb("section_b".to_string()), sim_dir)
        .unwrap());

    assert_relative_eq!(output_tt, 0.9);
    assert_relative_eq!(output_ss, 1.2);
}

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

pub(crate) trait HasRes2: Schema {
    fn resistor(value: usize) -> <Self as Schema>::Primitive;
    fn pos() -> &'static str;
    fn neg() -> &'static str;
}

impl HasRes2 for Spice {
    fn resistor(value: usize) -> spice::Primitive {
        spice::Primitive::Res2 {
            value: ComponentValue::Fixed(Decimal::from(value)),
            params: Default::default(),
        }
    }
    fn pos() -> &'static str {
        "1"
    }
    fn neg() -> &'static str {
        "2"
    }
}

impl HasRes2 for Spectre {
    fn resistor(value: usize) -> spectre::Primitive {
        spectre::Primitive::RawInstance {
            cell: ArcStr::from("resistor"),
            ports: vec!["pos".into(), "neg".into()],
            params: HashMap::from_iter([(ArcStr::from("r"), Decimal::from(value).into())]),
        }
    }
    fn pos() -> &'static str {
        "pos"
    }
    fn neg() -> &'static str {
        "neg"
    }
}

/// Creates a 1:3 resistive voltage divider.
pub(crate) fn vdivider<S: HasRes2>() -> Library<S> {
    let mut lib = LibraryBuilder::new();
    let res = lib.add_primitive(S::resistor(100));

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", res);
    r1.connect(S::pos(), vdd);
    r1.connect(S::neg(), int);
    vdivider.add_instance(r1);

    let mut r2 = Instance::new("r2", res);
    r2.connect(S::pos(), int);
    r2.connect(S::neg(), out);
    vdivider.add_instance(r2);

    let mut r3 = Instance::new("r3", res);
    r3.connect(S::pos(), out);
    r3.connect(S::neg(), vss);
    vdivider.add_instance(r3);

    vdivider.expose_port(vdd, Direction::InOut);
    vdivider.expose_port(vss, Direction::InOut);
    vdivider.expose_port(out, Direction::Output);
    lib.add_cell(vdivider);

    lib.build().unwrap()
}

/// Creates a 1:3 resistive voltage divider using blackboxed resistors.
pub(crate) fn vdivider_blackbox() -> Library<Spice> {
    let mut lib = LibraryBuilder::new();
    let wrapper = lib.add_primitive(spice::Primitive::BlackboxInstance {
        contents: BlackboxContents {
            elems: vec![
                "R".into(),
                BlackboxElement::InstanceName,
                " ".into(),
                BlackboxElement::Port("pos".into()),
                " ".into(),
                BlackboxElement::Port("neg".into()),
                " 3300".into(),
            ],
        },
    });

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", wrapper);
    r1.connect("pos", vdd);
    r1.connect("neg", int);
    vdivider.add_instance(r1);

    let mut r2 = Instance::new("r2", wrapper);
    r2.connect("pos", int);
    r2.connect("neg", out);
    vdivider.add_instance(r2);

    let mut r3 = Instance::new("r3", wrapper);
    r3.connect("pos", out);
    r3.connect("neg", vss);
    vdivider.add_instance(r3);

    vdivider.expose_port(vdd, Direction::InOut);
    vdivider.expose_port(vss, Direction::InOut);
    vdivider.expose_port(out, Direction::Output);
    lib.add_cell(vdivider);

    lib.build().unwrap()
}

#[test]
fn vdivider_is_valid() {
    let lib = vdivider::<Spice>();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}

#[test]
fn vdivider_blackbox_is_valid() {
    let lib = vdivider_blackbox();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}

#[test]
fn netlist_spice_vdivider() {
    let lib = vdivider::<Spice>();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rr1").count(), 1);
    assert_eq!(string.matches("Rr2").count(), 1);
    assert_eq!(string.matches("Rr3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
}

#[test]
fn netlist_spice_vdivider_is_repeatable() {
    let lib = vdivider::<Spice>();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let golden = String::from_utf8(buf).unwrap();

    for i in 0..100 {
        let lib = vdivider::<Spice>();
        let mut buf: Vec<u8> = Vec::new();
        Spice
            .write_scir_netlist(&lib, &mut buf, Default::default())
            .unwrap();
        let attempt = String::from_utf8(buf).unwrap();
        assert_eq!(
            attempt, golden,
            "netlister output changed even though the inputs were the same (iteration {i})"
        );
    }
}

#[test]
fn netlist_spice_vdivider_blackbox() {
    let lib = vdivider_blackbox();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rr1").count(), 1);
    assert_eq!(string.matches("Rr2").count(), 1);
    assert_eq!(string.matches("Rr3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
    assert_eq!(string.matches("3300").count(), 3);
}

#[test]
fn netlist_spectre_vdivider() {
    let lib = vdivider::<Spectre>();
    let mut buf: Vec<u8> = Vec::new();
    let includes = Vec::new();
    NetlisterInstance::new(
        &Spectre {},
        &lib,
        &mut buf,
        NetlistOptions::new(NetlistKind::Cells, &includes),
    )
    .export()
    .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a Spectre netlist parser, we can parse the Spectre back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("subckt").count(), 1);
    assert_eq!(string.matches("ends").count(), 1);
    assert_eq!(string.matches("r1").count(), 1);
    assert_eq!(string.matches("r2").count(), 1);
    assert_eq!(string.matches("r3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
    assert_eq!(string.matches("resistor r=100").count(), 3);
}
