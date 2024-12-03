//! Traits and types for defining interfaces and signals in Substrate.

use std::{
    borrow::Borrow,
    ops::{Deref, Index},
};

use arcstr::ArcStr;
pub use codegen::Io;
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    schematic::{CellId, HasNestedView, InstanceId, InstancePath},
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
pub trait BundleType: FlatLen + HasNameTree {}
impl<T: FlatLen + HasNameTree> BundleType for T {}

/// A bundle type with an associated bundle `Bundle` of `B`.
pub trait BundleOfType<B: BundlePrimitive>: BundleType {
    /// The bundle of primitive `B` associated with this bundle type.
    type Bundle: BundleOf<B>;
}

/// Indicates that a bundle type specifies signal directions for all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A trait implemented by block input/output interfaces.
// TODO: Remove layout hardware type requirement.
pub trait Io: BundleType + Directed + layout::HardwareType {}
impl<T: BundleType + Directed + layout::HardwareType> Io for T {}

/// A bundle primitive representing an instantiation of a [`Signal`].
pub trait BundlePrimitive: Clone + Bundle<BundleType = Signal> + BundleOf<Self> {}

/// A construct with an associated [`BundleType`].
pub trait Bundle: Send + Sync {
    /// The Rust type of the [`BundleType`] associated with this bundle.
    type BundleType: BundleType;
}

impl<T: Bundle> Bundle for &T {
    type BundleType = T::BundleType;
}

/// A bundle that is made up of primitive `T`.
pub trait BundleOf<T: BundlePrimitive>:
    Bundle<BundleType: BundleOfType<T, Bundle = Self>> + FlatLen + Flatten<T>
{
}
impl<
        S: BundlePrimitive,
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

/// The interface to a standard 4-terminal MOSFET.
#[derive(Debug, Default, Clone)]
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

impl FlatLen for MosIo {
    fn len(&self) -> usize {
        4
    }
}

impl Flatten<Direction> for MosIo {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend([
            Direction::InOut,
            Direction::Input,
            Direction::InOut,
            Direction::InOut,
        ]);
    }
}

impl HasNameTree for MosIo {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(
            ["d", "g", "s", "b"]
                .iter()
                .map(|name| NameTree {
                    fragment: Some(NameFragment::Str(ArcStr::from(*name))),
                    children: vec![],
                })
                .collect(),
        )
    }
}

impl<B: BundlePrimitive> BundleOfType<B> for MosIo {
    type Bundle = MosIoBundle<B>;
}

impl schematic::BundleType for MosIo {
    fn instantiate<'n>(
        &self,
        ids: &'n [schematic::Node],
    ) -> (
        <Self as schematic::BundleOfType<schematic::Node>>::Bundle,
        &'n [schematic::Node],
    ) {
        if let [d, g, s, b, rest @ ..] = ids {
            (
                MosIoBundle {
                    d: *d,
                    g: *g,
                    s: *s,
                    b: *b,
                },
                rest,
            )
        } else {
            unreachable!();
        }
    }
    fn terminal_view(
        cell: CellId,
        cell_io: &<Self as schematic::BundleOfType<schematic::Node>>::Bundle,
        instance: InstanceId,
        instance_io: &<Self as schematic::BundleOfType<schematic::Node>>::Bundle,
    ) -> <Self as schematic::BundleOfType<schematic::Terminal>>::Bundle {
        MosIoBundle {
            d: <Signal as schematic::BundleType>::terminal_view(
                cell,
                &cell_io.d,
                instance,
                &instance_io.d,
            ),
            g: <Signal as schematic::BundleType>::terminal_view(
                cell,
                &cell_io.d,
                instance,
                &instance_io.d,
            ),
            s: <Signal as schematic::BundleType>::terminal_view(
                cell,
                &cell_io.d,
                instance,
                &instance_io.d,
            ),
            b: <Signal as schematic::BundleType>::terminal_view(
                cell,
                &cell_io.d,
                instance,
                &instance_io.d,
            ),
        }
    }
}

pub struct MosIoBundle<T: BundlePrimitive> {
    pub d: <Signal as BundleOfType<T>>::Bundle,
    pub g: <Signal as BundleOfType<T>>::Bundle,
    pub s: <Signal as BundleOfType<T>>::Bundle,
    pub b: <Signal as BundleOfType<T>>::Bundle,
}

impl<T: substrate::types::BundlePrimitive> substrate::types::FlatLen for MosIoBundle<T> {
    fn len(&self) -> usize {
        4
    }
}

impl<T: BundlePrimitive> Bundle for MosIoBundle<T> {
    type BundleType = MosIo;
}

impl<B: BundlePrimitive> Flatten<B> for MosIoBundle<B> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<B>,
    {
        Flatten::<B>::flatten(&self.d, output);
        Flatten::<B>::flatten(&self.g, output);
        Flatten::<B>::flatten(&self.s, output);
        Flatten::<B>::flatten(&self.b, output);
    }
}

impl schematic::Connect for MosIoBundle<schematic::Node> {
    fn view(
        &self,
    ) -> <<Self as schematic::Bundle>::BundleType as schematic::BundleOfType<schematic::Node>>::Bundle
    {
        MosIoBundle {
            d: <<Signal as substrate::types::BundleOfType<schematic::Node>>::Bundle as schematic::Connect>::view(&self.d),
            g: <<Signal as substrate::types::BundleOfType<schematic::Node>>::Bundle as schematic::Connect>::view(&self.g),
            s: <<Signal as substrate::types::BundleOfType<schematic::Node>>::Bundle as schematic::Connect>::view(&self.s),
            b: <<Signal as substrate::types::BundleOfType<schematic::Node>>::Bundle as schematic::Connect>::view(&self.b),
        }
    }
}

impl schematic::Connect for MosIoBundle<schematic::Terminal> {
    fn view(
        &self,
    ) -> <<Self as schematic::Bundle>::BundleType as schematic::BundleOfType<schematic::Node>>::Bundle
    {
        MosIoBundle {
            d: <<Signal as substrate::types::BundleOfType<schematic::Terminal>>::Bundle as schematic::Connect>::view(&self.d),
            g: <<Signal as substrate::types::BundleOfType<schematic::Terminal>>::Bundle as schematic::Connect>::view(&self.g),
            s: <<Signal as substrate::types::BundleOfType<schematic::Terminal>>::Bundle as schematic::Connect>::view(&self.s),
            b: <<Signal as substrate::types::BundleOfType<schematic::Terminal>>::Bundle as schematic::Connect>::view(&self.b),
        }
    }
}

impl<T: schematic::BundlePrimitive> HasNestedView for MosIoBundle<T> {
    type NestedView = MosIoBundle<<T as HasNestedView>::NestedView>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        MosIoBundle {
            d: <<Signal as substrate::types::BundleOfType<T>>::Bundle as HasNestedView>::nested_view(&self.d, parent),
            g: <<Signal as substrate::types::BundleOfType<T>>::Bundle as HasNestedView>::nested_view(&self.g, parent),
            s: <<Signal as substrate::types::BundleOfType<T>>::Bundle as HasNestedView>::nested_view(&self.s, parent),
            b: <<Signal as substrate::types::BundleOfType<T>>::Bundle as HasNestedView>::nested_view(&self.b, parent),
        }
    }
}

/// The interface to which simulation testbenches should conform.
#[derive(Debug, Default, Clone, Io)]
pub struct TestbenchIo {
    /// The global ground net.
    pub vss: InOut<Signal>,
}

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
