//! A horizontal and vertical rectangular dimension with no specified location.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::dir::Dir;
use crate::point::Point;
use crate::rect::Rect;

/// A horizontal and vertical rectangular dimension with no specified location.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct Dims {
    /// The width dimension.
    w: i64,
    /// The height dimension.
    h: i64,
}

impl Dims {
    /// Creates a new [`Dims`] from a width and height.
    pub fn new(w: i64, h: i64) -> Self {
        Self { w, h }
    }
    /// Creates a new [`Dims`] with width and height equal to `value`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(Dims::square(100), Dims::new(100, 100));
    /// ```
    pub fn square(value: i64) -> Self {
        Self { w: value, h: value }
    }

    /// Returns the dimension in the specified direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let dims = Dims::new(100, 200);
    /// assert_eq!(dims.dim(Dir::Vert), 200);
    /// assert_eq!(dims.dim(Dir::Horiz), 100);
    /// ```
    pub fn dim(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Vert => self.h,
            Dir::Horiz => self.w,
        }
    }

    /// Returns the direction of the longer dimension.
    ///
    /// If the width and height are equal, returns [`Dir::Horiz`].
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let dims = Dims::new(100, 200);
    /// assert_eq!(dims.longer_dir(), Dir::Vert);
    /// let dims = Dims::new(200, 100);
    /// assert_eq!(dims.longer_dir(), Dir::Horiz);
    /// let dims = Dims::new(100, 100);
    /// assert_eq!(dims.longer_dir(), Dir::Horiz);
    /// ```
    pub fn longer_dir(&self) -> Dir {
        if self.w >= self.h {
            Dir::Horiz
        } else {
            Dir::Vert
        }
    }

    /// Returns the direction of the longer dimension.
    ///
    /// If the width and height are equal, returns [`None`].
    /// Otherwise, returns a `Some` variant containing the longer direction.
    pub fn longer_dir_strict(&self) -> Option<Dir> {
        match self.w.cmp(&self.h) {
            Ordering::Greater => Some(Dir::Horiz),
            Ordering::Equal => None,
            Ordering::Less => Some(Dir::Vert),
        }
    }

    /// Returns a new [`Dims`] object with the horizontal and vertical dimensions flipped.
    pub fn transpose(self) -> Self {
        Self {
            w: self.h,
            h: self.w,
        }
    }

    /// Returns the width (i.e. the horizontal dimension).
    #[inline]
    pub fn width(&self) -> i64 {
        self.w
    }

    /// Returns the height (i.e. the vertical dimension).
    #[inline]
    pub fn height(&self) -> i64 {
        self.h
    }

    /// Returns the width (i.e. the horizontal dimension).
    ///
    /// A shorthand for [`Dims::width`].
    #[inline]
    pub fn w(&self) -> i64 {
        self.width()
    }

    /// Returns the height (i.e. the vertical dimension).
    ///
    /// A shorthand for [`Dims::height`].
    #[inline]
    pub fn h(&self) -> i64 {
        self.height()
    }

    /// Converts this dimension object into a [`Rect`].
    ///
    /// See [`Rect::from_dims`] for more information.
    #[inline]
    pub fn into_rect(self) -> Rect {
        Rect::from_dims(self)
    }

    /// Converts this dimension object into a [`Point`] with coordinates `(self.w(), self.h())`.
    #[inline]
    pub fn into_point(self) -> Point {
        Point::new(self.w(), self.h())
    }
}

impl std::ops::Add<Dims> for Dims {
    type Output = Self;
    fn add(self, rhs: Dims) -> Self::Output {
        Self {
            w: self.w + rhs.w,
            h: self.h + rhs.h,
        }
    }
}

impl std::ops::Sub<Dims> for Dims {
    type Output = Self;
    fn sub(self, rhs: Dims) -> Self::Output {
        Self {
            w: self.w - rhs.w,
            h: self.h - rhs.h,
        }
    }
}

impl std::ops::Mul<i64> for Dims {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            w: self.w * rhs,
            h: self.h * rhs,
        }
    }
}

impl std::ops::Mul<(usize, usize)> for Dims {
    type Output = Self;
    fn mul(self, rhs: (usize, usize)) -> Self::Output {
        Self {
            w: self.w * rhs.0 as i64,
            h: self.h * rhs.1 as i64,
        }
    }
}

impl std::ops::AddAssign<Dims> for Dims {
    fn add_assign(&mut self, rhs: Dims) {
        self.w += rhs.w;
        self.h += rhs.h;
    }
}

impl std::ops::SubAssign<Dims> for Dims {
    fn sub_assign(&mut self, rhs: Dims) {
        self.w -= rhs.w;
        self.h -= rhs.h;
    }
}

impl std::ops::MulAssign<i64> for Dims {
    fn mul_assign(&mut self, rhs: i64) {
        self.w *= rhs;
        self.h *= rhs;
    }
}

impl From<Rect> for Dims {
    /// Obtains [`Dims`] from the given [`Rect`] using [`Rect::dims`].
    #[inline]
    fn from(value: Rect) -> Self {
        value.dims()
    }
}

impl From<Point> for Dims {
    /// Create a new dimension object from a point.
    ///
    /// The width field of the resulting [`Dims`] will be the point's x-coordinate.
    /// The height field of the resulting [`Dims`] will be the point's y-coordinate.
    #[inline]
    fn from(value: Point) -> Self {
        Self::new(value.x, value.y)
    }
}
