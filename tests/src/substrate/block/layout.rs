use geometry::{
    align::AlignBboxMut,
    prelude::{AlignBbox, AlignMode, Bbox},
    rect::Rect,
    union::BoundingUnion,
};

use substrate::{
    layout::{draw::DrawContainer, element::Shape, HasLayout, HasLayoutImpl, Instance},
    pdk::{
        layers::{Layer, LayerInfo},
        PdkLayers,
    },
    supported_pdks, Layers, LayoutData,
};
use tracing::{span, Level};

use crate::substrate::pdk::{ExamplePdkA, ExamplePdkB};

use super::{Buffer, BufferIo, BufferN, Inverter};

impl HasLayout for Inverter {
    type Data = ();
}

impl HasLayoutImpl<ExamplePdkA> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(
            cell.ctx.pdk.layers.polya,
            Rect::from_sides(0, 0, 100, 200),
        ));

        io.din.push(Shape::new(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(0, 75, 25, 125),
        ));

        io.dout.push(Shape::new(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(75, 75, 100, 125),
        ));

        io.vdd.push(Shape::new(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(25, 175, 75, 200),
        ));

        io.vss.push(Shape::new(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(25, 0, 75, 25),
        ));

        Ok(())
    }
}

impl HasLayoutImpl<ExamplePdkB> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkB, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(
            cell.ctx.pdk.layers.polyb,
            Rect::from_sides(0, 0, 200, 100),
        ));

        io.din.push(Shape::new(
            cell.ctx.pdk.layers.met1b,
            Rect::from_sides(0, 25, 25, 75),
        ));

        io.dout.push(Shape::new(
            cell.ctx.pdk.layers.met1b,
            Rect::from_sides(175, 25, 200, 75),
        ));

        io.vdd.push(Shape::new(
            cell.ctx.pdk.layers.met1b,
            Rect::from_sides(75, 75, 125, 100),
        ));

        io.vss.push(Shape::new(
            cell.ctx.pdk.layers.met1b,
            Rect::from_sides(75, 0, 125, 25),
        ));

        Ok(())
    }
}

pub struct DerivedLayers {
    m1: LayerInfo,
    #[allow(dead_code)]
    m2: LayerInfo,
}

impl From<&PdkLayers<ExamplePdkA>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkA>) -> Self {
        Self {
            m1: value.met1a.info(),
            m2: value.met2a.info(),
        }
    }
}

impl From<&PdkLayers<ExamplePdkB>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkB>) -> Self {
        Self {
            m1: value.met1b.info(),
            m2: value.met2b.info(),
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
    #[transform]
    pub inv1: Instance<Inverter>,
    #[transform]
    pub inv2: Instance<Inverter>,
}

impl HasLayout for Buffer {
    type Data = BufferData;
}

#[supported_pdks(ExamplePdkA, ExamplePdkB)]
impl HasLayoutImpl<T> for Buffer {
    fn layout(
        &self,
        io: &mut super::BufferIoLayoutBuilder,
        cell: &mut substrate::layout::CellBuilder<T, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let derived_layers = DerivedLayers::from(cell.ctx.pdk.layers.as_ref());
        let installed_layers = cell.ctx.install_layers::<ExtraLayers>();

        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone());
        cell.draw(inv2.clone());

        cell.draw(Shape::new(
            &derived_layers.m2,
            inv1.io()
                .dout
                .primary
                .bounding_union(&inv2.io().din.primary),
        ));

        io.din.push(inv1.io().din.primary.clone());
        io.dout.push(inv2.io().dout.primary.clone());

        io.vdd.push(Shape::new(
            &derived_layers.m1,
            inv1.io().vdd.primary.bounding_union(&inv2.io().vdd.primary),
        ));

        io.vss.push(Shape::new(
            &derived_layers.m1,
            inv1.io().vss.primary.bounding_union(&inv2.io().vss.primary),
        ));

        cell.draw(Shape::new(
            installed_layers.marker1,
            inv1.bbox().bounding_union(&inv2.bbox()).unwrap(),
        ));

        Ok(BufferData { inv1, inv2 })
    }
}

#[derive(Default, LayoutData)]
pub struct BufferNData {
    #[transform]
    pub buffers: Vec<Instance<Buffer>>,
}

impl HasLayout for BufferN {
    type Data = BufferNData;
}

#[supported_pdks(ExamplePdkA, ExamplePdkB)]
impl HasLayoutImpl<T> for BufferN {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<T, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut buffer = cell.generate(Buffer::new(self.strength));

        let mut data = BufferNData::default();

        cell.draw(buffer.clone());
        data.buffers.push(buffer.clone());

        for i in 1..self.n {
            buffer.align_bbox_mut(AlignMode::ToTheRight, buffer.bbox(), 10);
            cell.draw(buffer.clone());
            data.buffers.push(buffer.clone());

            cell.draw(Shape::new(
                buffer.io().dout.primary.layer(),
                buffer
                    .io()
                    .din
                    .primary
                    .bounding_union(&data.buffers[i - 1].io().dout.primary),
            ));

            io.vdd.push(buffer.io().vdd.primary.clone());
            io.vss.push(buffer.io().vss.primary.clone());
        }

        io.din.push(data.buffers[0].io().din.primary.clone());
        io.dout
            .push(data.buffers[self.n - 1].io().dout.primary.clone());

        Ok(data)
    }
}
