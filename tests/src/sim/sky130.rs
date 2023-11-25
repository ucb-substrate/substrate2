use crate::paths::get_path;
use crate::shared::pdk::{sky130_commercial_ctx, sky130_open_ctx};
use approx::assert_abs_diff_eq;
use ngspice::Ngspice;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::stdcells::And2;
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::block::Block;
use substrate::io::{PowerIoSchematic, SchematicType, Terminal, TestbenchIo};
use substrate::schematic::primitives::DcVsource;
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::{SimController, SimulationContext, Simulator, Testbench};
use substrate::type_dispatch::impl_dispatch;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct And2Tb {
    vdd: Decimal,
    a: Decimal,
    b: Decimal,
}

impl ExportsNestedData for And2Tb {
    type NestedData = Terminal;
}

#[impl_dispatch({Ngspice; Spectre})]
impl<S> Schematic<S> for And2Tb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<S>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vddsrc = cell.instantiate(DcVsource::new(self.vdd));
        let asrc = cell.instantiate(DcVsource::new(self.a));
        let bsrc = cell.instantiate(DcVsource::new(self.b));
        let and2 = cell
            .sub_builder::<Sky130Pdk>()
            .instantiate_blocking(And2::S0)
            .unwrap();

        let pwr = PowerIoSchematic {
            vdd: *vddsrc.io().p,
            vss: *vddsrc.io().n,
        };

        cell.connect(io.vss, vddsrc.io().n);
        cell.connect_multiple(&[vddsrc.io().n, asrc.io().n, bsrc.io().n]);
        cell.connect(
            &and2.io().pwr,
            sky130pdk::stdcells::PowerIoSchematic::with_bodies_tied_to_rails(pwr),
        );
        cell.connect(and2.io().a, asrc.io().p);
        cell.connect(and2.io().b, bsrc.io().p);

        Ok(and2.io().x)
    }
}

#[impl_dispatch({Spectre, spectre::tran::Tran; Ngspice, ngspice::tran::Tran})]
impl<S, A> SaveTb<S, A, tran::Voltage> for And2Tb {
    fn save_tb(
        ctx: &SimulationContext<S>,
        cell: &Cell<Self>,
        opts: &mut <S as Simulator>::Options,
    ) -> <tran::Voltage as FromSaved<S, A>>::SavedKey {
        tran::Voltage::save(ctx, cell.data(), opts)
    }
}

impl Testbench<Ngspice> for And2Tb {
    type Output = tran::Voltage;

    fn run(&self, sim: SimController<Ngspice, Self>) -> Self::Output {
        let mut opts = ngspice::Options::default();
        sim.set_option(Sky130Corner::Tt, &mut opts);
        sim.simulate(
            opts,
            ngspice::tran::Tran {
                step: dec!(1e-9),
                stop: dec!(2e-9),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}
impl Testbench<Spectre> for And2Tb {
    type Output = tran::Voltage;

    fn run(&self, sim: SimController<Spectre, Self>) -> Self::Output {
        let mut opts = spectre::Options::default();
        sim.set_option(Sky130Corner::Tt, &mut opts);
        sim.simulate(
            opts,
            spectre::tran::Tran {
                stop: dec!(2e-9),
                errpreset: Some(spectre::ErrPreset::Conservative),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}

#[test]
fn sky130_and2_ngspice() {
    let test_name = "sky130_and2_ngspice";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_open_ctx();

    for (a, b, expected) in [(dec!(1.8), dec!(1.8), 1.8f64), (dec!(1.8), dec!(0), 0f64)] {
        let vout = ctx
            .simulate::<Ngspice, _>(
                And2Tb {
                    vdd: dec!(1.8),
                    a,
                    b,
                },
                &sim_dir,
            )
            .unwrap();
        assert_abs_diff_eq!(*vout.last().unwrap(), expected, epsilon = 1e-6);
    }
}

#[cfg(feature = "spectre")]
#[test]
fn sky130_and2_spectre() {
    let test_name = "sky130_and2_spectre";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();

    for (a, b, expected) in [(dec!(1.8), dec!(1.8), 1.8f64), (dec!(1.8), dec!(0), 0f64)] {
        let vout = ctx
            .simulate::<spectre::Spectre, _>(
                And2Tb {
                    vdd: dec!(1.8),
                    a,
                    b,
                },
                &sim_dir,
            )
            .unwrap();
        assert_abs_diff_eq!(*vout.last().unwrap(), expected, epsilon = 1e-6);
    }
}
