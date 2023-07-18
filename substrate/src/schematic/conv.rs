//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use opacity::Opacity;
use scir::{Cell, CellId as ScirCellId, CellInner, Instance, Library};
use uniquify::Names;

use crate::io::{Node, NodePath};

use super::{CellId, InstanceId, RawCell};

/// An SCIR library with associated conversion metadata.
#[derive(Debug, Clone)]
pub struct RawLib {
    /// The SCIR library.
    pub scir: scir::Library,
    /// Associated conversion metadata.
    ///
    /// Can be used to retrieve SCIR objects from their corresponding Substrate IDs.
    pub conv: ScirLibConversion,
}

/// Metadata associated with a conversion from a Substrate schematic to a SCIR library.
///
/// Provides helpers for retrieving SCIR objects from their Substrate IDs.
#[derive(Debug, Clone, Default)]
pub struct ScirLibConversion {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
}

impl ScirLibConversion {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Converts a Substrate [`NodePath`] to a SCIR [`scir::NodePath`].
    pub fn convert_path(&self, path: &NodePath) -> Option<scir::NodePath> {
        let mut cell = self.cells.get(&path.top)?;
        assert!(cell.top);

        let top = cell.id;

        let mut instances = Vec::new();
        for inst in &path.path {
            let (id, next_cell) = cell.instances.get(inst).unwrap();
            instances.push(*id);
            cell = self.cells.get(next_cell)?;
        }

        let (signal, index) = *cell.signals.get(&path.node)?;

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
    pub(crate) id: scir::CellId,
    /// Map Substrate nodes to SCIR signal IDs and indices.
    pub(crate) signals: HashMap<Node, (scir::SignalId, Option<usize>)>,
    /// Map Substrate instance IDs to SCIR instances and their underlying Substrate cell.
    pub(crate) instances: HashMap<InstanceId, (scir::InstanceId, CellId)>,
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

#[derive(Debug, Clone)]
struct ScirExportData {
    lib: Library,
    id_mapping: HashMap<CellId, ScirCellId>,
    conv: ScirLibConversion,
    cell_names: Names<CellId>,
}

impl ScirExportData {
    fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            lib: Library::new(name),
            id_mapping: HashMap::new(),
            conv: ScirLibConversion::new(),
            cell_names: Names::new(),
        }
    }
}

impl RawCell {
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    pub(crate) fn to_scir_lib(&self, testbench: ExportAsTestbench) -> RawLib {
        let mut data = ScirExportData::new(self.name.clone());
        let id = self.to_scir_cell(&mut data);
        data.lib.set_top(id, testbench.as_bool());
        data.conv.set_top(self.id);

        RawLib {
            scir: data.lib,
            conv: data.conv,
        }
    }

    fn to_scir_cell(&self, data: &mut ScirExportData) -> ScirCellId {
        // Create the SCIR cell as a whitebox for now.
        // If this Substrate cell is actually a blackbox,
        // the contents of this SCIR cell will be made into a blackbox
        // by calling `cell.set_contents`.
        let name = data.cell_names.assign_name(self.id, &self.name);
        let mut cell = Cell::new_whitebox(name);
        let mut signals = HashMap::new();
        let mut instances = HashMap::new();

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
            signals.insert(src, (s.signal(), None));
        }

        for port in self.ports.iter() {
            cell.expose_port(nodes[&port.node()]);
        }

        let contents = match self.contents.as_ref() {
            Opacity::Opaque(s) => Opacity::Opaque(s.clone()),
            Opacity::Clear(contents) => {
                let mut inner = CellInner::new();
                for (i, instance) in contents.instances.iter().enumerate() {
                    if !data.id_mapping.contains_key(&instance.child.id) {
                        instance.child.to_scir_cell(data);
                    }
                    let child: ScirCellId = *data.id_mapping.get(&instance.child.id).unwrap();

                    let mut sinst = Instance::new(arcstr::format!("xinst{i}"), child);
                    assert_eq!(instance.child.ports.len(), instance.connections.len());
                    for (port, &conn) in instance.child.ports.iter().zip(&instance.connections) {
                        let scir_port_name = instance.child.node_name(port.node());
                        sinst.connect(scir_port_name, nodes[&conn]);
                    }
                    let id = inner.add_instance(sinst);
                    instances.insert(instance.id, (id, instance.child.id));
                }

                for p in contents.primitives.iter() {
                    match p {
                        super::PrimitiveDevice::Res2 { pos, neg, value } => {
                            inner.add_primitive(scir::PrimitiveDevice::Res2 {
                                pos: nodes[pos],
                                neg: nodes[neg],
                                value: scir::Expr::NumericLiteral(*value),
                            });
                        }
                        super::PrimitiveDevice::RawInstance {
                            ports,
                            cell,
                            params,
                        } => {
                            inner.add_primitive(scir::PrimitiveDevice::RawInstance {
                                ports: ports.iter().map(|p| nodes[p]).collect(),
                                cell: cell.clone(),
                                params: params.clone(),
                            });
                        }
                        super::PrimitiveDevice::ScirInstance {
                            lib,
                            cell,
                            name,
                            connections,
                        } => {
                            let mapping = data.lib.merge(lib);
                            let cell = mapping.new_cell_id(*cell);
                            let mut inst = scir::Instance::new(name, cell);

                            for (port, elems) in connections {
                                let concat: scir::Concat = elems.iter().map(|n| nodes[n]).collect();
                                inst.connect(port, concat);
                            }
                            inner.add_instance(inst);
                        }
                    };
                }
                Opacity::Clear(inner)
            }
        };

        cell.set_contents(contents);

        let id = data.lib.add_cell(cell);
        data.id_mapping.insert(self.id, id);
        data.conv.add_cell(
            self.id,
            ScirCellConversion {
                top: false,
                id,
                signals,
                instances,
            },
        );

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
