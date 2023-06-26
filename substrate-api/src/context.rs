//! The global context.

use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use crate::error::Result;
use crate::layout::builder::CellBuilder as LayoutCellBuilder;
use crate::layout::cell::Cell as LayoutCell;
use crate::layout::context::LayoutContext;
use crate::layout::HasLayoutImpl;
use crate::pdk::Pdk;
use crate::schematic::Cell as SchematicCell;
use crate::schematic::{CellBuilder as SchematicCellBuilder, HardwareType, NodeContext};
use crate::schematic::{HasSchematicImpl, SchematicContext};

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/generate.md")]
/// ```
#[derive(Debug)]
pub struct Context<PDK> {
    pdk: Arc<PDK>,
    inner: Arc<RwLock<ContextInner>>,
}

impl<PDK> Clone for Context<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ContextInner {
    schematic: SchematicContext,
    layout: LayoutContext,
}

impl<PDK: Pdk> Context<PDK> {
    /// Creates a new global context.
    pub fn new(pdk: PDK) -> Self {
        Self {
            pdk: Arc::new(pdk),
            inner: Default::default(),
        }
    }

    /// Generates a layout for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: HasLayoutImpl<PDK>>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<LayoutCell<T>>>> {
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.layout.get_id();
        inner_mut.layout.gen.generate(block.clone(), move || {
            let mut cell_builder = LayoutCellBuilder::new(id, context_clone);
            let data = block.layout(&mut cell_builder);
            data.map(|data| LayoutCell::new(block, data, Arc::new(cell_builder.into())))
        })
    }

    /// Generates a schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_schematic<T: HasSchematicImpl<PDK>>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<SchematicCell<T>>>> {
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.schematic.get_id();
        inner_mut.schematic.gen.generate(block.clone(), move || {
            let mut node_ctx = NodeContext::new();
            let io = block.io();
            let nodes = node_ctx.nodes(io.num_signals() as usize);
            let (io, nodes) = io.instantiate(&nodes);
            assert!(nodes.is_empty());
            let mut cell_builder = SchematicCellBuilder {
                id,
                ctx: context_clone,
                node_ctx,
                instances: Vec::new(),
                phantom: PhantomData,
            };
            let data = block.schematic(io, &mut cell_builder);
            data.map(|data| SchematicCell::new(block, data, Arc::new(cell_builder.finish())))
        })
    }
}

impl ContextInner {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
