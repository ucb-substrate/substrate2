//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    ops::{Deref, Index},
};

use arcstr::ArcStr;
pub use codegen::Io;
use geometry::transform::{HasTransformedView, Transformed};
use layout::{HardwareType as LayoutType, PortGeometry};
use schematic::{HardwareType as SchematicType, Node};
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    schematic::{CellId, InstanceId, InstancePath},
};

pub use crate::scir::Direction;

mod impls;
pub mod layout;
pub mod schematic;

// BEGIN TRAITS

/// A trait implemented by block input/output interfaces.
pub trait Io: Directed + SchematicType + LayoutType {}
impl<T: Directed + SchematicType + LayoutType> Io for T {}

/// Indicates that a hardware type specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

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

/// A schematic hardware data struct.
///
/// Only intended for use by Substrate procedural macros.
pub trait StructData {
    /// Returns a list of the names of the fields in this struct.
    fn fields(&self) -> Vec<ArcStr>;

    /// Returns the list of nodes contained by the field of the given name.
    fn field_nodes(&self, name: &str) -> Option<Vec<Node>>;
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
///
/// Recursively overrides the direction of all components of `T` to be [`Input`](Direction::Input)
pub struct Input<T>(pub T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An output port of hardware type `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`Output`](Direction::Output)
pub struct Output<T>(pub T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An inout port of hardware type `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`InOut`](Direction::InOut)
pub struct InOut<T>(pub T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// Flip the direction of all ports in `T`
///
/// See [`Direction::flip`]
pub struct Flipped<T>(pub T);

/// A type representing a single hardware wire.
#[derive(Debug, Default, Clone, Copy)]
pub struct Signal;

impl Signal {
    /// Creates a new [`Signal`].
    #[inline]
    pub fn new() -> Self {
        Self
    }
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

/// The interface to which simulation testbenches should conform.
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

/// The interface for VDD and VSS rails.
#[derive(Debug, Default, Clone, Io)]
pub struct PowerIo {
    /// The VDD rail.
    pub vdd: InOut<Signal>,
    /// The VSS rail.
    pub vss: InOut<Signal>,
}

/// A pair of differential signals.
#[derive(Debug, Default, Copy, Clone, Io)]
pub struct DiffPair {
    /// The positive signal.
    pub p: InOut<Signal>,
    /// The negative signal.
    pub n: InOut<Signal>,
}

// END COMMON IO TYPES
