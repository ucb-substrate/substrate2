//! Substrate's simulation API.

use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use data::Save;
use impl_trait_for_tuples::impl_for_tuples;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::block::Block;
use crate::context::{Context, Installation};
use crate::schematic::conv::RawLib;
use crate::schematic::schema::Schema;
use crate::schematic::{Cell, HasNestedView, NestedView, Schematic};
use crate::types::TestbenchIo;
use codegen::simulator_tuples;

pub mod data;
pub mod options;
pub mod waveform;

/// A single simulator analysis.
pub trait Analysis {
    /// The output produced by this analysis.
    type Output;
}

/// A circuit simulator.
pub trait Simulator: Installation + Any + Send + Sync {
    /// The schema that this simulator uses.
    type Schema: Schema;
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
        ctx: &SimulationContext<Self>,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>, Self::Error>;

    /// Simulates the given, possibly composite, analysis.
    fn simulate<A>(
        &self,
        ctx: &SimulationContext<Self>,
        options: Self::Options,
        input: A,
    ) -> Result<A::Output, Self::Error>
    where
        A: SupportedBy<Self>,
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
pub struct SimulationContext<S: Simulator + ?Sized> {
    /// The simulator's intended working directory.
    pub work_dir: PathBuf,
    /// The SCIR library to simulate with associated Substrate metadata.
    pub lib: Arc<RawLib<S::Schema>>,
    /// The global context.
    pub ctx: Context,
}

/// Indicates that a particular analysis is supported by a simulator.
pub trait SupportedBy<S: Simulator>: Analysis {
    /// Convert the analysis into inputs accepted by this simulator.
    fn into_input(self, inputs: &mut Vec<<S as Simulator>::Input>);
    /// Convert this simulator's outputs to the analysis's expected output.
    fn from_output(outputs: &mut impl Iterator<Item = <S as Simulator>::Output>) -> Self::Output;
}

/// Controls simulation options.
pub struct SimController<S: Simulator, T: Schematic> {
    pub(crate) simulator: Arc<S>,
    /// The current testbench cell.
    pub tb: Arc<Cell<T>>,
    pub(crate) ctx: SimulationContext<S>,
}

impl<S: Simulator, T: Testbench<S>> SimController<S, T> {
    /// Run the given analysis, returning the default output.
    pub fn simulate_default<A: SupportedBy<S>>(
        &self,
        options: S::Options,
        input: A,
    ) -> Result<A::Output, S::Error> {
        self.simulator.simulate(&self.ctx, options, input)
    }

    /// Run the given analysis, returning the desired output type.
    pub fn simulate<A: SupportedBy<S>>(
        &self,
        mut options: S::Options,
        input: A,
    ) -> Result<<NestedView<<T as Schematic>::NestedData> as Save<S, A>>::Save, S::Error>
    where
        T: Schematic<NestedData: HasNestedView<NestedView: Save<S, A>>>,
    {
        let key = <NestedView<<T as Schematic>::NestedData> as Save<S, A>>::save(
            &self.tb.data(),
            &self.ctx,
            &mut options,
        );
        let output = self.simulate_default(options, input)?;
        Ok(
            <<<T as Schematic>::NestedData as HasNestedView>::NestedView>::from_saved(
                &output, &key,
            ),
        )
    }

    /// Set an option by mutating the given options.
    pub fn set_option<O>(&self, opt: O, options: &mut S::Options)
    where
        O: options::SimOption<S>,
    {
        opt.set_option(options, &self.ctx);
    }
}

/// A testbench that can be simulated.
pub trait Testbench<S: Simulator>: Schematic<Schema = S::Schema> + Block<Io = TestbenchIo> {}
impl<S: Simulator, T: Schematic<Schema = S::Schema> + Block<Io = TestbenchIo>> Testbench<S> for T {}

#[impl_for_tuples(64)]
impl Analysis for Tuple {
    for_tuples!( type Output = ( #( Tuple::Output ),* ); );
}

simulator_tuples!(64);
