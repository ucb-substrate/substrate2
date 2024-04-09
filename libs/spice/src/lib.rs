//! SPICE netlist exporter.
#![warn(missing_docs)]

use crate::parser::conv::ScirConverter;
use crate::parser::{ParsedSpice, Parser};

use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::schema::{FromSchema, NoSchema, NoSchemaError, Schema};
use scir::{Instance, Library, NetlistLibConversion, ParamValue, SliceOnePath};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::schematic::primitives::Resistor;
use substrate::schematic::{CellBuilder, Schematic};
use unicase::UniCase;

pub mod netlist;
pub mod parser;
#[cfg(test)]
mod tests;

/// The SPICE schema.
pub struct Spice;

impl Spice {
    /// Converts [`ParsedSpice`] to a [`Library`].
    pub fn scir_lib_from_parsed(parsed: &ParsedSpice) -> Library<Spice> {
        let conv = ScirConverter::new(&parsed.ast);
        conv.convert().unwrap()
    }

    /// Converts a SPICE string to a [`Library`].
    pub fn scir_lib_from_str(source: &str) -> Library<Spice> {
        let parsed = Parser::parse(source).unwrap();
        Spice::scir_lib_from_parsed(&parsed)
    }

    /// Converts a SPICE file to a [`Library`].
    pub fn scir_lib_from_file(path: impl AsRef<Path>) -> Library<Spice> {
        let parsed = Parser::parse_file(path).unwrap();
        Spice::scir_lib_from_parsed(&parsed)
    }

    /// Converts [`ParsedSpice`] to an unconnected [`ScirBinding`](substrate::schematic::ScirBinding)
    /// associated with the cell named `cell_name`.
    pub fn scir_cell_from_parsed(
        parsed: &ParsedSpice,
        cell_name: &str,
    ) -> substrate::schematic::ScirBinding<Spice> {
        let lib = Spice::scir_lib_from_parsed(parsed);
        let cell_id = lib.cell_id_named(cell_name);
        substrate::schematic::ScirBinding::new(lib, cell_id)
    }

    /// Converts a SPICE string to an unconnected [`ScirBinding`](substrate::schematic::ScirBinding)
    /// associated with the cell named `cell_name`.
    pub fn scir_cell_from_str(
        source: &str,
        cell_name: &str,
    ) -> substrate::schematic::ScirBinding<Spice> {
        let parsed = Parser::parse(source).unwrap();
        Spice::scir_cell_from_parsed(&parsed, cell_name)
    }

    /// Converts a SPICE file to an unconnected [`ScirBinding`](substrate::schematic::ScirBinding)
    /// associated with the cell named `cell_name`.
    pub fn scir_cell_from_file(
        path: impl AsRef<Path>,
        cell_name: &str,
    ) -> substrate::schematic::ScirBinding<Spice> {
        let parsed = Parser::parse_file(path).unwrap();
        Spice::scir_cell_from_parsed(&parsed, cell_name)
    }

    /// Converts a [`SliceOnePath`] to a Spectre path string corresponding to the associated
    /// node voltage.
    pub fn node_voltage_path(
        lib: &Library<Spice>,
        conv: &NetlistLibConversion,
        path: &SliceOnePath,
    ) -> String {
        lib.convert_slice_one_path_with_conv(conv, path.clone(), |name, index| {
            if let Some(index) = index {
                arcstr::format!("{}\\[{}\\]", name, index)
            } else {
                name.into()
            }
        })
        .join(".")
    }
}

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

/// A SPICE primitive.
#[derive(Debug, Clone)]
pub enum Primitive {
    /// A resistor primitive with ports "1" and "2" and value `value`.
    Res2 {
        /// The resistor value.
        value: Decimal,
    },
    /// A capacitor primitive with ports "1" and "2" and value `value`.
    Cap2 {
        /// The capacitor value.
        value: Decimal,
    },
    /// A diode primitive with ports "1" and "2".
    Diode2 {
        /// The name of the diode model.
        model: ArcStr,
        /// Parameters associated with the MOS primitive.
        params: HashMap<UniCase<ArcStr>, ParamValue>,
    },
    /// A MOS primitive with ports "D", "G", "S", and "B".
    Mos {
        /// The name of the MOS model.
        model: ArcStr,
        /// Parameters associated with the MOS primitive.
        params: HashMap<UniCase<ArcStr>, ParamValue>,
    },
    /// A raw instance with an associated cell.
    RawInstance {
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
        /// The associated cell.
        cell: ArcStr,
        /// Parameters associated with the raw instance.
        params: HashMap<UniCase<ArcStr>, ParamValue>,
    },
    /// A raw instance with an associated cell.
    ///
    /// Creates the corresponding SUBCKT with the given body.
    RawInstanceWithCell {
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
        /// The associated cell.
        cell: ArcStr,
        /// Parameters associated with the raw instance.
        params: HashMap<UniCase<ArcStr>, ParamValue>,
        /// The body of the associated cell.
        body: ArcStr,
    },
    /// An instance with blackboxed contents.
    BlackboxInstance {
        /// The contents of the cell.
        contents: BlackboxContents,
    },
}

/// Contents of a blackboxed instance.
#[derive(Debug, Clone)]
pub struct BlackboxContents {
    /// The elements that make up this blackbox.
    pub elems: Vec<BlackboxElement>,
}

impl BlackboxContents {
    /// Pushes an new element to the blackbox.
    pub fn push(&mut self, elem: impl Into<BlackboxElement>) {
        self.elems.push(elem.into());
    }
}

/// An element of a blackbox instance.
#[derive(Debug, Clone)]
pub enum BlackboxElement {
    /// A placeholder for the instance's name.
    InstanceName,
    /// A raw blackbox string.
    RawString(ArcStr),
    /// A port of the SCIR instantiation of this blackbox.
    Port(ArcStr),
}

impl FromIterator<BlackboxElement> for BlackboxContents {
    fn from_iter<T: IntoIterator<Item = BlackboxElement>>(iter: T) -> Self {
        Self {
            elems: iter.into_iter().collect(),
        }
    }
}

impl From<BlackboxElement> for BlackboxContents {
    fn from(value: BlackboxElement) -> Self {
        Self { elems: vec![value] }
    }
}

impl<T: Into<ArcStr>> From<T> for BlackboxContents {
    fn from(value: T) -> Self {
        Self {
            elems: vec![BlackboxElement::RawString(value.into())],
        }
    }
}

impl<T: Into<ArcStr>> From<T> for BlackboxElement {
    fn from(value: T) -> Self {
        Self::RawString(value.into())
    }
}

impl Primitive {
    /// Returns the ports for a given [`Primitive`].
    pub fn ports(&self) -> Vec<ArcStr> {
        match self {
            Primitive::Res2 { .. } => vec!["1".into(), "2".into()],
            Primitive::Cap2 { .. } => vec!["1".into(), "2".into()],
            Primitive::Diode2 { .. } => vec!["1".into(), "2".into()],
            Primitive::Mos { .. } => vec!["D".into(), "G".into(), "S".into(), "B".into()],
            Primitive::RawInstance { ports, .. } => ports.clone(),
            Primitive::RawInstanceWithCell { ports, .. } => ports.clone(),
            Primitive::BlackboxInstance { contents } => contents
                .elems
                .iter()
                .filter_map(|x| {
                    if let BlackboxElement::Port(p) = x {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
        }
    }
}

impl Schematic<Spice> for Resistor {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spice>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = substrate::schematic::PrimitiveBinding::new(Primitive::Res2 {
            value: self.value(),
        });
        prim.connect("1", io.p);
        prim.connect("2", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}
