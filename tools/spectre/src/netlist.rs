//! Spectre netlist exporter.
#![warn(missing_docs)]

use arcstr::ArcStr;
use opacity::Opacity;
use scir::Slice;
use scir::{BinOp, Cell, Expr, Library, PrimitiveDevice};
use std::io::prelude::*;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, std::io::Error>;

/// A Spectre file include.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Include {
    path: PathBuf,
    section: Option<ArcStr>,
}

impl<T: Into<PathBuf>> From<T> for Include {
    fn from(value: T) -> Self {
        Self {
            path: value.into(),
            section: None,
        }
    }
}

impl Include {
    /// Creates a new [`Include`].
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::from(path)
    }

    /// Returns a new [`Include`] with the given section.
    pub fn section(mut self, section: impl Into<ArcStr>) -> Self {
        self.section = Some(section.into());
        self
    }
}

/// A Spectre netlister.
///
/// The netlister can write to any type that implements [`Write`].
/// Since the netlister may issue many small write calls,
pub struct Netlister<'a, W: Write> {
    lib: &'a Library,
    includes: &'a [Include],
    out: &'a mut W,
}

impl<'a, W: Write> Netlister<'a, W> {
    /// Create a new Spectre netlister writing to the given output stream.
    pub fn new(lib: &'a Library, includes: &'a [Include], out: &'a mut W) -> Self {
        Self { lib, includes, out }
    }

    /// Exports this netlister's library to its output stream.
    #[inline]
    pub fn export(mut self) -> Result<()> {
        self.export_library()?;
        self.out.flush()?;
        Ok(())
    }

    fn export_library(&mut self) -> Result<()> {
        writeln!(self.out, "// {}\n", self.lib.name())?;
        writeln!(self.out, "simulator lang=spectre\n")?;
        writeln!(self.out, "// This is a generated file.")?;
        writeln!(
            self.out,
            "// Be careful when editing manually: this file may be overwritten.\n"
        )?;

        for include in self.includes {
            if let Some(section) = &include.section {
                writeln!(self.out, "include {:?} section={}", include.path, section)?;
            } else {
                writeln!(self.out, "include {:?}", include.path)?;
            }
        }
        writeln!(self.out)?;

        for (id, cell) in self.lib.cells() {
            self.export_cell(cell, self.lib.should_inline(id))?;
        }
        Ok(())
    }

    fn export_cell(&mut self, cell: &Cell, inline: bool) -> Result<()> {
        let indent = if inline { "" } else { "  " };

        let ground = if inline {
            let ground = cell
                .ports()
                .next()
                .expect("testbench should have one port: ground");
            let ground = cell.signal(ground.signal()).name.clone();
            Some(ground)
        } else {
            None
        };
        let ground = ground.as_ref();

        if !inline {
            write!(self.out, "subckt {} (", cell.name())?;
            for port in cell.ports() {
                let sig = cell.signal(port.signal());
                if let Some(width) = sig.width {
                    for i in 0..width {
                        write!(self.out, " {}\\[{}\\]", sig.name, i)?;
                    }
                } else {
                    write!(self.out, " {}", sig.name)?;
                }
            }
            writeln!(self.out, " )\n")?;
        }

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
                        scir::BlackboxElement::Slice(s) => self.write_slice(cell, *s, ground)?,
                    }
                }
                writeln!(self.out)?;
            }
            Opacity::Clear(contents) => {
                for (_id, inst) in contents.instances() {
                    write!(self.out, "{}{} (", indent, inst.name())?;
                    let child = self.lib.cell(inst.cell());
                    for port in child.ports() {
                        let port_name = &child.signal(port.signal()).name;
                        let conn = inst.connection(port_name);
                        for part in conn.parts() {
                            self.write_slice(cell, *part, ground)?;
                        }
                    }
                    writeln!(self.out, " ) {}", child.name())?;
                }

                for (i, device) in contents.primitives().enumerate() {
                    match device {
                        PrimitiveDevice::Res2 { pos, neg, value } => {
                            write!(self.out, "{}res{} (", indent, i)?;
                            self.write_slice(cell, *pos, ground)?;
                            self.write_slice(cell, *neg, ground)?;
                            write!(self.out, " ) resistor r=")?;
                            self.write_expr(value)?;
                        }
                        PrimitiveDevice::RawInstance {
                            ports,
                            cell: child,
                            params,
                        } => {
                            write!(self.out, "{}rawinst{} (", indent, i)?;
                            for port in ports.iter() {
                                self.write_slice(cell, *port, ground)?;
                            }
                            write!(self.out, " ) {}", child)?;
                            for (key, value) in params.iter() {
                                write!(self.out, " {key}=")?;
                                self.write_expr(value)?;
                            }
                        }
                        _ => todo!(),
                    }
                    writeln!(self.out)?;
                }
            }
        };

        if !inline {
            writeln!(self.out, "\nends {}", cell.name())?;
        }
        writeln!(self.out)?;
        Ok(())
    }

    fn write_slice(
        &mut self,
        cell: &Cell,
        slice: Slice,
        rename_ground: Option<&ArcStr>,
    ) -> Result<()> {
        let sig_name = &cell.signal(slice.signal()).name;
        if let Some(range) = slice.range() {
            for i in range.indices() {
                // Ground renaming cannot apply to buses.
                // TODO assert that the ground port has width 1.
                write!(self.out, " {}\\[{}\\]", sig_name, i)?;
            }
        } else {
            let rename = rename_ground.map(|g| sig_name == g).unwrap_or_default();
            if rename {
                write!(self.out, " 0")?;
            } else {
                write!(self.out, " {}", sig_name)?;
            }
        }
        Ok(())
    }

    fn write_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::NumericLiteral(dec) => write!(self.out, "{}", dec)?,
            // boolean literals have no spectre value
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
