//! A layout cell builder.

use std::marker::PhantomData;

use crate::context::Context;
use crate::error::Result;

use super::{
    cell::Instance,
    draw::DrawContainer,
    element::{CellId, Element, RawCell, Shape},
    HasLayout,
};

/// A layout cell builder.
///
/// Constructed once for each invocation of [`HasLayout::layout`].
pub struct CellBuilder<T> {
    phantom: PhantomData<T>,
    cell: RawCell,
    context: Context,
}

impl<T> CellBuilder<T> {
    pub(crate) fn new(id: CellId, context: Context) -> Self {
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
    #[doc = include_str!("../../docs/layout/buffer.md")]
    /// ```
    pub fn generate<I: HasLayout>(&mut self, block: I) -> Instance<I> {
        let cell = self.context.generate_layout(block);
        Instance::new(cell)
    }

    /// Generate an instance of `block`.
    ///
    /// Blocks on generation, returning only once the instance's cell is populated. Useful for
    /// handling errors thrown by the generation of a cell immediately.
    pub fn generate_blocking<I: HasLayout>(&mut self, block: I) -> Result<Instance<I>> {
        let cell = self.context.generate_layout(block);
        let res = cell.wait().as_ref().map(|_| ()).map_err(|e| e.clone());
        res.map(|_| Instance::new(cell))
    }
}

impl<T> DrawContainer for CellBuilder<T> {
    fn draw_element(&mut self, element: Element) {
        self.cell.draw_element(element);
    }

    fn draw_blockage(&mut self, shape: Shape) {
        self.cell.draw_blockage(shape);
    }
}
