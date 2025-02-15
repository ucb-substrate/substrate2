//! Integer coordinate polygons.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::bbox::Bbox;
use crate::contains::{Containment, Contains};
use crate::point::Point;
use crate::rect::Rect;
use crate::transform::{TransformMut, TransformRef, Transformation, TranslateMut, TranslateRef};

/// A polygon, with vertex coordinates given
#[derive(Debug, Default, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

impl TranslateRef for Polygon {
    fn translate_ref(&self, p: Point) -> Self {
        Self {
            points: self.points.translate_ref(p),
        }
    }
}

impl TranslateMut for Polygon {
    fn translate_mut(&mut self, p: Point) {
        self.points.translate_mut(p);
    }
}

impl TransformRef for Polygon {
    fn transform_ref(&self, trans: Transformation) -> Self {
        Self {
            points: self.points.transform_ref(trans),
        }
    }
}

impl TransformMut for Polygon {
    fn transform_mut(&mut self, trans: Transformation) {
        self.points.transform_mut(trans);
    }
}

impl Contains<Point> for Polygon {
    /// Determines if a point is contained within a polygon.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let points = vec![
    ///     Point { x: -4, y: 0 },
    ///     Point { x: 0, y: 0 },
    ///     Point { x: 1, y: 2 },
    ///     Point { x: 2, y: 2 },
    ///     Point { x: -4, y: 5 },
    /// ];
    /// let p1 = Point::new(0,0);
    /// let p2 = Point::new(0,4);
    /// let p3 = Point::new(-5,3);
    /// let p4 = Point::new(-2,4);
    /// let p5 = Point::new(-2,2);
    /// let polygon = Polygon::from_verts(points);
    /// assert_eq!(polygon.contains(&p1), Containment::Full);
    /// assert_eq!(polygon.contains(&p2), Containment::None);
    /// assert_eq!(polygon.contains(&p3), Containment::None);
    /// assert_eq!(polygon.contains(&p4), Containment::Full);
    /// assert_eq!(polygon.contains(&p5), Containment::Full);
    /// ```
    fn contains(&self, p: &Point) -> Containment {
        use std::cmp::{max, min};
        let mut contains = false;
        for i in 0..self.points().len() {
            let p0 = self.points[i];
            let p1 = self.points[(i + 1) % self.points.len()];
            // Ensure counter-clockwise ordering.
            let (p0, p1) = if p0.y < p1.y { (p0, p1) } else { (p1, p0) };
            let miny = min(p0.y, p1.y);
            let maxy = max(p0.y, p1.y);
            if p.y >= miny && p.y <= maxy {
                let dy = p1.y - p0.y;
                let dx = p1.x - p0.x;
                let test = -(p.x - p0.x) * dy + (p.y - p0.y) * dx;
                match test.cmp(&0) {
                    Ordering::Greater => {
                        // Note: this edge cannot be horizontal.
                        if p.y > miny {
                            contains = !contains;
                        }
                    }
                    Ordering::Equal => {
                        if p.x >= min(p0.x, p1.x) && p.x <= max(p0.x, p1.x) {
                            return Containment::Full;
                        }
                    }
                    _ => {}
                }
            }
        }

        if contains {
            Containment::Full
        } else {
            Containment::None
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::prelude::*;

    #[test]
    fn point_in_polygon() {
        let points = vec![
            Point { x: -4, y: 0 },
            Point { x: 0, y: 0 },
            Point { x: 1, y: 2 },
            Point { x: 2, y: 2 },
            Point { x: -4, y: 5 },
        ];
        let p1 = Point::new(0, 0);
        let p2 = Point::new(0, 4);
        let p3 = Point::new(-5, 3);
        let p4 = Point::new(-2, 4);
        let p5 = Point::new(-2, 2);
        let polygon = Polygon::from_verts(points);
        assert_eq!(polygon.contains(&p1), Containment::Full);
        assert_eq!(polygon.contains(&p2), Containment::None);
        assert_eq!(polygon.contains(&p3), Containment::None);
        assert_eq!(polygon.contains(&p4), Containment::Full);
        assert_eq!(polygon.contains(&p5), Containment::Full);

        let points = vec![
            Point::new(-10, 0),
            Point::new(-10, -10),
            Point::new(10, -10),
            Point::new(10, 0),
            Point::new(5, 0),
            Point::new(5, 5),
            Point::new(-5, 5),
            Point::new(-5, 0),
        ];

        let polygon = Polygon::from_verts(points);
        assert_eq!(polygon.contains(&Point::new(-10, 5)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(-10, 3)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(0, 5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(5, 5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(0, 2)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(-5, -5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(-10, -10)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(-10, -12)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(12, -10)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(7, 3)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(5, 3)), Containment::Full);

        let points = vec![
            Point::new(0, 0),
            Point::new(0, 5),
            Point::new(10, 5),
            Point::new(10, 0),
        ];

        let polygon = Polygon::from_verts(points);
        assert_eq!(polygon.contains(&Point::new(-1, 0)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(0, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(3, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(10, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(12, 0)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(-1, 5)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(0, 5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(3, 5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(10, 5)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(12, 5)), Containment::None);

        let points = vec![Point::new(0, 0), Point::new(10, 10), Point::new(20, 0)];

        let polygon = Polygon::from_verts(points);
        assert_eq!(polygon.contains(&Point::new(0, 10)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(10, 10)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(20, 10)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(3, 3)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(14, 6)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(2, 1)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(12, 7)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(-2, 0)), Containment::None);
        assert_eq!(polygon.contains(&Point::new(0, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(10, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(20, 0)), Containment::Full);
        assert_eq!(polygon.contains(&Point::new(21, 0)), Containment::None);
    }
}
