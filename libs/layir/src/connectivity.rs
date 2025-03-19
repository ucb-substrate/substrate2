use geometry::bbox::Bbox;
use geometry::rect::{self, Rect};
use aph_disjoint_set::DisjointSet;
use crate::CellId;
use crate::Instance;

use crate::{Cell, Library, Shape, LibraryBuilder};
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
        lib : &'a LibraryBuilder<Self>,
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
        lib : &'a LibraryBuilder<Self>,
    ) -> Vec<&'a Shape<Self>> {
        let mut ret : Vec<&'a Shape<Self>> = vec![];

        for elem in cell.elements() {
            if let Element::Shape(shape) = elem {
                ret.push(shape);
            }
        }

        let list_of_cells = Self::get_cells(cell, lib);

        for inst in list_of_cells.into_iter() {
            for thing in Self::flatten_cell(inst, lib).into_iter() {
                ret.push(thing);
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

    fn connected_components<'a>(
        cell : &'a Cell<Self>,
        lib : &'a LibraryBuilder<Self>,
    ) -> (Vec<&'a Shape<Self>>, Vec<Vec<&'a Shape<Self>>>) {
        
        let all_shapes = Self::flatten_cell(cell, lib);

        let mut djs  = DisjointSet::new(all_shapes.len());

        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (part_index, part_shape) in all_shapes.clone().into_iter().enumerate() {
                if start_index != part_index {
                    if Self::intersect_shapes(start_shape, part_shape) && start_shape.layer().connected(part_shape.layer()) {
                        djs.union(start_index, part_index);
                    }
                }
            }
        }

        let mut ret : Vec<Vec<&Shape<Self>>> = vec![vec![]; all_shapes.clone().len()];
        
        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (part_index, part_shape) in all_shapes.clone().into_iter().enumerate() {
                if djs.is_united(start_index, part_index) {
                    if !(Self::vec_has(part_shape, &ret[start_index])) {
                        ret[start_index].push(part_shape);
                    }
                    if !(Self::vec_has(start_shape, &ret[part_index])) {
                        ret[part_index].push(start_shape);
                    }
                }
            }
        }



        (all_shapes, ret)
    }

}

#[cfg(test)]
mod tests {
    use std::io::empty;

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
        let mut big_cell : Cell<Layer> = Cell::new("big cell test");
        let mut small_cell : Cell<Layer> = Cell::new("small cell test");
        let mut lib : LibraryBuilder<Layer> = LibraryBuilder::new();

        lib.add_cell(small_cell.clone());

        let small_cell_id = lib.cells().next().unwrap().0;

        lib.add_cell(big_cell.clone());

        let small_cell_instance = Instance::new(small_cell_id, "small_cell_test");
        big_cell.add_instance(small_cell_instance);

        

        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let v1_shape = Shape::new(Layer::Via1, Rect::from_sides(10, 10, 100, 100));
        let m2_shape = Shape::new(Layer::Met2, Rect::from_sides(10, 10, 50, 50));
        let m3_shape = Shape::new(Layer::Met2, Rect::from_sides(1000, 1000, 5000, 5000));
        let m4_shape = Shape::new(Layer::Met2, Rect::from_sides(0, 0, 30, 30));
        
        big_cell.add_element(m1_shape.clone());
        big_cell.add_element(v1_shape.clone());
        big_cell.add_element(m2_shape.clone());
        big_cell.add_element(m3_shape.clone());
        small_cell.add_element(m4_shape.clone());

        let asdf = Layer::get_cells(&big_cell, &lib);
        assert_eq!(asdf, vec![&small_cell]);

        let x = Layer::connected_components(&big_cell, &lib);

        assert_eq!(x.0, vec![&m1_shape, &v1_shape, &m2_shape, &m3_shape, &m4_shape]);
        assert_eq!(x.1[0], vec![&m1_shape, &v1_shape, &m2_shape]);
        assert_eq!(x.1[1], vec![&m1_shape, &v1_shape, &m2_shape]);
        assert_eq!(x.1[2], vec![&m1_shape, &v1_shape, &m2_shape]);
        assert_eq!(x.1[3], vec![&m3_shape]);
        assert_eq!(x.1[4], vec![&m1_shape, &v1_shape, &m2_shape, &m4_shape]);

    }
}
