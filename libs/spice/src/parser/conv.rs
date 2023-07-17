//! Convert SPICE netlists to other formats.
//!
//! Currently, we only support converting to SCIR.
//!
//! TODO: bus ports, expressions, validation, ArcStr deduplication.

use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;

use super::{Ast, Component, Elem, Subckt, Substr};

type SubcktName = Substr;

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
    pub fn convert(mut self) -> scir::Library {
        self.map_subckts();
        for elem in self.ast.elems.iter() {
            match elem {
                Elem::Subckt(subckt) => {
                    self.convert_subckt(subckt);
                }
                _ => continue,
            }
        }
        self.lib
    }

    fn map_subckts(&mut self) {
        for elem in self.ast.elems.iter() {
            match elem {
                Elem::Subckt(s) => {
                    self.subckts.insert(s.name.clone(), s);
                }
                _ => continue,
            }
        }
    }

    fn convert_subckt(&mut self, subckt: &Subckt) -> scir::CellId {
        if let Some(&id) = self.ids.get(&subckt.name) {
            return id;
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
                    let prim = scir::PrimitiveDevice::Res2 {
                        pos: node(&res.pos, &mut cell),
                        neg: node(&res.neg, &mut cell),
                        value: str_as_numeric_lit(&res.value),
                    };
                    cell.add_primitive(prim);
                }
                Component::Instance(inst) => {
                    let blackbox = self.blackbox_cells.contains(&inst.child);
                    if blackbox {
                        let ports = inst.ports.iter().map(|s| node(s, &mut cell)).collect();
                        let child = ArcStr::from(inst.child.as_str());
                        let params = inst
                            .params
                            .iter()
                            .map(|(k, v)| (ArcStr::from(k.as_str()), str_as_numeric_lit(v)))
                            .collect();
                        let prim = scir::PrimitiveDevice::RawInstance {
                            ports,
                            cell: child,
                            params,
                        };
                        cell.add_primitive(prim);
                    } else {
                        let subckt = self.subckts.get(&inst.name).unwrap();
                        let id = self.convert_subckt(subckt);
                        let mut sinst = scir::Instance::new(inst.name.as_str(), id);
                        let child = self.subckts.get(&inst.child).unwrap();

                        for (cport, iport) in child.ports.iter().zip(inst.ports.iter()) {
                            sinst.connect(cport.as_str(), node(iport, &mut cell));
                        }

                        for (k, v) in inst.params.iter() {
                            sinst.set_param(k.as_str(), str_as_numeric_lit(v));
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
        id
    }
}

fn str_as_numeric_lit(s: &Substr) -> scir::Expr {
    scir::Expr::NumericLiteral(s.parse().unwrap())
}
