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
    pdk::{layers::LayerId, Pdk},
};

use super::{draw::DrawContainer, CellBuilder, HasLayout, Instance};

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

/// A raw layout cell.
#[derive(Default, Debug, Clone)]
pub struct RawCell {
    pub(crate) id: CellId,
    pub(crate) name: ArcStr,
    pub(crate) elements: Vec<Element>,
    pub(crate) blockages: Vec<Shape>,
    #[allow(dead_code)]
    ports: HashMap<NameBuf, PortGeometry>,
}

impl RawCell {
    pub(crate) fn new(id: CellId, name: ArcStr) -> Self {
        Self {
            id,
            name,
            elements: Vec::new(),
            blockages: Vec::new(),
            ports: HashMap::new(),
        }
    }

    pub(crate) fn from_ports_and_builder<PDK: Pdk, T: HasLayout>(
        ports: HashMap<NameBuf, PortGeometry>,
        cell_builder: CellBuilder<PDK, T>,
    ) -> Self {
        Self {
            ports,
            ..cell_builder.cell
        }
    }
}

impl DrawContainer for RawCell {
    fn draw_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    fn draw_blockage(&mut self, shape: Shape) {
        self.blockages.push(shape);
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
#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct RawInstance {
    pub(crate) cell: Arc<RawCell>,
    pub(crate) loc: Point,
    pub(crate) orientation: Orientation,
}

impl RawInstance {
    /// Returns the current transformation of `self`.
    pub fn transformation(&self) -> Transformation {
        Transformation::from_offset_and_orientation(self.loc, self.orientation)
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
#[derive(Debug, Clone)]
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
#[derive(Default, Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum Element {
    /// A raw layout instance.
    Instance(RawInstance),
    /// A primitive layout shape.
    Shape(Shape),
    /// A primitive text annotation.
    Text(Text),
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
