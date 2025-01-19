// begin-code-snippet imports
use crate::Inverter;
use crate::InverterIoKind;
use crate::SKY130_MAGIC_TECH_FILE;
use crate::SKY130_NETGEN_SETUP_FILE;

use magic_netgen::Pex;
use ngspice::Ngspice;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use sky130pdk::corner::Sky130Corner;
use sky130pdk::layout::to_gds;
use sky130pdk::Sky130OpenSchema;
use spice::Spice;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use substrate::block::Block;
use substrate::context::Context;
use substrate::error::Result;
use substrate::schematic::{CellBuilder, ConvertSchema, Schematic};
use substrate::simulation::waveform::{EdgeDir, TimeWaveform};
use substrate::simulation::Pvt;
use substrate::types::schematic::{IoNodeBundle, Node};
use substrate::types::{Signal, TestbenchIo};
// end-code-snippet imports

#[allow(dead_code)]
mod schematic_only_tb {
    use ngspice::blocks::{Pulse, Vsource};
    use sky130pdk::Sky130OpenSchema;

    use super::*;

    // begin-code-snippet struct-and-impl
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Block)]
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
    impl Schematic for InverterTb {
        type Schema = Ngspice;
        type NestedData = Node;
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> Result<Self::NestedData> {
            let inv = cell
                .sub_builder::<Sky130OpenSchema>()
                .instantiate(ConvertSchema::new(self.dut));

            let vdd = cell.signal("vdd", Signal);
            let dout = cell.signal("dout", Signal);

            let vddsrc = cell.instantiate(Vsource::dc(self.pvt.voltage));
            cell.connect(vddsrc.io().p, vdd);
            cell.connect(vddsrc.io().n, io.vss);

            let vin = cell.instantiate(Vsource::pulse(Pulse {
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

    // begin-code-snippet schematic-design-script
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
        pub fn run(&self, ctx: &mut Context, work_dir: impl AsRef<Path>) -> Inverter {
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
                let sim_dir = work_dir.join(format!("pw{pw}"));
                let sim = ctx
                    .get_sim_controller(tb, sim_dir)
                    .expect("failed to create sim controller");
                let mut opts = ngspice::Options::default();
                sim.set_option(pvt.corner, &mut opts);
                let output = sim
                    .simulate(
                        opts,
                        ngspice::tran::Tran {
                            stop: dec!(2e-9),
                            step: dec!(1e-11),
                            ..Default::default()
                        },
                    )
                    .expect("failed to run simulation");

                let vout = output.as_ref();
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
    // end-code-snippet schematic-design-script

    // begin-code-snippet schematic-tests
    #[cfg(test)]
    mod tests {
        use crate::sky130_open_ctx;

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

            let inv = script.run(&mut ctx, work_dir);
            println!("Designed inverter:\n{:#?}", inv);
        }
    }
    // end-code-snippet schematic-tests
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum InverterDut {
    Schematic(Inverter),
    Extracted(Pex<ConvertSchema<ConvertSchema<Inverter, Sky130OpenSchema>, Spice>>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Block)]
#[substrate(io = "TestbenchIo")]
pub struct InverterTb {
    pvt: Pvt<Sky130Corner>,
    dut: InverterDut,
}

impl InverterTb {
    #[inline]
    pub fn new(pvt: Pvt<Sky130Corner>, dut: impl Into<InverterDut>) -> Self {
        Self {
            pvt,
            dut: dut.into(),
        }
    }
}

impl Schematic for InverterTb {
    type Schema = Ngspice;
    type NestedData = Node;
    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> Result<Self::NestedData> {
        let invio = cell.signal(
            "dut",
            InverterIoKind {
                vdd: Signal,
                vss: Signal,
                din: Signal,
                dout: Signal,
            },
        );

        match self.dut.clone() {
            InverterDut::Schematic(inv) => {
                cell.sub_builder::<Sky130OpenSchema>()
                    .instantiate_connected_named(ConvertSchema::new(inv), &invio, "inverter");
            }
            InverterDut::Extracted(inv) => {
                cell.sub_builder::<Spice>()
                    .instantiate_connected_named(inv, &invio, "inverter");
            }
        };

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
        cell.connect(invio.din, vin.io().p);
        cell.connect(vin.io().n, io.vss);

        cell.connect(invio.vdd, vdd);
        cell.connect(invio.vss, io.vss);
        cell.connect(invio.dout, dout);

        Ok(dout)
    }
}

// begin-code-snippet design-extracted
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
    /// Whether or not to run extracted simulations.
    pub extracted: bool,
}

impl InverterDesign {
    pub fn run(&self, ctx: &mut Context, work_dir: impl AsRef<Path>) -> Inverter {
        let work_dir = work_dir.as_ref();
        let pvt = Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25));

        let mut opt = None;
        for pw in self.pw.iter().copied() {
            let dut = Inverter {
                nw: self.nw,
                pw,
                lch: self.lch,
            };
            let inverter = if self.extracted {
                let work_dir = work_dir.join(format!("pw{pw}"));
                let layout_path = work_dir.join("layout.gds");
                ctx.write_layout(dut, to_gds, &layout_path)
                    .expect("failed to write layout");
                InverterDut::Extracted(Pex {
                    schematic: Arc::new(ConvertSchema::new(ConvertSchema::new(dut))),
                    gds_path: work_dir.join("layout.gds"),
                    layout_cell_name: dut.name(),
                    work_dir,
                    magic_tech_file_path: PathBuf::from(SKY130_MAGIC_TECH_FILE),
                    netgen_setup_file_path: PathBuf::from(SKY130_NETGEN_SETUP_FILE),
                })
            } else {
                InverterDut::Schematic(dut)
            };
            let tb = InverterTb::new(pvt, inverter);
            let sim_dir = work_dir.join(format!("pw{pw}"));
            let sim = ctx
                .get_sim_controller(tb, sim_dir)
                .expect("failed to create sim controller");
            let mut opts = ngspice::Options::default();
            sim.set_option(pvt.corner, &mut opts);
            let output = sim
                .simulate(
                    opts,
                    ngspice::tran::Tran {
                        stop: dec!(2e-9),
                        step: dec!(1e-11),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            let vout = output.as_ref();
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
// end-code-snippet design-extracted

// begin-code-snippet tests-extracted
#[cfg(test)]
mod tests {
    use crate::sky130_open_ctx;

    use super::*;

    #[test]
    pub fn design_inverter_ngspice_extracted() {
        let work_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/design_inverter_ngspice_extracted"
        );
        let mut ctx = sky130_open_ctx();
        let script = InverterDesign {
            nw: 1_200,
            pw: (3_000..=5_000).step_by(200).collect(),
            lch: 150,
            extracted: true,
        };

        let inv = script.run(&mut ctx, work_dir);
        println!("Designed inverter:\n{:#?}", inv);
    }

    #[test]
    pub fn design_inverter_ngspice_schematic() {
        let work_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/design_inverter_ngspice_schematic"
        );
        let mut ctx = sky130_open_ctx();
        let script = InverterDesign {
            nw: 1_200,
            pw: (3_000..=5_000).step_by(200).collect(),
            lch: 150,
            extracted: false,
        };

        let inv = script.run(&mut ctx, work_dir);
        println!("Designed inverter:\n{:#?}", inv);
    }
}
// end-code-snippet tests-extracted
