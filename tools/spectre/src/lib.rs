//! Spectre plugin for Substrate.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::os::unix::prelude::PermissionsExt;
use std::sync::Arc;

use error::*;
use netlist::Netlister;
use psfparser::binary::ast::Trace;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::io::{NestedNode, NodePath};
use substrate::schematic::conv::RawLib;
use substrate::simulation::data::HasNodeData;
use substrate::simulation::{Analysis, SimulationContext, Simulator, Supports};
use templates::{write_run_script, RunScriptContext};

pub mod blocks;
pub mod error;
pub mod netlist;
pub(crate) mod templates;

/// A transient analysis.
#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tran {
    /// Stop time (sec).
    pub stop: Decimal,
    /// Start time (sec).
    ///
    /// Defaults to 0.
    pub start: Option<Decimal>,
}

/// The result of a transient analysis.
#[derive(Debug, Clone)]
pub struct TranOutput {
    lib: Arc<RawLib>,
    /// A map from signal name to values.
    pub values: HashMap<String, Vec<f64>>,
}

impl HasNodeData<str, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &str) -> Option<&Vec<f64>> {
        self.values.get(k)
    }
}

impl HasNodeData<scir::NodePath, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &scir::NodePath) -> Option<&Vec<f64>> {
        self.get_data(&*node_path(
            &self.lib,
            &self.lib.scir.simplify_path(k.clone()),
        ))
    }
}

impl HasNodeData<NodePath, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &NodePath) -> Option<&Vec<f64>> {
        self.get_data(&self.lib.conv.convert_path(k)?)
    }
}

impl HasNodeData<NestedNode, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &NestedNode) -> Option<&Vec<f64>> {
        self.get_data(&self.lib.conv.convert_path(&k.path())?)
    }
}

impl Analysis for Tran {
    type Output = TranOutput;
}

/// Inputs directly supported by Spectre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    /// Transient simulation input.
    Tran(Tran),
}

/// Outputs directly produced by Spectre.
#[derive(Debug, Clone)]
pub enum Output {
    /// Transient simulation output.
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
    fn simulate(
        &self,
        ctx: &SimulationContext,
        _options: Opts,
        input: Vec<Input>,
    ) -> Result<Vec<Output>> {
        std::fs::create_dir_all(&ctx.work_dir)?;
        let netlist = ctx.work_dir.join("netlist.scs");
        let f = std::fs::File::create(&netlist)?;
        let mut w = BufWriter::new(f);
        let netlister = Netlister::new(&ctx.lib.scir, &mut w);
        netlister.export()?;

        for (i, an) in input.iter().enumerate() {
            write!(w, "analysis{i} ")?;
            an.netlist(&mut w)?;
            writeln!(w)?;
        }

        w.flush()?;
        drop(w);

        let output_dir = ctx.work_dir.join("psf/");
        let log = ctx.work_dir.join("spectre.log");
        let run_script = ctx.work_dir.join("simulate.sh");
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
            .current_dir(&ctx.work_dir)
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
                    let ast = psfparser::binary::parse(&file).map_err(|e| {
                        tracing::error!("error parsing PSF file: {}", e);
                        Error::PsfParse
                    })?;
                    let mut tid_map = HashMap::new();
                    let mut values = HashMap::new();
                    for sweep in ast.sweeps.iter() {
                        tid_map.insert(sweep.id, sweep.name);
                    }
                    for trace in ast.traces.iter() {
                        match trace {
                            Trace::Group(g) => {
                                for s in g.signals.iter() {
                                    tid_map.insert(s.id, s.name);
                                }
                            }
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
                    outputs.push(
                        TranOutput {
                            lib: ctx.lib.clone(),
                            values,
                        }
                        .into(),
                    );
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
        config: &substrate::simulation::SimulationContext,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>> {
        self.simulate(config, options, input)
    }
}

pub(crate) fn node_path(lib: &RawLib, path: &scir::NodePath) -> String {
    let mut str_path = String::new();
    let scir = &lib.scir;
    let mut cell = scir.cell(path.top);
    for instance in &path.instances {
        let inst = cell.instance(*instance);
        str_path.push_str(inst.name());
        str_path.push('.');
        cell = scir.cell(inst.cell());
    }
    str_path.push_str(&cell.signal(path.signal).name);
    if let Some(index) = path.index {
        str_path.push_str(&format!("[{}]", index));
    }
    str_path
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
