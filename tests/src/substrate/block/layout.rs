use geometry::{
    prelude::{AlignBbox, AlignMode},
    rect::Rect,
};

use substrate::{
    layout::{cell::Instance, draw::DrawContainer, element::Shape, HasLayout, HasLayoutImpl},
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
        cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));

        Ok(())
    }
}

impl HasLayoutImpl<ExamplePdkB> for Inverter {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<ExamplePdkB, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(Rect::from_sides(0, 0, 200, 100)));

        Ok(())
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
        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone());
        cell.draw(inv2.clone());

        Ok(BufferData { inv1, inv2 })
    }
}
