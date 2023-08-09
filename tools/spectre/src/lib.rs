//! Spectre plugin for Substrate.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::tran::{Tran, TranCurrentKey, TranOutput, TranVoltageKey};
use arcstr::ArcStr;
use cache::error::TryInnerError;
use cache::CacheableWithState;
use error::*;
use indexmap::{IndexMap, IndexSet};
use netlist::Netlister;
use psfparser::binary::ast::Trace;
use rust_decimal::Decimal;
use scir::netlist::{Include, NetlistLibConversion};
use scir::Library;
use serde::{Deserialize, Serialize};
use substrate::execute::Executor;
use substrate::io::NodePath;
use substrate::simulation::{SetInitialCondition, SimulationContext, Simulator};
use substrate::type_dispatch::impl_dispatch;
use templates::{write_run_script, RunScriptContext};

pub mod blocks;
pub mod error;
pub mod netlist;
pub(crate) mod templates;
pub mod tran;

/// Spectre error presets.
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum ErrPreset {
    /// Liberal.
    Liberal,
    /// Moderate.
    #[default]
    Moderate,
    /// Conservative.
    Conservative,
}

impl Display for ErrPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Liberal => write!(f, "liberal"),
            Self::Moderate => write!(f, "moderate"),
            Self::Conservative => write!(f, "conservative"),
        }
    }
}

/// Spectre simulator global configuration.
#[derive(Debug, Clone, Default)]
pub struct Spectre {}

/// Spectre per-simulation options.
///
/// A single simulation contains zero or more analyses.
#[derive(Debug, Clone, Default)]
pub struct Options {
    includes: IndexSet<Include>,
    saves: IndexMap<netlist::SimSignal, u64>,
    ics: IndexMap<netlist::SimSignal, Decimal>,
    next_save_key: u64,
}

impl Options {
    /// Include the given file.
    pub fn include(&mut self, path: impl Into<PathBuf>) {
        self.includes.insert(Include::new(path));
    }
    /// Include the given section of a file.
    pub fn include_section(&mut self, path: impl Into<PathBuf>, section: impl Into<ArcStr>) {
        self.includes.insert(Include::new(path).section(section));
    }

    fn save_inner(&mut self, save: impl Into<netlist::SimSignal>) -> u64 {
        let save = save.into();

        if let Some(key) = self.saves.get(&save) {
            *key
        } else {
            let save_key = self.next_save_key;
            self.next_save_key += 1;
            self.saves.insert(save, save_key);
            save_key
        }
    }

    fn set_ic_inner(&mut self, key: impl Into<netlist::SimSignal>, value: Decimal) {
        self.ics.insert(key.into(), value);
    }

    /// Marks a transient voltage to be saved in all transient analyses.
    pub fn save_tran_voltage(&mut self, save: impl Into<netlist::SimSignal>) -> TranVoltageKey {
        TranVoltageKey(self.save_inner(save))
    }

    /// Marks a transient current to be saved in all transient analyses.
    pub fn save_tran_current(&mut self, save: impl Into<netlist::SimSignal>) -> TranCurrentKey {
        TranCurrentKey(vec![self.save_inner(save)])
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; netlist::SimSignal})]
impl<K> SetInitialCondition<K, Decimal> for Options {
    fn set_initial_condition(&mut self, key: K, value: Decimal, _ctx: &SimulationContext) {
        self.set_ic_inner(key, value);
    }
}

impl SetInitialCondition<&scir::SignalPath, Decimal> for Options {
    fn set_initial_condition(
        &mut self,
        key: &scir::SignalPath,
        value: Decimal,
        _ctx: &SimulationContext,
    ) {
        self.set_ic_inner(netlist::SimSignal::ScirVoltage(key.clone()), value);
    }
}

impl SetInitialCondition<&NodePath, Decimal> for Options {
    fn set_initial_condition(&mut self, key: &NodePath, value: Decimal, ctx: &SimulationContext) {
        self.set_initial_condition(ctx.lib.convert_node_path(key).unwrap(), value, ctx);
    }
}

#[impl_dispatch({scir::SignalPath; NodePath})]
impl<T> SetInitialCondition<T, Decimal> for Options {
    fn set_initial_condition(&mut self, key: T, value: Decimal, ctx: &SimulationContext) {
        self.set_initial_condition(&key, value, ctx);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
struct CachedSim {
    simulation_netlist: Vec<u8>,
}

struct CachedSimState {
    input: Vec<Input>,
    netlist: PathBuf,
    output_dir: PathBuf,
    log: PathBuf,
    run_script: PathBuf,
    work_dir: PathBuf,
    executor: Arc<dyn Executor>,
}

impl CacheableWithState<CachedSimState> for CachedSim {
    type Output = Vec<HashMap<String, Vec<f64>>>;
    type Error = Arc<Error>;

    fn generate_with_state(
        &self,
        state: CachedSimState,
    ) -> std::result::Result<Self::Output, Self::Error> {
        let inner = || -> Result<Self::Output> {
            let CachedSimState {
                input,
                netlist,
                output_dir,
                log,
                run_script,
                work_dir,
                executor,
            } = state;
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
            #[cfg(any(unix, target_os = "redox"))]
            perms.set_mode(0o744);
            std::fs::set_permissions(&run_script, perms)?;

            let mut command = std::process::Command::new("/bin/bash");
            command.arg(&run_script).current_dir(&work_dir);
            executor
                .execute(command, Default::default())
                .map_err(|_| Error::SpectreError)?;

            let mut raw_outputs = Vec::with_capacity(input.len());
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
                        raw_outputs.push(values);
                    }
                }
            }
            Ok(raw_outputs)
        };
        inner().map_err(Arc::new)
    }
}

impl Spectre {
    fn simulate(
        &self,
        ctx: &SimulationContext,
        options: Options,
        input: Vec<Input>,
    ) -> Result<Vec<Output>> {
        std::fs::create_dir_all(&ctx.work_dir)?;
        let netlist = ctx.work_dir.join("netlist.scs");
        let mut f = std::fs::File::create(&netlist)?;
        let mut w = Vec::new();

        let mut includes = options.includes.into_iter().collect::<Vec<_>>();
        let mut saves = options.saves.keys().cloned().collect::<Vec<_>>();
        let ics = options
            .ics
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>();
        // Sorting the include list makes repeated netlist invocations
        // produce the same output. If we were to iterate over the HashSet directly,
        // the order of includes may change even if the contents of the set did not change.
        includes.sort();
        saves.sort();

        let netlister = Netlister::new(&ctx.lib.scir, &includes, &saves, &ics, &mut w);
        let conv = netlister.export()?;

        writeln!(w)?;
        for (i, an) in input.iter().enumerate() {
            write!(w, "analysis{i} ")?;
            an.netlist(&mut w)?;
            writeln!(w)?;
        }
        f.write_all(&w)?;

        let output_dir = ctx.work_dir.join("psf/");
        let log = ctx.work_dir.join("spectre.log");
        let run_script = ctx.work_dir.join("simulate.sh");
        let work_dir = ctx.work_dir.clone();
        let executor = ctx.executor.clone();

        let raw_outputs = ctx
            .cache
            .get_with_state(
                "spectre.simulation.outputs",
                CachedSim {
                    simulation_netlist: w,
                },
                CachedSimState {
                    input,
                    netlist,
                    output_dir,
                    log,
                    run_script,
                    work_dir,
                    executor,
                },
            )
            .try_inner()
            .map_err(|e| match e {
                TryInnerError::CacheError(e) => Error::Caching(e),
                TryInnerError::GeneratorError(e) => Error::Generator(e.clone()),
            })?
            .clone();

        let conv = Arc::new(conv);
        let outputs = raw_outputs
            .into_iter()
            .map(|mut raw_values| {
                TranOutput {
                    lib: ctx.lib.clone(),
                    conv: conv.clone(),
                    time: Arc::new(raw_values.remove("time").unwrap()),
                    raw_values: raw_values
                        .into_iter()
                        .map(|(k, v)| (ArcStr::from(k), Arc::new(v)))
                        .collect(),
                    saved_values: options
                        .saves
                        .iter()
                        .map(|(k, v)| (*v, k.to_string(&ctx.lib.scir, &conv)))
                        .collect(),
                }
                .into()
            })
            .collect();

        Ok(outputs)
    }
}

impl Simulator for Spectre {
    type Input = Input;
    type Options = Options;
    type Output = Output;
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

#[allow(dead_code)]
pub(crate) fn instance_path(
    lib: &Library,
    conv: &NetlistLibConversion,
    path: &scir::InstancePath,
) -> String {
    lib.convert_instance_path(conv, path).0.join(".")
}

pub(crate) fn node_voltage_path(
    lib: &Library,
    conv: &NetlistLibConversion,
    path: &scir::SignalPath,
) -> String {
    let scir::NamedSignalPath {
        mut instances,
        signal,
        index,
    } = lib.convert_signal_path(conv, path);
    instances.push(signal);
    let mut str_path = instances.join(".");
    if let Some(index) = index {
        str_path.push_str(&format!("[{}]", index));
    }
    str_path
}

pub(crate) fn node_current_path(
    lib: &Library,
    conv: &NetlistLibConversion,
    path: &scir::SignalPath,
) -> String {
    let scir::NamedSignalPath {
        instances,
        signal,
        index,
    } = lib.convert_signal_path(conv, path);
    let mut str_path = instances.join(".");
    str_path.push(':');
    str_path.push_str(&signal);
    if let Some(index) = index {
        str_path.push_str(&format!("[{}]", index));
    }
    str_path
}

/// Inputs directly supported by Spectre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    /// Transient simulation input.
    Tran(Tran),
}

impl From<Tran> for Input {
    fn from(value: Tran) -> Self {
        Self::Tran(value)
    }
}

/// Outputs directly produced by Spectre.
#[derive(Debug, Clone)]
pub enum Output {
    /// Transient simulation output.
    Tran(TranOutput),
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
            write!(out, " start={start}")?;
        }
        if let Some(errpreset) = self.errpreset {
            write!(out, " errpreset={errpreset}")?;
        }
        Ok(())
    }
}
