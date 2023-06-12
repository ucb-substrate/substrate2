//! Axis-aligned rectangles.

use serde::{Deserialize, Serialize};

use crate::bbox::Bbox;
use crate::corner::Corner;
use crate::dims::Dims;
use crate::dir::Dir;
use crate::edge::Edge;
use crate::point::Point;
use crate::side::Side;
use crate::span::Span;
use crate::transform::{Transform, Transformation, Translate};
use crate::trim::Trim;

/// An axis-aligned rectangle, specified by lower-left and upper-right corners.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rect {
    /// The lower-left corner.
    p0: Point,
    /// The upper-right corner.
    p1: Point,
}

impl Rect {
    /// Creates a rectangle with corners `(0, 0), (dims.w(), dims.h())`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let dims = Dims::new(100, 200);
    /// let rect = Rect::from_dims(dims);
    /// assert_eq!(rect.top(), 200);
    /// assert_eq!(rect.bot(), 0);
    /// assert_eq!(rect.left(), 0);
    /// assert_eq!(rect.right(), 100);
    /// ```
    pub fn from_dims(dims: Dims) -> Self {
        Self::new(Point::zero(), Point::new(dims.w(), dims.h()))
    }

    /// Returns the center point of the rectangle.
    ///
    /// Note that the center point will be rounded to integer coordinates.
    /// The current behavior is to round down, but this is subject to change;
    /// users should not rely on this behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 200, 100);
    /// assert_eq!(rect.center(), Point::new(100, 50));
    /// ```
    ///
    /// Center points are rounded:
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 55, 45);
    /// assert_eq!(rect.center(), Point::new(27, 22));
    /// ```
    pub fn center(&self) -> Point {
        Point::new((self.p0.x + self.p1.x) / 2, (self.p0.y + self.p1.y) / 2)
    }

    /// Creates a zero-area rectangle containing the given point.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_point(Point::new(25, 60));
    /// assert_eq!(rect.top(), 60);
    /// assert_eq!(rect.bot(), 60);
    /// assert_eq!(rect.left(), 25);
    /// assert_eq!(rect.right(), 25);
    /// ```
    #[inline]
    pub fn from_point(p: Point) -> Self {
        Self { p0: p, p1: p }
    }

    /// Creates a rectangle from all 4 sides (left, bottom, right, top).
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(15, 20, 30, 40);
    /// assert_eq!(rect.left(), 15);
    /// assert_eq!(rect.bot(), 20);
    /// assert_eq!(rect.right(), 30);
    /// assert_eq!(rect.top(), 40);
    /// ```
    #[inline]
    pub fn from_sides(left: i64, bot: i64, right: i64, top: i64) -> Self {
        Self::new(Point::new(left, bot), Point::new(right, top))
    }

    /// Creates a zero-area empty rectangle containing the given `(x, y)` coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_xy(25, 60);
    /// assert_eq!(rect.top(), 60);
    /// assert_eq!(rect.bot(), 60);
    /// assert_eq!(rect.left(), 25);
    /// assert_eq!(rect.right(), 25);
    /// ```
    pub fn from_xy(x: i64, y: i64) -> Self {
        let p = Point::new(x, y);
        Self::from_point(p)
    }

    /// Creates a new rectangle from the given opposite corner points.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::new(Point::new(15, 20), Point::new(30, 40));
    /// assert_eq!(rect.left(), 15);
    /// assert_eq!(rect.bot(), 20);
    /// assert_eq!(rect.right(), 30);
    /// assert_eq!(rect.top(), 40);
    /// ```
    #[inline]
    pub fn new(p0: Point, p1: Point) -> Self {
        Self {
            p0: Point::new(p0.x.min(p1.x), p0.y.min(p1.y)),
            p1: Point::new(p0.x.max(p1.x), p0.y.max(p1.y)),
        }
    }

    /// Creates a rectangle from horizontal and vertical [`Span`]s.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let hspan = Span::new(15, 30);
    /// let vspan = Span::new(20, 40);
    /// let rect = Rect::from_spans(hspan, vspan);
    /// assert_eq!(rect.left(), 15);
    /// assert_eq!(rect.bot(), 20);
    /// assert_eq!(rect.right(), 30);
    /// assert_eq!(rect.top(), 40);
    /// ```
    pub fn from_spans(h: Span, v: Span) -> Self {
        Self {
            p0: Point::new(h.start(), v.start()),
            p1: Point::new(h.stop(), v.stop()),
        }
    }

    /// Returns the bottom y-coordinate of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.bot(), 20);
    /// ```
    #[inline]
    pub fn bot(&self) -> i64 {
        self.p0.y
    }

    /// Returns the top y-coordinate of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.top(), 40);
    /// ```
    #[inline]
    pub fn top(&self) -> i64 {
        self.p1.y
    }

    /// Returns the left x-coordinate of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.left(), 10);
    /// ```
    #[inline]
    pub fn left(&self) -> i64 {
        self.p0.x
    }

    /// Returns the right x-coordinate of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.right(), 30);
    /// ```
    #[inline]
    pub fn right(&self) -> i64 {
        self.p1.x
    }

    /// Returns the horizontal [`Span`] of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.hspan(), Span::new(10, 30));
    /// ```
    pub fn hspan(&self) -> Span {
        Span::new(self.p0.x, self.p1.x)
    }

    /// Returns a new [`Rect`] with the given `hspan` and the same vertical span.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// let new_hspan = Span::new(100, 200);
    /// let new_rect = rect.with_hspan(new_hspan);
    /// assert_eq!(new_rect, Rect::from_sides(100, 20, 200, 40));
    /// ```
    pub fn with_hspan(self, hspan: Span) -> Self {
        Rect::new(
            Point::new(hspan.start(), self.p0.y),
            Point::new(hspan.stop(), self.p1.y),
        )
    }

    /// Returns a [`Rect`] with the given `vspan` and the same horizontal span.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// let new_vspan = Span::new(100, 200);
    /// let new_rect = rect.with_vspan(new_vspan);
    /// assert_eq!(new_rect, Rect::from_sides(10, 100, 30, 200));
    /// ```
    pub fn with_vspan(self, vspan: Span) -> Self {
        Rect::new(
            Point::new(self.p0.x, vspan.start()),
            Point::new(self.p1.x, vspan.stop()),
        )
    }

    /// Returns a [`Rect`] with the given `span` in the given `dir`, and the current span in the
    /// other direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// let new_vspan = Span::new(100, 200);
    /// let new_rect = rect.with_span(new_vspan, Dir::Vert);
    /// assert_eq!(new_rect, Rect::from_sides(10, 100, 30, 200));
    /// ```
    pub fn with_span(self, span: Span, dir: Dir) -> Self {
        match dir {
            Dir::Vert => self.with_vspan(span),
            Dir::Horiz => self.with_hspan(span),
        }
    }

    /// Returns the vertical span of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 40);
    /// assert_eq!(rect.vspan(), Span::new(20, 40));
    /// ```
    pub fn vspan(&self) -> Span {
        Span::new(self.p0.y, self.p1.y)
    }

    /// Returns the horizontal width of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.width(), 20);
    /// ```
    #[inline]
    pub fn width(&self) -> i64 {
        self.hspan().length()
    }

    /// Returns the vertical height of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.height(), 30);
    /// ```
    #[inline]
    pub fn height(&self) -> i64 {
        self.vspan().length()
    }

    /// Returns the area of the rectangle.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.area(), 600);
    /// ```
    #[inline]
    pub fn area(&self) -> i64 {
        self.width() * self.height()
    }

    /// Returns the lower edge of the rectangle in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.lower_edge(Dir::Vert), 20);
    /// assert_eq!(rect.lower_edge(Dir::Horiz), 10);
    /// ```
    pub fn lower_edge(&self, dir: Dir) -> i64 {
        self.span(dir).start()
    }

    /// Returns the upper edge of the rectangle in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.upper_edge(Dir::Vert), 50);
    /// assert_eq!(rect.upper_edge(Dir::Horiz), 30);
    /// ```
    pub fn upper_edge(&self, dir: Dir) -> i64 {
        self.span(dir).stop()
    }

    /// Returns the span of the rectangle in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 20, 30, 50);
    /// assert_eq!(rect.span(Dir::Vert), Span::new(20, 50));
    /// assert_eq!(rect.span(Dir::Horiz), Span::new(10, 30));
    /// ```
    pub fn span(&self, dir: Dir) -> Span {
        match dir {
            Dir::Horiz => self.hspan(),
            Dir::Vert => self.vspan(),
        }
    }

    /// Returns the edges of two rectangles along the given `dir` in increasing order.
    ///
    /// For [`Dir::Horiz`], returns the sorted x-coordinates of all **vertical** edges.
    /// For [`Dir::Vert`], returns the sorted y-coordinates of all **horizontal** edges.
    fn sorted_edges(&self, other: Self, dir: Dir) -> [i64; 4] {
        let mut edges = [
            self.lower_edge(dir),
            self.upper_edge(dir),
            other.lower_edge(dir),
            other.upper_edge(dir),
        ];
        edges.sort();
        edges
    }

    /// Returns the span between the inner two edges of two rectangles along the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(10, 25, 30, 50);
    /// let r2 = Rect::from_sides(20, 15, 70, 35);
    /// assert_eq!(r1.inner_span(r2, Dir::Horiz), Span::new(20, 30));
    /// assert_eq!(r1.inner_span(r2, Dir::Vert), Span::new(25, 35));
    /// ```
    ///
    /// The "order" of `r1` and `r2` does not matter:
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(10, 25, 30, 50);
    /// let r2 = Rect::from_sides(20, 15, 70, 35);
    /// assert_eq!(r2.inner_span(r1, Dir::Horiz), Span::new(20, 30));
    /// assert_eq!(r2.inner_span(r1, Dir::Vert), Span::new(25, 35));
    /// ```
    #[inline]
    pub fn inner_span(&self, other: Self, dir: Dir) -> Span {
        let edges = self.sorted_edges(other, dir);
        Span::new(edges[1], edges[2])
    }

    /// Returns the span between the outer two edges of two rectangles along the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(10, 25, 30, 50);
    /// let r2 = Rect::from_sides(20, 15, 70, 35);
    /// assert_eq!(r1.outer_span(r2, Dir::Horiz), Span::new(10, 70));
    /// assert_eq!(r1.outer_span(r2, Dir::Vert), Span::new(15, 50));
    /// ```
    ///
    /// The "order" of `r1` and `r2` does not matter:
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(10, 25, 30, 50);
    /// let r2 = Rect::from_sides(20, 15, 70, 35);
    /// assert_eq!(r2.outer_span(r1, Dir::Horiz), Span::new(10, 70));
    /// assert_eq!(r2.outer_span(r1, Dir::Vert), Span::new(15, 50));
    /// ```
    #[inline]
    pub fn outer_span(&self, other: Self, dir: Dir) -> Span {
        let edges = self.sorted_edges(other, dir);
        Span::new(edges[0], edges[3])
    }

    /// Returns the edge of a rectangle closest to the coordinate `x` along a given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 25, 30, 50);
    /// assert_eq!(rect.edge_closer_to(14, Dir::Horiz), 10);
    /// assert_eq!(rect.edge_closer_to(22, Dir::Horiz), 30);
    /// assert_eq!(rect.edge_closer_to(23, Dir::Vert), 25);
    /// assert_eq!(rect.edge_closer_to(37, Dir::Vert), 25);
    /// assert_eq!(rect.edge_closer_to(38, Dir::Vert), 50);
    /// assert_eq!(rect.edge_closer_to(59, Dir::Vert), 50);
    /// ```
    pub fn edge_closer_to(&self, x: i64, dir: Dir) -> i64 {
        let (x0, x1) = self.span(dir).into();
        if (x - x0).abs() <= (x - x1).abs() {
            x0
        } else {
            x1
        }
    }

    /// Returns the edge of a rectangle farthest from the coordinate `x` along a given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(10, 25, 30, 50);
    /// assert_eq!(rect.edge_farther_from(14, Dir::Horiz), 30);
    /// assert_eq!(rect.edge_farther_from(22, Dir::Horiz), 10);
    /// assert_eq!(rect.edge_farther_from(23, Dir::Vert), 50);
    /// assert_eq!(rect.edge_farther_from(37, Dir::Vert), 50);
    /// assert_eq!(rect.edge_farther_from(38, Dir::Vert), 25);
    /// assert_eq!(rect.edge_farther_from(59, Dir::Vert), 25);
    /// ```
    pub fn edge_farther_from(&self, x: i64, dir: Dir) -> i64 {
        let (x0, x1) = self.span(dir).into();
        if (x - x0).abs() <= (x - x1).abs() {
            x1
        } else {
            x0
        }
    }

    /// Creates a rectangle from two [`Span`]s, where the first is parallel to `dir`,
    /// and the second is perpendicular.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let span1 = Span::new(10, 30);
    /// let span2 = Span::new(25, 50);
    /// let rect = Rect::from_dir_spans(Dir::Horiz, span1, span2);
    /// assert_eq!(rect, Rect::from_sides(10, 25, 30, 50));
    /// let rect = Rect::from_dir_spans(Dir::Vert, span1, span2);
    /// assert_eq!(rect, Rect::from_sides(25, 10, 50, 30));
    /// ```
    #[inline]
    pub fn from_dir_spans(dir: Dir, parallel_span: Span, perp_span: Span) -> Self {
        match dir {
            Dir::Vert => Self::from_spans(perp_span, parallel_span),
            Dir::Horiz => Self::from_spans(parallel_span, perp_span),
        }
    }

    /// Returns the length of this rectangle in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 200, 100);
    /// assert_eq!(rect.length(Dir::Horiz), 200);
    /// assert_eq!(rect.length(Dir::Vert), 100);
    /// ```
    ///
    pub fn length(&self, dir: Dir) -> i64 {
        self.span(dir).length()
    }

    /// Returns the direction in which the rectangle is longer, choosing [`Dir::Horiz`] if the sides
    /// are equal.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.longer_dir(), Dir::Vert);
    /// let rect = Rect::from_sides(0, 0, 200, 100);
    /// assert_eq!(rect.longer_dir(), Dir::Horiz);
    /// let rect = Rect::from_sides(0, 0, 100, 100);
    /// assert_eq!(rect.longer_dir(), Dir::Horiz);
    /// ```
    #[inline]
    pub fn longer_dir(&self) -> Dir {
        if self.height() > self.width() {
            Dir::Vert
        } else {
            Dir::Horiz
        }
    }

    /// Returns the direction in which the rectangle is shorter, choosing [`Dir::Vert`] if the sides
    /// are equal.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.shorter_dir(), Dir::Horiz);
    /// let rect = Rect::from_sides(0, 0, 200, 100);
    /// assert_eq!(rect.shorter_dir(), Dir::Vert);
    /// let rect = Rect::from_sides(0, 0, 100, 100);
    /// assert_eq!(rect.shorter_dir(), Dir::Vert);
    /// ```
    #[inline]
    pub fn shorter_dir(&self) -> Dir {
        !self.longer_dir()
    }

    /// Computes the rectangular union of this `Rect` with another `Rect`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = Rect::from_sides(-50, 20, 120, 160);
    /// assert_eq!(r1.union(r2), Rect::from_sides(-50, 0, 120, 200));
    /// ```
    pub fn union(self, other: Self) -> Self {
        Rect::new(
            Point::new(self.p0.x.min(other.p0.x), self.p0.y.min(other.p0.y)),
            Point::new(self.p1.x.max(other.p1.x), self.p1.y.max(other.p1.y)),
        )
    }

    /// Computes the rectangular intersection of this `Rect` with another `Rect`.
    ///
    /// Returns `None` if the intersection is empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = Rect::from_sides(-50, 20, 120, 160);
    /// assert_eq!(r1.intersection(r2), Some(Rect::from_sides(0, 20, 100, 160)));
    ///
    /// let r1 = Rect::from_sides(0, 0, 100, 200);
    /// let r2 = Rect::from_sides(120, -60, 240, 800);
    /// assert_eq!(r1.intersection(r2), None);
    /// ```
    pub fn intersection(self, other: Self) -> Option<Self> {
        let pmin = Point::new(self.p0.x.max(other.p0.x), self.p0.y.max(other.p0.y));
        let pmax = Point::new(self.p1.x.min(other.p1.x), self.p1.y.min(other.p1.y));

        // Check for empty intersection, and return None
        if pmin.x > pmax.x || pmin.y > pmax.y {
            return None;
        }

        // Otherwise, return the intersection
        Some(Rect::new(pmin, pmax))
    }

    /// Expands the rectangle by `amount` on all sides.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.expand_all(20), Rect::from_sides(-20, -20, 120, 220));
    /// ```
    #[inline]
    pub fn expand_all(&self, amount: i64) -> Self {
        Self::new(
            Point::new(self.p0.x - amount, self.p0.y - amount),
            Point::new(self.p1.x + amount, self.p1.y + amount),
        )
    }

    /// Expands the rectangle by `amount` on both sides associated with the direction `dir`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.expand_dir(Dir::Horiz, 20), Rect::from_sides(-20, 0, 120, 200));
    /// assert_eq!(rect.expand_dir(Dir::Vert, 20), Rect::from_sides(0, -20, 100, 220));
    /// ```
    #[inline]
    pub fn expand_dir(&self, dir: Dir, amount: i64) -> Self {
        match dir {
            Dir::Horiz => Self::new(
                Point::new(self.p0.x - amount, self.p0.y),
                Point::new(self.p1.x + amount, self.p1.y),
            ),
            Dir::Vert => Self::new(
                Point::new(self.p0.x, self.p0.y - amount),
                Point::new(self.p1.x, self.p1.y + amount),
            ),
        }
    }

    /// Expands the rectangle by `amount` on the given side.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.expand_side(Side::Top, 20), Rect::from_sides(0, 0, 100, 220));
    /// assert_eq!(rect.expand_side(Side::Bot, 20), Rect::from_sides(0, -20, 100, 200));
    /// assert_eq!(rect.expand_side(Side::Left, 20), Rect::from_sides(-20, 0, 100, 200));
    /// assert_eq!(rect.expand_side(Side::Right, 20), Rect::from_sides(0, 0, 120, 200));
    /// ```
    #[inline]
    pub fn expand_side(&self, side: Side, amount: i64) -> Self {
        match side {
            Side::Top => Self::new(
                Point::new(self.p0.x, self.p0.y),
                Point::new(self.p1.x, self.p1.y + amount),
            ),
            Side::Bot => Self::new(
                Point::new(self.p0.x, self.p0.y - amount),
                Point::new(self.p1.x, self.p1.y),
            ),
            Side::Right => Self::new(
                Point::new(self.p0.x, self.p0.y),
                Point::new(self.p1.x + amount, self.p1.y),
            ),
            Side::Left => Self::new(
                Point::new(self.p0.x - amount, self.p0.y),
                Point::new(self.p1.x, self.p1.y),
            ),
        }
    }

    /// Expands the rectangle by `amount` at the given corner.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let rect = Rect::from_sides(0, 0, 100, 200);
    /// assert_eq!(rect.expand_corner(Corner::LowerLeft, 20), Rect::from_sides(-20, -20, 100, 200));
    /// assert_eq!(rect.expand_corner(Corner::LowerRight, 20), Rect::from_sides(0, -20, 120, 200));
    /// assert_eq!(rect.expand_corner(Corner::UpperLeft, 20), Rect::from_sides(-20, 0, 100, 220));
    /// assert_eq!(rect.expand_corner(Corner::UpperRight, 20), Rect::from_sides(0, 0, 120, 220));
    /// ```
    #[inline]
    pub fn expand_corner(self, corner: Corner, amount: i64) -> Self {
        match corner {
            Corner::LowerLeft => {
                Self::from_sides(self.p0.x - amount, self.p0.y - amount, self.p1.x, self.p1.y)
            }
            Corner::LowerRight => {
                Self::from_sides(self.p0.x, self.p0.y - amount, self.p1.x + amount, self.p1.y)
            }
            Corner::UpperLeft => {
                Self::from_sides(self.p0.x - amount, self.p0.y, self.p1.x, self.p1.y + amount)
            }
            Corner::UpperRight => {
                Self::from_sides(self.p0.x, self.p0.y, self.p1.x + amount, self.p1.y + amount)
            }
        }
    }

    /// Shrinks the rectangle by `amount` on all sides.
    #[inline]
    pub fn shrink(&self, amount: i64) -> Self {
        assert!(2 * amount <= self.width());
        assert!(2 * amount <= self.height());
        Self::new(
            Point::new(self.p0.x + amount, self.p0.y + amount),
            Point::new(self.p1.x - amount, self.p1.y - amount),
        )
    }

    /// Returns the dimensions of the rectangle as [`Dims`].
    #[inline]
    pub fn dims(&self) -> Dims {
        Dims::new(self.width(), self.height())
    }

    /// Returns the desired corner of the rectangle.
    pub fn corner(&self, corner: Corner) -> Point {
        match corner {
            Corner::LowerLeft => self.p0,
            Corner::LowerRight => Point::new(self.p1.x, self.p0.y),
            Corner::UpperLeft => Point::new(self.p0.x, self.p1.y),
            Corner::UpperRight => self.p1,
        }
    }

    #[inline]
    pub fn side(&self, side: Side) -> i64 {
        match side {
            Side::Top => self.top(),
            Side::Bot => self.bot(),
            Side::Right => self.right(),
            Side::Left => self.left(),
        }
    }

    #[inline]
    pub fn edge(&self, side: Side) -> Edge {
        Edge::new(side, self.side(side), self.span(side.edge_dir()))
    }

    /// Snaps the corners of this rectangle to the given grid.
    ///
    /// Note that the rectangle may have zero area after snapping.
    #[inline]
    pub fn snap_to_grid(&self, grid: i64) -> Self {
        Self::new(self.p0.snap_to_grid(grid), self.p1.snap_to_grid(grid))
    }

    pub fn cutout(&self, clip: Rect) -> [Rect; 4] {
        let src = *self;
        let t_span = Span::new(clip.top(), src.top());
        let b_span = Span::new(src.bot(), clip.bot());
        let l_span = Span::new(src.left(), clip.left());
        let r_span = Span::new(clip.right(), src.right());

        [
            Rect::from_spans(src.hspan(), t_span),
            Rect::from_spans(src.hspan(), b_span),
            Rect::from_spans(l_span, src.vspan()),
            Rect::from_spans(r_span, src.vspan()),
        ]
    }
}

impl Bbox for Rect {
    fn bbox(&self) -> Option<Rect> {
        Some(*self)
    }
}

impl Trim<Rect> for Rect {
    type Output = Self;

    fn trim(&self, bounds: &Rect) -> Option<Self::Output> {
        self.intersection(*bounds)
    }
}

impl Translate for Rect {
    fn translate(self, p: Point) -> Self {
        Self::new(self.p0.translate(p), self.p1.translate(p))
    }
}

impl Transform for Rect {
    fn transform(self, trans: Transformation) -> Self {
        let (p0, p1) = (self.p0, self.p1);
        let p0p = p0.transform(trans);
        let p1p = p1.transform(trans);

        let p0 = Point::new(std::cmp::min(p0p.x, p1p.x), std::cmp::min(p0p.y, p1p.y));
        let p1 = Point::new(std::cmp::max(p0p.x, p1p.x), std::cmp::max(p0p.y, p1p.y));
        Rect { p0, p1 }
    }
}

impl From<Dims> for Rect {
    #[inline]
    fn from(value: Dims) -> Self {
        Self::from_dims(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn sorted_edges() {
        let r1 = Rect::from_sides(10, 25, 30, 50);
        let r2 = Rect::from_sides(20, 15, 70, 35);
        assert_eq!(r1.sorted_edges(r2, Dir::Horiz), [10, 20, 30, 70]);
        assert_eq!(r1.sorted_edges(r2, Dir::Vert), [15, 25, 35, 50]);
        assert_eq!(r2.sorted_edges(r1, Dir::Horiz), [10, 20, 30, 70]);
        assert_eq!(r2.sorted_edges(r1, Dir::Vert), [15, 25, 35, 50]);
    }
}
