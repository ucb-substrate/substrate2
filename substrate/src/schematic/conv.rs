//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use opacity::Opacity;
use scir::{Cell, CellId as ScirCellId, CellInner, Instance, Library};
use uniquify::Names;

use crate::io::{Node, NodePath};

use super::{BlackboxElement, CellId, InstanceId, RawCell, RawInstance};

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
#[derive(Debug, Clone)]
pub struct ScirLibConversion {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
    pub(crate) top: scir::CellId,
}

#[derive(Debug, Clone, Default)]
struct ScirLibConversionBuilder {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
    pub(crate) top: Option<scir::CellId>,
}

impl ScirLibConversionBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn build(self) -> ScirLibConversion {
        ScirLibConversion {
            cells: self.cells,
            top: self.top.unwrap(),
        }
    }

    #[inline]
    pub(crate) fn set_top(&mut self, id: CellId, scir_id: scir::CellId) {
        self.cells.get_mut(&id).unwrap().top = true;
        self.top = Some(scir_id);
    }

    #[inline]
    pub(crate) fn add_cell(&mut self, id: CellId, conv: ScirCellConversion) {
        self.cells.insert(id, conv);
    }
}

impl ScirLibConversion {
    /// Converts a Substrate [`NodePath`] to a SCIR [`scir::NodePath`].
    pub fn convert_path(&self, path: &NodePath) -> Option<scir::NodePath> {
        let mut cell = self.cells.get(&path.top)?;
        assert!(cell.top);

        let mut instances = Vec::new();
        for inst in &path.path {
            let conv = cell.instances.get(inst).unwrap();
            match conv.instance.as_ref() {
                Opacity::Opaque(id) => {
                    instances.push(*id);
                    cell = self.cells.get(&conv.child)?;
                }
                Opacity::Clear(conv) => {
                    cell = conv;
                }
            }
        }

        let (signal, index) = *cell.signals.get(&path.node)?;

        Some(scir::NodePath {
            signal,
            index,
            instances,
            top: self.top,
        })
    }
}

/// A single-node slice.
type SliceOne = (scir::SignalId, Option<usize>);

/// A converted SCIR instance.
type ConvertedScirInstance = Opacity<scir::InstanceId, ScirCellConversion>;

/// Data used to map between a Substrate cell and a SCIR cell.
///
/// Flattened cells do not have a conversion.
#[derive(Default, Debug, Clone)]
pub(crate) struct ScirCellConversion {
    /// Whether or not this cell is the top cell.
    pub(crate) top: bool,
    // /// SCIR cell name.
    // pub(crate) id: scir::CellId,
    /// Map Substrate nodes to SCIR signal IDs and indices.
    pub(crate) signals: HashMap<Node, SliceOne>,
    /// Map Substrate instance IDs to SCIR instances and their underlying Substrate cell.
    pub(crate) instances: HashMap<InstanceId, ScirInstanceConversion>,
}

impl ScirCellConversion {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScirInstanceConversion {
    /// The Substrate cell ID of the child cell.
    child: CellId,
    /// The SCIR instance.
    ///
    /// If the instance is not inlined/flattened, this will be an opaque instance ID.
    /// If the instance is inlined, this will be a [`ScirCellConversion`].
    instance: ConvertedScirInstance,
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
    conv: ScirLibConversionBuilder,
    cell_names: Names<CellId>,
}

impl ScirExportData {
    fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            lib: Library::new(name),
            id_mapping: HashMap::new(),
            conv: ScirLibConversionBuilder::new(),
            cell_names: Names::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
enum FlatExport<'a> {
    Yes(&'a RawInstance),
    #[default]
    No,
}

impl<'a> FlatExport<'a> {
    #[inline]
    pub fn is_yes(&self) -> bool {
        matches!(self, FlatExport::Yes(_))
    }

    #[inline]
    pub fn is_no(&self) -> bool {
        !self.is_yes()
    }
}

struct ScirExportContext {
    cell: scir::Cell,
}

impl ScirExportContext {
    #[inline]
    pub fn new(cell: scir::Cell) -> Self {
        Self { cell }
    }

    fn whitebox_contents_mut(&mut self) -> &mut CellInner {
        self.cell.contents_mut().as_mut().unwrap_clear()
    }
}

impl RawCell {
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    pub(crate) fn to_scir_lib(&self, testbench: ExportAsTestbench) -> RawLib {
        let mut data = ScirExportData::new(self.name.clone());
        let scir_id = self.to_scir_cell(&mut data);
        data.lib.set_top(scir_id, testbench.as_bool());
        data.conv.set_top(self.id, scir_id);

        RawLib {
            scir: data.lib,
            conv: data.conv.build(),
        }
    }

    fn to_scir_cell(&self, data: &mut ScirExportData) -> ScirCellId {
        // Create the SCIR cell as a whitebox for now.
        // If this Substrate cell is actually a blackbox,
        // the contents of this SCIR cell will be made into a blackbox
        // by calling `cell.set_contents`.
        let name = data.cell_names.assign_name(self.id, &self.name);
        let cell = Cell::new_whitebox(name);
        let mut ctx = ScirExportContext::new(cell);

        let conv = self.to_scir_cell_inner(data, &mut ctx, FlatExport::No);

        let ScirExportContext { cell } = ctx;
        let id = data.lib.add_cell(cell);
        data.conv.add_cell(self.id, conv);
        data.id_mapping.insert(self.id, id);

        id
    }

    fn to_scir_cell_inner(
        &self,
        data: &mut ScirExportData,
        ctx: &mut ScirExportContext,
        flatten: FlatExport,
    ) -> ScirCellConversion {
        // FIXME this function is wrong for nested flattened instances.

        if flatten.is_yes() {
            assert!(self.contents.is_clear());
        }

        let mut conv = ScirCellConversion::new();
        let mut nodes = HashMap::new();
        let mut roots_added = HashSet::new();

        if flatten.is_yes() {
            // Flattened cells need to add all non-IO nodes to the enclosing cell.
            for port in self.ports.iter() {
                let root = self.roots[&port.node()];
                roots_added.insert(root);
                // TODO: nodes.insert(root, slice in enclosing cell)
            }
        }

        for (&src, &root) in self.roots.iter() {
            let s = if !roots_added.contains(&root) {
                let s = ctx.cell.add_node(self.node_name(root));
                roots_added.insert(root);
                nodes.insert(root, s);
                s
            } else {
                nodes[&root]
            };
            nodes.insert(src, s);
            conv.signals.insert(src, (s.signal(), None));
        }

        match self.contents.as_ref() {
            Opacity::Opaque(contents) => {
                assert!(flatten.is_no(), "cannot flat-export a blackbox cell");
                let transformed = contents
                    .elems
                    .iter()
                    .map(|e| match e {
                        BlackboxElement::RawString(s) => {
                            scir::BlackboxElement::RawString(s.clone())
                        }
                        BlackboxElement::Node(n) => scir::BlackboxElement::Slice(nodes[n]),
                    })
                    .collect();
                ctx.cell.set_contents(Opacity::Opaque(transformed));
            }
            Opacity::Clear(contents) => {
                let contents_mut = ctx.cell.contents_mut().as_mut();
                let clear = contents_mut.is_clear();
                assert!(clear, "cannot flatten a cell into a blackbox parent cell");
                for (i, instance) in contents.instances.iter().enumerate() {
                    if instance.child.flatten {
                        instance
                            .child
                            .to_scir_cell_inner(data, ctx, FlatExport::Yes(instance));
                        // TODO populate instance metadata (conv.instances)
                    } else {
                        if !data.id_mapping.contains_key(&instance.child.id) {
                            instance.child.to_scir_cell(data);
                        }
                        let child: ScirCellId = *data.id_mapping.get(&instance.child.id).unwrap();

                        let mut sinst = Instance::new(arcstr::format!("xinst{i}"), child);
                        assert_eq!(instance.child.ports.len(), instance.connections.len());
                        for (port, &conn) in instance.child.ports.iter().zip(&instance.connections)
                        {
                            let scir_port_name = instance.child.node_name(port.node());
                            sinst.connect(scir_port_name, nodes[&conn]);
                        }
                        let id = ctx.whitebox_contents_mut().add_instance(sinst);
                        conv.instances.insert(
                            instance.id,
                            ScirInstanceConversion {
                                child: instance.child.id,
                                instance: Opacity::Opaque(id),
                            },
                        );
                    }
                }
                for p in contents.primitives.iter() {
                    match p {
                        super::PrimitiveDevice::Res2 { pos, neg, value } => {
                            ctx.whitebox_contents_mut().add_primitive(
                                scir::PrimitiveDevice::Res2 {
                                    pos: nodes[pos],
                                    neg: nodes[neg],
                                    value: scir::Expr::NumericLiteral(*value),
                                },
                            );
                        }
                        super::PrimitiveDevice::RawInstance {
                            ports,
                            cell,
                            params,
                        } => {
                            ctx.whitebox_contents_mut().add_primitive(
                                scir::PrimitiveDevice::RawInstance {
                                    ports: ports.iter().map(|p| nodes[p]).collect(),
                                    cell: cell.clone(),
                                    params: params.clone(),
                                },
                            );
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
                            ctx.whitebox_contents_mut().add_instance(inst);
                        }
                    };
                }
            }
        }

        if flatten.is_no() {
            for port in self.ports.iter() {
                ctx.cell.expose_port(nodes[&port.node()]);
            }
        }

        conv
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
