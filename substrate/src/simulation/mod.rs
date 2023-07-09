//! Substrate's simulation API.

use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use impl_trait_for_tuples::impl_for_tuples;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::context::PdkData;
use crate::execute::Executor;
use crate::io::{SchematicType, TestbenchIo};
use crate::pdk::Pdk;
use crate::schematic::conv::RawLib;
use crate::schematic::{Cell, HasSchematic, TestbenchCellBuilder};
use crate::simulator_tuples;

pub mod data;
pub mod waveform;

/// A single simulator analysis.
pub trait Analysis {
    /// The output produced by this analysis.
    type Output;
}

/// A circuit simulator.
pub trait Simulator: Any + Send + Sync {
    /// The input type this simulator accepts.
    type Input;
    /// Options shared across all analyses for a given simulator run.
    type Options;
    /// The output type produced by this simulator.
    type Output;
    /// The error type returned by the simulator.
    type Error;

    /// Simulates the given set of analyses.
    fn simulate_inputs(
        &self,
        ctx: &SimulationContext,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>, Self::Error>;

    /// Simulates the given, possibly composite, analysis.
    fn simulate<A>(
        &self,
        ctx: &SimulationContext,
        options: Self::Options,
        input: A,
    ) -> Result<A::Output, Self::Error>
    where
        A: Analysis + SupportedBy<Self>,
        Self: Sized,
    {
        let mut inputs = Vec::new();
        input.into_input(&mut inputs);
        let output = self.simulate_inputs(ctx, options, inputs)?;
        let mut output = output.into_iter();
        Ok(A::from_output(&mut output))
    }
}

/// Substrate-defined simulation context.
pub struct SimulationContext {
    /// The simulator's intended working directory.
    pub work_dir: PathBuf,
    /// The SCIR library to simulate with associated Substrate metadata.
    pub lib: Arc<RawLib>,
    /// The executor to which simulation commands should be submitted.
    pub executor: Arc<dyn Executor>,
}

/// Indicates that a simulator supports a certain analysis.
pub trait Supports<A: Analysis>: Simulator {
    /// Convert this analysis into inputs accepted by the simulator.
    fn into_input(a: A, inputs: &mut Vec<Self::Input>);
    /// Convert the simulator outputs to this analysis's output.
    fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> A::Output;
}

/// Indicates that a particular analysis is supported by a simulator.
///
/// Where possible, prefer implementing [`Supports`].
pub trait SupportedBy<S: Simulator>: Analysis {
    /// Convert the analysis into inputs accepted by this simulator.
    fn into_input(self, inputs: &mut Vec<S::Input>);
    /// Convert this simulator's outputs to the analysis's expected output.
    fn from_output(outputs: &mut impl Iterator<Item = S::Output>) -> Self::Output;
}

impl<S, A> SupportedBy<S> for A
where
    A: Analysis,
    S: Supports<A>,
{
    fn into_input(self, inputs: &mut Vec<<S as Simulator>::Input>) {
        S::into_input(self, inputs);
    }
    fn from_output(outputs: &mut impl Iterator<Item = <S as Simulator>::Output>) -> Self::Output {
        S::from_output(outputs)
    }
}

/// Controls simulation options.
pub struct SimController<PDK: Pdk, S> {
    pub(crate) simulator: Arc<S>,
    /// The current PDK.
    pub pdk: PdkData<PDK>,
    pub(crate) ctx: SimulationContext,
}

impl<PDK: Pdk, S: Simulator> SimController<PDK, S> {
    /// Run the given analysis.
    pub fn simulate<A: Analysis + SupportedBy<S>>(
        &self,
        options: S::Options,
        input: A,
    ) -> Result<A::Output, S::Error> {
        self.simulator.simulate(&self.ctx, options, input)
    }
}

/// A testbench that can be simulated.
pub trait Testbench<PDK: Pdk, S: Simulator>:
    HasTestbenchSchematicImpl<PDK, S> + Block<Io = TestbenchIo>
{
    /// The output produced by this testbench.
    type Output: Any + Serialize + Deserialize<'static>;
    /// Run the testbench using the given simulation controller.
    fn run(&self, cell: &Cell<Self>, sim: SimController<PDK, S>) -> Self::Output;
}

/// A testbench block that has a schematic compatible with the given PDK and simulator.
pub trait HasTestbenchSchematicImpl<PDK: Pdk, S: Simulator>: Block + HasSchematic {
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Data,
        cell: &mut TestbenchCellBuilder<PDK, S, Self>,
    ) -> crate::error::Result<Self::Data>;
}

#[impl_for_tuples(64)]
impl Analysis for Tuple {
    for_tuples!( type Output = ( #( Tuple::Output ),* ); );
}

simulator_tuples!(64);
