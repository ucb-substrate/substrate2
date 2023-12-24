use geometry::{
    prelude::{AlignBbox, AlignMode, Bbox},
    rect::Rect,
    side::Sides,
    union::BoundingUnion,
};

use substrate::io::layout::IoShape;
use substrate::pdk::Pdk;
use substrate::{
    layout::{
        element::Shape,
        tiling::{ArrayTiler, Tile, TileAlignMode},
        ExportsLayoutData, Instance, Layout, LayoutData,
    },
    pdk::layers::{DerivedLayerFamily, DerivedLayers, HasPin, Layers},
    pdk::PdkLayers,
};

use crate::shared::pdk::{ExamplePdkA, ExamplePdkB};

use super::{Buffer, BufferN, BufferNxM, Inverter};

impl ExportsLayoutData for Inverter {
    type LayoutData = ();
}

impl Layout<ExamplePdkA> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::LayoutData> {
        cell.draw(Shape::new(
            cell.ctx.layers.polya,
            Rect::from_sides(0, 0, 100, 200),
        ))?;

        io.din.set(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(0, 75, 25, 125),
        ));

        io.dout.set(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(75, 75, 100, 125),
        ));

        io.vdd.set(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(25, 175, 75, 200),
        ));

        io.vss.set(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(25, 0, 75, 25),
        ));

        Ok(())
    }
}

impl Layout<ExamplePdkB> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::LayoutData> {
        cell.draw(Shape::new(
            cell.ctx.layers.polyb,
            Rect::from_sides(0, 0, 200, 100),
        ))?;

        io.din.set(IoShape::new(
            cell.ctx.layers.met1b,
            cell.ctx.layers.met1b_pin,
            cell.ctx.layers.met1b_label,
            Rect::from_sides(0, 25, 25, 75),
        ));

        io.dout.set(IoShape::new(
            cell.ctx.layers.met1b,
            cell.ctx.layers.met1b_pin,
            cell.ctx.layers.met1b_label,
            Rect::from_sides(175, 25, 200, 75),
        ));

        io.vdd.set(IoShape::new(
            cell.ctx.layers.met1b,
            cell.ctx.layers.met1b_pin,
            cell.ctx.layers.met1b_label,
            Rect::from_sides(75, 75, 125, 100),
        ));

        io.vss.set(IoShape::new(
            cell.ctx.layers.met1b,
            cell.ctx.layers.met1b_pin,
            cell.ctx.layers.met1b_label,
            Rect::from_sides(75, 0, 125, 25),
        ));

        Ok(())
    }
}

#[derive(DerivedLayers)]
pub struct BufferDerivedLayers {
    #[layer_family]
    m1: M1,
    m2: M2,
}

#[derive(DerivedLayerFamily, Clone, Copy)]
pub struct M1 {
    #[layer(primary)]
    pub drawing: M1Drawing,
    #[layer(pin)]
    pub pin: M1Pin,
    #[layer(label)]
    pub label: M1Label,
}

impl From<&PdkLayers<ExamplePdkA>> for BufferDerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkA>) -> Self {
        Self {
            m1: M1 {
                drawing: M1Drawing::new(value.met1a.drawing),
                pin: M1Pin::new(value.met1a.pin),
                label: M1Label::new(value.met1a.label),
            },
            m2: M2::new(value.met2a),
        }
    }
}

impl From<&PdkLayers<ExamplePdkB>> for BufferDerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkB>) -> Self {
        Self {
            m1: M1 {
                drawing: M1Drawing::new(value.met1b),
                pin: M1Pin::new(value.met1b_pin),
                label: M1Label::new(value.met1b_label),
            },
            m2: M2::new(value.met2b),
        }
    }
}

#[derive(Layers)]
pub struct ExtraLayers {
    marker1: Marker1,
    marker2: Marker2,
}

#[derive(LayoutData)]
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl ExportsLayoutData for Buffer {
    type LayoutData = BufferData;
}

impl<PDK: Pdk> Layout<PDK> for Buffer
where
    for<'a> &'a PDK::Layers: Into<BufferDerivedLayers>,
    Inverter: Layout<PDK>,
{
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let derived_layers: BufferDerivedLayers = cell.ctx.layers.as_ref().into();
        let installed_layers = cell.ctx.install_layers::<ExtraLayers>();

        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone())?;
        cell.draw(inv2.clone())?;

        cell.draw(Shape::new(
            derived_layers.m2,
            inv1.io().dout.bounding_union(&inv2.io().din),
        ))?;

        io.din.set(inv1.io().din);
        io.dout.set(inv2.io().dout);

        io.vdd.set(IoShape::with_layers(
            derived_layers.m1,
            inv1.io().vdd.bounding_union(&inv2.io().vdd),
        ));

        io.vss.set(IoShape::with_layers(
            derived_layers.m1,
            inv1.io().vss.bounding_union(&inv2.io().vss),
        ));

        cell.draw(Shape::new(
            installed_layers.marker1,
            inv1.bbox().bounding_union(&inv2.bbox()).unwrap(),
        ))?;

        Ok(BufferData { inv1, inv2 })
    }
}

#[derive(Default, LayoutData)]
pub struct BufferNData {
    pub buffers: Vec<Instance<Buffer>>,
}

impl ExportsLayoutData for BufferN {
    type LayoutData = BufferNData;
}

impl<PDK: Pdk> Layout<PDK> for BufferN
where
    Buffer: Layout<PDK>,
{
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let buffer = cell.generate(Buffer::new(self.strength));

        let mut data = BufferNData::default();
        let mut tiler = ArrayTiler::new(TileAlignMode::PosAdjacent, TileAlignMode::Center);
        let buffers = tiler
            .push_num(
                Tile::from_bbox(buffer.clone()).with_padding(Sides::uniform(5)),
                self.n,
            )
            .iter()
            .map(|key| tiler[*key].clone())
            .collect::<Vec<_>>();

        for i in 0..self.n {
            if i > 0 {
                cell.draw(Shape::new(
                    buffers[i].io().dout.layer().drawing(),
                    buffers[i]
                        .io()
                        .din
                        .bounding_union(&buffers[i - 1].io().dout),
                ))?;
            }

            io.vdd.push(buffers[i].io().vdd.clone());
            io.vss.push(buffers[i].io().vss.clone());
        }

        io.din.set(buffers[0].io().din);
        io.dout.set(buffers[self.n - 1].io().dout);

        data.buffers = buffers;

        cell.draw(tiler)?;

        Ok(data)
    }
}

impl ExportsLayoutData for BufferNxM {
    type LayoutData = ();
}

impl<PDK: Pdk> Layout<PDK> for BufferNxM
where
    for<'a> &'a PDK::Layers: Into<BufferDerivedLayers>,
    BufferN: Layout<PDK>,
{
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::layout::HardwareType>::Builder,
        cell: &mut substrate::layout::CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let derived_layers: BufferDerivedLayers = cell.ctx.layers.as_ref().into();
        let buffern = cell.generate::<BufferN>(BufferN::new(self.strength, self.n));
        let mut tiler = ArrayTiler::new(TileAlignMode::Center, TileAlignMode::NegAdjacent);

        for i in 0..self.m {
            let key = tiler.push(Tile::from_bbox(buffern.clone()).with_padding(Sides::uniform(10)));
            io.vdd.merge(tiler[key].io().vdd);
            io.vss.merge(tiler[key].io().vss);
            io.din[i].set(tiler[key].io().din);
            io.dout[i].set(tiler[key].io().dout);
        }

        cell.draw(tiler)?;

        cell.draw(Shape::new(
            derived_layers.m2,
            cell.bbox().unwrap().expand_all(10),
        ))?;
        Ok(())
    }
}
