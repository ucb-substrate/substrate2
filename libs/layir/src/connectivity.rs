use geometry::bbox::Bbox;
use geometry::contains::Containment;
use geometry::point::Point;
use geometry::prelude::Contains;
use geometry::rect::{self, Rect};
use geometry::union::BoundingUnion;

use crate::{Cell, Shape};
use crate::Element;
trait Connectivity: Sized + PartialEq {
    fn connected_layers(&self) -> Vec<Self>;

    fn connected(&self, other: &Self) -> bool {
        self.connected_layers().contains(other)
    }

    fn connected_shapes<'a, 'b>(
        cell: &'a Cell<Self>,
        start: &'b Shape<Self>,
    ) -> Vec<&'a Shape<Self>> {
        
        let mut ret : Vec<&'a Shape<Self>> = vec![];

        for elem in cell.elements() {
           
            if let Element::Shape(shape) = elem {
                let shape_bbox : Rect = shape.bbox().expect("REASON");
                let start_bbox : Rect = start.bbox().expect("REASON");

                if shape_bbox.intersection(start_bbox) != None {
                    ret.push(shape);
                }
            }
        }
        
        ret

    }
}

#[cfg(test)]
mod tests {
    use geometry::rect::Rect;

    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Layer {
        Met1,
        Via1,
        Met2,
    }

    impl Connectivity for Layer {
        fn connected_layers(&self) -> Vec<Self> {
            match self {
                Self::Met1 => vec![Self::Met1, Self::Via1],
                Self::Via1 => vec![Self::Met1, Self::Via1, Self::Met2],
                Self::Met2 => vec![Self::Via1, Self::Met2],
            }
        }
    }

    #[test]
    fn test_connectivity() {
        let mut cell = Cell::new("test");
        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        cell.add_element(m1_shape.clone());
        assert_eq!(Layer::connected_shapes(&cell, &m1_shape), vec![&m1_shape]);
    }
}
