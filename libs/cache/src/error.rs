//! Cache error types.

use std::sync::Arc;

/// A result type returning cache errors.
pub type Result<T> = std::result::Result<T, Error>;

/// A result type returning reference counted cache errors.
///
/// Stores an [`Arc<Error>`] since the error will be stuck inside a [`OnceCell`] and is
/// cannot be owned without cloning.
pub type ArcResult<T> = std::result::Result<T, Arc<Error>>;

/// The error type for cache functions.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(missing_docs)]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[allow(missing_docs)]
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[allow(missing_docs)]
    #[error(transparent)]
    Rusqlite(#[from] tokio_rusqlite::Error),
    #[allow(missing_docs)]
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[allow(missing_docs)]
    #[error(transparent)]
    Flexbuffers(#[from] flexbuffers::DeserializationError),
    #[allow(missing_docs)]
    #[error(transparent)]
    Rpc(#[from] tonic::Status),
    #[allow(missing_docs)]
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    /// An error thrown when a user-provided generator panics.
    #[error("generator panicked")]
    Panic,
    /// An error thrown by failing to connect to the cache server.
    #[error("failed to connect to cache server")]
    Connection,
}
