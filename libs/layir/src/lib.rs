#![allow(dead_code)]

pub mod id;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Deref,
};

use crate::id::Id;
use arcstr::ArcStr;
use geometry::prelude::Transformation;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use uniquify::Names;

pub struct Cells;

// The reason this uses [`Cells`] instead of [`Cell`]
// is because `Cell` has a generic type parameter.
pub type CellId = Id<Cells>;
pub type InstanceId = Id<Instance>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryBuilder<L> {
    cell_id: CellId,
    cells: IndexMap<CellId, Cell<L>>,
    name_map: HashMap<ArcStr, CellId>,
    names: Names<CellId>,
}

pub struct Library<L>(LibraryBuilder<L>);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cell<L> {
    name: ArcStr,
    instance_id: InstanceId,
    instances: IndexMap<InstanceId, Instance>,
    instance_name_map: HashMap<ArcStr, InstanceId>,
    elements: Vec<Element<L>>,
    ports: IndexMap<ArcStr, Port<L>>,
}

/// A location at which this cell should be connected.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Port<L> {
    direction: Direction,
    elements: Vec<Element<L>>,
}

/// Port directions.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Input.
    Input,
    /// Output.
    Output,
    /// Input or output.
    ///
    /// Represents ports whose direction is not known
    /// at generator elaboration time (e.g. the output of a tristate buffer).
    #[default]
    InOut,
}

/// A primitive layout element.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Element<L> {
    /// A primitive layout shape.
    Shape(Shape<L>),
    /// A primitive text annotation.
    Text(Text<L>),
}

/// A primitive layout shape consisting of a layer and a geometric shape.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Shape<L> {
    layer: L,
    shape: geometry::shape::Shape,
}

/// A primitive text annotation consisting of a layer, string, and location.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Text<L> {
    layer: L,
    text: ArcStr,
    trans: Transformation,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    child: CellId,
    name: ArcStr,
    trans: Transformation,
}

impl<L> Default for LibraryBuilder<L> {
    fn default() -> Self {
        Self {
            cell_id: Id::new(),
            names: Default::default(),
            name_map: Default::default(),
            cells: Default::default(),
        }
    }
}

impl<L> LibraryBuilder<L> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_cell(&mut self, mut cell: Cell<L>) -> CellId {
        let id = self.cell_id.alloc();
        cell.name = self.names.assign_name(id, &cell.name);
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
        id
    }

    pub fn cell(&self, id: CellId) -> &Cell<L> {
        self.cells.get(&id).unwrap()
    }

    pub fn try_cell(&self, id: CellId) -> Option<&Cell<L>> {
        self.cells.get(&id)
    }

    pub fn cell_named(&self, name: &str) -> &Cell<L> {
        self.cell(*self.name_map.get(name).unwrap())
    }

    pub fn try_cell_named(&self, name: &str) -> Option<&Cell<L>> {
        self.try_cell(*self.name_map.get(name)?)
    }

    /// Gets the cell ID corresponding to the given name.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given name.
    /// For a non-panicking alternative, see [`try_cell_id_named`](LibraryBuilder::try_cell_id_named).
    pub fn cell_id_named(&self, name: &str) -> CellId {
        match self.name_map.get(name) {
            Some(&cell) => cell,
            None => {
                tracing::error!("no cell named `{}`", name);
                panic!("no cell named `{}`", name);
            }
        }
    }

    /// Gets the cell ID corresponding to the given name.
    pub fn try_cell_id_named(&self, name: &str) -> Option<CellId> {
        self.name_map.get(name).copied()
    }

    /// Iterates over the `(id, cell)` pairs in this library.
    pub fn cells(&self) -> impl Iterator<Item = (CellId, &Cell<L>)> {
        self.cells.iter().map(|(id, cell)| (*id, cell))
    }

    /// Returns cell IDs in topological order.
    pub fn topological_order(&self) -> Vec<CellId> {
        let mut state = IndexSet::new();
        for (cell, _) in self.cells() {
            self.dfs_postorder(cell, &mut state);
        }
        let ids = state.into_iter().collect::<Vec<_>>();
        assert_eq!(ids.len(), self.cells.len());
        ids
    }

    fn dfs_postorder(&self, id: CellId, state: &mut IndexSet<CellId>) {
        if state.contains(&id) {
            return;
        }

        let cell = self.cell(id);
        for (_, inst) in cell.instances() {
            self.dfs_postorder(inst.child(), state);
        }
        state.insert(id);
    }

    /// The list of cell IDs instantiated by the given root cells.
    ///
    /// The list returned will include the root cell IDs.
    pub(crate) fn cells_used_by(&self, roots: impl IntoIterator<Item = CellId>) -> Vec<CellId> {
        let mut stack = VecDeque::new();
        let mut visited = HashSet::new();
        for root in roots {
            stack.push_back(root);
        }

        while let Some(id) = stack.pop_front() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            let cell = self.cell(id);
            for (_, inst) in cell.instances() {
                stack.push_back(inst.child);
            }
        }

        visited.drain().collect()
    }

    pub fn build(self) -> Result<Library<L>, BuildError> {
        Ok(Library(self))
    }
}

#[derive(Clone, Debug)]
pub struct BuildError;

impl<L> Deref for Library<L> {
    type Target = LibraryBuilder<L>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L> Cell<L> {
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            name: name.into(),
            instance_id: Id::new(),
            instances: Default::default(),
            instance_name_map: Default::default(),
            elements: Default::default(),
            ports: Default::default(),
        }
    }

    /// The name of the cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Iterate over the ports of this cell.
    #[inline]
    pub fn ports(&self) -> impl Iterator<Item = (&ArcStr, &Port<L>)> {
        self.ports.iter()
    }

    pub fn add_port(&mut self, name: impl Into<ArcStr>, port: Port<L>) {
        self.ports.insert(name.into(), port);
    }

    /// Get a port of this cell by name.
    ///
    /// # Panics
    ///
    /// Panics if the provided port does not exist.
    #[inline]
    pub fn port(&self, name: &str) -> &Port<L> {
        self.try_port(name).unwrap()
    }

    /// Get a port of this cell by name.
    #[inline]
    pub fn try_port(&self, name: &str) -> Option<&Port<L>> {
        self.ports.get(name)
    }

    /// Get the instance associated with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no instance with the given ID exists.
    #[inline]
    pub fn instance(&self, id: InstanceId) -> &Instance {
        self.instances.get(&id).unwrap()
    }

    /// Get the instance associated with the given ID.
    #[inline]
    pub fn try_instance(&self, id: InstanceId) -> Option<&Instance> {
        self.instances.get(&id)
    }

    /// Gets the instance with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no instance has the given name.
    pub fn instance_named(&self, name: &str) -> &Instance {
        self.instance(*self.instance_name_map.get(name).unwrap())
    }

    /// Gets the instance with the given name.
    pub fn try_instance_named(&self, name: &str) -> Option<&Instance> {
        self.try_instance(*self.instance_name_map.get(name)?)
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) -> InstanceId {
        let id = self.instance_id.alloc();
        self.instance_name_map.insert(instance.name.clone(), id);
        self.instances.insert(id, instance);
        id
    }

    /// Iterate over the instances of this cell.
    #[inline]
    pub fn instances(&self) -> impl Iterator<Item = (InstanceId, &Instance)> {
        self.instances.iter().map(|x| (*x.0, x.1))
    }

    pub fn add_element(&mut self, element: impl Into<Element<L>>) {
        self.elements.push(element.into())
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element<L>> {
        self.elements.iter()
    }
}

impl<L> Port<L> {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            elements: Default::default(),
        }
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element<L>> {
        self.elements.iter()
    }

    pub fn add_element(&mut self, element: impl Into<Element<L>>) {
        self.elements.push(element.into())
    }
}

impl<L> From<Shape<L>> for Element<L> {
    fn from(value: Shape<L>) -> Self {
        Self::Shape(value)
    }
}

impl<L> From<Text<L>> for Element<L> {
    fn from(value: Text<L>) -> Self {
        Self::Text(value)
    }
}

impl<L> Shape<L> {
    #[inline]
    pub fn new(layer: L, shape: geometry::shape::Shape) -> Self {
        Self { layer, shape }
    }

    #[inline]
    pub fn layer(&self) -> &L {
        &self.layer
    }

    #[inline]
    pub fn shape(&self) -> &geometry::shape::Shape {
        &self.shape
    }
}

impl<L> Text<L> {
    #[inline]
    pub fn new(layer: L, text: impl Into<ArcStr>) -> Self {
        Self {
            layer,
            text: text.into(),
            trans: Default::default(),
        }
    }

    #[inline]
    pub fn with_transformation(
        layer: L,
        text: impl Into<ArcStr>,
        trans: impl Into<Transformation>,
    ) -> Self {
        Self {
            layer,
            text: text.into(),
            trans: trans.into(),
        }
    }

    #[inline]
    pub fn layer(&self) -> &L {
        &self.layer
    }

    #[inline]
    pub fn text(&self) -> &ArcStr {
        &self.text
    }

    #[inline]
    pub fn transformation(&self) -> Transformation {
        self.trans
    }
}

impl Instance {
    pub fn new(child: CellId, name: impl Into<ArcStr>) -> Self {
        Self {
            child,
            name: name.into(),
            trans: Default::default(),
        }
    }

    pub fn with_transformation(
        child: CellId,
        name: impl Into<ArcStr>,
        transformation: impl Into<Transformation>,
    ) -> Self {
        Self {
            child,
            name: name.into(),
            trans: transformation.into(),
        }
    }

    #[inline]
    pub fn child(&self) -> CellId {
        self.child
    }

    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    #[inline]
    pub fn transformation(&self) -> Transformation {
        self.trans
    }
}
