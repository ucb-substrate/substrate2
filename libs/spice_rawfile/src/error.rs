//! Error handling.
use thiserror::Error;

/// An error from manipulating a rawfile.
#[derive(Debug, Error)]
pub enum Error {
    /// A parsing error.
    #[error("parse error")]
    Parse,
    /// An I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// The result type returned by most SPICE rawfile functions.
pub type Result<T> = std::result::Result<T, Error>;
