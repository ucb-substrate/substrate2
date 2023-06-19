use self::{cell::SchematicCell, context::SchematicCtx, interface::AnalogInterface};
use crate::block::Block;

pub mod cell;
pub mod context;
pub mod instance;
pub mod interface;
pub mod parallel;

pub trait HasSchematic: Block {
    type Interface: AnalogInterface<Self>;

    fn schematic(&self, ctx: &mut SchematicCtx) -> SchematicCell<Self>;
}
