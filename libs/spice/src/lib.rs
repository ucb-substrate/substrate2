//! SPICE netlist exporter.
#![warn(missing_docs)]

use arcstr::ArcStr;
use indexmap::IndexMap;
use scir::netlist::{
    Include, NetlistKind, NetlistLibConversion, NetlistPrimitiveDeviceKind, NetlisterInstance,
    RenameGround, SpiceLikeNetlister,
};
use scir::{Expr, Library, SignalInfo};
use std::io::prelude::*;

pub mod parser;

#[derive(Debug, Clone)]
pub enum Primitive {
    Res2 {
        value: Expr,
    },
    Mos {
        name: ArcStr,
        params: IndexMap<ArcStr, Expr>,
    },
    RawInstance {
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: IndexMap<ArcStr, Expr>,
    },
}

/// A SPICE netlister.
pub struct Netlister<'a, W>(NetlisterInstance<'a, NetlisterImpl, Primitive, W>);

struct NetlisterImpl;

impl SpiceLikeNetlister for NetlisterImpl {
    type Primitive = Primitive;

    fn write_prelude<W: Write>(
        &mut self,
        out: &mut W,
        lib: &Library<Self::Primitive>,
    ) -> std::io::Result<()> {
        writeln!(out, "* {}", lib.name())?;
        writeln!(out, "* This is a generated file. Be careful when editing manually: this file may be overwritten.\n")?;
        Ok(())
    }

    fn write_include<W: Write>(
        &mut self,
        out: &mut W,
        include: &scir::netlist::Include,
    ) -> std::io::Result<()> {
        if let Some(section) = &include.section {
            write!(out, ".LIB {:?} {}", include.path, section)?;
        } else {
            write!(out, ".INCLUDE {:?}", include.path)?;
        }
        Ok(())
    }

    fn write_start_subckt<W: Write>(
        &mut self,
        out: &mut W,
        name: &ArcStr,
        ports: &[&SignalInfo],
    ) -> std::io::Result<()> {
        write!(out, ".SUBCKT {}", name)?;
        for sig in ports {
            if let Some(width) = sig.width {
                for i in 0..width {
                    write!(out, " {}[{}]", sig.name, i)?;
                }
            } else {
                write!(out, " {}", sig.name)?;
            }
        }
        Ok(())
    }

    fn write_end_subckt<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> std::io::Result<()> {
        write!(out, ".ENDS {}", name)
    }

    fn write_instance<W: Write>(
        &mut self,
        out: &mut W,
        name: &ArcStr,
        connections: impl Iterator<Item = ArcStr>,
        child: &ArcStr,
    ) -> std::io::Result<ArcStr> {
        let name = arcstr::format!("X{}", name);
        write!(out, "{}", name)?;

        for connection in connections {
            write!(out, " {}", connection)?;
        }

        write!(out, " {}", child)?;

        Ok(name)
    }

    fn write_primitive<W: Write>(
        &mut self,
        out: &mut W,
        name: &ArcStr,
        kind: NetlistPrimitiveDeviceKind,
    ) -> std::io::Result<ArcStr> {
        Ok(match kind {
            NetlistPrimitiveDeviceKind::Res2 { pos, neg, value } => {
                let name = arcstr::format!("R{}", name);
                write!(out, "{}", name)?;
                for port in [pos, neg] {
                    write!(out, " {}", port)?;
                }
                write!(out, " ")?;
                self.write_expr(out, value)?;
                name
            }
            NetlistPrimitiveDeviceKind::RawInstance { ports, cell } => {
                let name = arcstr::format!("X{}", name);
                write!(out, "{}", name)?;
                for port in ports {
                    write!(out, " {}", port)?;
                }
                write!(out, " {}", cell)?;
                name
            }
            _ => todo!(),
        })
    }
}

impl<'a, W: Write> Netlister<'a, W> {
    /// Create a new SPICE netlister writing to the given output stream.
    pub fn new(lib: &'a Library<Primitive>, includes: &'a [Include], out: &'a mut W) -> Self {
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
