use crate::CellId;
use crate::Instance;
use aph_disjoint_set::DisjointSet;
use geometry::bbox::Bbox;
use geometry::rect::{self, Rect};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::Element;
use crate::{Cell, Library, LibraryBuilder, Shape, TransformMut, TransformRef, Transformation};

/// Returns true if two shapes overlap on a flat plane, and false otherwise.
fn intersect_shapes<L>(shape1: &Shape<L>, shape2: &Shape<L>) -> bool {
    let shape1_bbox: Rect = shape1.bbox_rect();
    let shape2_bbox: Rect = shape2.bbox_rect();
    shape1_bbox.intersection(shape2_bbox) != None
}

/// Returns a vector of all shapes from a given cell instance and its children
/// with their coordinates transformed into the coordinate system of the instance's parent.
fn flatten_instance<L>(inst: &Instance, lib: &Library<L>) -> Vec<Shape<L>>
where
    L: Connectivity + Clone,
{
    let mut ret: Vec<Shape<L>> = vec![];

    let cellid: CellId = inst.child();
    let transform: Transformation = inst.transformation();

    // Add all shape elements directly from child cell after applying the instances transformation
    for elem in lib.cell(cellid).elements() {
        if let Element::Shape(shape) = elem {
            let transformed_shape = shape.transform_ref(transform);
            ret.push(transformed_shape);
        }
    }

    // Recursively flatten child instances and apply cumulative transformations
    for instance in lib.cell(cellid).instances() {
        let mut flattened_shapes = flatten_instance::<L>(instance.1, lib);
        for flattened_shape in &mut flattened_shapes {
            *flattened_shape = flattened_shape.transform_ref(transform);
        }
        ret.append(&mut flattened_shapes);
    }

    ret
}

/// Returns a vector of all shapes from a single cell's transformed child instances and itself.
fn flatten_cell<L>(cell: &Cell<L>, lib: &Library<L>) -> Vec<Shape<L>>
where
    L: Connectivity + Clone,
{
    let mut ret: Vec<Shape<L>> = vec![];

    // No transformation needed
    for elem in cell.elements() {
        if let Element::Shape(shape) = elem {
            ret.push(shape.clone());
        }
    }

    for inst in cell.instances() {
        for flattened_shape in flatten_instance::<L>(inst.1, lib) {
            ret.push(flattened_shape);
        }
    }

    ret
}

pub trait Connectivity: Sized + PartialEq + Eq + Clone + Hash {
    fn connected_layers(&self) -> Vec<Self>;

    /// Returns a vector of layers connected to a given layer.
    fn connected(&self, other: &Self) -> bool {
        self.connected_layers().contains(other)
    }

    /// Returns a vector of unique hashsets of all connected groups of connected child shapes in a given cell.
    fn connected_components(cell: &Cell<Self>, lib: &Library<Self>) -> Vec<HashSet<Shape<Self>>>
    where
        Self: Clone,
    {
        // All sub-shapes contained in given cell
        let all_shapes = flatten_cell::<Self>(cell, lib);
        let mut djs = DisjointSet::new(all_shapes.len());

        // Build disjoint sets based on overlap and layer connectivity
        for (start_index, start_shape) in all_shapes.clone().into_iter().enumerate() {
            for (other_index, other_shape) in all_shapes.clone().into_iter().enumerate() {
                if start_index != other_index {
                    if intersect_shapes::<Self>(&start_shape, &other_shape)
                        && start_shape.layer().connected(other_shape.layer())
                    {
                        djs.union(start_index, other_index);
                    }
                }
            }
        }

        let mut component_map: HashMap<usize, HashSet<Shape<Self>>> = HashMap::new();

        for (start_index, start_shape) in all_shapes.into_iter().enumerate() {
            let root: usize = djs.get_root(start_index).into_inner();
            component_map
                .entry(root)
                .or_insert_with(HashSet::new)
                .insert(start_shape);
        }

        component_map.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Instance, Library, LibraryBuilder, Shape};
    use geometry::rect::Rect;
    use std::collections::{HashMap, HashSet};

    // This struct helps check if two shapes are connected after connected_components has been run
    struct ComponentLookup<L>
    where
        L: Connectivity + Clone,
    {
        shape_to_component_id: HashMap<Shape<L>, usize>,
    }

    impl<L> ComponentLookup<L>
    where
        L: Connectivity + Clone,
    {
        /// Creates a new ComponentLookup from a vector of connected components.
        fn new(components: Vec<HashSet<Shape<L>>>) -> Self {
            let mut shape_to_component_id = HashMap::new();
            for (component_id, component_set) in components.into_iter().enumerate() {
                for shape in component_set.into_iter() {
                    shape_to_component_id.insert(shape, component_id);
                }
            }
            ComponentLookup {
                shape_to_component_id,
            }
        }

        /// Returns true if both shapes are found and belong to the same component, and false otherwise.
        fn are_connected(&self, s1: &Shape<L>, s2: &Shape<L>) -> bool {
            let comp_id1 = self.shape_to_component_id.get(s1);
            let comp_id2 = self.shape_to_component_id.get(s2);

            match (comp_id1, comp_id2) {
                // If both shapes are found, check if their component IDs are the same
                (Some(&id1), Some(&id2)) => id1 == id2,
                _ => false,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum Layer {
        Met1,
        Via1,
        Met2,
        Outside,
    }

    impl Connectivity for Layer {
        fn connected_layers(&self) -> Vec<Self> {
            match self {
                Self::Met1 => vec![Self::Met1, Self::Via1],
                Self::Via1 => vec![Self::Met1, Self::Via1, Self::Met2],
                Self::Met2 => vec![Self::Via1, Self::Met2],
                Self::Outside => vec![],
            }
        }
    }

    #[test]
    fn test_complete() {
        let mut small_cell: Cell<Layer> = Cell::new("small cell test");
        let mut big_cell: Cell<Layer> = Cell::new("big cell test");
        let mut lib_builder: LibraryBuilder<Layer> = LibraryBuilder::new();

        // Build small cell first and add to big cell
        let m2_shape1 = Shape::new(Layer::Met2, Rect::from_sides(0, 0, 30, 30));
        small_cell.add_element(m2_shape1.clone());

        lib_builder.add_cell(small_cell.clone());
        let small_cell_id = lib_builder.cells().next().unwrap().0;

        let small_cell_instance = Instance::new(small_cell_id, "small_cell_test");
        big_cell.add_instance(small_cell_instance);

        // Build big cell
        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        let v1_shape = Shape::new(Layer::Via1, Rect::from_sides(10, 10, 100, 100));
        let m2_shape2 = Shape::new(Layer::Met2, Rect::from_sides(10, 10, 50, 50));
        let m2_shape3 = Shape::new(Layer::Met2, Rect::from_sides(1000, 1000, 5000, 5000));
        big_cell.add_element(m1_shape.clone());
        big_cell.add_element(v1_shape.clone());
        big_cell.add_element(m2_shape2.clone());
        big_cell.add_element(m2_shape3.clone());

        lib_builder.add_cell(big_cell.clone());

        // Build fixed library
        let lib = lib_builder.clone().build().unwrap();

        // Add an outside shape for testing
        let outside_shape = Shape::new(Layer::Outside, Rect::from_sides(0, 0, 100, 100));

        // Find all connected components of big_cell's child shapes
        let components_vec = Layer::connected_components(&big_cell, &lib);

        let component_lookup = ComponentLookup::new(components_vec.clone());

        // Expected connections
        assert!(
            component_lookup.are_connected(&m1_shape, &v1_shape),
            "m1_shape should be connected to v1_shape"
        );
        assert!(
            component_lookup.are_connected(&m1_shape, &m2_shape2),
            "m1_shape should be connected to m2_shape2"
        );
        assert!(
            component_lookup.are_connected(&m1_shape, &m2_shape1), // m2_shape1 is from the instance
            "m1_shape should be connected to m2_shape1"
        );

        // Expected disconnection
        assert!(
            !component_lookup.are_connected(&m1_shape, &m2_shape3),
            "m1_shape should not be connected to m2_shape3 (isolated)"
        );
        assert!(
            !component_lookup.are_connected(&m1_shape, &outside_shape),
            "m1_shape should not be connected to outside_shape"
        );

        // Double check number of connected components in library
        assert_eq!(
            components_vec.len(),
            2,
            "Expected 2 total connected components."
        );
    }
}
