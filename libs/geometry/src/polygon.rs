//! Integer coordinate polygons.

use serde::{Deserialize, Serialize};

use crate::bbox::Bbox;
use crate::contains::{Containment, Contains};
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
        Self { points: vec }
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
        self.points.iter().map(|point| point.y).min().unwrap()
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
        self.points.iter().map(|point| point.y).max().unwrap()
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
        self.points.iter().map(|point| point.x).min().unwrap()
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
        self.points.iter().map(|point| point.x).max().unwrap()
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
        let x = self.points.iter().map(|point| point.x).sum::<i64>() / self.points.len() as i64;
        let y = self.points.iter().map(|point| point.y).sum::<i64>() / self.points.len() as i64;
        Point::new(x, y)
    }

    /// Helper function that checks if a point is contained within a triangle
    ///
    /// # Example
    ///
    /// ```
    /// use geometry::prelude::*;
    /// 
    /// 
    /// ```
    pub fn traingle_contains(p: Point, v1: Point, v2: Point, v3: Point) -> bool {
        let total_area = Polygon::triangle_area(v1, v2, v3);
        let sum_area = Polygon::triangle_area(p, v2, v3) + Polygon::triangle_area(v1, p, v3) + Polygon::triangle_area(v1, v2, p);
        return sum_area == total_area;
    }

    /// Helper function that finds the area of a given triangle
    ///
    /// # Example
    ///
    /// ```
    /// use geometry::prelude::*;
    /// 
    /// 
    /// ```
    pub fn triangle_area(v1: Point, v2: Point, v3: Point) -> f32 {
        return ((v1.x*(v2.y-v3.y) + v2.x*(v3.y-v1.y)+ v3.x*(v1.y-v2.y)) as f32 / 2.0).abs();
    }
}

impl Bbox for Polygon {
    fn bbox(&self) -> Option<Rect> {
        let polygon = self;
        Rect::from_sides_option(
            polygon.left(),
            polygon.bot(),
            polygon.right(),
            polygon.top(),
        )
    }
}

impl TranslateMut for Polygon {
    fn translate_mut(&mut self, p: Point) {
        self.points.translate_mut(p);
    }
}

impl TransformMut for Polygon {
    fn transform_mut(&mut self, trans: Transformation) {
        self.points.transform_mut(trans);
    }
}



impl Contains<Point> for Polygon {
        fn contains(&self, p: &Point) -> Containment {
        for (index, _) in self.points.iter().skip(1).enumerate() {
            let v1 = self.points.get(0).unwrap();
            let v2 = self.points.get(index).unwrap();
            let v3 = self.points.get((index+1)%self.points.len()).unwrap();
            if Polygon::traingle_contains(*p, *v1, *v2, *v3) == true {
                return Containment::Full
            }
        }
        Containment::None
    }
}