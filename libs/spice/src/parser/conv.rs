//! Convert SPICE netlists to other formats.
//!
//! Currently, we only support converting to SCIR.
//!
//! TODO: bus ports, expressions, validation, ArcStr deduplication.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use thiserror::Error;

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
}

/// Converts a parsed SPICE netlist to [`scir`].
///
/// The converter only converts subcircuits.
/// Top-level component instantiations are ignored.
pub struct ScirConverter<'a> {
    ast: &'a Ast,
    lib: scir::Library,
    blackbox_cells: HashSet<Substr>,
    subckts: HashMap<SubcktName, &'a Subckt>,
    ids: HashMap<SubcktName, scir::CellId>,
}

impl<'a> ScirConverter<'a> {
    /// Create a new SCIR converter.
    pub fn new(name: impl Into<ArcStr>, ast: &'a Ast) -> Self {
        Self {
            ast,
            lib: scir::Library::new(name),
            blackbox_cells: Default::default(),
            subckts: Default::default(),
            ids: Default::default(),
        }
    }

    /// Blackboxes the given cell.
    pub fn blackbox(&mut self, cell_name: impl Into<Substr>) {
        self.blackbox_cells.insert(cell_name.into());
    }

    /// Consumes the converter, yielding a SCIR [library](scir::Library)].
    pub fn convert(mut self) -> ConvResult<scir::Library> {
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
        Ok(self.lib)
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

        let mut cell = scir::Cell::new_whitebox(ArcStr::from(subckt.name.as_str()));
        let mut nodes: HashMap<Substr, scir::Slice> = HashMap::new();
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
                Component::Mos(_mos) => todo!(),
                Component::Res(res) => {
                    let prim = scir::PrimitiveDeviceKind::Res2 {
                        pos: node(&res.pos, &mut cell),
                        neg: node(&res.neg, &mut cell),
                        value: str_as_numeric_lit(&res.value)?,
                    };
                    cell.add_primitive(prim.into());
                }
                Component::Instance(inst) => {
                    let blackbox = self.blackbox_cells.contains(&inst.child);
                    if blackbox {
                        let ports = inst.ports.iter().map(|s| node(s, &mut cell)).collect();
                        let child = ArcStr::from(inst.child.as_str());
                        let params = inst
                            .params
                            .iter()
                            .map(|(k, v)| Ok((ArcStr::from(k.as_str()), str_as_numeric_lit(v)?)))
                            .collect::<ConvResult<HashMap<_, _>>>()?;
                        let kind = scir::PrimitiveDeviceKind::RawInstance { ports, cell: child };
                        cell.add_primitive(scir::PrimitiveDevice::from_params(kind, params));
                    } else {
                        let subckt = self
                            .subckts
                            .get(&inst.child)
                            .ok_or_else(|| ConvError::MissingSubckt(inst.child.clone()))?;
                        let id = self.convert_subckt(subckt)?;
                        let mut sinst = scir::Instance::new(inst.name.as_str(), id);
                        let child = self
                            .subckts
                            .get(&inst.child)
                            .ok_or_else(|| ConvError::MissingSubckt(inst.child.clone()))?;

                        for (cport, iport) in child.ports.iter().zip(inst.ports.iter()) {
                            sinst.connect(cport.as_str(), node(iport, &mut cell));
                        }

                        for (k, v) in inst.params.iter() {
                            sinst.set_param(k.as_str(), str_as_numeric_lit(v)?);
                        }

                        cell.add_instance(sinst);
                    }
                }
            };
        }

        for port in subckt.ports.iter() {
            let port = node(port, &mut cell);
            cell.expose_port(port);
        }

        let id = self.lib.add_cell(cell);
        self.ids.insert(subckt.name.clone(), id);
        Ok(id)
    }
}

fn str_as_numeric_lit(s: &Substr) -> ConvResult<scir::Expr> {
    Ok(scir::Expr::NumericLiteral(
        s.parse()
            .map_err(|_| ConvError::InvalidLiteral(s.clone()))?,
    ))
}
