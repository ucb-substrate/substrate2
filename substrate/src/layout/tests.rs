use gds::GdsUnits;
use gdsconv::GdsLayer;
use geometry::{
    align::{AlignBbox, AlignMode},
    bbox::Bbox,
    rect::Rect,
    side::Sides,
    transform::{TransformMut, TransformRef, TranslateMut, TranslateRef},
    union::BoundingUnion,
};
use layir::{Cell, LibraryBuilder, Shape};

use crate::{
    block::Block,
    context::Context,
    tests::{get_path, Buffer, BufferIo, BufferIoView, BufferN, BufferNxM, BufferNxMIo, Inverter},
    types::{
        codegen::{PortGeometryBundle, View},
        layout::{PortGeometry, PortGeometryBuilder},
        ArrayBundle, Signal,
    },
};

use super::{
    schema::Schema,
    tiling::{ArrayTiler, GridTile, GridTiler, Tile, TileAlignMode},
    CellBundle, Instance, Layout,
};

fn gds_units() -> GdsUnits {
    GdsUnits::new(1., 1e-9)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExampleLayer {
    A,
    B,
    C,
}

pub struct ExampleSchema;

impl ExampleLayer {
    pub fn gds_layer(&self) -> GdsLayer {
        match self {
            Self::A => GdsLayer(0, 0),
            Self::B => GdsLayer(1, 0),
            Self::C => GdsLayer(2, 0),
        }
    }
    pub fn gds_pin_layer(&self) -> Option<GdsLayer> {
        let layer = match self {
            Self::A => GdsLayer(0, 1),
            Self::B => GdsLayer(1, 1),
            Self::C => GdsLayer(2, 1),
        };
        Some(layer)
    }
}

// TODO: cell IDs are not preserved
pub fn to_gds(lib: &layir::Library<ExampleLayer>) -> (layir::Library<GdsLayer>, GdsUnits) {
    let mut olib = LibraryBuilder::<GdsLayer>::new();
    let cells = lib.topological_order();
    for cell in cells {
        let cell = lib.cell(cell);
        let mut ocell = Cell::new(cell.name());
        for elt in cell.elements() {
            ocell.add_element(elt.map_layer(ExampleLayer::gds_layer));
        }
        for (_, inst) in cell.instances() {
            let name = lib.cell(inst.child()).name();
            let child_id = olib.cell_id_named(name);
            ocell.add_instance(layir::Instance::new(child_id, inst.name()));
        }
        for (name, port) in cell.ports() {
            ocell.add_port(
                name,
                port.map_layer(|layer| ExampleLayer::gds_pin_layer(layer).unwrap()),
            );
        }
        olib.add_cell(ocell);
    }
    (olib.build().unwrap(), gds_units())
}

impl Schema for ExampleSchema {
    type Layer = ExampleLayer;
}

impl Layout for Inverter {
    type Schema = ExampleSchema;
    type Bundle = View<BufferIo, PortGeometryBundle<ExampleSchema>>;
    type Data = ();
    fn layout(
        &self,
        cell: &mut super::CellBuilder<Self::Schema>,
    ) -> crate::error::Result<(Self::Bundle, Self::Data)> {
        cell.draw(Shape::new(
            ExampleLayer::A,
            Rect::from_sides(0, 0, 100, 200),
        ))?;

        Ok((
            BufferIoView {
                din: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    Rect::from_sides(0, 75, 25, 125),
                )),
                dout: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    Rect::from_sides(75, 75, 100, 125),
                )),
                vdd: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    Rect::from_sides(25, 175, 75, 200),
                )),
                vss: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    Rect::from_sides(25, 0, 75, 25),
                )),
            },
            (),
        ))
    }
}

#[derive(TranslateMut, TransformMut, TranslateRef, TransformRef)]
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl Layout for Buffer {
    type Schema = ExampleSchema;
    type Bundle = View<BufferIo, PortGeometryBundle<ExampleSchema>>;
    type Data = BufferData;

    fn layout(
        &self,
        cell: &mut super::CellBuilder<Self::Schema>,
    ) -> crate::error::Result<(Self::Bundle, Self::Data)> {
        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone())?;
        cell.draw(inv2.clone())?;

        cell.draw(Shape::new(
            ExampleLayer::C,
            inv1.io()
                .dout
                .primary
                .bounding_union(&inv2.io().din.primary),
        ))?;

        Ok((
            CellBundle::<Self> {
                din: inv1.io().din,
                dout: inv1.io().dout,
                vdd: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    inv1.io().vdd.primary.bounding_union(&inv2.io().vdd.primary),
                )),
                vss: PortGeometry::new(Shape::new(
                    ExampleLayer::B,
                    inv1.io().vss.primary.bounding_union(&inv2.io().vss.primary),
                )),
            },
            BufferData { inv1, inv2 },
        ))
    }
}

#[derive(Default, TranslateMut, TransformMut, TranslateRef, TransformRef)]
pub struct BufferNData {
    pub buffers: Vec<Instance<Buffer>>,
}

impl Layout for BufferN {
    type Schema = ExampleSchema;
    type Bundle = View<BufferIo, PortGeometryBundle<ExampleSchema>>;
    type Data = BufferNData;
    fn layout(
        &self,
        cell: &mut super::CellBuilder<Self::Schema>,
    ) -> crate::error::Result<(Self::Bundle, Self::Data)> {
        let buffer = cell.generate(Buffer::new(self.strength));

        let mut tiler = ArrayTiler::new(TileAlignMode::PosAdjacent, TileAlignMode::Center);
        let buffers = tiler
            .push_num(
                Tile::from_bbox(buffer.clone()).with_padding(Sides::uniform(5)),
                self.n,
            )
            .iter()
            .map(|key| tiler[*key].clone())
            .collect::<Vec<_>>();

        let mut vdd = PortGeometryBuilder::new();
        let mut vss = PortGeometryBuilder::new();

        for i in 0..self.n {
            if i > 0 {
                cell.draw(Shape::new(
                    *buffers[i].io().dout.primary.layer(),
                    buffers[i]
                        .io()
                        .din
                        .primary
                        .bounding_union(&buffers[i - 1].io().dout),
                ))?;
            }

            vdd.merge(buffers[i].io().vdd);
            vss.merge(buffers[i].io().vss);
        }

        cell.draw(tiler)?;

        Ok((
            CellBundle::<Self> {
                din: buffers[0].io().din,
                dout: buffers[self.n - 1].io().dout,
                vdd: vdd.build().unwrap(),
                vss: vss.build().unwrap(),
            },
            BufferNData { buffers },
        ))
    }
}

impl Layout for BufferNxM {
    type Schema = ExampleSchema;
    type Bundle = View<BufferNxMIo, PortGeometryBundle<ExampleSchema>>;
    type Data = ();
    fn layout(
        &self,
        cell: &mut super::CellBuilder<Self::Schema>,
    ) -> crate::error::Result<(Self::Bundle, Self::Data)> {
        let buffern = cell.generate::<BufferN>(BufferN::new(self.strength, self.n));
        let mut tiler = ArrayTiler::new(TileAlignMode::Center, TileAlignMode::NegAdjacent);

        let mut vdd = PortGeometryBuilder::new();
        let mut vss = PortGeometryBuilder::new();
        let mut din = vec![PortGeometryBuilder::new(); self.io().din.len()];
        let mut dout = vec![PortGeometryBuilder::new(); self.io().dout.len()];

        for i in 0..self.m {
            let key = tiler.push(Tile::from_bbox(buffern.clone()).with_padding(Sides::uniform(10)));
            vdd.merge(tiler[key].io().vdd);
            vss.merge(tiler[key].io().vss);
            din[i].merge(tiler[key].io().din);
            dout[i].merge(tiler[key].io().dout);
        }

        cell.draw(tiler)?;

        cell.draw(Shape::new(
            ExampleLayer::C,
            cell.bbox().unwrap().expand_all(10),
        ))?;

        Ok((
            CellBundle::<Self> {
                din: ArrayBundle::new(
                    Signal,
                    din.into_iter().map(|b| b.build().unwrap()).collect(),
                ),
                dout: ArrayBundle::new(
                    Signal,
                    dout.into_iter().map(|b| b.build().unwrap()).collect(),
                ),
                vdd: vdd.build().unwrap(),
                vss: vss.build().unwrap(),
            },
            (),
        ))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Block, Hash)]
#[substrate(io = "()")]
pub struct GridTilerExample;

impl Layout for GridTilerExample {
    type Schema = ExampleSchema;
    type Bundle = ();
    type Data = ();

    fn layout(
        &self,
        cell: &mut super::CellBuilder<Self::Schema>,
    ) -> crate::error::Result<(Self::Bundle, Self::Data)> {
        let mut tiler = GridTiler::new();

        let tile1 = Tile::from_bbox(Shape::new(
            ExampleLayer::A,
            Rect::from_sides(0, 0, 100, 100),
        ))
        .with_padding(Sides::uniform(10));

        let tile2 = Tile::from_bbox(Shape::new(
            ExampleLayer::B,
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

        Ok(((), ()))
    }
}

#[test]
fn layout_generation_and_data_propagation_work() {
    let test_name = "layout_generation_and_data_propagation_work";

    let block = Buffer::new(5);

    let ctx = Context::new();
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

    ctx.write_layout(block, to_gds, get_path(test_name, "layout_pdk_a.gds"))
        .expect("failed to write layout");
}

#[test]
fn nested_transform_views_work() {
    let test_name = "nested_transform_views_work";

    let block = BufferN::new(5, 10);

    let ctx = Context::new();
    ctx.write_layout(block, to_gds, get_path(test_name, "layout.gds"))
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

    let ctx = Context::new();
    ctx.write_layout(block, to_gds, get_path(test_name, "layout.gds"))
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

    let ctx = Context::new();
    let block1 = ctx.generate_layout(block1);
    let block2 = ctx.generate_layout(block2);
    let block3 = ctx.generate_layout(block3);
    ctx.write_layout_all(
        [
            block1.cell().raw().as_ref(),
            block2.cell().raw().as_ref(),
            block3.cell().raw().as_ref(),
        ],
        to_gds,
        get_path(test_name, "layout.gds"),
    )
    .expect("failed to write layout");
}

#[test]
fn grid_tiler_works_with_various_spans() {
    let test_name = "grid_tiler_works_with_various_spans";

    let ctx = Context::new();
    ctx.write_layout(GridTilerExample, to_gds, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");
}
