//! Atoll: Automatic transformation of logical layout.
//!
//! Atoll projects are made of one or more **blocks**.
//! Each block is a compact, rectangular grid of devices.
//! Each block in turn is composed of a set of tiles drawn from a TileSet.
//! TileSets provide a tile configurator for each tile archetype.
//!
//! The set of tile archetypes is given by the Cartesian product
//! of [`Col`] and [`Row`].
//!
//! A tile configurator takes tile configuration info and produces
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

use std::ops::Range;

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
