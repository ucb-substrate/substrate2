use geometry::{
    prelude::{AlignBbox, AlignMode},
    rect::Rect,
};

use crate::layout::{cell::Instance, draw::DrawContainer, element::Shape, HasLayout};

use super::{Buffer, Inverter};

impl HasLayout for Inverter {
    type Data = ();

    fn layout(
        &self,
        cell: &mut crate::layout::builder::CellBuilder<Self>,
    ) -> crate::error::Result<Self::Data> {
        cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));

        Ok(())
    }
}

pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl HasLayout for Buffer {
    type Data = BufferData;

    fn layout(
        &self,
        cell: &mut crate::layout::builder::CellBuilder<Self>,
    ) -> crate::error::Result<Self::Data> {
        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone());
        cell.draw(inv2.clone());

        Ok(BufferData { inv1, inv2 })
    }
}
