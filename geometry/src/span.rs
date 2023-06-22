//! A one-dimensional span.
//!
//! A span represents the closed interval `[start, stop]`.
use serde::{Deserialize, Serialize};

use crate::contains::{Containment, Contains};
use crate::intersect::Intersect;
use crate::sign::Sign;
use crate::snap::snap_to_grid;
use crate::union::BoundingUnion;

/// A closed interval of coordinates in one dimension.
///
/// Represents the range `[start, stop]`.
#[derive(
    Debug, Default, Clone, Copy, Hash, Ord, PartialOrd, Serialize, Deserialize, PartialEq, Eq,
)]
pub struct Span {
    start: i64,
    stop: i64,
}

impl Span {
    /// Creates a new [`Span`] from 0 until the specified stop.
    ///
    /// # Panics
    ///
    /// This function panics if `stop` is less than 0.
    pub const fn until(stop: i64) -> Self {
        assert!(stop >= 0);
        Self { start: 0, stop }
    }

    /// Creates a new [`Span`] between two integers.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `start` is less
    /// than or equal to `stop`.
    pub const unsafe fn new_unchecked(start: i64, stop: i64) -> Self {
        Self { start, stop }
    }

    /// Creates a new [`Span`] between two integers.
    pub fn new(start: i64, stop: i64) -> Self {
        use std::cmp::{max, min};
        let lower = min(start, stop);
        let upper = max(start, stop);
        Self {
            start: lower,
            stop: upper,
        }
    }

    /// Creates a span of zero length encompassing the given point.
    pub const fn from_point(x: i64) -> Self {
        Self { start: x, stop: x }
    }

    /// Creates a span of the given length starting from `start`.
    pub const fn with_start_and_length(start: i64, length: i64) -> Self {
        Self {
            stop: start + length,
            start,
        }
    }

    /// Creates a span of the given length ending at `stop`.
    pub const fn with_stop_and_length(stop: i64, length: i64) -> Self {
        Self {
            start: stop - length,
            stop,
        }
    }

    /// Creates a span with the given endpoint and length.
    ///
    /// If `sign` is [`Sign::Pos`], `point` is treated as the ending/stopping point of the span.
    /// If `sign` is [`Sign::Neg`], `point` is treated as the beginning/starting point of the span.
    pub const fn with_point_and_length(sign: Sign, point: i64, length: i64) -> Self {
        match sign {
            Sign::Pos => Self::with_stop_and_length(point, length),
            Sign::Neg => Self::with_start_and_length(point, length),
        }
    }

    /// Creates a new [`Span`] expanded by `amount` in the direction indicated by `pos`.
    pub const fn expand(mut self, sign: Sign, amount: i64) -> Self {
        match sign {
            Sign::Pos => self.stop += amount,
            Sign::Neg => self.start -= amount,
        }
        self
    }

    /// Creates a new [`Span`] expanded by `amount` in both directions.
    pub const fn expand_all(mut self, amount: i64) -> Self {
        self.stop += amount;
        self.start -= amount;
        self
    }

    /// Gets the starting ([`Sign::Neg`]) or stopping ([`Sign::Pos`]) endpoint of a span.
    #[inline]
    pub const fn endpoint(&self, sign: Sign) -> i64 {
        match sign {
            Sign::Neg => self.start(),
            Sign::Pos => self.stop(),
        }
    }

    /// Gets the shortest distance between this span and a point.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let span = Span::new(10, 20);
    /// assert_eq!(span.dist_to(4), 6);
    /// assert_eq!(span.dist_to(12), 0);
    /// assert_eq!(span.dist_to(27), 7);
    /// ```
    pub fn dist_to(&self, point: i64) -> i64 {
        if point < self.start() {
            self.start() - point
        } else if point > self.stop() {
            point - self.stop()
        } else {
            0
        }
    }

    /// Creates a new [`Span`] with center `center` and length `span`.
    ///
    /// `span` must be a positive, even integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let span = Span::from_center_span(0, 40);
    /// assert_eq!(span, Span::new(-20, 20));
    /// ```
    ///
    /// # Panics
    ///
    /// Passing an odd `span` to this method results in a panic:
    ///
    /// ```should_panic
    /// # use geometry::prelude::*;
    /// let span = Span::from_center_span(0, 25);
    /// ```
    ///
    /// Passing a negative `span` to this method also results in a panic:
    ///
    /// ```should_panic
    /// # use geometry::prelude::*;
    /// let span = Span::from_center_span(0, -200);
    /// ```
    pub fn from_center_span(center: i64, span: i64) -> Self {
        assert!(span >= 0);
        assert_eq!(span % 2, 0);

        Self::new(center - (span / 2), center + (span / 2))
    }

    /// Creates a new [`Span`] with center `center` and length `span` and snap the edges to the
    /// grid.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let span = Span::from_center_span_gridded(0, 40, 5);
    /// assert_eq!(span, Span::new(-20, 20));
    ///
    /// let span = Span::from_center_span_gridded(35, 40, 5);
    /// assert_eq!(span, Span::new(15, 55));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if `span` is negative, odd, or not an integer multiple of `grid`.
    pub fn from_center_span_gridded(center: i64, span: i64, grid: i64) -> Self {
        assert!(span >= 0);
        assert_eq!(span % 2, 0);
        assert_eq!(span % grid, 0);

        let start = snap_to_grid(center - (span / 2), grid);

        Self::new(start, start + span)
    }

    /// Gets the center of the span.
    #[inline]
    pub const fn center(&self) -> i64 {
        (self.start + self.stop) / 2
    }

    /// Gets the length of the span.
    #[inline]
    pub const fn length(&self) -> i64 {
        self.stop - self.start
    }

    /// Gets the start of the span.
    #[inline]
    pub const fn start(&self) -> i64 {
        self.start
    }

    /// Gets the stop of the span.
    #[inline]
    pub const fn stop(&self) -> i64 {
        self.stop
    }

    /// Checks if the span intersects with the [`Span`] `other`.
    #[inline]
    pub const fn intersects(&self, other: &Self) -> bool {
        !(other.stop < self.start || self.stop < other.start)
    }

    /// Creates a new minimal [`Span`] that contains all of the elements of `spans`.
    pub fn merge(spans: impl IntoIterator<Item = Self>) -> Self {
        use std::cmp::{max, min};
        let mut spans = spans.into_iter();
        let (mut start, mut stop) = spans
            .next()
            .expect("Span::merge requires at least one span")
            .into();

        for span in spans {
            start = min(start, span.start);
            stop = max(stop, span.stop);
        }

        assert!(start <= stop);

        Span { start, stop }
    }

    /// Merges adjacent spans when `merge_fn` evaluates to true.
    #[doc(hidden)]
    pub fn merge_adjacent(
        spans: impl IntoIterator<Item = Self>,
        mut merge_fn: impl FnMut(Span, Span) -> bool,
    ) -> impl Iterator<Item = Span> {
        let mut spans: Vec<Span> = spans.into_iter().collect();
        spans.sort_by_key(|span| span.start());

        let mut merged_spans = Vec::new();

        let mut j = 0;
        while j < spans.len() {
            let mut curr_span = spans[j];
            j += 1;
            while j < spans.len() && merge_fn(curr_span, spans[j]) {
                curr_span = curr_span.union(spans[j]);
                j += 1;
            }
            merged_spans.push(curr_span);
        }

        merged_spans.into_iter()
    }

    /// Calculates the smallest interval containing this span and `other`.
    pub fn union(self, other: Self) -> Self {
        use std::cmp::{max, min};
        Self {
            start: min(self.start, other.start),
            stop: max(self.stop, other.stop),
        }
    }

    /// Calculates the minimal bounding interval of all spans provided.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let spans = vec![
    ///     Span::new(10, 40),
    ///     Span::new(35, 60),
    ///     Span::new(20, 30),
    ///     Span::new(-10, 5),
    /// ];
    /// assert_eq!(Span::union_all(spans.into_iter()), Span::new(-10, 60));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if the provided iterator has no elements.
    /// If your iterator may be empty, consider using [`Span::union_all_option`].
    pub fn union_all<T>(spans: impl Iterator<Item = T>) -> Self
    where
        T: Into<Self>,
    {
        spans
            .fold(None, |acc: Option<Span>, s| match acc {
                Some(acc) => Some(acc.union(s.into())),
                None => Some(s.into()),
            })
            .unwrap()
    }

    /// Calculates the minimal bounding interval of all `Option<Span>`s provided.
    ///
    /// All `None` elements in the iterator are ignored.
    /// If the iterator has no `Some(_)` elements, this function returns [`None`].
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let spans = vec![
    ///     Some(Span::new(10, 40)),
    ///     Some(Span::new(35, 60)),
    ///     None,
    ///     Some(Span::new(20, 30)),
    ///     None,
    ///     Some(Span::new(-10, 5)),
    /// ];
    /// assert_eq!(Span::union_all_option(spans.into_iter()), Some(Span::new(-10, 60)));
    /// ```
    pub fn union_all_option<T>(spans: impl Iterator<Item = T>) -> Option<Self>
    where
        T: Into<Option<Self>>,
    {
        spans
            .filter_map(|s| s.into())
            .fold(None, |acc, s| match acc {
                Some(acc) => Some(acc.union(s)),
                None => Some(s),
            })
    }

    /// Calculates the intersection of this span with `other`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let _start = std::cmp::max(self.start(), other.start());
        let _stop = std::cmp::min(self.stop(), other.stop());
        if _start > _stop {
            None
        } else {
            Some(Self::new(_start, _stop))
        }
    }

    /// Returns a new [`Span`] representing the union of the current span with the given point.
    pub fn add_point(self, pos: i64) -> Self {
        use std::cmp::{max, min};
        Self {
            start: min(self.start, pos),
            stop: max(self.stop, pos),
        }
    }

    /// Shrinks the given side by the given amount.
    ///
    /// Behavior is controlled by the given [`Sign`]:
    /// * If `side` is [`Sign::Pos`], shrinks from the positive end (ie. decreases the `stop`).
    /// * If `side` is [`Sign::Neg`], shrinks from the negative end (ie. increases the `start`).
    pub fn shrink(self, side: Sign, amount: i64) -> Self {
        assert!(self.length() >= amount);
        match side {
            Sign::Pos => Self::new(self.start, self.stop - amount),
            Sign::Neg => Self::new(self.start + amount, self.stop),
        }
    }

    /// Shrinks the span by the given amount on all sides.
    pub const fn shrink_all(self, amount: i64) -> Self {
        assert!(self.length() >= 2 * amount);
        Self {
            start: self.start + amount,
            stop: self.stop - amount,
        }
    }

    /// Translates the span by the given amount.
    pub const fn translate(self, amount: i64) -> Self {
        Self {
            start: self.start + amount,
            stop: self.stop + amount,
        }
    }

    /// The minimum separation between this span and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let s1 = Span::new(10, 20);
    /// let s2 = Span::new(30, 50);
    /// let s3 = Span::new(25, 40);
    /// assert_eq!(s1.dist_to_span(s2), 10);
    /// assert_eq!(s1.dist_to_span(s3), 5);
    /// assert_eq!(s2.dist_to_span(s3), 0);
    /// assert_eq!(s3.dist_to_span(s3), 0);
    /// ```
    pub fn dist_to_span(self, other: Span) -> i64 {
        std::cmp::max(
            0,
            self.union(other).length() - self.length() - other.length(),
        )
    }

    /// Returns whether the span's center is at an integer coordinate.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let span = Span::new(0, 100);
    /// assert!(span.has_integer_center());
    ///
    /// let span = Span::new(0, 99);
    /// assert!(!span.has_integer_center());
    /// ```
    pub fn has_integer_center(&self) -> bool {
        (self.start() + self.stop()) % 2 == 0
    }
}

impl Intersect<Span> for Span {
    type Output = Self;
    fn intersect(&self, other: &Span) -> Option<Self::Output> {
        self.intersection(*other)
    }
}

impl Contains<Span> for Span {
    fn contains(&self, other: &Span) -> Containment {
        if other.start() >= self.start() && other.stop() <= self.stop() {
            Containment::Full
        } else if other.start() <= self.stop() || other.stop() >= self.start() {
            Containment::Partial
        } else {
            Containment::None
        }
    }
}

impl BoundingUnion<Span> for Span {
    type Output = Span;
    fn bounding_union(&self, other: &Span) -> Self::Output {
        self.union(*other)
    }
}

impl From<(i64, i64)> for Span {
    #[inline]
    fn from(tup: (i64, i64)) -> Self {
        Self::new(tup.0, tup.1)
    }
}

impl From<Span> for (i64, i64) {
    #[inline]
    fn from(s: Span) -> Self {
        (s.start(), s.stop())
    }
}
