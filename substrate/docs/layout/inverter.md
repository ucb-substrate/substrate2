# use serde::{Serialize, Deserialize};
# use geometry::prelude::Rect;
# use substrate::block::Block;
# use substrate::layout::{draw::DrawContainer, element::Shape, HasLayout};
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
impl HasLayout for Inverter {
    type Data = ();

    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));
        Ok(())
    }
}
