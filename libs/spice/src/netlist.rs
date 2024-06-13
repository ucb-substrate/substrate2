//! Utilities for writing SPICE netlisters for SCIR libraries.

use arcstr::ArcStr;
use itertools::Itertools;
use std::collections::HashMap;

use std::io::{Result, Write};
use std::path::PathBuf;

use crate::{BlackboxElement, Primitive, Spice};
use scir::schema::Schema;
use scir::{
    Cell, ChildId, Library, NetlistCellConversion, NetlistLibConversion, SignalInfo, Slice,
};

use substrate::schematic::netlist::ConvertibleNetlister;

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

/// A schema with a SPICE-like netlist format.
pub trait HasSpiceLikeNetlist: Schema {
    /// Writes a prelude to the beginning of the output stream.
    ///
    /// Should include a newline after if needed.
    #[allow(unused_variables)]
    fn write_prelude<W: Write>(&self, out: &mut W, lib: &Library<Self>) -> Result<()> {
        Ok(())
    }
    /// Writes an include statement.
    ///
    /// A newline will be added afterward.
    fn write_include<W: Write>(&self, out: &mut W, include: &Include) -> Result<()>;
    /// Writes a begin subcircuit statement.
    ///
    /// A newline will be added afterward.
    fn write_start_subckt<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        ports: &[&SignalInfo],
    ) -> Result<()>;
    /// Writes an end subcircuit statement.
    ///
    /// A newline will be added afterward.
    fn write_end_subckt<W: Write>(&self, out: &mut W, name: &ArcStr) -> Result<()>;
    /// Writes a SCIR instance.
    ///
    /// A newline will be added afterward.
    fn write_instance<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        connections: Vec<ArcStr>,
        child: &ArcStr,
    ) -> Result<ArcStr>;
    /// Writes a primitive instantiation.
    ///
    /// A newline will be added afterward.
    fn write_primitive_inst<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        connections: HashMap<ArcStr, Vec<ArcStr>>,
        primitive: &<Self as Schema>::Primitive,
    ) -> Result<ArcStr>;
    /// Writes a slice.
    ///
    /// Should not include a newline at the end.
    fn write_slice<W: Write>(&self, out: &mut W, slice: Slice, info: &SignalInfo) -> Result<()> {
        if let Some(range) = slice.range() {
            for i in range.indices() {
                if i > range.start() {
                    write!(out, " ")?;
                }
                write!(out, "{}[{}]", &info.name, i)?;
            }
        } else {
            write!(out, "{}", &info.name)?;
        }
        Ok(())
    }
    /// Writes a postlude to the end of the output stream.
    #[allow(unused_variables)]
    fn write_postlude<W: Write>(&self, out: &mut W, lib: &Library<Self>) -> Result<()> {
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
#[derive(Clone, Debug, Default)]
#[enumify::enumify(no_as_ref, no_as_mut)]
pub enum NetlistKind {
    /// A netlist that is a collection of cells.
    #[default]
    Cells,
    /// A testbench netlist that should have its top cell inlined and its ground renamed to
    /// the simulator ground node.
    Testbench(RenameGround),
}

/// Configuration for SPICE netlists.
#[derive(Clone, Debug, Default)]
pub struct NetlistOptions<'a> {
    kind: NetlistKind,
    includes: &'a [Include],
}

impl<'a> NetlistOptions<'a> {
    /// Creates a new [`NetlistOptions`].
    pub fn new(kind: NetlistKind, includes: &'a [Include]) -> Self {
        Self { kind, includes }
    }
}

/// An instance of a netlister.
pub struct NetlisterInstance<'a, S: Schema, W> {
    schema: &'a S,
    lib: &'a Library<S>,
    out: &'a mut W,
    opts: NetlistOptions<'a>,
}

impl<'a, S: Schema, W> NetlisterInstance<'a, S, W> {
    /// Creates a new [`NetlisterInstance`].
    pub fn new(
        schema: &'a S,
        lib: &'a Library<S>,
        out: &'a mut W,
        opts: NetlistOptions<'a>,
    ) -> Self {
        Self {
            schema,
            lib,
            out,
            opts,
        }
    }
}

impl<'a, S: HasSpiceLikeNetlist, W: Write> NetlisterInstance<'a, S, W> {
    /// Exports a SCIR library to the output stream as a SPICE-like netlist.
    pub fn export(mut self) -> Result<NetlistLibConversion> {
        let lib = self.export_library()?;
        self.out.flush()?;
        Ok(lib)
    }

    fn export_library(&mut self) -> Result<NetlistLibConversion> {
        self.schema.write_prelude(self.out, self.lib)?;
        for include in self.opts.includes {
            self.schema.write_include(self.out, include)?;
            writeln!(self.out)?;
        }
        writeln!(self.out)?;

        let mut conv = NetlistLibConversion::new();

        for (id, cell) in self.lib.cells() {
            conv.cells
                .insert(id, self.export_cell(cell, self.lib.is_top(id))?);
        }

        self.schema.write_postlude(self.out, self.lib)?;
        Ok(conv)
    }

    fn export_cell(&mut self, cell: &Cell, is_top: bool) -> Result<NetlistCellConversion> {
        let is_testbench_top = is_top && self.opts.kind.is_testbench();

        let indent = if is_testbench_top { "" } else { "  " };

        let ground = match (is_testbench_top, &self.opts.kind) {
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

        if !is_testbench_top {
            let ports: Vec<&SignalInfo> = cell
                .ports()
                .map(|port| cell.signal(port.signal()))
                .collect();
            self.schema
                .write_start_subckt(self.out, cell.name(), &ports)?;
            writeln!(self.out, "\n")?;
        }

        let mut conv = NetlistCellConversion::new();
        for (id, inst) in cell.instances() {
            write!(self.out, "{}", indent)?;
            let mut connections: HashMap<_, _> = inst
                .connections()
                .iter()
                .map(|(k, v)| {
                    Ok((
                        k.clone(),
                        v.parts()
                            .map(|part| self.make_slice(cell, *part, &ground))
                            .collect::<Result<Vec<_>>>()?,
                    ))
                })
                .collect::<Result<_>>()?;
            let name = match inst.child() {
                ChildId::Cell(child_id) => {
                    let child = self.lib.cell(child_id);
                    let ports = child
                        .ports()
                        .flat_map(|port| {
                            let port_name = &child.signal(port.signal()).name;
                            connections.remove(port_name).unwrap()
                        })
                        .collect::<Vec<_>>();
                    self.schema
                        .write_instance(self.out, inst.name(), ports, child.name())?
                }
                ChildId::Primitive(child_id) => {
                    let child = self.lib.primitive(child_id);
                    self.schema
                        .write_primitive_inst(self.out, inst.name(), connections, child)?
                }
            };
            conv.instances.insert(id, name);
            writeln!(self.out)?;
        }

        if !is_testbench_top {
            writeln!(self.out)?;
            self.schema.write_end_subckt(self.out, cell.name())?;
            writeln!(self.out, "\n")?;
        }
        Ok(conv)
    }

    fn make_slice(
        &mut self,
        cell: &Cell,
        slice: Slice,
        rename_ground: &Option<(ArcStr, ArcStr)>,
    ) -> Result<ArcStr> {
        let sig_info = cell.signal(slice.signal());
        if let Some((signal, replace_with)) = rename_ground {
            if signal == &sig_info.name && slice.range().is_none() {
                // Ground renaming cannot apply to buses.
                // TODO assert that the ground port has width 1.
                return Ok(replace_with.clone());
            }
        }
        let mut buf = Vec::new();
        self.schema.write_slice(&mut buf, slice, sig_info)?;
        Ok(ArcStr::from(std::str::from_utf8(&buf).expect(
            "slice should only have UTF8-compatible characters",
        )))
    }
}

impl HasSpiceLikeNetlist for Spice {
    fn write_prelude<W: Write>(&self, out: &mut W, lib: &Library<Self>) -> std::io::Result<()> {
        writeln!(out, "* Substrate SPICE library")?;
        writeln!(out, "* This is a generated file. Be careful when editing manually: this file may be overwritten.\n")?;

        for (_, p) in lib.primitives() {
            if let Primitive::RawInstanceWithCell {
                cell, ports, body, ..
            } = p
            {
                write!(out, ".SUBCKT {}", cell)?;
                for port in ports {
                    write!(out, " {}", port)?;
                }
                writeln!(out)?;
                writeln!(out, "{}", body)?;
                self.write_end_subckt(out, cell)?;
                writeln!(out)?;
            }
        }
        Ok(())
    }

    fn write_include<W: Write>(&self, out: &mut W, include: &Include) -> std::io::Result<()> {
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
        connections: Vec<ArcStr>,
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

    fn write_primitive_inst<W: Write>(
        &self,
        out: &mut W,
        name: &ArcStr,
        mut connections: HashMap<ArcStr, Vec<ArcStr>>,
        primitive: &<Self as Schema>::Primitive,
    ) -> std::io::Result<ArcStr> {
        let name = match &primitive {
            Primitive::Res2 { value, params } => {
                let name = arcstr::format!("R{}", name);
                write!(out, "{}", name)?;
                for port in ["1", "2"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {value}")?;
                for (key, value) in params.iter().sorted_by_key(|(key, _)| *key) {
                    write!(out, " {key}={value}")?;
                }
                name
            }
            Primitive::Cap2 { value } => {
                let name = arcstr::format!("C{}", name);
                write!(out, "{}", name)?;
                for port in ["1", "2"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {value}")?;
                name
            }
            Primitive::Diode2 {
                model: mname,
                params,
            } => {
                let name = arcstr::format!("D{}", name);
                write!(out, "{}", name)?;
                for port in ["1", "2"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {}", mname)?;
                for (key, value) in params.iter().sorted_by_key(|(key, _)| *key) {
                    write!(out, " {key}={value}")?;
                }
                name
            }
            Primitive::Mos {
                model: mname,
                params,
            } => {
                let name = arcstr::format!("M{}", name);
                write!(out, "{}", name)?;
                for port in ["D", "G", "S", "B"] {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {}", mname)?;
                for (key, value) in params.iter().sorted_by_key(|(key, _)| *key) {
                    write!(out, " {key}={value}")?;
                }
                name
            }
            Primitive::RawInstance {
                cell,
                ports,
                params,
            }
            | Primitive::RawInstanceWithCell {
                cell,
                ports,
                params,
                ..
            } => {
                let name = arcstr::format!("X{}", name);
                write!(out, "{}", name)?;
                for port in ports {
                    for part in connections.remove(port).unwrap() {
                        write!(out, " {}", part)?;
                    }
                }
                write!(out, " {}", cell)?;
                for (key, value) in params.iter().sorted_by_key(|(key, _)| *key) {
                    write!(out, " {key}={value}")?;
                }
                name
            }
            Primitive::BlackboxInstance { contents } => {
                // TODO: See if there is a way to translate the name based on the
                // contents, or make documentation explaining that blackbox instances
                // cannot be addressed by path.
                for elem in &contents.elems {
                    match elem {
                        BlackboxElement::InstanceName => write!(out, "{}", name)?,
                        BlackboxElement::RawString(s) => write!(out, "{}", s)?,
                        BlackboxElement::Port(p) => {
                            for part in connections.get(p).unwrap() {
                                write!(out, "{}", part)?
                            }
                        }
                    }
                }
                name.clone()
            }
        };
        writeln!(out)?;
        Ok(name)
    }
}

impl ConvertibleNetlister<Spice> for Spice {
    type Error = std::io::Error;
    type Options<'a> = NetlistOptions<'a>;

    fn write_scir_netlist<W: Write>(
        &self,
        lib: &Library<Spice>,
        out: &mut W,
        opts: Self::Options<'_>,
    ) -> std::result::Result<NetlistLibConversion, Self::Error> {
        NetlisterInstance::new(self, lib, out, opts).export()
    }
}
