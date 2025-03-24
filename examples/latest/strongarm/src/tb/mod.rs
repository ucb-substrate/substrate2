//! StrongARM testbenches.

use approx::abs_diff_eq;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
use substrate::schematic::schema::{FromSchema, Schema};
use substrate::schematic::{Cell, CellBuilder, NestedData, NestedView, Schematic};
use substrate::simulation::data::Save;
use substrate::simulation::options::{SimOption, Temperature};
use substrate::simulation::waveform::{EdgeDir, TimeWaveform, WaveformRef};
use substrate::simulation::{Pvt, SimController, SimulationContext, Simulator, Testbench};
use substrate::types::schematic::{Node, NodeBundle};
use substrate::types::{DiffPair, Signal, TestbenchIo};

use crate::ClockedDiffComparatorIo;

/// A transient testbench that provides a differential input voltage and
/// measures the output waveform.
#[derive_where::derive_where(Copy, Clone, Debug, Hash, PartialEq, Eq; T, C)]
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

impl<T: Block, PDK: Any, C: Clone + Debug + Hash + PartialEq + Eq + Send + Sync + Any> Block
    for StrongArmTranTb<T, PDK, C>
{
    type Io = TestbenchIo;

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

impl<T, S, C> Schematic for StrongArmTranTb<T, S, C>
where
    Spectre: scir::schema::FromSchema<S>,
    S: Schema,
    StrongArmTranTb<T, S, C>: Block<Io = TestbenchIo>,
    T: Block<Io = ClockedDiffComparatorIo> + Schematic<Schema = S> + Clone,
{
    type Schema = Spectre;
    type NestedData = StrongArmTranTbNodes;
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let dut = cell.sub_builder::<S>().instantiate(self.dut.clone());

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
            NodeBundle::<ClockedDiffComparatorIo> {
                input: NodeBundle::<DiffPair> { p: vinp, n: vinn },
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

/// The decision made by a comparator.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
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

impl<T, S, C: SimOption<Spectre> + Copy> StrongArmTranTb<T, S, C>
where
    StrongArmTranTb<T, S, C>:
        Block<Io = TestbenchIo> + Schematic<Schema = Spectre, NestedData = StrongArmTranTbNodes>,
{
    pub fn run(&self, sim: SimController<Spectre, Self>) -> Option<ComparatorDecision> {
        let mut opts = spectre::Options::default();
        sim.set_option(self.pvt.corner, &mut opts);
        sim.set_option(Temperature::from(self.pvt.temp), &mut opts);
        let wav = sim
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

        let von = wav.von.last().unwrap().x();
        let vop = wav.vop.last().unwrap().x();

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
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
pub struct StrongArmHighSpeedTb<T, PDK, C> {
    params: StrongArmHighSpeedTbParams<T, C>,
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

impl<T: Block, PDK: Any, C: Clone + Debug + Hash + PartialEq + Eq + Send + Sync + Any> Block
    for StrongArmHighSpeedTb<T, PDK, C>
{
    type Io = TestbenchIo;

    fn name(&self) -> ArcStr {
        arcstr::literal!("strong_arm_high_speed_tb")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl<T: Block<Io = ClockedDiffComparatorIo> + Schematic<Schema = S> + Clone, S: Schema, C> Schematic
    for StrongArmHighSpeedTb<T, S, C>
where
    Spectre: scir::schema::FromSchema<S>,
    StrongArmHighSpeedTb<T, S, C>: Block<Io = TestbenchIo>,
{
    type Schema = Spectre;
    type NestedData = StrongArmTranTbNodes;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let dut = cell.sub_builder::<S>().instantiate(self.params.dut.clone());

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
            NodeBundle::<ClockedDiffComparatorIo> {
                input: NodeBundle::<DiffPair> { p: vinp, n: vinn },
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

/// The output of the [`StrongArmHighSpeedTb`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StrongArmHighSpeedTbOutput {
    /// Whether the testbench used an inverted clock.
    inverted_clk: bool,
    /// The sequence of decisions made by the comparator.
    pub decisions: Vec<Option<ComparatorDecision>>,
}

impl<T, S, C: SimOption<Spectre> + Copy> StrongArmHighSpeedTb<T, S, C>
where
    StrongArmHighSpeedTb<T, S, C>:
        Block<Io = TestbenchIo> + Schematic<Schema = Spectre, NestedData = StrongArmTranTbNodes>,
{
    pub fn run(&self, sim: SimController<Spectre, Self>) -> StrongArmHighSpeedTbOutput {
        let mut opts = spectre::Options::default();
        sim.set_option(self.params.pvt.corner, &mut opts);
        let wav = sim
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

        let von = wav.von.as_ref();
        let vop = wav.vop.as_ref();
        let clk = wav.clk.as_ref();
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
