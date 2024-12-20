use std::fmt;

#[derive(Debug)]
pub enum Error {
    Netgen(std::process::ExitStatus),
    Io(std::io::Error),
    Tera(tera::Error),
    Internal(anyhow::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")?;
        Ok(())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<tera::Error> for Error {
    fn from(value: tera::Error) -> Self {
        Self::Tera(value)
    }
}

impl std::error::Error for Error {}
