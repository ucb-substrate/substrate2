//! An enumeration of geometric shapes and their properties.

use crate::{
    bbox::Bbox,
    prelude::Transform,
    rect::Rect,
    transform::{HasTransformedView, TransformMut, TranslateMut},
    union::BoundingUnion,
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

impl HasTransformedView for Shape {
    type TransformedView<'a> = Shape;

    fn transformed_view(&self, trans: crate::prelude::Transformation) -> Self::TransformedView<'_> {
        self.clone().transform(trans)
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

impl<T: Bbox> BoundingUnion<T> for Shape {
    type Output = Option<Rect>;

    fn bounding_union(&self, other: &T) -> Self::Output {
        self.bbox().bounding_union(&other.bbox())
    }
}
