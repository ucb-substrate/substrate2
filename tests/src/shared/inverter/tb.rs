use std::path::Path;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130Pdk;
use spectre::analysis::tran::Tran;
use spectre::blocks::{Pulse, Vsource};
use spectre::{Options, Spectre};
use substrate::block::Block;
use substrate::context::PdkContext;
use substrate::io::schematic::{HardwareType, Node};
use substrate::io::Signal;
use substrate::io::TestbenchIo;
use substrate::pdk::corner::Pvt;
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::waveform::{EdgeDir, TimeWaveform, WaveformRef};
use substrate::simulation::{SimulationContext, Simulator, Testbench};

use super::Inverter;

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

impl ExportsNestedData for InverterTb {
    type NestedData = Node;
}

impl Schematic<Spectre> for InverterTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut sub_cell = cell.sub_builder::<Sky130Pdk>();
        let inv = sub_cell.instantiate(self.dut);

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
        }));
        cell.connect(inv.io().din, vin.io().p);
        cell.connect(vin.io().n, io.vss);

        cell.connect(inv.io().vdd, vdd);
        cell.connect(inv.io().vss, io.vss);
        cell.connect(inv.io().dout, dout);

        Ok(dout)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverterTbData {
    pub tr: f64,
    pub tf: f64,
}

#[derive(FromSaved, Serialize, Deserialize)]
pub struct InverterTbOutput {
    pub time: tran::Time,
    pub vout: tran::Voltage,
}

impl SaveTb<Spectre, Tran, InverterTbOutput> for InverterTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <InverterTbOutput as FromSaved<Spectre, Tran>>::SavedKey {
        InverterTbOutputSavedKey {
            time: tran::Time::save(ctx, (), opts),
            vout: tran::Voltage::save(ctx, cell.data(), opts),
        }
    }
}

impl Testbench<Spectre> for InverterTb {
    type Output = InverterTbData;
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        let mut opts = Options::default();
        sim.set_option(self.pvt.corner, &mut opts);
        let output: InverterTbOutput = sim
            .simulate(
                opts,
                Tran {
                    stop: dec!(2e-9),
                    errpreset: Some(spectre::ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let vout = output.vout;
        let time = &output.time;
        let vout = WaveformRef::new(time, &vout);
        let mut trans = vout.transitions(
            0.2 * self.pvt.voltage.to_f64().unwrap(),
            0.8 * self.pvt.voltage.to_f64().unwrap(),
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

        InverterTbData { tf, tr }
    }
}

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
    pub fn run(&self, ctx: &mut PdkContext<Sky130Pdk>, work_dir: impl AsRef<Path>) -> Inverter {
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
            let data = ctx.simulate(tb, work_dir.join(format!("pw{pw}"))).unwrap();
            println!("Simulating with pw = {pw} gave:\n{:#?}", data);
            let diff = (data.tr - data.tf).abs();
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
