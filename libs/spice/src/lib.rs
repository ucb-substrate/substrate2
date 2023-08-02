//! SPICE netlist exporter.
#![warn(missing_docs)]

use opacity::Opacity;
use scir::{BinOp, Cell, Expr, Library};
use scir::{PrimitiveDeviceKind, Slice};
use std::io::{prelude::*, BufWriter};

pub mod parser;

type Result<T> = std::result::Result<T, std::io::Error>;

/// A SPICE netlister.
pub struct Netlister<'a, W: Write> {
    lib: &'a Library,
    out: BufWriter<&'a mut W>,
}

impl<'a, W: Write> Netlister<'a, W> {
    /// Create a new SPICE netlister writing to the given output stream.
    pub fn new(lib: &'a Library, out: &'a mut W) -> Self {
        Self {
            lib,
            out: BufWriter::new(out),
        }
    }

    /// Exports this netlister's library to its output stream.
    #[inline]
    pub fn export(mut self) -> Result<()> {
        self.export_library()?;
        self.out.flush()?;
        Ok(())
    }

    fn export_library(&mut self) -> Result<()> {
        writeln!(self.out, "* {}", self.lib.name())?;
        writeln!(self.out, "* This is a generated file. Be careful when editing manually: this file may be overwritten.\n")?;
        for (_, cell) in self.lib.cells() {
            self.export_cell(cell)?;
        }
        Ok(())
    }

    fn export_cell(&mut self, cell: &Cell) -> Result<()> {
        write!(self.out, ".SUBCKT {}", cell.name())?;
        for port in cell.ports() {
            let sig = cell.signal(port.signal());
            if let Some(width) = sig.width {
                for i in 0..width {
                    write!(self.out, " {}[{}]", sig.name, i)?;
                }
            } else {
                write!(self.out, " {}", sig.name)?;
            }
        }
        writeln!(self.out, "\n")?;

        match cell.contents() {
            Opacity::Opaque(contents) => {
                for (i, elem) in contents.elems.iter().enumerate() {
                    match elem {
                        scir::BlackboxElement::RawString(s) => {
                            if i > 0 {
                                write!(self.out, " {}", s)?
                            } else {
                                write!(self.out, "{}", s)?
                            }
                        }
                        scir::BlackboxElement::Slice(s) => self.write_slice(cell, *s)?,
                    }
                }
                writeln!(self.out)?;
            }
            Opacity::Clear(contents) => {
                for (_id, inst) in contents.instances() {
                    write!(self.out, "X{}", inst.name())?;
                    let child = self.lib.cell(inst.cell());
                    for port in child.ports() {
                        let port_name = &child.signal(port.signal()).name;
                        let conn = inst.connection(port_name);
                        for part in conn.parts() {
                            self.write_slice(cell, *part)?;
                        }
                    }
                    writeln!(self.out, " {}", child.name())?;
                }

                for (i, device) in contents.primitives().enumerate() {
                    match &device.kind {
                        PrimitiveDeviceKind::Res2 { pos, neg, value } => {
                            write!(self.out, "R{}", i)?;
                            self.write_slice(cell, *pos)?;
                            self.write_slice(cell, *neg)?;
                            write!(self.out, " ")?;
                            self.write_expr(value)?;
                            // todo!("parameters");
                        }
                        _ => todo!(),
                    }
                    for (key, value) in device.params.iter() {
                        write!(self.out, " {key}=")?;
                        self.write_expr(value)?;
                    }
                    writeln!(self.out)?;
                }
            }
        };

        writeln!(self.out, "\n.ENDS {}\n", cell.name())?;
        Ok(())
    }

    fn write_slice(&mut self, cell: &Cell, slice: Slice) -> Result<()> {
        let sig_name = &cell.signal(slice.signal()).name;
        if let Some(range) = slice.range() {
            for i in range.indices() {
                write!(self.out, " {}[{}]", sig_name, i)?;
            }
        } else {
            write!(self.out, " {}", sig_name)?;
        }
        Ok(())
    }

    fn write_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::NumericLiteral(dec) => write!(self.out, "{}", dec)?,
            // boolean literals have no spice value
            Expr::BoolLiteral(_) => (),
            Expr::StringLiteral(s) | Expr::Var(s) => write!(self.out, "{}", s)?,
            Expr::BinOp { op, left, right } => {
                write!(self.out, "(")?;
                self.write_expr(left)?;
                write!(self.out, ")")?;
                match op {
                    BinOp::Add => write!(self.out, "+")?,
                    BinOp::Sub => write!(self.out, "-")?,
                    BinOp::Mul => write!(self.out, "*")?,
                    BinOp::Div => write!(self.out, "/")?,
                };
                write!(self.out, "(")?;
                self.write_expr(right)?;
                write!(self.out, ")")?;
                todo!();
            }
        }
        Ok(())
    }
}
