//! Layout for sky130.

use gdsconv::GdsLayer;
use layir::{Cell, Instance, LibraryBuilder};

use crate::layers::Sky130Layer;

/// Convert a sky130 layout library to a GDS layout library.
// TODO: cell IDs are not preserved
pub fn to_gds(lib: &layir::Library<Sky130Layer>) -> layir::Library<GdsLayer> {
    let mut olib = LibraryBuilder::<GdsLayer>::new();
    let cells = lib.topological_order();
    for cell in cells {
        let cell = lib.cell(cell);
        let mut ocell = Cell::new(cell.name());
        for elt in cell.elements() {
            ocell.add_element(elt.map_layer(Sky130Layer::gds_layer));
        }
        for (_, inst) in cell.instances() {
            let name = lib.cell(inst.child()).name();
            let child_id = olib.cell_id_named(name);
            ocell.add_instance(Instance::new(child_id, inst.name()));
        }
        for (name, port) in cell.ports() {
            ocell.add_port(
                name,
                port.map_layer(|layer| Sky130Layer::gds_pin_layer(layer).unwrap()),
            );
        }
        olib.add_cell(ocell);
    }
    olib.build().unwrap()
}
