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
#![warn(missing_docs)]

use grid::Grid;
use std::collections::{HashMap, HashSet};
use substrate::geometry::prelude::{Dir, Point};
use substrate::layout::tracks::{EnumeratedTracks, FiniteTracks, Tracks};

/// Identifies nets in a routing solver.
pub type NetId = usize;

/// A column in an Atoll grid.
pub enum Col {
    /// A device column.
    Device,
    /// A column dedicated to routing.
    Routing {
        /// Whether taps should be placed under the routing layers.
        ///
        /// Requires allocation of power/ground tracks in the routing column.
        tap: bool,
    },
}

/// A row in an Atoll grid.
pub enum Row {
    /// A device row.
    Device,
    /// A row dedicated to routing.
    Routing,
}

/// A single Atoll block.
pub struct Atoll {
    /// PMOS rows.
    pub pmos: Vec<Row>,
    /// NMOS rows.
    pub nmos: Vec<Row>,
    /// Columns (shared between PMOS and NMOS).
    pub cols: Vec<Col>,
}

/// Identifies a routing layer.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LayerId(usize);

impl From<usize> for LayerId {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// A coordinate identifying a track position in a routing volume.
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

/// A routing grid with explicitly enumerated tracks.
pub type EnumeratedRoutingGrid = RoutingGrid<EnumeratedTracks, EnumeratedTracks>;

/// A grid defined by two adjacent routing layers.
#[derive(Copy, Clone, Debug)]
pub struct RoutingGrid<H, V> {
    /// The lower metal layer.
    pub layer: LayerId,
    /// The horizontal-traveling tracks.
    pub htracks: H,
    /// The vertical-traveling tracks.
    pub vtracks: V,
}

impl<H, V> RoutingGrid<H, V>
where
    H: FiniteTracks,
    V: FiniteTracks,
{
    /// The number of rows in this routing grid.
    pub fn rows(&self) -> usize {
        let (min, max) = self.htracks.range();
        usize::try_from(max - min).unwrap()
    }

    /// The number of columns in this routing grid.
    pub fn cols(&self) -> usize {
        let (min, max) = self.vtracks.range();
        usize::try_from(max - min).unwrap()
    }
}

impl<T> RoutingGrid<T, T> {
    /// The tracks traveling in the given direction.
    pub fn tracks(&self, dir: Dir) -> &T {
        match dir {
            Dir::Horiz => &self.htracks,
            Dir::Vert => &self.vtracks,
        }
    }
}

/// A type that contains an x-y coordinate.
pub trait Xy {
    /// Returns the coordinate represented by `self`.
    fn xy(&self) -> (i64, i64);
}

impl<T: Xy> Xy for &T {
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

impl<H, V> RoutingGrid<H, V>
where
    H: Tracks,
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

/// The state of a routing grid.
#[derive(Clone, Debug)]
pub struct GridState<S = PointState> {
    /// The routing grid.
    pub grid: EnumeratedRoutingGrid,
    /// The state associated to each point in the routing grid.
    pub states: grid::Grid<S>,
}

impl<S: Clone> GridState<S> {
    /// Create a new `GridState` for the given routing grid.
    ///
    /// The given `state` is associated to each cell in the routing grid.
    pub fn new(grid: EnumeratedRoutingGrid, state: S) -> Self {
        let rows = grid.rows();
        let cols = grid.cols();
        let states = Grid::init(rows, cols, state);
        Self { grid, states }
    }
}

impl<S> GridState<S> {
    /// The number of rows in this grid.
    pub fn rows(&self) -> i64 {
        i64::try_from(self.states.rows()).unwrap()
    }

    /// The number of columns in this grid.
    pub fn cols(&self) -> i64 {
        i64::try_from(self.states.cols()).unwrap()
    }

    /// Get the state associated to the point with the given `(x, y)` coordinates.
    pub fn get(&self, x: i64, y: i64) -> &S {
        self.states.get(x as usize, y as usize).unwrap()
    }
}

/// The state of a point on a routing grid.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PointState {
    /// The grid point is available for routing.
    Available,
    /// The grid point is obstructed.
    Obstructed,
    /// The grid point is occupied by a known net.
    Routed(NetId),
}

impl PointState {
    /// Whether or not the given point can be used to route the given net.
    pub fn is_available_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => true,
            Self::Routed(n) => *n == net,
            Self::Obstructed => false,
        }
    }
}

/// Allowed track directions on a routing layer.
///
/// Adjacent routing layers must have alternating track directions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoutingDir {
    /// Layer should be used for vertical routing.
    Vert,
    /// Layer should be used for horizontal routing.
    Horiz,
    /// Layer can be used for either horizontal or vertical routing.
    Any {
        /// The direction of the tracks that form the coordinate system for this layer.
        track_dir: Dir,
    },
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
        matches!(*self, Self::Horiz | Self::Any { .. })
    }
    /// Whether or not this routing direction allows vertical movement.
    pub fn supports_vert(&self) -> bool {
        matches!(*self, Self::Vert | Self::Any { .. })
    }

    /// The direction in which tracks following this routing direction travel.
    pub fn track_dir(&self) -> Dir {
        match *self {
            Self::Vert => Dir::Vert,
            Self::Horiz => Dir::Horiz,
            Self::Any { track_dir } => track_dir,
        }
    }
}

/// The state of a single layer in a routing volume.
pub struct RoutingSlice {
    dir: RoutingDir,
    grid: GridState<PointState>,
}

impl RoutingSlice {
    /// The tracks within this routing slice.
    pub fn tracks(&self) -> &EnumeratedTracks {
        let dir = self.dir.track_dir();
        self.grid.grid.tracks(dir)
    }

    /// Create a new routing slice from the given routing grid.
    ///
    /// Tracks will travel in the given direction.
    pub fn new(dir: RoutingDir, grid: EnumeratedRoutingGrid) -> Self {
        Self {
            dir,
            grid: GridState::new(grid, PointState::Available),
        }
    }

    /// The number of rows in this slice.
    #[inline]
    pub fn rows(&self) -> i64 {
        self.grid.rows()
    }

    /// The number of columns in this slice.
    #[inline]
    pub fn cols(&self) -> i64 {
        self.grid.cols()
    }

    /// Whether or not `pos` is a valid coordinate in this slice.
    pub fn is_valid_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.grid.cols() && pos.y >= 0 && pos.y < self.grid.rows()
    }

    /// Whether or not the given net can occupy the given coordinate.
    pub fn is_available_for_net(&self, net: NetId, pos: Pos) -> bool {
        self.grid.get(pos.x, pos.y).is_available_for_net(net)
    }
}

/// A transition crossing layer boundaries.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InterlayerTransition {
    /// The source position.
    src: Pos,
    /// The destination position.
    dst: Pos,
    /// The set of points that must be available when performing this transition.
    ///
    /// Upon use of this transition, all points will be marked as occupied.
    /// `src` and `dst` are always required, and should not be included in this list.
    requires: Vec<Pos>,
}

impl InterlayerTransition {
    /// Creates a new interlayer transition from `src` to `dst`.
    ///
    /// The set of extra required points will be empty.
    pub fn new(src: Pos, dst: Pos) -> Self {
        Self {
            src,
            dst,
            requires: Default::default(),
        }
    }

    /// Swaps the source and destination of this transition.
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.src, &mut self.dst);
    }

    /// Returns a new transition with the source and destination reversed.
    #[inline]
    pub fn reversed(&self) -> Self {
        Self {
            src: self.dst,
            dst: self.src,
            requires: self.requires.clone(),
        }
    }
}

/// A position within a routing volume.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pos {
    /// The routing layer.
    layer: LayerId,
    /// The x-coordinate.
    x: i64,
    /// The y-coordinate.
    y: i64,
}

impl Pos {
    /// Create a new [`Pos`].
    pub fn new(layer: impl Into<LayerId>, x: i64, y: i64) -> Self {
        Self {
            layer: layer.into(),
            x,
            y,
        }
    }

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

/// Configuration for specifying a routing via.
pub struct ViaConfig {
    /// The width of the via in the direction orthogonal to the tracks of the higher layer.
    od_width: i64,
}

/// Configuration for specifying a routing conductor.
pub struct MetalConfig {
    /// The allowed routing directions for this layer.
    dir: RoutingDir,
    /// The tracks.
    tracks: EnumeratedTracks,
}

/// Configuration for specifying a routing volume.
pub struct RoutingVolumeConfig {
    /// The bottom routing conductor layer.
    mbot: MetalConfig,
    /// A list of via and conductor layer pairs.
    layers: Vec<(ViaConfig, MetalConfig)>,
}

impl RoutingVolume {
    /// Create a [`RoutingVolume`] from the given configuration.
    pub fn new(cfg: RoutingVolumeConfig) -> Self {
        assert!(cfg.layers.len() >= 2);

        let mut i = cfg.layers.len() - 1;
        let mut out = Self::empty();

        loop {
            let (_, topm) = &cfg.layers[i];
            let botm = if i > 0 {
                &cfg.layers[i - 1].1
            } else {
                &cfg.mbot
            };

            let dir = topm.dir.track_dir();
            let bot_dir = botm.dir.track_dir();

            assert!(
                dir == !bot_dir,
                "adjacent layers must have opposite track directions"
            );

            let (htracks, vtracks) = if dir == Dir::Horiz {
                (topm.tracks.clone(), botm.tracks.clone())
            } else {
                (botm.tracks.clone(), topm.tracks.clone())
            };

            let top_id = LayerId(i + 1);
            let bot_id = LayerId(i);
            let slice = RoutingSlice::new(
                topm.dir,
                EnumeratedRoutingGrid {
                    layer: LayerId(i + 1),
                    htracks,
                    vtracks,
                },
            );

            for x in 0..slice.cols() {
                for y in 0..slice.rows() {
                    let src = Pos {
                        layer: top_id,
                        x,
                        y,
                    };
                    let dst = Pos {
                        layer: bot_id,
                        x,
                        y,
                    };
                    let ilt = InterlayerTransition::new(src, dst);
                    out.add_ilt(ilt);
                    let ilt = InterlayerTransition::new(dst, src);
                    out.add_ilt(ilt);
                }
            }

            out.slices.insert(LayerId(i), slice);

            // special case: also handle the bottom-most layer
            if i == 0 {
                let grid = out.slice(LayerId(1)).grid.grid.clone();
                let slice = RoutingSlice::new(botm.dir, grid);

                out.slices.insert(LayerId(i), slice);
                break;
            }

            i -= 1;
        }

        for i in 1..cfg.layers.len() - 1 {
            let bot = out.slice(i - 1);
            let mid = out.slice(i);
            let top = out.slice(i + 1);
            let via = &cfg.layers[i + 1].0;
            let mut ilts = Vec::new();
            for (tt, tspan) in top.tracks().tracks().enumerate() {
                let tt = tt as i64;
                let mut targets = HashSet::new();
                for (bt, bspan) in bot.tracks().tracks().enumerate() {
                    if (bspan.center() - tspan.center()).abs() > via.od_width {
                        continue;
                    }
                    targets.insert(bt as i64);
                }

                for &bt in targets.iter() {
                    for (mt, _) in mid.tracks().tracks().enumerate() {
                        let mt = mt as i64;
                        let (x, y, lx, ly) = match mid.dir.track_dir() {
                            Dir::Horiz => (tt, mt, bt, mt),
                            Dir::Vert => (mt, tt, mt, bt),
                        };
                        let src = Pos::new(LayerId(i + 1), x, y);
                        let dst = Pos::new(LayerId(i), lx, ly);
                        let requires: Vec<Pos> = targets
                            .iter()
                            .map(|&bt| match mid.dir.track_dir() {
                                Dir::Horiz => Pos::new(LayerId(i), bt, mt),
                                Dir::Vert => Pos::new(LayerId(i), mt, bt),
                            })
                            .collect();

                        let ilt = InterlayerTransition {
                            src,
                            dst,
                            requires: requires.clone(),
                        };
                        ilts.push(ilt.reversed());
                        ilts.push(ilt);
                    }
                }
            }

            for ilt in ilts {
                out.add_ilt(ilt);
            }
        }

        out
    }

    fn empty() -> Self {
        Self {
            slices: HashMap::new(),
            ilts: HashMap::new(),
        }
    }

    fn slice(&self, layer: impl Into<LayerId>) -> &RoutingSlice {
        let layer = layer.into();
        &self.slices[&layer]
    }

    /// Returns an iterator over the points accessible to `net` starting from `pos`.
    pub fn next(&self, pos: Pos, net: NetId) -> impl Iterator<Item = Pos> {
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
                if ilt
                    .requires
                    .iter()
                    .all(|pos| self.is_available_for_net(net, *pos))
                {
                    successors.push(ilt.dst);
                }
            }
        }

        successors.into_iter()
    }

    fn is_available_for_net(&self, net: NetId, pos: Pos) -> bool {
        let slice = self.slice(pos.layer);
        slice.is_available_for_net(net, pos)
    }

    /// Adds the given interlayer transition to the set of allowed transitions.
    pub fn add_ilt(&mut self, ilt: InterlayerTransition) {
        let entry = self.ilts.entry(ilt.src).or_default();
        entry.insert(ilt);
    }
}
