//! Cache error types.

use std::sync::Arc;

/// A result type returning cache errors.
pub type Result<T> = std::result::Result<T, Error>;

/// A result type returning reference counted cache errors.
///
/// Stores an [`Arc<Error>`] since the error will be stuck inside a
/// [`OnceCell`](once_cell::sync::OnceCell) and cannot be owned without cloning.
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
    /// The user-provided generator panicked.
    #[error("generator panicked")]
    Panic,
    /// Exponential backoff for polling the server failed.
    #[error(transparent)]
    Backoff(#[from] Box<backoff::Error<Error>>),
    /// The desired cache entry is currently being loaded.
    #[error("entry is currently being loaded")]
    EntryLoading,
    /// The desired cache entry is currently unavailable.
    #[error("entry is currently unavailable")]
    EntryUnavailable,
    /// The desired cache entry cannot be assigned.
    #[error("entry cannot be assigned")]
    EntryUnassignable,
}

/// The error type returned by [`CacheHandle::try_inner`](crate::CacheHandle::try_inner).
pub enum TryInnerError<'a, E> {
    /// An error thrown by the cache.
    CacheError(Arc<Error>),
    /// An error thrown by the generator.
    GeneratorError(&'a E),
}

impl<'a, E> From<Arc<Error>> for TryInnerError<'a, E> {
    fn from(value: Arc<Error>) -> Self {
        Self::CacheError(value)
    }
}

impl<'a, E> From<&'a E> for TryInnerError<'a, E> {
    fn from(value: &'a E) -> Self {
        Self::GeneratorError(value)
    }
}
