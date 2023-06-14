use std::sync::Arc;

use super::{cell::SchematicCell, HasSchematic};

#[derive(Default)]
pub struct SchematicCtx {}

impl SchematicCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate<T>(&mut self, block: T) -> Arc<SchematicCell<T>>
    where
        T: HasSchematic,
    {
        let cell = block.schematic(self);
        Arc::new(cell)
    }
}
