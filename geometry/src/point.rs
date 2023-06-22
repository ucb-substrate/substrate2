//! 2-D points.

use serde::{Deserialize, Serialize};

use crate::dir::Dir;
use crate::snap::snap_to_grid;
use crate::transform::{TransformMut, Transformation, TranslateMut};

/// A point in two-dimensional space.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    /// The x-coordinate of the point.
    pub x: i64,
    /// The y-coordinate of the point.
    pub y: i64,
}

impl Point {
    /// Creates a new [`Point`] from (x,y) coordinates.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// Creates a new point from the given direction and coordinates.
    ///
    /// If `dir` is [`Dir::Horiz`], `a` becomes the x-coordinate and `b` becomes the y-coordinate.
    /// If `dir` is [`Dir::Vert`], `a` becomes the y-coordinate and `b` becomes the x-coordinate.
    pub const fn from_dir_coords(dir: Dir, a: i64, b: i64) -> Self {
        match dir {
            Dir::Horiz => Self::new(a, b),
            Dir::Vert => Self::new(b, a),
        }
    }

    /// Returns the origin, `(0, 0)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let origin = Point::zero();
    /// assert_eq!(origin, Point::new(0, 0));
    /// ```
    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Gets the coordinate associated with direction `dir`.
    pub const fn coord(&self, dir: Dir) -> i64 {
        match dir {
            Dir::Horiz => self.x,
            Dir::Vert => self.y,
        }
    }

    /// Snaps the x and y coordinates of this point to the nearest multiple of `grid`.
    #[inline]
    pub fn snap_to_grid(&self, grid: i64) -> Self {
        self.snap_x_to_grid(grid).snap_y_to_grid(grid)
    }

    /// Snaps only the x-coordinate of this point to the nearest multiple of `grid`.
    #[inline]
    pub fn snap_x_to_grid(&self, grid: i64) -> Self {
        let x = snap_to_grid(self.x, grid);
        Self { x, y: self.y }
    }

    /// Snaps only the y-coordinate of this point to the nearest multiple of `grid`.
    #[inline]
    pub fn snap_y_to_grid(&self, grid: i64) -> Self {
        let y = snap_to_grid(self.y, grid);
        Self { x: self.x, y }
    }
}

impl TranslateMut for Point {
    fn translate_mut(&mut self, p: Point) {
        self.x += p.x;
        self.y += p.y;
    }
}

impl TransformMut for Point {
    fn transform_mut(&mut self, trans: Transformation) {
        let xf = self.x as f64;
        let yf = self.y as f64;
        let x = trans.a[0][0] * xf + trans.a[0][1] * yf + trans.b[0];
        let y = trans.a[1][0] * xf + trans.a[1][1] * yf + trans.b[1];
        self.x = x.round() as i64;
        self.y = y.round() as i64;
    }
}
