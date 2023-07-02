//! Spectre plugin for Substrate.

use std::io::{BufWriter, Write};
use std::path::PathBuf;

use error::*;
use netlist::Netlister;
use rust_decimal::Decimal;
use substrate_api::simulation::{Analysis, SimulationConfig, Simulator, Supports};

pub mod error;
pub mod netlist;

/// A transient analysis.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Tran {
    pub stop: Decimal,
    pub start: Option<Decimal>,
}

/// The result of a transient analysis.
pub struct TranOutput {}

impl Analysis for Tran {
    type Output = TranOutput;
}

pub enum Input {
    Tran(Tran),
}

pub enum Output {
    Tran(TranOutput),
}

impl From<Tran> for Input {
    fn from(value: Tran) -> Self {
        Self::Tran(value)
    }
}

impl From<TranOutput> for Output {
    fn from(value: TranOutput) -> Self {
        Self::Tran(value)
    }
}

impl TryFrom<Output> for TranOutput {
    type Error = Error;
    fn try_from(value: Output) -> Result<Self> {
        match value {
            Output::Tran(t) => Ok(t),
        }
    }
}

impl Supports<Tran> for Spectre {
    fn into_input(a: Tran, inputs: &mut Vec<Self::Input>) {
        inputs.push(a.into());
    }
    fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> <Tran as Analysis>::Output {
        let item = outputs.next().unwrap();
        item.try_into().unwrap()
    }
}

/// Spectre simulator global configuration.
#[derive(Debug, Clone, Default)]
pub struct Spectre {}

/// Spectre per-simulation options.
///
/// A single simulation contains zero or more analyses.
pub struct Opts {}

impl Spectre {
    pub fn simulate(
        &self,
        config: &SimulationConfig,
        options: Opts,
        input: Vec<Input>,
    ) -> Result<Vec<Output>> {
        std::fs::create_dir_all(&config.work_dir)?;
        let path = config.work_dir.join("netlist.scs");
        let f = std::fs::File::create(path)?;
        let mut w = BufWriter::new(f);
        let netlister = Netlister::new(&config.lib, &mut w);
        netlister.export()?;

        for (i, an) in input.iter().enumerate() {
            write!(w, "analysis{i} ")?;
            an.netlist(&mut w)?;
            writeln!(w)?;
        }

        // TODO run simulation and parse outputs

        let mut outputs = Vec::with_capacity(input.len());
        for (_, an) in input.iter().enumerate() {
            match an {
                Input::Tran(_) => outputs.push(TranOutput {}.into()),
            }
        }

        Ok(outputs)
    }
}

impl Simulator for Spectre {
    type Input = Input;
    type Output = Output;
    type Options = Opts;
    type Error = Error;
    fn simulate_inputs(
        &self,
        config: &substrate_api::simulation::SimulationConfig,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>> {
        self.simulate(config, options, input)
    }
}

impl Input {
    fn netlist<W: Write>(&self, out: &mut W) -> Result<()> {
        match self {
            Self::Tran(t) => t.netlist(out),
        }
    }
}

impl Tran {
    fn netlist<W: Write>(&self, out: &mut W) -> Result<()> {
        write!(out, "tran stop={}", self.stop)?;
        if let Some(ref start) = self.start {
            write!(out, "start={}", start)?;
        }
        Ok(())
    }
}
