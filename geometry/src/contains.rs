//! Traits for checking whether a shape contains another shape.

use serde::{Deserialize, Serialize};

/// Ways in which an inner shape can be contained within an enclosing shape.
#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, Ord, PartialOrd,
)]
pub enum Containment {
    /// The enclosing shape does not contain any part of the inner shape.
    #[default]
    None,
    /// The shape is partially contained in the enclosing shape.
    Partial,
    /// The shape is fully contained in the enclosing shape.
    Full,
}

/// Provides information on whether a shape contains another shape.
pub trait Contains<T> {
    /// Returns a [`Containment`] indicating how `other` is enclosed within this shape.
    ///
    /// * If `other` is entirely contained, returns [`Containment::Full`].
    /// * If `other` is only partially contained, returns [`Containment::Partial`].
    /// * If no part of `other` lies within this shape, returns [`Containment::None`].
    fn contains(&self, other: &T) -> Containment;

    /// Returns true if `other` is fully enclosed in this shape.
    #[inline]
    fn encloses(&self, other: &T) -> bool {
        self.contains(other).is_full()
    }

    /// Returns true if `other` is fully or partially enclosed in this shape.
    #[inline]
    fn partially_intersects(&self, other: &T) -> bool {
        self.contains(other).intersects()
    }
}

impl Containment {
    /// Returns true when fully contained.
    #[inline]
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Returns true if there is **at least** partial containment.
    #[inline]
    pub fn intersects(&self) -> bool {
        matches!(self, Self::Full | Self::Partial)
    }

    /// Returns true if there is **only** partial containment.
    #[inline]
    pub fn only_partially_intersects(&self) -> bool {
        matches!(self, Self::Partial)
    }

    /// Returns true if there is no containment.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
