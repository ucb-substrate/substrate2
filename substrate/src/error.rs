//! Error types and error handling utilities.

/// A result type returning Substrate errors.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for Substrate functions.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// An internal Substrate error that indicates a bug in the source code.
    #[error("internal Substrate error")]
    Internal,
}
