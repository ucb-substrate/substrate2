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

    fn intersect_shapes<'a, 'b>(
        shape1 : &'a Shape<Self>,
        shape2 : &'b Shape<Self>,
    ) -> bool {
        let shape1_bbox : Rect = shape1.bbox().expect("error");
        let shape2_bbox : Rect = shape2.bbox().expect("error");
        shape1_bbox.intersection(shape2_bbox) != None
    } 



    fn connected_shapes<'a, 'b>(
        cell: &'a Cell<Self>,
        start: &'b Shape<Self>,
    ) -> Vec<&'a Shape<Self>> {
        
        let mut ret : Vec<&'a Shape<Self>> = vec![];

        for elem in cell.elements() {
           
            if let Element::Shape(shape) = elem {

                if start.layer() == shape.layer() && Self::intersect_shapes(start, shape) { //This is the same layer case
                    ret.push(shape);
                } else if start.layer().connected(shape.layer()) && Self::intersect_shapes(start, shape)  {//If layers are connected, and they intersect
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
        let m1_shape1 = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let m1_shape2 = Shape::new(Layer::Met1, Rect::from_sides(100, 50, 200, 100));
        let m1_shape3 = Shape::new(Layer::Met1, Rect::from_sides(200, 200, 400, 400));


        cell.add_element(m1_shape1.clone());
        cell.add_element(m1_shape2.clone());
        cell.add_element(m1_shape3.clone());

        assert_eq!(Layer::connected_shapes(&cell, &m1_shape2), vec![&m1_shape1, &m1_shape2]);
    }


    #[test]
    fn test_connectivity2() {
        let mut cell = Cell::new("test");
        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let v1_shape = Shape::new(Layer::Via1, Rect::from_sides(10, 10, 100, 100));
        let m2_shape = Shape::new(Layer::Met2, Rect::from_sides(10, 10, 50, 50));


        cell.add_element(m1_shape.clone());
        cell.add_element(v1_shape.clone());
        cell.add_element(m2_shape.clone());

        assert_eq!(Layer::connected_shapes(&cell, &m1_shape), vec![&m1_shape, &v1_shape]);
        assert_eq!(Layer::connected_shapes(&cell, &v1_shape), vec![&m1_shape, &v1_shape, &m2_shape]);
        assert_eq!(Layer::connected_shapes(&cell, &m2_shape), vec![&v1_shape, &m2_shape]);

    }
}
