use rust_decimal_macros::dec;
use test_log::test;

use crate::*;

#[test]
fn duplicate_instance_names() {
    let mut lib = Library::new("duplicate_instance_names");
    let mut wrapper = Cell::new_whitebox("resistor_wrapper");
    let pos = wrapper.add_node("pos");
    let neg = wrapper.add_node("neg");
    wrapper
        .contents_mut()
        .as_mut()
        .unwrap_clear()
        .add_primitive(PrimitiveDevice::new(
            "res0",
            PrimitiveDeviceKind::Res2 {
                pos,
                neg,
                value: dec!(3300).into(),
            },
        ));
    wrapper.expose_port(pos, Direction::InOut);
    wrapper.expose_port(neg, Direction::InOut);
    let wrapper = lib.add_cell(wrapper);

    let mut vdivider = Cell::new_whitebox("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", wrapper);
    r1.connect("pos", vdd);
    r1.connect("neg", int);
    vdivider.add_instance(r1);

    // Duplicate instance name
    let mut r2 = Instance::new("r1", wrapper);
    r2.connect("pos", int);
    r2.connect("neg", out);
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
    let mut lib = Library::new("library");
    let mut cell1 = Cell::new_blackbox("cell1");
    cell1.add_blackbox_elem("* content");
    let cell1 = lib.add_cell(cell1);

    let mut cell2 = Cell::new_whitebox("cell2");
    cell2.add_instance(Instance::new("cell1", cell1));
    lib.add_cell(cell2);

    let issues = lib.validate();
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(issues.num_errors(), 0);
}

#[test]
#[should_panic]
fn cannot_add_instance_to_blackbox() {
    let mut lib = Library::new("library");
    let mut cell1 = Cell::new_blackbox("cell1");
    cell1.add_blackbox_elem("* content");
    let cell1 = lib.add_cell(cell1);

    let mut cell2 = Cell::new_blackbox("cell2");
    cell2.add_blackbox_elem("* content");
    cell2.add_instance(Instance::new("cell1", cell1));
}

#[test]
#[should_panic]
fn cannot_add_primitive_to_blackbox() {
    let mut cell = Cell::new_blackbox("cell");
    cell.add_blackbox_elem("* content");
    cell.add_primitive(PrimitiveDevice::new(
        "rawinst",
        PrimitiveDeviceKind::RawInstance {
            ports: vec![],
            cell: "raw_subckt".into(),
        },
    ));
}
