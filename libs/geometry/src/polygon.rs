//! Integer coordinate polygons.

use serde::{Deserialize, Serialize};

use crate::bbox::Bbox;
use crate::point::Point;
use crate::rect::Rect;
use crate::transform::{TransformMut, Transformation, TranslateMut};

/// A polygon, with vertex coordinates given
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Polygon {
    /// Vector of points that make up the polygon.
    points: Vec<Point>,
}

impl Polygon {
    /// Creates a polygon with given vertices.
    pub fn from_verts(vec: Vec<Point>) -> Self {
        Self{points: vec}
    }

    /// Returns the bottom y-coordinate in the polygon.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.bot(), 0);
    /// ```
    pub fn bot(&self) -> i64 {
        self.points.iter().map(|point|point.y).min().unwrap() 
    }

    /// Returns the top y-coordinate in the polygon.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.top(), 5);
    /// ```
    pub fn top(&self) -> i64 {
        self.points.iter().map(|point|point.y).max().unwrap() 
    }

    /// Returns the leftmost x-coordinate in the polygon.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.left(), -4);
    /// ```
    pub fn left(&self) -> i64 {
        self.points.iter().map(|point|point.x).min().unwrap() 
    }

    /// Returns the rightmost x-coordinate in the polygon.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.right(), 1);
    /// ```
    pub fn right(&self) -> i64 {
        self.points.iter().map(|point|point.x).max().unwrap() 
    }

    /// Returns a the vector of points representing the polygon.
    pub fn points(&self) -> &Vec<Point> {
        &self.points
    }

    /// Returns the center point of the polygon.
    ///
    /// Returns a point with x-coordinate equal to the average of all x-coordinates
    /// and y-coordinate equal to the average of all y-coordinates.
    /// Note that the current behavior is to round down.
    /// 
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.center(), Point::new(-1, 2));
    /// ```
    pub fn center(&self) -> Point {
        let x = self.points.iter().map(|point|point.x).sum::<i64>()/self.points.len() as i64;
        let y = self.points.iter().map(|point|point.y).sum::<i64>()/self.points.len() as i64;
        Point::new(x, y)
    }
}

impl Bbox for Polygon {
    fn bbox(&self) -> Option<Rect> {
        match self {
            polygon => {
                Rect::from_sides_option(polygon.left(), polygon.bot(), polygon.right(), polygon.top())
            }
        }
    }
}

impl TranslateMut for Polygon {
    fn translate_mut(&mut self, p: Point) {
        self.points.translate_mut(p);
    }
}

impl TransformMut for Polygon {
    fn transform_mut(&mut self,trans:Transformation) {
        self.points.transform_mut(trans);
    }
}
