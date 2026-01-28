//! Spectre DC sweeps and operating point analyses.

use crate::{InstanceTail, SimSignal, Spectre};
use arcstr::ArcStr;
use scir::{NamedSliceOne, SliceOnePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate::{
    schematic::conv::ConvertedNodePath,
    simulation::{
        Analysis, SimulationContext, Simulator, SupportedBy,
        data::{Save, SaveOutput},
    },
    types::schematic::{NestedNode, NestedTerminal, RawNestedNode},
};

/// A DC operating point analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcOp;

/// The result of a [`DcOp`] analysis.
#[derive(Debug, Clone)]
pub struct OpOutput {
    /// A map from signal name to value.
    pub raw_values: HashMap<ArcStr, f64>,
    /// A map from a save ID to a raw value identifier.
    pub(crate) saved_values: HashMap<u64, ArcStr>,
}

/// An identifier for a saved DC voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoltageSaveKey(pub(crate) u64);

/// An identifier for a saved DC current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentSaveKey(pub(crate) Vec<u64>);

impl Analysis for DcOp {
    type Output = OpOutput;
}

impl SupportedBy<Spectre> for DcOp {
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

impl Save<Spectre, DcOp> for SaveOutput {
    type SaveKey = ();
    type Saved = OpOutput;

    fn save(
        &self,
        _ctx: &SimulationContext<Spectre>,
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, DcOp>>::SaveKey {
    }

    fn from_saved(
        output: &<DcOp as Analysis>::Output,
        _key: &<Self as Save<Spectre, DcOp>>::SaveKey,
    ) -> <Self as Save<Spectre, DcOp>>::Saved {
        output.clone()
    }
}

impl Save<Spectre, DcOp> for NestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = f64;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, DcOp>>::SaveKey {
        opts.save_dc_voltage(SimSignal::ScirVoltage(
            match ctx.lib.convert_node_path(&self.path()).unwrap() {
                ConvertedNodePath::Cell(path) => path.clone(),
                ConvertedNodePath::Primitive {
                    instances, port, ..
                } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
            },
        ))
    }

    fn from_saved(
        output: &<DcOp as Analysis>::Output,
        key: &<Self as Save<Spectre, DcOp>>::SaveKey,
    ) -> <Self as Save<Spectre, DcOp>>::Saved {
        *output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
    }
}

impl Save<Spectre, DcOp> for RawNestedNode {
    type SaveKey = VoltageSaveKey;
    type Saved = f64;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, DcOp>>::SaveKey {
        let itail = InstanceTail {
            instance: ctx.lib.convert_instance_path(self.instances()).unwrap(),
            tail: self.tail().clone(),
        };
        opts.save_dc_voltage(itail)
    }

    fn from_saved(
        output: &<DcOp as Analysis>::Output,
        key: &<Self as Save<Spectre, DcOp>>::SaveKey,
    ) -> <Self as Save<Spectre, DcOp>>::Saved {
        *output
            .raw_values
            .get(output.saved_values.get(&key.0).unwrap())
            .unwrap()
    }
}

/// Data saved from a nested terminal in an [`DcOp`] simulation.
pub struct NestedTerminalOutput {
    /// The voltage at the terminal.
    pub v: f64,
    /// The current flowing through the terminal.
    pub i: f64,
}

impl Save<Spectre, DcOp> for NestedTerminal {
    type SaveKey = (VoltageSaveKey, CurrentSaveKey);
    type Saved = NestedTerminalOutput;

    fn save(
        &self,
        ctx: &SimulationContext<Spectre>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <Self as Save<Spectre, DcOp>>::SaveKey {
        (
            <NestedNode as Save<Spectre, DcOp>>::save(self, ctx, opts),
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
        output: &<DcOp as Analysis>::Output,
        key: &<Self as Save<Spectre, DcOp>>::SaveKey,
    ) -> <Self as Save<Spectre, DcOp>>::Saved {
        let v = *output
            .raw_values
            .get(output.saved_values.get(&key.0.0).unwrap())
            .unwrap();
        let i = key
            .1
            .0
            .iter()
            .map(|key| {
                output
                    .raw_values
                    .get(output.saved_values.get(key).unwrap())
                    .unwrap()
            })
            .sum();

        NestedTerminalOutput { v, i }
    }
}
