//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    collections::HashMap,
    ops::{Deref, Index},
};

use arcstr::ArcStr;
use geometry::{
    prelude::Bbox,
    rect::Rect,
    transform::{HasTransformedView, Transformation, Transformed},
    union::BoundingUnion,
};
use serde::{Deserialize, Serialize};
use tracing::Level;

use crate::layout::element::NamedPorts;
use crate::{
    block::Block,
    error::Result,
    layout::error::LayoutError,
    pdk::layers::{HasPin, LayerId},
    schematic::{CellId, HasNestedView, InstanceId, InstancePath},
    Io,
};

mod impls;

// BEGIN TRAITS

/// A trait implemented by block input/output interfaces.
pub trait Io: Directed + SchematicType + LayoutType {
    // TODO
}

/// Indicates that a hardware type specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A marker trait indicating that a hardware type does not specify signal directions.
pub trait Undirected {}

/// Flatten a structure into a list.
pub trait Flatten<T>: FlatLen {
    /// Flatten a structure into a list.
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<T>;

    /// Flatten into a [`Vec`].
    fn flatten_vec(&self) -> Vec<T> {
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        self.flatten(&mut vec);
        assert_eq!(vec.len(), len, "Flatten::flatten_vec produced a Vec with an incorrect length: expected {} from FlatLen::len, got {}", len, vec.len());
        vec
    }
}

/// The length of the flattened list.
pub trait FlatLen {
    /// The length of the flattened list.
    fn len(&self) -> usize;
    /// Whether or not the flattened representation is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An object with named flattened components.
pub trait HasNameTree {
    /// Return a tree specifying how nodes contained within this type should be named.
    ///
    /// Important: empty types (i.e. those with a flattened length of 0) must return [`None`].
    /// All non-empty types must return [`Some`].
    fn names(&self) -> Option<Vec<NameTree>>;

    /// Returns a flattened list of node names.
    fn flat_names(&self, root: Option<NameFragment>) -> Vec<NameBuf> {
        self.names()
            .map(|t| NameTree::with_optional_fragment(root, t).flatten())
            .unwrap_or_default()
    }
}

/// A schematic hardware type.
pub trait SchematicType: FlatLen + HasNameTree + Clone {
    /// The **Rust** type representing schematic instances of this **hardware** type.
    type Data: SchematicData + HasNestedView + Clone + Send + Sync;

    /// Instantiates a schematic data struct with populated nodes.
    ///
    /// Must consume exactly [`FlatLen::len`] elements of the node list.
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]);
}

/// A trait indicating that this type can be connected to T.
pub trait Connect<T> {}

/// A layout hardware type.
pub trait LayoutType: FlatLen + HasNameTree + Clone {
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Data: LayoutData;
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Builder: LayoutDataBuilder<Self::Data>;

    /// Instantiates a schematic data struct with populated nodes.
    fn builder(&self) -> Self::Builder;
}

/// A schematic hardware data struct.
///
/// Only intended for use by Substrate procedural macros.
pub trait StructData {
    /// Returns a list of the names of the fields in this struct.
    fn fields(&self) -> Vec<ArcStr>;

    /// Returns the list of nodes contained by the field of the given name.
    fn field_nodes(&self, name: &str) -> Option<Vec<Node>>;
}

/// Schematic hardware data.
///
/// An instance of a [`SchematicType`].
pub trait SchematicData: FlatLen + Flatten<Node> {}
impl<T> SchematicData for T where T: FlatLen + Flatten<Node> {}

/// Layout hardware data.
///
/// An instance of a [`LayoutType`].
pub trait LayoutData: FlatLen + Flatten<PortGeometry> + HasTransformedView + Send + Sync {}
impl<T> LayoutData for T where T: FlatLen + Flatten<PortGeometry> + HasTransformedView + Send + Sync {}

/// Layout hardware data builder.
///
/// A builder for an instance of a [`LayoutData`].
pub trait LayoutDataBuilder<T: LayoutData> {
    /// Builds an instance of [`LayoutData`].
    fn build(self) -> Result<T>;
}

/// A custom layout type that can be derived from an existing layout type.
pub trait CustomLayoutType<T: LayoutType>: LayoutType {
    /// Creates this layout type from another layout type.
    fn from_layout_type(other: &T) -> Self;
}

/// Construct an instance of `Self` hierarchically given a name buffer and a source of type `T`.
pub trait HierarchicalBuildFrom<T> {
    /// Build `self` from the given root path and source.
    fn build_from(&mut self, path: &mut NameBuf, source: &T);

    /// Build `self` from the given source, starting with an empty top-level name buffer.
    fn build_from_top(&mut self, source: &T) {
        let mut buf = NameBuf::new();
        self.build_from(&mut buf, source);
    }

    /// Build `self` from the given source, starting with a top-level name buffer containing the
    /// given name fragment.
    fn build_from_top_prefix(&mut self, prefix: impl Into<NameFragment>, source: &T) {
        let mut buf = NameBuf::new();
        buf.push(prefix);
        self.build_from(&mut buf, source);
    }
}

// END TRAITS

// BEGIN TYPES

/// A portion of a node name.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum NameFragment {
    /// An element identified by a string name, such as a struct field.
    Str(ArcStr),
    /// A numbered element of an array/bus.
    Idx(usize),
}

/// An owned node name, consisting of an ordered list of [`NameFragment`]s.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameBuf {
    fragments: Vec<NameFragment>,
}

/// A tree for hierarchical node naming.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NameTree {
    fragment: Option<NameFragment>,
    children: Vec<NameTree>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An input port of hardware type `T`.
pub struct Input<T: Undirected>(pub T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An output port of hardware type `T`.
pub struct Output<T: Undirected>(pub T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An inout port of hardware type `T`.
pub struct InOut<T: Undirected>(pub T);

/// A type representing a single hardware wire.
#[derive(Debug, Default, Clone, Copy)]
pub struct Signal;

/// A type representing a single hardware layout port with a single [`Shape`](crate::layout::element::Shape) as
/// its geometry.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShapePort;

/// A generic layout port that consists of several shapes.
#[derive(Debug, Default, Clone, Copy)]
pub struct LayoutPort;

/// A single node in a circuit.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Node(u32);

/// A nested node within a cell.
///
/// Created when accessing nodes from instances propagated through data.
#[derive(Clone, Debug)]
pub struct NestedNode {
    pub(crate) node: Node,
    pub(crate) path: InstancePath,
}

/// A path from a top level cell to a nested node.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePath {
    pub(crate) node: Node,
    pub(crate) path: Vec<InstanceId>,
    pub(crate) top: CellId,
}

impl NestedNode {
    /// Returns the path to this node.
    pub fn path(&self) -> NodePath {
        NodePath {
            node: self.node,
            path: self.path.path.iter().copied().collect(),
            top: self.path.top,
        }
    }
}

impl From<NestedNode> for NodePath {
    fn from(value: NestedNode) -> Self {
        value.path()
    }
}

impl From<&NestedNode> for NodePath {
    fn from(value: &NestedNode) -> Self {
        value.path()
    }
}

/// The priority a node has in determining the name of a merged node.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) enum NodePriority {
    /// An IO / externally-visible signal name.
    ///
    /// Has the highest priority in determining node names.
    Io = 3,
    /// An explicitly named signal.
    Named = 2,
    /// A signal with an automatically-generated name.
    ///
    /// Has the lowest priority in determining node names.
    Auto = 1,
}

/// The value associated to a node in a schematic builder's union find data structure.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[doc(hidden)]
pub struct NodeUfValue {
    /// The overall priority of a set of merged nodes.
    ///
    /// Taken to be the highest among priorities of all nodes
    /// in the merged set.
    priority: NodePriority,
    /// The node that provides `priority`.
    ///
    /// For example, if priority is NodePriority::Io, `node`
    /// should be the node identifier representing the IO node.
    pub(crate) source: Node,
}

/// A node unification table for connectivity management.
pub type NodeUf = ena::unify::InPlaceUnificationTable<Node>;

impl ena::unify::UnifyValue for NodeUfValue {
    type Error = ena::unify::NoError;

    fn unify_values(value1: &Self, value2: &Self) -> std::result::Result<Self, Self::Error> {
        if value1.priority == NodePriority::Io
            && value2.priority == NodePriority::Io
            && value1.source != value2.source
        {
            panic!("shorted IOs are not supported")
        }
        Ok(if value1.priority >= value2.priority {
            *value1
        } else {
            *value2
        })
    }
}

impl ena::unify::UnifyKey for Node {
    type Value = Option<NodeUfValue>;
    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "Node"
    }
}

pub(crate) struct NodeContext {
    uf: NodeUf,
}

impl NodeContext {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            uf: Default::default(),
        }
    }
    pub(crate) fn node(&mut self, priority: NodePriority) -> Node {
        let id = self.uf.new_key(Default::default());
        self.uf.union_value(
            id,
            Some(NodeUfValue {
                priority,
                source: id,
            }),
        );
        id
    }
    #[inline]
    pub fn into_inner(self) -> NodeUf {
        self.uf
    }
    pub fn nodes(&mut self, n: usize, priority: NodePriority) -> Vec<Node> {
        (0..n).map(|_| self.node(priority)).collect()
    }
    pub(crate) fn connect(&mut self, n1: Node, n2: Node) {
        self.uf.union(n1, n2);
    }
}

/// A layer ID that describes where the components of an [`IoShape`] are drawn.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct IoLayerId {
    drawing: LayerId,
    pin: LayerId,
    label: LayerId,
}

impl HasPin for IoLayerId {
    fn drawing(&self) -> LayerId {
        self.drawing
    }
    fn pin(&self) -> LayerId {
        self.pin
    }
    fn label(&self) -> LayerId {
        self.label
    }
}

/// A shape used to describe the geometry of a port.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IoShape {
    layer: IoLayerId,
    shape: geometry::shape::Shape,
}

impl Bbox for IoShape {
    fn bbox(&self) -> Option<Rect> {
        self.shape.bbox()
    }
}

impl IoShape {
    /// Creates a new [`IoShape`] from a full specification of the layers on which it should be
    /// drawn.
    pub fn new(
        drawing: impl AsRef<LayerId>,
        pin: impl AsRef<LayerId>,
        label: impl AsRef<LayerId>,
        shape: impl Into<geometry::shape::Shape>,
    ) -> Self {
        Self {
            layer: IoLayerId {
                drawing: *drawing.as_ref(),
                pin: *pin.as_ref(),
                label: *label.as_ref(),
            },
            shape: shape.into(),
        }
    }

    /// Creates a new [`IoShape`] based on the layers specified in `layers`.
    pub fn with_layers(layers: impl HasPin, shape: impl Into<geometry::shape::Shape>) -> Self {
        Self {
            layer: IoLayerId {
                drawing: layers.drawing(),
                pin: layers.pin(),
                label: layers.label(),
            },
            shape: shape.into(),
        }
    }

    /// Returns the underlying [`Shape`](geometry::shape::Shape) of `self`.
    pub fn shape(&self) -> &geometry::shape::Shape {
        &self.shape
    }

    /// Returns the [`IoLayerId`] of `self`.
    pub fn layer(&self) -> IoLayerId {
        self.layer
    }
}

impl<T: Bbox> BoundingUnion<T> for IoShape {
    type Output = Rect;

    fn bounding_union(&self, other: &T) -> Self::Output {
        self.bbox().unwrap().bounding_union(&other.bbox())
    }
}

impl HasTransformedView for IoShape {
    type TransformedView<'a> = IoShape;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView<'_> {
        IoShape {
            shape: self.shape.transformed_view(trans),
            ..*self
        }
    }
}

/// A layout port with a generic set of associated geometry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct PortGeometry {
    /// The primary shape of the port.
    ///
    /// **Not** contained in `named_shapes` or `unnamed_shapes`.
    pub primary: IoShape,
    /// A set of unnamed shapes contained by the port.
    pub unnamed_shapes: Vec<IoShape>,
    /// A set of named shapes contained by the port.
    pub named_shapes: HashMap<ArcStr, IoShape>,
}

/// A set of transformed geometry associated with a layout port.
#[allow(dead_code)]
#[derive(Clone)]
pub struct TransformedPortGeometry<'a> {
    /// The primary shape of the port.
    ///
    /// This field is a copy of a shape contained in one of the other fields, so it is not drawn
    /// explicitly. It is kept separately for ease of access.
    pub primary: IoShape,
    /// A set of unnamed shapes contained by the port.
    pub unnamed_shapes: Transformed<'a, [IoShape]>,
    /// A set of named shapes contained by the port.
    pub named_shapes: Transformed<'a, HashMap<ArcStr, IoShape>>,
}

/// A set of geometry associated with a layout port.
#[derive(Clone, Debug, Default)]
pub struct PortGeometryBuilder {
    primary: Option<IoShape>,
    unnamed_shapes: Vec<IoShape>,
    named_shapes: HashMap<ArcStr, IoShape>,
}

impl PortGeometryBuilder {
    /// Push an unnamed shape to the port.
    ///
    /// If the primary shape has not been set yet, sets the primary shape to the new shape
    /// **instead** of adding to the unnamed shapes list.
    ///
    /// The primary shape can be overriden using [`PortGeometryBuilder::set_primary`].
    pub fn push(&mut self, shape: IoShape) {
        if self.primary.is_none() {
            self.primary = Some(shape.clone());
        } else {
            self.unnamed_shapes.push(shape);
        }
    }

    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes.
    pub fn merge(&mut self, other: impl Into<PortGeometry>) {
        let other = other.into();
        self.primary = Some(other.primary);
        self.unnamed_shapes.extend(other.unnamed_shapes);
        self.named_shapes.extend(other.named_shapes);
    }

    /// Sets the primary shape of this port.
    pub fn set_primary(&mut self, shape: IoShape) {
        self.primary = Some(shape);
    }
}

/// A simple builder that allows setting data at runtime.
///
/// ```
/// # use substrate::io::OptionBuilder;
/// let mut builder = OptionBuilder::default();
/// builder.set(5);
/// assert_eq!(builder.build().unwrap(), 5);
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct OptionBuilder<T>(Option<T>);

impl<T> Default for OptionBuilder<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> OptionBuilder<T> {
    /// Constructs a new, empty `OptionBuilder`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value of the data contained by the builder.
    pub fn set(&mut self, inner: T) {
        let _ = self.0.insert(inner);
    }

    /// Returns the data contained by the builder.
    pub fn build(self) -> Result<T> {
        Ok(self.0.ok_or(LayoutError::IoDefinition)?)
    }
}

/// Port directions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Input.
    Input,
    /// Output.
    Output,
    /// Input or output.
    ///
    /// Represents ports whose direction is not known
    /// at generator elaboration time.
    #[default]
    InOut,
}

impl Direction {
    /// Returns the flipped direction.
    ///
    /// [`Direction::InOut`] is unchanged by flipping.
    ///
    /// # Examples
    ///
    /// ```
    /// use substrate::io::Direction;
    /// assert_eq!(Direction::Input.flip(), Direction::Output);
    /// assert_eq!(Direction::Output.flip(), Direction::Input);
    /// assert_eq!(Direction::InOut.flip(), Direction::InOut);
    /// ```
    #[inline]
    pub fn flip(&self) -> Self {
        match *self {
            Self::Input => Self::Output,
            Self::Output => Self::Input,
            Self::InOut => Self::InOut,
        }
    }
}

/// A signal exposed by a cell.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    direction: Direction,
    node: Node,
}

/// An array containing some number of elements of type `T`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Array<T> {
    len: usize,
    ty: T,
}

impl<T> Array<T> {
    /// Create a new array of the given length and hardware type.
    #[inline]
    pub fn new(len: usize, ty: T) -> Self {
        Self { len, ty }
    }
}

/// An instantiated array containing a fixed number of elements of type `T`.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArrayData<T> {
    elems: Vec<T>,
    ty_len: usize,
}

// END TYPES

// BEGIN COMMON IO TYPES

/// The interface to a standard 4-terminal MOSFET.
#[derive(Debug, Default, Clone, Io)]
pub struct MosIo {
    /// The drain.
    pub d: InOut<Signal>,
    /// The gate.
    pub g: Input<Signal>,
    /// The source.
    pub s: InOut<Signal>,
    /// The body.
    pub b: InOut<Signal>,
}

/// A trait indicating that a block is a standard 4 terminal MOSFET.
pub trait Mos: Block<Io = MosIo> {}

impl<T> Mos for T where T: Block<Io = MosIo> {}

/// The interface to which simulation testbenches should conform.
/// TODO: Add trait bound to ensure testbenches have this IO, need to refactor crates.
#[derive(Debug, Default, Clone, Io)]
pub struct TestbenchIo {
    /// The global ground net.
    pub vss: InOut<Signal>,
}

/// The interface for 2-terminal blocks.
#[derive(Debug, Default, Clone, Io)]
pub struct TwoTerminalIo {
    /// The positive terminal.
    pub p: InOut<Signal>,
    /// The negative terminal.
    pub n: InOut<Signal>,
}

// END COMMON IO TYPES
