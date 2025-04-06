//! ATOLL Segment Folding.

use crate::abs::{GridCoord, TrackCoord};
use crate::grid::{AbstractLayer, RoutingGrid, RoutingState};
use crate::route::ViaMaker;
use crate::{
    get_abstract,
    grid::{LayerStack, PdkLayer},
    NetId, PointState, TileBuilder, TileData,
};
use crate::{AtollContext, Tile};
use arcstr::ArcStr;
use itertools::Itertools;
use layir::Shape;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use substrate::geometry::align::AlignMode;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::rect::Rect;
use substrate::layout::CellBuilder;
use substrate::types::schematic::IoNodeBundle;
use substrate::{
    block::Block,
    context::Context,
    geometry::{dir::Dir, side::Side},
    layout,
    layout::Layout,
};

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
    Parallel { layer: usize },
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

impl<T: Tile + Clone + Foldable> Tile for FoldedArray<T> {
    type Schema = T::Schema;
    type LayoutBundle = ();
    type LayoutData = ();
    type NestedData = ();

    fn tile<'a>(
        &self,
        io: &'a IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<TileData<Self>> {
        let mut prev = Rect::default();
        let zero = Rect::default();
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut inst = cell.generate(self.tile.clone());
                if col == 0 {
                    inst.align_rect_mut(zero, AlignMode::Left, 0);
                    inst.align_rect_mut(prev, AlignMode::Beneath, 0);
                } else {
                    inst.align_rect_mut(prev, AlignMode::ToTheRight, 0);
                    inst.align_rect_mut(prev, AlignMode::Bottom, 0);
                }
                prev = inst.lcm_bounds();
                cell.draw(inst)?;
            }
        }
        self.analyze(cell.ctx().clone(), cell)?;
        Ok(TileData {
            nested_data: (),
            layout_bundle: (),
            layout_data: (),
            outline: cell.layout.bbox_rect(),
        })
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct LayerTrack {
    pub layer: usize,
    pub track: usize,
}

pub trait Foldable: Tile {
    type ViaMaker: ViaMaker<<Self::Schema as layout::schema::Schema>::Layer>;

    fn via_maker() -> Self::ViaMaker;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[enumify::enumify]
enum MatchMapping {
    Parallel(usize),
    Escape(EscapeMapping),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct EscapeMapping {
    p1: usize,
    p2: Vec<usize>,
}

impl<T: Tile + Clone + Foldable> FoldedArray<T> {
    fn analyze(
        &self,
        ctx: Context,
        cell: &mut TileBuilder<'_, <Self as Tile>::Schema>,
    ) -> substrate::error::Result<()> {
        assert!(self.rows >= 1);
        assert!(self.cols >= 1);

        let stack =
            ctx.get_installation::<LayerStack<
                PdkLayer<<<T as Tile>::Schema as substrate::layout::schema::Schema>::Layer>,
            >>()
            .expect("must install ATOLL layer stack");
        let (abs, paths) = get_abstract(self.tile.clone(), ctx.clone())?;
        // identify layers to analyze: parallel pins + 1, escape pins + 0/1/2
        let mut chk_layers = HashSet::new();
        for cfg in self.pins.iter() {
            match cfg {
                PinConfig::Parallel { layer, .. } => {
                    assert!(layer + 1 < stack.len());
                    chk_layers.insert(layer + 1);
                }
                PinConfig::Ignore => (),
                _ => unimplemented!(),
            }
        }
        // analyze layers for passthrough tracks
        let state = abs.routing_state();
        let mut free_tracks = HashMap::new();
        for layer in chk_layers {
            let dir = abs.grid.stack.layer(layer).dir.track_dir();
            let grid = state.layer(layer);
            let tracks: Vec<_> = match dir {
                // might be wrong
                Dir::Vert => grid
                    .iter_rows()
                    .enumerate()
                    .skip(1)
                    .filter_map(|(i, mut col)| {
                        col.all(|elt| *elt == PointState::Available).then_some(i)
                    })
                    .collect(),
                Dir::Horiz => grid
                    .iter_cols()
                    .enumerate()
                    .skip(1)
                    .filter_map(|(i, mut row)| {
                        row.all(|elt| *elt == PointState::Available).then_some(i)
                    })
                    .collect(),
            };
            free_tracks.insert(layer, tracks);
        }

        // create pin matching problem instance
        let mut match_input = Vec::new();
        let mut match_mapping = HashMap::new();

        for (net, cfg) in abs.ports.iter().zip(self.pins.iter()) {
            match *cfg {
                PinConfig::Parallel { layer } => {
                    match_mapping.insert(net, MatchMapping::Parallel(match_input.len()));
                    let tracks = free_tracks[&(layer + 1)]
                        .iter()
                        .map(|track| LayerTrack {
                            layer: layer + 1,
                            track: *track,
                        })
                        .collect();
                    match_input.push(tracks);
                }
                PinConfig::Escape { layer, side } => {
                    let dir = abs.grid.stack.layer(layer).dir.track_dir();
                    // number of tracks needed on layer + 2
                    let num_p2 = if dir == Dir::Horiz {
                        assert!(side == Side::Left || side == Side::Right);
                        self.cols - 1
                    } else {
                        assert!(side == Side::Top || side == Side::Bot);
                        self.rows - 1
                    };

                    // get track coordinate `coordl` of max width on layer
                    // for each track intersecting `coordl` on layer+1,
                    // add it to the acceptable track list if it is free throughout the cell
                    let mut p1trks = Vec::new();
                    let coordl =
                        max_extent_track(&state, layer, dir, *net).expect("pin not present");
                    if dir == Dir::Horiz {
                        let (coord_gdl_min, coord_gdl_max) = state
                            .layer(layer)
                            .iter_col(coordl)
                            .enumerate()
                            .filter_map(|(i, elt)| elt.is_routed_for_net(*net).then_some(i))
                            .minmax()
                            .into_option()
                            .unwrap();
                        let min = abs
                            .grid_to_physical(GridCoord {
                                layer,
                                x: coord_gdl_min,
                                y: coordl,
                            })
                            .x;
                        let max = abs
                            .grid_to_physical(GridCoord {
                                layer,
                                x: coord_gdl_max,
                                y: coordl,
                            })
                            .x;
                        let layer1 = layer + 1;
                        for p1trk in 0..state.layer(layer1).rows() {
                            let trkx = abs
                                .grid_to_physical(GridCoord {
                                    layer: layer1,
                                    x: p1trk,
                                    y: coordl,
                                })
                                .x;
                            if trkx > min && trkx < max && free_tracks[&layer1].contains(&p1trk) {
                                // p1trk is a candidate track
                                p1trks.push(LayerTrack {
                                    layer: layer1,
                                    track: p1trk,
                                });
                            }
                        }
                    } else {
                        let (coord_gdl_min, coord_gdl_max) = state
                            .layer(layer)
                            .iter_row(coordl)
                            .enumerate()
                            .filter_map(|(i, elt)| elt.is_routed_for_net(*net).then_some(i))
                            .minmax()
                            .into_option()
                            .unwrap();
                        let min = abs
                            .grid_to_physical(GridCoord {
                                layer,
                                x: coordl,
                                y: coord_gdl_min,
                            })
                            .y;
                        let max = abs
                            .grid_to_physical(GridCoord {
                                layer,
                                x: coordl,
                                y: coord_gdl_max,
                            })
                            .y;
                        let layer1 = layer + 1;
                        for p1trk in 0..state.layer(layer1).rows() {
                            let trky = abs
                                .grid_to_physical(GridCoord {
                                    layer: layer1,
                                    x: coordl,
                                    y: p1trk,
                                })
                                .y;
                            if trky > min && trky < max && free_tracks[&layer1].contains(&p1trk) {
                                // p1trk is a candidate track
                                p1trks.push(LayerTrack {
                                    layer: layer1,
                                    track: p1trk,
                                });
                            }
                        }
                    }

                    let mapping = EscapeMapping {
                        p1: match_input.len(),
                        p2: (match_input.len() + 1..).take(num_p2).collect(),
                    };
                    match_input.push(p1trks);
                    for _ in 0..num_p2 {
                        let tracks = free_tracks[&(layer + 2)]
                            .iter()
                            .map(|track| LayerTrack {
                                layer: layer + 2,
                                track: *track,
                            })
                            .collect();
                        match_input.push(tracks);
                    }
                    match_mapping.insert(net, MatchMapping::Escape(mapping));
                }
                PinConfig::Ignore => (),
                _ => unimplemented!(),
            }
        }

        // match pins to tracks
        let match_output =
            create_match(MatchInput { items: match_input }).expect("failed to create matching");

        let bbox = cell.layout.bbox_rect();
        // strap parallel pins on matched track
        let grid = RoutingGrid::new((*stack).clone(), 0..(abs.top_layer + 1));
        for (net, cfg) in abs.ports.iter().zip(self.pins.iter()) {
            match cfg {
                PinConfig::Parallel { layer } => {
                    let track_idx = |dir, base, i, delta| {
                        if dir == Dir::Horiz {
                            base as i64 - ((i + 1) * delta) as i64
                        } else {
                            base as i64 + (i * delta) as i64
                        }
                    };
                    let track = match_output.pair[*match_mapping[net].as_ref().unwrap_parallel()];
                    let dir = abs.grid.stack.layer(track.layer).dir.track_dir();
                    let layer_grid = state.layer(track.layer);
                    let (counth, deltah) = if dir == Dir::Vert {
                        (self.cols, layer_grid.rows())
                    } else {
                        (self.rows, layer_grid.cols())
                    };
                    for i in 0..counth {
                        let idxh = track_idx(dir, track.track, i, deltah);
                        let span = grid.tracks(track.layer).get(idxh);
                        let rect = Rect::from_dir_spans(dir, bbox.span(dir), span);
                        let layer = stack.layer(track.layer).layer.clone();
                        cell.layout.draw(Shape::new(layer, rect))?;
                    }

                    let dir = !dir;
                    let layer = *layer;
                    let lower_track = max_extent_track(&state, layer, dir, *net)
                        .expect("pin not present on specified layer");
                    let layer_grid = state.layer(layer);
                    let (countl, deltal) = if dir == Dir::Vert {
                        (self.cols, layer_grid.rows())
                    } else {
                        (self.rows, layer_grid.cols())
                    };
                    for i in 0..countl {
                        let idxl = track_idx(dir, lower_track, i, deltal);
                        let span = grid.tracks(layer).get(idxl);
                        let rect = Rect::from_dir_spans(dir, bbox.span(dir), span);
                        let layer = stack.layer(layer).layer.clone();
                        cell.layout.draw(Shape::new(layer, rect))?;

                        let via_maker = T::via_maker();
                        for j in 0..counth {
                            let idxh = track_idx(!dir, track.track, j, deltah);
                            let coord = if dir == Dir::Horiz {
                                TrackCoord {
                                    layer: track.layer,
                                    x: idxh,
                                    y: idxl,
                                }
                            } else {
                                TrackCoord {
                                    layer: track.layer,
                                    x: idxl,
                                    y: idxh,
                                }
                            };
                            for shape in via_maker.draw_via(ctx.clone(), coord) {
                                cell.layout.draw(shape)?;
                            }
                        }
                    }
                }
                PinConfig::Ignore => (),
                _ => unimplemented!(),
            }
        }
        // route escape pins on pin, pin+1 OR pin+1, pin+0/2
        Ok(())
    }
}

struct MatchInput<T> {
    /// `items[i]` is the set of items with which `i` can be paired.
    items: Vec<Vec<T>>,
}

struct MatchOutput<T> {
    /// `pair[i]` is the item matched to `i`.
    pair: Vec<T>,
}

fn create_match<T: Hash + Eq + Clone>(input: MatchInput<T>) -> Option<MatchOutput<T>> {
    let mut assigned = HashSet::new();
    let mut pair = Vec::new();
    for item in input.items {
        let mut found = false;
        for candidate in item {
            if !assigned.contains(&candidate) {
                pair.push(candidate.clone());
                assigned.insert(candidate);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }
    Some(MatchOutput { pair })
}

fn max_extent_track(
    state: &RoutingState<AbstractLayer>,
    layer: usize,
    dir: Dir,
    net: NetId,
) -> Option<usize> {
    Some(
        match dir {
            Dir::Horiz => state
                .layer(layer)
                .iter_cols()
                .enumerate()
                .filter_map(|(i, mut row)| {
                    let sum: usize = row
                        .clone()
                        .filter_map(|elt| elt.is_routed_for_net(net).then_some(1))
                        .sum();
                    (row.all(|elt| elt.is_available_for_net(net)) && (sum > 0)).then_some((i, sum))
                })
                .max_by_key(|x| x.1),
            Dir::Vert => state
                .layer(layer)
                .iter_rows()
                .enumerate()
                .filter_map(|(i, mut col)| {
                    let sum: usize = col
                        .clone()
                        .filter_map(|elt| elt.is_routed_for_net(net).then_some(1))
                        .sum();
                    (col.all(|elt| elt.is_available_for_net(net)) && (sum > 0)).then_some((i, sum))
                })
                .max_by_key(|x| x.1),
        }?
        .0,
    )
}
