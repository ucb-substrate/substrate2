use geometry::bbox::Bbox;
use geometry::rect::{self, Rect};
use std::rc::Rc;
use std::cell::RefCell;

use crate::CellId;
use crate::Instance;

use crate::{Cell, Library, Shape};
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

    fn get_cells<'a>(
        cell : &'a Cell<Self>,
        lib : &'a Library<Self>,
    ) -> Vec<&'a Cell<Self>> {
        let mut ret : Vec<&'a Cell<Self>> = vec![];

        for inst_pair in cell.instances() {
            let inst : &Instance = inst_pair.1;
            let cellid : CellId = inst.child();
            ret.push(lib.cell(cellid));
        }
        ret
    }

    fn flatten_cell<'a>(
        cell: &'a Cell<Self>,
        lib : &'a Library<Self>,
    ) -> Vec<&'a Shape<Self>> {
        let mut ret : Vec<&'a Shape<Self>> = vec![];

        for elem in cell.elements() {
            if let Element::Shape(shape) = elem {
                ret.push(shape);
            }
        }

        for inst in Self::get_cells(cell, lib) {
            for shape in Self::flatten_cell(inst, lib) {
                ret.push(shape);
            }
        }

        ret
    }

    
    fn vec_has<'a>(
        part : &'a Shape<Self>,
        part_list : &Vec<&'a Shape<Self>>,
    ) -> bool {
        let temp_list = part_list.clone();
        for thing in temp_list.into_iter() {
            if thing == part {
                return true;
            }
        }
        return false;
    }

    fn vec_union<'a>(
        vec1 : Vec<&'a Shape<Self>>,
        vec2 : Vec<&'a Shape<Self>>,
    ) -> Vec<&'a Shape<Self>> {
        let mut ret = vec1.clone();
        let temp_vec2 = vec2.clone();

        for thing in temp_vec2.into_iter() {
            if !(Self::vec_has(thing, &ret)) {
                ret.push(thing);
            }
        }
        ret
    }

    fn connected_components<'a>(
        cell : &'a Cell<Self>,
        lib : &'a Library<Self>,
    ) -> Vec<(&'a Shape<Self>, Rc<RefCell<Rc<RefCell<Vec<&'a Shape<Self>>>>>>)> {
        let all_shapes = Self::flatten_cell(cell, lib);
        let mut ret_refs: Vec<(&'a Shape<Self>, Rc<RefCell<Rc<RefCell<Vec<&'a Shape<Self>>>>>>)> = vec![];

        for thing in all_shapes.clone().into_iter() {
            let temp_vec = Rc::new(RefCell::new(Rc::new(RefCell::new(vec![thing]))));
            ret_refs.push((thing, temp_vec));
        }

        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (part_index, part_shape) in all_shapes.clone().into_iter().enumerate() {
                if start_index != part_index {
                    if Self::intersect_shapes(start_shape, part_shape) && start_shape.layer().connected(part_shape.layer()) {

                        let start_vec = &mut *ret_refs[start_index].1.borrow_mut();
                        let part_vec = &mut *ret_refs[part_index].1.borrow_mut();

                        let start_actual_vec = start_vec.borrow();
                        let part_actual_vec = part_vec.borrow();

                        let z = start_actual_vec.clone();
                        let h = part_actual_vec.clone();

                        let temp_vec : Rc<RefCell<Vec<&Shape<Self>>>> = Rc::new(RefCell::new(Self::vec_union(z, h)));

                        *ret_refs[start_index].1.borrow_mut() = temp_vec.clone();
                        *ret_refs[part_index].1.borrow_mut() = temp_vec.clone();


           
                    }
                }
            }
        }


        ret_refs
    }

}

#[cfg(test)]
mod tests {
    use geometry::rect::Rect;

    use crate::LibraryBuilder;

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

    #[test]
    fn test_complete() {
        let mut big_cell : Cell<Layer> = Cell::new("test");
        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let v1_shape = Shape::new(Layer::Via1, Rect::from_sides(10, 10, 100, 100));
        let m2_shape = Shape::new(Layer::Met2, Rect::from_sides(10, 10, 50, 50));
        
        big_cell.add_element(m1_shape.clone());
        big_cell.add_element(v1_shape.clone());
        big_cell.add_element(m2_shape.clone());

        assert_eq!(Layer::connected_shapes(&big_cell, &m1_shape), vec![&m1_shape, &v1_shape]);
        assert_eq!(Layer::connected_shapes(&big_cell, &v1_shape), vec![&m1_shape, &v1_shape, &m2_shape]);
        assert_eq!(Layer::connected_shapes(&big_cell, &m2_shape), vec![&v1_shape, &m2_shape]);
        
    }
}
