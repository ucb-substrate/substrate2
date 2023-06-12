//! Traits for checking whether a shape contains another shape.

/// Whether an object **completely** contains another object.
pub trait Contains<T> {
    /// Returns whether or not this object **completely** contains `other`.
    fn contains(&self, other: &T) -> bool;
}

/// Whether an object **partially** contains another object.
pub trait PartialContains<T> {
    /// Returns whether or not this object contains **any** part of `other`.
    fn partial_contains(&self, other: &T) -> bool;
}
