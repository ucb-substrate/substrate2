//! Spectre netlist exporter.
#![warn(missing_docs)]

use crate::{Spectre, SpectrePrimitive};
use arcstr::ArcStr;
use scir::netlist::{
    HasSpiceLikeNetlist, Include, NetlistKind, NetlistLibConversion, NetlisterInstance,
    RenameGround,
};
use scir::schema::Schema;
use scir::Slice;
use scir::{Library, SignalInfo};
use std::collections::HashMap;
use std::io::prelude::*;

type Result<T> = std::result::Result<T, std::io::Error>;

impl HasSpiceLikeNetlist for Spectre {
    fn write_prelude<W: Write>(&self, out: &mut W, lib: &Library<Spectre>) -> std::io::Result<()> {
        writeln!(out, "// {}\n", lib.name())?;
        writeln!(out, "simulator lang=spectre\n")?;
        writeln!(out, "// This is a generated file.")?;
        writeln!(
            out,
            "// Be careful when editing manually: this file may be overwritten.\n"
        )?;
        Ok(())
    }

    fn write_include<W: Write>(&self, out: &mut W, include: &Include) -> std::io::Result<()> {
        if let Some(section) = &include.section {
            write!(out, "include {:?} section={}", include.path, section)?;
        } else {
            write!(out, "include {:?}", include.path)?;
        }
        Ok(())
    }

    fn write_start_subckt<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        ports: &[&SignalInfo],
    ) -> std::io::Result<()> {
        write!(out, "subckt {} (", name)?;
        for sig in ports {
            if let Some(width) = sig.width {
                for i in 0..width {
                    write!(out, " {}\\[{}\\]", sig.name, i)?;
                }
            } else {
                write!(out, " {}", sig.name)?;
            }
        }
        write!(out, " )")?;
        Ok(())
    }

    fn write_end_subckt<W: Write>(&self, out: &mut W, name: &ArcStr) -> std::io::Result<()> {
        write!(out, "ends {}", name)
    }

    fn write_instance<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        connections: impl Iterator<Item = ArcStr>,
        child: &ArcStr,
    ) -> std::io::Result<ArcStr> {
        write!(out, "{} (", name)?;

        for connection in connections {
            write!(out, " {}", connection)?;
        }

        write!(out, " ) {}", child)?;

        Ok(name.clone())
    }

    fn write_primitive_inst<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        mut connections: HashMap<ArcStr, impl Iterator<Item = ArcStr>>,
        primitive: &<Self as Schema>::Primitive,
    ) -> std::io::Result<ArcStr> {
        match primitive {
            SpectrePrimitive::RawInstance {
                cell,
                ports,
                params,
            } => {
                let connections = ports
                    .iter()
                    .flat_map(|port| connections.remove(port).unwrap());
                self.write_instance(out, name, connections, cell)?;
                self.write_params(out, params)?;
            }
        }

        Ok(name.clone())
    }

    fn write_slice<W: Write>(&self, out: &mut W, slice: Slice, info: &SignalInfo) -> Result<()> {
        if let Some(range) = slice.range() {
            for i in range.indices() {
                write!(out, "{}\\[{}\\]", &info.name, i)?;
            }
        } else {
            write!(out, "{}", &info.name)?;
        }
        Ok(())
    }
}
