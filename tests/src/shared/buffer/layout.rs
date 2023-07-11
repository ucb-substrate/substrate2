use geometry::{
    align::AlignBboxMut,
    prelude::{AlignBbox, AlignMode, Bbox},
    rect::Rect,
    union::BoundingUnion,
};

use substrate::{
    io::IoShape,
    layout::{element::Shape, HasLayout, HasLayoutImpl, Instance},
    pdk::{layers::HasPin, PdkLayers},
    supported_pdks, DerivedLayerFamily, DerivedLayers, Layers, LayoutData,
};

use crate::shared::pdk::{ExamplePdkA, ExamplePdkB};

use super::{Buffer, BufferN, BufferNxM, Inverter};

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
        ))?;

        io.din.set(IoShape::with_layers(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(0, 75, 25, 125),
        ));

        io.dout.set(IoShape::with_layers(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(75, 75, 100, 125),
        ));

        io.vdd.set(IoShape::with_layers(
            cell.ctx.pdk.layers.met1a,
            Rect::from_sides(25, 175, 75, 200),
        ));

        io.vss.set(IoShape::with_layers(
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
        ))?;

        io.din.set(IoShape::new(
            cell.ctx.pdk.layers.met1b,
            cell.ctx.pdk.layers.met1b_pin,
            cell.ctx.pdk.layers.met1b_label,
            Rect::from_sides(0, 25, 25, 75),
        ));

        io.dout.set(IoShape::new(
            cell.ctx.pdk.layers.met1b,
            cell.ctx.pdk.layers.met1b_pin,
            cell.ctx.pdk.layers.met1b_label,
            Rect::from_sides(175, 25, 200, 75),
        ));

        io.vdd.set(IoShape::new(
            cell.ctx.pdk.layers.met1b,
            cell.ctx.pdk.layers.met1b_pin,
            cell.ctx.pdk.layers.met1b_label,
            Rect::from_sides(75, 75, 125, 100),
        ));

        io.vss.set(IoShape::new(
            cell.ctx.pdk.layers.met1b,
            cell.ctx.pdk.layers.met1b_pin,
            cell.ctx.pdk.layers.met1b_label,
            Rect::from_sides(75, 0, 125, 25),
        ));

        Ok(())
    }
}

#[derive(DerivedLayers)]
pub struct DerivedLayers {
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

impl From<&PdkLayers<ExamplePdkA>> for DerivedLayers {
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

impl From<&PdkLayers<ExamplePdkB>> for DerivedLayers {
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
    #[substrate(transform)]
    pub inv1: Instance<Inverter>,
    #[substrate(transform)]
    pub inv2: Instance<Inverter>,
}

impl HasLayout for Buffer {
    type Data = BufferData;
}

#[supported_pdks(ExamplePdkA, ExamplePdkB)]
impl HasLayoutImpl<T> for Buffer {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<T, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let derived_layers = DerivedLayers::from(cell.ctx.pdk.layers.as_ref());
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
    #[substrate(transform)]
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

        cell.draw(buffer.clone())?;
        data.buffers.push(buffer.clone());

        for i in 1..self.n {
            buffer.align_bbox_mut(AlignMode::ToTheRight, buffer.bbox(), 10);
            cell.draw(buffer.clone())?;
            data.buffers.push(buffer.clone());

            cell.draw(Shape::new(
                buffer.io().dout.layer().drawing(),
                buffer
                    .io()
                    .din
                    .bounding_union(&data.buffers[i - 1].io().dout),
            ))?;

            io.vdd.push(buffer.io().vdd.clone());
            io.vss.push(buffer.io().vss.clone());
        }

        io.din.set(data.buffers[0].io().din);
        io.dout.set(data.buffers[self.n - 1].io().dout);

        Ok(data)
    }
}

impl HasLayout for BufferNxM {
    type Data = ();
}

#[supported_pdks(ExamplePdkA, ExamplePdkB)]
impl HasLayoutImpl<T> for BufferNxM {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<T, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut buffern = cell.generate(BufferN::new(self.strength, self.n));

        for i in 0..self.n {
            if i != 0 {
                buffern.align_bbox_mut(AlignMode::Beneath, buffern.bbox(), 20);
            }

            io.vdd.merge(buffern.io().vdd);
            io.vss.merge(buffern.io().vss);
            io.din[i].set(buffern.io().din);
            io.dout[i].set(buffern.io().dout);

            cell.draw(buffern.clone())?;
        }

        Ok(())
    }
}
