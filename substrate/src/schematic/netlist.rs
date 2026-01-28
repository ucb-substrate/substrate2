//! Netlist export.

use std::{io::Write, path::Path, sync::Arc};

use scir::NetlistLibConversion;

use crate::context::Context;

use super::{Schematic, conv::RawLib, schema::Schema};

/// A netlister that tracks how cells and instances are translated between SCIR and the output netlist format.
pub trait ConvertibleNetlister<S: Schema + ?Sized>:
    scir::netlist::ConvertibleNetlister<S, Error: Into<substrate::error::Error>>
{
    /// Writes a netlist of a Substrate block to the given output stream.
    fn write_netlist<B: Schematic<Schema = S>, W: Write>(
        &self,
        ctx: &Context,
        block: B,
        out: &mut W,
        opts: Self::Options<'_>,
    ) -> substrate::error::Result<(RawLib<S>, NetlistLibConversion)> {
        let raw_lib = ctx.export_scir(block)?;

        let conv = self
            .write_scir_netlist(&raw_lib.scir, out, opts)
            .map_err(|e| e.into())?;
        Ok((raw_lib, conv))
    }

    /// Writes a netlist of a Substrate block to a file at the given path.
    ///
    /// The file and any parent directories will be created if necessary.
    fn write_netlist_to_file<B: Schematic<Schema = S>>(
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
