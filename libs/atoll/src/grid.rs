//! Uniform routing grids and layer stacks.
use crate::{NetId, PointState, RoutingDir};
use grid::Grid;
use num::integer::{div_ceil, div_floor};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::abs::GridCoord;
use crate::route::RoutingNode;
use std::ops::{Index, IndexMut, Range};
use substrate::context::{ContextBuilder, Installation};
use substrate::geometry::corner::Corner;
use substrate::geometry::dims::Dims;
use substrate::geometry::dir::Dir;
use substrate::geometry::point::Point;
use substrate::geometry::rect::Rect;
use substrate::geometry::span::Span;
use substrate::layout::element::Shape;
use substrate::layout::tracks::{RoundingMode, Tracks, UniformTracks};
use substrate::layout::{Draw, DrawReceiver};
use substrate::pdk::layers::LayerId;
use substrate::pdk::Pdk;

/// The offset of tracks relative to tile edges.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TrackOffset {
    /// Tracks centered at tile edges.
    None,
    /// Tracks offset by half-pitch from tile edges.
    HalfPitch,
}

/// An ATOLL-compatible routing layer.
pub trait AtollLayer {
    /// The preferred routing direction.
    fn dir(&self) -> RoutingDir;
    /// The line width on this layer.
    fn line(&self) -> i64;
    /// The space between adjacent tracks on this layer.
    fn space(&self) -> i64;
    /// An offset that shifts the first track of the layer.
    fn offset(&self) -> TrackOffset;
    /// The amount by which this layer should extend beyond the center line of a track on the this layer's grid defining layer.
    fn endcap(&self) -> i64 {
        0
    }
    /// The minimum spacing between adjacent vias on the same metal track.
    fn via_spacing(&self) -> usize {
        1
    }
    /// The minimum spacing between adjacent vias on the same power strap.
    fn strap_via_spacing(&self) -> usize {
        1
    }

    /// The line + space of this layer.
    ///
    /// The default implementation should not generally be overridden.
    fn pitch(&self) -> i64 {
        self.line() + self.space()
    }
    /// The offset of the first track in physical units.
    fn physical_offset(&self) -> i64 {
        match self.offset() {
            TrackOffset::None => 0,
            TrackOffset::HalfPitch => self.pitch() / 2,
        }
    }
}

/// An abstract layer with no relation to a physical layer in any process.
///
/// Just a set of numeric constants.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AbstractLayer {
    /// The preferred routing direction.
    pub dir: RoutingDir,
    /// The line width.
    pub line: i64,
    /// The space between adjacent tracks.
    pub space: i64,
    /// The offset of tracks relative to tile edges.
    pub offset: TrackOffset,
    /// How far to extend a track beyond the center-to-center intersection point with a track on the layer below.
    pub endcap: i64,
    /// The minimum spacing between adjacent vias on the same metal track.
    pub via_spacing: usize,
    /// The minimum spacing between adjacent vias on the same power strap.
    pub strap_via_spacing: usize,
}

/// An ATOLL-layer associated with a layer provided by a PDK.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PdkLayer {
    /// The [`LayerId`] of the PDK layer.
    pub id: LayerId,
    /// The constants associated with this layer.
    pub inner: AbstractLayer,
}

impl AsRef<LayerId> for PdkLayer {
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}

impl AbstractLayer {
    /// The (infinite) set of tracks on this layer.
    pub fn tracks(&self) -> UniformTracks {
        UniformTracks::with_offset(self.line, self.space, self.physical_offset())
    }
}

/// A stack of layers with alternating track directions
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct LayerStack<L> {
    /// The list of layers, ordered from bottom to top.
    pub layers: Vec<L>,
    /// The coordinate at which all vertical tracks are aligned.
    pub offset_x: i64,
    /// The coordinate at which all horizontal tracks are aligned.
    pub offset_y: i64,
}

impl<L: Any + Send + Sync> Installation for LayerStack<L> {
    fn post_install(&self, _ctx: &mut ContextBuilder) {}
}

/// A contiguous slice of layers in a [`LayerStack`].
#[derive(Copy, Clone)]
pub struct LayerSlice<'a, L> {
    stack: &'a LayerStack<L>,
    start: usize,
    end: usize,
}

impl AtollLayer for AbstractLayer {
    fn dir(&self) -> RoutingDir {
        self.dir
    }

    fn line(&self) -> i64 {
        self.line
    }

    fn space(&self) -> i64 {
        self.space
    }

    fn offset(&self) -> TrackOffset {
        self.offset
    }

    fn endcap(&self) -> i64 {
        self.endcap
    }

    fn via_spacing(&self) -> usize {
        self.via_spacing
    }

    fn strap_via_spacing(&self) -> usize {
        self.strap_via_spacing
    }
}

impl AtollLayer for PdkLayer {
    fn dir(&self) -> RoutingDir {
        self.inner.dir()
    }

    fn line(&self) -> i64 {
        self.inner.line()
    }

    fn space(&self) -> i64 {
        self.inner.space()
    }

    fn offset(&self) -> TrackOffset {
        self.inner.offset()
    }

    fn endcap(&self) -> i64 {
        self.inner.endcap()
    }

    fn via_spacing(&self) -> usize {
        self.inner.via_spacing()
    }

    fn strap_via_spacing(&self) -> usize {
        self.inner.strap_via_spacing()
    }
}

impl<L> LayerStack<L> {
    /// Whether or not this layer stack is empty (ie. contains no layers).
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// The number of layers in this stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// A slice containing all layers in this stack.
    pub fn all(&self) -> LayerSlice<L> {
        self.slice(0..self.layers.len())
    }

    /// A slice containing the specified set of layers.
    pub fn slice(&self, range: Range<usize>) -> LayerSlice<L> {
        LayerSlice {
            stack: self,
            start: range.start,
            end: range.end,
        }
    }

    /// The layer with the given index.
    ///
    /// # Panics
    ///
    /// Panics if the layer index is out of bounds.
    pub fn layer(&self, layer: usize) -> &L {
        &self.layers[layer]
    }
}

impl<L: AtollLayer> LayerStack<L> {
    /// The set of tracks on the given layer index.
    pub fn tracks(&self, layer: usize) -> UniformTracks {
        let layer = &self.layers[layer];
        let ofs = match layer.dir().track_dir() {
            Dir::Vert => self.offset_x,
            Dir::Horiz => self.offset_y,
        };
        UniformTracks::with_offset(layer.line(), layer.space(), layer.physical_offset() + ofs)
    }

    /// Returns whether or not the layer stack is valid.
    ///
    /// Checks that all layers have alternating track directions.
    pub fn is_valid(&self) -> bool {
        if self.len() <= 1 {
            return false;
        }

        let mut dir = self.layer(0).dir().track_dir();
        for layer in &self.layers[1..] {
            let next_dir = layer.dir().track_dir();
            if next_dir == dir {
                return false;
            }
            dir = next_dir;
        }
        true
    }
}

impl LayerStack<PdkLayer> {
    /// Returns the index corresponding to the given [`LayerId`].
    pub fn layer_idx(&self, id: LayerId) -> Option<usize> {
        for (i, layer) in self.layers.iter().enumerate() {
            if layer.id == id {
                return Some(i);
            }
        }
        None
    }
}

impl<'a, L: AtollLayer> LayerSlice<'a, L> {
    /// A single LCM unit in the given direction.
    pub fn lcm_unit(&self, dir: Dir) -> i64 {
        (self.start..self.end)
            .map(|l| self.layer(l))
            .filter(|&l| l.dir().track_dir() == !dir)
            .map(|l| l.pitch())
            .fold(1, num::integer::lcm)
    }

    /// A single LCM unit width.
    pub fn lcm_unit_width(&self) -> i64 {
        self.lcm_unit(Dir::Horiz)
    }

    /// A single LCM unit height.
    pub fn lcm_unit_height(&self) -> i64 {
        self.lcm_unit(Dir::Vert)
    }

    /// The LCM units in both directions.
    pub fn lcm_units(&self) -> Dims {
        Dims::new(self.lcm_unit_width(), self.lcm_unit_height())
    }

    /// The range of layer indices in this slice.
    fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// The set of tracks on the given layer.
    pub fn tracks(&self, layer: usize) -> UniformTracks {
        assert!(
            self.range().contains(&layer),
            "layer {layer} out of bounds for layer slice"
        );
        self.stack.tracks(layer)
    }

    /// The layer with the given index.
    ///
    /// Note that the index is with respect to the underlying layer stack,
    /// not the start index of the slice.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn layer(&self, layer: usize) -> &L {
        assert!(
            self.range().contains(&layer),
            "layer {layer} out of bounds for layer slice"
        );
        self.stack.layer(layer)
    }

    /// Expands a rectangle in physical coordinates to LCM units.
    pub fn expand_to_lcm_units(&self, rect: Rect) -> Rect {
        let xmin = div_floor(rect.left(), self.lcm_unit_width());
        let xmax = div_ceil(rect.right(), self.lcm_unit_width());
        let ymin = div_floor(rect.bot(), self.lcm_unit_height());
        let ymax = div_ceil(rect.top(), self.lcm_unit_height());
        Rect::from_sides(xmin, ymin, xmax, ymax)
    }

    /// Shrinks a rectangle in physical coordinates to LCM units.
    pub fn shrink_to_lcm_units(&self, rect: Rect) -> Option<Rect> {
        let xmin = div_ceil(rect.left(), self.lcm_unit_width());
        let xmax = div_floor(rect.right(), self.lcm_unit_width());
        let ymin = div_ceil(rect.bot(), self.lcm_unit_height());
        let ymax = div_floor(rect.top(), self.lcm_unit_height());
        Rect::from_sides_option(xmin, ymin, xmax, ymax)
    }

    /// Converts a point in LCM units to a point in physical units.
    pub fn lcm_to_physical(&self, pt: Point) -> Point {
        Point::new(pt.x * self.lcm_unit_width(), pt.y * self.lcm_unit_height())
    }

    /// Converts a rectangle in LCM units to a point in physical units.
    pub fn lcm_to_physical_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            self.lcm_to_physical(rect.lower_left()),
            self.lcm_to_physical(rect.upper_right()),
        )
    }
}

/// A fixed-size routing grid.
///
/// Does not store the state of track points in the grid.
/// For that functionality, see [`RoutingState`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RoutingGrid<L> {
    /// The layer stack.
    pub stack: LayerStack<L>,
    /// The start layer (inclusive)
    start: usize,
    /// The end layer (not inclusive)
    end: usize,
}

impl<L> RoutingGrid<L> {
    /// Creates a new routing grid with the given properties.
    pub fn new(stack: LayerStack<L>, layers: Range<usize>) -> Self {
        Self {
            stack,
            start: layers.start,
            end: layers.end,
        }
    }

    /// Returns the set of layers contained in this grid.
    pub fn layers(&self) -> Range<usize> {
        self.start..self.end
    }

    /// The layer slice representing the layers used by this routing grid.
    pub fn slice(&self) -> LayerSlice<'_, L> {
        self.stack.slice(self.start..self.end)
    }
}

impl<L: AtollLayer> RoutingGrid<L> {
    /// The layer that defines the cross tracks for `layer`.
    pub(crate) fn grid_defining_layer(&self, layer: usize) -> usize {
        // The grid for layer N is formed by the tracks for layer N and the tracks for layer N-1,
        // except for N=0. For layer 0, the grid is formed by layers 0 and 1.
        if layer == 0 {
            1
        } else {
            layer - 1
        }
    }

    /// Calculates the span of a particular track on the given layer.
    pub fn track_span(&self, layer: usize, track: i64) -> Span {
        let slice = self.stack.slice(self.start..self.end);
        let tracks = slice.tracks(layer);

        tracks.track(track)
    }

    /// The tracks on the given layer.
    pub fn tracks(&self, layer: usize) -> UniformTracks {
        self.stack.tracks(layer)
    }

    /// Returns the track grid for the given layer.
    ///
    /// Returns a tuple containing the vertical going tracks followed by the horizontal going tracks.
    /// In other words, the first element of the tuple is indexed by an x-coordinate,
    /// and the second element of the tuple is indexed by a y-coordinate.
    pub fn track_grid(&self, layer: usize) -> (UniformTracks, UniformTracks) {
        let tracks = self.tracks(layer);
        let adj_tracks = self.stack.tracks(self.grid_defining_layer(layer));

        match self.stack.layer(layer).dir().track_dir() {
            Dir::Horiz => (adj_tracks, tracks),
            Dir::Vert => (tracks, adj_tracks),
        }
    }

    /// Calculates the bounds of a particular track on the given layer.
    ///
    /// The start and end coordinates are with respect to tracks on the grid defining layer.
    pub fn track(&self, layer: usize, track: i64, start: i64, end: i64) -> Rect {
        let slice = self.stack.slice(self.start..self.end);

        // note that the grid defining layer may be outside the slice,
        // e.g. if the slice contains layers 2 through 5, the grid defining layer of 2 is 1.
        let adj_tracks = self.stack.tracks(self.grid_defining_layer(layer));

        // This allows `start` to be larger than `end`.
        let (start, end) = sorted2(start, end);

        let track = self.track_span(layer, track);
        let endcap = slice.layer(layer).endcap();
        let start = adj_tracks.track(start).center() - endcap;
        let end = adj_tracks.track(end).center() + endcap;

        Rect::from_dir_spans(
            self.stack.layer(layer).dir().track_dir(),
            Span::new(start, end),
            track,
        )
    }

    /// Returns the physical coordinates of the grid point defined by the given `track` and `cross_track`
    /// on layer `layer`, where `track` is the track on `layer` and `cross_track` is a perpendicular track.
    pub fn track_point(&self, layer: usize, track: i64, cross_track: i64) -> Point {
        let tracks = self.stack.tracks(layer);
        let cross_tracks = self.stack.tracks(self.grid_defining_layer(layer));

        let track = tracks.track(track).center();
        let cross = cross_tracks.track(cross_track).center();

        Point::from_dir_coords(self.stack.layer(layer).dir().track_dir(), cross, track)
    }

    /// Returns the physical coordinates of the grid point defined by the given `x_track` and `y_track`
    /// on layer `layer`.
    pub fn xy_track_point(&self, layer: usize, x_track: i64, y_track: i64) -> Point {
        let tracks = self.stack.tracks(layer);
        let gdl_tracks = self.stack.tracks(self.grid_defining_layer(layer));

        match self.stack.layer(layer).dir().track_dir() {
            Dir::Horiz => Point::new(
                gdl_tracks.track(x_track).center(),
                tracks.track(y_track).center(),
            ),
            Dir::Vert => Point::new(
                tracks.track(x_track).center(),
                gdl_tracks.track(y_track).center(),
            ),
        }
    }

    /// Rounds the given point to the routing grid, returning a point in track coordinates on the given layer.
    pub fn point_to_grid(
        &self,
        p: Point,
        layer: usize,
        round_x: RoundingMode,
        round_y: RoundingMode,
    ) -> Point {
        let gdl = self.grid_defining_layer(layer);
        let trk = self.stack.tracks(layer);
        let gdtrk = self.stack.tracks(gdl);
        match self.stack.layer(layer).dir().track_dir() {
            Dir::Vert => Point::new(
                trk.to_track_idx(p.x, round_x),
                gdtrk.to_track_idx(p.y, round_y),
            ),
            Dir::Horiz => Point::new(
                gdtrk.to_track_idx(p.x, round_x),
                trk.to_track_idx(p.y, round_y),
            ),
        }
    }

    /// Rounds the given point to the routing grid, returning a point in track coordinates on the given layer.
    ///
    /// Rounds in the direction that makes the rectangle smaller.
    /// If the resulting rectangle contains no track points, returns [`None`].
    pub fn shrink_to_grid(&self, rect: Rect, layer: usize) -> Option<Rect> {
        let ll = self.point_to_grid(
            rect.corner(Corner::LowerLeft),
            layer,
            RoundingMode::Up,
            RoundingMode::Up,
        );
        let ur = self.point_to_grid(
            rect.corner(Corner::UpperRight),
            layer,
            RoundingMode::Down,
            RoundingMode::Down,
        );
        Rect::from_corners_option(ll, ur)
    }

    /// Rounds the given point to the routing grid, returning a point in track coordinates on the given layer.
    ///
    /// Rounds in the direction that makes the rectangle larger.
    pub fn expand_to_grid(&self, rect: Rect, layer: usize) -> Rect {
        let ll = self.point_to_grid(
            rect.corner(Corner::LowerLeft),
            layer,
            RoundingMode::Down,
            RoundingMode::Down,
        );
        let ur = self.point_to_grid(
            rect.corner(Corner::UpperRight),
            layer,
            RoundingMode::Up,
            RoundingMode::Up,
        );
        Rect::new(ll, ur)
    }

    /// The number of PDK units in one horizontal grid coordinate on this layer.
    pub(crate) fn xpitch(&self, layer: usize) -> i64 {
        match self.stack.layer(layer).dir().track_dir() {
            Dir::Vert => self.stack.layer(layer).pitch(),
            Dir::Horiz => self.stack.layer(self.grid_defining_layer(layer)).pitch(),
        }
    }

    /// The number of PDK units in one vertical grid coordinate on this layer.
    pub(crate) fn ypitch(&self, layer: usize) -> i64 {
        match self.stack.layer(layer).dir().track_dir() {
            Dir::Horiz => self.stack.layer(layer).pitch(),
            Dir::Vert => self.stack.layer(self.grid_defining_layer(layer)).pitch(),
        }
    }

    /// The number of PDK units in one grid coordinate in the given direction on this layer.
    #[allow(dead_code)]
    pub(crate) fn dir_pitch(&self, layer: usize, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.xpitch(layer),
            Dir::Vert => self.ypitch(layer),
        }
    }
}

/// A struct that draws all tracks on a routing grid for debugging or visualization.
pub struct DebugRoutingGrid<L> {
    grid: RoutingGrid<L>,
    nx: i64,
    ny: i64,
}

impl<L> DebugRoutingGrid<L> {
    /// Create a new debugging grid from the given routing grid.
    pub fn new(grid: RoutingGrid<L>, nx: i64, ny: i64) -> Self {
        Self { grid, nx, ny }
    }

    /// The number of repeated LCM units in the given direction.
    fn ndir(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.nx,
            Dir::Vert => self.ny,
        }
    }

    /// The coordinate of the last track on the given layer.
    fn max_coord(&self, layer: usize) -> i64
    where
        L: AtollLayer,
    {
        let slice = self.grid.stack.slice(self.grid.start..self.grid.end);
        let layer = slice.layer(layer);
        let lcm = slice.lcm_unit(!layer.dir().track_dir());
        let pitch = layer.pitch();
        let (units, rem) = (lcm / pitch, lcm % pitch);
        assert_eq!(
            rem, 0,
            "expected lcm ({lcm}) to be an integer multiple of the layer pitch ({pitch})"
        );
        units * self.ndir(!layer.dir().track_dir())
    }
}

impl<L: AsRef<LayerId> + AtollLayer, PDK: Pdk> Draw<PDK> for DebugRoutingGrid<L> {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> substrate::error::Result<()> {
        for layer in self.grid.start..self.grid.end {
            for track in 0..self.max_coord(layer) {
                let cross_layer = self.grid.grid_defining_layer(layer);
                let r = self
                    .grid
                    .track(layer, track, 0, self.max_coord(cross_layer));
                let shape = Shape::new(self.grid.stack.layer(layer), r);
                recv.draw(shape)?;
            }
        }
        Ok(())
    }
}

fn sorted2<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// A fixed-size routing grid.
#[derive(Clone, Debug)]
pub struct RoutingState<L> {
    pub(crate) grid: RoutingGrid<L>,
    pub(crate) layers: Vec<Grid<PointState>>,
    pub(crate) roots: HashMap<NetId, NetId>,
}

impl<L> Index<GridCoord> for RoutingState<L> {
    type Output = PointState;

    fn index(&self, index: GridCoord) -> &Self::Output {
        &self.layers[index.layer][(index.x, index.y)]
    }
}

impl<L> IndexMut<GridCoord> for RoutingState<L> {
    fn index_mut(&mut self, index: GridCoord) -> &mut Self::Output {
        &mut self.layers[index.layer][(index.x, index.y)]
    }
}

impl<L> RoutingState<L> {
    /// Returns a reference to the routing grid.
    pub fn grid(&self) -> &RoutingGrid<L> {
        &self.grid
    }
    /// Returns a reference to a layer's state.
    pub fn layer(&self, layer: usize) -> &Grid<PointState> {
        &self.layers[layer]
    }

    /// Returns a mutable reference to a layer's state.
    pub fn layer_mut(&mut self, layer: usize) -> &mut Grid<PointState> {
        &mut self.layers[layer]
    }
}

pub(crate) struct InterlayerTransition {
    pub(crate) to: GridCoord,
    pub(crate) requires: Option<GridCoord>,
}

impl<L: AtollLayer + Clone> RoutingState<L> {
    /// Creates a new [`RoutingState`].
    pub fn new(stack: LayerStack<L>, top: usize, nx: i64, ny: i64) -> Self {
        assert!(nx >= 0);
        assert!(ny >= 0);

        let grid = RoutingGrid::new(stack.clone(), 0..top + 1);
        let slice = stack.slice(0..top + 1);
        let mut layers = Vec::new();
        for i in 0..=top {
            let layer = stack.layer(i);
            let dim = slice.lcm_unit(!layer.dir().track_dir());
            let num_tracks = dim / layer.pitch();
            let perp_layer = stack.layer(grid.grid_defining_layer(i));
            let perp_dim = slice.lcm_unit(!perp_layer.dir().track_dir());
            let perp_tracks = perp_dim / perp_layer.pitch();
            let grid = if layer.dir().track_dir() == Dir::Vert {
                Grid::init(
                    (nx * num_tracks) as usize,
                    (ny * perp_tracks) as usize,
                    PointState::Available,
                )
            } else {
                Grid::init(
                    (nx * perp_tracks) as usize,
                    (ny * num_tracks) as usize,
                    PointState::Available,
                )
            };
            layers.push(grid);
        }
        Self {
            grid,
            layers,
            roots: HashMap::new(),
        }
    }

    /// Finds a single grid coordinate belonging to the given net.
    ///
    /// Searches the topmost layer first, starting from the lower left.
    /// Returns the first grid point with a matching net ID, or [`None`]
    /// if no grid point has the given net ID.
    pub fn find(&self, net: NetId) -> Option<GridCoord> {
        for (i, layer) in self.layers.iter().enumerate().rev() {
            let (nx, ny) = layer.size();
            for x in 0..nx {
                for y in 0..ny {
                    if layer[(x, y)].is_routed_for_net(net) {
                        return Some(GridCoord { layer: i, x, y });
                    }
                }
            }
        }
        None
    }

    /// Finds all grid coordinates belonging to the given net.
    ///
    /// Searches the topmost layer first, starting from the lower left.
    /// Returns the first grid point with a matching net ID, or [`None`]
    /// if no grid point has the given net ID.
    pub fn find_all(&self, net: NetId) -> Vec<GridCoord> {
        let mut coords = Vec::new();
        for (i, layer) in self.layers.iter().enumerate().rev() {
            let (nx, ny) = layer.size();
            for x in 0..nx {
                for y in 0..ny {
                    if layer[(x, y)].is_routed_for_net(net) {
                        coords.push(GridCoord { layer: i, x, y });
                    }
                }
            }
        }
        coords
    }

    /// Relabels a net with a new ID, usually after being connected to another net.
    pub(crate) fn relabel_net(&mut self, old: NetId, new: NetId) {
        for layer in self.layers.iter_mut().rev() {
            let (nx, ny) = layer.size();
            for x in 0..nx {
                for y in 0..ny {
                    if let PointState::Routed { net, has_via } = layer[(x, y)] {
                        if net == old {
                            layer[(x, y)] = PointState::Routed { net: new, has_via };
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn is_routed_for_net(&self, coord: GridCoord, net: NetId) -> bool {
        if let PointState::Routed { net: grid_net, .. } = self[coord] {
            self.roots[&net] == self.roots[&grid_net]
        } else {
            false
        }
    }

    #[inline]
    pub(crate) fn is_available_for_net(&self, coord: GridCoord, net: NetId) -> bool {
        match self[coord] {
            PointState::Routed { net: grid_net, .. } => self.roots[&grid_net] == self.roots[&net],
            PointState::Available => true,
            PointState::Blocked { .. } => false,
            PointState::Reserved { .. } => false,
        }
    }

    #[inline]
    pub(crate) fn has_via(&self, coord: GridCoord) -> bool {
        match self[coord] {
            PointState::Routed { has_via, .. } => has_via,
            PointState::Blocked { has_via, .. } => has_via,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_available_or_reserved_for_net(&self, coord: GridCoord, net: NetId) -> bool {
        match self[coord] {
            PointState::Routed { .. } => false,
            PointState::Available => true,
            PointState::Blocked { .. } => false,
            PointState::Reserved { net: grid_net } => self.roots[&net] == self.roots[&grid_net],
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn is_available(&self, coord: GridCoord) -> bool {
        match self[coord] {
            PointState::Routed { .. } => false,
            PointState::Available => true,
            PointState::Blocked { .. } => false,
            PointState::Reserved { .. } => false,
        }
    }

    #[inline]
    pub(crate) fn in_bounds(&self, coord: GridCoord) -> bool {
        let (nx, ny) = self.layer(coord.layer).size();
        // Edge of tile is not in bounds
        coord.x > 0 && coord.y > 0 && coord.x < nx && coord.y < ny
    }

    #[allow(dead_code)]
    pub(crate) fn forms_new_connection_for_net(&self, coord: GridCoord, net: NetId) -> bool {
        if let PointState::Routed { net: grid_net, .. } = self[coord] {
            self.roots[&net] == self.roots[&grid_net] && grid_net != net
        } else {
            false
        }
    }

    pub(crate) fn are_routed_for_same_net(&self, src: GridCoord, dst: GridCoord) -> bool {
        if let (PointState::Routed { net: src_net, .. }, PointState::Routed { net: dst_net, .. }) =
            (self[src], self[dst])
        {
            src_net == dst_net
        } else {
            false
        }
    }

    /// The cost to traverse from `src` to `dst`.
    ///
    /// Important: the two coordinates must be adjacent.
    fn cost(&self, src: GridCoord, dst: GridCoord, net: NetId) -> usize {
        let manhattan_dist = (num::abs(src.x as i64 - dst.x as i64)
            + num::abs(src.y as i64 - dst.y as i64)) as usize;
        if self.is_routed_for_net(src, net) && self.is_routed_for_net(dst, net) {
            0
        } else if src.layer != dst.layer {
            6
        } else {
            4 * manhattan_dist
        }
    }

    fn successors_vert(&self, node: RoutingNode, net: NetId, out: &mut Vec<(RoutingNode, usize)>) {
        let RoutingNode { coord, has_via } = node;

        let layer = self.layer(coord.layer);
        let via_spacing = self.grid.slice().layer(coord.layer).via_spacing();
        let jump = if has_via { via_spacing } else { 1 };

        // Prevents routing on edge of tile.
        if coord.y > jump {
            let next = GridCoord {
                y: coord.y - jump,
                ..coord
            };
            if self.is_available_for_net(next, net) {
                out.push((
                    RoutingNode {
                        coord: next,
                        has_via: self.has_via(next),
                    },
                    self.cost(coord, next, net),
                ));
            }
        }
        if coord.y < layer.size().1 - jump {
            let next = GridCoord {
                y: coord.y + jump,
                ..coord
            };
            if self.is_available_for_net(next, net) {
                out.push((
                    RoutingNode {
                        coord: next,
                        has_via: self.has_via(next),
                    },
                    self.cost(coord, next, net),
                ));
            }
        }
    }

    fn successors_horiz(&self, node: RoutingNode, net: NetId, out: &mut Vec<(RoutingNode, usize)>) {
        let RoutingNode { coord, has_via } = node;
        let layer = self.layer(coord.layer);
        let via_spacing = self.grid.slice().layer(coord.layer).via_spacing();
        let jump = if has_via { via_spacing } else { 1 };

        // Prevents routing on edge of tile.
        if coord.x > jump {
            let next = GridCoord {
                x: coord.x - jump,
                ..coord
            };
            if self.is_available_for_net(next, net) {
                out.push((
                    RoutingNode {
                        coord: next,
                        has_via: self.has_via(next),
                    },
                    self.cost(coord, next, net),
                ));
            }
        }
        if coord.x < layer.size().0 - jump {
            let next = GridCoord {
                x: coord.x + jump,
                ..coord
            };
            if self.is_available_for_net(next, net) {
                out.push((
                    RoutingNode {
                        coord: next,
                        has_via: self.has_via(next),
                    },
                    self.cost(coord, next, net),
                ));
            }
        }
    }

    pub(crate) fn ilt_down(&self, coord: GridCoord) -> Option<InterlayerTransition> {
        let routing_dir = self.grid.slice().layer(coord.layer).dir();
        if coord.layer == 0 {
            return None;
        }
        let next_layer = coord.layer - 1;
        let interp_dir = !routing_dir.track_dir();
        let pp = self.grid_to_rel_physical(coord);
        let nearest_grid =
            self.grid
                .point_to_grid(pp, next_layer, RoundingMode::Nearest, RoundingMode::Nearest);
        assert_eq!(
            nearest_grid.coord(!interp_dir),
            coord.coord(!interp_dir) as i64
        );
        assert!(nearest_grid.x >= 0);
        assert!(nearest_grid.y >= 0);

        let lower_tracks = self.grid.tracks(self.grid.grid_defining_layer(next_layer));
        let upper_tracks = self.grid.tracks(coord.layer);
        let curr = upper_tracks.track(coord.coord(interp_dir) as i64).center();
        let nearest = lower_tracks.track(nearest_grid.coord(interp_dir)).center();

        let next = GridCoord {
            layer: next_layer,
            x: nearest_grid.x as usize,
            y: nearest_grid.y as usize,
        };

        match curr.cmp(&nearest) {
            Ordering::Less => {
                // need to check if coordinate-1 is in bounds
                let ck = next.with_coord(interp_dir, next.coord(interp_dir) - 1);
                if !self.in_bounds(ck) {
                    return None;
                }
                Some(InterlayerTransition {
                    to: next,
                    requires: Some(ck),
                })
            }
            Ordering::Equal => Some(InterlayerTransition {
                to: next,
                requires: None,
            }),
            Ordering::Greater => {
                if coord.coord(interp_dir) > 0 {
                    let ck = next.with_coord(interp_dir, next.coord(interp_dir) + 1);
                    Some(InterlayerTransition {
                        to: next,
                        requires: Some(ck),
                    })
                } else {
                    None
                }
            }
        }
    }
    pub(crate) fn ilt_up(&self, coord: GridCoord) -> Option<InterlayerTransition> {
        let routing_dir = self.grid.slice().layer(coord.layer).dir();
        let next_layer = coord.layer + 1;
        if next_layer >= self.grid.end {
            return None;
        }
        let interp_dir = routing_dir.track_dir();
        let pp = self.grid_to_rel_physical(coord);
        let nearest_grid =
            self.grid
                .point_to_grid(pp, next_layer, RoundingMode::Nearest, RoundingMode::Nearest);
        assert_eq!(
            nearest_grid.coord(!interp_dir),
            coord.coord(!interp_dir) as i64
        );
        assert!(nearest_grid.x >= 0);
        assert!(nearest_grid.y >= 0);

        let lower_tracks = self.grid.tracks(self.grid.grid_defining_layer(coord.layer));
        let upper_tracks = self.grid.tracks(next_layer);
        let curr = lower_tracks.track(coord.coord(interp_dir) as i64).center();
        let nearest = upper_tracks.track(nearest_grid.coord(interp_dir)).center();

        let next = GridCoord {
            layer: next_layer,
            x: nearest_grid.x as usize,
            y: nearest_grid.y as usize,
        };

        match curr.cmp(&nearest) {
            Ordering::Less => {
                // need to check if coordinate+1 is in bounds and on opposite side of next
                let ck = coord.with_coord(interp_dir, coord.coord(interp_dir) + 1);
                let ck_coord = lower_tracks.track(ck.coord(interp_dir) as i64).center();
                if ck_coord <= nearest || !self.in_bounds(ck) {
                    return None;
                }
                Some(InterlayerTransition {
                    to: next,
                    requires: Some(ck),
                })
            }
            Ordering::Equal => Some(InterlayerTransition {
                to: next,
                requires: None,
            }),
            Ordering::Greater => {
                if coord.coord(interp_dir) > 0 {
                    let ck = coord.with_coord(interp_dir, coord.coord(interp_dir) - 1);
                    let ck_coord = lower_tracks.track(ck.coord(interp_dir) as i64).center();
                    if ck_coord >= nearest {
                        return None;
                    }
                    Some(InterlayerTransition {
                        to: next,
                        requires: Some(ck),
                    })
                } else {
                    None
                }
            }
        }
    }

    /// The set of points reachable from `coord`.
    ///
    /// M0 <-> M1 vias are always easy, since both layers have the same grid.
    ///
    /// MN <-> M(N+1) vias require interpolation.
    /// We first identify the coordinate that stays constant in the transition.
    /// If MN has horizontal going tracks, the y-coordinate is constant.
    /// If MN has vertical going tracks, the x-coordinate is constant.
    /// The non-constant coordinate must be interpolated.
    ///
    /// Interpolation is done as follows:
    /// * We translate the track coordinate to a physical coordinate, and retrieve the coordinate in the interpolated direction
    /// * We find two track coordinates on M(N+1): one by rounding up; the other by rounding down.
    /// * If the two coordinates are equal, we report a successor to that grid point.
    /// * Otherwise, we report a successor to the track with a closer physical coordinate, assuming
    /// the farther coordinate is unobstructed.
    pub fn successors(
        &self,
        node: RoutingNode,
        path: &[RoutingNode],
        net: NetId,
    ) -> Vec<(RoutingNode, usize)> {
        let RoutingNode { coord, .. } = node;
        let routing_dir = self.grid.slice().layer(coord.layer).dir();
        let mut successors = Vec::new();

        match routing_dir {
            RoutingDir::Vert => {
                self.successors_vert(node, net, &mut successors);
            }
            RoutingDir::Horiz => {
                self.successors_horiz(node, net, &mut successors);
            }
            RoutingDir::Any { .. } => {
                self.successors_vert(node, net, &mut successors);
                self.successors_horiz(node, net, &mut successors);
            }
        }

        for ilt in [self.ilt_up(coord), self.ilt_down(coord)]
            .into_iter()
            .flatten()
        {
            if ilt.to.layer != coord.layer {
                let mut has_via = false;
                for top in [ilt.to, coord] {
                    let track_dir = self.grid.slice().layer(top.layer).dir().track_dir();
                    let via_spacing = self.grid.slice().layer(top.layer).via_spacing();
                    let routing_coord = top.coord(track_dir);
                    for i in (routing_coord + 1)
                        .checked_sub(via_spacing)
                        .unwrap_or_default()
                        ..routing_coord + via_spacing
                    {
                        let check_coord = top.with_coord(track_dir, i);
                        if i != routing_coord
                            && self.in_bounds(check_coord)
                            && self.has_via(check_coord)
                        {
                            has_via = true;
                        }
                    }

                    for nodes in path.windows(2) {
                        if nodes[0].coord.layer != nodes[1].coord.layer {
                            for node in nodes {
                                if node.coord != top
                                    && node.coord.layer == top.layer
                                    && node.coord.coord(!track_dir) == top.coord(!track_dir)
                                    && node.coord.coord(track_dir)
                                        >= (routing_coord + 1)
                                            .checked_sub(via_spacing)
                                            .unwrap_or_default()
                                    && node.coord.coord(track_dir) < routing_coord + via_spacing
                                {
                                    has_via = true;
                                }
                            }
                        }
                    }
                }
                if has_via {
                    continue;
                }
            }
            if self.is_available_for_net(ilt.to, net)
                && ilt
                    .requires
                    .map(|n| self.is_available_or_reserved_for_net(n, net))
                    .unwrap_or(true)
            {
                successors.push((
                    RoutingNode {
                        coord: ilt.to,
                        has_via: ilt.to.layer != coord.layer || self.has_via(ilt.to),
                    },
                    self.cost(coord, ilt.to, net),
                ));
            }
        }

        successors
    }

    /// Converts the given grid point to physical coordinates.
    ///
    /// Assumes that (0,0) in grid coordinates is the same as (0,0) in track coordinates.
    /// In other words, this assumes that grid and track coordinates are the same;
    /// there is not shift between grid and track coordinates.
    pub fn grid_to_rel_physical(&self, coord: GridCoord) -> Point {
        self.grid
            .xy_track_point(coord.layer, coord.x as i64, coord.y as i64)
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::*;

    fn layer_stack() -> LayerStack<AbstractLayer> {
        LayerStack {
            layers: vec![
                AbstractLayer {
                    dir: RoutingDir::Horiz,
                    line: 100,
                    space: 200,
                    offset: TrackOffset::None,
                    endcap: 20,
                    via_spacing: 1,
                    strap_via_spacing: 1,
                },
                AbstractLayer {
                    dir: RoutingDir::Vert,
                    line: 120,
                    space: 200,
                    offset: TrackOffset::None,
                    endcap: 20,
                    via_spacing: 1,
                    strap_via_spacing: 1,
                },
                AbstractLayer {
                    dir: RoutingDir::Horiz,
                    line: 200,
                    space: 400,
                    offset: TrackOffset::None,
                    endcap: 40,
                    via_spacing: 1,
                    strap_via_spacing: 1,
                },
                AbstractLayer {
                    dir: RoutingDir::Vert,
                    line: 200,
                    space: 400,
                    offset: TrackOffset::None,
                    endcap: 50,
                    via_spacing: 1,
                    strap_via_spacing: 1,
                },
            ],
            offset_x: 0,
            offset_y: 0,
        }
    }

    #[test]
    fn lcm_units() {
        let layers = layer_stack();
        let slice = layers.all();
        assert_eq!(slice.lcm_unit(Dir::Horiz), 4_800);
        assert_eq!(slice.lcm_unit(Dir::Vert), 600);
        assert!(layers.is_valid());
    }
}
