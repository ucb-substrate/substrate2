//! Substrate to SCIR conversion.

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use arcstr::ArcStr;
use scir::{Cell, CellId as ScirCellId, Instance, Library, Slice};

use crate::io::{Node, NodePath};

use super::{CellId, InstanceId, RawCell};

#[derive(Debug, Clone)]
pub struct RawLib {
    pub lib: scir::Library,
    pub conv: ScirLibConversion,
}

impl Deref for RawLib {
    type Target = scir::Library;
    fn deref(&self) -> &Self::Target {
        &self.lib
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScirLibConversion {
    /// Map from SCIR cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
}

impl ScirLibConversion {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn convert_path(&self, path: &NodePath) -> Option<scir::NodePath> {
        let mut cell = self.cells.get(&path.top)?;
        let top = cell.name.clone();
        assert!(cell.top);

        let mut instances = Vec::new();
        for inst in &path.path {
            let (name, next_cell) = cell.instances.get(inst).unwrap();
            instances.push(name.clone());
            cell = self.cells.get(next_cell)?;
        }

        let (signal, index) = cell.signals.get(&path.node)?.clone();

        Some(scir::NodePath {
            signal,
            index,
            instances,
            top,
        })
    }

    pub(crate) fn add_cell(&mut self, id: CellId, conv: ScirCellConversion) {
        self.cells.insert(id, conv);
    }

    pub(crate) fn set_top(&mut self, id: CellId) {
        self.cells.get_mut(&id).unwrap().top = true;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScirCellConversion {
    pub(crate) top: bool,
    /// SCIR cell name.
    pub(crate) name: ArcStr,
    /// Map Substrate nodes to SCIR signal names and indices.
    pub(crate) signals: HashMap<Node, (ArcStr, Option<usize>)>,
    /// Map Substrate instance IDs to SCIR instances and their underlying Substrate cell.
    pub(crate) instances: HashMap<InstanceId, (ArcStr, CellId)>,
}

impl ScirCellConversion {
    pub(crate) fn new(name: ArcStr) -> Self {
        Self {
            top: false,
            name,
            signals: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    pub(crate) fn add_signal(&mut self, node: Node, name: ArcStr, index: Option<usize>) {
        self.signals.insert(node, (name, index));
    }

    pub(crate) fn add_instance(&mut self, id: InstanceId, name: ArcStr, cell: CellId) {
        self.instances.insert(id, (name, cell));
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) enum ExportAsTestbench {
    No,
    Yes,
}

impl ExportAsTestbench {
    pub fn as_bool(&self) -> bool {
        match *self {
            Self::No => false,
            Self::Yes => true,
        }
    }
}

impl From<bool> for ExportAsTestbench {
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
}

impl RawCell {
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    pub(crate) fn to_scir_lib(&self, testbench: ExportAsTestbench) -> RawLib {
        let mut lib = Library::new(self.name.clone());
        let mut cells = HashMap::new();
        let mut conv = ScirLibConversion::new();
        let id = self.to_scir_cell(&mut lib, &mut cells, &mut conv);
        lib.set_top(id, testbench.as_bool());
        conv.set_top(self.id);

        RawLib { lib, conv }
    }

    fn to_scir_cell(
        &self,
        lib: &mut Library,
        cells: &mut HashMap<CellId, ScirCellId>,
        conv: &mut ScirLibConversion,
    ) -> ScirCellId {
        let mut cell = Cell::new(self.name.clone());
        let mut cell_conv = ScirCellConversion::new(self.name.clone());

        let mut nodes = HashMap::new();
        let mut roots_added = HashSet::new();

        for (&src, &root) in self.roots.iter() {
            let s = if !roots_added.contains(&root) {
                let s = cell.add_node(self.node_name(root));
                roots_added.insert(root);
                nodes.insert(root, s);
                s
            } else {
                nodes[&root]
            };
            nodes.insert(src, s);
            cell_conv.add_signal(src, cell.signal(s.signal()).name.clone(), None);
        }

        for (i, instance) in self.instances.iter().enumerate() {
            if !cells.contains_key(&instance.child.id) {
                instance.child.to_scir_cell(lib, cells, conv);
            }
            let child: ScirCellId = *cells.get(&instance.child.id).unwrap();

            let mut sinst = Instance::new(arcstr::format!("xinst{i}"), child);
            assert_eq!(instance.child.ports.len(), instance.connections.len());
            for (port, &conn) in instance.child.ports.iter().zip(&instance.connections) {
                let scir_port_name = instance.child.node_name(port.node());
                sinst.connect(scir_port_name, nodes[&conn]);
            }
            cell_conv.add_instance(instance.id, sinst.name().clone(), instance.child.id);
            cell.add_instance(sinst);
        }

        for p in self.primitives.iter() {
            let sp = match p {
                super::PrimitiveDevice::Res2 { pos, neg, value } => scir::PrimitiveDevice::Res2 {
                    pos: nodes[pos],
                    neg: nodes[neg],
                    value: scir::Expr::NumericLiteral(*value),
                },
                super::PrimitiveDevice::RawInstance {
                    ports,
                    cell,
                    params,
                } => scir::PrimitiveDevice::RawInstance {
                    ports: ports.iter().map(|p| nodes[p]).collect(),
                    cell: cell.clone(),
                    params: params.clone(),
                },
            };
            cell.add_primitive(sp);
        }

        for port in self.ports.iter() {
            cell.expose_port(nodes[&port.node()]);
        }

        let id = lib.add_cell(cell);
        cells.insert(self.id, id);
        conv.add_cell(self.id, cell_conv);

        id
    }

    /// The name associated with the given node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist within this cell.
    fn node_name(&self, node: Node) -> String {
        let node = self.roots[&node];
        self.node_names[&node].to_string()
    }
}
