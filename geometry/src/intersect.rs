//! Intersections of geometric objects.

/// Trait for calculating the intersection with another geometric object.
pub trait Intersect<T: ?Sized> {
    type Output;
    /// Calculates the intersection of this shape with `other`.
    ///
    /// If no part of this shape lies within `bounds`,
    /// returns [`None`].
    fn intersect(self, bounds: &T) -> Option<Self::Output>;
}
