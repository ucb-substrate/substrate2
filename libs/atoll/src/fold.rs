//! ATOLL Segment Folding.

use crate::abs::{Abstract, GridCoord, TrackCoord};
use crate::grid::{AbstractLayer, RoutingGrid, RoutingState};
use crate::route::ViaMaker;
use crate::{
    get_abstract,
    grid::{LayerStack, PdkLayer},
    NetId, PointState, TileBuilder, TileData,
};
use crate::{Orientation, Tile};
use arcstr::ArcStr;
use itertools::Itertools;
use layir::Shape;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use substrate::geometry::align::AlignMode;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::rect::Rect;
use substrate::layout::tracks::RoundingMode;
use substrate::types::schematic::{IoNodeBundle, Node};
use substrate::types::Flatten;
use substrate::{
    block::Block,
    context::Context,
    geometry::{dir::Dir, side::Side},
    layout,
};

/// Folded array of an ATOLL tile.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FoldedArray<T> {
    /// Tile being folded.
    pub tile: T,
    /// Number of rows.
    pub rows: usize,
    /// Number of cols.
    pub cols: usize,
    /// Pin configuration.
    pub pins: Vec<PinConfig>,
    /// The top layer of the array.
    pub top_layer: usize,
    /// The direction in which series/parallel pins of the tile abut.
    pub dir: Dir,
}

/// Segment folding pin configuration.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PinConfig {
    /// Series connection.
    ///
    Series {
        /// Index of the other pin.
        partner: usize,
        /// Pin layer.
        layer: usize,
    },
    /// Parallel connection.
    Parallel {
        /// Pin layer.
        layer: usize,
    },
    /// Escape to a boundary.
    Escape {
        /// Escape side.
        side: Side,
        /// Pin layer.
        layer: usize,
    },
    /// Ignore the pin.
    Ignore,
}

impl<T: Block> Block for FoldedArray<T> {
    type Io = ();

    fn name(&self) -> ArcStr {
        arcstr::format!("folded_{}_{}x{}", self.tile.name(), self.rows, self.cols)
    }

    fn io(&self) -> Self::Io {}
}

impl<T: Tile + Clone + Foldable> Tile for FoldedArray<T> {
    type Schema = T::Schema;
    type LayoutBundle = ();
    type LayoutData = ();
    type NestedData = ();

    fn tile<'a>(
        &self,
        _io: &'a IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<TileData<Self>> {
        self.analyze(cell.ctx().clone(), cell)?;
        cell.set_top_layer(self.top_layer);

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
    layer: usize,
    track: usize,
}

/// A foldable tile.
pub trait Foldable: Tile {
    /// The via maker to be used for the tile.
    type ViaMaker: ViaMaker<<Self::Schema as layout::schema::Schema>::Layer>;

    /// The via maker to be used for the tile.
    fn via_maker() -> Self::ViaMaker;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[enumify::enumify]
enum MatchMapping {
    Parallel(usize),
    Series(usize),
    Escape(EscapeMapping),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct EscapeMapping {
    p1: usize,
    p2: Vec<usize>,
}

struct EscapePinData {
    coordl: usize,
    dir: Dir,
    #[allow(dead_code)]
    coord_gdl_min: usize,
    #[allow(dead_code)]
    coord_gdl_max: usize,
}

#[derive(Debug, Clone)]
struct SeriesPinData {
    coord_output: usize,
    coord_input: usize,
    igdl_min: usize,
    igdl_max: usize,
    ogdl_min: usize,
    ogdl_max: usize,
    /// width of tile on layer
    dir_width: usize,
    /// width of tile on layer + 1
    dir_width_h: usize,
}

impl<T: Tile + Clone + Foldable> FoldedArray<T> {
    fn analyze(
        &self,
        ctx: Context,
        cell: &mut TileBuilder<'_, <Self as Tile>::Schema>,
    ) -> substrate::error::Result<()> {
        let mut prev = Rect::default();
        let zero = Rect::default();
        let mut prev_nodes = vec![];
        let mut series_partners = HashSet::new();
        let mut n_series_pairs = 0;
        // Identify series pin pairs.
        for pin in self.pins.iter() {
            if let PinConfig::Series { partner, .. } = *pin {
                series_partners.insert(partner);
                n_series_pairs += 1;
            }
        }
        // make variable immutable
        let n_series_pairs = n_series_pairs;
        // Tile the array.
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut inst = cell.generate(self.tile.clone());
                match self.dir {
                    Dir::Horiz => {
                        if row % 2 == 1 {
                            inst.orient_mut(Orientation::ReflectHoriz);
                        }
                    }
                    Dir::Vert => {
                        if col % 2 == 1 {
                            inst.orient_mut(Orientation::ReflectVert);
                        }
                    }
                }
                if col == 0 {
                    inst.align_rect_mut(zero, AlignMode::Left, 0);
                    inst.align_rect_mut(prev, AlignMode::Beneath, 0);
                } else {
                    inst.align_rect_mut(prev, AlignMode::ToTheRight, 0);
                    inst.align_rect_mut(prev, AlignMode::Bottom, 0);
                }
                // TODO only works for horizontal series pins
                prev = inst.lcm_bounds();
                let nodes: Vec<Node> = inst.io().flatten_vec();
                for (i, (node, cfg)) in nodes.iter().zip(self.pins.iter()).enumerate() {
                    match *cfg {
                        PinConfig::Series { partner, .. } => {
                            if !prev_nodes.is_empty() {
                                cell.connect(node, prev_nodes[partner]);
                            }
                        }
                        PinConfig::Parallel { .. } => {
                            if !prev_nodes.is_empty() {
                                cell.connect(node, prev_nodes[i]);
                            }
                            cell.skip_routing(*node);
                        }
                        _ => {
                            if !series_partners.contains(&i) {
                                cell.skip_routing(*node);
                            }
                        }
                    }
                }
                prev_nodes = nodes;
                cell.draw(inst)?;
            }
        }
        assert!(self.rows >= 1);
        assert!(self.cols >= 1);

        let stack =
            ctx.get_installation::<LayerStack<
                PdkLayer<<<T as Tile>::Schema as substrate::layout::schema::Schema>::Layer>,
            >>()
            .expect("must install ATOLL layer stack");
        let (abs, _paths) = get_abstract(self.tile.clone(), ctx.clone())?;
        // identify layers to analyze: parallel pins + 1, escape pins + 0/1/2
        let mut chk_layers = HashSet::new();
        for cfg in self.pins.iter() {
            match cfg {
                PinConfig::Parallel { layer, .. } => {
                    assert!(layer + 1 < stack.len());
                    chk_layers.insert(layer + 1);
                }
                PinConfig::Escape { layer, .. } => {
                    assert!(layer + 2 < stack.len());
                    chk_layers.insert(layer + 1);
                    chk_layers.insert(layer + 2);
                }
                PinConfig::Series { layer, .. } => {
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
        let mut escape_pin_data = HashMap::new();
        let mut series_pin_data = HashMap::new();

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
                        self.cols
                    } else {
                        assert!(side == Side::Top || side == Side::Bot);
                        self.rows
                    };

                    // get track coordinate `coordl` of max width on layer
                    // for each track intersecting `coordl` on layer+1,
                    // add it to the acceptable track list if it is free throughout the cell
                    let mut p1trks = Vec::new();
                    let coordl =
                        max_extent_track(&state, layer, dir, *net).expect("pin not present");
                    let (coord_gdl_min, coord_gdl_max) = if dir == Dir::Horiz {
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
                        (coord_gdl_min, coord_gdl_max)
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
                        for p1trk in 0..state.layer(layer1).cols() {
                            let trky = abs
                                .grid_to_physical(GridCoord {
                                    layer: layer1,
                                    x: coordl,
                                    y: p1trk,
                                })
                                .y;
                            println!("{min} <= {trky} <= {max}?");
                            if trky >= min && trky <= max && free_tracks[&layer1].contains(&p1trk) {
                                // p1trk is a candidate track
                                p1trks.push(LayerTrack {
                                    layer: layer1,
                                    track: p1trk,
                                });
                            }
                        }
                        (coord_gdl_min, coord_gdl_max)
                    };

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
                    escape_pin_data.insert(
                        net,
                        EscapePinData {
                            coordl,
                            dir,
                            coord_gdl_min,
                            coord_gdl_max,
                        },
                    );
                }
                PinConfig::Ignore => (),
                PinConfig::Series { partner, layer } => {
                    let output_net = *net;
                    let dir = abs.grid.stack.layer(layer).dir.track_dir();
                    let input_net = abs.ports[partner];
                    let dir_width = if dir == Dir::Horiz {
                        state.layer(layer).cols()
                    } else {
                        state.layer(layer).rows()
                    };
                    let dir_width_h = if !dir == Dir::Horiz {
                        state.layer(layer + 1).cols()
                    } else {
                        state.layer(layer + 1).rows()
                    };
                    println!("dir width = {dir_width}");
                    let coord_output =
                        max_extent_track(&state, layer, dir, output_net).expect("pin not present");
                    let coord_input =
                        max_extent_track(&state, layer, dir, input_net).expect("pin not present");
                    let (igdl_min, igdl_max) =
                        max_gdl_extent_on_track(&state, layer, dir, coord_input, input_net)
                            .unwrap();
                    let (ogdl_min, ogdl_max) =
                        max_gdl_extent_on_track(&state, layer, dir, coord_output, output_net)
                            .unwrap();
                    match_mapping.insert(net, MatchMapping::Series(match_input.len()));
                    let tracks = (0..n_series_pairs)
                        .map(|i| LayerTrack {
                            layer: layer + 1,
                            track: dir_width_h + i,
                        })
                        .collect();
                    match_input.push(tracks);
                    series_pin_data.insert(
                        net,
                        SeriesPinData {
                            coord_output,
                            coord_input,
                            igdl_min,
                            igdl_max,
                            ogdl_min,
                            ogdl_max,
                            dir_width,
                            dir_width_h,
                        },
                    );
                }
            }
        }

        println!("{:#?}", match_input);

        // match pins to tracks
        let match_output =
            create_match(MatchInput { items: match_input }).expect("failed to create matching");

        let bbox = cell.layout.bbox_rect();
        // strap parallel pins on matched track
        println!("top layer = {}", self.top_layer);
        let grid = RoutingGrid::new((*stack).clone(), 0..(self.top_layer + 1));
        let track_idx = |dir, base, i, delta| {
            if dir == Dir::Horiz {
                base as i64 - ((i + 1) * delta) as i64
            } else {
                base as i64 + (i * delta) as i64
            }
        };
        for (net, cfg) in abs.ports.iter().zip(self.pins.iter()) {
            match *cfg {
                PinConfig::Parallel { layer } => {
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
                    let lower_track = max_extent_full_track(&state, layer, dir, *net)
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
                PinConfig::Escape { .. } => {
                    let data = &escape_pin_data[net];
                    let mapping = match_mapping[net].as_ref().unwrap_escape();
                    let dir = data.dir;
                    let p1trk = match_output.pair[mapping.p1];
                    if dir == Dir::Horiz {
                        for _r in 0..self.rows {
                            for c in 0..self.cols {
                                // route on p1trk to p2trks[c]
                                let p2trk = match_output.pair[mapping.p2[c]];
                                let layer_grid = state.layer(p2trk.layer);
                                let delta2 = layer_grid.cols();
                                let idx2 = track_idx(dir, p2trk.track, c, delta2);
                                let span = grid.tracks(p2trk.layer).get(idx2);
                                let rect = Rect::from_dir_spans(dir, bbox.span(dir), span);
                                let layer = stack.layer(p2trk.layer).layer.clone();
                                cell.layout.draw(Shape::new(layer, rect))?;
                            }
                        }
                    } else {
                        for r in 0..self.rows {
                            for c in 0..self.cols {
                                let p2trk = match_output.pair[mapping.p2[r]];
                                let pt = abs.grid_to_physical(GridCoord {
                                    layer: p2trk.layer,
                                    x: p2trk.track,
                                    y: p1trk.track,
                                });
                                let trkcoords = grid.point_to_grid(
                                    pt,
                                    p1trk.layer,
                                    RoundingMode::Nearest,
                                    RoundingMode::Nearest,
                                );
                                let dstx = abs
                                    .track_to_grid(TrackCoord {
                                        layer: p1trk.layer,
                                        x: trkcoords.x,
                                        y: trkcoords.y,
                                    })
                                    .x;
                                // route on p1trk to p2trks[r]
                                // coordl to round(p2trks[r])
                                let layer_grid = state.layer(p1trk.layer);
                                let delta1 = layer_grid.cols();
                                let delta1gdl =
                                    state.layer(grid.grid_defining_layer(p1trk.layer)).rows();
                                let min = std::cmp::min(data.coordl, dstx);
                                let max = std::cmp::max(data.coordl, dstx);
                                let idx1 = track_idx(!dir, p1trk.track, r, delta1);
                                let min_idx1gdl = track_idx(dir, min, c, delta1gdl);
                                let max_idx1gdl = track_idx(dir, max, c, delta1gdl);
                                let coordl_shifted = track_idx(dir, max, c, delta1gdl);
                                let rect = grid.track(p1trk.layer, idx1, min_idx1gdl, max_idx1gdl);
                                let layer = stack.layer(p1trk.layer).layer.clone();
                                let via_maker = T::via_maker();
                                for shape in via_maker.draw_via(
                                    ctx.clone(),
                                    TrackCoord {
                                        layer: p1trk.layer,
                                        x: coordl_shifted,
                                        y: idx1,
                                    },
                                ) {
                                    cell.layout.draw(shape)?;
                                }
                                cell.layout.draw(Shape::new(layer, rect))?;

                                // route on p2trks[r] to edge of cell
                                let p2trk = match_output.pair[mapping.p2[r]];
                                let layer_grid = state.layer(p2trk.layer);
                                let delta2 = layer_grid.rows();
                                let idx2 = track_idx(dir, p2trk.track, c, delta2);
                                let span = grid.tracks(p2trk.layer).get(idx2);
                                let rect = Rect::from_dir_spans(dir, bbox.span(dir), span);
                                let layer = stack.layer(p2trk.layer).layer.clone();
                                for shape in via_maker.draw_via(
                                    ctx.clone(),
                                    TrackCoord {
                                        layer: p2trk.layer,
                                        x: idx2,
                                        y: idx1,
                                    },
                                ) {
                                    cell.layout.draw(shape)?;
                                }
                                cell.layout.draw(Shape::new(layer, rect))?;
                            }
                        }
                    }
                }
                PinConfig::Series { layer, .. } => {
                    let track = match_output.pair[*match_mapping[net].as_ref().unwrap_series()];
                    let dir = !self.dir;
                    let layer_grid = state.layer(track.layer);
                    let bot_grid = state.layer(layer);
                    let (n_perp, n_par, deltah, delta_gdl) = if self.dir == Dir::Vert {
                        (self.cols, self.rows, layer_grid.cols(), bot_grid.cols())
                    } else {
                        (self.rows, self.cols, layer_grid.rows(), bot_grid.rows())
                    };
                    let data = &series_pin_data[net];
                    let pdk_layer = stack.layer(track.layer).layer.clone();
                    for i in 0..n_perp - 1 {
                        let tidx = if i % 2 == 0 {
                            ((n_par - 1) * deltah + track.track) as i64
                        } else {
                            -((track.track - deltah) as i64)
                        };
                        // TODO: this only works if dir is horizontal
                        let coord_output =
                            data.coord_output as i64 - (data.dir_width * (i + 1)) as i64;
                        let coord_input =
                            data.coord_input as i64 - (data.dir_width * (i + 2)) as i64;
                        let gdl_min = std::cmp::min(coord_input, coord_output);
                        let gdl_max = std::cmp::max(coord_input, coord_output);
                        println!("tidx = {tidx}");
                        let recth = grid.track(track.layer, tidx, gdl_min, gdl_max);
                        cell.layout.draw(Shape::new(pdk_layer.clone(), recth))?;

                        let (igdl_min, igdl_max, ogdl_min, ogdl_max) = if i % 2 == 0 {
                            (
                                delta_gdl - data.igdl_min + delta_gdl * (n_perp - 1),
                                delta_gdl - data.igdl_max + delta_gdl * (n_perp - 1),
                                data.ogdl_min + delta_gdl * (n_perp - 1),
                                data.ogdl_max + delta_gdl * (n_perp - 1),
                            )
                        } else {
                            (
                                data.igdl_min,
                                data.igdl_max,
                                delta_gdl - data.ogdl_min,
                                delta_gdl - data.ogdl_max,
                            )
                        };
                        let r_input =
                            grid.track(layer, coord_input, igdl_min as i64, igdl_max as i64);
                        let r_input = Rect::from_dir_spans(
                            self.dir,
                            r_input.span(self.dir).union(recth.span(self.dir)),
                            r_input.span(!self.dir),
                        );
                        let pdk_layer = stack.layer(layer).layer.clone();
                        cell.layout.draw(Shape::new(pdk_layer.clone(), r_input))?;
                        let via_maker = T::via_maker();
                        let track_coord = if self.dir == Dir::Horiz {
                            TrackCoord {
                                layer: track.layer,
                                x: tidx,
                                y: coord_input,
                            }
                        } else {
                            TrackCoord {
                                layer: track.layer,
                                x: coord_input,
                                y: tidx,
                            }
                        };
                        for shape in via_maker.draw_via(ctx.clone(), track_coord) {
                            cell.layout.draw(shape)?;
                        }

                        let r_output =
                            grid.track(layer, coord_output, ogdl_min as i64, ogdl_max as i64);
                        let r_output = Rect::from_dir_spans(
                            self.dir,
                            r_output.span(self.dir).union(recth.span(self.dir)),
                            r_output.span(!self.dir),
                        );
                        cell.layout.draw(Shape::new(pdk_layer.clone(), r_output))?;
                        let track_coord = if self.dir == Dir::Horiz {
                            TrackCoord {
                                layer: track.layer,
                                x: tidx,
                                y: coord_output,
                            }
                        } else {
                            TrackCoord {
                                layer: track.layer,
                                x: coord_output,
                                y: tidx,
                            }
                        };
                        for shape in via_maker.draw_via(ctx.clone(), track_coord) {
                            cell.layout.draw(shape)?;
                        }
                    }
                    for i in 0..n_perp {
                        for j in 0..n_par - 1 {
                            assert_eq!(
                                data.coord_input, data.coord_output,
                                "series input/output pins must be aligned"
                            );
                            let coord_output =
                                data.coord_output as i64 - (data.dir_width * (i + 1)) as i64;
                            let (a, b) = if i % 2 == 0 {
                                (
                                    data.ogdl_min + delta_gdl * j,
                                    data.igdl_max + delta_gdl * (j + 1),
                                )
                            } else {
                                (
                                    delta_gdl - data.ogdl_min + delta_gdl * (j + 1),
                                    delta_gdl - data.igdl_max + delta_gdl * j,
                                )
                            };
                            let min = std::cmp::min(a, b);
                            let max = std::cmp::max(a, b);
                            let rect = grid.track(layer, coord_output, min as i64, max as i64);
                            let pdk_layer = stack.layer(layer).layer.clone();
                            cell.layout.draw(Shape::new(pdk_layer.clone(), rect))?;
                        }
                    }
                }
                PinConfig::Ignore => (),
                _ => unimplemented!(),
            }
        }
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

fn max_extent_full_track(
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

fn max_gdl_extent_on_track(
    state: &RoutingState<AbstractLayer>,
    layer: usize,
    dir: Dir,
    track: usize,
    net: NetId,
) -> Option<(usize, usize)> {
    if dir == Dir::Horiz {
        state
            .layer(layer)
            .iter_col(track)
            .enumerate()
            .filter_map(|(i, elt)| elt.is_routed_for_net(net).then_some(i))
            .minmax()
            .into_option()
    } else {
        state
            .layer(layer)
            .iter_row(track)
            .enumerate()
            .filter_map(|(i, elt)| elt.is_routed_for_net(net).then_some(i))
            .minmax()
            .into_option()
    }
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
                .filter_map(|(i, row)| {
                    let sum: usize = row
                        .clone()
                        .filter_map(|elt| elt.is_routed_for_net(net).then_some(1))
                        .sum();
                    (sum > 0).then_some((i, sum))
                })
                .max_by_key(|x| x.1),
            Dir::Vert => state
                .layer(layer)
                .iter_rows()
                .enumerate()
                .filter_map(|(i, col)| {
                    let sum: usize = col
                        .clone()
                        .filter_map(|elt| elt.is_routed_for_net(net).then_some(1))
                        .sum();
                    (sum > 0).then_some((i, sum))
                })
                .max_by_key(|x| x.1),
        }?
        .0,
    )
}
