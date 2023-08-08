//! A SPICE rawfile parser.
#![warn(missing_docs)]

use error::{Error, Result};
use parser::Analysis;
use serde::Serialize;

pub mod error;
pub mod parser;

/// A parsed SPICE rawfile.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rawfile<'a> {
    /// The analyses contained in this file.
    pub analyses: Vec<Analysis<'a>>,
}

/// Parse the given rawfile data.
pub fn parse<T>(input: &T) -> Result<Rawfile<'_>>
where
    T: AsRef<[u8]>,
{
    match parser::analyses(input.as_ref()) {
        Ok((_, analyses)) => Ok(Rawfile { analyses }),
        Err(_) => Err(Error::Parse),
    }
}
