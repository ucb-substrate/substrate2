// begin-code-snippet imports
use super::Inverter;

use ngspice::tran::Tran;
use ngspice::Ngspice;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130Pdk;
use std::path::Path;
use substrate::block::Block;
use substrate::context::{Context, PdkContext};
use substrate::io::{Node, SchematicType, Signal, TestbenchIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::waveform::{EdgeDir, TimeWaveform, WaveformRef};
use substrate::simulation::{SimulationContext, Simulator, Testbench};
// end-code-snippet imports

// begin-code-snippet struct-and-impl
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct InverterTb {
    pvt: Pvt<Sky130Corner>,
    dut: Inverter,
}

impl InverterTb {
    #[inline]
    pub fn new(pvt: Pvt<Sky130Corner>, dut: Inverter) -> Self {
        Self { pvt, dut }
    }
}
// end-code-snippet struct-and-impl

// begin-code-snippet schematic
impl ExportsNestedData for InverterTb {
    type NestedData = Node;
}

impl Schematic<Ngspice> for InverterTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Ngspice>,
    ) -> substrate::error::Result<Self::NestedData> {
        let inv = cell.sub_builder::<Sky130Pdk>().instantiate(self.dut);

        let vdd = cell.signal("vdd", Signal);
        let dout = cell.signal("dout", Signal);

        let vddsrc = cell.instantiate(ngspice::blocks::Vsource::dc(self.pvt.voltage));
        cell.connect(vddsrc.io().p, vdd);
        cell.connect(vddsrc.io().n, io.vss);

        let vin = cell.instantiate(ngspice::blocks::Vsource::pulse(ngspice::blocks::Pulse {
            val0: 0.into(),
            val1: self.pvt.voltage,
            delay: Some(dec!(0.1e-9)),
            width: Some(dec!(1e-9)),
            fall: Some(dec!(1e-12)),
            rise: Some(dec!(1e-12)),
            period: None,
            num_pulses: Some(dec!(1)),
        }));
        cell.connect(inv.io().din, vin.io().p);
        cell.connect(vin.io().n, io.vss);

        cell.connect(inv.io().vdd, vdd);
        cell.connect(inv.io().vss, io.vss);
        cell.connect(inv.io().dout, dout);

        Ok(dout)
    }
}
// end-code-snippet schematic

// begin-code-snippet testbench
#[derive(Debug, Clone, Serialize, Deserialize, FromSaved)]
pub struct Vout {
    t: tran::Time,
    v: tran::Voltage,
}

impl SaveTb<Ngspice, ngspice::tran::Tran, Vout> for InverterTb {
    fn save_tb(
        ctx: &SimulationContext<Ngspice>,
        cell: &Cell<Self>,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Vout as FromSaved<Ngspice, Tran>>::SavedKey {
        VoutSavedKey {
            t: tran::Time::save(ctx, (), opts),
            v: tran::Voltage::save(ctx, cell.data(), opts),
        }
    }
}

impl Testbench<Ngspice> for InverterTb {
    type Output = Vout;
    fn run(&self, sim: substrate::simulation::SimController<Ngspice, Self>) -> Self::Output {
        let mut opts = ngspice::Options::default();
        sim.set_option(self.pvt.corner, &mut opts);
        sim.simulate(
            opts,
            ngspice::tran::Tran {
                stop: dec!(2e-9),
                step: dec!(1e-11),
                ..Default::default()
            },
        )
        .expect("failed to run simulation")
    }
}
// end-code-snippet testbench

// begin-code-snippet design
/// Designs an inverter for balanced pull-up and pull-down times.
///
/// The NMOS width is kept constant; the PMOS width is swept over
/// the given range.
pub struct InverterDesign {
    /// The fixed NMOS width.
    pub nw: i64,
    /// The set of PMOS widths to sweep.
    pub pw: Vec<i64>,
    /// The transistor channel length.
    pub lch: i64,
}

impl InverterDesign {
    pub fn run<S: Simulator>(
        &self,
        ctx: &mut PdkContext<Sky130Pdk>,
        work_dir: impl AsRef<Path>,
    ) -> Inverter
    where
        InverterTb: Testbench<S, Output = Vout>,
    {
        let work_dir = work_dir.as_ref();
        let pvt = Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25));

        let mut opt = None;
        for pw in self.pw.iter().copied() {
            let dut = Inverter {
                nw: self.nw,
                pw,
                lch: self.lch,
            };
            let tb = InverterTb::new(pvt, dut);
            let output = ctx
                .simulate(tb, work_dir.join(format!("pw{pw}")))
                .expect("failed to run simulation");

            let vout = WaveformRef::new(&output.t, &output.v);
            let mut trans = vout.transitions(
                0.2 * pvt.voltage.to_f64().unwrap(),
                0.8 * pvt.voltage.to_f64().unwrap(),
            );
            // The input waveform has a low -> high, then a high -> low transition.
            // So the first transition of the inverter output is high -> low.
            // The duration of this transition is the inverter fall time.
            let falling_transition = trans.next().unwrap();
            assert_eq!(falling_transition.dir(), EdgeDir::Falling);
            let tf = falling_transition.duration();
            let rising_transition = trans.next().unwrap();
            assert_eq!(rising_transition.dir(), EdgeDir::Rising);
            let tr = rising_transition.duration();

            println!("Simulating with pw = {pw} gave tf = {}, tr = {}", tf, tr);
            let diff = (tr - tf).abs();
            if let Some((pdiff, _)) = opt {
                if diff < pdiff {
                    opt = Some((diff, dut));
                }
            } else {
                opt = Some((diff, dut));
            }
        }

        opt.unwrap().1
    }
}
// end-code-snippet design

// begin-code-snippet sky130-open-ctx
/// Create a new Substrate context for the SKY130 open PDK.
///
/// Sets the PDK root to the value of the `SKY130_OPEN_PDK_ROOT`
/// environment variable and installs Spectre with default configuration.
///
/// # Panics
///
/// Panics if the `SKY130_OPEN_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_open_ctx() -> PdkContext<Sky130Pdk> {
    let pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(Ngspice::default())
        .install(Sky130Pdk::open(pdk_root))
        .build()
        .with_pdk()
}
// end-code-snippet sky130-open-ctx

// begin-code-snippet tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn design_inverter_ngspice() {
        let work_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/design_inverter_ngspice");
        let mut ctx = sky130_open_ctx();
        let script = InverterDesign {
            nw: 1_200,
            pw: (3_000..=5_000).step_by(200).collect(),
            lch: 150,
        };

        let inv = script.run::<Ngspice>(&mut ctx, work_dir);
        println!("Designed inverter:\n{:#?}", inv);
    }
}
// end-code-snippet tests

// begin-code-snippet spectre-support
#[cfg(feature = "spectre")]
pub mod spectre_support {
    use super::*;
    use spectre::Spectre;

    impl Schematic<Spectre> for InverterTb {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut CellBuilder<Spectre>,
        ) -> substrate::error::Result<Self::NestedData> {
            let inv = cell.sub_builder::<Sky130Pdk>().instantiate(self.dut);

            let vdd = cell.signal("vdd", Signal);
            let dout = cell.signal("dout", Signal);

            let vddsrc = cell.instantiate(spectre::blocks::Vsource::dc(self.pvt.voltage));
            cell.connect(vddsrc.io().p, vdd);
            cell.connect(vddsrc.io().n, io.vss);

            let vin = cell.instantiate(spectre::blocks::Vsource::pulse(spectre::blocks::Pulse {
                val0: 0.into(),
                val1: self.pvt.voltage,
                delay: Some(dec!(0.1e-9)),
                width: Some(dec!(1e-9)),
                fall: Some(dec!(1e-12)),
                rise: Some(dec!(1e-12)),
                period: None,
            }));
            cell.connect(inv.io().din, vin.io().p);
            cell.connect(vin.io().n, io.vss);

            cell.connect(inv.io().vdd, vdd);
            cell.connect(inv.io().vss, io.vss);
            cell.connect(inv.io().dout, dout);

            Ok(dout)
        }
    }

    impl substrate::simulation::data::SaveTb<Spectre, spectre::tran::Tran, Vout> for InverterTb {
        fn save_tb(
            ctx: &SimulationContext<Spectre>,
            cell: &Cell<Self>,
            opts: &mut <Spectre as Simulator>::Options,
        ) -> <Vout as FromSaved<Spectre, spectre::tran::Tran>>::SavedKey {
            VoutSavedKey {
                t: tran::Time::save(ctx, (), opts),
                v: tran::Voltage::save(ctx, cell.data(), opts),
            }
        }
    }

    impl Testbench<Spectre> for InverterTb {
        type Output = Vout;
        fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
            let mut opts = spectre::Options::default();
            sim.set_option(self.pvt.corner, &mut opts);
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

    /// Create a new Substrate context for the SKY130 commercial PDK.
    ///
    /// Sets the PDK root to the value of the `SKY130_COMMERCIAL_PDK_ROOT`
    /// environment variable and installs Spectre with default configuration.
    ///
    /// # Panics
    ///
    /// Panics if the `SKY130_COMMERCIAL_PDK_ROOT` environment variable is not set,
    /// or if the value of that variable is not a valid UTF-8 string.
    pub fn sky130_commercial_ctx() -> PdkContext<Sky130Pdk> {
        let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
            .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
        Context::builder()
            .install(Spectre::default())
            .install(Sky130Pdk::commercial(pdk_root))
            .build()
            .with_pdk()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        pub fn design_inverter_spectre() {
            let work_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/design_inverter_spectre");
            let mut ctx = sky130_commercial_ctx();
            let script = InverterDesign {
                nw: 1_200,
                pw: (3_000..=5_000).step_by(200).collect(),
                lch: 150,
            };
            let inv = script.run::<Spectre>(&mut ctx, work_dir);
            println!("Designed inverter:\n{:#?}", inv);
        }
    }
}
// end-code-snippet spectre-support
