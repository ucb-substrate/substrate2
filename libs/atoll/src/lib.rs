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

pub mod abs;
pub mod grid;

use crate::abs::{generate_abstract, Abstract};
use crate::grid::{LayerStack, PdkLayer};
use ena::unify::UnifyKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::context::{prepare_cell_builder, PdkContext};
use substrate::geometry::polygon::Polygon;
use substrate::geometry::prelude::{Bbox, Dir, Point};
use substrate::geometry::transform::Translate;
use substrate::io::layout::{Builder, PortGeometry};
use substrate::io::schematic::{Bundle, Connect, Node, TerminalView};
use substrate::io::Flatten;
use substrate::layout::element::Shape;
use substrate::layout::{ExportsLayoutData, Layout};
use substrate::pdk::layers::HasPin;
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellId, ExportsNestedData, Schematic};
use substrate::{io, layout, schematic};

/// Identifies nets in a routing solver.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NetId(pub(crate) usize);

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PointState {
    /// The grid point is available for routing.
    Available,
    /// The grid point is blocked.
    Blocked,
    /// The grid point is occupied by a known net.
    Routed {
        /// The net occupying this routing space.
        net: NetId,
        /// Indicates if there is a via to the layer immediately below.
        via_up: bool,
        /// Indicates if there is a via to the layer immediately above.
        via_down: bool,
    },
}

impl PointState {
    /// Whether or not the given point can be used to route the given net.
    pub fn is_available_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => true,
            Self::Routed { net: n, .. } => *n == net,
            Self::Blocked => false,
        }
    }
}

/// Allowed track directions on a routing layer.
///
/// Adjacent routing layers must have alternating track directions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NodeKey(u32);

impl UnifyKey for NodeKey {
    type Value = ();

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "NodeKey"
    }
}

#[derive(Clone, Debug)]
struct NodeInfo {
    key: NodeKey,
    geometry: Vec<PortGeometry>,
}

/// The orientation of an instance.
///
/// Orientations are applied relative to the child cell's coordinate frame.
#[derive(Clone, Copy, Debug, Default)]
pub enum Orientation {
    /// The default orientation.
    #[default]
    R0,
    /// Rotated 180 degrees.
    R180,
    /// Mirrored about the x-axis.
    MX,
    /// Mirrored about the y-axis.
    MY,
}

/// An ATOLL instance representing both a schematic and layout instance.
pub struct Instance<T: ExportsNestedData + ExportsLayoutData> {
    schematic: schematic::Instance<T>,
    layout: layout::Instance<T>,
    abs: Abstract,
    /// The location of the instance in LCM units according to the
    /// top layer in the associated [`Abstract`].
    loc: Point,
    orientation: Orientation,
}

impl<T: ExportsNestedData + ExportsLayoutData> Instance<T> {
    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate_mut(&mut self, p: Point) {
        self.loc += p;
    }

    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate(mut self, p: Point) -> Self {
        self.translate_mut(p);
        self
    }

    /// Orients this instance in the given orientation.
    pub fn orient_mut(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    /// Orients this instance in the given orientation.
    pub fn orient(mut self, orientation: Orientation) -> Self {
        self.orient_mut(orientation);
        self
    }

    /// The ports of this instance.
    ///
    /// Used for node connection purposes.
    pub fn io(&self) -> &TerminalView<<T::Io as io::schematic::HardwareType>::Bundle> {
        self.schematic.io()
    }

    /// Decomposes this ATOLL instance into Substrate schematic and layout instances.
    pub fn into_instances(self) -> (schematic::Instance<T>, layout::Instance<T>) {
        // todo: apply loc and orientation to layout instance
        let loc = self.physical_loc();
        (self.schematic, self.layout.translate(loc))
    }

    /// Returns the physical location of this instance.
    pub fn physical_loc(&self) -> Point {
        let slice = self.abs.slice();
        let w = slice.lcm_unit_width();
        let h = slice.lcm_unit_height();
        Point::new(self.loc.x * w, self.loc.y * h)
    }
}

/// A builder for ATOLL tiles.
pub struct TileBuilder<'a, PDK: Pdk + Schema> {
    nodes: HashMap<Node, NodeInfo>,
    connections: ena::unify::InPlaceUnificationTable<NodeKey>,
    schematic: &'a mut schematic::CellBuilder<PDK>,
    layout: &'a mut layout::CellBuilder<PDK>,
    layer_stack: Arc<LayerStack<PdkLayer>>,
}

impl<'a, PDK: Pdk + Schema> TileBuilder<'a, PDK> {
    fn new<T: io::schematic::IsBundle>(
        schematic_io: &'a T,
        schematic: &'a mut schematic::CellBuilder<PDK>,
        layout: &'a mut layout::CellBuilder<PDK>,
    ) -> Self {
        let mut nodes = HashMap::new();
        let mut connections = ena::unify::InPlaceUnificationTable::new();
        let io_nodes: Vec<Node> = schematic_io.flatten_vec();
        let keys: Vec<NodeKey> = io_nodes.iter().map(|_| connections.new_key(())).collect();
        nodes.extend(
            io_nodes
                .into_iter()
                .zip(keys.into_iter().map(|key| NodeInfo {
                    key,
                    geometry: vec![],
                })),
        );
        // todo: fix how layer is provided
        let layer_stack = layout
            .ctx()
            .get_installation::<LayerStack<PdkLayer>>()
            .unwrap();

        Self {
            nodes,
            connections,
            schematic,
            layout,
            layer_stack,
        }
    }

    /// Generates an ATOLL instance from a Substrate block that implements [`Schematic`]
    /// and [`Layout`].
    pub fn generate_primitive<B: Clone + Schematic<PDK> + Layout<PDK>>(
        &mut self,
        block: B,
    ) -> Instance<B> {
        let layout = self.layout.generate(block.clone());
        let schematic = self.schematic.instantiate(block);
        let abs = generate_abstract(layout.raw_cell(), self.layer_stack.as_ref());
        Instance {
            layout,
            schematic,
            abs,
            loc: Default::default(),
            orientation: Default::default(),
        }
    }

    /// Generates an ATOLL instance from a block that implements [`Tile`].
    pub fn generate<B: Clone + Tile<PDK>>(&mut self, block: B) -> Instance<TileWrapper<B>> {
        let wrapper = TileWrapper::new(block);
        let layout = self.layout.generate(wrapper.clone());
        let schematic = self.schematic.instantiate(wrapper);
        // todo: generate abstract from AtollTile trait directly
        let abs = generate_abstract(layout.raw_cell(), self.layer_stack.as_ref());
        Instance {
            layout,
            schematic,
            abs,
            loc: Default::default(),
            orientation: Default::default(),
        }
    }

    /// Draws an ATOLL instance in layout.
    pub fn draw<B: ExportsNestedData + Layout<PDK>>(
        &mut self,
        instance: &Instance<B>,
    ) -> substrate::error::Result<()> {
        self.layout
            .draw(instance.layout.clone().translate(instance.physical_loc()))?;

        Ok(())
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: Flatten<Node>,
        D2: Flatten<Node>,
        D1: Connect<D2>,
    {
        // todo: fix
        // let s1f: Vec<Node> = s1.flatten_vec();
        // let s2f: Vec<Node> = s2.flatten_vec();
        // assert_eq!(s1f.len(), s2f.len());
        // s1f.into_iter().zip(s2f).for_each(|(a, b)| {
        //     self.connections
        //         .union(self.nodes[&a].key, self.nodes[&b].key);
        // });
        self.schematic.connect(s1, s2);
    }

    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: io::schematic::HardwareType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as io::schematic::HardwareType>::Bundle {
        let bundle = self.schematic.signal(name, ty);

        let nodes: Vec<Node> = bundle.flatten_vec();
        let keys: Vec<NodeKey> = nodes.iter().map(|_| self.connections.new_key(())).collect();
        self.nodes
            .extend(nodes.into_iter().zip(keys.into_iter().map(|key| NodeInfo {
                key,
                geometry: vec![],
            })));

        bundle
    }

    // todo: add function for matching geometry

    /// Gets the global context.
    pub fn ctx(&self) -> &PdkContext<PDK> {
        self.layout.ctx()
    }
}

/// A builder for an ATOLL tile's IOs.
pub struct IoBuilder<'a, B: Block> {
    /// The schematic bundle representation of the block's IO.
    pub schematic: &'a Bundle<<B as Block>::Io>,
    /// The layout builder representation of the block's IO.
    pub layout: &'a mut Builder<<B as Block>::Io>,
}

/// A tile that can be instantiated in ATOLL.
pub trait Tile<PDK: Pdk + Schema>: ExportsNestedData + ExportsLayoutData {
    /// Builds a block's ATOLL tile.
    fn tile<'a>(
        &self,
        io: IoBuilder<'a, Self>,
        cell: &mut TileBuilder<'a, PDK>,
    ) -> substrate::error::Result<(
        <Self as ExportsNestedData>::NestedData,
        <Self as ExportsLayoutData>::LayoutData,
    )>;
}

/// A wrapper of a block implementing [`Tile`] that can be instantiated in Substrate
/// schematics and layouts.
#[derive(Debug, Copy, Clone, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileWrapper<T>(T);

impl<T> TileWrapper<T> {
    /// Creates a new wrapper of `block`.
    pub fn new(block: T) -> Self {
        Self(block)
    }
}

impl<T> Deref for TileWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Block> Block for TileWrapper<T> {
    type Io = <T as Block>::Io;

    fn id() -> ArcStr {
        <T as Block>::id()
    }

    fn name(&self) -> ArcStr {
        <T as Block>::name(&self.0)
    }

    fn io(&self) -> Self::Io {
        <T as Block>::io(&self.0)
    }
}

impl<T: ExportsNestedData> ExportsNestedData for TileWrapper<T> {
    type NestedData = <T as ExportsNestedData>::NestedData;
}

impl<T: ExportsLayoutData> ExportsLayoutData for TileWrapper<T> {
    type LayoutData = <T as ExportsLayoutData>::LayoutData;
}

impl<T, PDK: Pdk + Schema> Schematic<PDK> for TileWrapper<T>
where
    T: Tile<PDK>,
{
    fn schematic(
        &self,
        io: &Bundle<<Self as Block>::Io>,
        cell: &mut schematic::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut layout_io = io::layout::HardwareType::builder(&self.io());
        let mut layout_cell = layout::CellBuilder::new(cell.ctx().with_pdk());
        let atoll_io = IoBuilder {
            schematic: io,
            layout: &mut layout_io,
        };
        let mut cell = TileBuilder::new(io, cell, &mut layout_cell);
        let (schematic_data, _) = <T as Tile<PDK>>::tile(&self.0, atoll_io, &mut cell)?;
        Ok(schematic_data)
    }
}

impl<T, PDK: Pdk + Schema> Layout<PDK> for TileWrapper<T>
where
    T: Tile<PDK>,
{
    fn layout(
        &self,
        io: &mut Builder<<Self as Block>::Io>,
        cell: &mut layout::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let (mut schematic_cell, schematic_io) =
            prepare_cell_builder(CellId::default(), (**cell.ctx()).clone(), self);
        let io = IoBuilder {
            schematic: &schematic_io,
            layout: io,
        };
        let mut cell = TileBuilder::new(&schematic_io, &mut schematic_cell, cell);
        let (_, layout_data) = <T as Tile<PDK>>::tile(&self.0, io, &mut cell)?;

        let mut to_connect = HashMap::new();
        for (_, port) in cell.nodes {
            to_connect
                .entry(cell.connections.find(port.key))
                .or_insert(Vec::new())
                .extend(port.geometry);
        }

        for (_, ports) in to_connect {
            for pair in ports.windows(2) {
                let a = &pair[0];
                let b = &pair[1];
                let a_center = a.primary.shape().bbox().unwrap().center();
                let b_center = b.primary.shape().bbox().unwrap().center();
                cell.layout.draw(Shape::new(
                    a.primary.layer().drawing(),
                    Polygon::from_verts(vec![
                        a_center,
                        b_center,
                        b_center - Point::new(20, 20),
                        a_center - Point::new(20, 20),
                    ]),
                ))?;
            }
        }

        Ok(layout_data)
    }
}
