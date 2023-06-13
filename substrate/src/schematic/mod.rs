use self::{
    cell::SchematicCell, context::SchematicCtx, instance::SchematicInstance,
    interface::AnalogInterface,
};
use crate::block::Block;

pub mod cell;
pub mod context;
pub mod instance;
pub mod interface;

pub trait HasSchematic: Block {
    type Interface: AnalogInterface<Self>;
    type Cell: SchematicCell<Self>;
    type Instance: SchematicInstance<Self>;

    fn schematic(&self, ctx: &mut SchematicCtx) -> Self::Cell;
}
