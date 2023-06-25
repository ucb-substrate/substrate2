//! A layout cell builder.

use std::marker::PhantomData;

use crate::error::Result;
use crate::{context::Context, pdk::Pdk};

use super::HasLayoutImpl;
use super::{
    cell::Instance,
    draw::DrawContainer,
    element::{CellId, Element, RawCell, Shape},
};

/// A layout cell builder.
///
/// Constructed once for each invocation of [`HasLayoutImpl::layout`].
pub struct CellBuilder<PDK, T> {
    phantom: PhantomData<T>,
    cell: RawCell,
    context: Context<PDK>,
}

impl<PDK: Pdk, T> CellBuilder<PDK, T> {
    pub(crate) fn new(id: CellId, context: Context<PDK>) -> Self {
        Self {
            phantom: PhantomData,
            cell: RawCell::new(id),
            context,
        }
    }

    pub(crate) fn into_cell(self) -> RawCell {
        self.cell
    }

    /// Generate an instance of `block`.
    ///
    /// Returns immediately, allowing generation to complete in the background. Attempting to
    /// acceess the generated instance's cell will block until generation is complete.
    ///
    /// # Examples
    ///
    /// ```
    #[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/layout/buffer.md")]
    /// ```
    pub fn generate<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.context.generate_layout(block);
        Instance::new(cell)
    }

    /// Generate an instance of `block`.
    ///
    /// Blocks on generation, returning only once the instance's cell is populated. Useful for
    /// handling errors thrown by the generation of a cell immediately.
    pub fn generate_blocking<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Result<Instance<I>> {
        let cell = self.context.generate_layout(block);
        let res = cell.wait().as_ref().map(|_| ()).map_err(|e| e.clone());
        res.map(|_| Instance::new(cell))
    }
}

impl<PDK, T> DrawContainer for CellBuilder<PDK, T> {
    fn draw_element(&mut self, element: Element) {
        self.cell.draw_element(element);
    }

    fn draw_blockage(&mut self, shape: Shape) {
        self.cell.draw_blockage(shape);
    }
}
