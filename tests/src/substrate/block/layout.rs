use geometry::{
    prelude::{AlignBbox, AlignMode, Bbox},
    rect::Rect,
    union::BoundingUnion,
};

use substrate::{
    layout::{cell::Instance, draw::DrawContainer, element::Shape, HasLayout, HasLayoutImpl},
    pdk::{layers::LayerId, PdkLayers},
    supported_pdks,
};

use crate::substrate::pdk::{ExamplePdkA, ExamplePdkB};

use super::{Buffer, Inverter};

impl HasLayout for Inverter {
    type Data = ();
}

impl HasLayoutImpl<ExamplePdkA> for Inverter {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(
            &cell.ctx.layers.polya,
            Rect::from_sides(0, 0, 100, 200),
        ));

        Ok(())
    }
}

impl HasLayoutImpl<ExamplePdkB> for Inverter {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<ExamplePdkB, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(
            &cell.ctx.layers.polyb,
            Rect::from_sides(0, 0, 200, 100),
        ));

        Ok(())
    }
}

pub struct DerivedLayers {
    m1: LayerId,
    #[allow(dead_code)]
    m2: LayerId,
}

impl From<&PdkLayers<ExamplePdkA>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkA>) -> Self {
        Self {
            m1: value.met1a.id,
            m2: value.met2a.id,
        }
    }
}

impl From<&PdkLayers<ExamplePdkB>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkB>) -> Self {
        Self {
            m1: value.met1b.id,
            m2: value.met2b.id,
        }
    }
}

pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl HasLayout for Buffer {
    type Data = BufferData;
}

#[supported_pdks(ExamplePdkA, ExamplePdkB)]
impl HasLayoutImpl<T> for Buffer {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<T, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let layers = DerivedLayers::from(cell.ctx.layers.as_ref());

        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone());
        cell.draw(inv2.clone());

        cell.draw(Shape::new(
            layers.m1,
            inv1.bbox().bounding_union(&inv2.bbox()).unwrap(),
        ));

        Ok(BufferData { inv1, inv2 })
    }
}
