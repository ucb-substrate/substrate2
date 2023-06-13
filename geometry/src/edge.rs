//! The edges of rectangular geometry.

use serde::{Deserialize, Serialize};

use crate::dir::Dir;
use crate::side::Side;
use crate::span::Span;

/// An edge of a rectangle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Edge {
    /// The side of the rectangle this edge corresponds to.
    side: Side,
    /// The coordinate of the edge.
    coord: i64,
    /// The perpendicular span of the edge.
    span: Span,
}

impl Edge {
    /// Create a new edge.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// ```
    pub fn new(side: Side, coord: i64, span: Span) -> Self {
        Self { side, coord, span }
    }

    /// The side (of a rectangle) to which this edge corresponds.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.side(), Side::Left);
    /// ```
    pub fn side(&self) -> Side {
        self.side
    }

    /// The coordinate of the edge.
    ///
    /// For left/right edges, this will be the x coordinate of the edge.
    /// For top/bottom edges, this will be the y coordinate of the edge.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.coord(), 20);
    /// ```
    pub fn coord(&self) -> i64 {
        self.coord
    }

    /// The span of the edge.
    ///
    /// For left/right edges, this will be the range of y-coordinates encompassed by the edge.
    /// For top/bottom edges, this will be the range of x-coordinates encompassed by the edge.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.span(), Span::new(40, 100));
    /// ```
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns an `Edge` with the same properties as the provided `Edge` but with a new span.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.span(), Span::new(40, 100));
    /// let edge_new = edge.with_span(Span::new(20, 100));
    /// assert_eq!(edge_new.span(), Span::new(20, 100));
    /// ```
    pub fn with_span(&self, span: Span) -> Edge {
        Edge { span, ..*self }
    }

    /// The direction perpendicular to the edge.
    ///
    /// For left/right edges, this will be [`Dir::Horiz`].
    /// For top/bottom edges, this will be [`Dir::Vert`].
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.norm_dir(), Dir::Horiz);
    /// let edge = Edge::new(Side::Right, 20, Span::new(40, 100));
    /// assert_eq!(edge.norm_dir(), Dir::Horiz);
    /// let edge = Edge::new(Side::Top, 20, Span::new(40, 100));
    /// assert_eq!(edge.norm_dir(), Dir::Vert);
    /// let edge = Edge::new(Side::Bot, 20, Span::new(40, 100));
    /// assert_eq!(edge.norm_dir(), Dir::Vert);
    /// ```
    pub fn norm_dir(&self) -> Dir {
        self.side.coord_dir()
    }

    /// The direction parallel to the edge.
    ///
    /// For left/right edges, this will be [`Dir::Vert`].
    /// For top/bottom edges, this will be [`Dir::Horiz`].
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.edge_dir(), Dir::Vert);
    /// let edge = Edge::new(Side::Right, 20, Span::new(40, 100));
    /// assert_eq!(edge.edge_dir(), Dir::Vert);
    /// let edge = Edge::new(Side::Top, 20, Span::new(40, 100));
    /// assert_eq!(edge.edge_dir(), Dir::Horiz);
    /// let edge = Edge::new(Side::Bot, 20, Span::new(40, 100));
    /// assert_eq!(edge.edge_dir(), Dir::Horiz);
    /// ```
    pub fn edge_dir(&self) -> Dir {
        self.side.edge_dir()
    }

    /// Returns a new [`Edge`] offset some amount **away** from this edge.
    ///
    /// Left edges will be offset to the left; right edges will be offset to the right.
    /// Top edges will be offset upwards; bottom edges will be offset downwards.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let edge = Edge::new(Side::Left, 20, Span::new(40, 100));
    /// assert_eq!(edge.offset(10), Edge::new(Side::Left, 10, Span::new(40, 100)));
    ///
    /// let edge = Edge::new(Side::Top, 20, Span::new(40, 100));
    /// assert_eq!(edge.offset(10), Edge::new(Side::Top, 30, Span::new(40, 100)));
    /// ```
    pub fn offset(&self, offset: i64) -> Edge {
        Edge {
            coord: self.coord + self.side.sign().as_int() * offset,
            ..*self
        }
    }
}
