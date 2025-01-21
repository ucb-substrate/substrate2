//! ngspice transient analysis options and data structures.

use crate::{InstanceTail, Ngspice, ProbeStmt, SaveStmt};
use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::{NamedSliceOne, SliceOnePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use substrate::schematic::conv::ConvertedNodePath;
use substrate::simulation::data::{Save, SaveOutput, SaveTime};
use substrate::simulation::waveform::{TimePoint, TimeWaveform, WaveformRef};
use substrate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};
use substrate::types::schematic::{NestedNode, NestedTerminal, RawNestedNode};

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
pub struct Output {
    /// The time points of the transient simulation.
    pub time: Arc<Vec<f64>>,
    /// A map from signal name to values.
    pub raw_values: HashMap<ArcStr, Arc<Vec<f64>>>,
    /// A map from a save ID to a raw value identifier.
    pub(crate) saved_values: HashMap<u64, ArcStr>,
}

/// An output transient waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OutputWaveform {
    /// Time samples.
    pub t: Arc<Vec<f64>>,
    /// Values corresponding to time samples in `t`.
    pub x: Arc<Vec<f64>>,
}

impl OutputWaveform {
    /// Converts an [`OutputWaveform`] to a [`WaveformRef`].
    pub fn as_ref(&self) -> WaveformRef<'_, f64> {
        WaveformRef::new(self.t.as_ref(), self.x.as_ref())
    }
}

impl TimeWaveform for OutputWaveform {
    type Data = f64;
    fn get(&self, idx: usize) -> Option<TimePoint<f64>> {
        self.as_ref().get(idx)
    }

    fn len(&self) -> usize {
        self.t.len()
    }
}

impl Save<Ngspice, Tran> for SaveOutput {
    type SaveKey = ();
    type Saved = Output;
    fn save(
        &self,
        _ctx: &SimulationContext<Ngspice>,
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Self as Save<Ngspice, Tran>>::SaveKey {
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        _key: &<Self as Save<Ngspice, Tran>>::SaveKey,
    ) -> <Self as Save<Ngspice, Tran>>::Saved {
        output.clone()
    }
}

impl Save<Ngspice, Tran> for SaveTime {
    type SaveKey = ();
    type Saved = Arc<Vec<f64>>;
    fn save(
        &self,
        _ctx: &SimulationContext<Ngspice>,
        _opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Self as Save<Ngspice, Tran>>::SaveKey {
    }
    fn from_saved(
        output: &<Tran as Analysis>::Output,
        _key: &<Self as Save<Ngspice, Tran>>::SaveKey,
    ) -> <Self as Save<Ngspice, Tran>>::Saved {
        output.time.clone()
    }
}

/// An identifier for a saved transient voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoltageSaveKey(pub(crate) u64);

impl Save<Ngspice, Tran> for NestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = OutputWaveform;
    fn save(
        &self,
        ctx: &SimulationContext<Ngspice>,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Self as Save<Ngspice, Tran>>::SaveKey {
        opts.save_tran_voltage(SaveStmt::ScirVoltage(
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
        key: &<Self as Save<Ngspice, Tran>>::SaveKey,
    ) -> <Self as Save<Ngspice, Tran>>::Saved {
        OutputWaveform {
            t: output.time.clone(),
            x: output
                .raw_values
                .get(output.saved_values.get(&key.0).unwrap())
                .unwrap()
                .clone(),
        }
    }
}

impl Save<Ngspice, Tran> for RawNestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = OutputWaveform;

    fn save(
        &self,
        ctx: &SimulationContext<Ngspice>,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Self as Save<Ngspice, Tran>>::SaveKey {
        let itail = InstanceTail {
            instance: ctx.lib.convert_instance_path(self.instances()).unwrap(),
            tail: self.tail().clone(),
        };
        opts.save_tran_voltage(SaveStmt::InstanceTail(itail))
    }

    fn from_saved(
        output: &<Tran as Analysis>::Output,
        key: &<Self as Save<Ngspice, Tran>>::SaveKey,
    ) -> <Self as Save<Ngspice, Tran>>::Saved {
        let name = output.saved_values.get(&key.0).unwrap();
        OutputWaveform {
            t: output.time.clone(),
            x: output.raw_values.get(name).unwrap().clone(),
        }
    }
}

/// The output of saving a terminal.
pub struct NestedTerminalOutput {
    /// The voltage at the terminal.
    pub v: OutputWaveform,
    /// The current flowing through the terminal.
    pub i: OutputWaveform,
}

impl Save<Ngspice, Tran> for NestedTerminal {
    type SaveKey = (VoltageSaveKey, CurrentSaveKey);
    type Saved = NestedTerminalOutput;

    fn save(
        &self,
        ctx: &SimulationContext<Ngspice>,
        opts: &mut <Ngspice as Simulator>::Options,
    ) -> <Self as Save<Ngspice, Tran>>::SaveKey {
        (
            <NestedNode as Save<Ngspice, Tran>>::save(self, ctx, opts),
            CurrentSaveKey(
                ctx.lib
                    .convert_terminal_path(&self.path())
                    .unwrap()
                    .into_iter()
                    .flat_map(|path| {
                        opts.probe_tran_current(ProbeStmt::ScirCurrent(match path {
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
        key: &<Self as Save<Ngspice, Tran>>::SaveKey,
    ) -> <Self as Save<Ngspice, Tran>>::Saved {
        let v = OutputWaveform {
            t: output.time.clone(),
            x: output
                .raw_values
                .get(output.saved_values.get(&key.0 .0).unwrap())
                .unwrap()
                .clone(),
        };
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
            i: OutputWaveform {
                t: output.time.clone(),
                x: Arc::new(total_current),
            },
        }
    }
}

/// An identifier for a saved transient current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentSaveKey(pub(crate) Vec<u64>);

impl Analysis for Tran {
    type Output = Output;
}

impl SupportedBy<Ngspice> for Tran {
    fn into_input(self, inputs: &mut Vec<<Ngspice as Simulator>::Input>) {
        inputs.push(self.into());
    }
    fn from_output(
        outputs: &mut impl Iterator<Item = <Ngspice as Simulator>::Output>,
    ) -> <Self as Analysis>::Output {
        let item = outputs.next().unwrap();
        item.try_into().unwrap()
    }
}
