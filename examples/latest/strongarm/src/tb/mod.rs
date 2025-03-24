pub mod cds;
pub mod open;

//! StrongARM testbenches.

use approx::abs_diff_eq;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use spectre::analysis::tran::Tran;
use spectre::blocks::{Pulse, Vsource};
use spectre::{ErrPreset, Spectre};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use substrate::arcstr;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::io::schematic::{Bundle, HardwareType, Node};
use substrate::io::{DiffPair, Signal, TestbenchIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::schema::Schema;
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, NestedData, Schematic};
use substrate::scir::schema::FromSchema;
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::options::{SimOption, Temperature};
use substrate::simulation::waveform::{EdgeDir, TimeWaveform, WaveformRef};
use substrate::simulation::{SimController, SimulationContext, Simulator, Testbench};

use crate::strongarm::ClockedDiffComparatorIo;

/// A transient testbench that provides a differential input voltage and
/// measures the output waveform.
#[derive_where::derive_where(Copy, Clone, Debug, Hash, PartialEq, Eq; T, C)]
#[derive(Serialize, Deserialize)]
pub struct StrongArmTranTb<T, PDK, C> {
    /// The device-under-test.
    pub dut: T,

    /// The positive input voltage.
    pub vinp: Decimal,

    /// The negative input voltage.
    pub vinn: Decimal,

    /// Whether to pass an inverted clock to the DUT.
    ///
    /// If set to true, the clock will be held high when idle.
    /// The DUT should perform a comparison in response to a falling clock edge,
    /// rather than a rising clock edge.
    pub inverted_clk: bool,

    /// The PVT corner.
    pub pvt: Pvt<C>,

    #[serde(bound(deserialize = ""))]
    phantom: PhantomData<fn() -> PDK>,
}

impl<T, PDK, C> StrongArmTranTb<T, PDK, C> {
    /// Creates a new [`StrongArmTranTb`].
    pub fn new(dut: T, vinp: Decimal, vinn: Decimal, inverted_clk: bool, pvt: Pvt<C>) -> Self {
        Self {
            dut,
            vinp,
            vinn,
            pvt,
            inverted_clk,
            phantom: PhantomData,
        }
    }
}

impl<
        T: Block,
        PDK: Any,
        C: Serialize
            + DeserializeOwned
            + Copy
            + Clone
            + Debug
            + Hash
            + PartialEq
            + Eq
            + Send
            + Sync
            + Any,
    > Block for StrongArmTranTb<T, PDK, C>
{
    type Io = TestbenchIo;

    fn id() -> ArcStr {
        arcstr::literal!("strong_arm_tran_tb")
    }

    fn name(&self) -> ArcStr {
        arcstr::literal!("strong_arm_tran_tb")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

/// Nodes measured by [`StrongArmTranTb`].
#[derive(Clone, Debug, Hash, PartialEq, Eq, NestedData)]
pub struct StrongArmTranTbNodes {
    vop: Node,
    von: Node,
    vinn: Node,
    vinp: Node,
    clk: Node,
}

impl<T, PDK, C> ExportsNestedData for StrongArmTranTb<T, PDK, C>
where
    StrongArmTranTb<T, PDK, C>: Block,
{
    type NestedData = StrongArmTranTbNodes;
}

impl<T: Block<Io = ClockedDiffComparatorIo> + Schematic<PDK> + Clone, PDK: Schema, C>
    Schematic<Spectre> for StrongArmTranTb<T, PDK, C>
where
    StrongArmTranTb<T, PDK, C>: Block<Io = TestbenchIo>,
    Spectre: FromSchema<PDK>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let dut = cell.sub_builder::<PDK>().instantiate(self.dut.clone());

        let vinp = cell.signal("vinp", Signal);
        let vinn = cell.signal("vinn", Signal);
        let vdd = cell.signal("vdd", Signal);
        let clk = cell.signal("clk", Signal);

        let vvinp = cell.instantiate(Vsource::dc(self.vinp));
        let vvinn = cell.instantiate(Vsource::dc(self.vinn));
        let vvdd = cell.instantiate(Vsource::dc(self.pvt.voltage));
        let (val0, val1) = if self.inverted_clk {
            (self.pvt.voltage, dec!(0))
        } else {
            (dec!(0), self.pvt.voltage)
        };
        let vclk = cell.instantiate(Vsource::pulse(Pulse {
            val0,
            val1,
            period: Some(dec!(1000)),
            width: Some(dec!(100)),
            delay: Some(dec!(10e-9)),
            rise: Some(dec!(100e-12)),
            fall: Some(dec!(100e-12)),
        }));

        cell.connect(io.vss, vvinp.io().n);
        cell.connect(io.vss, vvinn.io().n);
        cell.connect(io.vss, vvdd.io().n);
        cell.connect(io.vss, vclk.io().n);
        cell.connect(vinp, vvinp.io().p);
        cell.connect(vinn, vvinn.io().p);
        cell.connect(vdd, vvdd.io().p);
        cell.connect(clk, vclk.io().p);

        let output = cell.signal("output", DiffPair::default());

        cell.connect(
            Bundle::<ClockedDiffComparatorIo> {
                input: Bundle::<DiffPair> { p: vinp, n: vinn },
                output: output.clone(),
                clock: clk,
                vdd,
                vss: io.vss,
            },
            dut.io(),
        );

        Ok(StrongArmTranTbNodes {
            vop: output.p,
            von: output.n,
            vinn,
            vinp,
            clk,
        })
    }
}

/// The resulting waveforms of a [`StrongArmTranTb`].
#[derive(Debug, Clone, Serialize, Deserialize, FromSaved)]
pub struct ComparatorSim {
    t: tran::Time,
    vop: tran::Voltage,
    von: tran::Voltage,
    vinn: tran::Voltage,
    vinp: tran::Voltage,
    clk: tran::Voltage,
}

/// The decision made by a comparator.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub enum ComparatorDecision {
    /// Negative.
    ///
    /// The negative input was larger than the positive input.
    Neg,
    /// Positive.
    ///
    /// The positive input was larger than the negative input.
    Pos,
}

impl<T, PDK, C> SaveTb<Spectre, Tran, ComparatorSim> for StrongArmTranTb<T, PDK, C>
where
    StrongArmTranTb<T, PDK, C>: Block<Io = TestbenchIo>,
{
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <ComparatorSim as FromSaved<Spectre, Tran>>::SavedKey {
        ComparatorSimSavedKey {
            t: tran::Time::save(ctx, (), opts),
            vop: tran::Voltage::save(ctx, cell.data().vop, opts),
            von: tran::Voltage::save(ctx, cell.data().von, opts),
            vinn: tran::Voltage::save(ctx, cell.data().vinn, opts),
            vinp: tran::Voltage::save(ctx, cell.data().vinp, opts),
            clk: tran::Voltage::save(ctx, cell.data().clk, opts),
        }
    }
}

impl<T, PDK, C: SimOption<Spectre> + Copy> Testbench<Spectre> for StrongArmTranTb<T, PDK, C>
where
    StrongArmTranTb<T, PDK, C>: Block<Io = TestbenchIo> + Schematic<Spectre>,
{
    type Output = Option<ComparatorDecision>;

    fn run(&self, sim: SimController<Spectre, Self>) -> Self::Output {
        let mut opts = spectre::Options::default();
        sim.set_option(self.pvt.corner, &mut opts);
        sim.set_option(Temperature::from(self.pvt.temp), &mut opts);
        let wav: ComparatorSim = sim
            .simulate(
                opts,
                Tran {
                    stop: dec!(30e-9),
                    start: None,
                    errpreset: Some(ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let von = *wav.von.last().unwrap();
        let vop = *wav.vop.last().unwrap();

        let vdd = self.pvt.voltage.to_f64().unwrap();
        if abs_diff_eq!(von, 0.0, epsilon = 1e-4) && abs_diff_eq!(vop, vdd, epsilon = 1e-4) {
            Some(ComparatorDecision::Pos)
        } else if abs_diff_eq!(von, vdd, epsilon = 1e-4) && abs_diff_eq!(vop, 0.0, epsilon = 1e-4) {
            Some(ComparatorDecision::Neg)
        } else {
            None
        }
    }
}

/// Parameters for [`StrongArmHighSpeedTb`].
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct StrongArmHighSpeedTbParams<T, C> {
    /// The device-under-test.
    pub dut: T,

    /// The p and n voltages giving a 0.
    pub v0: (Decimal, Decimal),

    /// The p and n voltages giving a 1.
    pub v1: (Decimal, Decimal),

    /// The clock period.
    pub period: Decimal,

    /// The number of cycles to test.
    pub cycles: usize,

    /// Threshold for valid voltage levels at comparator outputs, as a percent of VDD.
    ///
    /// For example, a threshold of 0.8 indicates that the comparator must output
    /// vop >= 0.8*VDD and von <= 0.2*VDD for the output to be considered a 1.
    pub thresh: Decimal,

    /// Rise time.
    pub tr: Decimal,

    /// Fall time.
    pub tf: Decimal,

    /// Whether to pass an inverted clock to the DUT.
    ///
    /// If set to true, the clock will be held high when idle.
    /// The DUT should perform a comparison in response to a falling clock edge,
    /// rather than a rising clock edge.
    pub inverted_clk: bool,

    /// The PVT corner.
    pub pvt: Pvt<C>,
}

/// A high speed StrongARM testbench.
///
/// Applies an alternating sequence of 0s and 1s,
/// and checks that the output rails correctly.
#[derive_where::derive_where(Copy, Clone, Debug, Hash, PartialEq, Eq; T, C)]
#[derive(Serialize, Deserialize)]
pub struct StrongArmHighSpeedTb<T, PDK, C> {
    params: StrongArmHighSpeedTbParams<T, C>,

    #[serde(bound(deserialize = ""))]
    phantom: PhantomData<fn() -> PDK>,
}

impl<T, PDK, C> StrongArmHighSpeedTb<T, PDK, C> {
    /// Creates a new [`StrongArmHighSpeedTb`].
    pub fn new(params: StrongArmHighSpeedTbParams<T, C>) -> Self {
        Self {
            params,
            phantom: PhantomData,
        }
    }
}

impl<
        T: Block,
        PDK: Any,
        C: Serialize
            + DeserializeOwned
            + Copy
            + Clone
            + Debug
            + Hash
            + PartialEq
            + Eq
            + Send
            + Sync
            + Any,
    > Block for StrongArmHighSpeedTb<T, PDK, C>
{
    type Io = TestbenchIo;

    fn id() -> ArcStr {
        arcstr::literal!("strong_arm_high_speed_tb")
    }

    fn name(&self) -> ArcStr {
        arcstr::literal!("strong_arm_high_speed_tb")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl<T, PDK, C> ExportsNestedData for StrongArmHighSpeedTb<T, PDK, C>
where
    StrongArmHighSpeedTb<T, PDK, C>: Block,
{
    type NestedData = StrongArmTranTbNodes;
}

impl<T: Block<Io = ClockedDiffComparatorIo> + Schematic<PDK> + Clone, PDK: Schema, C>
    Schematic<Spectre> for StrongArmHighSpeedTb<T, PDK, C>
where
    StrongArmHighSpeedTb<T, PDK, C>: Block<Io = TestbenchIo>,
    Spectre: FromSchema<PDK>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let dut = cell
            .sub_builder::<PDK>()
            .instantiate(self.params.dut.clone());

        let vinp = cell.signal("vinp", Signal);
        let vinn = cell.signal("vinn", Signal);
        let vdd = cell.signal("vdd", Signal);
        let clk = cell.signal("clk", Signal);

        let vvinp = cell.instantiate(Vsource::pulse(Pulse {
            val0: self.params.v0.0,
            val1: self.params.v1.0,
            period: Some(self.params.period * dec!(2)),
            rise: Some(self.params.tr),
            fall: Some(self.params.tf),
            width: None,
            delay: None,
        }));
        let vvinn = cell.instantiate(Vsource::pulse(Pulse {
            val0: self.params.v0.1,
            val1: self.params.v1.1,
            period: Some(self.params.period * dec!(2)),
            rise: Some(self.params.tr),
            fall: Some(self.params.tf),
            width: None,
            delay: None,
        }));

        let vvdd = cell.instantiate(Vsource::dc(self.params.pvt.voltage));
        let (val0, val1) = if self.params.inverted_clk {
            (self.params.pvt.voltage, dec!(0))
        } else {
            (dec!(0), self.params.pvt.voltage)
        };
        let vclk = cell.instantiate(Vsource::pulse(Pulse {
            val0,
            val1,
            period: Some(self.params.period),
            width: None,
            delay: Some(self.params.period / dec!(2)),
            rise: Some(self.params.tr),
            fall: Some(self.params.tf),
        }));

        cell.connect(io.vss, vvinp.io().n);
        cell.connect(io.vss, vvinn.io().n);
        cell.connect(io.vss, vvdd.io().n);
        cell.connect(io.vss, vclk.io().n);
        cell.connect(vinp, vvinp.io().p);
        cell.connect(vinn, vvinn.io().p);
        cell.connect(vdd, vvdd.io().p);
        cell.connect(clk, vclk.io().p);

        let output = cell.signal("output", DiffPair::default());

        cell.connect(
            Bundle::<ClockedDiffComparatorIo> {
                input: Bundle::<DiffPair> { p: vinp, n: vinn },
                output: output.clone(),
                clock: clk,
                vdd,
                vss: io.vss,
            },
            dut.io(),
        );

        Ok(StrongArmTranTbNodes {
            vop: output.p,
            von: output.n,
            vinn,
            vinp,
            clk,
        })
    }
}

impl<T, PDK, C> SaveTb<Spectre, Tran, ComparatorSim> for StrongArmHighSpeedTb<T, PDK, C>
where
    StrongArmHighSpeedTb<T, PDK, C>: Block<Io = TestbenchIo>,
{
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <ComparatorSim as FromSaved<Spectre, Tran>>::SavedKey {
        ComparatorSimSavedKey {
            t: tran::Time::save(ctx, (), opts),
            vop: tran::Voltage::save(ctx, cell.data().vop, opts),
            von: tran::Voltage::save(ctx, cell.data().von, opts),
            vinn: tran::Voltage::save(ctx, cell.data().vinn, opts),
            vinp: tran::Voltage::save(ctx, cell.data().vinp, opts),
            clk: tran::Voltage::save(ctx, cell.data().clk, opts),
        }
    }
}

/// The output of the [`StrongArmHighSpeedTb`].
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StrongArmHighSpeedTbOutput {
    /// Whether the testbench used an inverted clock.
    inverted_clk: bool,
    /// The sequence of decisions made by the comparator.
    pub decisions: Vec<Option<ComparatorDecision>>,
}

impl<T, PDK, C: SimOption<Spectre> + Copy> Testbench<Spectre> for StrongArmHighSpeedTb<T, PDK, C>
where
    StrongArmHighSpeedTb<T, PDK, C>: Block<Io = TestbenchIo> + Schematic<Spectre>,
{
    type Output = StrongArmHighSpeedTbOutput;

    fn run(&self, sim: SimController<Spectre, Self>) -> Self::Output {
        let mut opts = spectre::Options::default();
        sim.set_option(self.params.pvt.corner, &mut opts);
        let wav: ComparatorSim = sim
            .simulate(
                opts,
                Tran {
                    stop: self.params.period * Decimal::from(self.params.cycles + 2),
                    start: None,
                    errpreset: Some(ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        let von = WaveformRef::new(&wav.t, &wav.von);
        let vop = WaveformRef::new(&wav.t, &wav.vop);
        let clk = WaveformRef::new(&wav.t, &wav.clk);
        let vdd = self.params.pvt.voltage.to_f64().unwrap();
        let (clk_thresh, edge_dir) = if self.params.inverted_clk {
            (0.2, EdgeDir::Rising)
        } else {
            (0.8, EdgeDir::Falling)
        };
        let decisions = clk
            .edges(clk_thresh * vdd)
            .filter(|e| e.dir() == edge_dir)
            .map(|edge| {
                let t = edge.t();
                let von = von.sample_at(t);
                let vop = vop.sample_at(t);
                let thresh = self.params.thresh.to_f64().unwrap();
                if von >= thresh * vdd && vop <= (1. - thresh) * vdd {
                    Some(ComparatorDecision::Neg)
                } else if von <= (1. - thresh) * vdd && vop >= thresh * vdd {
                    Some(ComparatorDecision::Pos)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        StrongArmHighSpeedTbOutput {
            inverted_clk: self.params.inverted_clk,
            decisions,
        }
    }
}

impl StrongArmHighSpeedTbOutput {
    /// Returns true if the testbench output was correct.
    pub fn is_correct(&self) -> bool {
        for (i, item) in self.decisions.iter().enumerate() {
            if let Some(item) = *item {
                if (i % 2 == 0 && item != ComparatorDecision::Pos)
                    || (i % 2 != 0 && item != ComparatorDecision::Neg)
                {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl Display for StrongArmHighSpeedTbOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for decision in self.decisions.iter() {
            let s = match *decision {
                None => "X",
                Some(ComparatorDecision::Neg) => "0",
                Some(ComparatorDecision::Pos) => "1",
            };
            write!(f, "{}", s)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
