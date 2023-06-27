use rust_decimal_macros::dec;
use test_log::test;

use crate::*;

/// Creates a 1:3 resistive voltage divider.
pub(crate) fn vdivider() -> Library {
    let mut lib = Library::new();
    let mut wrapper = Cell::new("resistor_wrapper");
    let pos = wrapper.add_node("pos");
    let neg = wrapper.add_node("neg");
    wrapper.add_primitive(PrimitiveDevice::Res2 {
        pos,
        neg,
        value: dec!(3300).into(),
    });
    wrapper.expose_port(pos);
    wrapper.expose_port(neg);
    let wrapper = lib.add_cell(wrapper);

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", wrapper);
    r1.connect("pos", vdd);
    r1.connect("neg", int);
    vdivider.add_instance(r1);

    let mut r2 = Instance::new("r2", wrapper);
    r2.connect("pos", int);
    r2.connect("neg", out);
    vdivider.add_instance(r2);

    let mut r3 = Instance::new("r3", wrapper);
    r3.connect("pos", out);
    r3.connect("neg", vss);
    vdivider.add_instance(r3);

    vdivider.expose_port(vdd);
    vdivider.expose_port(vss);
    vdivider.expose_port(out);
    lib.add_cell(vdivider);

    lib
}

#[test]
fn vdivider_is_valid() {
    let lib = vdivider();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}

#[test]
fn duplicate_instance_names() {
    let mut lib = Library::new();
    let mut wrapper = Cell::new("resistor_wrapper");
    let pos = wrapper.add_node("pos");
    let neg = wrapper.add_node("neg");
    wrapper.add_primitive(PrimitiveDevice::Res2 {
        pos,
        neg,
        value: dec!(3300).into(),
    });
    wrapper.expose_port(pos);
    wrapper.expose_port(neg);
    let wrapper = lib.add_cell(wrapper);

    let mut vdivider = Cell::new("vdivider");
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

    vdivider.expose_port(vdd);
    vdivider.expose_port(vss);
    vdivider.expose_port(out);

    lib.add_cell(vdivider);

    let issues = lib.validate();
    assert_eq!(issues.num_warnings(), 1);
    assert_eq!(issues.num_errors(), 0);
}
