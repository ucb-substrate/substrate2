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
    block::Block,
    error::Result,
    schematic::{CellId, InstanceId, InstancePath},
};

pub use crate::scir::Direction;

mod impls;
pub mod layout;
pub mod schematic;

/// The [`BundleKind`] of a block's IO.
pub type IoKind<T> = <<T as Block>::Io as HasBundleKind>::BundleKind;

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

/// A bundle kind.
pub trait BundleKind:
    HasNameTree + HasBundleKind<BundleKind = Self> + Debug + Clone + PartialEq + Send + Sync
{
}
impl<T: HasNameTree + HasBundleKind<BundleKind = T> + Debug + Clone + PartialEq + Send + Sync>
    BundleKind for T
{
}

/// Indicates that an IO specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A trait implemented by block input/output interfaces.
pub trait Io: Directed + HasBundleKind + Clone {}
impl<T: Directed + HasBundleKind + Clone> Io for T {}

/// A construct with an associated [`BundleKind`].
pub trait HasBundleKind: Send + Sync {
    /// The Rust type of the [`BundleKind`] associated with this bundle.
    type BundleKind: BundleKind;

    /// Returns the [`BundleKind`] of this bundle.
    fn kind(&self) -> Self::BundleKind;
}

impl<T: HasBundleKind> HasBundleKind for &T {
    type BundleKind = T::BundleKind;

    fn kind(&self) -> Self::BundleKind {
        (*self).kind()
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

/// An input port of kind `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`Input`](Direction::Input)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Input<T>(pub T);

/// An output port of kind `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`Output`](Direction::Output)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Output<T>(pub T);

/// An inout port of kind `T`.
///
/// Recursively overrides the direction of all components of `T` to be [`InOut`](Direction::InOut)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct InOut<T>(pub T);

/// Flip the direction of all ports in `T`
///
/// See [`Direction::flip`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct Flipped<T>(pub T);

/// A type representing a single hardware wire in a [`BundleKind`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Signal;

impl Signal {
    /// Creates a new [`Signal`].
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

/// An array containing some number of elements of kind `T`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Array<T> {
    len: usize,
    kind: T,
}

impl<T> Array<T> {
    /// Create a new array of the given length and bundle kind.
    #[inline]
    pub fn new(len: usize, kind: T) -> Self {
        Self { len, kind }
    }
}

/// An instantiated array containing a fixed number of elements of `T`.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArrayBundle<T: HasBundleKind> {
    elems: Vec<T>,
    kind: T::BundleKind,
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
