//! Spectre transient analysis options and data structures.

use crate::{ErrPreset, InstanceTail, SimSignal, Spectre};
use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::{NamedSliceOne, SliceOnePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use substrate::schematic::conv::ConvertedNodePath;
use substrate::simulation::data::{Save, SaveOutput, SaveTime};
use substrate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};
use substrate::types::schematic::{NestedNode, NestedTerminal, RawNestedNode};

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

    /// The maximum frequency for noise power spectral density.
    ///
    /// A nonzero value turns on noise sources during transient analysis.
    ///
    /// Defaults to 0.
    pub noise_fmax: Option<Decimal>,

    /// The minimum frequency for noise power spectral density.
    pub noise_fmin: Option<Decimal>,
}

/// The result of a transient analysis.
#[derive(Debug, Clone)]
pub struct Output {
    /// The time points of the transient simulation.
    pub time: Arc<Vec<f64>>,
    /// A map from signal name to values.
    pub raw_values: HashMap<ArcStr, Arc<Vec<f64>>>,
    /// A map from a save ID to a raw value identifier.
    pub(crate) saved_values: HashMap<u64, ArcStr>,
}

impl Save<Spectre, Tran> for SaveOutput {
    type SaveKey = ();
    type Saved = Output;

    fn save(
        &self,
        _ctx: &SimulationContext<Spectre>,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Tran>>::SaveKey {
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        _key: &<Self as Save<Spectre, Tran>>::SaveKey,
    ) -> <Self as Save<Spectre, Tran>>::Saved {
        output.clone()
    }
}

impl Save<Spectre, Tran> for SaveTime {
    type SaveKey = ();
    type Saved = Arc<Vec<f64>>;

    fn save(
        &self,
        _ctx: &SimulationContext<Spectre>,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Tran>>::SaveKey {
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        _key: &<Self as Save<Spectre, Tran>>::SaveKey,
    ) -> <Self as Save<Spectre, Tran>>::Saved {
        output.time.clone()
    }
}

/// An identifier for a saved transient voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoltageSaveKey(pub(crate) u64);

impl Save<Spectre, Tran> for NestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = Arc<Vec<f64>>;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Tran>>::SaveKey {
        opts.save_tran_voltage(SimSignal::ScirVoltage(
            match ctx.lib.convert_node_path(&self.path()).unwrap() {
                ConvertedNodePath::Cell(path) => path.clone(),
                ConvertedNodePath::Primitive {
                    instances, port, ..
                } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
            },
        ))
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        key: &<Self as Save<Spectre, Tran>>::SaveKey,
    ) -> <Self as Save<Spectre, Tran>>::Saved {
        output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
            .clone()
    }
}

impl Save<Spectre, Tran> for RawNestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = Arc<Vec<f64>>;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Tran>>::SaveKey {
        let itail = InstanceTail {
            instance: ctx.lib.convert_instance_path(self.instances()).unwrap(),
            tail: self.tail().clone(),
        };
        opts.save_tran_voltage(itail)
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        key: &<Self as Save<Spectre, Tran>>::SaveKey,
    ) -> <Self as Save<Spectre, Tran>>::Saved {
        output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
            .clone()
    }
}

/// An identifier for a saved transient current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentSaveKey(pub(crate) Vec<u64>);

/// Data saved from a nested terminal in a transient simulation.
pub struct NestedTerminalOutput {
    /// The voltage at the terminal.
    pub v: Arc<Vec<f64>>,
    /// The current flowing through the terminal.
    pub i: Arc<Vec<f64>>,
}

impl Save<Spectre, Tran> for NestedTerminal {
    type SaveKey = (VoltageSaveKey, CurrentSaveKey);
    type Saved = NestedTerminalOutput;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Tran>>::SaveKey {
        (
            <NestedNode as Save<Spectre, Tran>>::save(&*self, ctx, opts),
            CurrentSaveKey(
                ctx.lib
                    .convert_terminal_path(&self.path())
                    .unwrap()
                    .into_iter()
                    .flat_map(|path| {
                        opts.save_tran_current(SimSignal::ScirCurrent(match path {
                            ConvertedNodePath::Cell(path) => path.clone(),
                            ConvertedNodePath::Primitive {
                                instances, port, ..
                            } => SliceOnePath::new(
                                instances.clone(),
                                NamedSliceOne::new(port.clone()),
                            ),
                        }))
                        .0
                    })
                    .collect(),
            ),
        )
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        key: &<Self as Save<Spectre, Tran>>::SaveKey,
    ) -> <Self as Save<Spectre, Tran>>::Saved {
        let v = output
            .raw_values
            .get(output.saved_values.get(&key.0 .0).unwrap())
            .unwrap()
            .clone();
        let currents: Vec<Arc<Vec<f64>>> = key
            .1
             .0
            .iter()
            .map(|key| {
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
        NestedTerminalOutput {
            v,
            i: Arc::new(total_current),
        }
    }
}

impl Analysis for Tran {
    type Output = Output;
}

impl SupportedBy<Spectre> for Tran {
    fn into_input(self, inputs: &mut Vec<<Spectre as Simulator>::Input>) {
        inputs.push(self.into());
    }
    fn from_output(
        outputs: &mut impl Iterator<Item = <Spectre as Simulator>::Output>,
    ) -> <Self as Analysis>::Output {
        let item = outputs.next().unwrap();
        item.try_into().unwrap()
    }
}
