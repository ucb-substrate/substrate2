use arcstr::ArcStr;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scir::schema::StringSchema;
use scir::*;
use spectre::{Spectre, SpectrePrimitive};
use spice::{Primitive, Spice};
use substrate::schematic::schema::Schema;

pub(crate) trait HasRes2: Schema {
    fn resistor(value: usize) -> <Self as Schema>::Primitive;
}

impl HasRes2 for Spice {
    fn resistor(value: usize) -> Primitive {
        Primitive::Res2 {
            value: Expr::NumericLiteral(Decimal::from(value)),
        }
    }
}

impl HasRes2 for Spectre {
    fn resistor(value: usize) -> SpectrePrimitive {
        SpectrePrimitive::RawInstance {
            cell: ArcStr::from("resistor"),
            ports: vec!["pos".into(), "neg".into()],
            params: IndexMap::from_iter([(
                ArcStr::from("res"),
                Expr::NumericLiteral(Decimal::from(value)),
            )]),
        }
    }
}

/// Creates a 1:3 resistive voltage divider.
pub(crate) fn vdivider<S: HasRes2>() -> Library<S> {
    let mut lib = LibraryBuilder::new("vdivider");
    let mut wrapper = lib.add_primitive(S::resistor(100));

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

/// Creates a 1:3 resistive voltage divider using blackboxed resistors.
pub(crate) fn vdivider_blackbox() -> Library<StringSchema> {
    let mut lib = LibraryBuilder::new("vdivider");
    // TODO: uncomment
    // let mut wrapper = Cell::new_blackbox("resistor_wrapper");
    // let pos = wrapper.add_node("pos");
    // let neg = wrapper.add_node("neg");
    // wrapper.add_blackbox_elem("Rblackbox");
    // wrapper.add_blackbox_elem(pos);
    // wrapper.add_blackbox_elem(neg);
    // wrapper.add_blackbox_elem("3300");
    // wrapper.expose_port(pos, Direction::InOut);
    // wrapper.expose_port(neg, Direction::InOut);
    // let wrapper = lib.add_cell(wrapper);

    // let mut vdivider = Cell::new("vdivider");
    // let vdd = vdivider.add_node("vdd");
    // let out = vdivider.add_node("out");
    // let int = vdivider.add_node("int");
    // let vss = vdivider.add_node("vss");

    // let contents = vdivider.contents_mut().as_mut().unwrap_cell();
    // let mut r1 = Instance::new("r1", wrapper);
    // r1.connect("pos", vdd);
    // r1.connect("neg", int);
    // contents.add_instance(r1);

    // let mut r2 = Instance::new("r2", wrapper);
    // r2.connect("pos", int);
    // r2.connect("neg", out);
    // contents.add_instance(r2);

    // let mut r3 = Instance::new("r3", wrapper);
    // r3.connect("pos", out);
    // r3.connect("neg", vss);
    // contents.add_instance(r3);

    // vdivider.expose_port(vdd, Direction::InOut);
    // vdivider.expose_port(vss, Direction::InOut);
    // vdivider.expose_port(out, Direction::Output);
    // lib.add_cell(vdivider);

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
    // TODO: uncomment
    // let netlister = Netlister::new(&lib, &[], &mut buf);
    // netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 2);
    assert_eq!(string.matches("ENDS").count(), 2);
    assert_eq!(string.matches("Xr1").count(), 1);
    assert_eq!(string.matches("Xr2").count(), 1);
    assert_eq!(string.matches("Xr3").count(), 1);
    assert_eq!(string.matches("resistor_wrapper").count(), 5);
    assert_eq!(string.matches("vdivider").count(), 3);
    assert_eq!(string.matches("* vdivider").count(), 1);
    assert_eq!(string.matches("Rres0 pos neg 3300").count(), 1);
}

#[test]
fn netlist_spice_vdivider_is_repeatable() {
    let lib = vdivider::<Spice>();
    let mut buf: Vec<u8> = Vec::new();
    // TODO: Uncomment
    // let netlister = Netlister::new(&lib, &[], &mut buf);
    // netlister.export().unwrap();
    let golden = String::from_utf8(buf).unwrap();

    for i in 0..100 {
        let lib = vdivider::<Spice>();
        let mut buf: Vec<u8> = Vec::new();
        // TODO: Uncomment
        // let netlister = Netlister::new(&lib, &[], &mut buf);
        // netlister.export().unwrap();
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
    // TODO: Uncomment
    // let netlister = Netlister::new(&lib, &[], &mut buf);
    // netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT").count(), 2);
    assert_eq!(string.matches("ENDS").count(), 2);
    assert_eq!(string.matches("Xr1").count(), 1);
    assert_eq!(string.matches("Xr2").count(), 1);
    assert_eq!(string.matches("Xr3").count(), 1);
    assert_eq!(string.matches("resistor_wrapper").count(), 5);
    assert_eq!(string.matches("vdivider").count(), 3);
    assert_eq!(string.matches("* vdivider").count(), 1);
    assert_eq!(string.matches("Rblackbox pos neg 3300").count(), 1);
}

#[test]
fn netlist_spectre_vdivider() {
    let lib = vdivider::<Spectre>();
    let mut buf: Vec<u8> = Vec::new();
    // TODO: Uncomment
    // let includes = Vec::new();
    // let netlister = spectre::netlist::Netlister::new(&lib, &includes, &mut buf);
    // netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a Spectre netlist parser, we can parse the Spectre back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("subckt").count(), 2);
    assert_eq!(string.matches("ends").count(), 2);
    assert_eq!(string.matches("r1").count(), 1);
    assert_eq!(string.matches("r2").count(), 1);
    assert_eq!(string.matches("r3").count(), 1);
    assert_eq!(string.matches("resistor_wrapper").count(), 5);
    assert_eq!(string.matches("vdivider").count(), 3);
    assert_eq!(string.matches("// vdivider").count(), 1);
    assert_eq!(
        string.matches("res0 ( pos neg ) resistor r=3300").count(),
        1
    );
}
