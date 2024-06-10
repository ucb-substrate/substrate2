use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::*;
use spectre::Spectre;
use spice::netlist::{NetlistKind, NetlistOptions, NetlisterInstance};
use spice::{BlackboxContents, BlackboxElement, ComponentValue, Spice};
use std::collections::HashMap;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::schema::Schema;

pub(crate) trait HasRes2: Schema {
    fn resistor(value: usize) -> <Self as Schema>::Primitive;
    fn pos() -> &'static str;
    fn neg() -> &'static str;
}

impl HasRes2 for Spice {
    fn resistor(value: usize) -> spice::Primitive {
        spice::Primitive::Res2 {
            value: ComponentValue::Fixed(Decimal::from(value)),
            params: Default::default(),
        }
    }
    fn pos() -> &'static str {
        "1"
    }
    fn neg() -> &'static str {
        "2"
    }
}

impl HasRes2 for Spectre {
    fn resistor(value: usize) -> spectre::Primitive {
        spectre::Primitive::RawInstance {
            cell: ArcStr::from("resistor"),
            ports: vec!["pos".into(), "neg".into()],
            params: HashMap::from_iter([(ArcStr::from("r"), Decimal::from(value).into())]),
        }
    }
    fn pos() -> &'static str {
        "pos"
    }
    fn neg() -> &'static str {
        "neg"
    }
}

/// Creates a 1:3 resistive voltage divider.
pub(crate) fn vdivider<S: HasRes2>() -> Library<S> {
    let mut lib = LibraryBuilder::new();
    let res = lib.add_primitive(S::resistor(100));

    let mut vdivider = Cell::new("vdivider");
    let vdd = vdivider.add_node("vdd");
    let out = vdivider.add_node("out");
    let int = vdivider.add_node("int");
    let vss = vdivider.add_node("vss");

    let mut r1 = Instance::new("r1", res);
    r1.connect(S::pos(), vdd);
    r1.connect(S::neg(), int);
    vdivider.add_instance(r1);

    let mut r2 = Instance::new("r2", res);
    r2.connect(S::pos(), int);
    r2.connect(S::neg(), out);
    vdivider.add_instance(r2);

    let mut r3 = Instance::new("r3", res);
    r3.connect(S::pos(), out);
    r3.connect(S::neg(), vss);
    vdivider.add_instance(r3);

    vdivider.expose_port(vdd, Direction::InOut);
    vdivider.expose_port(vss, Direction::InOut);
    vdivider.expose_port(out, Direction::Output);
    lib.add_cell(vdivider);

    lib.build().unwrap()
}

/// Creates a 1:3 resistive voltage divider using blackboxed resistors.
pub(crate) fn vdivider_blackbox() -> Library<Spice> {
    let mut lib = LibraryBuilder::new();
    let wrapper = lib.add_primitive(spice::Primitive::BlackboxInstance {
        contents: BlackboxContents {
            elems: vec![
                "R".into(),
                BlackboxElement::InstanceName,
                " ".into(),
                BlackboxElement::Port("pos".into()),
                " ".into(),
                BlackboxElement::Port("neg".into()),
                " 3300".into(),
            ],
        },
    });

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

    vdivider.expose_port(vdd, Direction::InOut);
    vdivider.expose_port(vss, Direction::InOut);
    vdivider.expose_port(out, Direction::Output);
    lib.add_cell(vdivider);

    lib.build().unwrap()
}

#[test]
fn vdivider_is_valid() {
    let lib = vdivider::<Spice>();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}

#[test]
fn vdivider_blackbox_is_valid() {
    let lib = vdivider_blackbox();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}

#[test]
fn netlist_spice_vdivider() {
    let lib = vdivider::<Spice>();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rr1").count(), 1);
    assert_eq!(string.matches("Rr2").count(), 1);
    assert_eq!(string.matches("Rr3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
}

#[test]
fn netlist_spice_vdivider_is_repeatable() {
    let lib = vdivider::<Spice>();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let golden = String::from_utf8(buf).unwrap();

    for i in 0..100 {
        let lib = vdivider::<Spice>();
        let mut buf: Vec<u8> = Vec::new();
        Spice
            .write_scir_netlist(&lib, &mut buf, Default::default())
            .unwrap();
        let attempt = String::from_utf8(buf).unwrap();
        assert_eq!(
            attempt, golden,
            "netlister output changed even though the inputs were the same (iteration {i})"
        );
    }
}

#[test]
fn netlist_spice_vdivider_blackbox() {
    let lib = vdivider_blackbox();
    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&lib, &mut buf, Default::default())
        .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rr1").count(), 1);
    assert_eq!(string.matches("Rr2").count(), 1);
    assert_eq!(string.matches("Rr3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
    assert_eq!(string.matches("3300").count(), 3);
}

#[test]
fn netlist_spectre_vdivider() {
    let lib = vdivider::<Spectre>();
    let mut buf: Vec<u8> = Vec::new();
    let includes = Vec::new();
    NetlisterInstance::new(
        &Spectre {},
        &lib,
        &mut buf,
        NetlistOptions::new(NetlistKind::Cells, &includes),
    )
    .export()
    .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a Spectre netlist parser, we can parse the Spectre back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("subckt").count(), 1);
    assert_eq!(string.matches("ends").count(), 1);
    assert_eq!(string.matches("r1").count(), 1);
    assert_eq!(string.matches("r2").count(), 1);
    assert_eq!(string.matches("r3").count(), 1);
    assert_eq!(string.matches("vdivider").count(), 2);
    assert_eq!(string.matches("resistor r=100").count(), 3);
}
