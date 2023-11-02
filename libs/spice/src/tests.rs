use crate::{BlackboxContents, BlackboxElement, Primitive, Spice};
use scir::netlist::{NetlistKind, NetlisterInstance};
use scir::{Cell, Direction, Instance, LibraryBuilder};

#[test]
fn scir_netlists_correctly() {
    let mut lib = LibraryBuilder::new();
    let mut top = Cell::new("top");
    let vdd = top.add_node("vdd");
    let vss = top.add_node("vss");
    top.expose_port(vdd, Direction::InOut);
    top.expose_port(vss, Direction::InOut);

    let r_blackbox = lib.add_primitive(Primitive::BlackboxInstance {
        contents: BlackboxContents {
            elems: vec![
                "R".into(),
                BlackboxElement::InstanceName,
                " ".into(),
                BlackboxElement::Port("P".into()),
                " ".into(),
                BlackboxElement::Port("N".into()),
                " 3000".into(),
            ],
        },
    });
    let mut inst = Instance::new("blackbox", r_blackbox);
    inst.connect("P", vdd);
    inst.connect("N", vss);
    top.add_instance(inst);

    let top = lib.add_cell(top);
    lib.set_top(top);
    let lib = lib.build().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let netlister = NetlisterInstance::new(NetlistKind::Cells, &Spice, &lib, &[], &mut buf);
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT top vdd vss").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rblackbox vdd vss 3000").count(), 1);
}
