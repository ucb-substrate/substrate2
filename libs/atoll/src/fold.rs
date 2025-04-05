//! ATOLL Segment Folding.

use std::collections::HashSet;

use crate::{
    get_abstract,
    grid::{LayerStack, PdkLayer},
    PointState,
};
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::{
    block::Block,
    context::Context,
    geometry::{dir::Dir, side::Side},
    layout::Layout,
};

use crate::{AtollContext, Tile};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FoldedArray<T> {
    pub tile: T,
    pub rows: usize,
    pub cols: usize,
    pub pins: Vec<PinConfig>,
}

/// Segment folding pin configuration.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PinConfig {
    /// Series connection.
    ///
    /// The index specifies the index of the other pin.
    Series { partner: usize, dir: Dir },
    /// Parallel connection.
    Parallel { dir: Dir, layer: usize },
    /// Escape to a boundary.
    Escape { side: Side, layer: usize },
    /// Ignore the pin.
    Ignore,
}

impl<T: Block> Block for FoldedArray<T> {
    type Io = ();

    fn name(&self) -> ArcStr {
        arcstr::format!("folded_{}_{}x{}", self.tile.name(), self.rows, self.cols)
    }

    fn io(&self) -> Self::Io {
        ()
    }
}

impl<T: Tile + Clone> FoldedArray<T> {
    fn analyze(&self, ctx: Context) -> substrate::error::Result<()> {
        let stack =
            ctx.get_installation::<LayerStack<
                PdkLayer<<<T as Tile>::Schema as substrate::layout::schema::Schema>::Layer>,
            >>()
            .expect("must install ATOLL layer stack");
        let (abs, paths) = get_abstract(self.tile.clone(), ctx)?;
        // identify layers to analyze: parallel pins + 1, escape pins + 0/1/2
        let mut chk_layers = HashSet::new();
        for cfg in self.pins.iter() {
            match cfg {
                PinConfig::Parallel { layer, .. } => {
                    assert!(layer + 1 < stack.len());
                    chk_layers.insert(layer + 1);
                }
                _ => unimplemented!(),
            }
        }
        // analyze layers for passthrough tracks
        let state = abs.routing_state();
        for layer in chk_layers {
            let dir = abs.grid.stack.layer(layer).dir.track_dir();
            let grid = state.layer(layer);
            let free_tracks: Vec<_> = match dir {
                Dir::Vert => grid
                    .iter_cols()
                    .enumerate()
                    .filter(|(i, col)| col.all(|elt| *elt == PointState::Available))
                    .map(|x| x.0)
                    .collect(),
                Dir::Horiz => grid
                    .iter_rows()
                    .enumerate()
                    .filter(|(i, row)| row.all(|elt| *elt == PointState::Available))
                    .map(|x| x.0)
                    .collect(),
            };
        }
        // create pin matching problem instance
        // match pins to tracks
        // strap parallel pins on matched track
        // route escape pins on pin, pin+1 OR pin+1, pin+0/2
        for (net, cfg) in abs.ports.iter().zip(self.pins.iter()) {
            match cfg {
                PinConfig::Parallel { .. } => {}
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}
