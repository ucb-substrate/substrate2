//! Atoll: Automatic transformation of logical layout.
//!
//! # Grid structure
//!
//! Atoll assumes that you have a set of metal layers `M0, M1, M2, ...`, where each metal
//! layer can be connected to the layer immediately above or below.
//! Atoll also assumes that each metal layer has a preferred direction, and that
//! horizontal and vertical metals alternate.
//!
//! Suppose that `P(i)` is the pitch of the i-th routing layer.
//! Each ATOLL tile must pick an integer X such that:
//! * A complete block can be assembled from layers `M0, M1, ..., MX`
//! * In particular, tiles contain no routing layers above `MX`.
//! * The width of all tiles is an integer multiple of `LCM { P(0), P(2), ... }`,
//!   assuming `M0` is vertical.
//! * The height of all tiles is an integer multiple of `LCM { P(1), P(3), ... }`,
//!   assuming `M0` is vertical (and therefore that `M1` is horizontal).
//!
//! The line and space widths must each be even integers, so that the center
//! of any track or space is also an integer.
//!
//! For symmetry guarantees, each routing layer can declare having no offset
//! or a half-pitch offset. Half-pitch offset routing layers have one more
//! track available as tiles may not use tracks that straddle ATOLL tile boundaries.
//! These tracks may be used by encompassing tiles, however.
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
//! # Routing
//!
//! Each routing layer can only travel in one direction. Therefore, we can assume
//! that if two adjacent coordinates are of the same net are adjacent in the routing direction,
//! they are connected. In the perpendicular direction, they are not connected.
//!
//! Endcaps may be specified, but should not cause DRC violations if there is a cut
//! between tracks. These only affect how track coordinates are converted to physical rectangles
//! by ATOLL's APIs.
//!
//! # Post layout hooks
//!
//! ATOLL should have a way of passing layout information such as instantiations of "primitive"
//! tiles to the plugin in an un-typed manner such that the plugin can run post-processing on the layout
//! before actually drawing it to GDS.
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
#![warn(missing_docs)]

pub mod abs;
pub mod grid;
pub mod route;
pub mod straps;

use crate::abs::{Abstract, InstanceAbstract, TrackCoord};
use crate::grid::{AtollLayer, LayerStack, PdkLayer};
use crate::route::{Path, Router, ViaMaker};
use ena::unify::UnifyKey;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;

use cache::mem::TypeCache;
use indexmap::{IndexMap, IndexSet};
use std::sync::{Arc, RwLock};
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::context::{prepare_cell_builder, PdkContext, PrivateInstallation};
use substrate::geometry::corner::Corner;
use substrate::geometry::prelude::{Dir, Point};
use substrate::geometry::transform::{
    Transform, TransformMut, Transformation, Translate, TranslateMut,
};
use substrate::io::layout::Builder;
use substrate::io::schematic::{Bundle, Connect, HardwareType, IsBundle, Node, TerminalView};
use substrate::io::Flatten;
use substrate::layout::element::Shape;

use crate::straps::{Strapper, StrappingParams};
use substrate::geometry::align::AlignMode;
use substrate::geometry::rect::Rect;
use substrate::layout::bbox::LayerBbox;
use substrate::layout::{ExportsLayoutData, Layout};
use substrate::pdk::layers::{Layer, Layers};
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellId, ExportsNestedData, Schematic};
use substrate::{geometry, io, layout, schematic};

#[derive(Default, Debug)]
struct AtollContext(RwLock<AtollContextInner>);

#[derive(Default, Debug)]
struct AtollContextInner {
    pub(crate) cell_cache: TypeCache,
}

impl PrivateInstallation for AtollContext {}

/// Identifies nets in a routing solver.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NetId(pub(crate) usize);

/// Virtual layers for use in ATOLL.
#[derive(Layers)]
#[allow(missing_docs)]
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
    Blocked {
        /// Whether there is a via at this point.
        has_via: bool,
    },
    /// The grid point is occupied by a known net.
    Routed {
        /// The net occupying this routing space.
        net: NetId,
        /// Whether there is a via at this point.
        has_via: bool,
    },
    /// The grid point is reserved for a known net.
    ///
    /// This means the net may or may not be physically routed on this grid point,
    /// but that no other net can occupy the grid point.
    ///
    /// A net can make an interlayer transition if it has reserved grid points
    /// adjacent to unaligned transitions.
    Reserved {
        /// The net reserving this routing space.
        net: NetId,
    },
}

impl PointState {
    /// Whether or not the given point can be used to route the given net.
    pub fn is_available_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => true,
            Self::Routed { net: n, .. } => *n == net,
            Self::Blocked { .. } => false,
            PointState::Reserved { .. } => false, // todo might need to change this
        }
    }

    /// Whether or not the given point is routed with the given net.
    pub fn is_routed_for_net(&self, net: NetId) -> bool {
        match self {
            Self::Available => false,
            Self::Routed { net: n, .. } => *n == net,
            Self::Blocked { .. } => false,
            PointState::Reserved { .. } => false,
        }
    }

    /// Whether or not the given point has a via.
    pub fn has_via(&self) -> bool {
        match self {
            Self::Routed { has_via, .. } => *has_via,
            Self::Blocked { has_via, .. } => *has_via,
            _ => false,
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
    net: NetId,
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
        geometry::orientation::NamedOrientation::from(value).into()
    }
}

/// An ATOLL instance representing both a schematic and layout instance.
pub struct Instance<T: ExportsNestedData + ExportsLayoutData> {
    schematic: schematic::Instance<T>,
    layout: layout::Instance<T>,
    raw: RawInstance,
}

/// An ATOLL instance with typed components stripped out.
pub struct RawInstance {
    abs: Abstract,
    /// The location of the instance in LCM units according to the
    /// top layer in the associated [`Abstract`].
    loc: Point,
    orientation: Orientation,
}

impl RawInstance {
    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate_mut(&mut self, p: Point) {
        self.loc += p;
    }

    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate(mut self, p: Point) -> Self {
        self.translate_mut(p);
        self
    }

    /// Aligns this instance with another rectangle in terms of LCM units on the same
    /// layer as the instance.
    pub fn align_rect_mut(&mut self, orect: Rect, mode: AlignMode, offset: i64) {
        let srect = self.lcm_bounds();
        match mode {
            AlignMode::Left => {
                self.translate_mut(Point::new(orect.left() - srect.left() + offset, 0));
            }
            AlignMode::Right => {
                self.translate_mut(Point::new(orect.right() - srect.right() + offset, 0));
            }
            AlignMode::Bottom => {
                self.translate_mut(Point::new(0, orect.bot() - srect.bot() + offset));
            }
            AlignMode::Top => {
                self.translate_mut(Point::new(0, orect.top() - srect.top() + offset));
            }
            AlignMode::ToTheRight => {
                self.translate_mut(Point::new(orect.right() - srect.left() + offset, 0));
            }
            AlignMode::ToTheLeft => {
                self.translate_mut(Point::new(orect.left() - srect.right() + offset, 0));
            }
            AlignMode::CenterHorizontal => {
                self.translate_mut(Point::new(
                    ((orect.left() + orect.right()) - (srect.left() + srect.right())) / 2 + offset,
                    0,
                ));
            }
            AlignMode::CenterVertical => {
                self.translate_mut(Point::new(
                    0,
                    ((orect.bot() + orect.top()) - (srect.bot() + srect.top())) / 2 + offset,
                ));
            }
            AlignMode::Beneath => {
                self.translate_mut(Point::new(0, orect.bot() - srect.top() + offset));
            }
            AlignMode::Above => {
                self.translate_mut(Point::new(0, orect.top() - srect.bot() + offset));
            }
        }
    }

    /// Aligns this instance with another rectangle in terms of LCM units on the same
    /// layer as the instance.
    pub fn align_rect(mut self, orect: Rect, mode: AlignMode, offset: i64) -> Self {
        self.align_rect_mut(orect, mode, offset);
        self
    }

    /// Aligns this instance with another instance.
    pub fn align_mut(&mut self, other: &RawInstance, mode: AlignMode, offset: i64) {
        let lcm_bounds = self
            .abs
            .grid
            .stack
            .slice(0..self.top_layer() + 1)
            .expand_to_lcm_units(other.physical_bounds());
        self.align_rect_mut(lcm_bounds, mode, offset);
    }

    /// Aligns this instance with another instance with the same top layer.
    pub fn align(mut self, other: &RawInstance, mode: AlignMode, offset: i64) -> Self {
        self.align_mut(other, mode, offset);
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

    /// Returns the physical location of this instance.
    pub fn physical_loc(&self) -> Point {
        let slice = self.abs.slice();
        let w = slice.lcm_unit_width();
        let h = slice.lcm_unit_height();
        Point::new(self.loc.x * w, self.loc.y * h)
    }

    /// Returns the top layer of this instance.
    pub fn top_layer(&self) -> usize {
        self.abs.top_layer
    }

    /// Returns the LCM bounds of this instance.
    pub fn lcm_bounds(&self) -> Rect {
        self.abs.lcm_bounds.translate(self.loc)
    }

    /// Returns the physical bounds of this instance.
    pub fn physical_bounds(&self) -> Rect {
        self.abs.physical_bounds().translate(self.physical_loc())
    }
}

impl<T: ExportsNestedData + ExportsLayoutData> Instance<T> {
    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate_mut(&mut self, p: Point) {
        self.raw.translate_mut(p);
    }

    /// Translates this instance by the given XY-coordinates in LCM units.
    pub fn translate(mut self, p: Point) -> Self {
        self.translate_mut(p);
        self
    }

    /// Aligns this instance with another rectangle in terms of LCM units on the same
    /// layer as the instance.
    pub fn align_rect_mut(&mut self, orect: Rect, mode: AlignMode, offset: i64) {
        self.raw.align_rect_mut(orect, mode, offset);
    }

    /// Aligns this instance with another rectangle in terms of LCM units on the same
    /// layer as the instance.
    pub fn align_rect(mut self, orect: Rect, mode: AlignMode, offset: i64) -> Self {
        self.align_rect_mut(orect, mode, offset);
        self
    }

    /// Aligns this instance with another instance.
    pub fn align_mut<T2: ExportsNestedData + ExportsLayoutData>(
        &mut self,
        other: &Instance<T2>,
        mode: AlignMode,
        offset: i64,
    ) {
        self.raw.align_mut(&other.raw, mode, offset);
    }

    /// Aligns this instance with another instance with the same top layer.
    pub fn align<T2: ExportsNestedData + ExportsLayoutData>(
        mut self,
        other: &Instance<T2>,
        mode: AlignMode,
        offset: i64,
    ) -> Self {
        self.align_mut(other, mode, offset);
        self
    }

    /// Orients this instance in the given orientation.
    pub fn orient_mut(&mut self, orientation: Orientation) {
        self.raw.orient_mut(orientation);
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
        self.raw.physical_loc()
    }

    /// Returns the top layer of this instance.
    pub fn top_layer(&self) -> usize {
        self.raw.top_layer()
    }

    /// Returns the LCM bounds of this instance.
    pub fn lcm_bounds(&self) -> Rect {
        self.raw.lcm_bounds()
    }

    /// Returns the physical bounds of this instance.
    pub fn physical_bounds(&self) -> Rect {
        self.raw.physical_bounds()
    }

    /// Returns a reference to the underlying [`RawInstance`].
    pub fn raw(&self) -> &RawInstance {
        &self.raw
    }

    /// Returns a mutable reference to the underlying [`RawInstance`].
    pub fn raw_mut(&mut self) -> &mut RawInstance {
        &mut self.raw
    }

    /// Returns the underlying [`RawInstance`].
    pub fn into_raw(self) -> RawInstance {
        self.raw
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AssignedGridPoints {
    pub(crate) net: Option<NetId>,
    pub(crate) layer: usize,
    pub(crate) bounds: Rect,
    pub(crate) only_if_available: bool,
}

/// A builder for ATOLL tiles.
pub struct TileBuilder<'a, PDK: Pdk + Schema + ?Sized> {
    nodes: IndexMap<Node, NodeInfo>,
    connections: ena::unify::InPlaceUnificationTable<NodeKey>,
    schematic: &'a mut schematic::CellBuilder<PDK>,
    /// The layout builder.
    pub layout: &'a mut layout::CellBuilder<PDK>,
    /// The layer stack.
    pub layer_stack: Arc<LayerStack<PdkLayer>>,
    /// Abstracts of instantiated instances.
    abs: Vec<InstanceAbstract>,
    assigned_nets: Vec<AssignedGridPoints>,
    layers_to_block: IndexSet<usize>,
    skip_nets: IndexSet<NetId>,
    skip_all_nets: IndexSet<NetId>,
    top_layer: usize,
    next_net_id: usize,
    router: Option<Arc<dyn Router>>,
    strapper: Option<Arc<dyn Strapper>>,
    via_maker: Option<Arc<dyn ViaMaker<PDK>>>,
    straps: Vec<(NetId, StrappingParams)>,
}

/// Fields required for building an abstract.
struct TileAbstractBuilder {
    nodes: IndexMap<Node, NodeInfo>,
    connections: ena::unify::InPlaceUnificationTable<NodeKey>,
    abs: Vec<InstanceAbstract>,
    assigned_nets: Vec<AssignedGridPoints>,
    skip_nets: IndexSet<NetId>,
    skip_all_nets: IndexSet<NetId>,
    top_layer: usize,
    router: Option<Arc<dyn Router>>,
    strapper: Option<Arc<dyn Strapper>>,
    straps: Vec<(NetId, StrappingParams)>,
    layers_to_block: IndexSet<usize>,
    layer_bbox: Option<Rect>,
    port_ids: Vec<NetId>,
}

/// Remaining fields of [`TileBuilder`] not contained in [`TileAbstractBuilder`].
struct TileBuilderUnused<'a, PDK: Pdk + Schema + ?Sized> {
    #[allow(dead_code)]
    schematic: &'a mut schematic::CellBuilder<PDK>,
    /// The layout builder.
    pub layout: &'a mut layout::CellBuilder<PDK>,
    /// The layer stack.
    #[allow(dead_code)]
    pub layer_stack: Arc<LayerStack<PdkLayer>>,
    #[allow(dead_code)]
    next_net_id: usize,
    via_maker: Option<Arc<dyn ViaMaker<PDK>>>,
}

/// A drawn ATOLL instance.
pub struct DrawnInstance<T: ExportsNestedData + ExportsLayoutData> {
    /// The underlying Substrate schematic instance.
    pub schematic: schematic::Instance<T>,
    /// The underlying Substrate layout instance.
    pub layout: layout::Instance<T>,
}

impl TileAbstractBuilder {
    fn finalize_abstract(self) -> (Abstract, Vec<Path>) {
        let TileAbstractBuilder {
            nodes,
            mut connections,
            abs,
            assigned_nets,
            top_layer,
            router,
            skip_nets,
            layers_to_block,
            skip_all_nets,
            strapper,
            straps,
            layer_bbox,
            port_ids,
        } = self;
        let mut abs = InstanceAbstract::merge(abs, top_layer, layer_bbox, port_ids, assigned_nets);

        for layer in layers_to_block {
            abs.block_available_on_layer(layer);
        }

        let mut routing_state = abs.routing_state();

        let mut roots = HashMap::new();

        // All of the nets that needed to be connected in the final abstract.
        let mut to_connect_raw = IndexMap::new();
        for (_, info) in nodes {
            to_connect_raw
                .entry(connections.find(info.key))
                .or_insert(IndexSet::new())
                .insert(info.net);
        }

        // Raw nets with skipped nets filtered out.
        let mut to_connect = to_connect_raw.clone();

        for (_, seq) in to_connect.iter_mut() {
            let group = *seq.iter().next().unwrap();
            for node in seq.iter() {
                roots.insert(*node, group);
            }
            for net in skip_nets.iter() {
                seq.swap_remove(net);
            }
        }
        routing_state.roots = roots;

        let to_connect: Vec<_> = to_connect
            .clone()
            .into_values()
            .filter(|nets| skip_all_nets.is_disjoint(nets))
            .map(Vec::from_iter)
            .collect();

        let mut paths = Vec::new();

        if let Some(router) = router {
            paths.extend(router.route(&mut routing_state, to_connect));
        }
        if let Some(strapper) = strapper {
            paths.extend(strapper.strap(&mut routing_state, straps));
        }
        for (_, nets) in to_connect_raw {
            for net in nets {
                routing_state.relabel_net(net, routing_state.roots[&net]);
            }
        }
        abs.from_routing_state(routing_state);
        (abs, paths)
    }
}

impl<'a, PDK: Pdk + Schema> TileBuilder<'a, PDK> {
    /// Splits off data not required for building an abstract.
    fn split_for_abstract(
        self,
        port_nodes: Vec<Node>,
    ) -> (TileAbstractBuilder, TileBuilderUnused<'a, PDK>) {
        let virtual_layers = self.layout.ctx.install_layers::<crate::VirtualLayers>();
        let layer_bbox = self.layout.layer_bbox(virtual_layers.outline.id());

        let port_ids = port_nodes.iter().map(|node| self.nodes[node].net).collect();

        let TileBuilder {
            nodes,
            connections,
            abs,
            assigned_nets,
            layers_to_block,
            skip_nets,
            skip_all_nets,
            top_layer,
            next_net_id,
            router,
            strapper,
            via_maker,
            straps,
            layer_stack,
            layout,
            schematic,
        } = self;
        (
            TileAbstractBuilder {
                nodes,
                connections,
                abs,
                assigned_nets,
                layers_to_block,
                skip_nets,
                skip_all_nets,
                top_layer,
                router,
                strapper,
                straps,
                layer_bbox,
                port_ids,
            },
            TileBuilderUnused {
                next_net_id,
                via_maker,
                layer_stack,
                layout,
                schematic,
            },
        )
    }
    fn register_bundle<T: Flatten<Node>>(&mut self, bundle: &T) {
        let nodes: Vec<Node> = bundle.flatten_vec();
        let keys: Vec<NodeKey> = nodes.iter().map(|_| self.connections.new_key(())).collect();

        let net_ids: Vec<_> = (0..nodes.len()).map(|_| self.generate_net_id()).collect();

        self.nodes.extend(
            nodes.into_iter().zip(
                keys.into_iter()
                    .zip(net_ids)
                    .map(|(key, net)| NodeInfo { key, net }),
            ),
        );
    }

    fn new<T: io::schematic::IsBundle>(
        schematic_io: &'a T,
        schematic: &'a mut schematic::CellBuilder<PDK>,
        layout: &'a mut layout::CellBuilder<PDK>,
    ) -> Self {
        let layer_stack = layout
            .ctx()
            .get_installation::<LayerStack<PdkLayer>>()
            .unwrap();

        let mut builder = Self {
            nodes: IndexMap::new(),
            connections: ena::unify::InPlaceUnificationTable::new(),
            schematic,
            layout,
            layer_stack,
            top_layer: 0,
            abs: Vec::new(),
            assigned_nets: Vec::new(),
            layers_to_block: IndexSet::new(),
            skip_nets: IndexSet::new(),
            skip_all_nets: IndexSet::new(),
            next_net_id: 0,
            router: None,
            strapper: None,
            via_maker: None,
            straps: Vec::new(),
        };

        builder.register_bundle(schematic_io);

        builder
    }

    /// Flattens the schematic of this tile.
    pub fn flatten(&mut self) {
        self.schematic.flatten()
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
            raw: RawInstance {
                abs,
                loc: Default::default(),
                orientation: Default::default(),
            },
        }
    }

    /// Generates an ATOLL instance from a Substrate block that implements [`Schematic`]
    /// and [`Layout`] and connects its IO to the given bundle.
    pub fn generate_primitive_connected<B: Clone + Schematic<PDK> + Layout<PDK>, C: IsBundle>(
        &mut self,
        block: B,
        io: C,
    ) -> Instance<B>
    where
        for<'b> &'b TerminalView<<B::Io as HardwareType>::Bundle>: Connect<C>,
    {
        let inst = self.generate_primitive(block);
        self.connect(inst.io(), io);

        inst
    }

    /// Generates an ATOLL instance from a block that implements [`Tile`].
    pub fn generate<B: Clone + Tile<PDK>>(&mut self, block: B) -> Instance<TileWrapper<B>> {
        let atoll_ctx = self.ctx().get_or_install(AtollContext::default());
        let ctx_clone = (**self.ctx()).clone();
        let abs_path =
            atoll_ctx
                .0
                .write()
                .unwrap()
                .cell_cache
                .generate(block.clone(), move |block| {
                    let (mut schematic_cell, schematic_io) =
                        prepare_cell_builder(CellId::default(), ctx_clone.clone(), block);
                    let mut layout_io = io::layout::HardwareType::builder(&block.io());
                    let mut layout_cell = layout::CellBuilder::new(ctx_clone.with_pdk());
                    let atoll_io = IoBuilder {
                        schematic: &schematic_io,
                        layout: &mut layout_io,
                    };
                    let mut cell =
                        TileBuilder::new(&schematic_io, &mut schematic_cell, &mut layout_cell);
                    let _ = <B as Tile<PDK>>::tile(block, atoll_io, &mut cell);

                    cell.split_for_abstract(schematic_io.flatten_vec())
                        .0
                        .finalize_abstract()
                });

        let (abs, _) = abs_path.get().clone();
        let wrapper = TileWrapper::new(block);
        let layout = self.layout.generate(wrapper.clone());
        let schematic = self.schematic.instantiate(wrapper);
        self.register_bundle(schematic.io());
        Instance {
            layout,
            schematic,
            raw: RawInstance {
                abs,
                loc: Default::default(),
                orientation: Default::default(),
            },
        }
    }

    /// Generates an ATOLL instance from a block that implements [`Tile`]
    /// and connects its IO to the given bundle.
    pub fn generate_connected<B: Clone + Tile<PDK>, C: IsBundle>(
        &mut self,
        block: B,
        io: C,
    ) -> Instance<TileWrapper<B>>
    where
        for<'b> &'b TerminalView<<B::Io as HardwareType>::Bundle>: Connect<C>,
    {
        let inst = self.generate(block);
        self.connect(inst.io(), io);

        inst
    }

    fn generate_net_id(&mut self) -> NetId {
        let ret = NetId(self.next_net_id);
        self.next_net_id += 1;
        ret
    }

    /// Draws an ATOLL instance in layout.
    pub fn draw<B: ExportsNestedData + Layout<PDK>>(
        &mut self,
        instance: Instance<B>,
    ) -> substrate::error::Result<DrawnInstance<B>> {
        let physical_loc = instance.physical_loc();

        let parent_net_ids = instance
            .io()
            .flatten_vec()
            .iter()
            .map(|node| self.nodes.get_mut(node).unwrap().net)
            .collect();

        self.set_top_layer(instance.raw.abs.top_layer);

        let virtual_layers = self.layout.ctx.install_layers::<crate::VirtualLayers>();
        let orig_bbox = instance.raw.abs.grid.slice().lcm_to_physical_rect(
            instance.raw.abs.grid.slice().expand_to_lcm_units(
                instance
                    .layout
                    .layer_bbox(virtual_layers.outline.id())
                    .unwrap(),
            ),
        );
        self.abs.push(InstanceAbstract::new(
            instance.raw.abs,
            instance.raw.loc,
            instance.raw.orientation,
            parent_net_ids,
        ));

        // todo: Use ATOLL virtual layer.
        let mut layout = instance.layout;
        layout.transform_mut(Transformation::from_offset_and_orientation(
            Point::zero(),
            instance.raw.orientation,
        ));
        let new_bbox = orig_bbox.transform(Transformation::from_offset_and_orientation(
            Point::zero(),
            instance.raw.orientation,
        ));
        layout.translate_mut(
            orig_bbox.corner(Corner::LowerLeft) - new_bbox.corner(Corner::LowerLeft) + physical_loc,
        );
        self.layout.draw(layout.clone())?;

        Ok(DrawnInstance {
            schematic: instance.schematic,
            layout,
        })
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

        self.register_bundle(&bundle);

        bundle
    }

    /// Assigns grid points to the provided node.
    ///
    /// If the provided node is `None`, blocks the grid point for routing.
    pub fn assign_grid_points(&mut self, node: Option<Node>, layer: usize, bounds: Rect) {
        self.assigned_nets.push(AssignedGridPoints {
            net: node.map(|node| self.nodes[&node].net),
            layer,
            bounds,
            only_if_available: false,
        })
    }

    /// Assigns grid points to the provided node, but only if the grid point is currently marked available.
    ///
    /// If the provided node is `None`, blocks the grid point for routing.
    pub fn assign_grid_points_if_available(
        &mut self,
        node: Option<Node>,
        layer: usize,
        bounds: Rect,
    ) {
        self.assigned_nets.push(AssignedGridPoints {
            net: node.map(|node| self.nodes[&node].net),
            layer,
            bounds,
            only_if_available: true,
        })
    }

    /// Blocks all remaining available grid points on the given layer.
    pub fn block_available_on_layer(&mut self, layer: usize) {
        self.layers_to_block.insert(layer);
    }

    /// Set up straps for the provided node.
    ///
    /// Order of calls to `set_strapping` may matter depending on the [`Strapper`] being used.
    pub fn set_strapping(&mut self, node: Node, params: StrappingParams) {
        self.straps.push((self.nodes[&node].net, params));
    }

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

    /// Sets the router.
    pub fn set_router<T: Any + Router>(&mut self, router: T) {
        self.router = Some(Arc::new(router));
    }

    /// Skips routing a net.
    pub fn skip_routing(&mut self, node: Node) {
        self.skip_nets.insert(self.nodes[&node].net);
    }

    /// Skips routing a net.
    pub fn skip_routing_all(&mut self, node: Node) {
        self.skip_all_nets.insert(self.nodes[&node].net);
    }

    /// Sets the strapper.
    pub fn set_strapper<T: Any + Strapper>(&mut self, strapper: T) {
        self.strapper = Some(Arc::new(strapper));
    }

    /// Sets the via maker.
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
pub trait Tile<PDK: Pdk + Schema + ?Sized>: ExportsNestedData + ExportsLayoutData {
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
pub struct TileWrapper<T> {
    block: T,
}

impl<T> TileWrapper<T> {
    /// Creates a new wrapper of `block`.
    pub fn new(block: T) -> Self {
        Self { block }
    }
}

impl<T> Deref for TileWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl<T: Block> Block for TileWrapper<T> {
    type Io = <T as Block>::Io;

    fn id() -> ArcStr {
        <T as Block>::id()
    }

    fn name(&self) -> ArcStr {
        <T as Block>::name(&self.block)
    }

    fn io(&self) -> Self::Io {
        <T as Block>::io(&self.block)
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
        let (schematic_data, _) = <T as Tile<PDK>>::tile(&self.block, atoll_io, &mut cell)?;
        Ok(schematic_data)
    }
}

impl<T, PDK: Pdk + Schema> Layout<PDK> for TileWrapper<T>
where
    T: Tile<PDK> + Clone,
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
        let (_, layout_data) = <T as Tile<PDK>>::tile(&self.block, io, &mut cell)?;

        let ctx_clone = (**cell.ctx()).clone();
        let atoll_ctx = ctx_clone.get_or_install(AtollContext::default());
        let block = (**self).clone();
        let (
            cell,
            TileBuilderUnused {
                layout, via_maker, ..
            },
        ) = cell.split_for_abstract(schematic_io.flatten_vec());
        let abs_path = atoll_ctx
            .0
            .write()
            .unwrap()
            .cell_cache
            .generate(block, move |_block| cell.finalize_abstract());

        let (abs, paths) = abs_path.get().clone();

        for path in paths {
            for (a, b) in path {
                let (a, b) = (abs.grid_to_track(a), abs.grid_to_track(b));
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

                    if track.width() > 0 && track.height() > 0 {
                        layout.draw(Shape::new(abs.grid.stack.layer(a.layer).id, track))?;
                    }
                } else if a.layer == b.layer + 1 || b.layer == a.layer + 1 {
                    let (a, b) = if b.layer > a.layer { (b, a) } else { (a, b) };
                    let (in_track, out_track) = if abs.grid.stack.layer(a.layer).dir().track_dir()
                        == Dir::Horiz
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
                    if track.width() > 0 && track.height() > 0 {
                        layout.draw(Shape::new(abs.grid.stack.layer(b.layer).id, track))?;
                    }
                    if let Some(maker) = &via_maker {
                        for shape in maker.draw_via(
                            layout.ctx().clone(),
                            TrackCoord {
                                layer: a.layer,
                                x: a.x,
                                y: a.y,
                            },
                        ) {
                            layout.draw(shape)?;
                        }
                    }
                }
            }
        }

        Ok(layout_data)
    }
}
