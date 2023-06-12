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
    /// Creates a rectangle with points `(0, 0), (dims.w(), dims.h())`.
    ///
    /// The caller should ensure that `dims.w()` and `dims.h()` are non-negative.
    /// See [`Dims`] for more information.
    pub fn from_dims(dims: Dims) -> Self {
        Self::new(Point::zero(), Point::new(dims.w(), dims.h()))
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point::new((self.p0.x + self.p1.x) / 2, (self.p0.y + self.p1.y) / 2)
    }

    /// Creates an empty rectangle containing the given point.
    pub fn from_point(p: Point) -> Self {
        Self { p0: p, p1: p }
    }

    /// Creates an empty rectangle containing the given `(x, y)` coordinates.
    pub fn from_xy(x: i64, y: i64) -> Self {
        let p = Point::new(x, y);
        Self::from_point(p)
    }

    /// Creates a new rectangle.
    pub fn new(p0: Point, p1: Point) -> Self {
        Self {
            p0: Point::new(p0.x.min(p1.x), p0.y.min(p1.y)),
            p1: Point::new(p0.x.max(p1.x), p0.y.max(p1.y)),
        }
    }

    /// Creates a rectangle from horizontal and vertical [`Span`]s.
    pub fn from_spans(h: Span, v: Span) -> Self {
        Self {
            p0: Point::new(h.start(), v.start()),
            p1: Point::new(h.stop(), v.stop()),
        }
    }

    /// Returns the bottom y-coordinate of the rectangle.
    #[inline]
    pub fn bottom(&self) -> i64 {
        self.p0.y
    }

    /// Returns the top y-coordinate of the rectangle.
    #[inline]
    pub fn top(&self) -> i64 {
        self.p1.y
    }

    /// Returns the left x-coordinate of the rectangle.
    #[inline]
    pub fn left(&self) -> i64 {
        self.p0.x
    }

    /// Returns the right x-coordinate of the rectangle.
    #[inline]
    pub fn right(&self) -> i64 {
        self.p1.x
    }

    /// Returns the horizontal span of the rectangle.
    pub fn hspan(&self) -> Span {
        Span::new(self.p0.x, self.p1.x)
    }

    /// Returns a [`Rect`] with the given `hspan` and the same vertical span.
    pub fn with_hspan(self, hspan: Span) -> Self {
        Rect::new(
            Point::new(hspan.start(), self.p0.y),
            Point::new(hspan.stop(), self.p1.y),
        )
    }

    /// Returns a [`Rect`] with the given `vspan` and the same horizontal span.
    pub fn with_vspan(self, vspan: Span) -> Self {
        Rect::new(
            Point::new(self.p0.x, vspan.start()),
            Point::new(self.p1.x, vspan.stop()),
        )
    }

    /// Returns a [`Rect`] with the given `span` in the given `dir`, and the current span in the
    /// other direction.
    pub fn with_span(self, span: Span, dir: Dir) -> Self {
        match dir {
            Dir::Vert => self.with_vspan(span),
            Dir::Horiz => self.with_hspan(span),
        }
    }

    /// Returns the vertical span of the rectangle.
    pub fn vspan(&self) -> Span {
        Span::new(self.p0.y, self.p1.y)
    }

    /// Returns the horizontal width of the rectangle.
    #[inline]
    pub fn width(&self) -> i64 {
        self.hspan().length()
    }

    /// Returns the vertical height of the rectangle.
    #[inline]
    pub fn height(&self) -> i64 {
        self.vspan().length()
    }

    /// Returns the area of the rectangle.
    #[inline]
    pub fn area(&self) -> i64 {
        self.width() * self.height()
    }

    /// Returns the lower edge of the rectangle in the [`Dir`] `dir.
    pub fn lower_edge(&self, dir: Dir) -> i64 {
        self.span(dir).start()
    }

    /// Returns the upper edge of the rectangle in the [`Dir`] `dir.
    pub fn upper_edge(&self, dir: Dir) -> i64 {
        self.span(dir).stop()
    }

    /// Returns the span of the rectangle in the [`Dir`] `dir.
    pub fn span(&self, dir: Dir) -> Span {
        match dir {
            Dir::Horiz => self.hspan(),
            Dir::Vert => self.vspan(),
        }
    }

    /// Returns the edges of two rectangles along the [`Dir`] `dir` in increasing order.
    fn sorted_edges(&self, other: &Self, dir: Dir) -> [i64; 4] {
        let mut edges = [
            self.lower_edge(dir),
            self.upper_edge(dir),
            other.lower_edge(dir),
            other.upper_edge(dir),
        ];
        edges.sort();
        edges
    }

    /// Returns the inner two edges of two rectangles along the [`Dir`] `dir` in increasing order.
    #[inline]
    pub fn inner_span(&self, other: &Self, dir: Dir) -> Span {
        let edges = self.sorted_edges(other, dir);
        Span::new(edges[1], edges[2])
    }

    /// Returns the outer two edges of two rectangles along the [`Dir`] `dir` in increasing order.
    #[inline]
    pub fn outer_span(&self, other: &Self, dir: Dir) -> Span {
        let edges = self.sorted_edges(other, dir);
        Span::new(edges[0], edges[3])
    }

    /// Returns the edge of a rectangle closest to the coordinate `x` along a given direction.
    pub fn edge_closer_to(&self, x: i64, dir: Dir) -> i64 {
        let (x0, x1) = self.span(dir).into();
        if (x - x0).abs() <= (x - x1).abs() {
            x0
        } else {
            x1
        }
    }

    /// Returns the edge of a rectangle farthest from the coordinate `x` along a given direction.
    pub fn edge_farther_from(&self, x: i64, dir: Dir) -> i64 {
        let (x0, x1) = self.span(dir).into();
        if (x - x0).abs() <= (x - x1).abs() {
            x1
        } else {
            x0
        }
    }

    /// Returns a builder for creating a rectangle from [`Span`]s.
    #[inline]
    pub fn from_dir_spans(dir: Dir, parallel_span: Span, perp_span: Span) -> Self {
        match dir {
            Dir::Vert => Self::from_spans(perp_span, parallel_span),
            Dir::Horiz => Self::from_spans(parallel_span, perp_span),
        }
    }

    /// Returns the length of this rectangle in the given direction.
    pub fn length(&self, dir: Dir) -> i64 {
        self.span(dir).length()
    }

    /// Returns the direction in which the rectangle is longer, choosing [`Dir::Vert`] if the sides
    /// are equal.
    #[inline]
    pub fn longer_dir(&self) -> Dir {
        if self.width() > self.height() {
            Dir::Horiz
        } else {
            Dir::Vert
        }
    }

    /// Returns the direction in which the rectangle is longer, choosing [`Dir::Vert`] if the sides
    /// are equal.
    #[inline]
    pub fn shorter_dir(&self) -> Dir {
        !self.longer_dir()
    }

    pub fn union(self, other: Self) -> Self {
        Rect::new(
            Point::new(self.p0.x.min(other.p0.x), self.p0.y.min(other.p0.y)),
            Point::new(self.p1.x.max(other.p1.x), self.p1.y.max(other.p1.y)),
        )
    }

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
    #[inline]
    pub fn expand(&self, amount: i64) -> Self {
        Self::new(
            Point::new(self.p0.x - amount, self.p0.y - amount),
            Point::new(self.p1.x + amount, self.p1.y + amount),
        )
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

    /// Expands the rectangle by `amount` on both sides associated with the direction `dir`.
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

    /// Grows this rectangle by a factor of 2 on the given [`Side`].
    ///
    /// Sometimes useful for half-track geometry.
    pub fn double(self, side: Side) -> Self {
        match side {
            Side::Top => Self::from_spans(
                self.hspan(),
                Span::with_start_and_length(self.bottom(), 2 * self.height()),
            ),
            Side::Bot => Self::from_spans(
                self.hspan(),
                Span::with_stop_and_length(self.top(), 2 * self.height()),
            ),
            Side::Left => Self::from_spans(
                Span::with_stop_and_length(self.right(), 2 * self.width()),
                self.vspan(),
            ),
            Side::Right => Self::from_spans(
                Span::with_start_and_length(self.left(), 2 * self.width()),
                self.vspan(),
            ),
        }
    }

    #[inline]
    pub fn side(&self, side: Side) -> i64 {
        match side {
            Side::Top => self.top(),
            Side::Bot => self.bottom(),
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
        let b_span = Span::new(src.bottom(), clip.bottom());
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
