//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

use arcstr::ArcStr;
use scir::{Cell, ChildId, Concat, IndexOwned, Instance, LibraryBuilder};
use serde::{Deserialize, Serialize};
use substrate::schematic::{ConvertedPrimitive, ScirBinding};
use uniquify::Names;

use crate::io::schematic::{Node, NodePath, TerminalPath};
use crate::schematic::schema::Schema;
use crate::schematic::{ConvertPrimitive, InstancePath, RawCellContents, RawCellKind};

use super::{CellId, InstanceId, RawCell};

/// An SCIR library with associated conversion metadata.
pub struct RawLib<S: Schema + ?Sized> {
    /// The SCIR library.
    pub scir: scir::Library<S>,
    /// Associated conversion metadata.
    ///
    /// Can be used to retrieve SCIR objects from their corresponding Substrate IDs.
    pub conv: ScirLibConversion,
}

impl<S: Schema<Primitive = impl Clone>> Clone for RawLib<S> {
    fn clone(&self) -> Self {
        Self {
            scir: self.scir.clone(),
            conv: self.conv.clone(),
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug>> std::fmt::Debug for RawLib<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawLib");
        let _ = builder.field("scir", &self.scir);
        let _ = builder.field("conv", &self.conv);
        builder.finish()
    }
}

/// Metadata associated with a conversion from a Substrate schematic to a SCIR library.
///
/// Provides helpers for retrieving SCIR objects from their Substrate IDs.
#[derive(Debug, Clone)]
pub struct ScirLibConversion {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, SubstrateCellConversion>,
    /// The Substrate ID of the top cell, if there is one.
    top: Option<CellId>,
}

impl ScirLibConversion {
    /// Get the SCIR cell ID corresponding to the given [`RawCell`] if a corresponding cell exists.
    ///
    /// May return none if the given raw cell corresponds to a primitive or to a flattened cell.
    pub fn corresponding_cell<S: Schema + ?Sized>(
        &self,
        cell: &RawCell<S>,
    ) -> Option<scir::CellId> {
        let conv = self.cells.get(&cell.id)?;
        match conv {
            SubstrateCellConversion::Cell(c) => c.cell_id,
            SubstrateCellConversion::Primitive(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ScirLibConversionBuilder {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, SubstrateCellConversion>,
    /// The Substrate ID of the top cell, if there is one.
    top: Option<CellId>,
}

/// A path within a SCIR library corresponding to a Substrate [`NodePath`].
#[derive(Debug, Clone)]
pub enum ConvertedNodePath {
    /// A path that corresponds to a node within a SCIR cell.
    Cell(scir::SliceOnePath),
    /// A path that corresponds to a port of a primitive.
    Primitive {
        /// The ID of the underlying primitive.
        id: scir::PrimitiveId,
        /// The instance path of the primitive instance.
        instances: scir::InstancePath,
        /// The port of the primitive instance.
        port: ArcStr,
        /// The index of the primitive port corresponding to the Substrate node.
        index: usize,
    },
}

impl ScirLibConversionBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn build(self) -> ScirLibConversion {
        ScirLibConversion {
            cells: self.cells,
            top: self.top,
        }
    }

    #[inline]
    pub(crate) fn add_cell(&mut self, id: CellId, conv: impl Into<SubstrateCellConversion>) {
        self.cells.insert(id, conv.into());
    }
}

impl<S: Schema> RawLib<S> {
    fn convert_instance_path_inner<'a>(
        &'a self,
        top: CellId,
        instances: impl IntoIterator<Item = &'a InstanceId>,
    ) -> Option<(
        scir::InstancePath,
        SubstrateCellConversionRef<&'a ScirCellConversion, &'a ScirPrimitiveConversion>,
    )> {
        let mut cell = self.conv.cells.get(&top)?.as_ref();
        let mut scir_id: scir::ChildId = self.scir.top_cell()?.into();

        let mut scir_instances = scir::InstancePath::new(self.scir.top_cell()?);
        for inst in instances {
            let conv = cell.get_cell()?.instances.get(inst).unwrap();
            match conv.instance.as_ref() {
                ConvertedScirInstanceContentRef::Cell(id) => {
                    scir_id = self.scir.cell(scir_id.into_cell()?).instance(*id).child();
                    scir_instances.push(*id);
                    if let Some(conv) = self.conv.cells.get(&conv.child) {
                        cell = conv.as_ref();
                    }
                }
                ConvertedScirInstanceContentRef::InlineCell(conv) => {
                    cell = SubstrateCellConversionRef::Cell(conv);
                }
            }
        }
        Some((scir_instances, cell))
    }

    /// Converts a Substrate [`NodePath`] to a SCIR [`scir::SliceOnePath`].
    pub fn convert_node_path(&self, path: &NodePath) -> Option<ConvertedNodePath> {
        let (instances, cell) = self.convert_instance_path_inner(path.top, &path.instances)?;

        Some(match cell {
            SubstrateCellConversionRef::Cell(cell) => ConvertedNodePath::Cell(
                scir::SliceOnePath::new(instances, *cell.signals.get(&path.node)?),
            ),
            SubstrateCellConversionRef::Primitive(p) => {
                let (port, index) = p.ports.get(&path.node)?.first()?;
                ConvertedNodePath::Primitive {
                    id: p.primitive_id,
                    instances,
                    port: port.clone(),
                    index: *index,
                }
            }
        })
    }

    /// Convert a node in the top cell to a SCIR node path.
    pub fn convert_node(&self, node: &Node) -> Option<ConvertedNodePath> {
        let top = self.conv.top?;
        let empty = Vec::new();
        let (instances, cell) = self.convert_instance_path_inner(top, &empty)?;

        Some(match cell {
            SubstrateCellConversionRef::Cell(cell) => ConvertedNodePath::Cell(
                scir::SliceOnePath::new(instances, *cell.signals.get(node)?),
            ),
            SubstrateCellConversionRef::Primitive(p) => {
                let (port, index) = p.ports.get(node)?.first()?;
                ConvertedNodePath::Primitive {
                    id: p.primitive_id,
                    instances,
                    port: port.clone(),
                    index: *index,
                }
            }
        })
    }

    /// Converts a Substrate [`InstancePath`] to a SCIR [`scir::InstancePath`].
    pub fn convert_instance_path(&self, path: &InstancePath) -> Option<scir::InstancePath> {
        let (instances, _) = self.convert_instance_path_inner(path.top, &path.path)?;
        Some(instances)
    }

    /// Converts a Substrate [`TerminalPath`] to a list of [`ConvertedNodePath`]s
    /// associated with the terminal at that path.
    ///
    /// Returns [`None`] if the path is invalid. Only flattened instances will
    /// return more than one [`ConvertedNodePath`].
    pub fn convert_terminal_path(&self, path: &TerminalPath) -> Option<Vec<ConvertedNodePath>> {
        let mut cell = self.conv.cells.get(&path.top)?.as_ref();

        let scir_id = self.scir.top_cell()?;
        let mut instances = scir::InstancePath::new(scir_id);
        let mut scir_id = ChildId::Cell(scir_id);
        let mut last_clear = false;
        for inst in &path.instances {
            let conv = cell.into_cell()?.instances.get(inst).unwrap();
            match conv.instance.as_ref() {
                ConvertedScirInstanceContentRef::Cell(id) => {
                    scir_id = self.scir.cell(scir_id.into_cell()?).instance(*id).child();
                    instances.push(*id);
                    cell = self.conv.cells.get(&conv.child)?.as_ref();
                    last_clear = false;
                }
                ConvertedScirInstanceContentRef::InlineCell(conv) => {
                    cell = SubstrateCellConversionRef::Cell(conv);
                    last_clear = true;
                }
            }
        }

        match cell {
            SubstrateCellConversionRef::Cell(cell) => {
                // If the last cell in the conversion was `Opacity::Clear`, the provided terminal is
                // virtual and thus may correspond to more than one `scir::SignalPath`.
                //
                // Run DFS to find all signal paths that are directly connected to this virtual
                // terminal.
                let slice = *cell.signals.get(&path.node)?;
                Some(if last_clear {
                    let mut signals = Vec::new();
                    self.find_connected_terminals(
                        cell,
                        self.scir.cell(scir_id.into_cell()?),
                        slice,
                        &mut instances,
                        &mut signals,
                    );
                    signals
                } else {
                    vec![ConvertedNodePath::Cell(scir::SliceOnePath::new(
                        instances, slice,
                    ))]
                })
            }
            SubstrateCellConversionRef::Primitive(p) => {
                let mut out = Vec::new();
                for (port, index) in p.ports.get(&path.node)? {
                    out.push(ConvertedNodePath::Primitive {
                        id: p.primitive_id,
                        instances: instances.clone(),
                        port: port.clone(),
                        index: *index,
                    });
                }
                Some(out)
            }
        }
    }

    /// Must ensure that `instances` is returned to its original value by the end of the
    /// function call.
    fn find_connected_terminals_in_scir_instance(
        &self,
        parent_cell: &scir::Cell,
        id: scir::InstanceId,
        slice: scir::SliceOne,
        instances: &mut scir::InstancePath,
        signals: &mut Vec<ConvertedNodePath>,
    ) -> Option<()> {
        instances.push(id);
        let inst = parent_cell.instance(id);
        for (name, conn) in inst.connections() {
            let mut port_index = 0;
            for part in conn.parts() {
                if slice.signal() == part.signal() {
                    let concat_index = match (slice.index(), part.range()) {
                        (None, None) => Some(port_index),
                        (Some(index), Some(range)) => {
                            if range.contains(index) {
                                Some(port_index + index - range.start())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(concat_index) = concat_index {
                        match inst.child() {
                            ChildId::Cell(id) => {
                                let child_cell = self.scir.cell(id);
                                let port = child_cell.port(name);
                                let port_slice = child_cell.signal(port.signal()).slice();
                                let tail = port_slice
                                    .slice_one()
                                    .unwrap_or_else(|| port_slice.index(concat_index));
                                signals.push(ConvertedNodePath::Cell(scir::SliceOnePath::new(
                                    instances.clone(),
                                    tail,
                                )));
                            }
                            ChildId::Primitive(id) => {
                                signals.push(ConvertedNodePath::Primitive {
                                    id,
                                    instances: instances.clone(),
                                    port: name.clone(),
                                    index: concat_index,
                                });
                            }
                        }
                    }
                }
                port_index += part.width();
            }
        }
        instances.pop().unwrap();
        Some(())
    }

    /// Must ensure that `instances` is returned to its original value by the end of the
    /// function call.
    fn find_connected_terminals(
        &self,
        conv: &ScirCellConversion,
        parent_cell: &scir::Cell,
        slice: scir::SliceOne,
        instances: &mut scir::InstancePath,
        signals: &mut Vec<ConvertedNodePath>,
    ) -> Option<()> {
        for (_, conv) in conv.instances.iter() {
            match conv.instance.as_ref() {
                ConvertedScirInstanceContentRef::Cell(id) => {
                    self.find_connected_terminals_in_scir_instance(
                        parent_cell,
                        *id,
                        slice,
                        instances,
                        signals,
                    );
                }
                ConvertedScirInstanceContentRef::InlineCell(conv) => {
                    self.find_connected_terminals(conv, parent_cell, slice, instances, signals);
                }
            }
        }
        Some(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[enumify::enumify]
enum ConvertedScirInstanceContent<U, F> {
    Cell(U),
    InlineCell(F),
}

/// A converted SCIR instance.
type ConvertedScirInstance = ConvertedScirInstanceContent<scir::InstanceId, ScirCellConversion>;

#[enumify::enumify]
#[derive(Debug, Clone)]
pub(crate) enum SubstrateCellConversion {
    Cell(ScirCellConversion),
    Primitive(ScirPrimitiveConversion),
}

impl From<ScirCellConversion> for SubstrateCellConversion {
    fn from(value: ScirCellConversion) -> Self {
        Self::Cell(value)
    }
}

impl From<ScirPrimitiveConversion> for SubstrateCellConversion {
    fn from(value: ScirPrimitiveConversion) -> Self {
        Self::Primitive(value)
    }
}

/// Data used to map between a Substrate cell and a SCIR cell.
///
/// Flattened cells do not have a conversion.
#[derive(Debug, Default, Clone)]
pub(crate) struct ScirCellConversion {
    /// The corresponding SCIR cell ID. [`None`] for flattened Substrate cells.
    pub(crate) cell_id: Option<scir::CellId>,
    /// Map Substrate nodes to SCIR signal IDs and indices.
    pub(crate) signals: HashMap<Node, scir::SliceOne>,
    /// Map Substrate instance IDs to SCIR instances and their underlying Substrate cell.
    pub(crate) instances: HashMap<InstanceId, ScirInstanceConversion>,
}

/// Data used to map between a Substrate cell and a SCIR cell.
///
/// Flattened cells do not have a conversion.
#[derive(Debug, Clone)]
pub(crate) struct ScirPrimitiveConversion {
    pub(crate) primitive_id: scir::PrimitiveId,
    /// Map Substrate nodes to a SCIR primitive port and an index within that port.
    pub(crate) ports: HashMap<Node, Vec<(ArcStr, usize)>>,
}

impl ScirCellConversion {
    #[inline]
    fn new() -> Self {
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

pub(crate) struct ScirLibExportContext<S: Schema + ?Sized> {
    lib: LibraryBuilder<S>,
    conv: ScirLibConversionBuilder,
    cell_names: Names<CellId>,
}

impl<S: Schema + ?Sized> Default for ScirLibExportContext<S> {
    fn default() -> Self {
        Self {
            lib: LibraryBuilder::new(),
            conv: ScirLibConversionBuilder::new(),
            cell_names: Names::new(),
        }
    }
}

impl<S: Schema<Primitive = impl Clone> + ?Sized> Clone for ScirLibExportContext<S> {
    fn clone(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            conv: self.conv.clone(),
            cell_names: self.cell_names.clone(),
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug
    for ScirLibExportContext<S>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ScirLibExportContext");
        let _ = builder.field("lib", &self.lib);
        let _ = builder.field("conv", &self.conv);
        let _ = builder.field("cell_names", &self.cell_names);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> ScirLibExportContext<S> {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone)]
enum FlatExport {
    Yes(Vec<scir::SliceOne>),
    #[default]
    No,
}

impl FlatExport {
    #[inline]
    pub fn is_yes(&self) -> bool {
        matches!(self, FlatExport::Yes(_))
    }

    #[inline]
    pub fn is_no(&self) -> bool {
        !self.is_yes()
    }
}

struct ScirCellExportContext {
    inst_idx: u64,
    cell: scir::Cell,
}

impl ScirCellExportContext {
    #[inline]
    pub fn new(cell: scir::Cell) -> Self {
        Self { inst_idx: 0, cell }
    }
}

impl<S: Schema + ?Sized> RawCell<S> {
    /// The name associated with the given node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist within this cell.
    fn node_name(&self, node: Node) -> String {
        let node = self.roots[&node];
        self.node_names[&node].to_string()
    }
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    ///
    /// Consider using [`export_multi_top_scir_lib`] if you need to export multiple cells
    /// to the same SCIR library.
    pub(crate) fn to_scir_lib(&self) -> Result<RawLib<S>, ConvError> {
        let mut lib_ctx = ScirLibExportContext::new();
        let scir_id = self.to_scir_cell(&mut lib_ctx)?;

        if let ChildId::Cell(scir_id) = scir_id {
            lib_ctx.lib.set_top(scir_id);
        }
        lib_ctx.conv.top = Some(self.id);

        Ok(RawLib {
            scir: lib_ctx.lib.build()?,
            conv: lib_ctx.conv.build(),
        })
    }

    /// Exports this [`RawCell`] to a SCIR cell if it has not already been exported. Should only be called
    /// on top cells or un-flattened cells.
    fn to_scir_cell(&self, lib_ctx: &mut ScirLibExportContext<S>) -> Result<ChildId, ConvError> {
        if let Some(conv) = lib_ctx.conv.cells.get(&self.id) {
            return match conv {
                SubstrateCellConversion::Cell(c) => Ok(c.cell_id.unwrap().into()),
                SubstrateCellConversion::Primitive(p) => Ok(p.primitive_id.into()),
            };
        }

        let name = lib_ctx.cell_names.assign_name(self.id, &self.name);

        Ok(match &self.contents {
            RawCellContents::Cell(_) => {
                let mut cell_ctx = ScirCellExportContext::new(Cell::new(name));
                let mut conv = self.export_instances(lib_ctx, &mut cell_ctx, FlatExport::No)?;
                let ScirCellExportContext {
                    cell: scir_cell, ..
                } = cell_ctx;

                let id = lib_ctx.lib.add_cell(scir_cell);
                conv.cell_id = Some(id);
                lib_ctx.conv.add_cell(self.id, conv);

                id.into()
            }
            RawCellContents::Scir(ScirBinding {
                lib,
                cell: id,
                port_map,
            }) => {
                let map = lib_ctx.lib.merge((**lib).clone());
                let id = map.new_cell_id(*id);
                let mut conv = ScirCellConversion::new();
                conv.cell_id = Some(id);
                let cell = lib_ctx.lib.cell(id);

                for port in cell.ports() {
                    let info = cell.signal(port.signal());
                    let nodes = &port_map.get(&info.name).unwrap_or_else(|| {
                        panic!(
                            "port {} not found in SCIR binding for cell {}",
                            info.name,
                            cell.name()
                        )
                    });

                    for (i, node) in nodes.iter().enumerate() {
                        conv.signals.insert(
                            *node,
                            if info.width.is_some() {
                                info.slice().index(i)
                            } else {
                                info.slice().slice_one().unwrap()
                            },
                        );
                    }
                }

                lib_ctx.conv.add_cell(self.id, conv);

                id.into()
            }
            RawCellContents::Primitive(p) => {
                let id = lib_ctx.lib.add_primitive(p.primitive.clone());
                let mut ports = HashMap::new();
                for (port, nodes) in &p.port_map {
                    for (i, node) in nodes.iter().enumerate() {
                        ports.entry(*node).or_insert(vec![]).push((port.clone(), i));
                    }
                }
                let conv = ScirPrimitiveConversion {
                    primitive_id: id,
                    ports,
                };
                lib_ctx.conv.add_cell(self.id, conv);

                id.into()
            }
            RawCellContents::ConvertedPrimitive(p) => {
                let id = lib_ctx.lib.add_primitive(
                    <ConvertedPrimitive<S> as ConvertPrimitive<S>>::convert_primitive(p)
                        .map_err(|_| ConvError::UnsupportedPrimitive)?,
                );
                let mut ports = HashMap::new();
                for (port, nodes) in p.port_map() {
                    for (i, node) in nodes.iter().enumerate() {
                        ports.entry(*node).or_insert(vec![]).push((port.clone(), i));
                    }
                }
                let conv = ScirPrimitiveConversion {
                    primitive_id: id,
                    ports,
                };
                lib_ctx.conv.add_cell(self.id, conv);

                id.into()
            }
        })
    }
    /// Exports the instances associated with `self` into the SCIR cell specified
    /// in `cell_ctx`.
    fn export_instances(
        &self,
        lib_ctx: &mut ScirLibExportContext<S>,
        cell_ctx: &mut ScirCellExportContext,
        flatten: FlatExport,
    ) -> Result<ScirCellConversion, ConvError> {
        if flatten.is_yes() {
            assert!(
                self.contents.is_cell(),
                "can only flat-export Substrate cells"
            );
        }
        let mut conv = ScirCellConversion::new();
        let mut nodes = HashMap::new();
        let mut roots_added = HashSet::new();

        if let FlatExport::Yes(ref ports) = flatten {
            // Flattened cells need to add all non-IO nodes to the enclosing cell.
            assert_eq!(ports.len(), self.ports.len());
            for (port, s) in self.ports.iter().zip(ports) {
                let root = self.roots[&port.node()];
                roots_added.insert(root);
                nodes.insert(root, *s);
            }
        }

        for (&src, &root) in self.roots.iter() {
            let s = if !roots_added.contains(&root) {
                let s = cell_ctx.cell.add_node(self.node_name(root));
                roots_added.insert(root);
                nodes.insert(root, s);
                s
            } else {
                nodes[&root]
            };
            nodes.insert(src, s);
            conv.signals.insert(src, s);
        }

        match self.contents.as_ref() {
            RawCellKind::Scir(_)
            | RawCellKind::Primitive(_)
            | RawCellKind::ConvertedPrimitive(_) => {
                // SCIR and primitive cells do not contain Substrate instances,
                // so they cannot be flattened, and thus should not have
                // [`RawCell::export_instances`] called on them.
                unreachable!()
            }
            RawCellKind::Cell(contents) => {
                for instance in contents.instances.iter() {
                    let child_id: ChildId = match &instance.child.contents {
                        RawCellContents::Primitive(_) | RawCellContents::ConvertedPrimitive(_) => {
                            instance.child.to_scir_cell(lib_ctx)?
                        }
                        _ => {
                            if instance.child.flatten {
                                let ports = instance.connections.iter().map(|c| nodes[c]).collect();
                                let inst_conv = instance.child.export_instances(
                                    lib_ctx,
                                    cell_ctx,
                                    FlatExport::Yes(ports),
                                )?;
                                conv.instances.insert(
                                    instance.id,
                                    ScirInstanceConversion {
                                        child: instance.child.id,
                                        instance: ConvertedScirInstance::InlineCell(inst_conv),
                                    },
                                );
                                continue;
                            } else {
                                instance.child.to_scir_cell(lib_ctx)?
                            }
                        }
                    };
                    let mut sinst =
                        Instance::new(arcstr::format!("inst{}", cell_ctx.inst_idx), child_id);
                    cell_ctx.inst_idx += 1;

                    assert_eq!(instance.child.ports.len(), instance.connections.len());

                    let mut conns = HashMap::new();
                    for (port, &conn) in instance.child.ports.iter().zip(&instance.connections) {
                        conns.insert(port.node(), conn);
                    }
                    let port_map = match &instance.child.contents {
                        RawCellContents::Primitive(p) => p.port_map.clone(),
                        RawCellContents::ConvertedPrimitive(p) => p.port_map().clone(),
                        RawCellContents::Scir(p) => p.port_map().clone(),
                        _ => HashMap::from_iter(instance.child.ports.iter().map(|port| {
                            (
                                instance.child.node_name(port.node()).into(),
                                vec![port.node()],
                            )
                        })),
                    };
                    for (port, port_nodes) in port_map {
                        sinst.connect(
                            port,
                            Concat::from_iter(
                                port_nodes.into_iter().map(|node| nodes[&conns[&node]]),
                            ),
                        );
                    }

                    if let RawCellContents::ConvertedPrimitive(p) = &instance.child.contents {
                        <ConvertedPrimitive<S> as ConvertPrimitive<S>>::convert_instance(
                            p, &mut sinst,
                        )
                        .map_err(|_| ConvError::UnsupportedPrimitive)?;
                    }

                    let id = cell_ctx.cell.add_instance(sinst);
                    conv.instances.insert(
                        instance.id,
                        ScirInstanceConversion {
                            child: instance.child.id,
                            instance: ConvertedScirInstanceContent::Cell(id),
                        },
                    );
                }
            }
        }

        if flatten.is_no() {
            for port in self.ports.iter() {
                cell_ctx
                    .cell
                    .expose_port(nodes[&port.node()], port.direction());
            }
        }
        Ok(conv)
    }
}

/// The error type for Substrate functions.
#[derive(thiserror::Error, Debug, Clone)]
pub enum ConvError {
    /// An error in validating the converted SCIR library.
    #[error("error in converted SCIR library")]
    Scir(#[from] scir::Issues),
    /// An unsupported primitive was encountered during conversion.
    #[error("unsupported primitive")]
    UnsupportedPrimitive,
}

/// Export a collection of cells and all their subcells as a SCIR library.
///
/// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
/// The resulting SCIR library will **not** have a top cell set.
/// If you want a SCIR library with a known top cell, consider using [`RawCell::to_scir_lib`] instead.
pub(crate) fn export_multi_top_scir_lib<S: Schema + ?Sized>(
    cells: &[&RawCell<S>],
) -> Result<RawLib<S>, ConvError> {
    let mut lib_ctx = ScirLibExportContext::new();

    for &cell in cells {
        cell.to_scir_cell(&mut lib_ctx)?;
    }

    Ok(RawLib {
        scir: lib_ctx.lib.build()?,
        conv: lib_ctx.conv.build(),
    })
}
