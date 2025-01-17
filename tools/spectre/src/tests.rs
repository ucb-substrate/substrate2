use std::path::{Path, PathBuf};

use approx::{assert_relative_eq, relative_eq};
use num::complex::Complex64;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spice::{BlackboxContents, BlackboxElement, Spice};
use substrate::block::Block;
use substrate::simulation::options::ic;
use substrate::simulation::options::ic::InitialCondition;
use substrate::simulation::waveform::TimeWaveform;
use substrate::simulation::{SimulationContext, Simulator, Testbench};
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

use crate::analysis::ac::{Ac, Sweep};
use crate::analysis::tran::Tran;
use crate::{
    blocks::{AcSource, Capacitor, Isource, RawInstance, Resistor, Vsource},
    ErrPreset, Options, Primitive, Spectre,
};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

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
            dc: dec!(0),
            mag: dec!(1),
            phase: dec!(0),
        }));
        cell.connect(isource.io().p, vout);
        cell.connect(isource.io().n, io.vss);

        Ok(vout)
    }
}

// TODO: uncomment
// impl Testbench<Spectre> for RcTb {
//     type Output = (f64, f64, Complex64);
//     fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {}
// }
//
// fn simulate_rc_tb(ctx: &Context, tb: RcTb, sim_dir: impl Into<PathBuf>) -> (f64, f64, Complex64) {
//     let sim = ctx
//         .get_sim_controller(tb, sim_dir)
//         .expect("failed to create sim controller");
//     let mut opts = Options::default();
//     sim.set_option(
//         InitialCondition {
//             path: sim.tb.data(),
//             value: ic::Voltage(tb.ic),
//         },
//         &mut opts,
//     );
//     let (tran_vout, ac_vout) = sim
//         .simulate(
//             opts,
//             (
//                 Tran {
//                     stop: dec!(10e-6),
//                     ..Default::default()
//                 },
//                 Ac {
//                     start: dec!(1e6),
//                     stop: dec!(2e6),
//                     sweep: Sweep::Linear(10),
//                     errpreset: Some(ErrPreset::Conservative),
//                 },
//             ),
//         )
//         .unwrap();
//
//     let first = tran_vout.first().unwrap();
//     let last = tran_vout.last().unwrap();
//     (*first, *last, ac_vout[2])
// }
//
// #[test]
// fn spectre_initial_condition() {
//     let test_name = "spectre_initial_condition";
//     let sim_dir = get_path(test_name, "sim/");
//     let ctx = spectre_ctx();
//
//     let (first, _, _) = simulate_rc_tb(&ctx, RcTb::new(dec!(1.4)), &sim_dir);
//     assert_relative_eq!(first, 1.4);
//
//     let (first, _, _) = simulate_rc_tb(&ctx, RcTb::new(dec!(2.1)), sim_dir);
//     assert_relative_eq!(first, 2.1);
// }
//
// #[test]
// fn spectre_rc_zin_ac() {
//     let test_name = "spectre_rc_zin_ac";
//     let sim_dir = get_path(test_name, "sim/");
//     let ctx = spectre_ctx();
//
//     let (_, _, z) = simulate_rc_tb(&ctx, RcTb::new(dec!(0)), sim_dir);
//     assert_relative_eq!(z.re, -17.286407017773225);
//     assert_relative_eq!(z.im, 130.3364383055986);
// }
