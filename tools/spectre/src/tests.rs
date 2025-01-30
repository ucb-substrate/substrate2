use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::{assert_relative_eq, relative_eq};
use arcstr::ArcStr;
use cache::multi::MultiCache;
use num::complex::Complex64;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scir::{Cell, Direction, Instance, Library, LibraryBuilder};
use serde::{Deserialize, Serialize};
use spice::netlist::{NetlistKind, NetlistOptions, NetlisterInstance};
use spice::{BlackboxContents, BlackboxElement, Spice};
use substrate::block::Block;
use substrate::cache::Cache;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::simulation::options::ic;
use substrate::simulation::options::ic::InitialCondition;
use substrate::simulation::waveform::TimeWaveform;
use substrate::types::schematic::Terminal;
use substrate::{
    context::Context,
    schematic::{CellBuilder, NestedData, PrimitiveBinding, Schematic},
    simulation::SimController,
    types::{
        schematic::{IoNodeBundle, Node},
        InOut, Io, Signal, TestbenchIo, TwoTerminalIo,
    },
};

use crate::analysis::ac::Ac;
use crate::analysis::dc::DcOp;
use crate::analysis::tran::Tran;
use crate::analysis::Sweep;
use crate::{
    blocks::{AcSource, Capacitor, Isource, RawInstance, Resistor, Vsource},
    ErrPreset, Options, Primitive, Spectre,
};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
const EXAMPLE_SCS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/example_lib.scs");

#[inline]
fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

fn spectre_ctx() -> Context {
    Context::builder().install(Spectre::default()).build()
}

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
        opts.include_section(EXAMPLE_SCS, &sim.tb.block().0);
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

        vout.n.first_x().unwrap()
    }

    let test_name = "spectre_can_include_sections";
    let sim_dir = PathBuf::from(BUILD_DIR).join(test_name).join("sim/");
    let ctx = spectre_ctx();

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
fn spectre_can_save_paths_with_flattened_instances() {
    #[derive(Clone, Debug, Hash, Eq, PartialEq, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct ScirResistor;

    impl Schematic for ScirResistor {
        type Schema = Spectre;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
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

    #[derive(Clone, Debug, Hash, Eq, PartialEq, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct VirtualResistor;

    impl Schematic for VirtualResistor {
        type Schema = Spectre;
        type NestedData = ();

        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            cell.instantiate_connected(ScirResistor, io);
            cell.instantiate_connected(Resistor::new(dec!(200)), io);
            let raw_res = cell.instantiate(RawInstance::with_params(
                arcstr::literal!("resistor"),
                vec![arcstr::literal!("pos"), arcstr::literal!("neg")],
                Vec::from_iter([(arcstr::literal!("r"), dec!(300).into())]),
            ));
            cell.connect(raw_res.io()[0], io.p);
            cell.connect(raw_res.io()[1], io.n);

            Ok(())
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Block)]
    #[substrate(io = "TestbenchIo")]
    struct VirtualResistorTb;

    impl Schematic for VirtualResistorTb {
        type Schema = Spectre;
        type NestedData = Terminal;
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            let vdd = cell.signal("vdd", Signal);
            let dut = cell.instantiate(VirtualResistor);

            cell.connect(dut.io().p, vdd);
            cell.connect(dut.io().n, io.vss);

            let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
            cell.connect(vsource.io().p, vdd);
            cell.connect(vsource.io().n, io.vss);

            Ok(dut.io().p)
        }
    }

    let test_name = "spectre_can_save_paths_with_flattened_instances";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = spectre_ctx();
    let sim = ctx
        .get_sim_controller::<Spectre, _>(VirtualResistorTb, sim_dir)
        .expect("failed to get sim controller");
    let output = sim
        .simulate(
            Options::default(),
            Tran {
                stop: dec!(2e-9),
                errpreset: Some(crate::ErrPreset::Conservative),
                ..Default::default()
            },
        )
        .expect("failed to run simulation");

    assert!(output
        .i
        .values()
        .all(|val| relative_eq!(val.x(), 1.8 * (1. / 100. + 1. / 200. + 1. / 300.))));
}

/// An RC testbench.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct RcTb {
    ic: Decimal,
}

impl RcTb {
    /// Create a new RC testbench with the given initial capacitor value.
    #[inline]
    pub fn new(ic: Decimal) -> Self {
        Self { ic }
    }
}

impl Schematic for RcTb {
    type Schema = Spectre;
    type NestedData = Node;
    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vout = cell.signal("vout", Signal);

        let r = cell.instantiate(Resistor::new(dec!(1000)));
        cell.connect(r.io().p, vout);
        cell.connect(r.io().n, io.vss);

        let c = cell.instantiate(Capacitor::new(dec!(1e-9)));
        cell.connect(c.io().p, vout);
        cell.connect(c.io().n, io.vss);

        let isource = cell.instantiate(Isource::ac(AcSource {
            dc: dec!(5e-3),
            mag: dec!(1),
            phase: dec!(0),
        }));
        cell.connect(isource.io().p, vout);
        cell.connect(isource.io().n, io.vss);

        Ok(vout)
    }
}

fn simulate_rc_tb(
    ctx: &Context,
    tb: RcTb,
    sim_dir: impl Into<PathBuf>,
) -> (f64, f64, Complex64, f64) {
    let sim = ctx
        .get_sim_controller(tb, sim_dir)
        .expect("failed to create sim controller");
    let mut opts = Options::default();
    sim.set_option(
        InitialCondition {
            path: sim.tb.data(),
            value: ic::Voltage(tb.ic),
        },
        &mut opts,
    );
    let (tran_vout, ac_vout, dc_vout) = sim
        .simulate(
            opts,
            (
                Tran {
                    stop: dec!(10e-6),
                    ..Default::default()
                },
                Ac {
                    start: dec!(1e6),
                    stop: dec!(2e6),
                    sweep: Sweep::Linear(10),
                },
                DcOp,
            ),
        )
        .unwrap();

    let first = tran_vout.first_x().unwrap();
    let last = tran_vout.last_x().unwrap();
    (first, last, ac_vout[2], dc_vout)
}

#[test]
fn spectre_initial_condition() {
    let test_name = "spectre_initial_condition";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = spectre_ctx();

    let (first, _, _, _) = simulate_rc_tb(&ctx, RcTb::new(dec!(1.4)), &sim_dir);
    assert_relative_eq!(first, 1.4);

    let (first, _, _, _) = simulate_rc_tb(&ctx, RcTb::new(dec!(2.1)), sim_dir);
    assert_relative_eq!(first, 2.1);
}

#[test]
fn spectre_rc_zin_ac() {
    let test_name = "spectre_rc_zin_ac";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = spectre_ctx();

    let (_, _, z, _) = simulate_rc_tb(&ctx, RcTb::new(dec!(0)), sim_dir);
    assert_relative_eq!(z.re, -17.286407017773225);
    assert_relative_eq!(z.im, 130.3364383055986);
}

#[test]
fn spectre_rc_dcop() {
    let test_name = "spectre_rc_dcop";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = spectre_ctx();

    let (_, _, _, vout) = simulate_rc_tb(&ctx, RcTb::new(dec!(0)), sim_dir);
    assert_relative_eq!(vout, 5.);
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

    simulate_rc_tb(&ctx, RcTb::new(dec!(0)), &sim_dir);
    simulate_rc_tb(&ctx, RcTb::new(dec!(0)), &sim_dir);

    assert_eq!(*count.lock().unwrap(), 1);
}

/// Creates a 1:3 resistive voltage divider.
pub(crate) fn vdivider() -> Library<Spectre> {
    let mut lib = LibraryBuilder::new();
    let res = lib.add_primitive(crate::Primitive::RawInstance {
        cell: ArcStr::from("resistor"),
        ports: vec!["pos".into(), "neg".into()],
        params: vec![(ArcStr::from("r"), dec!(100).into())],
    });

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", res);
    r1.connect("pos", vdd);
    r1.connect("neg", int);
    vdivider.add_instance(r1);

    let mut r2 = Instance::new("r2", res);
    r2.connect("pos", int);
    r2.connect("neg", out);
    vdivider.add_instance(r2);

    let mut r3 = Instance::new("r3", res);
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
fn netlist_spectre_vdivider() {
    let lib = vdivider();
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
