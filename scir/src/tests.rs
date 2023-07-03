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
        .add_primitive(PrimitiveDevice::Res2 {
            pos,
            neg,
            value: dec!(3300).into(),
        });
    wrapper.expose_port(pos);
    wrapper.expose_port(neg);
    let wrapper = lib.add_cell(wrapper);

    let mut vdivider = Cell::new_whitebox("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", wrapper);
    r1.connect("pos", vdd);
    r1.connect("neg", int);
    vdivider
        .contents_mut()
        .as_mut()
        .unwrap_clear()
        .add_instance(r1);

    // Duplicate instance name
    let mut r2 = Instance::new("r1", wrapper);
    r2.connect("pos", int);
    r2.connect("neg", out);
    vdivider
        .contents_mut()
        .as_mut()
        .unwrap_clear()
        .add_instance(r2);

    vdivider.expose_port(vdd);
    vdivider.expose_port(vss);
    vdivider.expose_port(out);

    lib.add_cell(vdivider);

    let issues = lib.validate();
    assert_eq!(issues.num_warnings(), 1);
    assert_eq!(issues.num_errors(), 0);
}
