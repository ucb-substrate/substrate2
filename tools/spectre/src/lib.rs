//! Spectre plugin for Substrate.
#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::Write;
use std::ops::Deref;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use arcstr::ArcStr;
use cache::error::TryInnerError;
use cache::CacheableWithState;
use error::*;
use netlist::{Include, Netlister};
use psfparser::binary::ast::Trace;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::execute::Executor;
use substrate::io::{NestedNode, NodePath};
use substrate::schematic::conv::RawLib;
use substrate::simulation::data::{FromSaved, HasNodeData, HasSaveKey, Save};
use substrate::simulation::{Analysis, SimulationContext, Simulator, Supports};
use substrate::type_dispatch::impl_dispatch;
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

    /// The error preset.
    pub errpreset: Option<ErrPreset>,
}

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

/// The result of a transient analysis.
#[derive(Debug, Clone)]
pub struct TranOutput {
    lib: Arc<RawLib>,
    /// A map from signal name to values.
    pub raw_values: HashMap<String, Vec<f64>>,
    /// A map from save key to values.
    saved_values: HashMap<SaveKey, Vec<f64>>,
}

impl HasSaveKey for TranOutput {
    type SaveKey = ();
}

impl<T> Save<Spectre, Tran, T> for TranOutput {
    fn save(
        _ctx: &SimulationContext,
        _to_save: T,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
    }
}

impl FromSaved<Spectre, Tran> for TranOutput {
    fn from_saved(output: &mut <Tran as Analysis>::Output, key: Self::SaveKey) -> Self {
        Self {
            lib: output.lib.clone(),
            raw_values: output.raw_values.drain().collect(),
            saved_values: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveKey(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranVoltage(Vec<f64>);

impl TranVoltage {
    pub fn into_inner(self) -> Vec<f64> {
        self.0
    }
}

impl HasSaveKey for TranVoltage {
    type SaveKey = SaveKey;
}

#[impl_dispatch({&str; ArcStr; String; netlist::Save})]
impl<T> Save<Spectre, Tran, T> for TranVoltage {
    fn save(
        _ctx: &SimulationContext,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        opts.save(to_save)
    }
}

impl Save<Spectre, Tran, &scir::SignalPath> for TranVoltage {
    fn save(
        ctx: &SimulationContext,
        to_save: &scir::SignalPath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, &*node_voltage_path(&ctx.lib, to_save), opts)
    }
}

impl Save<Spectre, Tran, &NodePath> for TranVoltage {
    fn save(
        ctx: &SimulationContext,
        to_save: &NodePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, ctx.lib.conv.convert_path(to_save).unwrap(), opts)
    }
}

impl Save<Spectre, Tran, &NestedNode> for TranVoltage {
    fn save(
        ctx: &SimulationContext,
        to_save: &NestedNode,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, to_save.path(), opts)
    }
}

#[impl_dispatch({scir::SignalPath; NodePath; NestedNode})]
impl<T> Save<Spectre, Tran, T> for TranVoltage {
    fn save(
        ctx: &SimulationContext,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, &to_save, opts)
    }
}

impl FromSaved<Spectre, Tran> for TranVoltage {
    fn from_saved(output: &mut <Tran as Analysis>::Output, key: Self::SaveKey) -> Self {
        TranVoltage(output.saved_values.remove(&key).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranCurrent(Vec<f64>);

impl TranCurrent {
    pub fn into_inner(self) -> Vec<f64> {
        self.0
    }
}

impl HasSaveKey for TranCurrent {
    type SaveKey = SaveKey;
}

#[impl_dispatch({&str; ArcStr; String; netlist::Save})]
impl<T> Save<Spectre, Tran, T> for TranCurrent {
    fn save(
        _ctx: &SimulationContext,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        opts.save(to_save)
    }
}

impl Save<Spectre, Tran, &scir::SignalPath> for TranCurrent {
    fn save(
        ctx: &SimulationContext,
        to_save: &scir::SignalPath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, &*node_current_path(&ctx.lib, to_save), opts)
    }
}

impl Save<Spectre, Tran, &NodePath> for TranCurrent {
    fn save(
        ctx: &SimulationContext,
        to_save: &NodePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, ctx.lib.conv.convert_path(to_save).unwrap(), opts)
    }
}

impl Save<Spectre, Tran, &NestedNode> for TranCurrent {
    fn save(
        ctx: &SimulationContext,
        to_save: &NestedNode,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, to_save.path(), opts)
    }
}

#[impl_dispatch({scir::SignalPath; NodePath; NestedNode})]
impl<T> Save<Spectre, Tran, T> for TranCurrent {
    fn save(
        ctx: &SimulationContext,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SaveKey {
        Self::save(ctx, &to_save, opts)
    }
}

impl FromSaved<Spectre, Tran> for TranCurrent {
    fn from_saved(output: &mut <Tran as Analysis>::Output, key: Self::SaveKey) -> Self {
        TranCurrent(output.saved_values.remove(&key).unwrap())
    }
}

impl HasNodeData<str, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &str) -> Option<&Vec<f64>> {
        self.raw_values.get(k)
    }
}

impl HasNodeData<scir::SignalPath, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &scir::SignalPath) -> Option<&Vec<f64>> {
        self.get_data(&*node_voltage_path(
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
#[derive(Debug, Clone, Default)]
pub struct Options {
    includes: HashSet<Include>,
    saves: HashMap<netlist::Save, SaveKey>,
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

    pub fn save(&mut self, save: impl Into<netlist::Save>) -> SaveKey {
        let save_key = SaveKey(self.next_save_key);
        self.next_save_key += 1;
        self.saves.insert(save.into(), save_key);
        save_key
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
        // Sorting the include list makes repeated netlist invocations
        // produce the same output. If we were to iterate over the HashSet directly,
        // the order of includes may change even if the contents of the set did not change.
        includes.sort();
        saves.sort();

        let netlister = Netlister::new(&ctx.lib.scir, &includes, &saves, &mut w);
        netlister.export()?;

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

        let outputs = raw_outputs
            .into_iter()
            .map(|raw_values| {
                let saved_values = raw_values
                    .iter()
                    .filter_map(|(name, voltage)| {
                        options
                            .saves
                            .get(&netlist::Save::new(name))
                            .map(|key| (*key, voltage.clone()))
                    })
                    .collect::<HashMap<_, _>>();
                TranOutput {
                    lib: ctx.lib.clone(),
                    raw_values,
                    saved_values,
                }
                .into()
            })
            .collect();

        Ok(outputs)
    }
}

impl Simulator for Spectre {
    type Input = Input;
    type Output = Output;
    type Options = Options;
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

pub(crate) fn node_voltage_path(lib: &RawLib, path: &scir::SignalPath) -> String {
    let named_path = &lib.scir.convert_path(path);
    let mut str_path = named_path.instances.join(".");
    str_path.push('.');
    str_path.push_str(&named_path.signal);
    if let Some(index) = named_path.index {
        str_path.push_str(&format!("[{}]", index));
    }
    str_path
}

pub(crate) fn node_current_path(lib: &RawLib, path: &scir::SignalPath) -> String {
    let named_path = &lib.scir.convert_path(path);
    let mut str_path = named_path.instances.join(".");
    str_path.push(':');
    str_path.push_str(&named_path.signal);
    if let Some(index) = named_path.index {
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
            write!(out, " start={start}")?;
        }
        if let Some(errpreset) = self.errpreset {
            write!(out, " errpreset={errpreset}")?;
        }
        Ok(())
    }
}
