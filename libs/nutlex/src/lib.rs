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

/// Nutmeg reading/writing options.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Options {
    /// The endianness of floating point numbers in the Nutmeg file.
    ///
    /// Only used when the floats are in binary format; ASCII formatted floats ignore this option.
    pub endianness: ByteOrder,
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        Self {
            endianness: ByteOrder::BigEndian,
        }
    }
}

/// Byte order for numbers.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ByteOrder {
    /// Big endian.
    BigEndian,
    /// Little endian.
    LittleEndian,
}

/// Parse the given rawfile data.
pub fn parse<T>(input: &T, options: Options) -> Result<Rawfile<'_>>
where
    T: AsRef<[u8]>,
{
    match parser::analyses(input.as_ref(), options) {
        Ok((_, analyses)) => Ok(Rawfile { analyses }),
        Err(_) => Err(Error::Parse),
    }
}
