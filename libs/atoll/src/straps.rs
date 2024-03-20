use crate::abs::GridCoord;
use crate::grid::{AtollLayer, PdkLayer, RoutingState};
use crate::route::{Path, Router, RoutingNode};
use crate::{NetId, PointState};
use grid::Grid;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use substrate::geometry::dir::Dir;
use substrate::geometry::span::Span;

pub struct StrappingParams {
    /// Starting layer.
    start: usize,
    /// Parameters for each layer.
    layers: Vec<LayerStrappingParams>,
}

impl StrappingParams {
    pub fn new(start: usize, layers: Vec<LayerStrappingParams>) -> Self {
        Self { start, layers }
    }
}

pub enum LayerStrappingParams {
    Enumerated(Vec<usize>),
    OffsetPeriod { offset: usize, period: usize },
}

/// An ATOLL strapper.
pub trait Strapper: Send + Sync {
    /// Returns paths that represent the straps and vias that were drawn.
    fn strap(
        &self,
        routing_state: &mut RoutingState<PdkLayer>,
        to_strap: Vec<(NetId, StrappingParams)>,
    ) -> Vec<Path>;
}

/// A strapper that greedily straps the given nets in order.
pub struct GreedyStrapper;

impl Strapper for GreedyStrapper {
    fn strap(
        &self,
        routing_state: &mut RoutingState<PdkLayer>,
        to_strap: Vec<(NetId, StrappingParams)>,
    ) -> Vec<Path> {
        #[derive(Copy, Clone)]
        struct Strap {
            net: NetId,
            layer: usize,
            track: usize,
            start: usize,
            stop: usize,
            has_via: bool,
        }
        let mut grid_to_strap: Vec<Grid<Option<usize>>> = Vec::new();
        for layer in 0..routing_state.layers.len() {
            let rows = routing_state.layer(layer).rows();
            let cols = routing_state.layer(layer).cols();

            grid_to_strap.push(Grid::new(rows, cols));
        }
        let mut tentative_straps: Vec<Strap> = Vec::new();
        let mut paths: Vec<Path> = Vec::new();

        for (net, params) in to_strap {
            for layer in params.start..params.start + params.layers.len() {
                let rows = routing_state.layer(layer).rows();
                let cols = routing_state.layer(layer).cols();
                let via_space = routing_state.grid.stack.layer(layer).via_spacing();

                let (inner_x, inner_dim, outer_dim) =
                    match routing_state.grid.stack.layer(layer).dir().track_dir() {
                        Dir::Horiz => (true, rows, cols),
                        Dir::Vert => (false, cols, rows),
                    };

                let tracks = match &params.layers[layer - params.start] {
                    LayerStrappingParams::Enumerated(tracks) => tracks.clone(),
                    LayerStrappingParams::OffsetPeriod { offset, period } => {
                        (*offset..outer_dim).step_by(*period).collect()
                    }
                };

                for i in tracks {
                    if i >= outer_dim || i < 1 {
                        continue;
                    }
                    let mut start = None;
                    let mut end = None;
                    let mut strap_has_via = false;
                    for j in 0..inner_dim {
                        let (x, y) = if inner_x { (j, i) } else { (i, j) };
                        let coord = GridCoord { layer, x, y };
                        if routing_state.is_available_for_net(coord, net) {
                            if let Some(end) = end {
                                if j >= end + via_space && start.is_none() {
                                    start = Some(j);
                                }
                            } else if start.is_none() {
                                start = Some(j);
                            }
                        } else {
                            if let Some(start) = start {
                                let stop = j.checked_sub(via_space);
                                if let Some(stop) = stop {
                                    if start < stop {
                                        tentative_straps.push(Strap {
                                            net,
                                            layer,
                                            track: i,
                                            start,
                                            stop,
                                            has_via: strap_has_via,
                                        });
                                    }
                                }
                            }
                            end = Some(j);
                            strap_has_via = false;
                            start = None;
                        }
                    }
                    if let Some(start) = start {
                        tentative_straps.push(Strap {
                            net,
                            layer,
                            track: i,
                            start,
                            stop: inner_dim - 1,
                            has_via: strap_has_via,
                        })
                    }
                }
            }
        }
        for i in 0..tentative_straps.len() {
            let strap = tentative_straps[i];
            for track_coord in strap.start..=strap.stop {
                let track_dir = routing_state
                    .grid
                    .stack
                    .layer(strap.layer)
                    .dir()
                    .track_dir();
                let (x, y) = match track_dir {
                    Dir::Horiz => (track_coord, strap.track),
                    Dir::Vert => (strap.track, track_coord),
                };
                let coord = GridCoord {
                    layer: strap.layer,
                    x,
                    y,
                };
                grid_to_strap[strap.layer][(x, y)] = Some(i);
                routing_state[coord] = PointState::Routed {
                    net: strap.net,
                    has_via: routing_state.has_via(coord),
                };
                if let Some(ilt) = routing_state.ilt_down(coord) {
                    let mut has_via = false;
                    if ilt.to.layer != coord.layer {
                        for top in [ilt.to, coord] {
                            let track_dir = routing_state
                                .grid
                                .slice()
                                .layer(top.layer)
                                .dir()
                                .track_dir();
                            let via_spacing =
                                routing_state.grid.slice().layer(top.layer).via_spacing();
                            let routing_coord = top.coord(track_dir);
                            for i in (routing_coord + 1)
                                .checked_sub(via_spacing)
                                .unwrap_or_default()
                                ..routing_coord + via_spacing
                            {
                                let check_coord = top.with_coord(track_dir, i);
                                if i != routing_coord
                                    && routing_state.in_bounds(check_coord)
                                    && routing_state.has_via(check_coord)
                                {
                                    has_via = true;
                                }
                            }
                        }
                    }
                    if !has_via
                        && routing_state.is_routed_for_net(ilt.to, strap.net)
                        && ilt
                            .requires
                            .map(|n| routing_state.is_available_or_reserved_for_net(n, strap.net))
                            .unwrap_or(true)
                    {
                        routing_state[coord] = PointState::Routed {
                            net: strap.net,
                            has_via: true,
                        };
                        routing_state[ilt.to] = PointState::Routed {
                            net: strap.net,
                            has_via: true,
                        };
                        tentative_straps[i].has_via = true;
                        if let Some(bot_strap) = grid_to_strap[ilt.to.layer][(ilt.to.x, ilt.to.y)] {
                            tentative_straps[bot_strap].has_via = true;
                        }
                        paths.push(vec![(coord, ilt.to)]);
                    }
                }
            }
        }

        paths.extend(
            tentative_straps
                .into_iter()
                .filter_map(|strap| {
                    if !strap.has_via {
                        return None;
                    }
                    let (x1, y1, x2, y2) = match routing_state
                        .grid
                        .stack
                        .layer(strap.layer)
                        .dir()
                        .track_dir()
                    {
                        Dir::Horiz => (strap.start, strap.track, strap.stop, strap.track),
                        Dir::Vert => (strap.track, strap.start, strap.track, strap.stop),
                    };
                    Some(vec![(
                        GridCoord {
                            layer: strap.layer,
                            x: x1,
                            y: y1,
                        },
                        GridCoord {
                            layer: strap.layer,
                            x: x2,
                            y: y2,
                        },
                    )])
                })
                .collect::<Vec<_>>(),
        );
        paths
    }
}
