//! An enumeration of geometric shapes and their properties.

use crate::{
    bbox::Bbox,
    contains::{Containment, Contains},
    point::Point,
    polygon::Polygon,
    rect::Rect,
    transform::{TransformMut, TransformRef, Transformation, TranslateMut, TranslateRef},
    union::BoundingUnion,
};

/// An enumeration of geometric shapes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Shape {
    /// A rectangle.
    Rect(Rect),
    /// A polygon.
    Polygon(Polygon),
}

impl Shape {
    /// If this shape is a rectangle, returns the contained rectangle.
    /// Otherwise, returns [`None`].
    pub fn rect(&self) -> Option<Rect> {
        match self {
            Self::Rect(r) => Some(*r),
            _ => None,
        }
    }

    /// If this shape is a polygon, returns the contained polygon.
    /// Otherwise, returns [`None`].
    pub fn polygon(&self) -> Option<&Polygon> {
        match self {
            Self::Polygon(p) => Some(p),
            _ => None,
        }
    }
}

impl TranslateRef for Shape {
    #[inline]
    fn translate_ref(&self, p: Point) -> Self {
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.translate_ref(p)),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.translate_ref(p)),
        }
    }
}

impl TranslateMut for Shape {
    #[inline]
    fn translate_mut(&mut self, p: Point) {
        match self {
            Shape::Rect(rect) => rect.translate_mut(p),
            Shape::Polygon(polygon) => polygon.translate_mut(p),
        };
    }
}

impl TransformRef for Shape {
    #[inline]
    fn transform_ref(&self, trans: Transformation) -> Self {
        match self {
            Shape::Rect(rect) => Shape::Rect(rect.transform_ref(trans)),
            Shape::Polygon(polygon) => Shape::Polygon(polygon.transform_ref(trans)),
        }
    }
}

impl TransformMut for Shape {
    #[inline]
    fn transform_mut(&mut self, trans: crate::prelude::Transformation) {
        match self {
            Shape::Rect(rect) => rect.transform_mut(trans),
            Shape::Polygon(polygon) => polygon.transform_mut(trans),
        }
    }
}

impl Bbox for Shape {
    fn bbox(&self) -> Option<Rect> {
        match self {
            Shape::Rect(rect) => rect.bbox(),
            Shape::Polygon(polygon) => polygon.bbox(),
        }
    }
}

impl From<Rect> for Shape {
    #[inline]
    fn from(value: Rect) -> Self {
        Self::Rect(value)
    }
}

impl From<Polygon> for Shape {
    #[inline]
    fn from(value: Polygon) -> Self {
        Self::Polygon(value)
    }
}

impl<T: Bbox> BoundingUnion<T> for Shape {
    type Output = Option<Rect>;

    fn bounding_union(&self, other: &T) -> Self::Output {
        self.bbox().bounding_union(&other.bbox())
    }
}

impl Contains<crate::point::Point> for Shape {
    fn contains(&self, p: &crate::point::Point) -> Containment {
        match self {
            Shape::Rect(rect) => rect.contains(p),
            Shape::Polygon(polygon) => polygon.contains(p),
        }
    }
}
