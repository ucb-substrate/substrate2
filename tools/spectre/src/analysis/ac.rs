//! Spectre AC small-signal analysis options and data structures.

use crate::{ErrPreset, InstanceTail, SimSignal, Spectre};
use arcstr::ArcStr;
use num::complex::Complex64;
use rust_decimal::Decimal;
use scir::{NamedSliceOne, SliceOnePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use substrate::{
    schematic::conv::ConvertedNodePath,
    simulation::{
        data::{Save, SaveFreq, SaveOutput},
        Analysis, SimulationContext, Simulator, SupportedBy,
    },
    types::schematic::{NestedNode, NestedTerminal, RawNestedNode},
};

/// Sweep kinds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sweep {
    /// Linear sweep with the given number of points.
    Linear(usize),
    /// Logarithmic sweep with the given number of points.
    Logarithmic(usize),
    /// Logarithmic sweep with the given number of points **per decade**.
    Decade(usize),
}

/// An AC analysis.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Ac {
    /// Start frequency (Hz).
    ///
    /// Defaults to 0.
    pub start: Decimal,
    /// Stop frequency (Hz).
    pub stop: Decimal,
    /// The sweep kind and number of points.
    pub sweep: Sweep,

    /// The error preset.
    pub errpreset: Option<ErrPreset>,
}

/// The result of an AC analysis.
#[derive(Debug, Clone)]
pub struct Output {
    /// The frequency points of the AC simulation.
    pub freq: Arc<Vec<f64>>,
    /// A map from signal name to values.
    pub raw_values: HashMap<ArcStr, Arc<Vec<Complex64>>>,
    /// A map from a save ID to a raw value identifier.
    pub(crate) saved_values: HashMap<u64, ArcStr>,
}

/// An identifier for a saved AC voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoltageSaveKey(pub(crate) u64);

/// An identifier for a saved AC current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentSaveKey(pub(crate) Vec<u64>);

impl Analysis for Ac {
    type Output = Output;
}

impl SupportedBy<Spectre> for Ac {
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

impl Save<Spectre, Ac> for SaveOutput {
    type SaveKey = ();
    type Saved = Output;

    fn save(
        &self,
        _ctx: &SimulationContext<Spectre>,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Ac>>::SaveKey {
    }

    fn from_saved(
        output: &<Ac as Analysis>::Output,
        _key: &<Self as Save<Spectre, Ac>>::SaveKey,
    ) -> <Self as Save<Spectre, Ac>>::Saved {
        output.clone()
    }
}

impl Save<Spectre, Ac> for SaveFreq {
    type SaveKey = ();
    type Saved = Arc<Vec<f64>>;

    fn save(
        &self,
        _ctx: &SimulationContext<Spectre>,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Ac>>::SaveKey {
    }

    fn from_saved(
        output: &<Ac as Analysis>::Output,
        _key: &<Self as Save<Spectre, Ac>>::SaveKey,
    ) -> <Self as Save<Spectre, Ac>>::Saved {
        output.freq.clone()
    }
}

impl Save<Spectre, Ac> for NestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = Arc<Vec<Complex64>>;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Ac>>::SaveKey {
        opts.save_ac_voltage(SimSignal::ScirVoltage(
            match ctx.lib.convert_node_path(&self.path()).unwrap() {
                ConvertedNodePath::Cell(path) => path.clone(),
                ConvertedNodePath::Primitive {
                    instances, port, ..
                } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
            },
        ))
    }

    fn from_saved(
        output: &<Ac as Analysis>::Output,
        key: &<Self as Save<Spectre, Ac>>::SaveKey,
    ) -> <Self as Save<Spectre, Ac>>::Saved {
        output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
            .clone()
    }
}

impl Save<Spectre, Ac> for RawNestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = Arc<Vec<Complex64>>;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Ac>>::SaveKey {
        let itail = InstanceTail {
            instance: ctx.lib.convert_instance_path(self.instances()).unwrap(),
            tail: self.tail().clone(),
        };
        opts.save_ac_voltage(itail)
    }

    fn from_saved(
        output: &<Ac as Analysis>::Output,
        key: &<Self as Save<Spectre, Ac>>::SaveKey,
    ) -> <Self as Save<Spectre, Ac>>::Saved {
        output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
            .clone()
    }
}

/// Data saved from a nested terminal in an AC simulation.
pub struct NestedTerminalOutput {
    /// The voltage at the terminal.
    pub v: Arc<Vec<Complex64>>,
    /// The current flowing through the terminal.
    pub i: Arc<Vec<Complex64>>,
}

impl Save<Spectre, Ac> for NestedTerminal {
    type SaveKey = (VoltageSaveKey, CurrentSaveKey);
    type Saved = NestedTerminalOutput;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, Ac>>::SaveKey {
        (
            <NestedNode as Save<Spectre, Ac>>::save(self, ctx, opts),
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
        output: &<Ac as Analysis>::Output,
        key: &<Self as Save<Spectre, Ac>>::SaveKey,
    ) -> <Self as Save<Spectre, Ac>>::Saved {
        let v = output
            .raw_values
            .get(output.saved_values.get(&key.0 .0).unwrap())
            .unwrap()
            .clone();
        let currents: Vec<Arc<Vec<Complex64>>> = key
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

        let mut total_current = vec![Complex64::ZERO; output.freq.len()];
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
