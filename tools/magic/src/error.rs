use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Error running Magic.
    #[error("error running Magic")]
    Magic(std::process::ExitStatus),
    /// Error performing I/O.
    #[error("error performing I/O")]
    Io(#[from] std::io::Error),
    /// Error rendering templates.
    #[error("error rendering templates")]
    Tera(#[from] tera::Error),
    /// An unknown internal error.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
