use geometry::{prelude::Bbox, rect::Rect};
use substrate::context::Context;

use crate::{
    paths::get_path,
    shared::{
        buffer::{Buffer, BufferN, Inverter},
        pdk::{ExamplePdkA, ExamplePdkB},
    },
};

#[test]
fn layout_generation_and_data_propagation_work() {
    let test_name = "layout_generation_and_data_propagation_work";

    let block = Buffer::new(5);

    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_layout(block);
    let cell = handle.cell();

    assert_eq!(cell.block(), &Buffer::new(5));
    assert_eq!(cell.data().inv1.block(), &Inverter::new(5));
    assert_eq!(cell.data().inv2.block(), &Inverter::new(5));

    assert_eq!(
        cell.data().inv1.bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data().inv1.cell().bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data().inv2.bbox(),
        Some(Rect::from_sides(110, 0, 210, 200))
    );

    assert_eq!(
        cell.data().inv2.cell().bbox(),
        Some(Rect::from_sides(110, 0, 210, 200))
    );

    assert_eq!(cell.bbox(), Some(Rect::from_sides(0, 0, 210, 200)));

    ctx.write_layout(block, get_path(test_name, "layout_pdk_a.gds"))
        .expect("failed to write layout");

    let mut ctx = Context::new(ExamplePdkB);
    let handle = ctx.generate_layout(Buffer::new(5));
    let cell = handle.cell();

    assert_eq!(
        cell.data().inv1.bbox(),
        Some(Rect::from_sides(0, 0, 200, 100))
    );

    assert_eq!(
        cell.data().inv2.bbox(),
        Some(Rect::from_sides(210, 0, 410, 100))
    );

    assert_eq!(cell.bbox(), Some(Rect::from_sides(0, 0, 410, 100)));

    ctx.write_layout(block, get_path(test_name, "layout_pdk_b.gds"))
        .expect("failed to write layout");
}

#[test]
fn nested_transform_views_work() {
    let test_name = "nested_transform_views_work";

    let block = BufferN::new(5, 10);

    let mut ctx = Context::new(ExamplePdkA);
    ctx.write_layout(block, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");

    let handle = ctx.generate_layout(block);
    let cell = handle.cell();

    assert_eq!(
        cell.data().buffers[9].data().inv2.bbox(),
        Some(Rect::from_sides(2090, 0, 2190, 200))
    );
}
