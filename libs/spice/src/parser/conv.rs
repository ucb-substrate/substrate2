//! Convert SPICE netlists to other formats.
//!
//! Currently, we only support converting to SCIR.
//!
//! TODO: bus ports, expressions, validation, ArcStr deduplication.

use std::collections::{HashMap, HashSet};

use crate::parser::shorts::ShortPropagator;
use crate::{ComponentValue, Primitive, Spice};
use arcstr::ArcStr;
use lazy_static::lazy_static;
use num_traits::Pow;
use regex::Regex;
use rust_decimal::prelude::One;
use rust_decimal::Decimal;
use scir::ParamValue;

use thiserror::Error;
use unicase::UniCase;

use super::{Ast, Component, DeviceValue, Elem, Node, Subckt, Substr};

/// The type representing subcircuit names.
pub type SubcktName = Substr;

/// A SPICE netlist conversion result.
pub type ConvResult<T> = std::result::Result<T, ConvError>;

/// A SPICE netlist conversion error.
#[derive(Debug, Error)]
pub enum ConvError {
    /// An instance of this subcircuit exists, but no definition was provided.
    #[error("an instance of subcircuit `{0}` exists, but no definition was provided")]
    MissingSubckt(Substr),
    /// Incorrect (missing/extra) connections for an instance.
    #[error("incorrect (missing/extra) connections for instance {inst} of cell `{child}` (in cell `{parent}`)")]
    IncorrectConnections {
        /// The name of the instance.
        inst: Substr,
        /// The name of the cell being instantiated.
        child: Substr,
        /// The name of the cell containing the offending instance.
        parent: Substr,
    },
    #[error("invalid literal: `{0}`")]
    /// The given expression is not a valid literal.
    InvalidLiteral(Substr),
    /// Attempted to export a blackboxed subcircuit.
    #[error("cannot export a blackboxed subcircuit")]
    ExportBlackbox,
    /// Netlist conversion produced invalid SCIR.
    #[error("netlist conversion produced SCIR containing errors: {0}")]
    InvalidScir(Box<scir::Issues>),
    /// A non-blackbox cell was instantiated with parameters.
    ///
    /// Substrate does not support SPICE-like parameters on non-blackbox cells.
    #[error("parameters for instance {inst} of cell `{child}` (in cell `{parent}`) are not allowed because `{child}` was not blackboxed")]
    UnsupportedParams {
        /// The name of the instance.
        inst: Substr,
        /// The name of the cell being instantiated.
        child: Substr,
        /// The name of the cell containing the offending instance.
        parent: Substr,
    },
}

/// Converts a parsed SPICE netlist to [`scir`].
///
/// The converter only converts subcircuits.
/// Top-level component instantiations are ignored.
pub struct ScirConverter<'a> {
    ast: &'a Ast,
    lib: scir::LibraryBuilder<Spice>,
    blackbox_cells: HashSet<Substr>,
    subckts: HashMap<SubcktName, &'a Subckt>,
    ids: HashMap<SubcktName, scir::CellId>,
}

impl<'a> ScirConverter<'a> {
    /// Create a new SCIR converter.
    pub fn new(ast: &'a Ast) -> Self {
        Self {
            ast,
            lib: scir::LibraryBuilder::new(),
            blackbox_cells: Default::default(),
            subckts: Default::default(),
            ids: Default::default(),
        }
    }

    /// Blackboxes the given cell.
    pub fn blackbox(&mut self, cell_name: impl Into<Substr>) {
        self.blackbox_cells.insert(cell_name.into());
    }

    /// Consumes the converter, yielding a SCIR [library](scir::Library).
    pub fn convert(mut self) -> ConvResult<scir::Library<Spice>> {
        self.subckts = map_subckts(self.ast);
        let subckts = self.subckts.values().copied().collect::<Vec<_>>();
        let mut shorts = ShortPropagator::analyze(self.ast, &self.blackbox_cells);
        for subckt in subckts {
            match self.convert_subckt(subckt, &mut shorts) {
                // Export blackbox errors can be ignored; we just skip
                // exporting a SCIR cell for blackboxed subcircuits.
                Ok(_) | Err(ConvError::ExportBlackbox) => (),
                Err(e) => return Err(e),
            };
        }
        let lib = self
            .lib
            .build()
            .map_err(|issues| ConvError::InvalidScir(Box::new(issues)))?;
        Ok(lib)
    }

    fn convert_subckt(
        &mut self,
        subckt: &Subckt,
        shorts: &mut ShortPropagator,
    ) -> ConvResult<scir::CellId> {
        if let Some(&id) = self.ids.get(&subckt.name) {
            return Ok(id);
        }

        if self.blackbox_cells.contains(&subckt.name) {
            return Err(ConvError::ExportBlackbox);
        }

        let parent_name = subckt.name.clone();

        let mut cell = scir::Cell::new(ArcStr::from(subckt.name.as_str()));
        let mut nodes: HashMap<Substr, scir::SliceOne> = HashMap::new();
        // TODO: this is an expensive clone
        let mut local_shorts = shorts.get_cell(&parent_name).clone();
        let mut node = |name: &Node, cell: &mut scir::Cell| {
            let name = local_shorts.root(name);
            if let Some(&node) = nodes.get(&name) {
                return node;
            }
            let id = cell.add_node(name.as_str());
            nodes.insert(name.clone(), id);
            id
        };

        for component in subckt.components.iter() {
            match component {
                Component::Mos(mos) => {
                    let model = ArcStr::from(mos.model.as_str());
                    let params = mos
                        .params
                        .iter()
                        .map(|(k, v)| {
                            Ok((
                                UniCase::new(ArcStr::from(k.as_str())),
                                match substr_as_numeric_lit(v) {
                                    Ok(v) => ParamValue::Numeric(v),
                                    Err(_) => ParamValue::String(v.to_string().into()),
                                },
                            ))
                        })
                        .collect::<ConvResult<HashMap<_, _>>>()?;
                    // TODO: Deduplicate primitives, though does not affect functionality
                    let id = self.lib.add_primitive(Primitive::Mos { model, params });
                    let mut sinst = scir::Instance::new(&mos.name[1..], id);
                    sinst.connect("D", node(&mos.d, &mut cell));
                    sinst.connect("G", node(&mos.g, &mut cell));
                    sinst.connect("S", node(&mos.s, &mut cell));
                    sinst.connect("B", node(&mos.b, &mut cell));
                    cell.add_instance(sinst);
                }
                Component::Diode(diode) => {
                    let model = ArcStr::from(diode.model.as_str());
                    let params = diode
                        .params
                        .iter()
                        .map(|(k, v)| {
                            Ok((
                                UniCase::new(ArcStr::from(k.as_str())),
                                match substr_as_numeric_lit(v) {
                                    Ok(v) => ParamValue::Numeric(v),
                                    Err(_) => ParamValue::String(v.to_string().into()),
                                },
                            ))
                        })
                        .collect::<ConvResult<HashMap<_, _>>>()?;
                    // TODO: Deduplicate primitives, though does not affect functionality
                    let id = self.lib.add_primitive(Primitive::Diode2 { model, params });
                    let mut sinst = scir::Instance::new(&diode.name[1..], id);
                    sinst.connect("1", node(&diode.pos, &mut cell));
                    sinst.connect("2", node(&diode.neg, &mut cell));
                    cell.add_instance(sinst);
                }
                Component::Res(res) => {
                    let value = match &res.value {
                        DeviceValue::Value(value) => {
                            ComponentValue::Fixed(substr_as_numeric_lit(value)?)
                        }
                        DeviceValue::Model(model) => {
                            ComponentValue::Model(ArcStr::from(model.as_str()))
                        }
                    };
                    let params = res
                        .params
                        .iter()
                        .map(|(k, v)| {
                            Ok((
                                UniCase::new(ArcStr::from(k.as_str())),
                                match substr_as_numeric_lit(v) {
                                    Ok(v) => ParamValue::Numeric(v),
                                    Err(_) => ParamValue::String(v.to_string().into()),
                                },
                            ))
                        })
                        .collect::<ConvResult<HashMap<_, _>>>()?;
                    let id = self.lib.add_primitive(Primitive::Res2 { value, params });
                    let mut sinst = scir::Instance::new(&res.name[1..], id);
                    sinst.connect("1", node(&res.pos, &mut cell));
                    sinst.connect("2", node(&res.neg, &mut cell));
                    cell.add_instance(sinst);
                }
                Component::Cap(cap) => {
                    let id = self.lib.add_primitive(Primitive::Cap2 {
                        value: substr_as_numeric_lit(&cap.value)?,
                    });
                    let mut sinst = scir::Instance::new(&cap.name[1..], id);
                    sinst.connect("1", node(&cap.pos, &mut cell));
                    sinst.connect("2", node(&cap.neg, &mut cell));
                    cell.add_instance(sinst);
                }
                Component::Instance(inst) => {
                    let blackbox = self.blackbox_cells.contains(&inst.child);
                    if let (false, Some(subckt)) = (blackbox, self.subckts.get(&inst.child)) {
                        // Parameters are not supported for instances of non-blackboxed subcircuit.
                        if !inst.params.values.is_empty() {
                            return Err(ConvError::UnsupportedParams {
                                inst: inst.name.clone(),
                                child: subckt.name.clone(),
                                parent: parent_name.clone(),
                            });
                        }

                        let id = self.convert_subckt(subckt, shorts)?;
                        let mut sinst = scir::Instance::new(&inst.name[1..], id);
                        let subckt = self
                            .subckts
                            .get(&inst.child)
                            .ok_or_else(|| ConvError::MissingSubckt(inst.child.clone()))?;

                        if subckt.ports.len() != inst.ports.len() {
                            return Err(ConvError::IncorrectConnections {
                                inst: inst.name.clone(),
                                child: subckt.name.clone(),
                                parent: parent_name.clone(),
                            });
                        }

                        let cshorts = shorts.get_cell(&inst.child);
                        for (cport, iport) in subckt.ports.iter().zip(inst.ports.iter()) {
                            // If child port is not its own root, do not connect to it: it must be shorted to another port
                            if cshorts.root(cport) == *cport {
                                sinst.connect(cport.as_str(), node(iport, &mut cell));
                            }
                        }

                        if !inst.params.values.is_empty() {
                            return Err(ConvError::IncorrectConnections {
                                inst: inst.name.clone(),
                                child: subckt.name.clone(),
                                parent: parent_name.clone(),
                            });
                        }

                        cell.add_instance(sinst);
                    } else {
                        let child = ArcStr::from(inst.child.as_str());
                        let params = inst
                            .params
                            .iter()
                            .map(|(k, v)| {
                                Ok((
                                    UniCase::new(ArcStr::from(k.as_str())),
                                    match substr_as_numeric_lit(v) {
                                        Ok(v) => ParamValue::Numeric(v),
                                        Err(_) => ParamValue::String(v.to_string().into()),
                                    },
                                ))
                            })
                            .collect::<ConvResult<HashMap<_, _>>>()?;
                        let ports: Vec<_> = (0..inst.ports.len())
                            .map(|i| arcstr::format!("{}", i + 1))
                            .collect();
                        let id = self.lib.add_primitive(Primitive::RawInstance {
                            cell: child,
                            ports: ports.clone(),
                            params,
                        });
                        let mut sinst = scir::Instance::new(&inst.name[1..], id);
                        for (cport, iport) in ports.iter().zip(inst.ports.iter()) {
                            sinst.connect(cport, node(iport, &mut cell));
                        }
                        cell.add_instance(sinst);
                    }
                }
            };
        }

        for port in subckt.ports.iter() {
            let port = node(port, &mut cell);
            // In the future, we may support parsing port directions from comments in the SPICE file.
            // For now, we simply expose all ports using the default direction.
            cell.expose_port(port, Default::default());
        }

        let id = self.lib.add_cell(cell);
        self.ids.insert(subckt.name.clone(), id);
        Ok(id)
    }
}

lazy_static! {
    static ref NUMERIC_LITERAL_REGEX: Regex =
        Regex::new(r"^(-?[0-9]+\.?[0-9]*)((t|g|x|meg|k|m|u|n|p|f)|(e(-?[0-9]+\.?[0-9]*)))?$")
            .expect("failed to compile numeric literal regex");
}

pub(crate) fn convert_str_to_numeric_lit(s: &str) -> Option<Decimal> {
    let caps = NUMERIC_LITERAL_REGEX.captures(s)?;
    let num: Decimal = caps.get(1)?.as_str().parse().ok()?;
    let multiplier = caps
        .get(3)
        .and_then(|s| {
            Decimal::from_scientific(match s.as_str().to_lowercase().as_str() {
                "t" => "1e12",
                "g" => "1e9",
                "x" | "meg" => "1e6",
                "k" => "1e3",
                "m" => "1e-3",
                "u" => "1e-6",
                "n" => "1e-9",
                "p" => "1e-12",
                "f" => "1e-15",
                _ => "1e0",
            })
            .ok()
        })
        .or_else(|| {
            caps.get(5)
                .and_then(|s| s.as_str().parse().ok())
                .map(|exp: Decimal| Decimal::TEN.pow(exp))
        })
        .unwrap_or_else(Decimal::one);

    Some(num * multiplier)
}

pub(crate) fn str_as_numeric_lit(s: &str) -> std::result::Result<Decimal, ()> {
    convert_str_to_numeric_lit(s).ok_or(())
}
fn substr_as_numeric_lit(s: &Substr) -> ConvResult<Decimal> {
    str_as_numeric_lit(s).map_err(|_| ConvError::InvalidLiteral(s.clone()))
}

pub(crate) fn map_subckts(ast: &Ast) -> HashMap<SubcktName, &Subckt> {
    let mut subckts = HashMap::new();
    for elem in ast.elems.iter() {
        match elem {
            Elem::Subckt(s) => {
                if subckts.insert(s.name.clone(), s).is_some() {
                    tracing::warn!(name=%s.name, "Duplicate subcircuits: found two subcircuits with the same name. The last one found will be used.");
                }
            }
            _ => continue,
        }
    }
    subckts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_literal_regex() {
        assert!(str_as_numeric_lit("2").is_ok());
        assert!(str_as_numeric_lit("1.23").is_ok());
        assert!(str_as_numeric_lit("-2").is_ok());
        assert!(str_as_numeric_lit("-1.23").is_ok());
        assert!(str_as_numeric_lit("0.0175668f").is_ok());
        assert!(str_as_numeric_lit("8.88268e-19").is_ok());
        assert!(str_as_numeric_lit("-0.0175668f").is_ok());
        assert!(str_as_numeric_lit("-8.88268e-19").is_ok());
        assert!(str_as_numeric_lit("8.88268e19").is_ok());
        assert!(str_as_numeric_lit("-8.88268e19").is_ok());
    }
}
