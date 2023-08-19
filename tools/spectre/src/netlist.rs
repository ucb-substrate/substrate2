//! Spectre netlist exporter.
#![warn(missing_docs)]

use crate::SpectrePrimitive;
use arcstr::ArcStr;
use scir::netlist::{
    Include, NetlistKind, NetlistLibConversion, NetlistPrimitiveDeviceKind, NetlisterInstance,
    RenameGround, SpiceLikeNetlister,
};
use scir::Slice;
use scir::{Library, SignalInfo};
use std::io::prelude::*;

type Result<T> = std::result::Result<T, std::io::Error>;

/// A Spectre netlister.
///
/// The netlister can write to any type that implements [`Write`].
/// Since the netlister may issue many small write calls,
pub struct Netlister<'a, W>(NetlisterInstance<'a, NetlisterImpl, SpectrePrimitive, W>);

struct NetlisterImpl;

impl SpiceLikeNetlister for NetlisterImpl {
    type Primitive = SpectrePrimitive;
    fn write_prelude<W: Write>(
        &mut self,
        out: &mut W,
        lib: &Library<SpectrePrimitive>,
    ) -> std::io::Result<()> {
        writeln!(out, "// {}\n", lib.name())?;
        writeln!(out, "simulator lang=spectre\n")?;
        writeln!(out, "// This is a generated file.")?;
        writeln!(
            out,
            "// Be careful when editing manually: this file may be overwritten.\n"
        )?;
        Ok(())
    }

    fn write_include<W: Write>(&mut self, out: &mut W, include: &Include) -> std::io::Result<()> {
        if let Some(section) = &include.section {
            write!(out, "include {:?} section={}", include.path, section)?;
        } else {
            write!(out, "include {:?}", include.path)?;
        }
        Ok(())
    }

    fn write_start_subckt<W: Write>(
        &mut self,
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

    fn write_end_subckt<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> std::io::Result<()> {
        write!(out, "ends {}", name)
    }

    fn write_instance<W: Write>(
        &mut self,
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

    fn write_primitive<W: Write>(
        &mut self,
        out: &mut W,
        name: &ArcStr,
        kind: NetlistPrimitiveDeviceKind,
    ) -> std::io::Result<ArcStr> {
        write!(out, "{} (", name)?;
        match kind {
            NetlistPrimitiveDeviceKind::Res2 { pos, neg, value } => {
                for port in [pos, neg] {
                    write!(out, " {port}")?;
                }
                write!(out, " ) resistor r=")?;
                self.write_expr(out, value)?;
            }
            NetlistPrimitiveDeviceKind::Cap2 { pos, neg, value } => {
                for port in [pos, neg] {
                    write!(out, " {port}")?;
                }
                write!(out, " ) capacitor c=")?;
                self.write_expr(out, value)?;
            }
            NetlistPrimitiveDeviceKind::Res3 {
                pos,
                neg,
                sub,
                value,
                model,
            } => {
                for port in [pos, neg, sub] {
                    write!(out, " {port}")?;
                }
                let model = model.as_ref().map(|s| s.as_str()).unwrap_or("resistor");
                write!(out, " ) {model}")?;
                if let Some(value) = value {
                    write!(out, " r=")?;
                    self.write_expr(out, value)?;
                }
            }
            NetlistPrimitiveDeviceKind::RawInstance { ports, cell } => {
                for port in ports {
                    write!(out, " {}", port)?;
                }
                write!(out, " ) {}", cell)?;
            }
        }
        Ok(name.clone())
    }

    fn write_slice<W: Write>(
        &mut self,
        out: &mut W,
        slice: Slice,
        info: &SignalInfo,
    ) -> Result<()> {
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

impl<'a, W: Write> Netlister<'a, W> {
    /// Create a new Spectre netlister writing to the given output stream.
    pub fn new(
        lib: &'a Library<SpectrePrimitive>,
        includes: &'a [Include],
        out: &'a mut W,
    ) -> Self {
        Self(NetlisterInstance::new(
            NetlisterImpl,
            if lib.is_testbench() {
                NetlistKind::Testbench(RenameGround::Yes(ArcStr::from("0")))
            } else {
                NetlistKind::Cells
            },
            lib,
            includes,
            out,
        ))
    }

    /// Exports the netlister's library to its output stream.
    pub fn export(self) -> std::io::Result<NetlistLibConversion> {
        self.0.export()
    }
}
