//! Traits for aligning geometric objects.

use serde::{Deserialize, Serialize};

use crate::{bbox::Bbox, point::Point, rect::Rect, transform::Translate};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// An enumeration of possible alignment modes between two geometric shapes.
pub enum AlignMode {
    /// Align the left sides of the two shapes.
    Left,
    /// Align the right sides of the two shapes.
    Right,
    /// Align the bottom sides of the two shapes.
    Bottom,
    /// Align the top sides of the two shapes.
    Top,
    /// Align the centers of the two shapes horizontally.
    CenterHorizontal,
    /// Align the centers of the two shapes vertically.
    CenterVertical,
    /// Align the left side of one shape to the right of
    /// the right side of the other.
    ToTheRight,
    /// Align the right side of one shape to the left of
    /// the left side of the other.
    ToTheLeft,
    /// Align the top side of one shape beneath
    /// the bottom side of the other.
    Beneath,
    /// Align the bottom side of one shape above
    /// the top side of the other.
    Above,
}

/// A geometric shape that can be aligned using the relationship between two [`Rect`]s.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// rect1.align(AlignMode::Left, rect1, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
///
/// // Alternate rectangle to align `rect1` with.
/// // Conceptually, this represents that `rect1` has
/// // 5 units of hangover space on all sides that should
/// // not contribute to alignment.
/// let rect1_alt = rect1.shrink_all(5).unwrap();
/// rect1.align(AlignMode::Left, rect1_alt, rect2, 0);
/// assert_eq!(rect1, Rect::from_sides(495, 0, 595, 200));
/// ```
pub trait AlignRect: Translate {
    /// Align `self` based on the relationship between `srect` and `orect`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    fn align(&mut self, mode: AlignMode, srect: Rect, orect: Rect, offset: i64) -> &mut Self {
        match mode {
            AlignMode::Left => {
                self.translate(Point::new(orect.left() - srect.left() + offset, 0));
            }
            AlignMode::Right => {
                self.translate(Point::new(orect.right() - srect.right() + offset, 0));
            }
            AlignMode::Bottom => {
                self.translate(Point::new(0, orect.bot() - srect.bot() + offset));
            }
            AlignMode::Top => {
                self.translate(Point::new(0, orect.top() - srect.top() + offset));
            }
            AlignMode::ToTheRight => {
                self.translate(Point::new(orect.right() - srect.left() + offset, 0));
            }
            AlignMode::ToTheLeft => {
                self.translate(Point::new(orect.left() - srect.right() + offset, 0));
            }
            AlignMode::CenterHorizontal => {
                self.translate(Point::new(
                    ((orect.left() + orect.right()) - (srect.left() + srect.right())) / 2 + offset,
                    0,
                ));
            }
            AlignMode::CenterVertical => {
                self.translate(Point::new(
                    0,
                    ((orect.bot() + orect.top()) - (srect.bot() + srect.top())) / 2 + offset,
                ));
            }
            AlignMode::Beneath => {
                self.translate(Point::new(0, orect.bot() - srect.top() + offset));
            }
            AlignMode::Above => {
                self.translate(Point::new(0, orect.top() - srect.top() + offset));
            }
        }
        self
    }
}

impl<T: Translate> AlignRect for T {}

/// A geometric shape that can be aligned with another shape using their bounding boxes.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// rect1.align_bbox(AlignMode::Left, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
/// ```
pub trait AlignBbox: AlignRect + Bbox {
    /// Align `self` using its bounding box and the bounding box of `other`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    fn align_bbox(&mut self, mode: AlignMode, other: impl Bbox, offset: i64) -> &mut Self {
        self.align(mode, self.bbox().unwrap(), other.bbox().unwrap(), offset)
    }
}

impl<T: AlignRect + Bbox> AlignBbox for T {}
