//! Unions of geometric objects.

/// Trait for calculating the union with another geometric object.
pub trait Union<T: ?Sized> {
    /// The type of the output shape representing the union.
    type Output;
    /// Calculates the union of this shape with `other`.
    fn union(&self, other: &T) -> Self::Output;
}

/// Trait for calculating a shape that bounds the union of two geometric objects.
pub trait BoundingUnion<T: ?Sized> {
    /// The type of the output shape representing the bounding union.
    type Output;
    /// Calculates the bounding union of this shape with `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// # use geometry::union::BoundingUnion;
    ///
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = Rect::from_sides(-50, 20, 120, 160);
    /// assert_eq!(r1.bounding_union(&r2), Rect::from_sides(-50, 0, 120, 200));
    ///
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = None;
    /// assert_eq!(r1.bounding_union(&r2), Rect::from_sides(0, 0, 100, 200));
    /// ```
    fn bounding_union(&self, other: &T) -> Self::Output;
}

impl<T: BoundingUnion<T, Output = T> + Clone> BoundingUnion<Option<T>> for T {
    type Output = T;

    fn bounding_union(&self, other: &Option<T>) -> Self::Output {
        if let Some(obj) = other {
            self.bounding_union(obj)
        } else {
            self.clone()
        }
    }
}

impl<T: BoundingUnion<T, Output = T> + Clone> BoundingUnion<T> for Option<T> {
    type Output = T;
    fn bounding_union(&self, other: &T) -> Self::Output {
        if let Some(obj) = self {
            obj.bounding_union(other)
        } else {
            other.clone()
        }
    }
}

impl<T: BoundingUnion<Option<T>, Output = T> + Clone> BoundingUnion<Option<T>> for Option<T> {
    type Output = Option<T>;
    fn bounding_union(&self, other: &Option<T>) -> Self::Output {
        if let Some(obj) = self {
            Some(obj.bounding_union(other))
        } else {
            other.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{rect::Rect, span::Span, union::BoundingUnion};

    #[test]
    fn union_works_for_option_rects() {
        let r1 = Rect::from_sides(0, 0, 100, 200);
        let r2 = Rect::from_sides(-50, 20, 120, 160);
        assert_eq!(r1.bounding_union(&r2), Rect::from_sides(-50, 0, 120, 200));

        let r1 = Rect::from_sides(0, 0, 100, 200);
        let r2 = None;
        assert_eq!(r1.bounding_union(&r2), Rect::from_sides(0, 0, 100, 200));
        assert_eq!(r2.bounding_union(&r1), Rect::from_sides(0, 0, 100, 200));

        let r1 = Some(Rect::from_sides(0, 0, 100, 200));
        let r2: Option<Rect> = None;
        assert_eq!(
            r1.bounding_union(&r2),
            Some(Rect::from_sides(0, 0, 100, 200))
        );
        assert_eq!(
            r2.bounding_union(&r1),
            Some(Rect::from_sides(0, 0, 100, 200))
        );

        let r1: Option<Rect> = None;
        let r2: Option<Rect> = None;
        assert_eq!(r1.bounding_union(&r2), None,);
    }

    #[test]
    fn union_works_for_option_spans() {
        let r1 = Span::new(0, 100);
        let r2 = Span::new(-50, 120);
        assert_eq!(r1.bounding_union(&r2), Span::new(-50, 120));

        let r1 = Span::new(0, 100);
        let r2 = None;
        assert_eq!(r1.bounding_union(&r2), Span::new(0, 100));
        assert_eq!(r2.bounding_union(&r1), Span::new(0, 100));

        let r1 = Some(Span::new(0, 100));
        let r2: Option<Span> = None;
        assert_eq!(r1.bounding_union(&r2), Some(Span::new(0, 100)));
        assert_eq!(r2.bounding_union(&r1), Some(Span::new(0, 100)));

        let r1: Option<Span> = None;
        let r2: Option<Span> = None;
        assert_eq!(r1.bounding_union(&r2), None,);
    }
}
