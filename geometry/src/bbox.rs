//! Axis-aligned rectangular bounding boxes.

use crate::rect::Rect;

/// A geometric shape that has a bounding box.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let rect = Rect::from_sides(0, 0, 100, 200);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(0, 0, 100, 200)));
/// let rect = Rect::from_xy(50, 70);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(50, 70, 50, 70)));
/// ```
pub trait Bbox {
    /// Compute the axis-aligned rectangular bounding box.
    ///
    /// If empty, this method should return `None`.
    /// Note that poinst and zero-area rectangles are not empty:
    /// these shapes contain a single point, and their bounding box
    /// implementations will return `Some(_)`.
    fn bbox(&self) -> Option<Rect>;
}
