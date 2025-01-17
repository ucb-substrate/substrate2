use std::path::PathBuf;

use gds::{GdsLibrary, GdsUnits};
use geometry::{prelude::Transformation, rect::Rect, shape::Shape as GShape};
use layir::{Cell, Element, Instance, Library, LibraryBuilder, Shape, Text};

use crate::{
    export::{export_gds, GdsExportOpts},
    import::{import_gds, GdsImportOpts},
    GdsLayer,
};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");

#[inline]
fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

#[inline]
fn test_data(file_name: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(file_name)
}

fn gdslib() -> Library<GdsLayer> {
    let mut lib = LibraryBuilder::new();
    let mut bot = Cell::new("bot");
    bot.add_element(Shape::new(
        GdsLayer(1, 0),
        GShape::Rect(Rect::from_sides(0, 0, 100, 100)),
    ));
    let bot = lib.add_cell(bot);
    let mut mid1 = Cell::new("mid1");
    mid1.add_element(Shape::new(
        GdsLayer(2, 0),
        GShape::Rect(Rect::from_sides(100, 0, 200, 100)),
    ));
    mid1.add_instance(Instance::new(bot, "xbot"));
    let mid1 = lib.add_cell(mid1);
    let mut mid2 = Cell::new("mid2");
    mid2.add_element(Shape::new(
        GdsLayer(3, 0),
        GShape::Rect(Rect::from_sides(0, 0, 100, 100)),
    ));
    mid2.add_instance(Instance::with_transformation(
        bot,
        "xbot",
        Transformation::translate(100, 0),
    ));
    let mid2 = lib.add_cell(mid2);
    let mut top = Cell::new("top");
    top.add_element(Shape::new(
        GdsLayer(4, 0),
        GShape::Rect(Rect::from_sides(0, 0, 200, 200)),
    ));
    top.add_instance(Instance::with_transformation(
        mid1,
        "xmid1",
        Transformation::translate(0, 100),
    ));
    top.add_instance(Instance::with_transformation(
        mid2,
        "xmid2",
        Transformation::identity(),
    ));
    lib.add_cell(top);
    lib.build().unwrap()
}

#[test]
fn test_export_layir_to_gds() {
    let lib = gdslib();
    let opts = GdsExportOpts {
        name: "top".into(),
        units: Some(GdsUnits::new(1., 1e-6)),
    };
    let gds = export_gds(lib, opts);

    gds.save(get_path("test_export_layir_to_gds", "layout.gds"))
        .expect("failed to write gds");

    assert_eq!(gds.structs.len(), 4);
    assert_eq!(gds.structs[0].name, "bot");
    assert_eq!(gds.structs[3].name, "top");
    assert_eq!(gds.structs[0].elems.len(), 1);
    assert_eq!(gds.structs[1].elems.len(), 2);
    assert_eq!(gds.structs[2].elems.len(), 2);
    assert_eq!(gds.structs[3].elems.len(), 3);
}

#[test]
fn test_gds_import() {
    let path = test_data("gds/test_sky130_simple.gds");
    let bytes = std::fs::read(path).expect("failed to read GDS");
    let rawlib = GdsLibrary::from_bytes(bytes).expect("failed to parse GDS");
    let lib =
        import_gds(&rawlib, GdsImportOpts { units: None }).expect("failed to import to LayIR");

    let a = lib.cell_named("A");
    let a_id = lib.cell_id_named("A");
    let b = lib.cell_named("B");
    let a_elems = a.elements().collect::<Vec<_>>();
    let b_insts = b.instances().collect::<Vec<_>>();
    let b_elems = b.elements().collect::<Vec<_>>();

    assert_eq!(a_elems.len(), 1, "expected 1 element in cell A");
    let a_elem_0 = a_elems[0];
    assert_eq!(
        a_elem_0,
        &Element::Shape(Shape::new(
            GdsLayer(68, 20),
            Rect::from_sides(0, 0, 500, 500)
        )),
    );

    assert_eq!(b_insts.len(), 4, "expected 4 instances in cell B");
    for (_, inst) in b_insts {
        assert_eq!(
            inst.child(),
            a_id,
            "expected all instances to be instances of cell A"
        );
    }

    assert_eq!(b_elems.len(), 3, "expected 3 elements in cell B");
    for elem in b_elems {
        match elem {
            Element::Shape(s) => {
                assert!([
                    Shape::new(GdsLayer(67, 20), Rect::from_sides(0, 0, 3000, 3000)),
                    Shape::new(GdsLayer(67, 16), Rect::from_sides(0, 0, 1000, 1000))
                ]
                .contains(s));
            }
            Element::Text(t) => {
                assert_eq!(
                    t,
                    &Text::with_transformation(
                        GdsLayer(67, 5),
                        "gnd",
                        Transformation::translate(500, 500)
                    )
                );
            }
        }
    }
}

#[test]
fn test_gds_import_invalid_units() {
    let bytes =
        std::fs::read(test_data("gds/test_sky130_invalid_units.gds")).expect("failed to read GDS");
    let rawlib = GdsLibrary::from_bytes(bytes).expect("failed to parse GDS");
    import_gds(
        &rawlib,
        GdsImportOpts {
            units: Some(GdsUnits::new(1e-3, 1e-9)),
        },
    )
    .expect_err("should fail due to unit mismatch with PDK");
}

#[test]
fn test_gds_reexport() {
    let bytes = std::fs::read(test_data("gds/buffer.gds")).expect("failed to read GDS");
    let rawlib = GdsLibrary::from_bytes(bytes).expect("failed to parse GDS");
    let lib =
        import_gds(&rawlib, GdsImportOpts { units: None }).expect("failed to import to LayIR");

    let gds_path = get_path("test_gds_reexport", "layout.gds");
    let rawlib2 = export_gds(
        lib,
        GdsExportOpts {
            name: "TOP".into(),
            units: None,
        },
    );
    rawlib2.save(&gds_path).expect("failed to save GDS");

    let bytes = std::fs::read(&gds_path).expect("failed to read GDS");
    let rawlib3 = GdsLibrary::from_bytes(bytes).expect("failed to parse GDS");
    let lib2 =
        import_gds(&rawlib3, GdsImportOpts { units: None }).expect("failed to import to LayIR");

    let a = lib2.cell_named("buffer");
    let a_elems = a.elements().collect::<Vec<_>>();
    let a_insts = a.instances().collect::<Vec<_>>();
    assert_eq!(a_insts.len(), 0);

    assert_eq!(a_elems.len(), 13, "expected 13 elements in cell buffer");
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| match s {
                Element::Shape(s) => s.layer() == &GdsLayer(68, 20),
                _ => false,
            })
            .count(),
        4
    );
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| match s {
                Element::Shape(s) => s.layer() == &GdsLayer(68, 16),
                _ => false,
            })
            .count(),
        4
    );
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| match s {
                Element::Text(t) => t.layer() == &GdsLayer(68, 5),
                _ => false,
            })
            .count(),
        4
    );
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| match s {
                Element::Shape(s) => s.layer() == &GdsLayer(66, 20),
                _ => false,
            })
            .count(),
        1
    );
}
