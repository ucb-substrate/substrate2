//! Spectre plugin for Substrate.
#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};
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
use rust_decimal::Decimal;
use scir::netlist::{Include, NetlistKind, NetlistLibConversion, NetlisterInstance, RenameGround};
use scir::schema::{FromSchema, NoSchema, NoSchemaError};
use scir::{Library, ParamValue, SliceOnePath};
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::execute::Executor;
use substrate::io::{NestedNode, NodePath, SchematicType};
use substrate::schematic::primitives::{Capacitor, RawInstance, Resistor};
use substrate::schematic::schema::Schema;
use substrate::schematic::{Primitive, PrimitiveSchematic};
use substrate::simulation::{SetInitialCondition, SimulationContext, Simulator};
use substrate::spice;
use substrate::spice::Spice;
use substrate::type_dispatch::impl_dispatch;
use templates::{write_run_script, RunScriptContext};

pub mod blocks;
pub mod error;
pub mod netlist;
pub(crate) mod templates;
pub mod tran;

/// Spectre primitives.
#[derive(Debug, Clone)]
pub enum SpectrePrimitive {
    /// A raw instance with an associated cell.
    RawInstance {
        /// The associated cell.
        cell: ArcStr,
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
        /// Parameters associated with the instance.
        params: HashMap<ArcStr, ParamValue>,
    },
    /// An external module with blackboxed contents.
    ExternalModule {
        /// The cell name.
        cell: ArcStr,
        /// The cell ports.
        ports: Vec<ArcStr>,
        /// The contents of the cell.
        contents: ArcStr,
    },
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

/// A signal referenced by a save/ic Spectre statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SimSignal {
    /// A raw string to follow "save/ic".
    Raw(ArcStr),
    /// A SCIR signal path representing a node whose voltage should be referenced.
    ScirVoltage(SliceOnePath),
    /// A SCIR signal path representing a terminal whose current should be referenced.
    ScirCurrent(SliceOnePath),
}

impl<T: Into<ArcStr>> From<T> for SimSignal {
    fn from(value: T) -> Self {
        Self::Raw(value.into())
    }
}

impl SimSignal {
    /// Creates a new [`SimSignal`].
    pub fn new(path: impl Into<ArcStr>) -> Self {
        Self::from(path)
    }

    pub(crate) fn to_string(&self, lib: &Library<Spectre>, conv: &NetlistLibConversion) -> ArcStr {
        match self {
            SimSignal::Raw(raw) => raw.clone(),
            SimSignal::ScirCurrent(scir) => ArcStr::from(node_current_path(lib, conv, scir)),
            SimSignal::ScirVoltage(scir) => ArcStr::from(node_voltage_path(lib, conv, scir)),
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
    includes: HashSet<Include>,
    saves: HashMap<SimSignal, u64>,
    ics: HashMap<SimSignal, Decimal>,
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

    fn save_inner(&mut self, save: impl Into<SimSignal>) -> u64 {
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

    fn set_ic_inner(&mut self, key: impl Into<SimSignal>, value: Decimal) {
        self.ics.insert(key.into(), value);
    }

    /// Marks a transient voltage to be saved in all transient analyses.
    pub fn save_tran_voltage(&mut self, save: impl Into<SimSignal>) -> TranVoltageKey {
        TranVoltageKey(self.save_inner(save))
    }

    /// Marks a transient current to be saved in all transient analyses.
    pub fn save_tran_current(&mut self, save: impl Into<SimSignal>) -> TranCurrentKey {
        TranCurrentKey(vec![self.save_inner(save)])
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; SimSignal})]
impl<K> SetInitialCondition<K, Decimal, Spectre> for Options {
    fn set_initial_condition(&mut self, key: K, value: Decimal, _ctx: &SimulationContext<Spectre>) {
        self.set_ic_inner(key, value);
    }
}

impl SetInitialCondition<&SliceOnePath, Decimal, Spectre> for Options {
    fn set_initial_condition(
        &mut self,
        key: &SliceOnePath,
        value: Decimal,
        _ctx: &SimulationContext<Spectre>,
    ) {
        self.set_ic_inner(SimSignal::ScirVoltage(key.clone()), value);
    }
}

impl SetInitialCondition<&NodePath, Decimal, Spectre> for Options {
    fn set_initial_condition(
        &mut self,
        key: &NodePath,
        value: Decimal,
        ctx: &SimulationContext<Spectre>,
    ) {
        self.set_initial_condition(ctx.lib.convert_node_path(key).unwrap(), value, ctx);
    }
}

impl SetInitialCondition<&NestedNode, Decimal, Spectre> for Options {
    fn set_initial_condition(
        &mut self,
        key: &NestedNode,
        value: Decimal,
        ctx: &SimulationContext<Spectre>,
    ) {
        self.set_initial_condition(key.path(), value, ctx);
    }
}

impl SetInitialCondition<NestedNode, Decimal, Spectre> for Options {
    fn set_initial_condition(
        &mut self,
        key: NestedNode,
        value: Decimal,
        ctx: &SimulationContext<Spectre>,
    ) {
        self.set_initial_condition(key.path(), value, ctx);
    }
}

#[impl_dispatch({SliceOnePath; NodePath})]
impl<T> SetInitialCondition<T, Decimal, Spectre> for Options {
    fn set_initial_condition(&mut self, key: T, value: Decimal, ctx: &SimulationContext<Spectre>) {
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
    output_path: PathBuf,
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
                output_path,
                log,
                run_script,
                work_dir,
                executor,
            } = state;
            write_run_script(
                RunScriptContext {
                    netlist: &netlist,
                    raw_output_path: &output_path,
                    log_path: &log,
                    bashrc: None,
                    format: "nutbin",
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

            let file = std::fs::read(&output_path)?;
            let ast = nutlex::parse(
                &file,
                nutlex::Options {
                    endianness: nutlex::ByteOrder::BigEndian,
                },
            )
            .map_err(|e| {
                tracing::error!("error parsing raw output file: {}", e);
                Error::Parse
            })?;
            assert_eq!(
                ast.analyses.len(),
                input.len(),
                "the output file has more analyses than the input"
            );

            for (input, output) in input.iter().zip(ast.analyses) {
                match input {
                    Input::Tran(_) => {
                        let mut values = HashMap::new();

                        let data = output.data.unwrap_real();
                        for (idx, vec) in data.into_iter().enumerate() {
                            let var_name = output
                                .variables
                                .get(idx)
                                .ok_or(Error::Parse)?
                                .name
                                .to_string();
                            values.insert(var_name, vec);
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
        ctx: &SimulationContext<Self>,
        options: Options,
        input: Vec<Input>,
    ) -> Result<Vec<Output>> {
        std::fs::create_dir_all(&ctx.work_dir)?;
        let netlist = ctx.work_dir.join("netlist.scs");
        let mut f = std::fs::File::create(&netlist)?;
        let mut w = Vec::new();

        let mut includes = options.includes.into_iter().collect::<Vec<_>>();
        let mut saves = options.saves.keys().cloned().collect::<Vec<_>>();
        let mut ics = options
            .ics
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>();
        // Sorting the include list makes repeated netlist invocations
        // produce the same output. If we were to iterate over the HashSet directly,
        // the order of includes may change even if the contents of the set did not change.
        includes.sort();
        saves.sort();
        ics.sort();

        let netlister = NetlisterInstance::new(
            NetlistKind::Testbench(RenameGround::Yes("0".into())),
            self,
            &ctx.lib.scir,
            &includes,
            &mut w,
        );
        let conv = netlister.export()?;

        writeln!(w)?;
        for save in saves {
            writeln!(w, "save {}", save.to_string(&ctx.lib.scir, &conv))?;
        }
        for (k, v) in ics {
            writeln!(w, "ic {}={}", k.to_string(&ctx.lib.scir, &conv), v)?;
        }

        writeln!(w)?;
        for (i, an) in input.iter().enumerate() {
            write!(w, "analysis{i} ")?;
            an.netlist(&mut w)?;
            writeln!(w)?;
        }
        f.write_all(&w)?;

        let output_path = ctx.work_dir.join("netlist.raw");
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
                    output_path,
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

impl scir::schema::Schema for Spectre {
    type Primitive = SpectrePrimitive;
}

impl FromSchema<NoSchema> for Spectre {
    type Error = NoSchemaError;

    fn convert_primitive(
        _primitive: <NoSchema as Schema>::Primitive,
    ) -> std::result::Result<<Self as Schema>::Primitive, Self::Error> {
        Err(NoSchemaError)
    }

    fn convert_instance(
        _instance: &mut scir::Instance,
        _primitive: &<NoSchema as Schema>::Primitive,
    ) -> std::result::Result<(), Self::Error> {
        Err(NoSchemaError)
    }
}

/// An error converting to/from the [`Spectre`] schema.
#[derive(Debug, Clone, Copy)]
pub enum SpectreConvError {
    /// A primitive that is not supported by the target schema was encountered.
    UnsupportedPrimitive,
    /// A primitive is missing a required parameter.
    MissingParameter,
    /// A primitive has an extra parameter.
    ExtraParameter,
    /// A primitive has an invalid value for a certain parameter.
    InvalidParameter,
    /// A primitive has an invalid port.
    InvalidPort,
    /// A blackbox that cannot be converted was encountered.
    Blackbox,
}

impl FromSchema<Spice> for Spectre {
    type Error = SpectreConvError;

    fn convert_primitive(
        primitive: <Spice as Schema>::Primitive,
    ) -> std::result::Result<<Self as Schema>::Primitive, Self::Error> {
        let spice::Primitive { kind, params } = primitive;
        Ok(match kind {
            spice::PrimitiveKind::ExternalModule { .. } => {
                // TODO: Add SPICE external module to Spectre, netlist
                // with simulator_lang = spice.
                return Err(SpectreConvError::Blackbox);
            }
            spice::PrimitiveKind::RawInstance { cell, ports } => SpectrePrimitive::RawInstance {
                cell,
                ports,
                params,
            },
            spice::PrimitiveKind::Mos { mname } => {
                todo!()
            }
            spice::PrimitiveKind::Res2 { value } => SpectrePrimitive::RawInstance {
                cell: "resistor".into(),
                ports: vec!["pos".into(), "neg".into()],
                params: HashMap::from_iter([("r".into(), value.into())]),
            },
        })
    }

    fn convert_instance(
        instance: &mut scir::Instance,
        primitive: &<Spice as Schema>::Primitive,
    ) -> std::result::Result<(), Self::Error> {
        match &primitive.kind {
            spice::PrimitiveKind::ExternalModule { .. } => {
                // TODO: Add SPICE external module to Spectre, netlist
                // with simulator_lang = spice.
                return Err(SpectreConvError::Blackbox);
            }
            spice::PrimitiveKind::RawInstance { cell, ports } => {}
            spice::PrimitiveKind::Mos { mname } => {
                todo!()
            }
            spice::PrimitiveKind::Res2 { value } => {
                instance.map_connections(|port| match port.as_ref() {
                    "1" => "pos".into(),
                    "2" => "neg".into(),
                    _ => port,
                });
            }
        }
        Ok(())
    }
}

impl PrimitiveSchematic<Spectre> for Resistor {
    fn schematic(&self, io: &<<Self as Block>::Io as SchematicType>::Bundle) -> Primitive<Spectre> {
        let mut prim = Primitive::new(SpectrePrimitive::RawInstance {
            cell: arcstr::literal!("resistor"),
            ports: vec![arcstr::literal!("pos"), arcstr::literal!("neg")],
            params: HashMap::from_iter([(
                arcstr::literal!("r"),
                ParamValue::Numeric(self.value()),
            )]),
        });
        prim.connect("pos", io.p);
        prim.connect("neg", io.n);
        prim
    }
}

impl PrimitiveSchematic<Spectre> for Capacitor {
    fn schematic(&self, io: &<<Self as Block>::Io as SchematicType>::Bundle) -> Primitive<Spectre> {
        let mut prim = Primitive::new(SpectrePrimitive::RawInstance {
            cell: arcstr::literal!("capacitor"),
            ports: vec![arcstr::literal!("pos"), arcstr::literal!("neg")],
            params: HashMap::from_iter([(
                arcstr::literal!("c"),
                ParamValue::Numeric(self.value()),
            )]),
        });
        prim.connect("pos", io.p);
        prim.connect("neg", io.n);
        prim
    }
}

impl PrimitiveSchematic<Spectre> for RawInstance {
    fn schematic(&self, io: &<<Self as Block>::Io as SchematicType>::Bundle) -> Primitive<Spectre> {
        let mut prim = Primitive::new(SpectrePrimitive::RawInstance {
            cell: self.cell.clone(),
            ports: self.ports.clone(),
            params: self.params.clone(),
        });
        for (i, port) in self.ports.iter().enumerate() {
            prim.connect(port, io[i]);
        }
        prim
    }
}

impl Simulator for Spectre {
    type Schema = Spectre;
    type Input = Input;
    type Options = Options;
    type Output = Output;
    type Error = Error;

    fn simulate_inputs(
        &self,
        config: &substrate::simulation::SimulationContext<Self>,
        options: Self::Options,
        input: Vec<Self::Input>,
    ) -> Result<Vec<Self::Output>> {
        self.simulate(config, options, input)
    }
}

#[allow(dead_code)]
pub(crate) fn instance_path(
    lib: &Library<Spectre>,
    conv: &NetlistLibConversion,
    path: &scir::InstancePath,
) -> String {
    lib.convert_instance_path_with_conv(conv, path.clone())
        .join(".")
}

pub(crate) fn node_voltage_path(
    lib: &Library<Spectre>,
    conv: &NetlistLibConversion,
    path: &SliceOnePath,
) -> String {
    lib.convert_slice_one_path_with_conv(conv, path.clone(), |name, index| {
        if let Some(index) = index {
            arcstr::format!("{}\\[{}\\]", name, index)
        } else {
            name.clone()
        }
    })
    .join(".")
}

pub(crate) fn node_current_path(
    lib: &Library<Spectre>,
    conv: &NetlistLibConversion,
    path: &SliceOnePath,
) -> String {
    let mut named_path = lib.convert_slice_one_path_with_conv(conv, path.clone(), |name, index| {
        if let Some(index) = index {
            arcstr::format!("{}\\[{}\\]", name, index)
        } else {
            name.clone()
        }
    });
    let signal = named_path.pop().unwrap();
    let mut str_path = named_path.join(".");
    str_path.push(':');
    str_path.push_str(&signal);
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
