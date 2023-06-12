//! Axis-aligned rectangular bounding boxes.

use crate::rect::Rect;

/// Compute the axis-aligned rectangular bounding box.
pub trait Bbox {
    /// Compute the axis-aligned rectangular bounding box.
    ///
    /// If empty, this method should return `None`.
    fn bbox(&self) -> Option<Rect>;
}
