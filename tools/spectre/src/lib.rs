//! Spectre plugin for Substrate.

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::os::unix::prelude::PermissionsExt;

use error::*;
use netlist::Netlister;
use psfparser::binary::ast::Trace;
use rust_decimal::Decimal;
use substrate_api::simulation::{Analysis, SimulationConfig, Simulator, Supports};
use templates::{write_run_script, RunScriptContext};

pub mod error;
pub mod netlist;
pub(crate) mod templates;

/// A transient analysis.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Tran {
    pub stop: Decimal,
    pub start: Option<Decimal>,
}

/// The result of a transient analysis.
pub struct TranOutput {
    pub values: HashMap<String, Vec<f64>>,
}

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
        _options: Opts,
        input: Vec<Input>,
    ) -> Result<Vec<Output>> {
        std::fs::create_dir_all(&config.work_dir)?;
        let netlist = config.work_dir.join("netlist.scs");
        let mut f = std::fs::File::create(&netlist)?;
        let mut w = BufWriter::new(f);
        let netlister = Netlister::new(&config.lib, &mut w);
        netlister.export()?;

        for (i, an) in input.iter().enumerate() {
            write!(w, "analysis{i} ")?;
            an.netlist(&mut w)?;
            writeln!(w)?;
        }

        w.flush()?;
        drop(w);
        f.flush()?;
        drop(f);

        // TODO run simulation and parse outputs

        let output_dir = config.work_dir.join("psf/");
        let log = config.work_dir.join("spectre.log");
        let run_script = config.work_dir.join("simulate.sh");
        write_run_script(
            RunScriptContext {
                netlist: &netlist,
                raw_output_dir: &output_dir,
                log_path: &log,
                bashrc: None,
                format: "psfbin",
                flags: "",
            },
            &run_script,
        )?;

        let mut perms = std::fs::metadata(&run_script)?.permissions();
        perms.set_mode(0o744);
        std::fs::set_permissions(&run_script, perms)?;

        let status = std::process::Command::new("/bin/bash")
            .arg(&run_script)
            .current_dir(&config.work_dir)
            .status()?;

        if !status.success() {
            return Err(Error::SpectreError);
        }

        let mut outputs = Vec::with_capacity(input.len());
        for (i, an) in input.iter().enumerate() {
            match an {
                Input::Tran(_) => {
                    let file = output_dir.join(format!("analysis{i}.tran.tran"));
                    let file = std::fs::read(file)?;
                    let ast = psfparser::binary::parse(&file).map_err(|_| Error::PsfParse)?;
                    let mut tid_map = HashMap::new();
                    let mut values = HashMap::new();
                    for trace in ast.traces.iter() {
                        match trace {
                            Trace::Group(_) => return Err(Error::PsfParse),
                            Trace::Signal(s) => {
                                tid_map.insert(s.id, s.name);
                            }
                        }
                    }
                    for (id, value) in ast.values.values.into_iter() {
                        let name = tid_map[&id].to_string();
                        let value = value.unwrap_real();
                        values.insert(name, value);
                    }
                    outputs.push(TranOutput { values }.into());
                }
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
