# use serde::{Serialize, Deserialize};
# use geometry::prelude::*;
# use substrate::block::Block;
# use substrate::layout::{cell::Instance, draw::DrawContainer, element::Shape, HasLayout};
# #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
# pub struct Inverter {
#     strength: usize,
# }
# impl Inverter {
#     pub fn new(strength: usize) -> Self {
#         Self { strength }
#     }
# }
# impl Block for Inverter {
#     fn id() -> arcstr::ArcStr {
#         arcstr::literal!("inverter")
#     }
#     fn name(&self) -> arcstr::ArcStr {
#         arcstr::format!("inverter_{}", self.strength)
#     }
# }
# #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
# pub struct Buffer {
#     strength: usize,
# }
#
# impl Buffer {
#     pub fn new(strength: usize) -> Self {
#         Self { strength }
#     }
# }
#
# impl Block for Buffer {
#     fn id() -> arcstr::ArcStr {
#         arcstr::literal!("buffer")
#     }
#
#     fn name(&self) -> arcstr::ArcStr {
#         arcstr::format!("buffer_{}", self.strength)
#     }
# }
# impl HasLayout for Inverter {
#     type Data = ();
#     fn layout(
#         &self,
#         cell: &mut substrate::layout::builder::CellBuilder<Self>,
#     ) -> substrate::error::Result<Self::Data> {
#         cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));
#         Ok(())
#     }
# }
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl HasLayout for Buffer {
    type Data = BufferData;

    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<Self>,
    ) -> substrate::error::Result<Self::Data> {
        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone());
        cell.draw(inv2.clone());

        Ok(BufferData { inv1, inv2 })
    }
}
