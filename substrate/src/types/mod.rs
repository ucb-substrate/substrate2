//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::{Deref, Index},
};

use arcstr::ArcStr;
pub use codegen::Io;
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

/// The length of the flattened list.
pub trait FlatLen {
    /// The length of the flattened list.
    fn len(&self) -> usize;
    /// Whether or not the flattened representation is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: FlatLen> FlatLen for &T {
    fn len(&self) -> usize {
        (*self).len()
    }
}

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

impl<S, T: Flatten<S>> Flatten<S> for &T {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<S>,
    {
        (*self).flatten(output)
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

/// A bundle type.
pub trait BundleType:
    HasNameTree + HasBundleType<BundleType = Self> + Debug + Clone + Eq + Send + Sync
{
    /// An associated bundle type that allows swapping in any [`BundlePrimitive`].
    type Bundle<B: BundlePrimitive>: Bundle<BundleType = Self> + BundleOf<B>;
}

/// A bundle type with an associated bundle `Bundle` of `B`.
pub trait HasBundleOf<B: BundlePrimitive>: BundleType {
    /// The bundle of primitive `B` associated with this bundle type.
    type Bundle: Bundle<BundleType = Self> + BundleOf<B>;
}

impl<B: BundlePrimitive, T: BundleType> HasBundleOf<B> for T {
    type Bundle = <T as BundleType>::Bundle<B>;
}

/// Indicates that a bundle type specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A trait implemented by block input/output interfaces.
// TODO: Remove layout hardware type requirement.
pub trait Io: Directed + HasBundleType + Clone {}
impl<T: Directed + HasBundleType + Clone> Io for T {}

/// A bundle primitive representing an instantiation of a [`Signal`].
pub trait BundlePrimitive: Clone + Bundle<BundleType = Signal> + BundleOf<Self> {}

/// A construct with an associated [`BundleType`].
pub trait HasBundleType {
    /// The Rust type of the [`BundleType`] associated with this bundle.
    type BundleType: BundleType;

    /// Returns the [`BundleType`] of this bundle.
    fn ty(&self) -> Self::BundleType;
}

impl<T: HasBundleType> HasBundleType for &T {
    type BundleType = T::BundleType;

    fn ty(&self) -> Self::BundleType {
        (*self).ty()
    }
}

/// A bundle of hardware wires.
pub trait Bundle: HasBundleType + Send + Sync {}
impl<T: HasBundleType + Send + Sync> Bundle for T {}

/// A bundle that is made up of primitive `T`.
pub trait BundleOf<T: BundlePrimitive>: Bundle + FlatLen + Flatten<T> {}
impl<S: BundlePrimitive, T: Bundle + FlatLen + Flatten<S>> BundleOf<S> for T {}

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

/// An input port of hardware type `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`Input`](Direction::Input)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Input<T>(pub T);

/// An output port of hardware type `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`Output`](Direction::Output)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Output<T>(pub T);

/// An inout port of hardware type `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`InOut`](Direction::InOut)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct InOut<T>(pub T);

/// Flip the direction of all ports in `T`
///
/// See [`Direction::flip`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Flipped<T>(pub T);

/// A type representing a single hardware wire.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
pub struct ArrayBundle<T: Bundle> {
    elems: Vec<T>,
    ty: T::BundleType,
}

// END TYPES

// BEGIN COMMON IO TYPES

// /// The interface to a standard 4-terminal MOSFET.
// #[derive(Debug, Default, Clone, Io)]
// pub struct MosIo {
//     /// The drain.
//     pub d: InOut<Signal>,
//     /// The gate.
//     pub g: Input<Signal>,
//     /// The source.
//     pub s: InOut<Signal>,
//     /// The body.
//     pub b: InOut<Signal>,
// }
//
// /// The interface to which simulation testbenches should conform.
// #[derive(Debug, Default, Clone, Io, PartialEq, Eq)]
// pub struct TestbenchIo {
//     /// The global ground net.
//     pub vss: InOut<Signal>,
// }
//
// /// The interface for 2-terminal blocks.
// #[derive(Debug, Default, Clone, Io)]
// pub struct TwoTerminalIo {
//     /// The positive terminal.
//     pub p: InOut<Signal>,
//     /// The negative terminal.
//     pub n: InOut<Signal>,
// }
//
// /// The interface for VDD and VSS rails.
// #[derive(Debug, Default, Clone, Io)]
// pub struct PowerIo {
//     /// The VDD rail.
//     pub vdd: InOut<Signal>,
//     /// The VSS rail.
//     pub vss: InOut<Signal>,
// }
//
// /// A pair of differential signals.
// // TODO: Create proc macro for defining un-directioned (non-IO) bundle types directly.
// #[derive(Debug, Default, Copy, Clone, Io)]
// pub struct DiffPair {
//     /// The positive signal.
//     pub p: InOut<Signal>,
//     /// The negative signal.
//     pub n: InOut<Signal>,
// }

// END COMMON IO TYPES
