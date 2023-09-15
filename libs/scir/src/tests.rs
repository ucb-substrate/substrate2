use test_log::test;

use crate::schema::{FromSchema, StringSchema};
use crate::*;

#[test]
fn duplicate_cell_names() {
    let c1 = Cell::new("duplicate_cell_name");
    let c2 = Cell::new("duplicate_cell_name");
    let mut lib = <LibraryBuilder>::new("duplicate_cell_names");
    lib.add_cell(c1);
    lib.add_cell(c2);
    let issues = lib.validate();
    assert!(issues.has_error() || issues.has_warning());
}

#[test]
fn duplicate_instance_names() {
    let mut lib = LibraryBuilder::<StringSchema>::new("duplicate_instance_names");
    let id = lib.add_primitive("res".into());

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
fn no_schema_conversion() {
    let mut lib = LibraryBuilder::<StringSchema>::new("duplicate_instance_names");
    let empty_cell = Cell::new("empty");
    let id = lib.add_cell(empty_cell);

    let no_schema_lib = lib.drop_schema().unwrap();
    assert_eq!(no_schema_lib.cell(id).name(), "empty");

    let mut lib: LibraryBuilder<StringSchema> = no_schema_lib.convert_schema().unwrap();
    assert_eq!(lib.cell(id).name(), "empty");

    let id = lib.add_primitive("res".into());

    let mut resistor = Cell::new("resistor");
    let vdd = resistor.add_node("vdd");
    let vss = resistor.add_node("vss");

    let mut r1 = Instance::new("r1", id);
    r1.connect("1", vdd);
    r1.connect("2", vss);
    resistor.add_instance(r1);

    resistor.expose_port(vdd, Direction::InOut);
    resistor.expose_port(vss, Direction::InOut);

    lib.add_cell(resistor);

    assert!(lib.drop_schema().is_err());
}

#[test]
fn schema_conversion() {
    pub struct PartiallyTypedSchema;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub enum PartiallyTypedPrimitive {
        PrimA,
        PrimB,
        Other(ArcStr),
    }

    impl Schema for PartiallyTypedSchema {
        type Primitive = PartiallyTypedPrimitive;
    }

    impl FromSchema<StringSchema> for PartiallyTypedSchema {
        type Error = ();

        fn recover_primitive(
            primitive: <StringSchema as Schema>::Primitive,
        ) -> Result<<Self as Schema>::Primitive, Self::Error> {
            Ok(match primitive.as_ref() {
                "prim_a" => PartiallyTypedPrimitive::PrimA,
                "prim_b" => PartiallyTypedPrimitive::PrimB,
                "invalid_prim" => {
                    return Err(());
                }
                _ => PartiallyTypedPrimitive::Other(primitive),
            })
        }

        fn recover_instance(
            instance: &mut Instance,
            primitive: &<StringSchema as Schema>::Primitive,
        ) -> Result<(), Self::Error> {
            instance.map_connections(|conn| match (primitive.as_ref(), conn.as_ref()) {
                ("prim_a", "a1") => arcstr::literal!("a_pt1"),
                ("prim_a", "a2") => arcstr::literal!("a_pt2"),
                ("prim_b", "b1") => arcstr::literal!("b_pt1"),
                ("prim_b", "b2") => arcstr::literal!("b_pt2"),
                _ => conn,
            });
            Ok(())
        }
    }

    impl FromSchema<PartiallyTypedSchema> for StringSchema {
        type Error = ();

        fn recover_primitive(
            primitive: <PartiallyTypedSchema as Schema>::Primitive,
        ) -> Result<<Self as Schema>::Primitive, Self::Error> {
            Ok(match primitive {
                PartiallyTypedPrimitive::PrimA => arcstr::literal!("prim_a"),
                PartiallyTypedPrimitive::PrimB => arcstr::literal!("prim_b"),
                PartiallyTypedPrimitive::Other(inner) => inner,
            })
        }

        fn recover_instance(
            instance: &mut Instance,
            primitive: &<PartiallyTypedSchema as Schema>::Primitive,
        ) -> Result<(), Self::Error> {
            instance.map_connections(|conn| match (primitive, conn.as_ref()) {
                (&PartiallyTypedPrimitive::PrimA, "a_pt1") => arcstr::literal!("a1"),
                (&PartiallyTypedPrimitive::PrimA, "a_pt2") => arcstr::literal!("a2"),
                (&PartiallyTypedPrimitive::PrimB, "b_pt1") => arcstr::literal!("b1"),
                (&PartiallyTypedPrimitive::PrimB, "b_pt2") => arcstr::literal!("b2"),
                _ => conn,
            });
            Ok(())
        }
    }

    let mut lib = LibraryBuilder::<StringSchema>::new("schema_conversion");

    let prim_a = lib.add_primitive("prim_a".into());
    let prim_b = lib.add_primitive("prim_b".into());

    let mut cell = Cell::new("prim_cell");
    let vdd = cell.add_node("vdd");
    let vss = cell.add_node("vss");

    let mut inst_a = Instance::new("inst_a", prim_a);
    inst_a.connect("a1", vdd);
    inst_a.connect("a2", vss);
    let inst_a = cell.add_instance(inst_a);

    let mut inst_b = Instance::new("inst_b", prim_b);
    inst_b.connect("b1", vdd);
    inst_b.connect("b2", vss);
    let b_inst = cell.add_instance(inst_b);

    cell.expose_port(vdd, Direction::InOut);
    cell.expose_port(vss, Direction::InOut);

    let cell = lib.add_cell(cell);

    let ptlib = lib.convert_schema::<PartiallyTypedSchema>().unwrap();
    let ptcell = ptlib.cell(cell);
    assert!(ptcell.instance(inst_a).connections().contains_key("a_pt1"));
    assert!(ptcell.instance(inst_a).connections().contains_key("a_pt2"));
    assert!(ptcell.instance(b_inst).connections().contains_key("b_pt1"));
    assert!(ptcell.instance(b_inst).connections().contains_key("b_pt2"));

    assert_eq!(ptlib.primitive(prim_a), &PartiallyTypedPrimitive::PrimA);
    assert_eq!(ptlib.primitive(prim_b), &PartiallyTypedPrimitive::PrimB);

    let mut orig_lib = ptlib.convert_schema::<StringSchema>().unwrap();
    let orig_cell = orig_lib.cell(cell);
    assert!(orig_cell.instance(inst_a).connections().contains_key("a1"));
    assert!(orig_cell.instance(inst_a).connections().contains_key("a2"));
    assert!(orig_cell.instance(b_inst).connections().contains_key("b1"));
    assert!(orig_cell.instance(b_inst).connections().contains_key("b2"));

    assert_eq!(orig_lib.primitive(prim_a), "prim_a");
    assert_eq!(orig_lib.primitive(prim_b), "prim_b");

    orig_lib.add_primitive("invalid_prim".into());
    assert!(orig_lib.convert_schema::<PartiallyTypedSchema>().is_err());
}
