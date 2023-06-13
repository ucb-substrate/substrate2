//! Traits for checking whether a shape contains another shape.

/// Ways in which an inner shape can be contained within an enclosing shape.
pub enum Containment {
    /// The shape is fully contained in the enclosing shape.
    Full,
    /// The shape is partially contained in the enclosing shape.
    Partial,
    /// The enclosing shape does not contain any part of the inner shape.
    None,
}

/// Provides information on whether a shape contains another shape.
pub trait Contains<T> {
    /// Returns a [`Containment`] indicating how `other` is enclosed within this shape.
    ///
    /// * If `other` is entirely contained, returns [`Containment::Full`].
    /// * If `other` is only partially contained, returns [`Containment::Partial`].
    /// * If no part of `other` lies within this shape, returns [`Containment::None`].
    fn contains(&self, other: &T) -> Containment;
}
