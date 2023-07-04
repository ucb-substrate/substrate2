//! Intersections of geometric objects.

/// Trait for calculating the intersection with another geometric object.
pub trait Intersect<T: ?Sized> {
    /// The type of the output shape representing the intersection.
    type Output;
    /// Calculates the intersection of this shape with `other`.
    ///
    /// If no part of this shape lies within `other`,
    /// returns [`None`].
    fn intersect(&self, other: &T) -> Option<Self::Output>;
}
