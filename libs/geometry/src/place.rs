//! Traits for placing a geometric object at a point.

use serde::{Deserialize, Serialize};

use crate::{
    bbox::Bbox, corner::Corner, point::Point, rect::Rect, side::Side, transform::TranslateMut,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// An enumeration of possible ways to place a geometric shape at a point.
pub enum PlaceMode {
    /// Place the corner of a geometric shape at a point.
    Corner(Corner),
    /// Place the side of a geometric shape at a point's coordinate along the same axis.
    Side(Side),
    /// Place the center of a side of a geometric shape at a point.
    SideCenter(Side),
    /// Place the center of a geometric shape at a point.
    Center,
    /// Place the x-coordinate of a geometric shape's center at the x-coordinate of a point.
    CenterX,
    /// Place the y-coordinate of a geometric shape's center at the y-coordinate of a point.
    CenterY,
}

/// A geometric shape that can be placed at a point.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// # use geometry::place::PlaceRectMut;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// rect1.place_mut(PlaceMode::Center, rect1, Point::new(25, 25));
/// assert_eq!(rect1, Rect::from_sides(-25, -75, 75, 125));
///
/// // Alternate rectangle to align `rect1` with.
/// // Conceptually, this represents that `rect1` has
/// // 5 units of hangover space on all sides that should
/// // not contribute to placement.
/// let rect1_alt = rect1.shrink_all(5).unwrap();
/// rect1.place_mut(PlaceMode::Corner(Corner::UpperRight), rect1_alt, Point::new(25, 25));
/// assert_eq!(rect1, Rect::from_sides(-70, -170, 30, 30));
/// ```
pub trait PlaceRectMut: TranslateMut {
    /// Places an object at the given point.
    ///
    /// For center alignments, the center's non-integer coordinates are rounded down to the nearest integer.
    /// This behavior is subject to change and should not be relied upon.
    fn place_mut(&mut self, mode: PlaceMode, srect: Rect, pt: Point) {
        match mode {
            PlaceMode::Corner(corner) => {
                let ofs = pt - srect.corner(corner);
                self.translate_mut(ofs);
            }
            PlaceMode::Side(side) => {
                let dir_ofs = side.coord_dir();
                let ofs = pt.coord(dir_ofs) - srect.side(side);
                self.translate_mut(Point::from_dir_coords(dir_ofs, ofs, 0));
            }
            PlaceMode::SideCenter(side) => {
                let edge = srect.edge(side);
                let center =
                    Point::from_dir_coords(side.coord_dir(), edge.coord(), edge.span().center());
                let ofs = pt - center;
                self.translate_mut(ofs);
            }
            PlaceMode::Center => {
                let ofs = pt - srect.center();
                self.translate_mut(ofs);
            }
            PlaceMode::CenterX => {
                let ofs = pt.x - srect.center().x;
                self.translate_mut(Point::new(ofs, 0));
            }
            PlaceMode::CenterY => {
                let ofs = pt.y - srect.center().y;
                self.translate_mut(Point::new(0, ofs));
            }
        }
    }
}

impl<T: TranslateMut> PlaceRectMut for T {}

/// A geometric shape that can be placed at a point.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// assert_eq!(
///     rect1.place(PlaceMode::Center, rect1, Point::new(25, 25)),
///     Rect::from_sides(-25, -75, 75, 125),
/// );
///
/// // Alternate rectangle to align `rect1` with.
/// // Conceptually, this represents that `rect1` has
/// // 5 units of hangover space on all sides that should
/// // not contribute to placement.
/// let rect1_alt = rect1.shrink_all(5).unwrap();
///
/// assert_eq!(
///     rect1.place(
///         PlaceMode::Corner(Corner::UpperRight),
///         rect1_alt,
///         Point::new(25, 25)
///     ),
///     Rect::from_sides(-70, -170, 30, 30),
/// );
/// ```
pub trait PlaceRect: PlaceRectMut + Sized {
    /// Places an object at the given point.
    ///
    /// For center alignments, the center's non-integer coordinates are rounded down to the nearest integer.
    /// This behavior is subject to change and should not be relied upon.
    ///
    /// Creates a new shape at the placed location.
    fn place(mut self, mode: PlaceMode, srect: Rect, pt: Point) -> Self {
        self.place_mut(mode, srect, pt);
        self
    }
}

impl<T: PlaceRectMut + Sized> PlaceRect for T {}

/// A geometric shape that can be placed at a point using its bounding box.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// # use geometry::place::PlaceBboxMut;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// rect1.place_bbox_mut(PlaceMode::Center, Point::new(25, 25));
/// assert_eq!(rect1, Rect::from_sides(-25, -75, 75, 125));
/// ```
pub trait PlaceBboxMut: PlaceRectMut + Bbox {
    /// Places an object at the given point using its bounding box.
    ///
    /// For center alignments, the center's non-integer coordinates are rounded down to the nearest integer.
    /// This behavior is subject to change and should not be relied upon.
    fn place_bbox_mut(&mut self, mode: PlaceMode, pt: Point) {
        self.place_mut(mode, self.bbox().unwrap(), pt)
    }
}

impl<T: PlaceRectMut + Bbox> PlaceBboxMut for T {}

/// A geometric shape that can be placed at a point using its bounding box.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let mut rect1 = Rect::from_sides(0, 0, 100, 200);
/// assert_eq!(
///     rect1.place_bbox(PlaceMode::Center, Point::new(25, 25)),
///     Rect::from_sides(-25, -75, 75, 125),
/// );
/// ```
pub trait PlaceBbox: PlaceBboxMut + Sized {
    /// Places an object at the given point using its boudning box.
    ///
    /// For center alignments, the center's non-integer coordinates are rounded down to the nearest integer.
    /// This behavior is subject to change and should not be relied upon.
    ///
    /// Creates a new shape at the placed location.
    fn place_bbox(mut self, mode: PlaceMode, pt: Point) -> Self {
        self.place_bbox_mut(mode, pt);
        self
    }
}

impl<T: PlaceBboxMut + Sized> PlaceBbox for T {}

#[cfg(test)]
mod tests {
    use crate::{
        corner::Corner,
        place::{PlaceBbox, PlaceMode, PlaceRect},
        point::Point,
        rect::Rect,
        side::Side,
    };

    #[test]
    fn place_and_place_bbox_work() {
        let rect1 = Rect::from_sides(0, 0, 100, 200);
        let pt = Point::new(75, 75);

        assert_eq!(
            rect1.place(PlaceMode::Corner(Corner::UpperLeft), rect1, pt),
            Rect::from_sides(75, -125, 175, 75)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::Corner(Corner::UpperLeft), pt),
            Rect::from_sides(75, -125, 175, 75)
        );

        assert_eq!(
            rect1.place(PlaceMode::Side(Side::Right), rect1, pt),
            Rect::from_sides(-25, 0, 75, 200)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::Side(Side::Right), pt),
            Rect::from_sides(-25, 0, 75, 200)
        );

        assert_eq!(
            rect1.place(PlaceMode::SideCenter(Side::Left), rect1, pt),
            Rect::from_sides(75, -25, 175, 175)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::SideCenter(Side::Left), pt),
            Rect::from_sides(75, -25, 175, 175)
        );

        assert_eq!(
            rect1.place(PlaceMode::Center, rect1, pt),
            Rect::from_sides(25, -25, 125, 175)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::Center, pt),
            Rect::from_sides(25, -25, 125, 175)
        );

        assert_eq!(
            rect1.place(PlaceMode::CenterX, rect1, pt),
            Rect::from_sides(25, 0, 125, 200)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::CenterX, pt),
            Rect::from_sides(25, 0, 125, 200)
        );

        assert_eq!(
            rect1.place(PlaceMode::CenterY, rect1, pt),
            Rect::from_sides(0, -25, 100, 175)
        );
        assert_eq!(
            rect1.place_bbox(PlaceMode::CenterY, pt),
            Rect::from_sides(0, -25, 100, 175)
        );
    }
}
