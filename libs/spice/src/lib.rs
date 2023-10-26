//! SPICE netlist exporter.
#![warn(missing_docs)]

use arcstr::ArcStr;
use itertools::Itertools;
use rust_decimal::Decimal;
use scir::netlist::HasSpiceLikeNetlist;
use scir::schema::{FromSchema, NoSchema, NoSchemaError, Schema};
use scir::{Instance, Library, ParamValue, SignalInfo};
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

    fn convert_primitive(
        _primitive: <NoSchema as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Err(NoSchemaError)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<NoSchema as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Err(NoSchemaError)
    }
}

/// SPICE primitives.
#[derive(Debug, Clone)]
pub struct Primitive {
    /// The kind of primitive.
    pub kind: PrimitiveKind,
    /// Parameters associated with the primitive.
    pub params: HashMap<ArcStr, ParamValue>,
}

/// An enumeration of SPICE primitive kinds.
#[derive(Debug, Clone)]
pub enum PrimitiveKind {
    /// A resistor primitive with ports "1" and "2" and value `value`.
    Res2 {
        /// The resistor value.
        value: Decimal,
    },
    /// A MOS primitive with ports "D", "G", "S", and "B".
    Mos {
        /// The name of the MOS model.
        mname: ArcStr,
    },
    /// A raw instance with an associated cell.
    RawInstance {
        /// The associated cell.
        cell: ArcStr,
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
    },
    /// An external module with blackboxed contents.
    ExternalModule {
        /// The cell name.
        cell: ArcStr,
        /// The cell ports.
        ports: Vec<ArcStr>,
        /// The contents of the cell.
        contents: ArcStr,
    },
}

impl PrimitiveKind {
    pub fn ports(&self) -> Vec<ArcStr> {
        match self {
            PrimitiveKind::Res2 { .. } => vec!["1".into(), "2".into()],
            PrimitiveKind::Mos { .. } => vec!["D".into(), "G".into(), "S".into(), "B".into()],
            PrimitiveKind::RawInstance { ports, .. }
            | PrimitiveKind::ExternalModule { ports, .. } => ports.clone(),
        }
    }
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

    fn write_primitive_subckt<W: Write>(
        &self,
        out: &mut W,
        primitive: &<Self as Schema>::Primitive,
    ) -> std::io::Result<()> {
        if let PrimitiveKind::ExternalModule {
            cell,
            ports,
            contents,
        } = &primitive.kind
        {
            write!(out, ".SUBCKT {}", cell)?;
            for port in ports {
                write!(out, " {}", port)?;
            }
            writeln!(out, "\n")?;

            writeln!(out, "{}", contents)?;

            self.write_end_subckt(out, cell)?;
            writeln!(out)?;
        };
        Ok(())
    }

    fn write_primitive_inst<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        mut connections: HashMap<ArcStr, impl Iterator<Item = ArcStr>>,
        primitive: &<Self as Schema>::Primitive,
    ) -> std::io::Result<ArcStr> {
        let name = match &primitive.kind {
            PrimitiveKind::Res2 { value } => {
                let name = arcstr::format!("R{}", name);
                write!(out, "{}", name)?;
                for port in ["1", "2"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {value}")?;
                name
            }
            PrimitiveKind::Mos { mname } => {
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
            PrimitiveKind::RawInstance { cell, ports }
            | PrimitiveKind::ExternalModule { cell, ports, .. } => {
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
        };
        for (key, value) in primitive.params.iter().sorted_by_key(|(key, _)| *key) {
            write!(out, " {key}={value}")?;
        }
        Ok(name)
    }
}
