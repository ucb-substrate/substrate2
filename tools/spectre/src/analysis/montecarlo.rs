//! Spectre Monte Carlo analysis options and data structures.

use crate::{Input, Spectre};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use substrate::simulation::data::{FromSaved, Save};
use substrate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};

/// Level of statistical variation to apply in a Monte Carlo analysis.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Variations {
    /// Batch-to-batch process variations.
    #[default]
    Process,
    /// Per-instance mismatch variations.
    Mismatch,
    /// Both process and mismatch variations.
    All,
}

impl Display for Variations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Process => "process",
                Self::Mismatch => "mismatch",
                Self::All => "all",
            }
        )
    }
}

/// A Monte Carlo analysis.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MonteCarlo<A> {
    /// Level of statistical variation to apply.
    pub variations: Variations,
    /// Number of Monte Carlo iterations to perform (not including nominal).
    pub numruns: usize,
    /// Starting seed for random number generator.
    pub seed: Option<u64>,
    /// Starting iteration number.
    pub firstrun: Option<usize>,
    /// The analysis to run.
    pub analysis: A,
}

/// A Monte Carlo simulation output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output<T>(pub(crate) Vec<T>);

impl<T> Deref for Output<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Output<T> {
    /// Returns the underlying vector of outputs for each
    /// iteration of the Monte Carlo simulation.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<A: SupportedBy<Spectre>> From<MonteCarlo<A>> for MonteCarlo<Vec<Input>> {
    fn from(value: MonteCarlo<A>) -> Self {
        let mut analysis = Vec::new();
        value.analysis.into_input(&mut analysis);
        MonteCarlo {
            variations: value.variations,
            numruns: value.numruns,
            seed: value.seed,
            firstrun: value.firstrun,
            analysis,
        }
    }
}

impl<A: Analysis, T: FromSaved<Spectre, A>> FromSaved<Spectre, MonteCarlo<A>> for Output<T> {
    type SavedKey = T::SavedKey;

    fn from_saved(output: &<MonteCarlo<A> as Analysis>::Output, key: &Self::SavedKey) -> Self {
        Output(
            output
                .0
                .iter()
                .map(|output| T::from_saved(output, key))
                .collect(),
        )
    }
}

impl<A: SupportedBy<Spectre>, T, S> Save<Spectre, MonteCarlo<A>, T> for Output<S>
where
    S: Save<Spectre, A, T>,
{
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as FromSaved<Spectre, MonteCarlo<A>>>::SavedKey {
        S::save(ctx, to_save, opts)
    }
}

impl<A: Analysis> Analysis for MonteCarlo<A> {
    type Output = Output<A::Output>;
}

impl<A: SupportedBy<Spectre>> SupportedBy<Spectre> for MonteCarlo<A> {
    fn into_input(self, inputs: &mut Vec<<Spectre as Simulator>::Input>) {
        inputs.push(self.into());
    }
    fn from_output(
        outputs: &mut impl Iterator<Item = <Spectre as Simulator>::Output>,
    ) -> <Self as Analysis>::Output {
        let item = outputs.next().unwrap();
        let output: Output<Vec<crate::Output>> = item.try_into().unwrap();
        Output(
            output
                .0
                .into_iter()
                .map(|out| A::from_output(&mut out.into_iter()))
                .collect(),
        )
    }
}
