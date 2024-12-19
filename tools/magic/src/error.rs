use std::fmt;

#[derive(Debug)]
pub enum Error {
    Magic(std::process::ExitStatus),
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

impl std::error::Error for Error {}
