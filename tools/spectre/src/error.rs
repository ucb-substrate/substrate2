use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("template error")]
    Template(#[from] tera::Error),
    #[error("spectre exited unsuccessfully")]
    SpectreError,
    #[error("error parsing PSF output file")]
    PsfParse,
}
