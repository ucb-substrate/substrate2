//! Traits for placing a geometric object at a point.

use serde::{Deserialize, Serialize};

use crate::{corner::Corner, point::Point, rect::Rect, side::Side, transform::TranslateMut};

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
/// rect1.place_mut(PlaceMode::Corner(Corner::UpperRight), rect1, Point::new(25, 25));
/// assert_eq!(rect1, Rect::from_sides(-70, -170, 30, 30));
/// ```
pub trait PlaceRectMut: TranslateMut {
    /// Places an object at the given point.
    ///
    /// For center alignments, the center's coordinates are rounded down to the nearest integer.
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
