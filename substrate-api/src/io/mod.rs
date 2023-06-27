//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    ops::{Deref, Index},
};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use tracing::Level;

use crate::{
    error::Result,
    layout::{element::Shape, error::LayoutError},
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

/// A schematic hardware type.
pub trait SchematicType: FlatLen + Clone {
    /// The **Rust** type representing schematic instances of this **hardware** type.
    type Data: SchematicData;

    /// Instantiates a schematic data struct with populated nodes.
    ///
    /// Must consume exactly [`FlatLen::len`] elements of the node list.
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]);
}

/// A trait indicating that this type can be connected to T.
pub trait Connect<T> {}

/// A layout hardware type.
pub trait LayoutType: FlatLen + Clone {
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Data: LayoutData;
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Builder: LayoutDataBuilder<Self::Data>;

    /// Instantiates a schematic data struct with populated nodes.
    fn builder(&self) -> Self::Builder;
}

/// Schematic hardware data.
///
/// An instance of a [`SchematicType`].
pub trait SchematicData: FlatLen + Flatten<Node> {}
impl<T> SchematicData for T where T: FlatLen + Flatten<Node> {}

/// Layout hardware data.
///
/// An instance of a [`LayoutType`].
pub trait LayoutData: FlatLen + Flatten<PortGeometry> {}
impl<T> LayoutData for T where T: FlatLen + Flatten<PortGeometry> {}

/// Layout hardware data builder.
///
/// A builder for an instance of a [`LayoutData`].
pub trait LayoutDataBuilder<T: LayoutData>: FlatLen {
    /// Builds an instance of [`LayoutData`].
    fn build(self) -> Result<T>;
}

// END TRAITS

// BEGIN TYPES

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

/// A single node in a circuit.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Node(u32);

/// A collection of [`Node`]s.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct NodeSet(HashSet<Node>);

/// A node unification table for connectivity management.
pub type NodeUf = ena::unify::InPlaceUnificationTable<Node>;

impl ena::unify::UnifyValue for NodeSet {
    type Error = ena::unify::NoError;

    fn unify_values(value1: &Self, value2: &Self) -> std::result::Result<Self, Self::Error> {
        Ok(Self(&value1.0 | &value2.0))
    }
}

impl ena::unify::UnifyKey for Node {
    type Value = NodeSet;
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
    pub(crate) fn node(&mut self) -> Node {
        let id = self.uf.new_key(Default::default());
        self.uf.union_value(id, NodeSet([id].into()));
        id
    }
    #[inline]
    pub fn into_inner(self) -> NodeUf {
        self.uf
    }
    pub fn nodes(&mut self, n: usize) -> Vec<Node> {
        (0..n).map(|_| self.node()).collect()
    }
    pub(crate) fn connect(&mut self, n1: Node, n2: Node) {
        self.uf.union(n1, n2);
    }
}

/// A set of geometry associated with a layout port.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PortGeometry {
    /// The primary shape of the port.
    ///
    /// This field is a copy of a shape contained in one of the other fields, so it is not drawn
    /// explicitly. It is kept separately for ease of access.
    primary: Shape,
    unnamed_shapes: Vec<Shape>,
    named_shapes: HashMap<ArcStr, Shape>,
}

/// A set of geometry associated with a layout port.
#[derive(Clone, Debug, Default)]
pub struct PortGeometryBuilder {
    primary: Option<Shape>,
    unnamed_shapes: Vec<Shape>,
    named_shapes: HashMap<ArcStr, Shape>,
}

impl PortGeometryBuilder {
    /// Push an unnamed shape to the port.
    ///
    /// If the primary shape has not been set yet, sets the primary shape to the new shape. This
    /// can be overriden using [`PortGeometryBuilder::set_primary`].
    pub fn push(&mut self, shape: Shape) {
        if self.primary.is_none() {
            self.primary = Some(shape.clone());
        }
        self.unnamed_shapes.push(shape);
    }

    /// Sets the primary shape of this port.
    pub fn set_primary(&mut self, shape: Shape) {
        self.primary = Some(shape);
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
pub struct Port {
    direction: Direction,
    node: Node,
}

/// An array containing some number of elements of type `T`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct ArrayData<T> {
    elems: Vec<T>,
    ty_len: usize,
}

// END TYPES
