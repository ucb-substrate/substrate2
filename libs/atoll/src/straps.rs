//! ATOLL strap routing APIs.

/// ATOLL strap routing APIs.
use crate::abs::GridCoord;
use crate::grid::{AtollLayer, PdkLayer, RoutingState};
use crate::route::Path;
use crate::{NetId, PointState};
use grid::Grid;
use substrate::geometry::dir::Dir;
use substrate::geometry::rect::Rect;

/// Parameters for strap routing.
#[derive(Clone, Debug)]
pub struct StrappingParams {
    /// Starting layer.
    start: usize,
    /// The bounding box that straps should be confined to (in the top layer's coordinate frame).
    bounds: Option<Rect>,
    /// Parameters for each layer.
    layers: Vec<LayerStrappingParams>,
}

impl StrappingParams {
    /// Creates a new [`StrappingParams`].
    pub fn new(start: usize, layers: Vec<LayerStrappingParams>) -> Self {
        Self {
            start,
            bounds: None,
            layers,
        }
    }

    /// Sets the bounding box that straps should be confined to. Provided in the top layer's coordinate frame.
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

/// Strap parameters for a particular ATOLL layer.
#[derive(Clone, Debug)]
pub enum LayerStrappingParams {
    /// Enumerated track indexes.
    Enumerated(Vec<usize>),
    /// A track offset and period describing the strap locations.
    OffsetPeriod {
        /// The offset of the first strap.
        offset: usize,
        /// The strapping period.
        period: usize,
    },
    /// Places straps wherever corresponding nets are found in the layer directly below.
    ViaDown {
        /// The minimum period between adjacent straps.
        ///
        /// TODO: currently unimplemented.
        min_period: usize,
    },
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

#[derive(Copy, Clone)]
struct Strap {
    net: NetId,
    layer: usize,
    track: usize,
    start: usize,
    stop: usize,
    has_via: bool,
}

struct GreedyStrapperState<'a> {
    routing_state: &'a mut RoutingState<PdkLayer>,
    to_strap: Vec<(NetId, StrappingParams)>,
    grid_to_strap: Vec<Grid<Option<usize>>>,
    tentative_straps: Vec<Strap>,
    paths: Vec<Path>,
}

impl<'a> GreedyStrapperState<'a> {
    fn new(
        routing_state: &'a mut RoutingState<PdkLayer>,
        to_strap: Vec<(NetId, StrappingParams)>,
    ) -> Self {
        let mut grid_to_strap: Vec<Grid<Option<usize>>> = Vec::new();
        for layer in 0..routing_state.layers.len() {
            let rows = routing_state.layer(layer).rows();
            let cols = routing_state.layer(layer).cols();

            grid_to_strap.push(Grid::new(rows, cols));
        }
        Self {
            routing_state,
            to_strap,
            grid_to_strap,
            tentative_straps: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn strap_idx(&self, coord: GridCoord) -> Option<usize> {
        self.grid_to_strap[coord.layer][(coord.x, coord.y)]
    }

    fn strap(&self, coord: GridCoord) -> Option<Strap> {
        self.strap_idx(coord).map(|idx| self.tentative_straps[idx])
    }

    fn push_strap_if_valid(&mut self, mut strap: Strap, params: &LayerStrappingParams) {
        if strap.start >= strap.stop {
            return;
        }

        let mut vias: Vec<(GridCoord, GridCoord)> = Vec::new();
        for track_coord in strap.start..=strap.stop {
            let track_dir = self
                .routing_state
                .grid
                .stack
                .layer(strap.layer)
                .dir()
                .track_dir();
            let strap_via_spacing = self
                .routing_state
                .grid
                .slice()
                .layer(strap.layer)
                .strap_via_spacing();
            let (x, y) = match track_dir {
                Dir::Horiz => (track_coord, strap.track),
                Dir::Vert => (strap.track, track_coord),
            };
            let coord = GridCoord {
                layer: strap.layer,
                x,
                y,
            };
            if let LayerStrappingParams::ViaDown { min_period } = params {
                for track in
                    (strap.track + 1).saturating_sub(*min_period)..strap.track + *min_period
                {
                    if track == strap.track {
                        continue;
                    }
                    let (x, y) = match track_dir {
                        Dir::Horiz => (track_coord, track),
                        Dir::Vert => (track, track_coord),
                    };
                    let check_coord = GridCoord {
                        layer: strap.layer,
                        x,
                        y,
                    };
                    // Cannot have routing on same layer within `min_period` of current track.
                    if self.routing_state.in_bounds(check_coord)
                        && (self.routing_state.is_routed_for_net(check_coord, strap.net)
                            || self
                                .strap(check_coord)
                                .map(|other_strap| {
                                    self.routing_state.roots[&other_strap.net]
                                        == self.routing_state.roots[&strap.net]
                                })
                                .unwrap_or(false))
                    {
                        return;
                    }
                }
            }
            if let Some(ilt) = self.routing_state.ilt_down(coord) {
                let mut has_via = false;
                if ilt.to.layer != coord.layer {
                    for top in [ilt.to, coord] {
                        let track_dir = self
                            .routing_state
                            .grid
                            .slice()
                            .layer(top.layer)
                            .dir()
                            .track_dir();
                        let via_spacing = if self.strap_idx(top).is_some() {
                            self.routing_state
                                .grid
                                .slice()
                                .layer(top.layer)
                                .strap_via_spacing()
                        } else {
                            self.routing_state
                                .grid
                                .slice()
                                .layer(top.layer)
                                .via_spacing()
                        };
                        let routing_coord = top.coord(track_dir);
                        for i in (routing_coord + 1)
                            .checked_sub(via_spacing)
                            .unwrap_or_default()
                            ..routing_coord + via_spacing
                        {
                            let check_coord = top.with_coord(track_dir, i);
                            if i != routing_coord
                                && self.routing_state.in_bounds(check_coord)
                                && self.routing_state.has_via(check_coord)
                            {
                                has_via = true;
                            }
                        }
                    }
                }
                if let Some((from, _)) = vias.last() {
                    if from.coord(track_dir) + strap_via_spacing > track_coord
                        && from.coord(track_dir) < track_coord + strap_via_spacing
                    {
                        has_via = true;
                    }
                }
                if !has_via
                    && (self.routing_state.is_routed_for_net(ilt.to, strap.net)
                        || self
                            .strap(ilt.to)
                            .map(|bot_strap| {
                                self.routing_state.roots[&bot_strap.net]
                                    == self.routing_state.roots[&strap.net]
                            })
                            .unwrap_or(false))
                    && ilt
                        .requires
                        .map(|n| {
                            self.routing_state
                                .is_available_or_reserved_for_net(n, strap.net)
                        })
                        .unwrap_or(true)
                {
                    strap.has_via = true;
                    vias.push((coord, ilt.to));
                }
            }
        }

        if !matches!(params, LayerStrappingParams::ViaDown { .. }) || strap.has_via {
            for via in vias {
                let (from, to) = via;
                self.routing_state[from] = PointState::Routed {
                    net: strap.net,
                    has_via: true,
                };
                self.routing_state[to] = PointState::Routed {
                    net: strap.net,
                    has_via: true,
                };
                if let Some(bot_strap) = self.strap_idx(to) {
                    self.tentative_straps[bot_strap].has_via = true;
                }
                self.paths.push(vec![(from, to)]);
            }
            self.tentative_straps.push(strap);

            for track_coord in strap.start..=strap.stop {
                let track_dir = self
                    .routing_state
                    .grid
                    .stack
                    .layer(strap.layer)
                    .dir()
                    .track_dir();
                let (x, y) = match track_dir {
                    Dir::Horiz => (track_coord, strap.track),
                    Dir::Vert => (strap.track, track_coord),
                };
                self.grid_to_strap[strap.layer][(x, y)] = Some(self.tentative_straps.len() - 1);
            }
        }
    }

    fn compute_tentative_straps(&mut self) {
        for (net, params) in self.to_strap.clone() {
            for layer in params.start..params.start + params.layers.len() {
                let rows = self.routing_state.layer(layer).rows();
                let cols = self.routing_state.layer(layer).cols();
                let via_space = self.routing_state.grid.stack.layer(layer).via_spacing();

                let (start_x, start_y, end_x, end_y) = if let Some(bbox) = &params.bounds {
                    let pdk_layer = self.routing_state.grid.stack.layer(layer);
                    let defining_layer = self
                        .routing_state
                        .grid
                        .stack
                        .layer(self.routing_state.grid.grid_defining_layer(layer));
                    let parallel_pitch = pdk_layer.pitch();
                    let perp_pitch = defining_layer.pitch();

                    let (xpitch, ypitch) = match pdk_layer.dir().track_dir() {
                        Dir::Horiz => (perp_pitch, parallel_pitch),
                        Dir::Vert => (parallel_pitch, perp_pitch),
                    };

                    let lcm_xpitch = self.routing_state.grid.slice().lcm_unit_width();
                    let lcm_ypitch = self.routing_state.grid.slice().lcm_unit_height();

                    let left = bbox.left() * lcm_xpitch / xpitch;
                    let bot = bbox.bot() * lcm_ypitch / ypitch;
                    let right = bbox.right() * lcm_xpitch / xpitch;
                    let top = bbox.top() * lcm_ypitch / ypitch;
                    (
                        left as usize,
                        bot as usize,
                        std::cmp::min(right as usize, rows),
                        std::cmp::min(top as usize, cols),
                    )
                } else {
                    (0, 0, rows, cols)
                };

                let (inner_x, inner_start, outer_start, inner_end, outer_end) =
                    match self.routing_state.grid.stack.layer(layer).dir().track_dir() {
                        Dir::Horiz => (true, start_x, start_y, end_x, end_y),
                        Dir::Vert => (false, start_y, start_x, end_y, end_x),
                    };

                let tracks = match &params.layers[layer - params.start] {
                    LayerStrappingParams::Enumerated(tracks) => tracks.clone(),
                    LayerStrappingParams::OffsetPeriod { offset, period } => {
                        (outer_start + *offset + 1..outer_end)
                            .step_by(*period)
                            .collect()
                    }
                    LayerStrappingParams::ViaDown { .. } => (outer_start + 1..outer_end).collect(),
                };

                for i in tracks {
                    if i >= outer_end || i < outer_start + 1 {
                        continue;
                    }
                    let mut start = None;
                    let mut end = None;
                    for j in inner_start + 1..inner_end {
                        let (x, y) = if inner_x { (j, i) } else { (i, j) };
                        let coord = GridCoord { layer, x, y };
                        if self.routing_state.is_available_for_net(coord, net)
                            && self.strap_idx(coord).is_none()
                        {
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
                                    self.push_strap_if_valid(
                                        Strap {
                                            net,
                                            layer,
                                            track: i,
                                            start,
                                            stop,
                                            has_via: false,
                                        },
                                        &params.layers[layer - params.start],
                                    );
                                }
                            }
                            end = Some(j);
                            start = None;
                        }
                    }
                    if let Some(start) = start {
                        let stop = inner_end.checked_sub(via_space);
                        if let Some(stop) = stop {
                            self.push_strap_if_valid(
                                Strap {
                                    net,
                                    layer,
                                    track: i,
                                    start,
                                    stop,
                                    has_via: false,
                                },
                                &params.layers[layer - params.start],
                            );
                        }
                    }
                }
            }
        }
    }

    fn finalize_straps(&mut self) {
        for strap in &self.tentative_straps {
            if !strap.has_via {
                continue;
            }
            for track_coord in strap.start..=strap.stop {
                let track_dir = self
                    .routing_state
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
                self.routing_state[coord] = PointState::Routed {
                    net: strap.net,
                    has_via: track_coord == strap.start
                        || track_coord == strap.stop
                        || self.routing_state.has_via(coord),
                };
            }
            let (x1, y1, x2, y2) = match self
                .routing_state
                .grid
                .stack
                .layer(strap.layer)
                .dir()
                .track_dir()
            {
                Dir::Horiz => (strap.start, strap.track, strap.stop, strap.track),
                Dir::Vert => (strap.track, strap.start, strap.track, strap.stop),
            };
            self.paths.push(vec![(
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
        }
    }
}

impl Strapper for GreedyStrapper {
    fn strap(
        &self,
        routing_state: &mut RoutingState<PdkLayer>,
        to_strap: Vec<(NetId, StrappingParams)>,
    ) -> Vec<Path> {
        let mut state = GreedyStrapperState::new(routing_state, to_strap);

        state.compute_tentative_straps();
        state.finalize_straps();

        state.paths
    }
}
