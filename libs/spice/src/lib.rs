//! SPICE netlist exporter.
#![warn(missing_docs)]

use arcstr::ArcStr;
use indexmap::IndexMap;
use scir::netlist::{
    HasSpiceLikeNetlist, Include, NetlistKind, NetlistLibConversion, NetlisterInstance,
    RenameGround,
};
use scir::schema::{FromSchema, NoSchema, NoSchemaError, Schema};
use scir::{Expr, Instance, Library, SignalInfo};
use std::collections::HashMap;
use std::io::prelude::*;

pub mod parser;

/// The SPICE schema.
pub struct Spice;

impl Schema for Spice {
    type Primitive = Primitive;
}

impl FromSchema<NoSchema> for Spice {
    type Error = NoSchemaError;

    fn recover_primitive(
        primitive: <NoSchema as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Err(NoSchemaError)
    }

    fn recover_instance(
        instance: &mut Instance,
        primitive: &<NoSchema as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Err(NoSchemaError)
    }
}

/// SPICE primitives.
#[derive(Debug, Clone)]
pub enum Primitive {
    /// A resistor primitive with ports "1" and "2" and value `value`.
    Res2 { value: Expr },
    /// A MOS primitive with ports "D", "G", "S", and "B" and name `mname`.
    Mos { mname: ArcStr },
    /// A raw instance with associated cell `cell`.
    RawInstance { cell: ArcStr, ports: Vec<ArcStr> },
}

impl HasSpiceLikeNetlist for Spice {
    fn write_prelude<W: Write>(&self, out: &mut W, lib: &Library<Self>) -> std::io::Result<()> {
        writeln!(out, "* {}", lib.name())?;
        writeln!(out, "* This is a generated file. Be careful when editing manually: this file may be overwritten.\n")?;
        Ok(())
    }

    fn write_include<W: Write>(
        &self,
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
        &self,
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

    fn write_end_subckt<W: Write>(&self, out: &mut W, name: &ArcStr) -> std::io::Result<()> {
        write!(out, ".ENDS {}", name)
    }

    fn write_instance<W: Write>(
        &self,
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
        &self,
        out: &mut W,
        name: &ArcStr,
        mut connections: HashMap<ArcStr, impl Iterator<Item = ArcStr>>,
        primitive: &<Self as Schema>::Primitive,
    ) -> std::io::Result<ArcStr> {
        Ok(match primitive {
            Primitive::Res2 { value } => {
                let name = arcstr::format!("R{}", name);
                write!(out, "{}", name)?;
                for port in ["1", "2"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " ")?;
                self.write_expr(out, value)?;
                name
            }
            Primitive::Mos { mname } => {
                let name = arcstr::format!("M{}", name);
                write!(out, "{}", name)?;
                for port in ["D", "G", "S", "B"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {}", mname)?;
                name
            }
            Primitive::RawInstance { cell, ports } => {
                let name = arcstr::format!("X{}", name);
                write!(out, "{}", name)?;
                for port in ports {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {}", cell)?;
                name
            }
        })
    }
}
