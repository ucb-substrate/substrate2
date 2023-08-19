use rust_decimal_macros::dec;
use test_log::test;

use crate::*;

#[test]
fn duplicate_instance_names() {
    let mut lib = LibraryBuilder::new("duplicate_instance_names");
    let id = lib.add_primitive("res");

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", id);
    r1.connect("1", vdd);
    r1.connect("2", int);
    vdivider.add_instance(r1);

    // Duplicate instance name
    let mut r2 = Instance::new("r1", id);
    r2.connect("1", int);
    r2.connect("2", out);
    vdivider.add_instance(r2);

    vdivider.expose_port(vdd, Direction::InOut);
    vdivider.expose_port(vss, Direction::InOut);
    vdivider.expose_port(out, Direction::Output);

    lib.add_cell(vdivider);

    let issues = lib.validate();
    assert_eq!(issues.num_warnings(), 1);
    assert_eq!(issues.num_errors(), 0);
}

#[test]
fn instantiate_blackbox() {
    let mut lib = LibraryBuilder::<()>::new("library");
    let mut cell1 = Cell::new_blackbox("cell1");
    cell1.add_blackbox_elem("* content");
    let cell1 = lib.add_cell(cell1);

    let mut cell2 = Cell::new("cell2");
    cell2.add_instance(Instance::new("cell1", cell1));
    lib.add_cell(cell2);

    let issues = lib.validate();
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(issues.num_errors(), 0);
}

#[test]
#[should_panic]
fn cannot_add_instance_to_blackbox() {
    let mut lib = LibraryBuilder::<()>::new("library");
    let mut cell1 = Cell::new_blackbox("cell1");
    cell1.add_blackbox_elem("* content");
    let cell1 = lib.add_cell(cell1);

    let mut cell2 = Cell::new_blackbox("cell2");
    cell2.add_blackbox_elem("* content");
    cell2.add_instance(Instance::new("cell1", cell1));
}
