//! Spectre errors.

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
    /// Error invoking Spectre.
    #[error("error running Spectre")]
    SpectreError,
    /// Error parsing PSF output files.
    #[error("error parsing PSF output file")]
    PsfParse,
    /// Error caching results.
    #[error("error caching spectre results")]
    Caching,
}
