//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use opacity::Opacity;
use scir::{Cell, CellId as ScirCellId, CellInner, Instance, Library};

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

impl RawCell {
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    pub(crate) fn to_scir_lib(&self, testbench: ExportAsTestbench) -> RawLib {
        let mut scir = Library::new(self.name.clone());
        let mut cells = HashMap::new();
        let mut conv = ScirLibConversion::new();
        let id = self.to_scir_cell(&mut scir, &mut cells, &mut conv);
        scir.set_top(id, testbench.as_bool());
        conv.set_top(self.id);

        RawLib { scir, conv }
    }

    fn to_scir_cell(
        &self,
        lib: &mut Library,
        cells: &mut HashMap<CellId, ScirCellId>,
        conv: &mut ScirLibConversion,
    ) -> ScirCellId {
        // Create the SCIR cell as a whitebox for now.
        // If this Substrate cell is actually a blackbox,
        // the contents of this SCIR cell will be made into a blackbox
        // by calling `cell.set_contents`.
        let mut cell = Cell::new_whitebox(self.name.clone());
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
                    let id = inner.add_instance(sinst);
                    instances.insert(instance.id, (id, instance.child.id));
                }

                for p in contents.primitives.iter() {
                    let sp = match p {
                        super::PrimitiveDevice::Res2 { pos, neg, value } => {
                            scir::PrimitiveDevice::Res2 {
                                pos: nodes[pos],
                                neg: nodes[neg],
                                value: scir::Expr::NumericLiteral(*value),
                            }
                        }
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
                    inner.add_primitive(sp);
                }
                Opacity::Clear(inner)
            }
        };

        cell.set_contents(contents);

        let id = lib.add_cell(cell);
        cells.insert(self.id, id);
        conv.add_cell(
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
