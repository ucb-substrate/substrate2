//! The global context.

use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use crate::error::Result;
use crate::layout::builder::CellBuilder;
use crate::layout::cell::Cell;
use crate::layout::context::LayoutContext;
use crate::layout::HasLayoutImpl;
use crate::pdk::layers::LayerContext;
use crate::pdk::layers::Layers;
use crate::pdk::Pdk;

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layer.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/generate.md")]
/// ```
#[derive(Debug)]
pub struct Context<PDK: Pdk> {
    pdk: Arc<PDK>,
    /// PDK-specific layers and associated data.
    pub layers: Arc<PDK::Layers>,
    inner: Arc<RwLock<ContextInner>>,
}

impl<PDK: Pdk> Clone for Context<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            layers: self.layers.clone(),
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ContextInner {
    layout: LayoutContext,
}

impl<PDK: Pdk> Context<PDK> {
    /// Creates a new global context.
    pub fn new(pdk: PDK) -> Self {
        // Instantiate PDK layers.
        let mut layer_ctx = LayerContext::new();
        let layers = Arc::new(PDK::Layers::new(&mut layer_ctx));

        Self {
            pdk: Arc::new(pdk),
            layers,
            inner: Default::default(),
        }
    }

    /// Generates a cell for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: HasLayoutImpl<PDK>>(
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
