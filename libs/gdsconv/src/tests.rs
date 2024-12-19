use std::path::PathBuf;

use gds::GdsUnits;
use geometry::{prelude::Transformation, rect::Rect, shape::Shape as GShape};
use layir::{Cell, Instance, Library, LibraryBuilder, Shape};

use crate::{export_gds, GdsExportOpts, GdsLayer};

pub const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

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

    gds.save(PathBuf::from(BUILD_DIR).join("test_export_layir_to_gds/layout.gds"))
        .expect("failed to write gds");

    assert_eq!(gds.structs.len(), 4);
    assert_eq!(gds.structs[0].name, "bot");
    assert_eq!(gds.structs[3].name, "top");
    assert_eq!(gds.structs[0].elems.len(), 1);
    assert_eq!(gds.structs[1].elems.len(), 2);
    assert_eq!(gds.structs[2].elems.len(), 2);
    assert_eq!(gds.structs[3].elems.len(), 3);
}
