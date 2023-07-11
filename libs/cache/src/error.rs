//! Cache error types.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    Rusqlite(#[from] tokio_rusqlite::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Flexbuffers(#[from] flexbuffers::DeserializationError),
    #[error(transparent)]
    Rpc(#[from] tonic::Status),
}
