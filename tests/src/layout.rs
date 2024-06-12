use geometry::prelude::{NamedOrientation, Point};
use geometry::side::Sides;
use geometry::{prelude::Bbox, rect::Rect};
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::context::PdkContext;
use substrate::geometry::transform::{Transform, TransformMut, Translate, TranslateMut};
use substrate::layout::element::Shape;
use substrate::layout::tiling::{GridTile, GridTiler, Tile};
use substrate::layout::{ExportsLayoutData, Instance, Layout, LayoutData};

use crate::shared::buffer::{BufferNxM, Inverter};

use crate::{
    paths::get_path,
    shared::{
        buffer::{Buffer, BufferN},
        pdk::{ExamplePdkA, ExamplePdkB},
    },
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, TranslateMut, TransformMut)]
pub struct TwoPointGroup {
    p1: Point,
    p2: Point,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TranslateMut, TransformMut)]
pub enum PointEnum {
    First(Point),
    Second { pt: Point },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Block, Serialize, Deserialize, Hash)]
#[substrate(io = "()")]
pub struct GridTilerExample;

impl ExportsLayoutData for GridTilerExample {
    type LayoutData = ();
}

impl Layout<ExamplePdkA> for GridTilerExample {
    fn layout(
        &self,
        _io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let mut tiler = GridTiler::new();

        let tile1 = Tile::from_bbox(Shape::new(
            cell.ctx.layers.polya,
            Rect::from_sides(0, 0, 100, 100),
        ))
        .with_padding(Sides::uniform(10));

        let tile2 = Tile::from_bbox(Shape::new(
            cell.ctx.layers.met2a,
            Rect::from_sides(0, 0, 220, 220),
        ))
        .with_padding(Sides::uniform(10));

        tiler.push_num(tile1.clone(), 6);
        tiler.end_row();
        tiler.push_num(tile1.clone(), 2);
        tiler.push(GridTile::new(tile2).with_colspan(2).with_rowspan(2));
        tiler.push_num(tile1.clone(), 2);
        tiler.end_row();
        tiler.push_num(tile1.clone(), 4);
        tiler.end_row();
        tiler.push_num(tile1, 6);

        let grid = tiler.tile();

        cell.draw(grid)?;

        Ok(())
    }
}

#[test]
fn layout_generation_and_data_propagation_work() {
    let test_name = "layout_generation_and_data_propagation_work";

    let block = Buffer::new(5);

    let ctx = PdkContext::new(ExamplePdkA);
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

    let ctx = PdkContext::new(ExamplePdkB);
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

    let ctx = PdkContext::new(ExamplePdkA);
    ctx.write_layout(block, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");

    let handle = ctx.generate_layout(block);
    let cell = handle.cell();

    assert_eq!(
        cell.data().buffers[9].data().inv2.bbox(),
        Some(Rect::from_sides(2090, 0, 2190, 200))
    );
}

#[test]
fn cell_builder_supports_bbox() {
    let test_name = "cell_builder_supports_bbox";

    let block = BufferNxM::new(5, 10, 6);

    let ctx = PdkContext::new(ExamplePdkA);
    ctx.write_layout(block, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");

    let handle = ctx.generate_layout(block);
    let cell = handle.cell();

    assert_eq!(cell.bbox(), Some(Rect::from_sides(-10, -1110, 2200, 210)));
}

#[test]
fn export_multi_top_layout() {
    let test_name = "export_multi_top_layout";

    let block1 = BufferNxM::new(5, 10, 6);
    let block2 = BufferNxM::new(5, 10, 6);
    let block3 = BufferNxM::new(8, 12, 4);

    let ctx = PdkContext::new(ExamplePdkA);
    let block1 = ctx.generate_layout(block1);
    let block2 = ctx.generate_layout(block2);
    let block3 = ctx.generate_layout(block3);
    ctx.write_layout_all(
        [
            block1.cell().raw().clone(),
            block2.cell().raw().clone(),
            block3.cell().raw().clone(),
        ],
        get_path(test_name, "layout.gds"),
    )
    .expect("failed to write layout");
}

#[test]
fn grid_tiler_works_with_various_spans() {
    let test_name = "grid_tiler_works_with_various_spans";

    let ctx = PdkContext::new(ExamplePdkA);
    ctx.write_layout(GridTilerExample, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");
}

#[test]
fn translate_two_point_group() {
    let group = TwoPointGroup {
        p1: Point::new(100, 200),
        p2: Point::new(-400, 300),
    };

    let group = group.translate(Point::new(100, 50));
    assert_eq!(
        group,
        TwoPointGroup {
            p1: Point::new(200, 250),
            p2: Point::new(-300, 350),
        }
    );
}

#[test]
fn transform_point_enum() {
    let mut group = PointEnum::First(Point::new(100, 200));
    group = group.transform(NamedOrientation::ReflectVert.into());
    assert_eq!(group, PointEnum::First(Point::new(100, -200)),);
}

// Test LayoutData proc macro
#[derive(LayoutData)]
pub enum MyData {
    Unit,
    Tuple(Instance<Inverter>),
    Strukt { val: Instance<Inverter> },
}
