//! Traits for aligning geometric objects.

use serde::{Deserialize, Serialize};

use crate::{
    bbox::Bbox,
    point::Point,
    rect::Rect,
    transform::{Translate, TranslateMut},
};

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
/// # use geometry::align::AlignRectMut;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// rect1.align_mut(AlignMode::Left, rect1, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
///
/// // Alternate rectangle to align `rect1` with.
/// // Conceptually, this represents that `rect1` has
/// // 5 units of hangover space on all sides that should
/// // not contribute to alignment.
/// let rect1_alt = rect1.shrink_all(5).unwrap();
/// rect1.align_mut(AlignMode::Left, rect1_alt, rect2, 0);
/// assert_eq!(rect1, Rect::from_sides(495, 0, 595, 200));
/// ```
pub trait AlignRectMut: TranslateMut {
    /// Align `self` based on the relationship between `srect` and `orect`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    ///
    /// For center alignments, if the centers are a non-integer number of units apart,
    /// the translation amount is rounded down to the nearest integer. This behavior is subject
    /// to change and should not be relied upon.
    fn align_mut(&mut self, mode: AlignMode, srect: Rect, orect: Rect, offset: i64) {
        match mode {
            AlignMode::Left => {
                self.translate_mut(Point::new(orect.left() - srect.left() + offset, 0));
            }
            AlignMode::Right => {
                self.translate_mut(Point::new(orect.right() - srect.right() + offset, 0));
            }
            AlignMode::Bottom => {
                self.translate_mut(Point::new(0, orect.bot() - srect.bot() + offset));
            }
            AlignMode::Top => {
                self.translate_mut(Point::new(0, orect.top() - srect.top() + offset));
            }
            AlignMode::ToTheRight => {
                self.translate_mut(Point::new(orect.right() - srect.left() + offset, 0));
            }
            AlignMode::ToTheLeft => {
                self.translate_mut(Point::new(orect.left() - srect.right() + offset, 0));
            }
            AlignMode::CenterHorizontal => {
                self.translate_mut(Point::new(
                    ((orect.left() + orect.right()) - (srect.left() + srect.right())) / 2 + offset,
                    0,
                ));
            }
            AlignMode::CenterVertical => {
                self.translate_mut(Point::new(
                    0,
                    ((orect.bot() + orect.top()) - (srect.bot() + srect.top())) / 2 + offset,
                ));
            }
            AlignMode::Beneath => {
                self.translate_mut(Point::new(0, orect.bot() - srect.top() + offset));
            }
            AlignMode::Above => {
                self.translate_mut(Point::new(0, orect.top() - srect.bot() + offset));
            }
        }
    }
}

impl<T: Translate> AlignRectMut for T {}

/// A geometric shape that can be aligned using the relationship between two [`Rect`]s.
///
/// Takes in an owned copy of the shape and returns the aligned version.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// let rect1 = rect1.align(AlignMode::Left, rect1, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
///
/// // Alternate rectangle to align `rect1` with.
/// // Conceptually, this represents that `rect1` has
/// // 5 units of hangover space on all sides that should
/// // not contribute to alignment.
/// let rect1_alt = rect1.shrink_all(5).unwrap();
/// assert_eq!(rect1.align(AlignMode::Left, rect1_alt, rect2, 0), Rect::from_sides(495, 0, 595, 200));
/// ```
pub trait AlignRect: AlignRectMut + Sized {
    /// Align `self` based on the relationship between `srect` and `orect`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    ///
    /// For center alignments, if the centers are a non-integer number of units apart,
    /// the translation amount is rounded down to the nearest integer. This behavior is subject
    /// to change and should not be relied upon.
    ///
    /// Creates a new shape at the aligned location of the original.
    fn align(mut self, mode: AlignMode, srect: Rect, orect: Rect, offset: i64) -> Self {
        self.align_mut(mode, srect, orect, offset);
        self
    }
}

impl<T: AlignRectMut + Sized> AlignRect for T {}

/// A geometric shape that can be aligned with another shape using their bounding boxes.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// # use geometry::align::AlignBboxMut;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// rect1.align_bbox_mut(AlignMode::Left, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
/// ```
pub trait AlignBboxMut: AlignRectMut + Bbox {
    /// Align `self` using its bounding box and the bounding box of `other`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    ///
    /// For center alignments, if the centers are a non-integer number of units apart,
    /// the translation amount is rounded down to the nearest integer. This behavior is subject
    /// to change and should not be relied upon.
    fn align_bbox_mut(&mut self, mode: AlignMode, other: impl Bbox, offset: i64) {
        self.align_mut(mode, self.bbox().unwrap(), other.bbox().unwrap(), offset);
    }
}

impl<T: AlignRectMut + Bbox> AlignBboxMut for T {}

/// A geometric shape that can be aligned with another shape using their bounding boxes.
///
/// Takes in an owned copy of the shape and returns the aligned version.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// let rect2 = Rect::from_sides(500, 600, 700, 700);
/// let rect1 = rect1.align_bbox(AlignMode::Left, rect2, 0);
/// assert_eq!(rect1.left(), rect2.left());
/// assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
/// ```
pub trait AlignBbox: AlignBboxMut + Sized {
    /// Align `self` using its bounding box and the bounding box of `other`.
    ///
    /// `offset` represents an offset from the base alignment in the positive direction
    /// along the alignment axis.
    ///
    /// For center alignments, if the centers are a non-integer number of units apart,
    /// the translation amount is rounded down to the nearest integer. This behavior is subject
    /// to change and should not be relied upon.
    ///
    /// Creates a new shape at the aligned location of the original.
    fn align_bbox(mut self, mode: AlignMode, other: impl Bbox, offset: i64) -> Self {
        self.align_bbox_mut(mode, other, offset);
        self
    }
}

impl<T: AlignBboxMut + Sized> AlignBbox for T {}

#[cfg(test)]
mod tests {
    use crate::{
        align::{AlignBboxMut, AlignMode, AlignRectMut},
        rect::Rect,
    };

    #[test]
    fn align_and_align_bbox_work() {
        let mut rect1 = Rect::from_sides(0, 0, 100, 200);
        let mut rect1_bbox = Rect::from_sides(0, 0, 100, 200);
        let rect2 = Rect::from_sides(500, 600, 700, 700);
        rect1.align_mut(AlignMode::Left, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Left, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(500, 0, 600, 200));
        assert_eq!(rect1_bbox, Rect::from_sides(500, 0, 600, 200));

        rect1.align_mut(AlignMode::Right, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Right, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(600, 0, 700, 200));
        assert_eq!(rect1_bbox, Rect::from_sides(600, 0, 700, 200));

        rect1.align_mut(AlignMode::Bottom, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Bottom, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(600, 600, 700, 800));
        assert_eq!(rect1_bbox, Rect::from_sides(600, 600, 700, 800));

        rect1.align_mut(AlignMode::Top, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Top, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(600, 500, 700, 700));
        assert_eq!(rect1_bbox, Rect::from_sides(600, 500, 700, 700));

        rect1.align_mut(AlignMode::ToTheLeft, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::ToTheLeft, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(400, 500, 500, 700));
        assert_eq!(rect1_bbox, Rect::from_sides(400, 500, 500, 700));

        rect1.align_mut(AlignMode::ToTheRight, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::ToTheRight, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(700, 500, 800, 700));
        assert_eq!(rect1_bbox, Rect::from_sides(700, 500, 800, 700));

        rect1.align_mut(AlignMode::Beneath, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Beneath, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(700, 400, 800, 600));
        assert_eq!(rect1_bbox, Rect::from_sides(700, 400, 800, 600));

        rect1.align_mut(AlignMode::Above, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::Above, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(700, 700, 800, 900));
        assert_eq!(rect1_bbox, Rect::from_sides(700, 700, 800, 900));

        rect1.align_mut(AlignMode::CenterHorizontal, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::CenterHorizontal, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(550, 700, 650, 900));
        assert_eq!(rect1_bbox, Rect::from_sides(550, 700, 650, 900));

        rect1.align_mut(AlignMode::CenterVertical, rect1, rect2, 0);
        rect1_bbox.align_bbox_mut(AlignMode::CenterVertical, rect2, 0);
        assert_eq!(rect1, Rect::from_sides(550, 550, 650, 750));
        assert_eq!(rect1_bbox, Rect::from_sides(550, 550, 650, 750));
    }
}
