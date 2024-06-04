//! Generate abstract views of layout cells.
use crate::grid::{AtollLayer, LayerSlice, LayerStack, PdkLayer, RoutingGrid, RoutingState};
use crate::{AssignedGridPoints, NetId, Orientation, PointState};
use grid::Grid;
use num::integer::{div_ceil, div_floor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::transform::{Transformation, TranslateMut};
use substrate::layout::element::Text;

use substrate::context::PdkContext;
use substrate::geometry::dir::Dir;
use substrate::geometry::point::Point;
use substrate::geometry::rect::Rect;
use substrate::geometry::union::BoundingUnion;
use substrate::io::layout::Builder;
use substrate::layout::bbox::LayerBbox;
use substrate::layout::element::Shape;
use substrate::layout::element::{CellId, Element, RawCell};
use substrate::layout::{CellBuilder, Draw, DrawReceiver, ExportsLayoutData, Layout};
use substrate::pdk::layers::{HasPin, Layer};
use substrate::pdk::Pdk;
use substrate::schematic::ExportsNestedData;
use substrate::{arcstr, layout};

/// An absolute coordinate referencing the defining tracks of a layer.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TrackCoord {
    /// The layer.
    pub layer: usize,
    /// The x-coordinate.
    pub x: i64,
    /// The y-coordinate.
    pub y: i64,
}

/// A coordinate within a grid (relative to the lower-left corner of the LCM bounds).
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GridCoord {
    /// The layer.
    pub layer: usize,
    /// The x-coordinate.
    pub x: usize,
    /// The y-coordinate.
    pub y: usize,
}

impl TrackCoord {
    /// Returns the coordinate in the given direction.
    pub fn coord(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.x,
            Dir::Vert => self.y,
        }
    }
}

impl GridCoord {
    /// Returns the coordinate in the given direction.
    pub fn coord(&self, dir: Dir) -> usize {
        match dir {
            Dir::Horiz => self.x,
            Dir::Vert => self.y,
        }
    }

    /// Set the coordinate along `dir` to `coord`, and return a copy of this `GridCoord`.
    pub fn with_coord(&self, dir: Dir, coord: usize) -> Self {
        match dir {
            Dir::Horiz => Self { x: coord, ..*self },
            Dir::Vert => Self { y: coord, ..*self },
        }
    }
}

/// The abstract view of an ATOLL tile.
///
/// # Coordinates
///
/// There are three coordinate systems used within the abstract view.
/// * Physical coordinates: the raw physical coordinate system of the cell, expressed in PDK database units.
/// * Track coordinates: track indices indexing the ATOLL [`LayerStack`]. Track 0 is typically centered at the origin, or immediately to the upper left of the origin.
/// * Grid coordinates: these have the same spacing as track coordinates, but are shifted so that (0, 0) is always at the lower left. These are used to index [`LayerAbstract`]s.
///
/// ATOLL provides the following utilities for converting between these coordinate systems:
/// * Grid to physical: [`Abstract::grid_to_physical`]
/// * Track to physical: [`Abstract::track_to_physical`]
/// * Grid to track: [`Abstract::grid_to_track`]
/// * Track to grid: [`Abstract::track_to_grid`]
/// * Physical to track: [`RoutingGrid::point_to_grid`]
///
/// For converting physical to grid: convert the physical coordinates to track coordinates,
/// then convert track coordinates to physical coordinates.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Abstract {
    /// The topmost ATOLL layer used within the tile.
    pub(crate) top_layer: usize,
    /// The bounds of the tile, in LCM units with respect to `top_layer`.
    ///
    /// The "origin" of the tile, in LCM units, is the lower left of this rectangle.
    pub(crate) lcm_bounds: Rect,
    /// The state of each layer, up to and including `top_layer`.
    ///
    /// Ports on layers not supported by ATOLL are ignored.
    layers: Vec<LayerAbstract>,
    /// A list of port net IDs.
    ///
    /// The order of net IDs matches that provided by [`layout::Cell::ports`].
    ports: Vec<NetId>,
    /// The routing grid used to produce this abstract view.
    pub(crate) grid: RoutingGrid<PdkLayer>,
}

impl Abstract {
    /// Returns the physical bounds of this abstract.
    pub fn physical_bounds(&self) -> Rect {
        let slice = self.slice();
        let w = slice.lcm_unit_width();
        let h = slice.lcm_unit_height();
        Rect::from_sides(
            self.lcm_bounds.left() * w,
            self.lcm_bounds.bot() * h,
            self.lcm_bounds.right() * w,
            self.lcm_bounds.top() * h,
        )
    }

    /// Returns a [`RoutingState`] corresponding to this abstract.
    pub fn routing_state(&self) -> RoutingState<PdkLayer> {
        let mut state = RoutingState::new(
            self.grid.stack.clone(),
            self.top_layer,
            self.lcm_bounds.width(),
            self.lcm_bounds.height(),
        );
        for (i, layer) in self.layers.iter().enumerate() {
            match layer {
                LayerAbstract::Available => {}
                LayerAbstract::Blocked => {
                    state
                        .layer_mut(i)
                        .fill(PointState::Blocked { has_via: false });
                }
                LayerAbstract::Detailed { states } => {
                    *state.layer_mut(i) = states.clone();
                }
            }
        }
        state
    }

    /// Populates an [`Abstract`] based on the provided [`RoutingState`].
    pub fn from_routing_state(&mut self, state: RoutingState<PdkLayer>) {
        for port in self.ports.iter_mut() {
            *port = state.roots[port];
        }
        for (layer, states) in self.layers.iter_mut().zip(state.layers) {
            *layer = LayerAbstract::Detailed { states }
        }
    }

    /// Converts a grid point to a physical point in the coordinates of the cell.
    ///
    /// See [coordinate systems](Abstract#coordinates) for more information.
    pub fn grid_to_physical(&self, coord: GridCoord) -> Point {
        let coord = self.grid_to_track(coord);
        self.track_to_physical(coord)
    }

    /// Converts a track point to a physical point in the coordinates of the cell.
    ///
    /// See [coordinate systems](Abstract#coordinates) for more information.
    pub fn track_to_physical(&self, coord: TrackCoord) -> Point {
        self.grid.xy_track_point(coord.layer, coord.x, coord.y)
    }

    fn xofs(&self, layer: usize) -> i64 {
        self.lcm_bounds.left() * self.slice().lcm_unit_width() / self.grid.xpitch(layer)
    }

    fn yofs(&self, layer: usize) -> i64 {
        self.lcm_bounds.bot() * self.slice().lcm_unit_height() / self.grid.ypitch(layer)
    }

    /// Converts a grid point to a track point in the coordinates of the cell.
    ///
    /// See [coordinate systems](Abstract#coordinates) for more information.
    pub fn grid_to_track(&self, coord: GridCoord) -> TrackCoord {
        TrackCoord {
            layer: coord.layer,
            x: coord.x as i64 + self.xofs(coord.layer),
            y: coord.y as i64 + self.yofs(coord.layer),
        }
    }

    /// Converts a track point to a grid point in the coordinates of the cell.
    ///
    /// See [coordinate systems](Abstract#coordinates) for more information.
    pub fn track_to_grid(&self, coord: TrackCoord) -> GridCoord {
        let x = coord.x - self.xofs(coord.layer);
        let y = coord.y - self.yofs(coord.layer);
        assert!(
            x >= 0,
            "track_to_grid: negative grid coordinates are not permitted: {x}"
        );
        assert!(
            y >= 0,
            "track_to_grid: negative grid coordinates are not permitted: {y}"
        );
        GridCoord {
            layer: coord.layer,
            x: x as usize,
            y: y as usize,
        }
    }

    pub(crate) fn slice(&self) -> LayerSlice<'_, PdkLayer> {
        self.grid.slice()
    }

    /// Returns the physical origin of this abstract (i.e. the physical coordinates
    /// of its lower left corner.
    pub fn physical_origin(&self) -> Point {
        self.lcm_bounds.lower_left() * self.slice().lcm_units()
    }

    pub(crate) fn block_available_on_layer(&mut self, layer: usize) {
        match &mut self.layers[layer] {
            abs @ LayerAbstract::Available => *abs = LayerAbstract::Blocked,
            LayerAbstract::Detailed { states } => {
                for i in 0..states.rows() {
                    for j in 0..states.cols() {
                        if let pt @ PointState::Available = &mut states[(i, j)] {
                            *pt = PointState::Blocked { has_via: false };
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Generates an abstract view of a layout cell.
    pub fn generate<PDK: Pdk, T: ExportsNestedData + ExportsLayoutData>(
        ctx: &PdkContext<PDK>,
        layout: &layout::Cell<T>,
    ) -> Abstract {
        let stack = ctx
            .get_installation::<LayerStack<PdkLayer>>()
            .expect("must install ATOLL layer stack");
        let virtual_layers = ctx.install_layers::<crate::VirtualLayers>();

        let cell = layout.raw();
        let bbox = cell
            .layer_bbox(virtual_layers.outline.id())
            .expect("cell must provide an outline on ATOLL virtual layer");

        let top = top_layer(cell, &stack).unwrap_or_default();
        let top = if top == 0 { 1 } else { top };

        let slice = stack.slice(0..top + 1);

        let xmin = div_floor(bbox.left(), slice.lcm_unit_width());
        let xmax = div_ceil(bbox.right(), slice.lcm_unit_width());
        let ymin = div_floor(bbox.bot(), slice.lcm_unit_height());
        let ymax = div_ceil(bbox.top(), slice.lcm_unit_height());
        let lcm_bounds = Rect::from_sides(xmin, ymin, xmax, ymax);

        let nx = lcm_bounds.width();
        let ny = lcm_bounds.height();

        let grid = RoutingGrid::new((*stack).clone(), 0..top + 1);
        let mut state = RoutingState::new((*stack).clone(), top, nx, ny);
        let mut ports = Vec::new();
        for (i, (_name, geom)) in cell.ports().enumerate() {
            let net = NetId(i);
            ports.push(net);
            for shape in geom.shapes() {
                if let Some(layer) = stack.layer_idx(shape.layer().drawing()) {
                    let rect = shape.bbox_rect();
                    if let Some(rect) = grid.shrink_to_grid(rect, layer) {
                        for x in rect.left()..=rect.right() {
                            for y in rect.bot()..=rect.top() {
                                let xofs = xmin * slice.lcm_unit_width() / grid.xpitch(layer);
                                let yofs = ymin * slice.lcm_unit_height() / grid.ypitch(layer);
                                let gx = usize::try_from(x.checked_sub(xofs).unwrap()).unwrap();
                                let gy = usize::try_from(y.checked_sub(yofs).unwrap()).unwrap();
                                if let Some(pt) = state.layer_mut(layer).get_mut(gx, gy) {
                                    *pt = PointState::Routed {
                                        net,
                                        has_via: false,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        let layers = state
            .layers
            .into_iter()
            .map(|states| LayerAbstract::Detailed { states })
            .collect();

        Abstract {
            top_layer: top,
            lcm_bounds,
            grid: RoutingGrid::new((*stack).clone(), 0..top + 1),
            ports,
            layers,
        }
    }
}

/// An abstract of an instance.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstanceAbstract {
    // todo: Arc and have instances reference same abstract if corresponding to same cell.
    abs: Abstract,
    orientation: Orientation,
    parent_net_ids: Vec<NetId>,
}

impl InstanceAbstract {
    pub(crate) fn new(
        mut abs: Abstract,
        loc: Point,
        orientation: Orientation,
        parent_net_ids: Vec<NetId>,
    ) -> Self {
        abs.lcm_bounds.translate_mut(loc);
        Self {
            abs,
            orientation,
            parent_net_ids,
        }
    }
}

impl InstanceAbstract {
    /// Returns the physical bounds of this instance.
    pub fn physical_bounds(&self) -> Rect {
        self.abs.physical_bounds()
    }

    /// Returns the LCM bounds of this instance.
    pub fn lcm_bounds(&self) -> Rect {
        self.abs.lcm_bounds
    }

    pub(crate) fn merge(
        abstracts: Vec<Self>,
        mut top_layer: usize,
        physical_bbox: Option<Rect>,
        ports: Vec<NetId>,
        assigned_grid_points: Vec<AssignedGridPoints>,
    ) -> Abstract {
        assert!(!abstracts.is_empty());

        for abs in &abstracts {
            top_layer = std::cmp::max(top_layer, abs.abs.top_layer);
        }
        let physical_bounds = abstracts
            .iter()
            .map(|abs| abs.physical_bounds())
            .reduce(|acc, next| acc.union(next))
            .unwrap()
            .bounding_union(&physical_bbox);

        let grid = RoutingGrid::new(abstracts[0].abs.grid.stack.clone(), 0..top_layer + 1);

        let new_bounds = grid.slice().expand_to_lcm_units(physical_bounds);
        let new_physical_bounds = grid.slice().lcm_to_physical_rect(new_bounds);
        let mut state = RoutingState::new(
            grid.stack.clone(),
            top_layer,
            new_bounds.width(),
            new_bounds.height(),
        );

        for inst in &abstracts {
            let net_translation: HashMap<_, _> = inst
                .abs
                .ports
                .iter()
                .copied()
                .zip(inst.parent_net_ids.iter().copied())
                .collect();
            for i in 0..=inst.abs.top_layer {
                let layer = grid.stack.layer(i);
                let parallel_pitch = layer.pitch();
                let perp_pitch = grid.stack.layer(grid.grid_defining_layer(i)).pitch();

                let (xpitch, ypitch) = match layer.dir().track_dir() {
                    Dir::Horiz => (perp_pitch, parallel_pitch),
                    Dir::Vert => (parallel_pitch, perp_pitch),
                };

                let left_offset =
                    (inst.physical_bounds().left() - new_physical_bounds.left()) / xpitch;
                let bot_offset =
                    (inst.physical_bounds().bot() - new_physical_bounds.bot()) / ypitch;
                let track_width = inst.physical_bounds().width() / xpitch;
                let track_height = inst.physical_bounds().height() / ypitch;

                for x in left_offset + 1..left_offset + track_width {
                    for y in bot_offset + 1..bot_offset + track_height {
                        let point_state = &mut state.layer_mut(i)[(x as usize, y as usize)];
                        match &inst.abs.layers[i] {
                            LayerAbstract::Available => {}
                            LayerAbstract::Blocked => {
                                assert_eq!(point_state, &PointState::Available);
                                *point_state = PointState::Blocked { has_via: false };
                            }
                            LayerAbstract::Detailed { states } => {
                                let new_state = states[match inst.orientation {
                                    Orientation::R0 => {
                                        ((x - left_offset) as usize, (y - bot_offset) as usize)
                                    }
                                    Orientation::R180 => (
                                        (left_offset + track_width - x) as usize,
                                        (bot_offset + track_height - y) as usize,
                                    ),
                                    Orientation::ReflectVert => (
                                        (x - left_offset) as usize,
                                        (bot_offset + track_height - y) as usize,
                                    ),
                                    Orientation::ReflectHoriz => (
                                        (left_offset + track_width - x) as usize,
                                        (y - bot_offset) as usize,
                                    ),
                                }];

                                // TODO: decide semantics for conflicting net labels
                                match new_state {
                                    PointState::Available => {}
                                    PointState::Blocked { has_via } => {
                                        *point_state = PointState::Blocked { has_via };
                                    }
                                    PointState::Routed { net, has_via } => {
                                        if let Some(translation) = net_translation.get(&net) {
                                            *point_state = PointState::Routed {
                                                net: *translation,
                                                has_via,
                                            };
                                        } else {
                                            *point_state = PointState::Blocked { has_via };
                                        }
                                    }
                                    PointState::Reserved { net } => {
                                        if let Some(translation) = net_translation.get(&net) {
                                            *point_state =
                                                PointState::Reserved { net: *translation };
                                        } else {
                                            *point_state = PointState::Blocked { has_via: false };
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for AssignedGridPoints {
            net,
            layer,
            bounds,
            only_if_available,
        } in assigned_grid_points
        {
            let pdk_layer = grid.stack.layer(layer);
            let defining_layer = grid.stack.layer(grid.grid_defining_layer(layer));
            let parallel_pitch = pdk_layer.pitch();
            let perp_pitch = defining_layer.pitch();

            let (xpitch, ypitch) = match pdk_layer.dir().track_dir() {
                Dir::Horiz => (perp_pitch, parallel_pitch),
                Dir::Vert => (parallel_pitch, perp_pitch),
            };

            let lcm_xpitch = grid.slice().lcm_unit_width();
            let lcm_ypitch = grid.slice().lcm_unit_height();

            let left = bounds.left() - new_bounds.left() * lcm_xpitch / xpitch;

            let bot = bounds.bot() - new_bounds.bot() * lcm_ypitch / ypitch;

            for i in left..=left + bounds.width() {
                for j in bot..=bot + bounds.height() {
                    if state.in_bounds(GridCoord {
                        x: i as usize,
                        y: j as usize,
                        layer,
                    }) {
                        let coords = (i as usize, j as usize);
                        let curr_state = state.layer(layer)[coords];
                        if !only_if_available || matches!(curr_state, PointState::Available) {
                            state.layer_mut(layer)[(i as usize, j as usize)] =
                                if let Some(net) = net {
                                    PointState::Routed {
                                        net,
                                        has_via: false,
                                    }
                                } else {
                                    PointState::Blocked { has_via: false }
                                };
                        }
                    }
                }
            }
        }

        Abstract {
            top_layer,
            lcm_bounds: new_bounds,
            layers: state
                .layers
                .into_iter()
                .map(|states| LayerAbstract::Detailed { states })
                .collect(),
            ports,
            grid,
        }
    }
}

/// The abstracted state of a single routing layer.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum LayerAbstract {
    /// The layer is fully available.
    Available,
    /// The layer is fully blocked.
    ///
    /// No routing on this layer is permitted.
    Blocked,
    /// The layer is available for routing and exposes the state of each point on the routing grid.
    Detailed {
        /// The state of the layer's grid points.
        states: Grid<PointState>,
    },
}

fn top_layer(cell: &RawCell, stack: &LayerStack<PdkLayer>) -> Option<usize> {
    let mut state = HashMap::new();
    top_layer_inner(cell, &mut state, stack)
}

fn top_layer_inner(
    cell: &RawCell,
    state: &mut HashMap<CellId, Option<usize>>,
    stack: &LayerStack<PdkLayer>,
) -> Option<usize> {
    if let Some(&layer) = state.get(&cell.id()) {
        return layer;
    }

    let mut top = None;

    for elt in cell.elements() {
        match elt {
            Element::Instance(inst) => {
                top = top.max(top_layer_inner(inst.raw_cell(), state, stack));
            }
            Element::Shape(s) => {
                if let Some(layer) = stack.layer_idx(s.layer()) {
                    top = top.max(Some(layer));
                }
            }
            Element::Text(_) => {
                // ignore text elements for the sake of calculating top layers
            }
        }
    }

    state.insert(cell.id(), top);
    top
}

/// A block that writes an abstract to a layout.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DebugAbstract {
    /// The abstract to be written.
    pub abs: Abstract,
    /// The layer stack corresponding to the abstract.
    pub stack: LayerStack<PdkLayer>,
}

impl Block for DebugAbstract {
    type Io = ();
    fn id() -> ArcStr {
        arcstr::literal!("debug_abstract")
    }
    fn name(&self) -> ArcStr {
        Self::id()
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsLayoutData for DebugAbstract {
    type LayoutData = ();
}

impl<PDK: Pdk> Layout<PDK> for DebugAbstract {
    #[inline]
    fn layout(
        &self,
        _io: &mut Builder<<Self as Block>::Io>,
        cell: &mut CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        cell.draw(self)?;
        Ok(())
    }
}

impl<PDK: Pdk> Draw<PDK> for &DebugAbstract {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> substrate::error::Result<()> {
        for (i, layer) in self.abs.layers.iter().enumerate() {
            let layer_id = self.abs.grid.stack.layer(i).id;
            match layer {
                LayerAbstract::Available => {}
                LayerAbstract::Blocked => {
                    recv.draw(Shape::new(layer_id, self.abs.physical_bounds()))?;
                }
                LayerAbstract::Detailed { states } => {
                    let (tx, ty) = states.size();
                    for x in 0..tx {
                        for y in 0..ty {
                            let pt = self.abs.grid_to_physical(GridCoord { layer: i, x, y });
                            let rect = match states[(x, y)] {
                                PointState::Available => Rect::from_point(pt).expand_all(22),
                                PointState::Blocked { has_via } => {
                                    if has_via {
                                        Rect::from_point(pt).expand_all(40)
                                    } else {
                                        Rect::from_point(pt).expand_all(33)
                                    }
                                }
                                PointState::Routed { has_via, .. } => {
                                    if has_via {
                                        Rect::from_point(pt).expand_all(37)
                                    } else {
                                        Rect::from_point(pt).expand_all(30)
                                    }
                                }
                                PointState::Reserved { .. } => Rect::from_point(pt).expand_all(35),
                            };
                            recv.draw(Shape::new(layer_id, rect))?;
                            let text = Text::new(
                                layer_id,
                                format!("({x},{y})"),
                                Transformation::translate(pt.x as f64, pt.y as f64),
                            );
                            recv.draw(text)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl<PDK: Pdk> Draw<PDK> for DebugAbstract {
    #[inline]
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> substrate::error::Result<()> {
        recv.draw(&self)
    }
}
