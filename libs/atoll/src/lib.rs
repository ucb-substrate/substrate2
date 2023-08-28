//! Atoll: Automatic transformation of logical layout.
//!
//! Atoll projects are made of one or more **blocks**.
//! Each block is a compact, rectangular grid of devices.
//! Each block in turn is composed of a set of tiles drawn from a TileSet.
//! TileSets provide a tile generator for each tile archetype.
//!
//! The set of tile archetypes is given by the Cartesian product
//! of [`Col`] and [`Row`].
//!
//! A tile generator takes tile configuration info and produces
//! a tile of its archetype.
//!
//! # Grid structure
//!
//! Inter-tile and inter-block routes are drawn on designated routing layers.
//! Intra-tile routing can be done on any layer; Atoll does not interface with such routing.
//!
//! Atoll assumes that you have a set of metal layers `M0, M1, M2, ...`, where each metal
//! layer can be connected to the layer immediately above or below.
//! Atoll also assumes that each metal layer has a preferred direction, and that
//! horizontal and vertical metals alternate.
//!
//! Blocks should leave some layers available for inter-block routing.
//!
//! Suppose that `P(i)` is the pitch of the i-th routing layer.
//! TileSets must pick an integer X such that:
//! * A complete block (including intra-block routing) can be assembled from layers `M0, M1, ..., MX`
//! * In particular, tiles contain no routing layers above `MX`.
//! * The width of all tiles is an integer multiple of `LCM { P(0), P(2), ... }`,
//!   assuming `M0` is vertical.
//! * The height of all tiles is an integer multiple of `LCM { P(1), P(3), ... }`,
//!   assuming `M0` is vertical (and therefore that `M1` is horizontal).
//!
//! All routing tracks must be fully internal to each tile.
//! The line and space widths must each be even integers, so that the center
//! of any track or space is also an integer.
//!
//! When the ratio `P(L+2) / P(L)` is not an integer, Atoll's routing algorithms assume
//! that if track `T` on layer `L+2` lies strictly between tracks `U1` and `U2` on layer `L`,
//! and track `T` makes a connection to track `V` on layer `L+1`, then the grid points
//! `(V, U1)` and `(V, U2)` must be left unused or must be connected to the same net as `(T, V)`.
//!
//! ## Track numbering
//!
//! Track coordinates have the form `(layer, x, y)`.
//! Each track coordinate references an intersection point between a track
//! on `layer` and a track on `layer + 1`.
//! If `layer` runs horizontally, `x` indexes the (vertical) tracks on `layer + 1`
//! and `y` indexes the horizontal tracks on `layer`.
//! The origin is the lower-left corner of the tile.
//!
//! # Tiles
//!
//! Each tile is conceptually composed of two slices: a device slice, and a routing slice.
//!
//! ## Device slices
//!
//! The device slice encompasses structures placed on base layers.
//! Typical device slices may produce:
//! * Transistors
//! * Resistors
//! * Capacitors
//! * Taps
//!
//! The device slice may perform some intra-device routing.
//! The device slice is responsible for connecting signals that must be exposed
//! to tracks in the routing slice.
//!
//! ## Routing slices
//!
//! Routing slices are responsible for:
//! * Bringing intra-device signals that need to be exposed to an edge track.
//! * Selecting routing paths for signals that go through a tile.
//!
//! A track is considered an edge track if at least one adjacent track on the
//! same layer falls outside the tile's boundary.
//!
//! # Routing
//!
//! There are three phases of routing:
//! * Global routing
//! * Intra-tile routing
//! * Inter-tile routing
//! * Inter-block routing
//!
//! Global routing assigns nets/devices to blocks and creates cut-through routes.
//!
//! Intra-tile routing brings all exposed device slice signals to
//! one or more edge tracks within the tile.
//!
//! Inter-tile routing connects signals within a block.
//! The inter-tile router accepts a list of signals that must be exposed for
//! inter-block routing, along with an optional preferred edge (top, bottom, left, or right)
//! for where those signals should be exposed.
//!
//! Each tile communicates a list of obstructed track coordinates to the inter-tile router.
//!
//! Inter-block routing connects signals across blocks.
//!
//! ## Cut-through routes
//!
//! It is sometimes necessary to route a signal through a tile on layers
//! that the tile itself may be using. To do this, the global router can
//! instruct the inter-tile router to create a cut-through route on a
//! specific layer.
//!
//! The inter-tile router is then responsible for routing a track on the given
//! layer from one side of the block to a track on the same layer exiting on the opposite
//! side of the block. Note that the entry and exit track indices need not be the same.
//!
//! For example, a cut-through route may enter on track 1 on the left side of a block
//! and exit on track 2 on the same layer on the right side of the block.
//!
//! Cut-through routes can be created for signals that are internally used by a block.
//! These routes enter on one side of a block, may branch to zero or more devices within the block,
//! and exit on the same layer on the other side of the block.
//!
//! ## Filler cells
//!
//! Filler cells (e.g. tap and decap cells) must have a width
//! equal to the GCD of the widths of all device cells.
//! Note that this GCD must be an integer multiple of the LCMs of
//! track pitches over all vertical running routing layers.
//! A similar requirement holds for filler cell height.
//!
//! # Power strapping
//!
//! Atoll can be configured to insert power straps on tracks
//! available after routing.
//!
//! Nonuniform power straps are only supported during inter-block routing,
//! and only for layers above `MX`.
//!
//! The inter-tile router supports 3 power strap modes:
//! * Straps first: gives priority to straps, adding obstructions to the routing grid
//!   where a signal track overlaps or is otherwise too close to a power strap.
//! * Grid adjust: makes the signal routing grid non-uniform so that signal tracks
//!   do not collide with power straps.
//! * Straps last: performs inter-tile routing first, then adds straps wherever
//!   possible, without disturbing routed signals.
//!

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use substrate::geometry::prelude::{Dir, Point};
use substrate::layout::tracks::{Tracks, EnumeratedTracks};

pub type NetId = usize;

pub enum TrackConfig {
    ConnectedTo(NetId),
    Obstructed(Vec<Range<i64>>),
    Available,
}

pub enum Col {
    Device,
    Routing { tap: bool },
}

pub enum Row {
    Device,
    Routing,
}

pub struct Atoll {
    pub pmos: Vec<Row>,
    pub nmos: Vec<Row>,
    pub cols: Vec<Col>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LayerId(usize);

pub struct Coordinate {
    /// The lower metal layer.
    pub layer: LayerId,
    /// The x-coordinate.
    ///
    /// Indexes the vertical-traveling tracks.
    pub x: i64,
    /// The y-coordinate.
    ///
    /// Indexes the horizontal-traveling tracks.
    pub y: i64,
}

/// A grid defined by two adjacent routing layers.
pub struct Grid<H, V> {
    /// The lower metal layer.
    pub layer: LayerId,
    /// The direction of the tracks on the lower layer.
    pub dir: Dir,

    pub htracks: H,
    pub vtracks: V,
}

pub trait Xy {
    fn xy(&self) -> (i64, i64);
}

impl<T> Xy for &T where T: Xy {
    fn xy(&self) -> (i64, i64) {
        (*self).xy()
    }
}

impl Xy for Coordinate {
    fn xy(&self) -> (i64, i64) {
        (self.x, self.y)
    }
}

impl Xy for Point {
    fn xy(&self) -> (i64, i64) {
        (self.x, self.y)
    }
}

impl Xy for (i64, i64) {
    fn xy(&self) -> (i64, i64) {
        *self
    }
}

impl<H, V> Grid<H, V>
where H: Tracks,
      V: Tracks,
{
    /// The center point of the corresponding grid tracks.
    pub fn point(&self, pt: impl Xy) -> Point {
        let (x, y) = pt.xy();
        let vtrack = self.vtracks.track(x);
        let htrack = self.htracks.track(y);
        Point::new(vtrack.center(), htrack.center())
    }
}

pub struct GridState<S = PointState> {
    pub grid: Grid<EnumeratedTracks, EnumeratedTracks>,
    pub states: grid::Grid<S>,
}

impl<S> GridState<S> {
    pub fn rows(&self) -> i64 {
        i64::try_from(self.states.rows()).unwrap()
    }
    pub fn cols(&self) -> i64 {
        i64::try_from(self.states.cols()).unwrap()
    }
    pub fn get(&self, x: i64, y: i64) -> &S {
        self.states.get(x as usize, y as usize).unwrap()
    }
}

pub struct GridStack<S = PointState> {
    pub layers: HashMap<LayerId, GridState<S>>,
}

/// The state of a point on a routing grid.
pub enum PointState {
    Available,
    Obstructed,
    Routed(NetId),
}

impl PointState {
    pub fn is_available_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => true,
            Self::Routed(n) => *n == net,
            Self::Obstructed => false,
        }
    }
}

pub struct GridCell {
    tracks: HashMap<LayerId, Box<dyn Tracks>>
}

pub struct TwoLayerTrackMap {
    /// Maps a top-track index to the set of bottom-track indices blocked.
    pub interferes: HashMap<i64, HashSet<i64>>,
    /// Maps a bottom-track index to an adjacent top-track index.
    ///
    /// We assume that the pitch of the top layer is greater than
    /// or equal to the pitch of the lower layer.
    ///
    /// The transition `L -> H` is available if and only if
    /// H is in `adj(L)` and for each track T in `interferes(H)`,
    /// T is available or routed on the same net.
    pub adjacent: HashMap<i64, HashSet<i64>>,
}

impl TwoLayerTrackMap {
    /// For now, we assume that if top track T interferes with bottom track B,
    /// B is adjacent to T.
    pub fn insert(&mut self, tt: i64, interferes: impl Into<HashSet<i64>>) {
        let interferes = interferes.into();
        let entry = self.interferes.entry(tt).or_default();
        entry.extend(interferes.iter().copied());

        for &intf in interferes.iter() {
            let entry = self.adjacent.entry(intf).or_default();
            entry.insert(tt);
        }

        todo!()
    }

    pub fn interferes(&self, tt: i64) -> impl Iterator<Item = i64> + '_ {
        self.interferes.get(&tt).unwrap().iter().copied()
    }

    pub fn adj(&self, bt: i64) -> impl Iterator<Item = i64> + '_ {
        self.adjacent.get(&bt).unwrap().iter().copied()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoutingDir {
    Vert,
    Horiz,
    Any,
    None,
}

impl RoutingDir {
    /// Whether or not this routing direction allows movement in the given direction.
    pub fn supports_dir(&self, dir: Dir) -> bool {
        match dir {
            Dir::Horiz => self.supports_horiz(),
            Dir::Vert => self.supports_vert(),
        }
    }
    /// Whether or not this routing direction allows horizontal movement.
    pub fn supports_horiz(&self) -> bool {
        matches!(*self, Self::Horiz | Self::Any)
    }
    /// Whether or not this routing direction allows vertical movement.
    pub fn supports_vert(&self) -> bool {
        matches!(*self, Self::Vert | Self::Any)
    }
}

/// The state of a single layer in a routing volume.
pub struct RoutingSlice {
    dir: RoutingDir,
    grid: GridState<PointState>,
}

impl RoutingSlice {
    pub fn is_valid_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.grid.cols()
            && pos.y >= 0 && pos.y < self.grid.rows()
    }

    pub fn is_available_for_net(&self, net: NetId, pos: Pos) -> bool {
        self.grid.get(pos.x, pos.y).is_available_for_net(net)
    }
}

pub struct InterlayerTransition {
    source: Pos,
    dst: Pos,
    requires: HashSet<Pos>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pos {
    layer: LayerId,
    x: i64,
    y: i64,
}

impl Pos {
    /// The index of the track going in the specified direction.
    pub fn track_coord(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Vert => self.x,
            Dir::Horiz => self.y,
        }
    }

    /// The index of the coordinate in the given direction.
    ///
    /// [`Dir::Horiz`] gives the x-coordinate;
    /// [`Dir::Vert`] gives the y-coordinate;
    pub fn coord(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.x,
            Dir::Vert => self.y,
        }
    }

    /// Returns a new `Pos` with the given coordinate indexing tracks going in the given direction.
    pub fn with_track_coord(&self, dir: Dir, coord: i64) -> Self {
        let Pos { layer, x, y } = *self;
        match dir {
            Dir::Vert => Self { layer, x: coord, y },
            Dir::Horiz => Self { layer, x, y: coord },
        }
    }

    /// Returns a new `Pos` with the given coordinate in the given direction.
    ///
    /// If `dir` is [`Dir::Horiz`], `coord` is taken as the new x coordinate.
    /// If `dir` is [`Dir::Vert`], `coord` is taken as the new y coordinate.
    pub fn with_coord(&self, dir: Dir, coord: i64) -> Self {
        let Pos { layer, x, y } = *self;
        match dir {
            Dir::Horiz => Self { layer, x: coord, y },
            Dir::Vert => Self { layer, x, y: coord },
        }
    }
}

/// M0: Any, M0V x M0H
/// M1: Horiz, M0V x M1H
/// M2: Vert, M2V x M1H
/// M3: Horiz, M2V x M3H
/// M4: Vert
///
/// Pos(M3, x, y) can jump to Pos(M2, x, adj(y))
/// Up and down are symmetric: if pos1 has an up
/// transition to pos2, pos2 has a down transition to pos1.
pub struct RoutingVolume {
    slices: HashMap<LayerId, RoutingSlice>,
    ilts: HashMap<Pos, HashSet<InterlayerTransition>>,
}

impl RoutingVolume {
    fn slice(&self, layer: LayerId) -> &RoutingSlice {
        &self.slices[&layer]
    }
    fn slice_mut(&mut self, layer: LayerId) -> &mut RoutingSlice {
        self.slices.get_mut(&layer).unwrap()
    }

    pub fn next(&self, pos: Pos, net: NetId) {
        let slice = self.slice(pos.layer);

        let mut successors = Vec::new();
        for dir in [Dir::Vert, Dir::Horiz] {
            if !slice.dir.supports_dir(dir) {
                continue;
            }

            let coord = pos.coord(dir);
            for ofs in [-1, 1] {
                let npos = pos.with_coord(dir, coord + ofs);
                if slice.is_valid_pos(npos) {
                    successors.push(npos);
                }
            }
        }

        if let Some(ilts) = self.ilts.get(&pos) {
        for ilt in ilts {
            if ilt.requires.iter().all(|pos| {
                self.is_available_for_net(net, *pos)
            }) {
                successors.push(ilt.dst);
            }
        }
        }
    }

    fn is_available_for_net(&self, net: NetId, pos: Pos) -> bool {
        let slice = self.slice(pos.layer);
        slice.is_available_for_net(net, pos)
    }
}
