//! Conversions between layout formats.

use std::collections::HashMap;

use layir::Cell;
use layir::Direction;
use layir::LibraryBuilder;
use layir::Port;

use super::element::Element;
use super::element::RawCell;

use super::element::CellId as SubCellId;
use layir::CellId as LayCellId;

/// Metadata associated with a conversion from a Substrate schematic to a LayIR library.
///
/// Provides helpers for retrieving LayIR objects from their Substrate IDs.
#[derive(Debug, Default, Clone)]
pub struct LayirLibConversion {
    /// Map from Substrate cell IDs to cell conversion metadata.
    pub(crate) cells: HashMap<SubCellId, layir::CellId>,
}

#[derive(Debug, Clone)]
pub(crate) struct LayirLibExportContext<L> {
    lib: layir::LibraryBuilder<L>,
    conv: LayirLibConversion,
}

/// A LayIR library with associated conversion metadata.
pub struct RawLib<L> {
    /// The LayIR library.
    pub layir: layir::Library<L>,
    /// Associated conversion metadata.
    ///
    /// Can be used to retrieve LayIR objects from their corresponding Substrate IDs.
    pub conv: LayirLibConversion,
}

impl<L> Default for LayirLibExportContext<L> {
    fn default() -> Self {
        Self {
            lib: LibraryBuilder::new(),
            conv: LayirLibConversion::default(),
        }
    }
}

impl<L> LayirLibExportContext<L> {
    #[inline]
    fn new() -> Self {
        Self::default()
    }
}

/// Error when exporting a layout cell to LayIR.
#[derive(thiserror::Error, Clone, Debug)]
#[error("error exporting layout cell to LayIR")]
pub struct LayirExportError;

impl<L: Clone> RawCell<L> {
    /// Export this cell and all subcells as a LayIR library.
    ///
    /// Returns the LayIR library and metadata for converting between LayIR and Substrate formats.
    ///
    /// Consider using [`export_multi_top_layir_lib`] if you need to export multiple cells
    /// to the same LayIR library.
    pub(crate) fn to_layir_lib(&self) -> Result<RawLib<L>, LayirExportError> {
        let mut lib_ctx = LayirLibExportContext::new();
        self.to_layir_cell(&mut lib_ctx)?;

        Ok(RawLib {
            layir: lib_ctx.lib.build().map_err(|_| LayirExportError)?,
            conv: lib_ctx.conv,
        })
    }

    /// Exports this [`RawCell`] to a LayIR cell if it has not already been exported. Should only be called
    /// on top cells or un-flattened cells.
    fn to_layir_cell(
        &self,
        lib_ctx: &mut LayirLibExportContext<L>,
    ) -> Result<LayCellId, LayirExportError> {
        if let Some(conv) = lib_ctx.conv.cells.get(&self.id) {
            return Ok(*conv);
        }

        let mut cell = Cell::new(self.name.clone());
        for elt in self.elements() {
            match elt {
                Element::Instance(inst) => {
                    let child = inst.raw_cell().to_layir_cell(lib_ctx)?;
                    let inst = layir::Instance::with_transformation(
                        child,
                        inst.raw_cell().name.clone(),
                        inst.trans,
                    );
                    cell.add_instance(inst);
                }
                Element::Shape(shape) => {
                    cell.add_element(shape.clone());
                }
                Element::Text(text) => {
                    cell.add_element(text.clone());
                }
            }
        }
        for (name, port) in self.ports() {
            // TODO: use correct port directions
            let mut lport = Port::new(Direction::InOut);
            lport.add_element(port.primary.clone());
            for shape in port.unnamed_shapes.iter() {
                lport.add_element(shape.clone());
            }
            for (_, shape) in port.named_shapes.iter() {
                lport.add_element(shape.clone());
            }
            cell.add_port(arcstr::format!("{}", name), lport);
        }
        let id = lib_ctx.lib.add_cell(cell);
        lib_ctx.conv.cells.insert(self.id, id);
        Ok(id)
    }
}

/// Export a collection of cells and all their subcells as a LayIR library.
///
/// Returns the LayIR library and metadata for converting between LayIR and Substrate formats.
/// The resulting LayIR library will **not** have a top cell set.
/// If you want a LayIR library with a known top cell, consider using [`RawCell::to_layir_lib`] instead.
pub(crate) fn export_multi_top_layir_lib<L: Clone>(
    cells: &[&RawCell<L>],
) -> Result<RawLib<L>, LayirExportError> {
    let mut lib_ctx = LayirLibExportContext::new();

    for &cell in cells {
        cell.to_layir_cell(&mut lib_ctx)?;
    }

    Ok(RawLib {
        layir: lib_ctx.lib.build().map_err(|_| LayirExportError)?,
        conv: lib_ctx.conv,
    })
}
