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
pub mod route;

use crate::abs::{Abstract, DebugAbstract, InstanceAbstract, TrackCoord};
use crate::grid::{AtollLayer, LayerStack, PdkLayer};
use crate::route::{Router, ViaMaker};
use ena::unify::UnifyKey;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;

use std::sync::Arc;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::context::{prepare_cell_builder, PdkContext};
use substrate::geometry::corner::Corner;
use substrate::geometry::polygon::Polygon;
use substrate::geometry::prelude::{Bbox, Dir, Point};
use substrate::geometry::transform::{TransformMut, Transformation, Translate, TranslateMut};
use substrate::io::layout::{Builder, PortGeometry};
use substrate::io::schematic::{Bundle, Connect, Node, TerminalView};
use substrate::io::{FlatLen, Flatten};
use substrate::layout::element::Shape;

use substrate::geometry::rect::Rect;
use substrate::layout::{ExportsLayoutData, Layout};
use substrate::pdk::layers::{HasPin, Layers};
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellId, ExportsNestedData, Schematic};
use substrate::{geometry, io, layout, schematic};

/// Identifies nets in a routing solver.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NetId(pub(crate) usize);

/// Virtual layers for use in ATOLL.
#[derive(Layers)]
pub struct VirtualLayers {
    /// The layer indicating the outline of an ATOLL tile.
    ///
    /// Must be aligned to the LCM grid of the cell's top layer or,
    /// if the cell's top layer is layer 0, layer 1.
    pub outline: Outline,
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

    pub fn is_routed_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => false,
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
    nets: Vec<NetId>,
}

/// The orientation of an instance.
///
/// Orientations are applied such that the bounding box of the instance is preserved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Orientation {
    /// The default orientation.
    #[default]
    R0,
    /// Rotated 180 degrees.
    R180,
    /// Reflect vertically (ie. about the x-axis).
    ReflectVert,
    /// Reflect horizontally (ie. about the y-axis).
    ReflectHoriz,
}

impl From<Orientation> for geometry::orientation::NamedOrientation {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::R0 => geometry::orientation::NamedOrientation::R0,
            Orientation::R180 => geometry::orientation::NamedOrientation::R180,
            Orientation::ReflectVert => geometry::orientation::NamedOrientation::ReflectVert,
            Orientation::ReflectHoriz => geometry::orientation::NamedOrientation::ReflectHoriz,
        }
    }
}

impl From<Orientation> for geometry::orientation::Orientation {
    fn from(value: Orientation) -> Self {
        Into::<geometry::orientation::NamedOrientation>::into(value).into()
    }
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

    /// Returns the physical location of this instance.
    pub fn physical_loc(&self) -> Point {
        let slice = self.abs.slice();
        let w = slice.lcm_unit_width();
        let h = slice.lcm_unit_height();
        Point::new(self.loc.x * w, self.loc.y * h)
    }

    pub fn top_layer(&self) -> usize {
        self.abs.top_layer
    }

    pub fn lcm_bounds(&self) -> Rect {
        self.abs.lcm_bounds.translate(self.loc)
    }

    pub fn physical_bounds(&self) -> Rect {
        self.abs.physical_bounds().translate(self.physical_loc())
    }
}

/// A builder for ATOLL tiles.
pub struct TileBuilder<'a, PDK: Pdk + Schema> {
    nodes: HashMap<Node, NodeInfo>,
    connections: ena::unify::InPlaceUnificationTable<NodeKey>,
    schematic: &'a mut schematic::CellBuilder<PDK>,
    layout: &'a mut layout::CellBuilder<PDK>,
    layer_stack: Arc<LayerStack<PdkLayer>>,
    /// Abstracts of instantiated instances.
    abs: Vec<InstanceAbstract>,
    top_layer: usize,
    next_net_id: usize,
    router: Option<Arc<dyn Router>>,
    via_maker: Option<Arc<dyn ViaMaker<PDK>>>,
}

impl<'a, PDK: Pdk + Schema> TileBuilder<'a, PDK> {
    fn register_bundle<T: Flatten<Node>>(&mut self, bundle: &T) {
        let nodes: Vec<Node> = bundle.flatten_vec();
        let keys: Vec<NodeKey> = nodes.iter().map(|_| self.connections.new_key(())).collect();
        self.nodes.extend(
            nodes
                .into_iter()
                .zip(keys.into_iter().map(|key| NodeInfo { key, nets: vec![] })),
        );
    }

    fn new<T: io::schematic::IsBundle>(
        schematic_io: &'a T,
        schematic: &'a mut schematic::CellBuilder<PDK>,
        layout: &'a mut layout::CellBuilder<PDK>,
    ) -> Self {
        let mut nodes = HashMap::new();
        let mut connections = ena::unify::InPlaceUnificationTable::new();
        // todo: fix how layer is provided
        let layer_stack = layout
            .ctx()
            .get_installation::<LayerStack<PdkLayer>>()
            .unwrap();

        let mut builder = Self {
            nodes,
            connections,
            schematic,
            layout,
            layer_stack,
            top_layer: 0,
            abs: Vec::new(),
            next_net_id: 0,
            router: None,
            via_maker: None,
        };

        builder.register_bundle(schematic_io);

        builder
    }

    /// Generates an ATOLL instance from a Substrate block that implements [`Schematic`]
    /// and [`Layout`].
    pub fn generate_primitive<B: Clone + Schematic<PDK> + Layout<PDK>>(
        &mut self,
        block: B,
    ) -> Instance<B> {
        let layout = self.layout.generate(block.clone());
        let schematic = self.schematic.instantiate(block);
        self.register_bundle(schematic.io());
        let abs = Abstract::generate(&self.layout.ctx, layout.raw_cell());

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
        self.register_bundle(schematic.io());
        // todo: generate abstract from AtollTile trait directly
        let abs = Abstract::generate(&self.layout.ctx, layout.raw_cell());
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
        instance: Instance<B>,
    ) -> substrate::error::Result<(schematic::Instance<B>, layout::Instance<B>)> {
        let physical_loc = instance.physical_loc();
        self.layout
            .draw(instance.layout.clone().translate(instance.physical_loc()))?;

        let parent_net_ids: Vec<_> = (0..instance.io().len())
            .map(|_| {
                self.next_net_id += 1;
                NetId(self.next_net_id)
            })
            .collect();

        instance
            .io()
            .flatten_vec()
            .iter()
            .zip(parent_net_ids.iter())
            .for_each(|(node, net)| self.nodes.get_mut(node).unwrap().nets.push(*net));

        self.set_top_layer(instance.abs.top_layer);

        self.abs.push(InstanceAbstract::new(
            instance.abs,
            instance.loc,
            instance.orientation,
            parent_net_ids,
        ));

        // todo: Use ATOLL virtual layer.
        let mut layout = instance.layout;
        let mut orig_bbox = layout.bbox().unwrap();
        layout.transform_mut(Transformation::from_offset_and_orientation(
            Point::zero(),
            instance.orientation,
        ));
        layout.translate_mut(
            orig_bbox.corner(Corner::LowerLeft) - layout.bbox().unwrap().corner(Corner::LowerLeft)
                + physical_loc,
        );

        Ok((instance.schematic, layout))
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: Flatten<Node>,
        D2: Flatten<Node>,
        D1: Connect<D2>,
    {
        let s1f: Vec<Node> = s1.flatten_vec();
        let s2f: Vec<Node> = s2.flatten_vec();
        assert_eq!(s1f.len(), s2f.len());
        s1f.into_iter().zip(s2f).for_each(|(a, b)| {
            self.connections
                .union(self.nodes[&a].key, self.nodes[&b].key);
        });
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
        self.nodes.extend(
            nodes
                .into_iter()
                .zip(keys.into_iter().map(|key| NodeInfo { key, nets: vec![] })),
        );

        bundle
    }

    // todo: add function for matching geometry

    /// Gets the global context.
    pub fn ctx(&self) -> &PdkContext<PDK> {
        self.layout.ctx()
    }

    /// Sets the top layer of this tile, dictating the layers available for routing.
    ///
    /// If the top layer is set to below the top layer of a constituent tile, it will be
    /// overwritten to the top layer of the constituent tile with the highest top layer.
    pub fn set_top_layer(&mut self, top_layer: usize) {
        self.top_layer = std::cmp::max(self.top_layer, top_layer);
    }

    pub fn set_router<T: Any + Router>(&mut self, router: T) {
        self.router = Some(Arc::new(router));
    }

    pub fn set_via_maker<T: Any + ViaMaker<PDK>>(&mut self, via_maker: T) {
        self.via_maker = Some(Arc::new(via_maker));
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

        let abs = InstanceAbstract::merge(cell.abs, cell.top_layer);

        if let Some(router) = cell.router {
            let mut to_connect = HashMap::new();
            for (_, info) in cell.nodes {
                to_connect
                    .entry(cell.connections.find(info.key))
                    .or_insert(Vec::new())
                    .extend(info.nets);
            }
            let to_connect: Vec<_> = to_connect.into_values().collect();
            let paths = router.route(abs.routing_state(), to_connect);

            for path in paths {
                for segment in path.windows(2) {
                    let (a, b) = (abs.grid_to_track(segment[0]), abs.grid_to_track(segment[1]));
                    if a.layer == b.layer {
                        // todo: handle multiple routing directions
                        assert!(a.x == b.x || a.y == b.y);
                        let layer = abs.grid.stack.layer(a.layer);
                        let (start_track, start_cross_track, end_track, end_cross_track) =
                            if layer.dir().track_dir() == Dir::Vert {
                                (a.x, a.y, b.x, b.y)
                            } else {
                                (a.y, a.x, b.y, b.x)
                            };
                        let start = abs
                            .grid
                            .track_point(a.layer, start_track, start_cross_track);
                        let end = abs.grid.track_point(b.layer, end_track, end_cross_track);
                        let track = Rect::from_point(start)
                            .union(Rect::from_point(end))
                            .expand_dir(
                                if a.x == b.x { Dir::Horiz } else { Dir::Vert },
                                abs.grid.stack.layer(a.layer).line() / 2,
                            )
                            .expand_dir(
                                if a.y == b.y { Dir::Horiz } else { Dir::Vert },
                                abs.grid.stack.layer(a.layer).endcap(),
                            );

                        cell.layout
                            .draw(Shape::new(abs.grid.stack.layer(a.layer).id, track))?;
                    } else if a.layer == b.layer + 1 || b.layer == a.layer + 1 {
                        let (a, b) = if b.layer > a.layer { (b, a) } else { (a, b) };
                        let (in_track, out_track) =
                            if abs.grid.stack.layer(a.layer).dir().track_dir() == Dir::Horiz
                                && a.x == b.x
                            {
                                (
                                    abs.grid.track(b.layer, b.x, b.y, b.y),
                                    abs.grid.track_point(a.layer, a.y, a.x),
                                )
                            } else if abs.grid.stack.layer(a.layer).dir().track_dir() == Dir::Vert
                                && a.y == b.y
                            {
                                (
                                    abs.grid.track(b.layer, b.y, b.x, b.x),
                                    abs.grid.track_point(a.layer, a.x, a.y),
                                )
                            } else {
                                panic!("cannot have a diagonal segment");
                            };

                        let track = Rect::from_spans(
                            in_track.hspan().add_point(out_track.x),
                            in_track.vspan().add_point(out_track.y),
                        );
                        cell.layout
                            .draw(Shape::new(abs.grid.stack.layer(b.layer).id, track))?;
                        if let Some(maker) = &cell.via_maker {
                            maker.draw_via(
                                cell.layout,
                                TrackCoord {
                                    layer: a.layer,
                                    x: a.x,
                                    y: a.y,
                                },
                            )?;
                        }
                    }
                }
            }
        }

        let debug = cell.layout.generate(DebugAbstract {
            abs,
            stack: (*cell.layer_stack).clone(),
        });
        cell.layout.draw(debug)?;

        Ok(layout_data)
    }
}
