//! Axis-aligned rectangular bounding boxes.

use impl_trait_for_tuples::impl_for_tuples;

use crate::{polygon::Polygon, rect::Rect, union::BoundingUnion};

/// A geometric shape that has a bounding box.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let rect = Rect::from_sides(0, 0, 100, 200);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(0, 0, 100, 200)));
/// let rect = Rect::from_xy(50, 70);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(50, 70, 50, 70)));
/// ```
pub trait Bbox {
    /// Computes the axis-aligned rectangular bounding box.
    ///
    /// If empty, this method should return `None`.
    /// Note that poinst and zero-area rectangles are not empty:
    /// these shapes contain a single point, and their bounding box
    /// implementations will return `Some(_)`.
    fn bbox(&self) -> Option<Rect>;

    /// Computes the axis-aligned rectangular bounding box, panicking
    /// if it is empty.
    fn bbox_rect(&self) -> Rect {
        self.bbox().unwrap()
    }
}

impl<T> Bbox for &T
where
    T: Bbox,
{
    fn bbox(&self) -> Option<Rect> {
        T::bbox(*self)
    }
}

#[impl_for_tuples(64)]
impl Bbox for TupleIdentifier {
    #[allow(clippy::let_and_return)]
    fn bbox(&self) -> Option<Rect> {
        let mut bbox = None;
        for_tuples!( #( bbox = bbox.bounding_union(&TupleIdentifier.bbox()); )* );
        bbox
    }
}

impl<T: Bbox> Bbox for Vec<T> {
    fn bbox(&self) -> Option<Rect> {
        let mut bbox = None;
        for item in self {
            bbox = bbox.bounding_union(&item.bbox());
        }
        bbox
    }
}

impl Bbox for Option<Rect> {
    fn bbox(&self) -> Option<Rect> {
        *self
    }
}

impl Bbox for Option<Polygon> {
    fn bbox(&self) -> Option<Rect> {
        match self {
            Some(polygon) => Rect::from_sides_option(
                polygon.left(),
                polygon.bot(),
                polygon.right(),
                polygon.top(),
            ),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bbox::Bbox, point::Point, polygon::Polygon, rect::Rect};

    #[test]
    fn bbox_works_for_tuples() {
        let tuple = (
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(-50, 20, 90, 250),
        );
        assert_eq!(tuple.bbox(), Some(Rect::from_sides(-50, 0, 100, 250)));
    }

    #[test]
    fn bbox_works_for_vecs() {
        let v = vec![
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(-50, 20, 90, 250),
        ];
        assert_eq!(v.bbox(), Some(Rect::from_sides(-50, 0, 100, 250)));
    }

    #[test]
    fn bbox_works_for_polygon() {
        let points = vec![
            Point { x: -10, y: 25 },
            Point { x: 0, y: 16 },
            Point { x: 40, y: -20 },
        ];
        let polygon = Polygon::from_verts(points);
        assert_eq!(polygon.bbox(), Some(Rect::from_sides(-10, -20, 40, 25)));
    }

    #[test]
    fn bbox_works_for_diff_types() {
        let points = vec![
            Point { x: -10, y: 25 },
            Point { x: 0, y: 16 },
            Point { x: 40, y: -20 },
        ];
        let tuple: (Rect, Polygon) = (
            Rect::from_sides(0, 0, 100, 200),
            Polygon::from_verts(points),
        );
        assert_eq!(tuple.bbox(), Some(Rect::from_sides(-10, -20, 100, 200)));
    }
}
