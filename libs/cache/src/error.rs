//! Cache error types.

/// A result type returning cache errors.
pub type Result<T> = std::result::Result<T, Error>;

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
    /// An error thrown when a user-provided generator panics.
    #[error("generator panicked")]
    Panic,
}
