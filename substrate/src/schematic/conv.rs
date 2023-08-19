//! Substrate to SCIR conversion.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use scir::{
    Cell, CellId as ScirCellId, CellInner, IndexOwned, Instance, InstancePathTail, LibraryBuilder,
    SignalPathTail, TopKind,
};
use serde::{Deserialize, Serialize};
use uniquify::Names;

use crate::io::{Node, NodePath, TerminalPath};
use crate::schematic::InstancePath;

use super::{BlackboxElement, CellId, InstanceId, RawCell, RawCellContent};

/// An SCIR library with associated conversion metadata.
#[derive(Debug, Clone)]
pub struct RawLib<P> {
    /// The SCIR library.
    pub scir: scir::Library<P>,
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
    pub(crate) id_mapping: HashMap<CellId, ScirCellId>,
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<CellId, ScirCellConversion>,
    pub(crate) top: scir::CellId,
}

#[derive(Debug, Clone, Default)]
struct ScirLibConversionBuilder {
    pub(crate) id_mapping: HashMap<CellId, ScirCellId>,
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
            id_mapping: self.id_mapping,
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

impl<P> RawLib<P> {
    fn convert_instance_path_inner<'a>(
        &self,
        top: CellId,
        instances: impl IntoIterator<Item = &'a InstanceId>,
    ) -> Option<(Vec<scir::InstanceId>, &ScirCellConversion, scir::CellId)> {
        todo!()
    }
    /// Converts a Substrate [`NodePath`] to a SCIR [`scir::SignalPath`].
    pub fn convert_node_path(&self, path: &NodePath) -> Option<scir::SignalPath> {
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
    pub fn convert_terminal_path(&self, path: &TerminalPath) -> Option<Vec<scir::SignalPath>> {
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
        signals: &mut Vec<scir::SignalPath>,
    ) {
        // let (signal, index) = slice;
        instances.push(id);
        let inst = parent_cell.instance(id);
        todo!();
        instances.pop().unwrap();
    }

    /// Must ensure that `instances` is returned to its original value by the end of the
    /// function call.
    fn find_connected_terminals(
        &self,
        conv: &ScirCellConversion,
        slice: scir::SliceOne,
        instances: &mut Vec<scir::InstanceId>,
        signals: &mut Vec<scir::SignalPath>,
    ) {
        let parent_cell = self.scir.cell(self.conv.id_mapping[&conv.id]);
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
    /// A Substrate primitive that translates to a [`scir::PrimitiveDevice`].
    Primitive {
        /// The SCIR ID of the translated primitive device.
        id: scir::PrimitiveDeviceId,
    },
    /// A Substrate primitive that translates to a [`scir::Instance`].
    Instance(scir::InstanceId),
}

#[derive(Debug, Clone)]
struct ScirExportData<P> {
    lib: LibraryBuilder<P>,
    conv: ScirLibConversionBuilder,
    cell_names: Names<CellId>,
}

impl<P> ScirExportData<P> {
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

struct ScirExportContext {
    id: CellId,
    inst_idx: u64,
    prim_idx: u64,
    cell: scir::Cell,
}

impl ScirExportContext {
    #[inline]
    pub fn new(id: CellId, cell: scir::Cell) -> Self {
        Self {
            id,
            inst_idx: 0,
            prim_idx: 0,
            cell,
        }
    }

    fn whitebox_contents_mut(&mut self) -> &mut CellInner {
        self.cell.contents_mut().as_mut().unwrap_cell()
    }
}

impl<P> RawCell<P> {
    /// Export this cell and all subcells as a SCIR library.
    ///
    /// Returns the SCIR library and metadata for converting between SCIR and Substrate formats.
    pub(crate) fn to_scir_lib(&self, kind: TopKind) -> Result<RawLib<P>, scir::Issues> {
        let mut data = ScirExportData::new(self.name.clone());
        let scir_id = self.to_scir_cell(&mut data);
        data.lib.set_top(scir_id, kind);
        data.conv.set_top(self.id, scir_id);

        Ok(RawLib {
            scir: data.lib.build()?,
            conv: data.conv.build(),
        })
    }

    fn to_scir_cell(&self, data: &mut ScirExportData<P>) -> ScirCellId {
        let name = data.cell_names.assign_name(self.id, &self.name);

        // Create the SCIR cell as a whitebox for now.
        // If this Substrate cell is actually a blackbox,
        // the contents of this SCIR cell will be made into a blackbox
        // by calling `cell.set_contents`.
        let cell = Cell::new(name);

        let mut ctx = ScirExportContext::new(self.id, cell);
        let conv = self.to_scir_cell_inner(data, &mut ctx, FlatExport::No);
        let ScirExportContext { cell, .. } = ctx;

        let id = data.lib.add_cell(cell);
        data.conv.add_cell(self.id, conv);
        data.conv.id_mapping.insert(self.id, id);

        id
    }

    fn to_scir_cell_inner(
        &self,
        data: &mut ScirExportData<P>,
        ctx: &mut ScirExportContext,
        flatten: FlatExport,
    ) -> ScirCellConversion {
        todo!()
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
