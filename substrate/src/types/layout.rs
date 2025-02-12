//! Traits and types for layout IOs.

use super::{FlatLen, Flatten, Unflatten};
use crate::error::Result;
use crate::layout::error::LayoutError;
use crate::layout::schema::Schema;
use arcstr::ArcStr;
use geometry::point::Point;
use geometry::prelude::{Bbox, Transformation};
use geometry::rect::Rect;
use geometry::transform::{TransformRef, TranslateRef};
use geometry::union::BoundingUnion;
use layir::Shape;
use std::collections::HashMap;
use tracing::Level;

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
    /// Create a new [`PortGeometry`] with the given primary shape.
    pub fn new(primary: impl Into<Shape<L>>) -> Self {
        Self {
            primary: primary.into(),
            unnamed_shapes: Default::default(),
            named_shapes: Default::default(),
        }
    }

    /// Returns an iterator over all shapes in a [`PortGeometry`].
    pub fn shapes(&self) -> impl Iterator<Item = &Shape<L>> {
        std::iter::once(&self.primary)
            .chain(self.unnamed_shapes.iter())
            .chain(self.named_shapes.values())
    }

    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes
    /// and moving their old values to the collection of unnamed shapes.
    #[allow(dead_code)]
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

impl<L> Unflatten<super::Signal, PortGeometry<L>> for PortGeometry<L> {
    fn unflatten<I>(_data: &super::Signal, source: &mut I) -> Option<Self>
    where
        I: Iterator<Item = PortGeometry<L>>,
    {
        source.next()
    }
}

impl<L: Send + Sync> super::HasBundleKind for PortGeometry<L> {
    type BundleKind = super::Signal;
    fn kind(&self) -> Self::BundleKind {
        super::Signal
    }
}

impl<L: Clone> TryFrom<layir::Port<L>> for PortGeometry<L> {
    type Error = LayoutError;
    fn try_from(port: layir::Port<L>) -> std::result::Result<Self, Self::Error> {
        let mut shapes = port.elements().filter_map(|elt| elt.get_shape().cloned());
        let primary = shapes.next().ok_or(LayoutError::EmptyPort)?;
        let unnamed_shapes = shapes.collect();
        Ok(PortGeometry {
            primary,
            unnamed_shapes,
            named_shapes: Default::default(),
        })
    }
}

/// A type that can be a bundle of layout ports.
///
/// Must have an associated bundle kind via [`HasBundleKind`](super::HasBundleKind).
pub trait LayoutBundle<S: Schema>:
    super::HasBundleKind
    + Flatten<PortGeometry<S::Layer>>
    + Unflatten<Self::BundleKind, PortGeometry<S::Layer>>
    + TransformRef
    + Send
    + Sync
{
}

impl<S, T> LayoutBundle<S> for T
where
    S: Schema,
    T: super::HasBundleKind
        + FlatLen
        + Flatten<PortGeometry<S::Layer>>
        + Unflatten<Self::BundleKind, PortGeometry<S::Layer>>
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
    /// Create a new, empty [`PortGeometryBuilder`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a port geometry.
    pub fn build(self) -> Result<PortGeometry<L>> {
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

impl<L> Flatten<PortGeometry<L>> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<PortGeometry<L>>,
    {
    }
}
