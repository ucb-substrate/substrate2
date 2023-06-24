//! The global context.

use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use crate::error::Result;
use crate::layout::builder::CellBuilder;
use crate::layout::context::LayoutContext;
use crate::layout::{cell::Cell, HasLayout};

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
#[derive(Debug, Default, Clone)]
pub struct Context {
    inner: Arc<RwLock<ContextInner>>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ContextInner {
    layout: LayoutContext,
}

impl Context {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn generate_layout<T: HasLayout>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<Cell<T>>>> {
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.layout.get_id();
        inner_mut.layout.gen.generate(block.clone(), move || {
            let mut cell_builder = CellBuilder::new(id, context_clone);
            let data = block.layout(&mut cell_builder);
            data.map(|data| Cell::new(block, data, Arc::new(cell_builder.into())))
        })
    }
}

impl ContextInner {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
