use std::fmt;

#[derive(Debug)]
pub enum Error {
    Pegasus(std::process::ExitStatus),
    Io(std::io::Error),
    OsString(std::ffi::OsString),
    Tera(tera::Error),
    Internal(anyhow::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")?;
        Ok(())
    }
}

impl std::error::Error for Error {}
