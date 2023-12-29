//! Uniform routing grids and layer stacks.
use crate::{PointState, RoutingDir};
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::ops::{Deref, DerefMut, Range};
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

/// An ATOLL-compatible routing layer.
pub trait AtollLayer {
    /// The preferred routing direction.
    fn dir(&self) -> RoutingDir;
    /// The line width on this layer.
    fn line(&self) -> i64;
    /// The space between adjacent tracks on this layer.
    fn space(&self) -> i64;
    /// An offset that shifts the first track of the layer.
    fn offset(&self) -> i64;
    /// The amount by which this layer should extend beyond the center line of a track on the this layer's grid defining layer.
    fn endcap(&self) -> i64 {
        0
    }

    /// The line + space of this layer.
    ///
    /// The default implementation should not generally be overridden.
    fn pitch(&self) -> i64 {
        self.line() + self.space()
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
    /// How much to shift the first track of the layer.
    pub offset: i64,
    /// How far to extend a track beyond the center-to-center intersection point with a track on the layer below.
    pub endcap: i64,
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
    pub fn tracks(&self, global_ofs: i64) -> UniformTracks {
        UniformTracks::with_offset(self.line, self.space, self.offset + global_ofs)
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

    fn offset(&self) -> i64 {
        self.offset
    }

    fn endcap(&self) -> i64 {
        self.endcap
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

    fn offset(&self) -> i64 {
        self.inner.offset()
    }

    fn endcap(&self) -> i64 {
        self.inner.endcap()
    }

    fn pitch(&self) -> i64 {
        self.inner.pitch()
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
        UniformTracks::with_offset(layer.line(), layer.space(), layer.offset() + ofs)
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
}

/// A fixed-size routing grid.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RoutingGrid<L> {
    pub(crate) stack: LayerStack<L>,
    start: usize,
    end: usize,
    nx: i64,
    ny: i64,
}

impl<L> RoutingGrid<L> {
    /// Creates a new routing grid with the given properties.
    pub fn new(stack: LayerStack<L>, layers: Range<usize>, nx: i64, ny: i64) -> Self {
        Self {
            stack,
            start: layers.start,
            end: layers.end,
            nx,
            ny,
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

    pub fn track_point(&self, layer: usize, track: i64, cross_track: i64) -> Point {
        let tracks = self.stack.tracks(layer);
        let cross_tracks = self.stack.tracks(self.grid_defining_layer(layer));

        let track = tracks.track(track).center();
        let cross = cross_tracks.track(cross_track).center();

        Point::from_dir_coords(self.stack.layer(layer).dir().track_dir(), cross, track)
    }

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

    /// The coordinate of the first track on the given layer.
    fn min_coord(&self, _layer: usize) -> i64 {
        // the first track is always labeled 0
        0
    }

    /// The coordinate of the last track on the given layer.
    fn max_coord(&self, layer: usize) -> i64 {
        let slice = self.stack.slice(self.start..self.end);
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

    /// The number of repeated LCM units in the given direction.
    fn ndir(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.nx,
            Dir::Vert => self.ny,
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
    pub(crate) fn dir_pitch(&self, layer: usize, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.xpitch(layer),
            Dir::Vert => self.ypitch(layer),
        }
    }
}

/// A struct that draws all tracks on a routing grid for debugging or visualization.
pub struct DebugRoutingGrid<L>(RoutingGrid<L>);

impl<L> DebugRoutingGrid<L> {
    /// Create a new debugging grid from the given routing grid.
    pub fn new(grid: RoutingGrid<L>) -> Self {
        Self(grid)
    }
}

impl<L> Deref for DebugRoutingGrid<L> {
    type Target = RoutingGrid<L>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L> DerefMut for DebugRoutingGrid<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<L: AsRef<LayerId> + AtollLayer, PDK: Pdk> Draw<PDK> for DebugRoutingGrid<L> {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> substrate::error::Result<()> {
        for layer in self.start..self.end {
            for track in self.min_coord(layer)..self.max_coord(layer) {
                let cross_layer = self.grid_defining_layer(layer);
                let r = self.track(
                    layer,
                    track,
                    self.min_coord(cross_layer),
                    self.max_coord(cross_layer),
                );
                let shape = Shape::new(self.stack.layer(layer), r);
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
#[derive(Clone)]
pub struct RoutingState<L> {
    pub(crate) grid: RoutingGrid<L>,
    pub(crate) layers: Vec<Grid<PointState>>,
}

impl<L> RoutingState<L> {
    pub fn layer(&self, layer: usize) -> &Grid<PointState> {
        &self.layers[layer]
    }

    pub fn layer_mut(&mut self, layer: usize) -> &mut Grid<PointState> {
        &mut self.layers[layer]
    }
}

impl<L: AtollLayer + Clone> RoutingState<L> {
    pub fn new(stack: LayerStack<L>, top: usize, nx: i64, ny: i64) -> Self {
        assert!(nx >= 0);
        assert!(ny >= 0);

        let grid = RoutingGrid::new(stack.clone(), 0..top + 1, nx, ny);
        let slice = stack.slice(0..top + 1);
        let mut layers = Vec::new();
        for i in 0..=top {
            let layer = stack.layer(i);
            let dim = slice.lcm_unit(!layer.dir().track_dir());
            let num_tracks = dim / layer.pitch();
            println!(
                "dim = {dim}, layer pitch = {}, num tracks = {}, nx = {}, ny = {}",
                layer.pitch(),
                num_tracks,
                nx,
                ny
            );
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
        Self { grid, layers }
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::*;

    #[test]
    fn lcm_units() {
        let layers = LayerStack {
            layers: vec![
                AbstractLayer {
                    dir: RoutingDir::Horiz,
                    line: 100,
                    space: 200,
                    offset: 0,
                    endcap: 20,
                },
                AbstractLayer {
                    dir: RoutingDir::Vert,
                    line: 120,
                    space: 200,
                    offset: 0,
                    endcap: 20,
                },
                AbstractLayer {
                    dir: RoutingDir::Horiz,
                    line: 200,
                    space: 400,
                    offset: 0,
                    endcap: 40,
                },
                AbstractLayer {
                    dir: RoutingDir::Vert,
                    line: 200,
                    space: 400,
                    offset: 0,
                    endcap: 50,
                },
            ],
            offset_x: 0,
            offset_y: 0,
        };

        let slice = layers.all();
        assert_eq!(slice.lcm_unit(Dir::Horiz), 4_800);
        assert_eq!(slice.lcm_unit(Dir::Vert), 600);
        assert!(layers.is_valid());
    }
}
