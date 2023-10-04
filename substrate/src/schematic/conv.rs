//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

use arcstr::ArcStr;
use scir::{Cell, CellId as ScirCellId, ChildId, Instance, LibraryBuilder, PrimitiveId};
use serde::{Deserialize, Serialize};
use substrate::schematic::{ConvertedPrimitive, ScirCellInner};
use uniquify::Names;

use crate::io::{Node, NodePath, TerminalPath};
use crate::pdk::Pdk;
use crate::schematic::schema::Schema;
use crate::schematic::{ConvertPrimitive, InstancePath, RawCellContents, RawCellKind};

use super::{CellId, InstanceId, RawCell};

/// An SCIR library with associated conversion metadata.
pub struct RawLib<S: Schema> {
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

pub(crate) struct RawLibBuilder<S: Schema> {
    pub(crate) scir: scir::LibraryBuilder<S>,
    pub(crate) conv: ScirLibConversionBuilder,
}

impl<S: Schema<Primitive = impl Clone>> Clone for RawLibBuilder<S> {
    fn clone(&self) -> Self {
        Self {
            scir: self.scir.clone(),
            conv: self.conv.clone(),
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug>> std::fmt::Debug for RawLibBuilder<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawLib");
        let _ = builder.field("scir", &self.scir);
        let _ = builder.field("conv", &self.conv);
        builder.finish()
    }
}

impl<S: Schema> RawLibBuilder<S> {
    pub(crate) fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            scir: LibraryBuilder::new(name),
            conv: Default::default(),
        }
    }

    pub(crate) fn build(self) -> RawLib<S> {
        RawLib {
            scir: self.scir.build().unwrap(),
            conv: self.conv.build(),
        }
    }
}

/// Metadata associated with a conversion from a Substrate schematic to a SCIR library.
///
/// Provides helpers for retrieving SCIR objects from their Substrate IDs.
#[derive(Debug, Clone)]
pub struct ScirLibConversion {
    pub(crate) cell_mapping: HashMap<CellId, ScirCellId>,
    pub(crate) primitive_mapping: HashMap<CellId, PrimitiveId>,
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
    pub(crate) top: Option<scir::CellId>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ScirLibConversionBuilder {
    pub(crate) cell_mapping: HashMap<CellId, ScirCellId>,
    pub(crate) primitive_mapping: HashMap<CellId, PrimitiveId>,
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
            cell_mapping: self.cell_mapping,
            primitive_mapping: self.primitive_mapping,
            cells: self.cells,
            top: self.top,
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

impl<S: Schema> RawLib<S> {
    fn convert_instance_path_inner<'a>(
        &self,
        top: CellId,
        instances: impl IntoIterator<Item = &'a InstanceId>,
    ) -> Option<(Vec<scir::InstanceId>, &ScirCellConversion, scir::CellId)> {
        todo!()
    }

    /// Converts a Substrate [`NodePath`] to a SCIR [`scir::SignalPath`].
    pub fn convert_node_path(&self, path: &NodePath) -> Option<scir::SliceOnePath> {
        todo!()
    }

    /// Converts a Substrate [`InstancePath`] to a SCIR [`scir::InstancePath`].
    pub fn convert_instance_path(&self, path: &InstancePath) -> Option<scir::InstancePath> {
        todo!()
    }

    /// Converts a Substrate [`TerminalPath`] to a list SCIR [`scir::SignalPath`]s that are
    /// associated with the terminal at that path.
    ///
    /// Returns [`None`] if the path is invalid. Only flattened instances will
    /// return more than one [`scir::SignalPath`], and unconnected terminals will return
    /// `Some(vec![])`.
    pub fn convert_terminal_path(&self, path: &TerminalPath) -> Option<Vec<scir::SliceOnePath>> {
        todo!()
    }

    /// Must ensure that `instances` is returned to its original value by the end of the
    /// function call.
    fn find_connected_terminals_in_scir_instance(
        &self,
        parent_cell: &scir::Cell,
        id: scir::InstanceId,
        slice: scir::SliceOne,
        instances: &mut Vec<scir::InstanceId>,
        signals: &mut Vec<scir::SliceOnePath>,
    ) {
        // let (signal, index) = slice;
        instances.push(id);
        let inst = parent_cell.instance(id);
        todo!();
    }

    /// Must ensure that `instances` is returned to its original value by the end of the
    /// function call.
    fn find_connected_terminals(
        &self,
        conv: &ScirCellConversion,
        slice: scir::SliceOne,
        instances: &mut Vec<scir::InstanceId>,
        signals: &mut Vec<scir::SliceOnePath>,
    ) {
        let parent_cell = self.scir.cell(self.conv.cell_mapping[&conv.id]);
        for (_, conv) in conv.instances.iter() {
            todo!();
        }
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

/// Data used to map between a Substrate cell and a SCIR cell.
///
/// Flattened cells do not have a conversion.
#[derive(Debug, Clone)]
pub(crate) struct ScirCellConversion {
    /// The Substrate cell ID that this conversion corresponds to.
    pub(crate) id: CellId,
    /// Whether or not this cell is the top cell.
    pub(crate) top: bool,
    /// Map Substrate nodes to SCIR signal IDs and indices.
    pub(crate) signals: HashMap<Node, scir::SliceOne>,
    /// Map Substrate instance IDs to SCIR instances and their underlying Substrate cell.
    pub(crate) instances: HashMap<InstanceId, ScirInstanceConversion>,
}

impl ScirCellConversion {
    #[inline]
    pub fn new(id: CellId) -> Self {
        Self {
            id,
            top: false,
            signals: HashMap::new(),
            instances: HashMap::new(),
        }
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

#[derive(Debug, Clone)]
pub(crate) enum ScirPrimitiveDeviceConversion {
    /// A Substrate primitive that translates to a [`scir::Instance`].
    Instance(scir::InstanceId),
}

pub(crate) struct ScirLibExportContext<S: Schema> {
    lib: LibraryBuilder<S>,
    conv: ScirLibConversionBuilder,
    cell_names: Names<CellId>,
}

impl<S: Schema<Primitive = impl Clone>> Clone for ScirLibExportContext<S> {
    fn clone(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            conv: self.conv.clone(),
            cell_names: self.cell_names.clone(),
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug>> std::fmt::Debug for ScirLibExportContext<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ScirLibExportContext");
        let _ = builder.field("lib", &self.lib);
        let _ = builder.field("conv", &self.conv);
        let _ = builder.field("cell_names", &self.cell_names);
        builder.finish()
    }
}

impl<S: Schema> ScirLibExportContext<S> {
    fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            lib: LibraryBuilder::new(name),
            conv: ScirLibConversionBuilder::new(),
            cell_names: Names::new(),
        }
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
    id: CellId,
    inst_idx: u64,
    prim_idx: u64,
    cell: scir::Cell,
}

impl ScirCellExportContext {
    #[inline]
    pub fn new(id: CellId, cell: scir::Cell) -> Self {
        Self {
            id,
            inst_idx: 0,
            prim_idx: 0,
            cell,
        }
    }
}

impl<S: Schema> RawCell<S> {
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
    pub(crate) fn to_scir_lib(&self) -> Result<RawLib<S>, ConvError> {
        let mut lib_ctx = ScirLibExportContext::new(&self.name);
        let scir_id = self.to_scir_cell(&mut lib_ctx)?;

        if let ChildId::Cell(scir_id) = scir_id {
            lib_ctx.lib.set_top(scir_id);
            lib_ctx.conv.set_top(self.id, scir_id);
        }

        Ok(RawLib {
            scir: lib_ctx.lib.build()?,
            conv: lib_ctx.conv.build(),
        })
    }

    fn to_scir_cell(&self, lib_ctx: &mut ScirLibExportContext<S>) -> Result<ChildId, ConvError> {
        let name = lib_ctx.cell_names.assign_name(self.id, &self.name);

        Ok(match &self.contents {
            RawCellContents::Cell(c) => {
                let mut cell_ctx = ScirCellExportContext::new(self.id, Cell::new(name));
                let conv = self.export_instances(lib_ctx, &mut cell_ctx, FlatExport::No)?;
                let ScirCellExportContext {
                    cell: scir_cell, ..
                } = cell_ctx;

                let id = lib_ctx.lib.add_cell(scir_cell);
                lib_ctx.conv.add_cell(self.id, conv);
                lib_ctx.conv.cell_mapping.insert(self.id, id);

                id.into()
            }
            RawCellContents::Scir(ScirCellInner { lib, cell: id }) => {
                let map = lib_ctx.lib.merge((**lib).clone());
                map.new_cell_id(*id).into()
            }
            RawCellContents::Primitive(p) => lib_ctx.lib.add_primitive(p.clone()).into(),
            RawCellContents::ConvertedPrimitive(p) => lib_ctx
                .lib
                .add_primitive(
                    <ConvertedPrimitive<S> as ConvertPrimitive<S>>::convert_primitive(p)
                        .map_err(|_| ConvError::UnsupportedPrimitive)?,
                )
                .into(),
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
                self.contents.is_cell() || self.contents.is_scir(),
                "can only flat-export cells"
            );
        }
        let mut conv = ScirCellConversion::new(cell_ctx.id);
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
            RawCellKind::Scir(contents) => {
                if flatten.is_yes() {
                    todo!()
                } else {
                    // If the SCIR cell does not need to flattened, merge in the SCIR cell's library
                    // and create a mapping from its associated Substrate cell ID to the ID in the merged
                    // library.
                    let mapping = lib_ctx.lib.merge((*contents.lib).clone());
                    lib_ctx
                        .conv
                        .cell_mapping
                        .insert(self.id, mapping.new_cell_id(contents.cell));
                }
            }
            RawCellKind::Primitive(_contents) => {
                // Primitive cells do not have nested instances that Substrate can handle,
                // so [`RawCell::export_instances`] should never be called on a primitive
                // cell.
                unreachable!()
            }
            RawCellKind::ConvertedPrimitive(_contents) => {
                // Primitive cells do not have nested instances that Substrate can handle,
                // so [`RawCell::export_instances`] should never be called on a primitive
                // cell.
                unreachable!()
            }
            RawCellKind::Cell(contents) => {
                for instance in contents.instances.iter() {
                    let child_id: ChildId = match &instance.child.contents {
                        RawCellContents::Primitive(p) => {
                            if !lib_ctx
                                .conv
                                .primitive_mapping
                                .contains_key(&instance.child.id)
                            {
                                lib_ctx.conv.primitive_mapping.insert(
                                    instance.child.id,
                                    lib_ctx.lib.add_primitive(p.clone()),
                                );
                            }

                            (*lib_ctx
                                .conv
                                .primitive_mapping
                                .get(&instance.child.id)
                                .unwrap())
                            .into()
                        }
                        RawCellContents::ConvertedPrimitive(p) => {
                            if !lib_ctx
                                .conv
                                .primitive_mapping
                                .contains_key(&instance.child.id)
                            {
                                lib_ctx.conv.primitive_mapping.insert(
                                    instance.child.id,
                                    lib_ctx.lib.add_primitive(
                                        <ConvertedPrimitive<S> as ConvertPrimitive<S>>::convert_primitive(p)
                                            .map_err(|_| ConvError::UnsupportedPrimitive)?,
                                    ),
                                );
                            }

                            (*lib_ctx
                                .conv
                                .primitive_mapping
                                .get(&instance.child.id)
                                .unwrap())
                            .into()
                        }
                        _ => {
                            if !lib_ctx.conv.cell_mapping.contains_key(&instance.child.id) {
                                let id = instance.child.to_scir_cell(lib_ctx)?.unwrap_cell();
                                lib_ctx.conv.cell_mapping.insert(instance.child.id, id);
                            }

                            (*lib_ctx.conv.cell_mapping.get(&instance.child.id).unwrap()).into()
                        }
                    };
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
                    } else {
                        let mut sinst =
                            Instance::new(arcstr::format!("inst{}", cell_ctx.inst_idx), child_id);
                        cell_ctx.inst_idx += 1;
                        assert_eq!(instance.child.ports.len(), instance.connections.len());
                        for (port, &conn) in instance.child.ports.iter().zip(&instance.connections)
                        {
                            let scir_port_name = instance.child.node_name(port.node());
                            sinst.connect(scir_port_name, nodes[&conn]);
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
#[derive(thiserror::Error, Debug)]
pub enum ConvError {
    /// An error in validating the converted SCIR library.
    #[error("error in converted SCIR library")]
    Scir(#[from] scir::Issues),
    #[error("unsupported primitive")]
    UnsupportedPrimitive,
}
