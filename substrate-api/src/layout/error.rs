//! Layout result and error types.

/// The [`LayoutError`] result type.
pub type LayoutResult<T> = Result<T, LayoutError>;

/// A layout error.
#[derive(thiserror::Error, Debug, Clone)]
pub enum LayoutError {
    /// An error with exporting a Substrate cell to GDS.
    #[error("error during gds export: {0:?}")]
    GdsExport(GdsExportError),
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
