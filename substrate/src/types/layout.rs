//! Traits and types for layout IOs.

use super::{
    FlatLen, Flatten, HasBundleKind, HasNameTree, NameBuf, NameFragment, NameTree, Signal,
};
use crate::error::Result;
use crate::layout::element::NamedPorts;
use crate::layout::error::LayoutError;
use crate::layout::schema::Schema;
use arcstr::ArcStr;
pub use codegen::LayoutType as HardwareType;
use geometry::point::Point;
use geometry::prelude::{Bbox, Transformation};
use geometry::rect::Rect;
use geometry::transform::{TransformRef, TranslateRef};
use geometry::union::BoundingUnion;
use layir::LayerBbox;
use layir::Shape;
use std::collections::HashMap;
use std::marker::PhantomData;
use tracing::Level;

/// A layout hardware type.
pub trait LayoutBundleKind<S: Schema>: super::BundleKind + FlatLen + HasNameTree + Clone {
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Bundle: LayoutBundle<S>;
    /// A builder for creating [`HardwareType::Bundle`].
    type Builder: LayoutBundleBuilder<S, Self::Bundle>;

    /// Creates an instance of the builder of the associated type.
    fn builder(&self) -> Self::Builder;
}

pub trait HasLayoutBundleKind<S: Schema>: super::HasBundleKind {
    type BundleKind: LayoutBundleKind<S>;
    /// Creates an instance of the builder of the associated type.
    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder;
}

/// A layout IO type.
pub trait Io<S: Schema>: super::Io + HasLayoutBundleKind<S> {}
impl<S: Schema, T: super::Io + HasLayoutBundleKind<S>> Io<S> for T {}

/// The associated bundle of a layout bundle kind.
pub type SchemaBundle<S, T> = <T as LayoutBundleKind<S>>::Bundle;

/// The associated builder of a layout type.
pub type Builder<S, T> = <T as LayoutBundleKind<S>>::Builder;

/// Layout hardware data builder.
///
/// A builder for an instance of bundle `T`.
pub trait LayoutBundleBuilder<S: Schema, T: LayoutBundle<S>> {
    /// Builds an instance of bundle `T`.
    fn build(self) -> Result<T>;
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

/// A type representing a single hardware layout port with a single [`Shape`](crate::layout::element::Shape) as
/// its geometry.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ShapePort<L>(PhantomData<L>);

impl<L> Default for ShapePort<L> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<L> ShapePort<L> {
    pub fn new() -> Self {
        Default::default()
    }
}

/// A generic layout port that consists of several shapes.
#[derive(Debug, Clone, Copy)]
pub struct Port<L>(PhantomData<L>);

impl<L> Default for Port<L> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<L> Port<L> {
    pub fn new() -> Self {
        Default::default()
    }
}

/// A layout port with a generic set of associated geometry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct PortGeometry<L> {
    /// The primary shape of the port.
    ///
    /// **Not** contained in `named_shapes` or `unnamed_shapes`.
    pub primary: Shape<L>,
    /// A set of unnamed shapes contained by the port.
    pub unnamed_shapes: Vec<Shape<L>>,
    /// A set of named shapes contained by the port.
    pub named_shapes: HashMap<ArcStr, Shape<L>>,
}

impl<L> PortGeometry<L> {
    /// Returns an iterator over all shapes in a [`PortGeometry`].
    pub fn shapes(&self) -> impl Iterator<Item = &Shape<L>> {
        std::iter::once(&self.primary)
            .chain(self.unnamed_shapes.iter())
            .chain(self.named_shapes.values())
    }

    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes
    /// and moving their old values to the collection of unnamed shapes.
    pub(crate) fn merge(&mut self, other: impl Into<PortGeometry<L>>) {
        let mut other = other.into();
        std::mem::swap(&mut self.primary, &mut other.primary);
        self.unnamed_shapes.push(other.primary);
        self.unnamed_shapes.extend(other.unnamed_shapes);
        for (name, shape) in other.named_shapes {
            if let Some(old_shape) = self.named_shapes.insert(name, shape) {
                self.unnamed_shapes.push(old_shape);
            }
        }
    }
}

impl<L> Bbox for PortGeometry<L> {
    fn bbox(&self) -> Option<Rect> {
        self.shapes().fold(None, |a, b| a.bounding_union(&b.bbox()))
    }
}

impl<S: Schema> HasLayoutBundleKind<S> for PortGeometry<S::Layer> {
    type BundleKind = Signal;

    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
        PortGeometryBuilder::default()
    }
}
impl<L: Send + Sync> super::HasBundleKind for PortGeometry<L> {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

/// A type that can be a bundle of layout ports.
///
/// An instance of a [`BundleKind`].
pub trait LayoutBundle<S: Schema>:
    HasLayoutBundleKind<S> + FlatLen + Flatten<PortGeometry<S::Layer>> + TransformRef + Send + Sync
{
}

impl<S, T> LayoutBundle<S> for T
where
    S: Schema,
    T: HasLayoutBundleKind<S>
        + FlatLen
        + Flatten<PortGeometry<S::Layer>>
        + TransformRef
        + Send
        + Sync,
{
}

/// A set of geometry associated with a layout port.
#[derive(Clone, Debug)]
pub struct PortGeometryBuilder<L> {
    primary: Option<Shape<L>>,
    unnamed_shapes: Vec<Shape<L>>,
    named_shapes: HashMap<ArcStr, Shape<L>>,
}

impl<L> Default for PortGeometryBuilder<L> {
    fn default() -> Self {
        Self {
            primary: None,
            unnamed_shapes: Vec::new(),
            named_shapes: HashMap::new(),
        }
    }
}

impl<L: Clone> PortGeometryBuilder<L> {
    /// Push an unnamed shape to the port.
    ///
    /// If the primary shape has not been set yet, sets the primary shape to the new shape
    /// **instead** of adding to the unnamed shapes list.
    ///
    /// The primary shape can be overriden using [`PortGeometryBuilder::set_primary`].
    pub fn push(&mut self, shape: Shape<L>) {
        if self.primary.is_none() {
            self.primary = Some(shape.clone());
        } else {
            self.unnamed_shapes.push(shape);
        }
    }
}

impl<L> PortGeometryBuilder<L> {
    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes
    /// and moving their old values to the collection of unnamed shapes.
    pub fn merge(&mut self, other: impl Into<PortGeometry<L>>) {
        let other = other.into();
        if let Some(old_primary) = self.primary.take() {
            self.unnamed_shapes.push(old_primary);
        }
        self.primary = Some(other.primary);
        self.unnamed_shapes.extend(other.unnamed_shapes);
        for (name, shape) in other.named_shapes {
            if let Some(old_shape) = self.named_shapes.insert(name, shape) {
                self.unnamed_shapes.push(old_shape);
            }
        }
    }

    /// Sets the primary shape of this port, moving the current primary
    /// to the set of unnamed shapes.
    pub fn set_primary(&mut self, shape: Shape<L>) {
        let old_primary = self.primary.take();
        self.primary = Some(shape);
        if let Some(old_primary) = old_primary {
            self.unnamed_shapes.push(old_primary);
        }
    }
}

/// A simple builder that allows setting data at runtime.
///
/// ```
/// # use substrate::types::layout::OptionBuilder;
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

impl<L> FlatLen for ShapePort<L> {
    fn len(&self) -> usize {
        1
    }
}

impl<S: Schema> HasBundleKind for ShapePort<S::Layer> {
    type BundleKind = Signal;
    fn kind(&self) -> <Self as HasBundleKind>::BundleKind {
        Signal
    }
}

impl<S: Schema> HasLayoutBundleKind<S> for ShapePort<S::Layer> {
    type BundleKind = Signal;
    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
        PortGeometryBuilder::default()
    }
}

impl<L> HasNameTree for ShapePort<L> {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl<L> FlatLen for Port<L> {
    fn len(&self) -> usize {
        1
    }
}

impl<S: Schema> LayoutBundleKind<S> for Port<S::Layer> {
    type Bundle = PortGeometry<S::Layer>;
    type Builder = PortGeometryBuilder<S::Layer>;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl<L> HasNameTree for Port<L> {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl<L> FlatLen for Shape<L> {
    fn len(&self) -> usize {
        1
    }
}

impl<L: Clone> Flatten<PortGeometry<L>> for Shape<L> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry<L>>,
    {
        output.extend(std::iter::once(PortGeometry {
            primary: self.clone(),
            unnamed_shapes: Vec::new(),
            named_shapes: HashMap::new(),
        }));
    }
}

impl<L: Clone> HierarchicalBuildFrom<NamedPorts<L>> for OptionBuilder<Shape<L>> {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts<L>) {
        self.set(source.get(path).unwrap().primary.clone());
    }
}

impl<T: super::HasBundleKind> super::HasBundleKind for OptionBuilder<T> {
    type BundleKind = <T as super::HasBundleKind>::BundleKind;
    fn kind(&self) -> Self::BundleKind {
        self.0.as_ref().map(|k| k.kind())
    }
}

impl<S: Schema, T: LayoutBundle<S>> LayoutBundleBuilder<S, T> for OptionBuilder<T> {
    fn build(self) -> Result<T> {
        self.build()
    }
}

impl<L> FlatLen for PortGeometry<L> {
    fn len(&self) -> usize {
        1
    }
}

impl<L: Clone> Flatten<PortGeometry<L>> for PortGeometry<L> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry<L>>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl<L: Clone> TranslateRef for PortGeometry<L> {
    fn translate_ref(&self, p: Point) -> Self {
        Self {
            primary: self.primary.translate_ref(p),
            unnamed_shapes: self.unnamed_shapes.translate_ref(p),
            named_shapes: self
                .named_shapes
                .iter()
                .map(|(k, v)| (k.clone(), v.translate_ref(p)))
                .collect(),
        }
    }
}

impl<L: Clone> TransformRef for PortGeometry<L> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        Self {
            primary: self.primary.transform_ref(trans),
            unnamed_shapes: self.unnamed_shapes.transform_ref(trans),
            named_shapes: self
                .named_shapes
                .iter()
                .map(|(k, v)| (k.clone(), v.transform_ref(trans)))
                .collect(),
        }
    }
}

impl<L> FlatLen for PortGeometryBuilder<L> {
    fn len(&self) -> usize {
        1
    }
}

impl<S: Schema> LayoutBundleBuilder<S, PortGeometry<S::Layer>> for PortGeometryBuilder<S::Layer> {
    fn build(self) -> Result<PortGeometry<S::Layer>> {
        Ok(PortGeometry {
            primary: self.primary.ok_or_else(|| {
                tracing::event!(
                    Level::ERROR,
                    "primary shape in port geometry was not specified"
                );
                LayoutError::IoDefinition
            })?,
            unnamed_shapes: self.unnamed_shapes,
            named_shapes: self.named_shapes,
        })
    }
}

impl<L: Clone> HierarchicalBuildFrom<NamedPorts<L>> for PortGeometryBuilder<L> {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts<L>) {
        let source = source.get(path).unwrap();
        self.primary = Some(source.primary.clone());
        self.unnamed_shapes.clone_from(&source.unnamed_shapes);
        self.named_shapes.clone_from(&source.named_shapes);
    }
}
