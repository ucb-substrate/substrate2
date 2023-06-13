use std::sync::Arc;

use arcstr::ArcStr;
use slotmap::{new_key_type, SlotMap};

use super::{
    cell::SchematicCell,
    instance::SchematicInstance,
    interface::{AnalogInterface, Port},
    HasSchematic,
};

#[derive(Default)]
pub struct SchematicCtx {}

impl SchematicCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate<T>(&mut self, name: impl Into<ArcStr>, block: T) -> T::Instance
    where
        T: HasSchematic,
    {
        let cell = block.schematic(self);
        T::Instance::new(name, cell.interface().clone(), Arc::new(cell))
    }
}
