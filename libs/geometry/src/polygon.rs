//! Integer coordinate polygons.

use serde::{Deserialize, Serialize};


use crate::point::Point;


/// A polygon, with vertex coordinates given
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]


pub struct Polygon {
    ///vector of points
    verticies: Vec<Point>,
}
impl Polygon {
    /// Creates a polygon with given verticies.
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
    /// let polygon = Polygon:from_verts(points);
    /// assert_eq!(polygon.stuff, stuff);
    /// ```
    pub fn from_verts(vec: Vec<Point>) -> Self {
        Self{verticies: vec}
    }
}