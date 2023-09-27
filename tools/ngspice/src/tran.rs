//! ngspice transient analysis options and data structures.

use crate::{node_voltage_path, Ngspice, NgspicePrimitive, ProbeStmt, SaveStmt};
use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::netlist::NetlistLibConversion;
use scir::SliceOnePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use substrate::io::{NodePath, TerminalPath};
use substrate::pdk::Pdk;
use substrate::schematic::conv::RawLib;
use substrate::schematic::primitives::Resistor;
use substrate::schematic::{Cell, ExportsNestedData, NestedInstance};
use substrate::simulation::data::{FromSaved, HasSimData, Save};
use substrate::simulation::{Analysis, SimulationContext, Simulator, Supports};
use substrate::type_dispatch::impl_dispatch;

/// A transient analysis.
#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tran {
    /// Suggested computing increment (sec).
    pub step: Decimal,
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
    pub(crate) lib: Arc<RawLib<Ngspice>>,
    pub(crate) conv: Arc<NetlistLibConversion>,
    /// The time points of the transient simulation.
    pub time: Arc<Vec<f64>>,
    /// A map from signal name to values.
    pub raw_values: HashMap<ArcStr, Arc<Vec<f64>>>,
    /// A map from a save ID to a raw value identifier.
    pub(crate) saved_values: HashMap<u64, ArcStr>,
}

impl FromSaved<Ngspice, Tran> for TranOutput {
    type Key = ();
    fn from_saved(output: &<Tran as Analysis>::Output, _key: Self::Key) -> Self {
        (*output).clone()
    }
}

impl<T: ExportsNestedData> Save<Ngspice, Tran, &Cell<T>> for TranOutput {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        _to_save: &Cell<T>,
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
    }
}

impl Save<Ngspice, Tran, ()> for TranOutput {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        _to_save: (),
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
    }
}

/// The time points of a transient simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranTime(pub(crate) Arc<Vec<f64>>);

impl Deref for TranTime {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromSaved<Ngspice, Tran> for TranTime {
    type Key = ();
    fn from_saved(output: &<Tran as Analysis>::Output, _key: Self::Key) -> Self {
        TranTime(output.time.clone())
    }
}

impl<T: ExportsNestedData> Save<Ngspice, Tran, &Cell<T>> for TranTime {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        _to_save: &Cell<T>,
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
    }
}

impl Save<Ngspice, Tran, ()> for TranTime {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        _to_save: (),
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
    }
}

/// An identifier for a saved transient voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranVoltageKey(pub(crate) u64);

/// A saved transient voltage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranVoltage(pub(crate) Arc<Vec<f64>>);

impl Deref for TranVoltage {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromSaved<Ngspice, Tran> for TranVoltage {
    type Key = TranVoltageKey;
    fn from_saved(output: &<Tran as Analysis>::Output, key: Self::Key) -> Self {
        TranVoltage(
            output
                .raw_values
                .get(output.saved_values.get(&key.0).unwrap())
                .unwrap()
                .clone(),
        )
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; SaveStmt})]
impl<T> Save<Ngspice, Tran, T> for TranVoltage {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        to_save: T,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        opts.save_tran_voltage(to_save)
    }
}

impl Save<Ngspice, Tran, &SliceOnePath> for TranVoltage {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        to_save: &SliceOnePath,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        opts.save_tran_voltage(SaveStmt::ScirVoltage(to_save.clone()))
    }
}

impl Save<Ngspice, Tran, &NodePath> for TranVoltage {
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: &NodePath,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        Self::save(ctx, ctx.lib.convert_node_path(to_save).unwrap(), opts)
    }
}

#[impl_dispatch({SliceOnePath; NodePath})]
impl<T> Save<Ngspice, Tran, T> for TranVoltage {
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: T,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        Self::save(ctx, &to_save, opts)
    }
}

/// An identifier for a saved transient current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranCurrentKey(pub(crate) Vec<u64>);

/// A saved transient current.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranCurrent(pub(crate) Arc<Vec<f64>>);

impl Deref for TranCurrent {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromSaved<Ngspice, Tran> for TranCurrent {
    type Key = TranCurrentKey;
    fn from_saved(output: &<Tran as Analysis>::Output, key: Self::Key) -> Self {
        println!("{:?}", output.raw_values);
        let currents: Vec<Arc<Vec<f64>>> = key
            .0
            .iter()
            .map(|key| {
                println!("{:?}", output.saved_values.get(key).unwrap());
                output
                    .raw_values
                    .get(output.saved_values.get(key).unwrap())
                    .unwrap()
                    .clone()
            })
            .collect();

        let mut total_current = vec![0.; output.time.len()];
        for tran_current in currents {
            for (i, current) in tran_current.iter().enumerate() {
                total_current[i] += *current;
            }
        }
        TranCurrent(Arc::new(total_current))
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; SaveStmt})]
impl<T> Save<Ngspice, Tran, T> for TranCurrent {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        to_save: T,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        opts.save_tran_current(to_save)
    }
}

#[impl_dispatch({
    &NestedInstance<Resistor>;
    NestedInstance<Resistor>
})]
impl<T> Save<Ngspice, Tran, T> for TranCurrent {
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: T,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        todo!()
        // opts.save_tran_current(SaveStmt::ResistorCurrent(
        //     ctx.lib.convert_instance_path(to_save.path()).unwrap(),
        // ))
    }
}

impl Save<Ngspice, Tran, &SliceOnePath> for TranCurrent {
    fn save(
        _ctx: &SimulationContext<Ngspice>,
        to_save: &SliceOnePath,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        opts.probe_tran_current(ProbeStmt::ScirCurrent(to_save.clone()))
    }
}

impl Save<Ngspice, Tran, &TerminalPath> for TranCurrent {
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: &TerminalPath,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        TranCurrentKey(
            ctx.lib
                .convert_terminal_path(to_save)
                .unwrap()
                .into_iter()
                .flat_map(|path| Self::save(ctx, path, opts).0)
                .collect(),
        )
    }
}

#[impl_dispatch({SliceOnePath; TerminalPath})]
impl<T> Save<Ngspice, Tran, T> for TranCurrent {
    fn save(
        ctx: &SimulationContext<Ngspice>,
        to_save: T,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> Self::Key {
        Self::save(ctx, &to_save, opts)
    }
}

impl HasSimData<str, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &str) -> Option<&Vec<f64>> {
        self.raw_values.get(k).map(|x| x.as_ref())
    }
}

impl HasSimData<SliceOnePath, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &SliceOnePath) -> Option<&Vec<f64>> {
        self.get_data(&*node_voltage_path(
            &self.lib.scir,
            &self.conv,
            &self.lib.scir.simplify_path(k.clone()),
        ))
    }
}

impl HasSimData<NodePath, Vec<f64>> for TranOutput {
    fn get_data(&self, k: &NodePath) -> Option<&Vec<f64>> {
        self.get_data(&self.lib.convert_node_path(k)?)
    }
}

impl Analysis for Tran {
    type Output = TranOutput;
}

impl Supports<Tran> for Ngspice {
    fn into_input(a: Tran, inputs: &mut Vec<Self::Input>) {
        inputs.push(a.into());
    }
    fn from_output(outputs: &mut impl Iterator<Item = Self::Output>) -> <Tran as Analysis>::Output {
        let item = outputs.next().unwrap();
        item.try_into().unwrap()
    }
}
