//! Substrate's layout generator framework.
use std::{marker::PhantomData, sync::Arc};

use arcstr::ArcStr;
use geometry::{
    prelude::{Bbox, Orientation, Point},
    transform::{Transform, TransformMut, Transformation, TranslateMut},
};
use once_cell::sync::OnceCell;

///
/// # Examples
///
/// ## Simple
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md")]
/// ```
///
/// ## With data
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/buffer.md")]
/// ```
use crate::block::Block;
use crate::generator::Generator;
use crate::io::LayoutType;
use crate::pdk::Pdk;
use crate::{context::Context, error::Result};

use self::{
    draw::{Draw, DrawContainer},
    element::{CellId, Element, RawCell, RawInstance, Shape},
};

pub mod draw;
pub mod element;
pub mod error;
pub mod gds;

/// A block that has a layout.
pub trait HasLayout: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Send + Sync;
}

/// A block that has a layout for process design kit `PDK`.
pub trait HasLayoutImpl<PDK: Pdk>: HasLayout {
    /// Generates the block's layout.
    fn layout(
        &self,
        io: &mut <<Self as Block>::Io as LayoutType>::Builder,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> Result<Self::Data>;
}

/// Layout-specific context data.
///
/// Stores generated layout cells as well as state used for assigning unique cell IDs.
#[derive(Debug, Default, Clone)]
pub struct LayoutContext {
    next_id: CellId,
    pub(crate) gen: Generator,
}

impl LayoutContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_id(&mut self) -> CellId {
        self.next_id.increment();
        self.next_id
    }
}

/// A generic layout cell.
///
/// Stores its underlying block, extra data created during generation, as well as a raw cell
/// containing its primitive elements.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/generate.md")]
/// ```
#[derive(Default, Clone)]
#[allow(dead_code)]
pub struct Cell<T: HasLayout> {
    /// Block whose layout this cell represents.
    pub block: T,
    /// Extra data created during layout generation.
    pub data: T::Data,
    pub(crate) raw: Arc<RawCell>,
}

impl<T: HasLayout> Cell<T> {
    pub(crate) fn new(block: T, data: T::Data, raw: Arc<RawCell>) -> Self {
        Self { block, data, raw }
    }
}

impl<T: HasLayout> Bbox for Cell<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox()
    }
}

/// A generic layout instance.
///
/// Stores a pointer to its underlying cell and its instantiated location and orientation.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Instance<T: HasLayout> {
    cell: Arc<OnceCell<Result<Cell<T>>>>,
    pub(crate) loc: Point,
    pub(crate) orientation: Orientation,
}

impl<T: HasLayout> Instance<T> {
    pub(crate) fn new(cell: Arc<OnceCell<Result<Cell<T>>>>) -> Self {
        Instance {
            cell,
            loc: Point::default(),
            orientation: Orientation::default(),
        }
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> &Result<Cell<T>> {
        self.cell.wait()
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> &Cell<T> {
        self.try_cell().as_ref().unwrap()
    }

    /// Returns the current transformation of `self`.
    pub fn transformation(&self) -> Transformation {
        Transformation::from_offset_and_orientation(self.loc, self.orientation)
    }
}

impl<T: HasLayout> Bbox for Instance<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.cell()
            .bbox()
            .map(|rect| rect.transform(self.transformation()))
    }
}

impl<T: HasLayout> TranslateMut for Instance<T> {
    fn translate_mut(&mut self, p: Point) {
        self.loc.translate_mut(p);
    }
}

impl<T: HasLayout> TransformMut for Instance<T> {
    fn transform_mut(&mut self, trans: Transformation) {
        let new_transform = Transformation::cascade(self.transformation(), trans);
        self.loc = new_transform.offset_point();
        self.orientation = new_transform.orientation();
    }
}

impl<I: HasLayout> Draw for Instance<I> {
    fn draw<T: DrawContainer + ?Sized>(self, container: &mut T) {
        RawInstance::from(self).draw(container);
    }
}

/// A layout cell builder.
///
/// Constructed once for each invocation of [`HasLayoutImpl::layout`].
pub struct CellBuilder<PDK: Pdk, T> {
    phantom: PhantomData<T>,
    cell: RawCell,
    /// The current global context.
    pub ctx: Context<PDK>,
}

impl<PDK: Pdk, T> CellBuilder<PDK, T> {
    pub(crate) fn new(id: CellId, name: ArcStr, ctx: Context<PDK>) -> Self {
        Self {
            phantom: PhantomData,
            cell: RawCell::new(id, name),
            ctx,
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
    #[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
    #[doc = include_str!("../../../docs/api/code/layout/buffer.md")]
    /// ```
    pub fn generate<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_layout(block);
        Instance::new(cell)
    }

    /// Generate an instance of `block`.
    ///
    /// Blocks on generation, returning only once the instance's cell is populated. Useful for
    /// handling errors thrown by the generation of a cell immediately.
    pub fn generate_blocking<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Result<Instance<I>> {
        let cell = self.ctx.generate_layout(block);
        let res = cell.wait().as_ref().map(|_| ()).map_err(|e| e.clone());
        res.map(|_| Instance::new(cell))
    }
}

impl<PDK: Pdk, T> DrawContainer for CellBuilder<PDK, T> {
    fn draw_element(&mut self, element: Element) {
        self.cell.draw_element(element);
    }

    fn draw_blockage(&mut self, shape: Shape) {
        self.cell.draw_blockage(shape);
    }
}
