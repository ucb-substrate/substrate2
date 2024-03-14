//! Spectre AC small-signal analysis options and data structures.

use crate::{ErrPreset, SimSignal, Spectre};
use arcstr::ArcStr;
use num::complex::Complex64;
use num::Zero;
use rust_decimal::Decimal;
use scir::{NamedSliceOne, SliceOnePath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use substrate::io::schematic::{NestedNode, NestedTerminal, NodePath, TerminalPath};
use substrate::schematic::conv::ConvertedNodePath;
use substrate::simulation::data::{ac, FromSaved, Save};
use substrate::simulation::{Analysis, SimulationContext, Simulator, SupportedBy};
use substrate::type_dispatch::impl_dispatch;

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

impl FromSaved<Spectre, Ac> for Output {
    type SavedKey = ();

    fn from_saved(output: &<Ac as Analysis>::Output, _key: &Self::SavedKey) -> Self {
        (*output).clone()
    }
}

impl Save<Spectre, Ac, ()> for Output {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        _to_save: (),
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
    }
}

impl FromSaved<Spectre, Ac> for ac::Freq {
    type SavedKey = ();
    fn from_saved(output: &<Ac as Analysis>::Output, _key: &Self::SavedKey) -> Self {
        ac::Freq(output.freq.clone())
    }
}

impl Save<Spectre, Ac, ()> for ac::Freq {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        _to_save: (),
        _opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
    }
}

/// An identifier for a saved AC voltage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoltageSavedKey(pub(crate) u64);

impl FromSaved<Spectre, Ac> for ac::Voltage {
    type SavedKey = VoltageSavedKey;
    fn from_saved(output: &<Ac as Analysis>::Output, key: &Self::SavedKey) -> Self {
        ac::Voltage(
            output
                .raw_values
                .get(output.saved_values.get(&key.0).unwrap())
                .unwrap()
                .clone(),
        )
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; SimSignal})]
impl<T> Save<Spectre, Ac, T> for ac::Voltage {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        opts.save_ac_voltage(to_save)
    }
}

impl Save<Spectre, Ac, &SliceOnePath> for ac::Voltage {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        to_save: &SliceOnePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        opts.save_ac_voltage(SimSignal::ScirVoltage(to_save.clone()))
    }
}

impl Save<Spectre, Ac, &ConvertedNodePath> for ac::Voltage {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: &ConvertedNodePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(
            ctx,
            match to_save {
                ConvertedNodePath::Cell(path) => path.clone(),
                ConvertedNodePath::Primitive {
                    instances, port, ..
                } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
            },
            opts,
        )
    }
}

impl Save<Spectre, Ac, &NodePath> for ac::Voltage {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: &NodePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, ctx.lib.convert_node_path(to_save).unwrap(), opts)
    }
}

#[impl_dispatch({SliceOnePath; ConvertedNodePath; NodePath})]
impl<T> Save<Spectre, Ac, T> for ac::Voltage {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, &to_save, opts)
    }
}

#[impl_dispatch({NestedNode; &NestedNode; NestedTerminal; &NestedTerminal})]
impl<T> Save<Spectre, Ac, T> for ac::Voltage {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, to_save.path(), opts)
    }
}

#[impl_dispatch({TerminalPath; &TerminalPath})]
impl<T> Save<Spectre, Ac, T> for ac::Voltage {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, to_save.as_ref(), opts)
    }
}

/// An identifier for a saved AC current.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentSavedKey(pub(crate) Vec<u64>);

impl FromSaved<Spectre, Ac> for ac::Current {
    type SavedKey = CurrentSavedKey;
    fn from_saved(output: &<Ac as Analysis>::Output, key: &Self::SavedKey) -> Self {
        let currents: Vec<Arc<Vec<Complex64>>> = key
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

        let mut total_current = vec![Complex64::zero(); output.freq.len()];
        for ac_current in currents {
            for (i, current) in ac_current.iter().enumerate() {
                total_current[i] += *current;
            }
        }
        ac::Current(Arc::new(total_current))
    }
}

#[impl_dispatch({&str; &String; ArcStr; String; SimSignal})]
impl<T> Save<Spectre, Ac, T> for ac::Current {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        opts.save_ac_current(to_save)
    }
}

impl Save<Spectre, Ac, &SliceOnePath> for ac::Current {
    fn save(
        _ctx: &SimulationContext<Spectre>,
        to_save: &SliceOnePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        opts.save_ac_current(SimSignal::ScirCurrent(to_save.clone()))
    }
}

impl Save<Spectre, Ac, &ConvertedNodePath> for ac::Current {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: &ConvertedNodePath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(
            ctx,
            match to_save {
                ConvertedNodePath::Cell(path) => path.clone(),
                ConvertedNodePath::Primitive {
                    instances, port, ..
                } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
            },
            opts,
        )
    }
}

impl Save<Spectre, Ac, &TerminalPath> for ac::Current {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: &TerminalPath,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        CurrentSavedKey(
            ctx.lib
                .convert_terminal_path(to_save)
                .unwrap()
                .into_iter()
                .flat_map(|path| Self::save(ctx, path, opts).0)
                .collect(),
        )
    }
}

#[impl_dispatch({SliceOnePath; ConvertedNodePath; TerminalPath})]
impl<T> Save<Spectre, Ac, T> for ac::Current {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, &to_save, opts)
    }
}

#[impl_dispatch({NestedTerminal; &NestedTerminal})]
impl<T> Save<Spectre, Ac, T> for ac::Current {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: T,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::SavedKey {
        Self::save(ctx, to_save.path(), opts)
    }
}

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
