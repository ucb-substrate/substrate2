//! Generate abstract views of layout cells.
use crate::grid::{LayerSlice, LayerStack, PdkLayer, RoutingGrid, RoutingState};
use crate::{NetId, PointState};
use grid::Grid;
use num::integer::{div_ceil, div_floor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::transform::Transformation;
use substrate::layout::element::Text;

use substrate::context::PdkContext;
use substrate::geometry::point::Point;
use substrate::geometry::rect::Rect;
use substrate::io::layout::Builder;
use substrate::layout::bbox::LayerBbox;
use substrate::layout::element::Shape;
use substrate::layout::element::{CellId, Element, RawCell};
use substrate::layout::{CellBuilder, Draw, DrawReceiver, ExportsLayoutData, Layout};
use substrate::pdk::layers::{HasPin, Layer};
use substrate::pdk::Pdk;
use substrate::schematic::ExportsNestedData;
use substrate::{arcstr, layout};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TrackCoord {
    pub layer: usize,
    pub x: i64,
    pub y: i64,
}

pub struct GridCoord {
    pub layer: usize,
    pub x: usize,
    pub y: usize,
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
/// * Grid to physical: [`AtollAbstract::grid_to_physical`]
/// * Track to physical: [`AtollAbstract::track_to_physical`]
/// * Grid to track: [`AtollAbstract::grid_to_track`]
/// * Track to grid: [`AtollAbstract::track_to_grid`]
/// * Physical to track: [`RoutingGrid::point_to_grid`]
///
/// For converting physical to grid: convert the physical coordinates to track coordinates,
/// then convert track coordinates to physical coordinates.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AtollAbstract {
    /// The topmost ATOLL layer used within the tile.
    pub(crate) top_layer: usize,
    /// The bounds of the tile, in LCM units with respect to `top_layer`.
    ///
    /// The "origin" of the tile, in LCM units, is the lower left of this rectangle.
    lcm_bounds: Rect,
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

impl AtollAbstract {
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

    /// Converts a grid point to a physical point in the coordinates of the cell.
    ///
    /// See [coordinate systems](AtollAbstract#coordinates) for more information.
    pub fn grid_to_physical(&self, coord: GridCoord) -> Point {
        let coord = self.grid_to_track(coord);
        self.track_to_physical(coord)
    }

    /// Converts a track point to a physical point in the coordinates of the cell.
    ///
    /// See [coordinate systems](AtollAbstract#coordinates) for more information.
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
    /// See [coordinate systems](AtollAbstract#coordinates) for more information.
    pub fn grid_to_track(&self, coord: GridCoord) -> TrackCoord {
        TrackCoord {
            layer: coord.layer,
            x: coord.x as i64 + self.xofs(coord.layer),
            y: coord.y as i64 + self.yofs(coord.layer),
        }
    }

    /// Converts a track point to a grid point in the coordinates of the cell.
    ///
    /// See [coordinate systems](AtollAbstract#coordinates) for more information.
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

    pub fn physical_origin(&self) -> Point {
        self.lcm_bounds.lower_left() * self.slice().lcm_units()
    }

    /// Generates an abstract view of a layout cell.
    pub fn generate<PDK: Pdk, T: ExportsNestedData + ExportsLayoutData>(
        ctx: &PdkContext<PDK>,
        layout: &layout::Cell<T>,
    ) -> AtollAbstract {
        let stack = ctx
            .get_installation::<LayerStack<PdkLayer>>()
            .expect("must install ATOLL layer stack");
        let virtual_layers = ctx.install_layers::<crate::VirtualLayers>();

        let cell = layout.raw();
        let bbox = cell
            .layer_bbox(virtual_layers.outline.id())
            .expect("cell must provide an outline on ATOLL virtual layer");

        let top = top_layer(cell, &*stack)
            .expect("cell did not have any ATOLL routing layers; cannot produce an abstract");
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
        for (i, (name, geom)) in cell.ports().enumerate() {
            let net = NetId(i);
            ports.push(net);
            if let Some(layer) = stack.layer_idx(geom.primary.layer().drawing()) {
                let rect = match geom.primary.shape() {
                    substrate::geometry::shape::Shape::Rect(r) => *r,
                    substrate::geometry::shape::Shape::Polygon(p) => {
                        p.bbox().expect("empty polygons are unsupported")
                    }
                };
                println!("source rect = {rect:?}");
                if let Some(rect) = grid.shrink_to_grid(rect, layer) {
                    println!("grid rect = {rect:?}");
                    for x in rect.left()..=rect.right() {
                        for y in rect.bot()..=rect.top() {
                            let xofs = xmin * slice.lcm_unit_width() / grid.xpitch(layer);
                            let yofs = ymin * slice.lcm_unit_height() / grid.ypitch(layer);
                            state.layer_mut(layer)[((x - xofs) as usize, (y - yofs) as usize)] =
                                PointState::Routed {
                                    net,
                                    via_down: false,
                                    via_up: false,
                                };
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

        AtollAbstract {
            top_layer: top,
            lcm_bounds,
            grid: RoutingGrid::new((*stack).clone(), 0..top + 1),
            ports,
            layers,
        }
    }
}

/// The abstracted state of a single routing layer.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum LayerAbstract {
    /// The layer is fully blocked.
    ///
    /// No routing on this layer is permitted.
    Blocked,
    /// The layer is available for routing and exposes the state of each point on the routing grid.
    Detailed { states: Grid<PointState> },
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DebugAbstract {
    pub abs: AtollAbstract,
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
                LayerAbstract::Blocked => {
                    recv.draw(Shape::new(layer_id, self.abs.physical_bounds()))?;
                }
                LayerAbstract::Detailed { states } => {
                    let (tx, ty) = states.size();
                    for x in 0..tx {
                        for y in 0..ty {
                            let pt = self.abs.grid_to_physical(GridCoord { layer: i, x, y });
                            let rect = match states[(x, y)] {
                                PointState::Available => Rect::from_point(pt).expand_all(20),
                                PointState::Obstructed => Rect::from_point(pt).expand_all(40),
                                PointState::Routed { .. } => Rect::from_point(pt).expand_all(30),
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