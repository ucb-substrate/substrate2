// begin-code-snippet imports
use super::Inverter;

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
use substrate::simulation::data::FromSaved;
use substrate::simulation::waveform::{EdgeDir, TimeWaveform, WaveformRef};
use substrate::simulation::{SimulationContext, Simulator, Testbench};
// end-code-snippet imports

// begin-code-snippet struct-and-impl
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "Cell")]
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
#[derive(Debug, Clone, FromSaved)]
pub struct NgspiceVout {
    t: ngspice::tran::TranTime,
    v: ngspice::tran::TranVoltage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vout {
    t: Vec<f64>,
    v: Vec<f64>,
}

impl From<NgspiceVout> for Vout {
    fn from(value: NgspiceVout) -> Self {
        Self {
            t: (*value.t).clone(),
            v: (*value.v).clone(),
        }
    }
}

impl substrate::simulation::data::Save<Ngspice, ngspice::tran::Tran, &Cell<InverterTb>>
    for NgspiceVout
{
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: &Cell<InverterTb>,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        Self::Key {
            t: ngspice::tran::TranTime::save(ctx, to_save, opts),
            v: ngspice::tran::TranVoltage::save(ctx, to_save.data(), opts),
        }
    }
}

impl Testbench<Sky130Pdk, Ngspice> for InverterTb {
    type Output = Vout;
    fn run(
        &self,
        sim: substrate::simulation::SimController<Sky130Pdk, Ngspice, Self>,
    ) -> Self::Output {
        let opts = ngspice::Options::default();
        let out: NgspiceVout = sim
            .simulate(
                opts,
                Some(&self.pvt.corner),
                ngspice::tran::Tran {
                    stop: dec!(2e-9),
                    step: dec!(1e-11),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");
        out.into()
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
        InverterTb: Testbench<Sky130Pdk, S, Output = Vout>,
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
        .with_simulator(Ngspice::default())
        .build()
        .with_pdk(Sky130Pdk::open(pdk_root))
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
            pw: (1_200..=5_000).step_by(200).collect(),
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

    #[derive(Debug, Clone, FromSaved)]
    pub struct SpectreVout {
        t: spectre::tran::TranTime,
        v: spectre::tran::TranVoltage,
    }

    impl From<SpectreVout> for Vout {
        fn from(value: SpectreVout) -> Self {
            Self {
                t: (*value.t).clone(),
                v: (*value.v).clone(),
            }
        }
    }

    impl substrate::simulation::data::Save<Spectre, spectre::tran::Tran, &Cell<InverterTb>>
        for SpectreVout
    {
        fn save(
            ctx: &SimulationContext<Spectre>,
            to_save: &Cell<InverterTb>,
            opts: &mut <Spectre as Simulator>::Options,
        ) -> Self::Key {
            Self::Key {
                t: spectre::tran::TranTime::save(ctx, to_save, opts),
                v: spectre::tran::TranVoltage::save(ctx, to_save.data(), opts),
            }
        }
    }

    impl Testbench<Sky130Pdk, Spectre> for InverterTb {
        type Output = Vout;
        fn run(
            &self,
            sim: substrate::simulation::SimController<Sky130Pdk, Spectre, Self>,
        ) -> Self::Output {
            let opts = spectre::Options::default();
            let out: SpectreVout = sim
                .simulate(
                    opts,
                    Some(&self.pvt.corner),
                    spectre::tran::Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(spectre::ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            out.into()
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
            .with_simulator(Spectre::default())
            .build()
            .with_pdk(Sky130Pdk::commercial(pdk_root))
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
                pw: (1_200..=5_000).step_by(200).collect(),
                lch: 150,
            };
            let inv = script.run::<Spectre>(&mut ctx, work_dir);
            println!("Designed inverter:\n{:#?}", inv);
        }
    }
}
// end-code-snippet spectre-support
