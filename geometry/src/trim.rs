//! Trim geometric objects to fit within a bounding shape.

/// Trait for trimming a geometric object to fit within a shape.
pub trait Trim<T: ?Sized> {
    type Output;
    /// Trims `self` to fit within `bounds`.
    ///
    /// If no part of shape lies within `bounds`,
    /// returns [`None`].
    fn trim(&self, bounds: &T) -> Option<Self::Output>;
}
