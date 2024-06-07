//! Traits and types for schematic IOs.

use crate::error::Result;
use crate::io::{FlatLen, Flatten, HasNameTree, NameBuf, NameFragment, NameTree, Signal};
use crate::layout::element::NamedPorts;
use crate::layout::error::LayoutError;
use crate::pdk::layers::{HasPin, LayerId};
use arcstr::ArcStr;
pub use codegen::LayoutType as HardwareType;
use geometry::prelude::{Bbox, Transformation};
use geometry::rect::Rect;
use geometry::transform;
use geometry::transform::HasTransformedView;
use geometry::union::BoundingUnion;
use std::collections::HashMap;
use substrate::layout::bbox::LayerBbox;
use tracing::Level;

/// A layout hardware type.
pub trait HardwareType: FlatLen + HasNameTree + Clone {
    /// The **Rust** type representing layout instances of this **hardware** type.
    type Bundle: IsBundle;
    /// A builder for creating [`HardwareType::Bundle`].
    type Builder: BundleBuilder<Self::Bundle>;

    /// Instantiates a schematic data struct with populated nodes.
    fn builder(&self) -> Self::Builder;
}

/// The associated bundle of a layout type.
pub type Bundle<T> = <T as HardwareType>::Bundle;

/// The associated builder of a layout type.
pub type Builder<T> = <T as HardwareType>::Builder;

/// Layout hardware data builder.
///
/// A builder for an instance of bundle `T`.
pub trait BundleBuilder<T: IsBundle> {
    /// Builds an instance of bundle `T`.
    fn build(self) -> Result<T>;
}

/// A custom layout type that can be derived from an existing layout type.
pub trait CustomHardwareType<T: HardwareType>: HardwareType {
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

/// A type representing a single hardware layout port with a single [`Shape`](crate::layout::element::Shape) as
/// its geometry.
#[derive(Debug, Default, Clone, Copy)]
pub struct ShapePort;

/// A generic layout port that consists of several shapes.
#[derive(Debug, Default, Clone, Copy)]
pub struct Port;

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

impl PortGeometry {
    /// Returns an iterator over all shapes in a [`PortGeometry`].
    pub fn shapes(&self) -> impl Iterator<Item = &IoShape> {
        std::iter::once(&self.primary)
            .chain(self.unnamed_shapes.iter())
            .chain(self.named_shapes.values())
    }

    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes
    /// and moving their old values to the collection of unnamed shapes.
    pub(crate) fn merge(&mut self, other: impl Into<PortGeometry>) {
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

impl Bbox for PortGeometry {
    fn bbox(&self) -> Option<Rect> {
        self.shapes().fold(None, |a, b| a.bounding_union(&b.bbox()))
    }
}

/// A type that can is a bundle of layout ports.
///
/// An instance of a [`HardwareType`].
pub trait IsBundle: FlatLen + Flatten<PortGeometry> + HasTransformedView + Send + Sync {}

impl<T> IsBundle for T where T: FlatLen + Flatten<PortGeometry> + HasTransformedView + Send + Sync {}

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

impl LayerBbox for IoShape {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        if self.layer.pin == layer || self.layer.drawing == layer {
            self.shape.bbox()
        } else {
            None
        }
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

impl transform::HasTransformedView for IoShape {
    type TransformedView = IoShape;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        IoShape {
            shape: self.shape.transformed_view(trans),
            ..*self
        }
    }
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

    /// Merges [`PortGeometry`] `other` into `self`, overwriting the primary and corresponding named shapes
    /// and moving their old values to the collection of unnamed shapes.
    pub fn merge(&mut self, other: impl Into<PortGeometry>) {
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
    pub fn set_primary(&mut self, shape: IoShape) {
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
/// # use substrate::io::layout::OptionBuilder;
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

impl FlatLen for ShapePort {
    fn len(&self) -> usize {
        1
    }
}

impl HardwareType for ShapePort {
    type Bundle = IoShape;
    type Builder = OptionBuilder<IoShape>;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl HasNameTree for ShapePort {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl CustomHardwareType<Signal> for ShapePort {
    fn from_layout_type(_other: &Signal) -> Self {
        ShapePort
    }
}

impl FlatLen for Port {
    fn len(&self) -> usize {
        1
    }
}

impl HardwareType for Port {
    type Bundle = PortGeometry;
    type Builder = PortGeometryBuilder;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl HasNameTree for Port {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl CustomHardwareType<Signal> for Port {
    fn from_layout_type(_other: &Signal) -> Self {
        Port
    }
}

impl FlatLen for IoShape {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<PortGeometry> for IoShape {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
        output.extend(std::iter::once(PortGeometry {
            primary: self.clone(),
            unnamed_shapes: Vec::new(),
            named_shapes: HashMap::new(),
        }));
    }
}

impl HierarchicalBuildFrom<NamedPorts> for OptionBuilder<IoShape> {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts) {
        self.set(source.get(path).unwrap().primary.clone());
    }
}

impl<T: IsBundle> BundleBuilder<T> for OptionBuilder<T> {
    fn build(self) -> Result<T> {
        self.build()
    }
}

impl FlatLen for PortGeometry {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<PortGeometry> for PortGeometry {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl transform::HasTransformedView for PortGeometry {
    type TransformedView = PortGeometry;

    fn transformed_view(
        &self,
        trans: geometry::transform::Transformation,
    ) -> Self::TransformedView {
        Self::TransformedView {
            primary: self.primary.transformed_view(trans),
            unnamed_shapes: self.unnamed_shapes.transformed_view(trans),
            named_shapes: self
                .named_shapes
                .iter()
                .map(|(k, v)| (k.clone(), v.transformed_view(trans)))
                .collect(),
        }
    }
}

impl FlatLen for PortGeometryBuilder {
    fn len(&self) -> usize {
        1
    }
}

impl BundleBuilder<PortGeometry> for PortGeometryBuilder {
    fn build(self) -> Result<PortGeometry> {
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

impl HierarchicalBuildFrom<NamedPorts> for PortGeometryBuilder {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts) {
        let source = source.get(path).unwrap();
        self.primary = Some(source.primary.clone());
        self.unnamed_shapes.clone_from(&source.unnamed_shapes);
        self.named_shapes.clone_from(&source.named_shapes);
    }
}
