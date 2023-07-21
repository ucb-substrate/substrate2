//! Layout result and error types.

use arcstr::ArcStr;
use rust_decimal::Decimal;

/// The [`LayoutError`] result type.
pub type LayoutResult<T> = Result<T, LayoutError>;

/// A layout error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum LayoutError {
    /// An error with exporting a Substrate cell to GDS.
    #[error("error during gds export: {0:?}")]
    GdsExport(GdsExportError),
    /// An error with defining the IO of a Substrate layout cell.
    #[error("error specifying layout IO")]
    IoDefinition,
}

impl From<GdsExportError> for LayoutError {
    fn from(e: GdsExportError) -> Self {
        Self::GdsExport(e)
    }
}

/// The [`GdsExportError`] result type.
pub type GdsExportResult<T> = Result<T, GdsExportError>;

/// A GDS export error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum GdsExportError {
    /// An error coverting an integer into a type that can be encoded in GDS.
    #[error("error converting an integer to the necessary type: {0:?}")]
    TryFromInt(std::num::TryFromIntError),
    /// An error in writing a GDS file.
    #[error("error writing GDS file: {0:?}")]
    Write(gds::GdsError),
}

impl From<std::num::TryFromIntError> for GdsExportError {
    fn from(e: std::num::TryFromIntError) -> Self {
        Self::TryFromInt(e)
    }
}

impl From<gds::GdsError> for GdsExportError {
    fn from(e: gds::GdsError) -> Self {
        Self::Write(e)
    }
}

/// The [`GdsImportError`] result type.
pub type GdsImportResult<T> = Result<T, GdsImportError>;

/// A GDS import error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum GdsImportError {
    /// An error coverting an integer into a type that can be encoded in GDS.
    #[error("error converting an integer to the necessary type: {0:?}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    /// An error in writing a GDS file.
    #[error("error writing GDS file: {0:?}")]
    Write(#[from] gds::GdsError),
    /// No cell of the given name exists in the GDS library.
    #[error("cell not found in GDS library: {0}")]
    CellNotFound(ArcStr),
    /// More than one cell with the given name was defined in the same GDS library.
    #[error("found more than one cell with the same name in a GDS library: {0}")]
    DuplicateCell(ArcStr),
    /// Use of an unsupported GDS feature.
    #[error("unsupported GDS feature: {0}")]
    Unsupported(ArcStr),
    /// An GDS struct contained an invalid GDS boundary.
    ///
    /// GDS boundaries must start and end at the same point.
    #[error("invalid GDS boundary (boundaries must start and end at the same point)")]
    InvalidGdsBoundary,
    /// The database unit in a GDS file does not match that expected by the PDK.
    #[error("GDS file units ({0}) do not match PDK units ({1})")]
    MismatchedUnits(Decimal, Decimal),
}
