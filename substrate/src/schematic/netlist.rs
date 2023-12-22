//! Netlist export.

use crate::context::Context;
use crate::schematic::conv::RawLib;
use crate::schematic::Schematic;
use scir::{Library, NetlistLibConversion};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use substrate::schematic::schema::Schema;

/// A netlister that tracks how cells and instances are translated between SCIR and the output netlist format.
pub trait ConvertibleNetlister<S: Schema + ?Sized> {
    /// The error type returned when writing out a SCIR netlist.
    type Error: Into<substrate::error::Error>;

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
    ) -> substrate::error::Result<NetlistLibConversion> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Arc::new)?;
        }
        let mut f = std::fs::File::create(path).map_err(Arc::new)?;
        let conv = self
            .write_scir_netlist(lib, &mut f, opts)
            .map_err(|e| e.into())?;
        Ok(conv)
    }

    /// Writes a netlist of a Substrate block to the given output stream.
    fn write_netlist<B: Schematic<S>, W: Write>(
        &self,
        ctx: &Context,
        block: B,
        out: &mut W,
        opts: Self::Options<'_>,
    ) -> substrate::error::Result<(RawLib<S>, NetlistLibConversion)> {
        let raw_lib = ctx.export_scir::<S, _>(block)?;

        let conv = self
            .write_scir_netlist(&raw_lib.scir, out, opts)
            .map_err(|e| e.into())?;
        Ok((raw_lib, conv))
    }

    /// Writes a netlist of a Substrate block to a file at the given path.
    ///
    /// The file and any parent directories will be created if necessary.
    fn write_netlist_to_file<B: Schematic<S>>(
        &self,
        ctx: &Context,
        block: B,
        path: impl AsRef<Path>,
        opts: Self::Options<'_>,
    ) -> substrate::error::Result<(RawLib<S>, NetlistLibConversion)> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Arc::new)?;
        }
        let mut f = std::fs::File::create(path).map_err(Arc::new)?;
        self.write_netlist(ctx, block, &mut f, opts)
    }
}
