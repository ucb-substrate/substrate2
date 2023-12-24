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

pub mod grid;

use ::grid::Grid;
use derive_where::derive_where;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::geometry::prelude::{Dir, Point, Transformation};
use substrate::geometry::transform::HasTransformedView;
use substrate::io::layout::{Builder, PortGeometry, PortGeometryBuilder, TransformedPortGeometry};
use substrate::io::schematic::{Bundle, Node, Terminal};
use substrate::io::{FlatLen, Flatten, Signal};
use substrate::layout::tracks::{EnumeratedTracks, FiniteTracks, Tracks};
use substrate::layout::{ExportsLayoutData, Layout};
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{
    CellId, ExportsNestedData, HasNestedView, InstanceId, InstancePath, Schematic,
};
use substrate::serde::Deserialize;
use substrate::{io, layout, schematic};

/// Identifies nets in a routing solver.
pub type NetId = usize;

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

// todo: how to connect by abutment (eg body terminals)

/// The abstract view of an ATOLL tile.
pub struct AtollAbstract {
    /// The topmost ATOLL layer used within the tile.
    top_layer: usize,
    /// The lower left corner of the tile, in LCM units with respect to `top_layer`.
    ll: Point,
    /// The upper right corner of the tile, in LCM units with respect to `top_layer`.
    ur: Point,
    /// The state of each layer, up to and including `top_layer`.
    layers: Vec<LayerAbstract>,
}

/// The abstracted state of a single routing layer.
pub enum LayerAbstract {
    /// The layer is fully blocked.
    ///
    /// No routing on this layer is permitted.
    Blocked,
}

pub struct AtollTileBuilder<S: Schema + ?Sized, PDK: Pdk> {
    connections: Vec<(PortGeometry, PortGeometry)>,
    schematic: schematic::CellBuilder<S>,
    layout: layout::CellBuilder<PDK>,
}

pub trait HardwareType {
    /// The **Rust** type representing ATOLL instances of this **hardware** type.
    type Bundle: IsBundle;
    /// A builder for creating [`HardwareType::Bundle`].
    type Builder: BundleBuilder<Self::Bundle>;

    /// Instantiates a builder for this hardware type's bundle.
    fn builder<'n>(&self, ids: &'n [Node]) -> (Self::Builder, &'n [Node]);
}

/// A bundle of schematic nodes.
///
/// An instance of a [`HardwareType`].
pub trait IsBundle: FlatLen + Flatten<AtollNode> + HasTerminalView + Clone + Send + Sync {}

impl<T> IsBundle for T where T: FlatLen + Flatten<AtollNode> + HasTerminalView + Clone + Send + Sync {}

pub trait HasTerminalView:
    io::schematic::HasTerminalView<TerminalView = <Self as HasTerminalView>::TerminalView>
{
    type TerminalView: HasTransformedView;
}

impl<T: io::schematic::HasTerminalView<TerminalView = impl HasTransformedView>> HasTerminalView
    for T
{
    type TerminalView = <T as io::schematic::HasTerminalView>::TerminalView;
}

/// ATOLL bundle builder.
///
/// A builder for an instance of bundle `T`.
pub trait BundleBuilder<T: IsBundle> {
    /// Builds an instance of bundle `T`.
    fn build(self) -> substrate::error::Result<T>;
}

#[derive(Debug, Clone)]
pub struct AtollNode(Node, PortGeometry);
#[derive(Debug, Clone)]
pub struct AtollNodeBuilder(Node, PortGeometryBuilder);
#[derive(Debug, Clone)]
pub struct AtollTerminal(Terminal, PortGeometry);
#[derive(Clone)]
pub struct TransformedAtollTerminal<'a>(Terminal, TransformedPortGeometry<'a>);

impl FlatLen for AtollNode {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<AtollNode> for AtollNode {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<AtollNode>,
    {
        output.extend(std::iter::once(self.clone()))
    }
}

impl io::schematic::HasTerminalView for AtollNode {
    type TerminalView = AtollTerminal;

    fn terminal_view(
        cell: CellId,
        cell_io: &Self,
        instance: InstanceId,
        instance_io: &Self,
    ) -> Self::TerminalView {
        AtollTerminal(
            <Node as io::schematic::HasTerminalView>::terminal_view(
                cell,
                &cell_io.0,
                instance,
                &instance_io.0,
            ),
            cell_io.1.clone(),
        )
    }
}

impl BundleBuilder<AtollNode> for AtollNodeBuilder {
    fn build(self) -> substrate::error::Result<AtollNode> {
        Ok(AtollNode(
            self.0,
            <PortGeometryBuilder as io::layout::BundleBuilder<PortGeometry>>::build(self.1)?,
        ))
    }
}

impl HasTransformedView for AtollTerminal {
    type TransformedView<'a> = TransformedAtollTerminal<'a>;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView<'_> {
        TransformedAtollTerminal(self.0, self.1.transformed_view(trans))
    }
}

// TODO: fix
impl HasNestedView for AtollTerminal {
    type NestedView = ();

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        ()
    }
}

impl FlatLen for AtollTerminal {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for AtollTerminal {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output)
    }
}

impl HardwareType for Signal {
    type Bundle = AtollNode;
    type Builder = AtollNodeBuilder;

    fn builder<'n>(&self, ids: &'n [Node]) -> (Self::Builder, &'n [Node]) {
        (
            AtollNodeBuilder(ids[0], PortGeometryBuilder::default()),
            &ids[1..],
        )
    }
}

pub trait AtollTile<S: Schema, PDK: Pdk>: ExportsNestedData + ExportsLayoutData {
    type Io: HardwareType;

    fn tile(
        &self,
        io: <<Self as AtollTile<S, PDK>>::Io as HardwareType>::Builder,
        cell: &mut AtollTileBuilder<S, PDK>,
    ) -> substrate::error::Result<(
        <Self as ExportsNestedData>::NestedData,
        <Self as ExportsLayoutData>::LayoutData,
    )>;
}

#[derive_where(Debug, Clone, Hash, PartialEq, Eq; T)]
#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct AtollTileWrapper<T, S, PDK> {
    inner: T,
    phantom: PhantomData<(S, PDK)>,
}

impl<T: Block, S: Schema, PDK: Pdk> Block for AtollTileWrapper<T, S, PDK> {
    type Io = <T as Block>::Io;

    fn id() -> ArcStr {
        <T as Block>::id()
    }

    fn name(&self) -> ArcStr {
        <T as Block>::name(&self.inner)
    }

    fn io(&self) -> Self::Io {
        <T as Block>::io(&self.inner)
    }
}

impl<T: ExportsNestedData, S: Schema, PDK: Pdk> ExportsNestedData for AtollTileWrapper<T, S, PDK> {
    type NestedData = <T as ExportsNestedData>::NestedData;
}

impl<T: ExportsLayoutData, S: Schema, PDK: Pdk> ExportsLayoutData for AtollTileWrapper<T, S, PDK> {
    type LayoutData = <T as ExportsLayoutData>::LayoutData;
}

impl<T, S: Schema, PDK: Pdk> Schematic<S> for AtollTileWrapper<T, S, PDK>
where
    T: AtollTile<S, PDK>,
{
    fn schematic(
        &self,
        io: &Bundle<<Self as Block>::Io>,
        cell: &mut schematic::CellBuilder<S>,
    ) -> substrate::error::Result<Self::NestedData> {
        todo!()
    }
}

impl<T, S: Schema, PDK: Pdk> Layout<PDK> for AtollTileWrapper<T, S, PDK>
where
    T: AtollTile<S, PDK>,
{
    fn layout(
        &self,
        io: &mut Builder<<Self as Block>::Io>,
        cell: &mut layout::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        todo!()
    }
}
