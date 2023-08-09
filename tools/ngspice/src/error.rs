//! Spectre errors.

use std::sync::Arc;

use thiserror::Error as ThisError;

/// The result type returned by Spectre library functions.
pub type Result<T> = std::result::Result<T, Error>;

/// Possible Spectre errors.
#[derive(ThisError, Debug)]
pub enum Error {
    /// I/O error.
    #[error("io error")]
    Io(#[from] std::io::Error),
    /// Template parsing/rendering error.
    #[error("template error")]
    Template(#[from] tera::Error),
    /// Error invoking ngspice.
    #[error("error running ngspice")]
    NgspiceError,
    /// Error parsing output rawfile.
    #[error("error parsing output rawfile")]
    RawfileParse(#[from] spice_rawfile::error::Error),
    /// Error generating results.
    #[error("error generating spectre results")]
    Generator(#[from] Arc<Error>),
    /// Error caching results.
    #[error("error generating spectre results")]
    Caching(#[from] Arc<cache::error::Error>),
}
