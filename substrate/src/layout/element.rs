//! Basic layout elements.
//!
//! Substrate layouts consist of cells, instances, geometric shapes, and text annotations.

use std::sync::Arc;

use arcstr::ArcStr;
use geometry::{
    prelude::{Bbox, Orientation, Point},
    rect::Rect,
    transform::{Transform, Transformation},
};
use serde::{Deserialize, Serialize};

use super::{builder::CellBuilder, draw::DrawContainer, HasLayout};

/// A context-wide unique identifier for a cell.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CellId(usize);

impl std::ops::Add<usize> for CellId {
    type Output = CellId;

    fn add(self, rhs: usize) -> Self::Output {
        CellId(self.0 + rhs)
    }
}

impl std::ops::AddAssign<usize> for CellId {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

/// A raw layout cell.
#[derive(Default, Debug, Clone)]
pub struct RawCell {
    #[allow(dead_code)]
    id: CellId,
    elements: Vec<Element>,
    blockages: Vec<Shape>,
    // TODO: ports: HashMap<ArcStr, PortGeometry>,
}

impl RawCell {
    pub(crate) fn new(id: CellId) -> Self {
        Self {
            id,
            elements: Vec::new(),
            blockages: Vec::new(),
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

impl<T: HasLayout> From<CellBuilder<T>> for RawCell {
    fn from(value: CellBuilder<T>) -> Self {
        value.into_cell()
    }
}

/// A raw layout instance.
///
/// Consists of a pointer to an underlying cell and its instantiated location and orientation.
#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct RawInstance {
    cell: Arc<RawCell>,
    loc: Point,
    orientation: Orientation,
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

/// A primitive layout shape consisting of a layer and a geometric shape.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Shape {
    // TODO: layer: LayerId,
    shape: geometry::shape::Shape,
}

impl Shape {
    /// Creates a new layout shape.
    pub fn new(shape: impl Into<geometry::shape::Shape>) -> Self {
        Self {
            shape: shape.into(),
        }
    }
}

impl Bbox for Shape {
    fn bbox(&self) -> Option<Rect> {
        self.shape.bbox()
    }
}

/// A primitive text annotation consisting of a layer, string, and location.
#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct Text {
    // TODO: layer: LayerId,
    text: ArcStr,
    loc: geometry::point::Point,
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
