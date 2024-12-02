//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    ops::{Deref, Index},
};

use arcstr::ArcStr;
pub use codegen::Io;
use layout::{HardwareType as LayoutType, PortGeometry};
use schematic::BundleType as SchematicType;
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
pub trait BundleType: FlatLen + HasNameTree {}
impl<T: FlatLen + HasNameTree> BundleType for T {}

pub trait BundleOfType<B: SignalBundle>: BundleType {
    type Bundle: BundleOf<B>;
}

/// Indicates that a bundle type specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A trait implemented by block input/output interfaces.
// TODO: Remove layout hardware type requirement.
pub trait Io: BundleType + Directed + layout::HardwareType {}
impl<T: BundleType + Directed + layout::HardwareType> Io for T {}

/// A bundle representing an instantiation of a [`Signal`].
pub trait SignalBundle: Clone + BundleOf<Self> {}

impl<T: SignalBundle> Flatten<Self> for T {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Self>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl<T: SignalBundle> FlatLen for T {
    fn len(&self) -> usize {
        1
    }
}

impl<T: SignalBundle> Bundle for T {
    type BundleType = Signal;
}

/// A construct with an associated [`BundleType`].
pub trait Bundle: Send + Sync {
    /// The Rust type of the [`BundleType`] associated with this bundle.
    type BundleType: BundleType;
}

pub trait BundleOf<T: SignalBundle>:
    Bundle<BundleType: BundleOfType<T, Bundle = Self>> + FlatLen + Flatten<T>
{
}
impl<
        S: SignalBundle,
        T: Bundle<BundleType: BundleOfType<S, Bundle = Self>> + FlatLen + Flatten<S>,
    > BundleOf<S> for T
{
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
pub struct ArrayBundle<T> {
    elems: Vec<T>,
    ty_len: usize,
}

// END TYPES

// BEGIN COMMON IO TYPES

// /// The interface to a standard 4-terminal MOSFET.
// #[derive(Debug, Default, Clone)]
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
// pub struct MosIoBundle<T: substrate::types::SignalBundle> {
//     pub d: <Signal as substrate::types::BundleType>::Bundle<T>,
//     pub g: <Signal as substrate::types::BundleType>::Bundle<T>,
//     pub s: <Signal as substrate::types::BundleType>::Bundle<T>,
//     pub b: <Signal as substrate::types::BundleType>::Bundle<T>,
//     ty: MosIo,
// }
//
// impl substrate::types::BundleType for MosIo {
//     type Bundle<B: SignalBundle> = MosIoBundle<B>;
// }
//
// impl<T: substrate::types::Bundle> substrate::types::FlatLen for MosIoBundle<T> {
//     fn len(&self) -> usize {
//         0 + <<Signal as substrate::types::BundleType>::Bundle<T> as FlatLen>::len(&self.d)
//     }
// }
//
// impl<T: SignalBundle> Bundle for MosIoBundle<T> {
//     type SignalBundle = T;
//     type BundleType = MosIo;
// }
//
// impl schematic::Connect for MosIoBundle<schematic::Node> {
//     fn view(&self) -> <Self::BundleType as self::BundleType>::Bundle<schematic::Node> {
//         MosIoBundle {
//             d: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Connect>::view(&self.d),
//             g: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Connect>::view(&self.g),
//             s: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Connect>::view(&self.s),
//             b: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Connect>::view(&self.b),
//             ty: self.ty.clone(),
//         }
//     }
// }
//
// impl schematic::Connect for MosIoBundle<schematic::Terminal> {
//     fn view(&self) -> <Self::BundleType as self::BundleType>::Bundle<schematic::Node> {
//         MosIoBundle {
//             d: <<Signal as substrate::types::BundleType>::Bundle<schematic::Terminal> as schematic::Connect>::view(&self.d),
//             g: <<Signal as substrate::types::BundleType>::Bundle<schematic::Terminal> as schematic::Connect>::view(&self.g),
//             s: <<Signal as substrate::types::BundleType>::Bundle<schematic::Terminal> as schematic::Connect>::view(&self.s),
//             b: <<Signal as substrate::types::BundleType>::Bundle<schematic::Terminal> as schematic::Connect>::view(&self.b),
//             ty: self.ty.clone(),
//         }
//     }
// }
//
// impl schematic::Bundle for MosIoBundle<schematic::Node> {
//     fn nested_view(
//         &self,
//         parent: &InstancePath,
//     ) -> <Self::BundleType as self::BundleType>::Bundle<
//         <<Self as self::Bundle>::SignalBundle as schematic::SignalBundle>::NestedSignal,
//     > {
//         MosIoBundle {
//             d: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Bundle>::nested_view(&self.d),
//             g: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Bundle>::nested_view(&self.g),
//             s: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Bundle>::nested_view(&self.s),
//             b: <<Signal as substrate::types::BundleType>::Bundle<schematic::Node> as schematic::Bundle>::nested_view(&self.b),
//             ty: self.ty.clone(),
//         }
//     }
// }
//
// /// The interface to which simulation testbenches should conform.
// #[derive(Debug, Default, Clone, Io)]
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
// #[derive(Debug, Default, Copy, Clone, Io)]
// pub struct DiffPair {
//     /// The positive signal.
//     pub p: InOut<Signal>,
//     /// The negative signal.
//     pub n: InOut<Signal>,
// }

// END COMMON IO TYPES
