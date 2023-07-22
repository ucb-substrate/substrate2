//! Basic layout elements.
//!
//! Substrate layouts consist of cells, instances, geometric shapes, and text annotations.

use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;
use geometry::{
    prelude::{Bbox, Orientation, Point},
    rect::Rect,
    transform::{HasTransformedView, Transform, Transformation},
    union::BoundingUnion,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    io::{NameBuf, PortGeometry},
    pdk::layers::LayerId,
};

use super::{HasLayout, Instance};

/// A context-wide unique identifier for a cell.
#[derive(
    Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct CellId(u64);

impl CellId {
    pub(crate) fn increment(&mut self) {
        *self = CellId(self.0 + 1)
    }
}

/// A mapping from names to ports.
pub type NamedPorts = HashMap<NameBuf, PortGeometry>;

/// A raw layout cell.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawCell {
    pub(crate) id: CellId,
    pub(crate) name: ArcStr,
    pub(crate) elements: Vec<Element>,
    pub(crate) blockages: Vec<Shape>,
    ports: NamedPorts,
    port_names: HashMap<String, NameBuf>,
}

impl RawCell {
    pub(crate) fn new(id: CellId, name: impl Into<ArcStr>) -> Self {
        Self {
            id,
            name: name.into(),
            elements: Vec::new(),
            blockages: Vec::new(),
            ports: HashMap::new(),
            port_names: HashMap::new(),
        }
    }

    pub(crate) fn with_ports(self, ports: HashMap<NameBuf, PortGeometry>) -> Self {
        let port_names = ports.keys().map(|k| (k.to_string(), k.clone())).collect();
        Self {
            ports,
            port_names,
            ..self
        }
    }

    #[doc(hidden)]
    pub fn port_map(&self) -> &NamedPorts {
        &self.ports
    }

    pub(crate) fn add_element(&mut self, elem: impl Into<Element>) {
        self.elements.push(elem.into());
    }

    pub(crate) fn add_blockage(&mut self, shape: impl Into<Shape>) {
        self.blockages.push(shape.into());
    }

    /// Adds a port to this cell.
    ///
    /// Primarily for use in GDS import.
    pub(crate) fn add_port(&mut self, name: impl Into<NameBuf>, port: impl Into<PortGeometry>) {
        let name = name.into();
        self.ports.insert(name.clone(), port.into());
        self.port_names.insert(name.to_string(), name);
    }

    /// The ID of this cell.
    pub fn id(&self) -> CellId {
        self.id
    }

    /// Returns an iterator over the elements of this cell.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    /// Returns an iterator over the ports of this cell, as `(name, geometry)` pairs.
    pub fn ports(&self) -> impl Iterator<Item = (&NameBuf, &PortGeometry)> {
        self.ports.iter()
    }

    /// Returns a reference to the port with the given name, if it exists.
    pub fn port_named(&self, name: &str) -> Option<&PortGeometry> {
        let name_buf = self.port_names.get(name)?;
        self.ports.get(name_buf)
    }
}

impl Bbox for RawCell {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.elements.bbox()
    }
}

/// A raw layout instance.
///
/// Consists of a pointer to an underlying cell and its instantiated location and orientation.
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct RawInstance {
    pub(crate) cell: Arc<RawCell>,
    pub(crate) loc: Point,
    pub(crate) orientation: Orientation,
}

impl RawInstance {
    /// Create a new raw instance of the given cell.
    pub fn new(
        cell: impl Into<Arc<RawCell>>,
        loc: Point,
        orientation: impl Into<Orientation>,
    ) -> Self {
        Self {
            cell: cell.into(),
            loc,
            orientation: orientation.into(),
        }
    }
    /// Set the orientation of this instance.
    pub(crate) fn set_orientation(&mut self, orientation: impl Into<Orientation>) {
        self.orientation = orientation.into();
    }
    /// Returns the current transformation of `self`.
    pub fn transformation(&self) -> Transformation {
        Transformation::from_offset_and_orientation(self.loc, self.orientation)
    }

    /// Returns a reference to the child cell.
    #[inline]
    pub fn cell(&self) -> &Arc<RawCell> {
        &self.cell
    }
}

impl Bbox for RawInstance {
    fn bbox(&self) -> Option<Rect> {
        self.cell
            .bbox()
            .map(|rect| rect.transform(self.transformation()))
    }
}

impl<T: HasLayout> TryFrom<Instance<T>> for RawInstance {
    type Error = Error;

    fn try_from(value: Instance<T>) -> Result<Self> {
        Ok(Self {
            cell: value.try_cell()?.raw,
            loc: value.loc,
            orientation: value.orientation,
        })
    }
}

/// A primitive layout shape consisting of a layer and a geometric shape.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub struct Shape {
    layer: LayerId,
    shape: geometry::shape::Shape,
}

impl Shape {
    /// Creates a new layout shape.
    pub fn new(layer: impl AsRef<LayerId>, shape: impl Into<geometry::shape::Shape>) -> Self {
        Self {
            layer: *layer.as_ref(),
            shape: shape.into(),
        }
    }

    /// Returns the layer that this shape is on.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Returns the geometric shape of this layout shape.
    pub fn shape(&self) -> &geometry::shape::Shape {
        &self.shape
    }
}

impl Bbox for Shape {
    fn bbox(&self) -> Option<Rect> {
        self.shape.bbox()
    }
}

impl<T: Bbox> BoundingUnion<T> for Shape {
    type Output = Rect;

    fn bounding_union(&self, other: &T) -> Self::Output {
        self.bbox().unwrap().bounding_union(&other.bbox())
    }
}

impl HasTransformedView for Shape {
    type TransformedView<'a> = Shape;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView<'_> {
        Shape {
            layer: self.layer,
            shape: self.shape.transformed_view(trans),
        }
    }
}

/// A primitive text annotation consisting of a layer, string, and location.
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Text {
    layer: LayerId,
    text: ArcStr,
    loc: Point,
    orientation: Orientation,
}

impl Text {
    /// Creates a new layout text annotation.
    pub fn new(
        layer: impl AsRef<LayerId>,
        text: impl Into<ArcStr>,
        loc: Point,
        orientation: Orientation,
    ) -> Self {
        Self {
            layer: *layer.as_ref(),
            text: text.into(),
            loc,
            orientation,
        }
    }

    /// Gets the layer that this annotation is on.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Gets the text of this annotation.
    pub fn text(&self) -> &ArcStr {
        &self.text
    }

    /// Gets the location of this annotation.
    pub fn loc(&self) -> Point {
        self.loc
    }

    /// Gets the orientation of this annotation.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
}

impl Bbox for Text {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        Some(Rect::from_point(self.loc))
    }
}

/// A primitive layout element.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// A raw layout instance.
    Instance(RawInstance),
    /// A primitive layout shape.
    Shape(Shape),
    /// A primitive text annotation.
    Text(Text),
}

/// A pointer to a primitive layout element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementRef<'a> {
    /// A raw layout instance.
    Instance(&'a RawInstance),
    /// A primitive layout shape.
    Shape(&'a Shape),
    /// A primitive text annotation.
    Text(&'a Text),
}

impl Element {
    /// Converts from `&Element` to `ElementRef`.
    ///
    /// Produces a new `ElementRef` containing a reference into
    /// the original element, but leaves the original in place.
    pub fn as_ref(&self) -> ElementRef<'_> {
        match self {
            Self::Instance(x) => ElementRef::Instance(x),
            Self::Shape(x) => ElementRef::Shape(x),
            Self::Text(x) => ElementRef::Text(x),
        }
    }

    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<RawInstance> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<Shape> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<Text> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl<'a> ElementRef<'a> {
    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<&'a RawInstance> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<&'a Shape> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<&'a Text> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl Bbox for Element {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        match self {
            Element::Instance(inst) => inst.bbox(),
            Element::Shape(shape) => shape.bbox(),
            Element::Text(text) => text.bbox(),
        }
    }
}

impl From<RawInstance> for Element {
    fn from(value: RawInstance) -> Self {
        Self::Instance(value)
    }
}

impl From<Shape> for Element {
    fn from(value: Shape) -> Self {
        Self::Shape(value)
    }
}

impl From<Text> for Element {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}
