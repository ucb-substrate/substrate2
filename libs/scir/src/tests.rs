use test_log::test;

use crate::schema::{FromSchema, StringSchema};
use crate::*;

#[test]
fn duplicate_cell_names() {
    let c1 = Cell::new("duplicate_cell_name");
    let c2 = Cell::new("duplicate_cell_name");
    let mut lib = <LibraryBuilder>::new();
    lib.add_cell(c1);
    lib.add_cell(c2);
    let issues = lib.validate();
    assert!(issues.has_error());
}

#[test]
fn duplicate_instance_names() {
    let mut lib = LibraryBuilder::<StringSchema>::new();
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
    assert!(issues.has_error());
}

#[test]
fn duplicate_signal_names() {
    let mut lib = LibraryBuilder::<StringSchema>::new();

    let mut cell = Cell::new("cell");
    cell.add_node("duplicate_signal");
    cell.add_node("duplicate_signal");
    lib.add_cell(cell);

    let issues = lib.validate();
    assert!(issues.has_error());
}

#[test]
fn no_schema_conversion() {
    let mut lib = LibraryBuilder::<StringSchema>::new();
    let empty_cell = Cell::new("empty");
    let id = lib.add_cell(empty_cell);

    let no_schema_lib = lib.drop_schema().unwrap();
    assert_eq!(no_schema_lib.cell(id).name(), "empty");

    let mut lib: LibraryBuilder<StringSchema> = no_schema_lib.convert_schema().unwrap();
    assert_eq!(lib.cell(id).name(), "empty");

    let id = lib.add_primitive("res".into());

    let mut resistor = Cell::new("ideal_resistor");
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

        fn convert_primitive(
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

        fn convert_instance(
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

        fn convert_primitive(
            primitive: <PartiallyTypedSchema as Schema>::Primitive,
        ) -> Result<<Self as Schema>::Primitive, Self::Error> {
            Ok(match primitive {
                PartiallyTypedPrimitive::PrimA => arcstr::literal!("prim_a"),
                PartiallyTypedPrimitive::PrimB => arcstr::literal!("prim_b"),
                PartiallyTypedPrimitive::Other(inner) => inner,
            })
        }

        fn convert_instance(
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

    let mut lib = LibraryBuilder::<StringSchema>::new();

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

/// Returns a SCIR library with nested cells and 3 varieties of [`SliceOnePath`]s that
/// address the VDD node of the innermost instance for testing purposes.
fn nested_lib(n: usize) -> (Library<StringSchema>, Vec<SliceOnePath>) {
    let mut lib = LibraryBuilder::<StringSchema>::new();

    let prim_inst = lib.add_primitive("prim_inst".into());

    let mut signals = Vec::<(SliceOne, SliceOne)>::new();
    let mut insts = Vec::<InstanceId>::new();
    let mut cells = Vec::<CellId>::new();

    for i in 0..n {
        let mut cell = Cell::new(format!("cell_{}", i));
        let vdd = cell.add_node("vdd");
        let vss = cell.add_node("vss");
        signals.push((vdd, vss));

        let mut inst = Instance::new(
            "inst",
            if i == 0 {
                ChildId::from(prim_inst)
            } else {
                ChildId::from(*cells.last().unwrap())
            },
        );
        if i < n - 1 {
            inst.connect("vdd", vdd);
        }
        inst.connect("vss", vss);
        insts.push(cell.add_instance(inst));

        // Do not expose VDD on topmost two cells to test path simplification.
        if i < n - 2 {
            cell.expose_port(vdd, Direction::InOut);
        }
        cell.expose_port(vss, Direction::InOut);
        cells.push(lib.add_cell(cell));
    }

    let lib = lib.build().unwrap();

    // Test name path API.
    let mut name_path = InstancePath::new(format!("cell_{}", n - 1));
    name_path.push_iter((1..n).map(|_| "inst"));
    let name_path = name_path.slice_one(NamedSliceOne::new("vdd"));

    // Test ID path API.
    let mut id_path = InstancePath::new(*cells.last().unwrap());
    id_path.push_iter((1..n).rev().map(|i| insts[i]));
    let id_path = id_path.slice_one(signals.first().unwrap().0);

    // Test mixing name and ID path APIs.
    let mut mixed_path = InstancePath::new(format!("cell_{}", n - 1));
    mixed_path.push_iter((1..n).rev().map(|i| {
        if i % 2 == 0 {
            InstancePathElement::from(insts[i])
        } else {
            InstancePathElement::from("inst")
        }
    }));
    let mixed_path = mixed_path.slice_one(signals.first().unwrap().0);

    (lib, vec![name_path, id_path, mixed_path])
}

#[test]
fn path_simplification() {
    const N: usize = 5;

    let (lib, paths) = nested_lib(N);

    for path in paths {
        assert_eq!(path.instances().len(), N - 1);
        let simplified_path = lib.simplify_path(path);
        // Simplified path should bubble up to `cell_{N-2}`.
        assert_eq!(simplified_path.instances().len(), 1);
    }
}

#[test]
fn name_path_conversion() {
    const N: usize = 5;

    let (lib, paths) = nested_lib(N);

    for path in paths {
        let name_path = lib.convert_slice_one_path(path, |name, index| {
            if let Some(index) = index {
                arcstr::format!("{}[{}]", name, index)
            } else {
                name.clone()
            }
        });

        assert_eq!(
            name_path.join("."),
            ["inst"; N - 1]
                .into_iter()
                .chain(["vdd"])
                .collect::<Vec<&str>>()
                .join(".")
        );
    }
}
