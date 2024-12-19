//! Netlist export.

use crate::schema::Schema;
use crate::{Library, NetlistLibConversion};
use std::io::Write;
use std::path::Path;

/// A netlister that tracks how cells and instances are translated between SCIR and the output netlist format.
pub trait ConvertibleNetlister<S: Schema + ?Sized> {
    /// The error type returned when writing out a SCIR netlist.
    type Error: From<std::io::Error>;

    /// The netlist options type.
    ///
    /// Many netlisters accept options, allowing the user to configure things like
    /// netlist indentation, naming conventions, etc. This is the type of the object
    /// that stores those options.
    type Options<'a>;

    /// Writes a netlist of a SCIR library to the provided output stream.
    fn write_scir_netlist<W: Write>(
        &self,
        lib: &Library<S>,
        out: &mut W,
        opts: Self::Options<'_>,
    ) -> Result<NetlistLibConversion, Self::Error>;

    /// Writes a netlist of a SCIR library to a file at the given path.
    ///
    /// The file and any parent directories will be created if necessary.
    fn write_scir_netlist_to_file(
        &self,
        lib: &Library<S>,
        path: impl AsRef<Path>,
        opts: Self::Options<'_>,
    ) -> Result<NetlistLibConversion, Self::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = std::fs::File::create(path)?;
        let conv = self.write_scir_netlist(lib, &mut f, opts)?;
        Ok(conv)
    }
}
