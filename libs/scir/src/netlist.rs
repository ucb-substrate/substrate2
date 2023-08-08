//! Utilities for writing netlisters for SCIR libraries.

use crate::{
    BinOp, BlackboxElement, Cell, CellId, Expr, InstanceId, Library, PrimitiveDeviceId,
    PrimitiveDeviceKind, SignalInfo, Slice,
};
use arcstr::ArcStr;
use opacity::Opacity;
use std::collections::HashMap;
use std::io::{Result, Write};
use std::path::PathBuf;

/// A netlist include statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Include {
    /// The path to include.
    pub path: PathBuf,
    /// The section of the provided file to include.
    pub section: Option<ArcStr>,
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

/// Metadata associated with the conversion from a SCIR library to a netlist.
#[derive(Debug, Clone, Default)]
pub struct NetlistLibConversion {
    /// Conversion metadata for each cell in the SCIR library.
    pub cells: HashMap<CellId, NetlistCellConversion>,
}

impl NetlistLibConversion {
    /// Creates a new [`NetlistLibConversion`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata associated with the conversion from a SCIR cell to a netlisted subcircuit.
#[derive(Debug, Clone, Default)]
pub struct NetlistCellConversion {
    /// The netlisted names of SCIR instances.
    pub instances: HashMap<InstanceId, ArcStr>,
    /// The netlisted names of SCIR primitives.
    pub primitives: HashMap<PrimitiveDeviceId, ArcStr>,
}

impl NetlistCellConversion {
    /// Creates a new [`NetlistCellConversion`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// A SPICE-like netlister.
///
/// Appropriate newlines will be added after each function call, so newlines added by
/// implementors may cause formatting issues.
pub trait SpiceLikeNetlister {
    /// Writes a prelude to the beginning of the output stream.
    #[allow(unused_variables)]
    fn write_prelude<W: Write>(&mut self, out: &mut W, lib: &Library) -> Result<()> {
        Ok(())
    }
    /// Writes an include statement.
    fn write_include<W: Write>(&mut self, out: &mut W, include: &Include) -> Result<()>;
    /// Writes a begin subcircuit statement.
    fn write_start_subckt<W: Write>(
        &mut self,
        out: &mut W,
        name: &ArcStr,
        ports: &[&SignalInfo],
    ) -> Result<()>;
    /// Writes an end subcircuit statement.
    fn write_end_subckt<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> Result<()>;
    /// Writes the portion of an instance declaration before the connections.
    fn write_start_instance<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> Result<ArcStr>;
    /// Writes the portion of an instance declaration after the connections.
    fn write_end_instance<W: Write>(&mut self, out: &mut W, child: &ArcStr) -> Result<()>;
    /// Writes the portion of a two-port resistor primitive declaration before the connections.
    fn write_start_res2<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> Result<ArcStr>;
    /// Writes the portion of a two-port resistor primitive declaration after the connections.
    fn write_end_res2<W: Write>(&mut self, out: &mut W, value: &Expr) -> Result<()>;
    /// Writes the portion of a raw instance declaration before the connections.
    fn write_start_raw_instance<W: Write>(&mut self, out: &mut W, name: &ArcStr) -> Result<ArcStr>;
    /// Writes the portion of a raw instance declaration after the connections.
    fn write_end_raw_instance<W: Write>(&mut self, out: &mut W, child: &ArcStr) -> Result<()>;
    /// Writes the parameters of a primitive device immediately following the written ending.
    fn write_params<W: Write>(
        &mut self,
        out: &mut W,
        params: &HashMap<ArcStr, Expr>,
    ) -> Result<()> {
        for (key, value) in params.iter() {
            write!(out, " {key}=")?;
            self.write_expr(out, value)?;
        }
        Ok(())
    }
    /// Writes a slice.
    fn write_slice<W: Write>(
        &mut self,
        out: &mut W,
        slice: Slice,
        info: &SignalInfo,
    ) -> Result<()> {
        if let Some(range) = slice.range() {
            for i in range.indices() {
                write!(out, "{}[{}]", &info.name, i)?;
            }
        } else {
            write!(out, "{}", &info.name)?;
        }
        Ok(())
    }
    /// Writes a SCIR expression.
    fn write_expr<W: Write>(&mut self, out: &mut W, expr: &Expr) -> Result<()> {
        match expr {
            Expr::NumericLiteral(dec) => write!(out, "{}", dec)?,
            // boolean literals have no spectre value
            Expr::BoolLiteral(_) => (),
            Expr::StringLiteral(s) | Expr::Var(s) => write!(out, "{}", s)?,
            Expr::BinOp { op, left, right } => {
                write!(out, "(")?;
                self.write_expr(out, left)?;
                write!(out, ")")?;
                match op {
                    BinOp::Add => write!(out, "+")?,
                    BinOp::Sub => write!(out, "-")?,
                    BinOp::Mul => write!(out, "*")?,
                    BinOp::Div => write!(out, "/")?,
                };
                write!(out, "(")?;
                self.write_expr(out, right)?;
                write!(out, ")")?;
                todo!();
            }
        }
        Ok(())
    }
    /// Writes a postlude to the end of the output stream.
    #[allow(unused_variables)]
    fn write_postlude<W: Write>(&mut self, out: &mut W, lib: &Library) -> Result<()> {
        Ok(())
    }
}

/// An enumeration describing whether the ground node of a testbench should be renamed.
#[derive(Clone, Debug)]
pub enum RenameGround {
    /// The ground node should be renamed to the provided [`ArcStr`].
    Yes(ArcStr),
    /// The ground node should not be renamed.
    No,
}

/// The type of netlist to be exported.
#[derive(Clone, Debug)]
pub enum NetlistKind {
    /// A testbench netlist that should have its top cell inlined and its ground renamed to
    /// the simulator ground node.
    Testbench(RenameGround),
    /// A netlist that is a collection of cells.
    Cells,
}

/// An instance of a netlister.
pub struct NetlisterInstance<'a, N, W> {
    netlister: N,
    kind: NetlistKind,
    lib: &'a Library,
    includes: &'a [Include],
    out: &'a mut W,
}

impl<'a, N, W> NetlisterInstance<'a, N, W> {
    /// Creates a new [`NetlisterInstance`].
    pub fn new(
        netlister: N,
        kind: NetlistKind,
        lib: &'a Library,
        includes: &'a [Include],
        out: &'a mut W,
    ) -> Self {
        Self {
            netlister,
            kind,
            lib,
            includes,
            out,
        }
    }
}

impl<'a, N: SpiceLikeNetlister, W: Write> NetlisterInstance<'a, N, W> {
    /// Exports a SCIR library to the output stream using a [`SpiceLikeNetlister`].
    pub fn export(mut self) -> Result<NetlistLibConversion> {
        let lib = self.export_library()?;
        self.out.flush()?;
        Ok(lib)
    }

    fn export_library(&mut self) -> Result<NetlistLibConversion> {
        self.netlister.write_prelude(self.out, self.lib)?;
        for include in self.includes {
            self.netlister.write_include(self.out, include)?;
            writeln!(self.out)?;
        }
        writeln!(self.out)?;

        let mut conv = NetlistLibConversion::new();

        for (id, cell) in self.lib.cells() {
            conv.cells
                .insert(id, self.export_cell(cell, self.lib.should_inline(id))?);
        }

        self.netlister.write_postlude(self.out, self.lib)?;
        Ok(conv)
    }

    fn export_cell(&mut self, cell: &Cell, inline: bool) -> Result<NetlistCellConversion> {
        let indent = if inline { "" } else { "  " };

        let ground = match (inline, &self.kind) {
            (true, NetlistKind::Testbench(RenameGround::Yes(replace_with))) => {
                let msg = "testbench should have one port: ground";
                let mut ports = cell.ports();
                let ground = ports.next().expect(msg);
                assert!(ports.next().is_none(), "{}", msg);
                let ground = &cell.signal(ground.signal()).name;
                Some((ground.clone(), replace_with.clone()))
            }
            _ => None,
        };

        if !inline {
            let ports: Vec<&SignalInfo> = cell
                .ports()
                .map(|port| cell.signal(port.signal()))
                .collect();
            self.netlister
                .write_start_subckt(self.out, cell.name(), &ports)?;
            writeln!(self.out, "\n")?;
        }

        let mut conv = NetlistCellConversion::new();
        match cell.contents() {
            Opacity::Opaque(contents) => {
                for (i, elem) in contents.elems.iter().enumerate() {
                    match elem {
                        BlackboxElement::RawString(s) => {
                            if i > 0 {
                                write!(self.out, " {}", s)?
                            } else {
                                write!(self.out, "{}", s)?
                            }
                        }
                        BlackboxElement::Slice(s) => self.write_slice(cell, *s, &ground)?,
                    }
                }
                writeln!(self.out)?;
            }
            Opacity::Clear(contents) => {
                for (id, inst) in contents.instances() {
                    let child = self.lib.cell(inst.cell());
                    write!(self.out, "{}", indent)?;
                    let name = self.netlister.write_start_instance(self.out, inst.name())?;
                    conv.instances.insert(id, name);
                    for port in child.ports() {
                        let port_name = &child.signal(port.signal()).name;
                        let conn = inst.connection(port_name);
                        for part in conn.parts() {
                            self.write_slice(cell, *part, &ground)?;
                        }
                    }
                    self.netlister.write_end_instance(self.out, child.name())?;
                    writeln!(self.out)?;
                }

                for (id, device) in contents.primitives() {
                    write!(self.out, "{}", indent)?;
                    match &device.kind {
                        PrimitiveDeviceKind::Res2 { pos, neg, value } => {
                            let name = self.netlister.write_start_res2(self.out, &device.name)?;
                            conv.primitives.insert(id, name);
                            self.write_slice(cell, *pos, &ground)?;
                            self.write_slice(cell, *neg, &ground)?;
                            self.netlister.write_end_res2(self.out, value)?;
                        }
                        PrimitiveDeviceKind::RawInstance { ports, cell: child } => {
                            let name = self
                                .netlister
                                .write_start_raw_instance(self.out, &device.name)?;
                            conv.primitives.insert(id, name);
                            for port in ports.iter() {
                                self.write_slice(cell, *port, &ground)?;
                            }
                            self.netlister.write_end_raw_instance(self.out, child)?;
                        }
                        _ => todo!(),
                    }
                    self.netlister.write_params(self.out, &device.params)?;
                    writeln!(self.out)?;
                }
            }
        };

        if !inline {
            writeln!(self.out)?;
            self.netlister.write_end_subckt(self.out, cell.name())?;
            writeln!(self.out, "\n")?;
        }
        Ok(conv)
    }

    fn write_slice(
        &mut self,
        cell: &Cell,
        slice: Slice,
        rename_ground: &Option<(ArcStr, ArcStr)>,
    ) -> Result<()> {
        write!(self.out, " ")?;
        let sig_info = cell.signal(slice.signal());
        if let Some((signal, replace_with)) = rename_ground {
            if signal == &sig_info.name && slice.range().is_none() {
                // Ground renaming cannot apply to buses.
                // TODO assert that the ground port has width 1.
                write!(self.out, "{}", replace_with)?;
                return Ok(());
            }
        }
        self.netlister.write_slice(self.out, slice, sig_info)?;
        Ok(())
    }
}
