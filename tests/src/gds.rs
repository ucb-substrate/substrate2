use substrate::pdk::layers::GdsLayerSpec;
use test_log::test;

use crate::paths::{get_path, test_data};
use crate::shared::pdk::sky130_open_ctx;

#[test]
fn test_gds_import() {
    let ctx = sky130_open_ctx();
    let cell_map = ctx
        .read_gds(test_data("gds/test_sky130_simple.gds"))
        .unwrap()
        .cells;

    let a = cell_map.get("A").unwrap();
    let b = cell_map.get("B").unwrap();
    let a_shapes = a
        .elements()
        .filter_map(|e| e.as_ref().shape())
        .collect::<Vec<_>>();
    let b_insts = b
        .elements()
        .filter_map(|e| e.as_ref().instance())
        .collect::<Vec<_>>();
    let b_shapes = b
        .elements()
        .filter_map(|e| e.as_ref().shape())
        .collect::<Vec<_>>();
    let b_texts = b
        .elements()
        .filter_map(|e| e.as_ref().text())
        .collect::<Vec<_>>();
    let mut b_ports = b.ports();

    assert_eq!(a_shapes.len(), 1, "expected 1 element in cell A");
    let a_shape_0 = a_shapes[0];
    assert!(
        matches!(
            a_shape_0.shape(),
            substrate::geometry::shape::Shape::Rect(_)
        ),
        "expected cell A to have a rectangle"
    );
    assert_eq!(
        a_shape_0.layer(),
        *ctx.layers.met1.drawing.as_ref(),
        "expected rectangle in cell A to be on met1_drawing"
    );

    assert_eq!(b_insts.len(), 4, "expected 4 instances in cell B");
    for inst in b_insts {
        assert_eq!(
            inst.cell().id(),
            a.id(),
            "expected all instances to be instances of cell A"
        );
    }

    // The pin rectangle should be imported as a port, not as an element.
    assert_eq!(b_shapes.len(), 1, "expected 1 element in cell B");
    assert_eq!(b_texts.len(), 0, "expected 0 annotations in cell B");
    let (name, _) = b_ports.next().unwrap();
    assert_eq!(name.to_string(), "gnd", "expected a GND port in cell B");
    assert!(b_ports.next().is_none(), "expected only 1 port in cell B");
}

#[test]
fn test_gds_import_nonexistent_layer() {
    let ctx = sky130_open_ctx();
    let cell_map = ctx
        .read_gds(test_data("gds/test_sky130_nonexistent_layer.gds"))
        .unwrap()
        .cells;

    let new_layer = ctx.get_gds_layer(GdsLayerSpec(0, 0)).unwrap();

    let a = cell_map.get("A").unwrap();
    let a_elems = a
        .elements()
        .filter_map(|e| e.as_ref().shape())
        .collect::<Vec<_>>();
    assert_eq!(a_elems.len(), 1, "expected 1 element in cell A");
    let a_elem_0 = a_elems[0];
    assert_eq!(
        a_elem_0.layer(),
        new_layer,
        "expected element to be on GDS layer (0, 0)"
    );
}

#[test]
fn test_gds_import_invalid_units() {
    let ctx = sky130_open_ctx();
    ctx.read_gds(test_data("gds/test_sky130_invalid_units.gds"))
        .expect_err("should fail due to unit mismatch with PDK");
}

#[test]
fn test_gds_reexport() {
    let gds_path = get_path("test_gds_reexport", "layout.gds");
    let ctx = sky130_open_ctx();

    // Imports a hard macro from a GDS file.
    ctx.write_layout(crate::hard_macro::BufferHardMacro, &gds_path)
        .expect("failed to write layout");
    println!("finished writing layout");

    let ctx_new = sky130_open_ctx();
    let cell_map = ctx_new
        .read_gds(gds_path)
        .expect("failed to import GDS file")
        .cells;
    let a = cell_map.get("buffer").unwrap();
    let b = cell_map.get("buffer_hard_macro").unwrap();
    let a_elems = a
        .elements()
        .filter_map(|e| e.as_ref().shape())
        .collect::<Vec<_>>();
    let a_insts = a
        .elements()
        .filter_map(|e| e.as_ref().instance())
        .collect::<Vec<_>>();
    assert_eq!(a_insts.len(), 0);
    let b_insts = b
        .elements()
        .filter_map(|e| e.as_ref().instance())
        .collect::<Vec<_>>();
    let b_elems = b
        .elements()
        .filter_map(|e| e.as_ref().shape())
        .collect::<Vec<_>>();
    let b_annotations = b
        .elements()
        .filter_map(|e| e.as_ref().text())
        .collect::<Vec<_>>();

    assert_eq!(a_elems.len(), 5, "expected 5 elements in cell A");
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| s.layer() == *ctx.layers.met1.drawing.as_ref())
            .count(),
        4
    );
    assert_eq!(
        a_elems
            .iter()
            .filter(|s| s.layer() == *ctx.layers.poly.drawing.as_ref())
            .count(),
        1
    );

    assert_eq!(b_insts.len(), 1, "expected 1 instance in cell B");
    for inst in b_insts {
        assert_eq!(
            inst.cell().id(),
            a.id(),
            "expected all instances to be instances of cell A"
        );
    }

    assert_eq!(b_elems.len(), 0, "expected no elements in cell B");
    assert_eq!(b_annotations.len(), 0, "expected 0 annotations in cell B");
    assert_eq!(b.ports().count(), 4);
    assert!(b.port_named("vdd").is_some());
    assert!(b.port_named("vss").is_some());
    assert!(b.port_named("din").is_some());
    assert!(b.port_named("dout").is_some());

    let r = b.port_named("vdd").unwrap().primary.shape().rect().unwrap();
    assert_eq!(r.width(), 50);
    assert_eq!(r.height(), 25);

    assert!(a.port_named("vdd").is_some());
    assert!(a.port_named("vss").is_some());
    assert!(a.port_named("din").is_some());
    assert!(a.port_named("dout").is_some());
    assert_eq!(a.ports().count(), 4);

    let r = a.port_named("vdd").unwrap().primary.shape().rect().unwrap();
    assert_eq!(r.width(), 50);
    assert_eq!(r.height(), 25);
}
