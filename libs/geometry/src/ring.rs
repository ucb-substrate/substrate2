//! Rectangular ring geometry.
//!
//! May be useful for drawing structures that enclose other structures,
//! such as guard rings.

use array_map::ArrayMap;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::transform::TranslateMut;

/// A rectangular ring surrounding an enclosed rectangle.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ring {
    /// Vertical span of top segment.
    topv: Span,
    /// Vertical span of bottom segment.
    botv: Span,
    /// Horizontal span of left segment.
    lefth: Span,
    /// Horizontal span of right segment.
    righth: Span,
}

/// Represents the ways [`Ring`] geometry can be specified.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RingContents {
    /// The ring must fit within the given rectangle.
    Outer(Rect),
    /// The ring must enclose the given rectangle.
    Inner(Rect),
}

impl RingContents {
    /// The rectangle stored in this enum variant.
    pub fn rect(&self) -> Rect {
        match self {
            Self::Outer(r) => *r,
            Self::Inner(r) => *r,
        }
    }

    /// Returns true if this is a [`RingContents::Outer`] variant.
    pub fn is_outer(&self) -> bool {
        matches!(self, Self::Outer(_))
    }

    /// Returns true if this is a [`RingContents::Inner`] variant.
    pub fn is_inner(&self) -> bool {
        matches!(self, Self::Inner(_))
    }
}

/// A utility for constructing a [`Ring`].
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RingBuilder {
    contents: Option<RingContents>,
    widths: ArrayMap<Side, i64, 4>,
}

impl Ring {
    /// Creates a new [`RingBuilder`].
    #[inline]
    pub fn builder() -> RingBuilder {
        RingBuilder::new()
    }

    /// Checks that the ring is valid.
    pub(crate) fn is_valid(&self) -> bool {
        self.topv.start() > self.botv.stop() && self.righth.start() > self.lefth.stop()
    }

    /// The horizontal span of the annulus of the ring.
    pub fn outer_hspan(&self) -> Span {
        Span::new(self.lefth.start(), self.righth.stop())
    }

    /// The horizontal span of the inner portion of the ring.
    pub fn inner_hspan(&self) -> Span {
        Span::new(self.lefth.stop(), self.righth.start())
    }

    /// The vertical span of the annulus of the ring.
    pub fn outer_vspan(&self) -> Span {
        Span::new(self.botv.start(), self.topv.stop())
    }

    /// The vertical span of the inner portion of the ring.
    pub fn inner_vspan(&self) -> Span {
        Span::new(self.botv.stop(), self.topv.start())
    }

    /// The outer annulus bounding box.
    pub fn outer(&self) -> Rect {
        Rect::from_spans(self.outer_hspan(), self.outer_vspan())
    }

    /// The inner rectangle.
    pub fn inner(&self) -> Rect {
        Rect::from_spans(self.inner_hspan(), self.inner_vspan())
    }

    /// The annular rectangle on the given side.
    #[inline]
    pub fn rect(&self, side: Side) -> Rect {
        match side {
            Side::Top => Rect::from_spans(self.outer_hspan(), self.topv),
            Side::Right => Rect::from_spans(self.righth, self.outer_vspan()),
            Side::Bot => Rect::from_spans(self.outer_hspan(), self.botv),
            Side::Left => Rect::from_spans(self.lefth, self.outer_vspan()),
        }
    }

    /// The annuluar rectangle on the given side, but limited to the width/height of the inner rectangle.
    #[inline]
    pub fn inner_rect(&self, side: Side) -> Rect {
        match side {
            Side::Top => Rect::from_spans(self.inner_hspan(), self.topv),
            Side::Right => Rect::from_spans(self.righth, self.inner_vspan()),
            Side::Bot => Rect::from_spans(self.inner_hspan(), self.botv),
            Side::Left => Rect::from_spans(self.lefth, self.inner_vspan()),
        }
    }

    /// The lower left annular corner.
    ///
    /// Shares a corner with the inner rect, but does not have any edges
    /// in common with the inner rect.
    #[inline]
    pub fn corner(&self, corner: Corner) -> Rect {
        match corner {
            Corner::LowerLeft => Rect::from_spans(self.lefth, self.botv),
            Corner::UpperLeft => Rect::from_spans(self.lefth, self.topv),
            Corner::LowerRight => Rect::from_spans(self.righth, self.botv),
            Corner::UpperRight => Rect::from_spans(self.righth, self.topv),
        }
    }

    /// The left annular rectangle.
    #[inline]
    pub fn left(&self) -> Rect {
        self.rect(Side::Left)
    }

    /// The right annular rectangle.
    #[inline]
    pub fn right(&self) -> Rect {
        self.rect(Side::Right)
    }

    /// The top annular rectangle.
    #[inline]
    pub fn top(&self) -> Rect {
        self.rect(Side::Top)
    }

    /// The bottom annular rectangle.
    #[inline]
    pub fn bot(&self) -> Rect {
        self.rect(Side::Bot)
    }

    /// All 4 annular rectangles.
    ///
    /// The order is subject to change.
    #[inline]
    pub fn rects(&self) -> [Rect; 4] {
        [self.top(), self.right(), self.bot(), self.left()]
    }

    /// The [`Rect`]s going in the horizontal direction (ie. the bottom and top rectangles).
    #[inline]
    pub fn hrects(&self) -> [Rect; 2] {
        [self.bot(), self.top()]
    }

    /// The [`Rect`]s going in the vertical direction (ie. the left and right rectangles).
    #[inline]
    pub fn vrects(&self) -> [Rect; 2] {
        [self.left(), self.right()]
    }

    /// The 4 inner annular rectangles.
    ///
    /// The order is subject to change.
    pub fn inner_rects(&self) -> [Rect; 4] {
        [
            self.inner_rect(Side::Top),
            self.inner_rect(Side::Right),
            self.inner_rect(Side::Bot),
            self.inner_rect(Side::Left),
        ]
    }

    /// The inner annular vertical-going (i.e. left and right) rectangles.
    pub fn inner_vrects(&self) -> [Rect; 2] {
        [self.inner_rect(Side::Left), self.inner_rect(Side::Right)]
    }

    /// The inner annular horizontal-going (i.e. top and bottom) rectangles.
    pub fn inner_hrects(&self) -> [Rect; 2] {
        [self.inner_rect(Side::Bot), self.inner_rect(Side::Top)]
    }

    /// The [`Rect`]s going in the given direction.
    ///
    /// Also see [`Ring::hrects`] and [`Ring::vrects`].
    pub fn dir_rects(&self, dir: Dir) -> [Rect; 2] {
        match dir {
            Dir::Horiz => self.hrects(),
            Dir::Vert => self.vrects(),
        }
    }
}

impl Bbox for Ring {
    #[inline]
    fn bbox(&self) -> Option<Rect> {
        self.outer().bbox()
    }
}

impl Contains<Point> for Ring {
    fn contains(&self, other: &Point) -> Containment {
        self.rects()
            .into_iter()
            .map(move |r| r.contains(other))
            .max()
            .unwrap()
    }
}

impl TranslateMut for Ring {
    fn translate_mut(&mut self, p: Point) {
        self.lefth.translate(p.x);
        self.righth.translate(p.x);
        self.botv.translate(p.y);
        self.topv.translate(p.y);
    }
}

impl From<RingBuilder> for Ring {
    fn from(value: RingBuilder) -> Self {
        let contents = value.contents.unwrap();
        let r = contents.rect();

        let sign = if contents.is_outer() {
            Sign::Pos
        } else {
            Sign::Neg
        };

        let topv = Span::with_point_and_length(sign, r.top(), value.widths[Side::Top]);
        let righth = Span::with_point_and_length(sign, r.right(), value.widths[Side::Right]);
        let lefth = Span::with_point_and_length(!sign, r.left(), value.widths[Side::Left]);
        let botv = Span::with_point_and_length(!sign, r.bot(), value.widths[Side::Bot]);

        let res = Self {
            topv,
            botv,
            lefth,
            righth,
        };

        if contents.is_outer() {
            assert_eq!(res.outer(), r);
        } else {
            assert_eq!(res.inner(), r);
        }

        assert!(res.is_valid());
        res
    }
}

impl RingBuilder {
    /// Creates a new [`RingBuilder`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a [`Ring`] from this builder.
    #[inline]
    pub fn build(&mut self) -> Ring {
        Ring::from(*self)
    }

    /// Set the outer region of the ring.
    ///
    /// Only one of inner and outer may be set.
    pub fn outer(&mut self, rect: Rect) -> &mut Self {
        self.contents = Some(RingContents::Outer(rect));
        self
    }

    /// Set the inner region of the ring.
    ///
    /// Only one of inner and outer may be set.
    pub fn inner(&mut self, rect: Rect) -> &mut Self {
        self.contents = Some(RingContents::Inner(rect));
        self
    }

    /// Set the width of the left side of the ring.
    pub fn left_width(&mut self, value: i64) -> &mut Self {
        self.widths[Side::Left] = value;
        self
    }

    /// Set the width of the right side of the ring.
    pub fn right_width(&mut self, value: i64) -> &mut Self {
        self.widths[Side::Right] = value;
        self
    }

    /// Set the height of the bottom of the ring.
    pub fn bot_height(&mut self, value: i64) -> &mut Self {
        self.widths[Side::Bot] = value;
        self
    }

    /// Set the height of the top of the ring.
    pub fn top_height(&mut self, value: i64) -> &mut Self {
        self.widths[Side::Top] = value;
        self
    }

    /// Sets the widths of the vertical-going parts of the ring to the given value.
    pub fn widths(&mut self, value: i64) -> &mut Self {
        self.left_width(value);
        self.right_width(value)
    }

    /// Sets the heights of the horizontal-going parts of the ring to the given value.
    pub fn heights(&mut self, value: i64) -> &mut Self {
        self.top_height(value);
        self.bot_height(value)
    }

    /// Sets the width of all ring edges to the given value.
    pub fn uniform_width(&mut self, value: i64) -> &mut Self {
        self.widths(value);
        self.heights(value)
    }

    /// Sets the width of segments running in the given direction.
    ///
    /// If `dir` is [`Dir::Vert`], sets the widths of the left/right regions.
    /// If `dir` is [`Dir::Horiz`], sets the heights of the top/bottom regions.
    pub fn dir_widths(&mut self, dir: Dir, value: i64) -> &mut Self {
        match dir {
            Dir::Vert => self.widths(value),
            Dir::Horiz => self.heights(value),
        }
    }

    /// Set the width of the given side.
    pub fn side_width(&mut self, side: Side, value: i64) -> &mut Self {
        use Side::*;
        match side {
            Top => self.top_height(value),
            Bot => self.bot_height(value),
            Left => self.left_width(value),
            Right => self.right_width(value),
        }
    }
}
