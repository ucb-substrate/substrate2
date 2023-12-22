//! Error types and error handling utilities.

use std::process::Command;
use std::sync::Arc;

use gds::GdsError;

use crate::layout::error::{GdsImportError, LayoutError};
use crate::schematic::conv::ConvError;

/// A result type returning Substrate errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type for Substrate functions.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// An error in a layout-related routine.
    #[error("error in layout: {0:?}")]
    Layout(LayoutError),
    /// An I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    /// An internal Substrate error that indicates a bug in the source code.
    #[error("internal Substrate error")]
    Internal,
    /// An error thrown by caching functions.
    #[error(transparent)]
    CacheError(#[from] Arc<cache::error::Error>),
    /// An error thrown when a thread spawned during generation panics.
    #[error("a thread panicked")]
    Panic,
    /// Executing a command failed.
    #[error("error executing command: {0:?}")]
    CommandFailed(Arc<Command>),
    /// GDS error.
    #[error("gds error: {0}")]
    Gds(#[from] GdsError),
    /// Error importing GDS.
    #[error("error importing GDS: {0}")]
    GdsImport(#[from] GdsImportError),
    /// An arbitrary error for external use.
    #[error(transparent)]
    Boxed(#[from] Arc<dyn std::error::Error + Send + Sync>),
    /// An [`anyhow::Error`] for external use.
    #[error(transparent)]
    Anyhow(#[from] Arc<anyhow::Error>),
    /// Schematic to SCIR conversion produced errors.
    #[error("error converting to SCIR: {0}")]
    ScirConversion(#[from] ConvError),
    /// An error indicating that the schema does not support an instantiated primitive.
    #[error("schema does not support primitive")]
    UnsupportedPrimitive,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(Arc::new(value))
    }
}

impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        Error::Layout(value)
    }
}
