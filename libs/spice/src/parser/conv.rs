//! Convert SPICE netlists to other formats.
//!
//! Currently, we only support converting to SCIR.
//!
//! TODO: bus ports, expressions, validation, ArcStr deduplication.

use std::collections::{HashMap, HashSet};

use crate::{Primitive, Spice};
use arcstr::ArcStr;
use regex::Regex;
use rust_decimal::Decimal;
use scir::ParamValue;
use thiserror::Error;
use unicase::UniCase;

use super::{Ast, Component, Elem, Subckt, Substr};

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
    #[error("invalid literal: `{0}`")]
    /// The given expression is not a valid literal.
    InvalidLiteral(Substr),
    /// Attempted to export a blackboxed subcircuit.
    #[error("cannot export a blackboxed subcircuit")]
    ExportBlackbox,
    /// Netlist conversion produced invalid SCIR.
    #[error("netlist conversion produced SCIR containing errors: {0}")]
    InvalidScir(Box<scir::Issues>),
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
        self.map_subckts();
        let subckts = self.subckts.values().copied().collect::<Vec<_>>();
        for subckt in subckts {
            match self.convert_subckt(subckt) {
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

    fn map_subckts(&mut self) {
        for elem in self.ast.elems.iter() {
            match elem {
                Elem::Subckt(s) => {
                    if self.subckts.insert(s.name.clone(), s).is_some() {
                        tracing::warn!(name=%s.name, "Duplicate subcircuits: found two subcircuits with the same name. The last one found will be used.");
                    }
                }
                _ => continue,
            }
        }
    }

    fn convert_subckt(&mut self, subckt: &Subckt) -> ConvResult<scir::CellId> {
        if let Some(&id) = self.ids.get(&subckt.name) {
            return Ok(id);
        }

        if self.blackbox_cells.contains(&subckt.name) {
            return Err(ConvError::ExportBlackbox);
        }

        let mut cell = scir::Cell::new(ArcStr::from(subckt.name.as_str()));
        let mut nodes: HashMap<Substr, scir::SliceOne> = HashMap::new();
        let mut node = |name: &Substr, cell: &mut scir::Cell| {
            if let Some(&node) = nodes.get(name) {
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
                                match str_as_numeric_lit(v) {
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
                Component::Res(res) => {
                    let id = self.lib.add_primitive(Primitive::Res2 {
                        value: str_as_numeric_lit(&res.value)?,
                    });
                    let mut sinst = scir::Instance::new(&res.name[1..], id);
                    sinst.connect("1", node(&res.pos, &mut cell));
                    sinst.connect("2", node(&res.neg, &mut cell));
                    cell.add_instance(sinst);
                }
                Component::Instance(inst) => {
                    let blackbox = self.blackbox_cells.contains(&inst.child);
                    if let (false, Some(subckt)) = (blackbox, self.subckts.get(&inst.child)) {
                        let id = self.convert_subckt(subckt)?;
                        let mut sinst = scir::Instance::new(&inst.name[1..], id);
                        let subckt = self
                            .subckts
                            .get(&inst.child)
                            .ok_or_else(|| ConvError::MissingSubckt(inst.child.clone()))?;

                        for (cport, iport) in subckt.ports.iter().zip(inst.ports.iter()) {
                            sinst.connect(cport.as_str(), node(iport, &mut cell));
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
                                    match str_as_numeric_lit(v) {
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

fn str_as_numeric_lit(s: &Substr) -> ConvResult<Decimal> {
    let re = Regex::new(r"^([0-9]+\.?[0-9]*)(t|g|x|meg|k|m|u|n|p|f?)$").unwrap();
    let caps = re.captures(s).ok_or(ConvError::InvalidLiteral(s.clone()))?;
    let num: Decimal = caps.get(1).unwrap().as_str().parse().unwrap();
    let multiplier = Decimal::from_scientific(
        match caps.get(2).unwrap().as_str().to_lowercase().as_str() {
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
        },
    )
    .unwrap();

    Ok(num * multiplier)
}
