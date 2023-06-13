//! Unions of geometric objects.

/// Trait for calculating the union with another geometric object.
pub trait Union<T: ?Sized> {
    /// The type of the output shape representing the union.
    type Output;
    /// Calculates the union of this shape with `other`.
    fn union(self, other: &T) -> Self::Output;
}
