//! Unions of geometric objects.

/// Trait for calculating the union with another geometric object.
pub trait Union<T: ?Sized> {
    /// The type of the output shape representing the union.
    type Output;
    /// Calculates the union of this shape with `other`.
    fn union(&self, other: &T) -> Self::Output;
}

/// Trait for calculating the bounding box of the union with another geometric object.
pub trait BboxUnion<T: ?Sized> {
    /// The type of the output shape representing the union.
    type Output;
    /// Computes the rectangular union of this bounding box with another bounding box.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = Rect::from_sides(-50, 20, 120, 160);
    /// assert_eq!(r1.union(r2), Rect::from_sides(-50, 0, 120, 200));
    ///
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = None
    /// assert_eq!(r1.union(r2), Rect::from_sides(0, 0, 100, 200));
    /// ```
    fn union(&self, other: &T) -> Self::Output;
}
