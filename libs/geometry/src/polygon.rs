//! Integer coordinate polygons.

use serde::{Deserialize, Serialize};

use crate::bbox::Bbox;
use crate::point::Point;
use crate::rect::Rect;
use crate::transform::{TransformMut, Transformation, TranslateMut};

/// A polygon, with vertex coordinates given
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]


pub struct Polygon {
    ///vector of points
    points: Vec<Point>,
}
impl Polygon {
    /// Creates a polygon with given vertices.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: 0.0, y: 0.0 },
    ///     Point { x: 1.0, y: 2.5 },
    ///     Point { x: -3.2, y: 5.7 },
    /// ];
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.stuff, stuff);
    /// ```
    pub fn from_verts(vec: Vec<Point>) -> Self {
        Self{points: vec}
    }

    pub fn bot(&self) -> i64 {
        self.points.iter().map(|point|point.y).min().unwrap() 
    }

    pub fn top(&self) -> i64 {
        self.points.iter().map(|point|point.y).max().unwrap() 
    }

    pub fn left(&self) -> i64 {
        self.points.iter().map(|point|point.x).min().unwrap() 
    }

    pub fn right(&self) -> i64 {
        self.points.iter().map(|point|point.x).max().unwrap() 
    }

    pub fn points(&self) -> &Vec<Point> {
        &self.points
    }

    pub fn center(&self) -> Point {
        let mut x = self.points.iter().map(|point|point.x).sum::<i64>()/self.points.len() as i64;
        let mut y = self.points.iter().map(|point|point.x).sum::<i64>()/self.points.len() as i64;
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
