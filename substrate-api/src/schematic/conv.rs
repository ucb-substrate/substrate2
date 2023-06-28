//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use scir::slice::Slice;
use scir::{Cell, CellId as ScirCellId, Instance, Library};

use crate::io::{NameBuf, Node};

use super::{CellId, RawCell};

impl RawCell {
    /// Export this cell and all subcells as a SCIR library.
    pub fn to_scir_lib(&self) -> scir::Library {
        let mut lib = Library::new(self.name.clone());
        let mut cells = HashMap::new();
        self.to_scir_cell(&mut lib, &mut cells);
        lib
    }

    fn to_scir_cell(&self, lib: &mut Library, cells: &mut HashMap<CellId, ScirCellId>) {
        let mut cell = Cell::new(self.name.clone());

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
        }

        for (i, instance) in self.instances.iter().enumerate() {
            if !cells.contains_key(&instance.child.id) {
                instance.child.to_scir_cell(lib, cells);
            }
            let child: ScirCellId = *cells.get(&instance.child.id).unwrap();

            let mut sinst = Instance::new(arcstr::format!("xinst{i}"), child);
            assert_eq!(instance.child.ports.len(), instance.connections.len());
            for (port, &conn) in instance.child.ports.iter().zip(&instance.connections) {
                let scir_port_name = instance.child.node_name(port.node());
                sinst.connect(scir_port_name, nodes[&conn]);
            }
            cell.add_instance(sinst);
        }

        for p in self.primitives.iter() {
            let sp = match p {
                super::PrimitiveDevice::Res2 { pos, neg, value } => scir::PrimitiveDevice::Res2 {
                    pos: nodes[pos],
                    neg: nodes[neg],
                    value: scir::Expr::NumericLiteral(*value),
                },
            };
            cell.add_primitive(sp);
        }

        for port in self.ports.iter() {
            cell.expose_port(nodes[&port.node()]);
        }

        let id = lib.add_cell(cell);
        cells.insert(self.id, id);
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
