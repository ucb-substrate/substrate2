use crate::CellId;
use crate::Instance;
use aph_disjoint_set::DisjointSet;
use geometry::bbox::Bbox;
use geometry::rect::{self, Rect};
use std::collections::HashSet;
use std::hash::Hash;

use crate::Element;
use crate::{Cell, Library, LibraryBuilder, Shape, TransformMut, TransformRef, Transformation};

/// Returns true if two shapes overlap on a flat plane, and false otherwise.
fn intersect_shapes<L>(shape1: &Shape<L>, shape2: &Shape<L>) -> bool {
    let shape1_bbox: Rect = shape1.bbox_rect();
    let shape2_bbox: Rect = shape2.bbox_rect();
    shape1_bbox.intersection(shape2_bbox) != None
}

/// Returns a vector of references to all direct child cells instances of a given cell.
fn get_child_cells<'a, L>(cell: &'a Cell<L>, lib: &'a Library<L>) -> Vec<&'a Cell<L>> {
    let mut ret: Vec<&Cell<L>> = vec![];

    for inst_pair in cell.instances() {
        let inst: &Instance = inst_pair.1;
        let cellid: CellId = inst.child();
        ret.push(lib.cell(cellid));
    }
    ret
}

/// Returns a vector of all shapes from transformed child instances from a single cell instance.
fn flatten_instance<L>(inst: &Instance, lib: &Library<L>) -> Vec<Shape<L>>
where
    L: Connectivity + Clone,
{
    let mut ret: Vec<Shape<L>> = vec![];

    let cellid: CellId = inst.child();
    let transform: Transformation = inst.transformation();

    // Add all Shape Elements (filter out Text Elements)
    for elem in lib.cell(cellid).elements() {
        if let Element::Shape(shape) = elem {
            let transformed_shape = shape.transform_ref(transform);
            ret.push(transformed_shape);
        }
    }

    for instance in lib.cell(cellid).instances() {
        // Recursively flatten child instances
        let mut flattened_shapes = flatten_instance::<L>(instance.1, lib);
        // And apply transformations after all flattening
        for flattened_shape in &mut flattened_shapes {
            *flattened_shape = flattened_shape.transform_ref(transform);
        }
        ret.append(&mut flattened_shapes);
    }

    ret
}

/// Returns a recursively generated 1-d vector of sub-shapes in a given parent cell.
fn flatten_cell<T>(cell: &Cell<T>, lib: &Library<T>) -> Vec<Shape<T>>
where
    T: Connectivity,
{
    let mut ret: Vec<Shape<T>> = vec![];

    for elem in cell.elements() {
        if let Element::Shape(shape) = elem {
            ret.push(shape);
        }
    }

    for inst in cell.instances() {
        for thing in flatten_instance::<T>(inst.1, lib) {
            ret.push(thing);
        }
    }

    ret
}

pub trait Connectivity: Sized + PartialEq + Clone + Hash {
    fn connected_layers(&self) -> Vec<Self>;

    /// Returns a vector of layers connected to a given layer.
    fn connected(&self, other: &Self) -> bool {
        self.connected_layers().contains(other)
    }

    /// Returns a vector containing hashsets of shapes connected to each sub-shape in a given cell.
    fn connected_components<'a>(
        cell: &'a Cell<Self>,
        lib: &'a Library<Self>,
    ) -> Vec<HashSet<&'a Shape<Self>>> {
        // All sub-shapes contained in given cell
        let all_shapes = flatten_cell::<Self>(cell, lib);
        let mut djs = DisjointSet::new(all_shapes.len());

        // Build disjoint sets
        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (other_index, other_shape) in all_shapes.clone().into_iter().enumerate() {
                if start_index != other_index {
                    if intersect_shapes::<Self>(start_shape, other_shape)
                        && start_shape.layer().connected(other_shape.layer())
                    {
                        djs.union(start_index, other_index);
                    }
                }
            }
        }
        // Ret is a vector of hashsets of shapes connected to the shapes in the referenced cell
        //let mut ret: Vec<HashSet<&Shape<Self>>> = vec![HashSet![]; all_shapes.clone().len()];
        let mut ret: Vec<HashSet<&Shape<Self>>> =
            (0..all_shapes.len()).map(|_| HashSet::new()).collect();
        // Build hashsets of connected shapes to return
        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (other_index, other_shape) in all_shapes.clone().into_iter().enumerate() {
                if djs.is_united(start_index, other_index) {
                    ret[start_index].insert(other_shape);
                    ret[other_index].insert(start_shape);
                }
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::io::empty;

    use geometry::rect::Rect;

    use crate::Library;

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
    fn test_complete() {
        let mut small_cell: Cell<Layer> = Cell::new("small cell test");
        let mut big_cell: Cell<Layer> = Cell::new("big cell test");
        let mut lib: LibraryBuilder<Layer> = LibraryBuilder::new();

        let m2_shape1 = Shape::new(Layer::Met2, Rect::from_sides(0, 0, 30, 30));
        small_cell.add_element(m2_shape1.clone());

        lib.add_cell(small_cell.clone());
        let small_cell_id = lib.cells().next().unwrap().0;

        lib.add_cell(big_cell.clone());

        let small_cell_instance = Instance::new(small_cell_id, "small_cell_test");
        big_cell.add_instance(small_cell_instance);

        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let v1_shape = Shape::new(Layer::Via1, Rect::from_sides(10, 10, 100, 100));
        let m2_shape2 = Shape::new(Layer::Met2, Rect::from_sides(10, 10, 50, 50));
        let m2_shape3 = Shape::new(Layer::Met2, Rect::from_sides(1000, 1000, 5000, 5000));

        big_cell.add_element(m1_shape.clone());
        big_cell.add_element(v1_shape.clone());
        big_cell.add_element(m2_shape2.clone());
        big_cell.add_element(m2_shape3.clone());

        let binding = lib.clone().build().unwrap();
        let big_cell_cells = Layer::get_child_cells(&big_cell, &binding);

        let x = Layer::connected_components(&big_cell, &binding);

        assert_eq!(
            x.0,
            vec![&m1_shape, &v1_shape, &m2_shape2, &m2_shape3, &m2_shape1]
        );
        assert_eq!(x.1[0], vec![&m1_shape, &v1_shape, &m2_shape2, &m2_shape1]);
        assert_eq!(x.1[1], vec![&m1_shape, &v1_shape, &m2_shape2, &m2_shape1]);
        assert_eq!(x.1[2], vec![&m1_shape, &v1_shape, &m2_shape2, &m2_shape1]);
        assert_eq!(x.1[3], vec![&m2_shape3]);
        assert_eq!(x.1[4], vec![&m1_shape, &v1_shape, &m2_shape2, &m2_shape1]);
    }
}
