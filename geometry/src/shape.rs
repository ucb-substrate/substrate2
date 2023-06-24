//! An enumeration of geometric shapes and their properties.

use crate::{
    bbox::Bbox,
    rect::Rect,
    transform::{TransformMut, TranslateMut},
};

/// An enumeration of geometric shapes.
#[derive(Debug, Clone)]
pub enum Shape {
    /// A rectangle.
    Rect(Rect),
}

impl TranslateMut for Shape {
    fn translate_mut(&mut self, p: crate::point::Point) {
        match self {
            Shape::Rect(rect) => rect.translate_mut(p),
        };
    }
}

impl TransformMut for Shape {
    fn transform_mut(&mut self, trans: crate::prelude::Transformation) {
        match self {
            Shape::Rect(rect) => rect.transform_mut(trans),
        }
    }
}

impl Bbox for Shape {
    fn bbox(&self) -> Option<Rect> {
        match self {
            Shape::Rect(rect) => rect.bbox(),
        }
    }
}

impl From<Rect> for Shape {
    #[inline]
    fn from(value: Rect) -> Self {
        Self::Rect(value)
    }
}
