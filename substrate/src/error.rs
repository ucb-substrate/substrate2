//! Error types and error handling utilities.

use std::sync::Arc;

use crate::layout::error::LayoutError;

/// A result type returning Substrate errors.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for Substrate functions.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// An error in a layout-related routine.
    #[error("error in layout: {0:?}")]
    Layout(LayoutError),
    /// An internal Substrate error that indicates a bug in the source code.
    #[error("internal Substrate error")]
    Internal,
    /// An error thrown when a thread spawned during generation panics.
    #[error("a thread panicked")]
    Panic,
    /// An arbitrary error for external use.
    #[error(transparent)]
    Boxed(#[from] Arc<dyn std::error::Error + Send + Sync>),
    /// An [`anyhow::Error`] for external use.
    #[error(transparent)]
    Anyhow(#[from] Arc<anyhow::Error>),
}

impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        Error::Layout(value)
    }
}