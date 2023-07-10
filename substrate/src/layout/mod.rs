//! Substrate's layout generator framework.
//!
//! # Examples
//!
//! ## Simple
//! ```
#![doc = include_str!("../../build/docs/prelude.rs.hidden")]
#![doc = include_str!("../../build/docs/pdk/layers.rs.hidden")]
#![doc = include_str!("../../build/docs/pdk/pdk.rs.hidden")]
#![doc = include_str!("../../build/docs/block/inverter.rs.hidden")]
#![doc = include_str!("../../build/docs/layout/inverter.rs")]
//! ```
//!
//! ## With data
//! ```
#![doc = include_str!("../../build/docs/prelude.rs.hidden")]
#![doc = include_str!("../../build/docs/pdk/layers.rs.hidden")]
#![doc = include_str!("../../build/docs/pdk/pdk.rs.hidden")]
#![doc = include_str!("../../build/docs/block/inverter.rs.hidden")]
#![doc = include_str!("../../build/docs/layout/inverter.rs.hidden")]
#![doc = include_str!("../../build/docs/block/buffer.rs.hidden")]
#![doc = include_str!("../../build/docs/layout/buffer.rs")]
//! ```

use std::{
    marker::PhantomData,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

use arcstr::ArcStr;
use cache::mem::{Cache, CacheHandle};
use geometry::{
    prelude::{Bbox, Orientation, Point},
    transform::{
        HasTransformedView, Transform, TransformMut, Transformation, Transformed, TranslateMut,
    },
};

use crate::io::LayoutType;
use crate::pdk::Pdk;
use crate::{block::Block, error::Error};
use crate::{context::Context, error::Result};

use self::element::{CellId, Element, RawCell, RawInstance, Shape};

pub mod element;
pub mod error;
pub mod gds;

/// An object used to store data created during layout generation.
pub trait Data: HasTransformedView + Send + Sync {}
impl<T: HasTransformedView + Send + Sync> Data for T {}

/// A block that has a layout.
pub trait HasLayout: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Data;
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
    pub(crate) cell_cache: Cache,
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
#[doc = include_str!("../../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../../build/docs/pdk/layers.rs.hidden")]
#[doc = include_str!("../../build/docs/pdk/pdk.rs.hidden")]
#[doc = include_str!("../../build/docs/block/inverter.rs.hidden")]
#[doc = include_str!("../../build/docs/layout/inverter.rs.hidden")]
#[doc = include_str!("../../build/docs/block/buffer.rs.hidden")]
#[doc = include_str!("../../build/docs/layout/buffer.rs.hidden")]
#[doc = include_str!("../../build/docs/layout/generate.rs")]
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct Cell<T: HasLayout> {
    /// Block whose layout this cell represents.
    block: Arc<T>,
    /// Extra data created during layout generation.
    data: T::Data,
    pub(crate) io: Arc<<T::Io as LayoutType>::Data>,
    pub(crate) raw: Arc<RawCell>,
}

impl<T: HasLayout> Cell<T> {
    pub(crate) fn new(
        block: Arc<T>,
        data: T::Data,
        io: Arc<<T::Io as LayoutType>::Data>,
        raw: Arc<RawCell>,
    ) -> Self {
        Self {
            block,
            data,
            io,
            raw,
        }
    }

    /// Returns the block whose layout this cell represents.
    pub fn block(&self) -> &T {
        &self.block
    }

    /// Returns extra data created by the cell's schematic generator.
    pub fn data(&self) -> &T::Data {
        &self.data
    }

    /// Returns the geometry of the cell's IO.
    pub fn io(&self) -> &<T::Io as LayoutType>::Data {
        self.io.as_ref()
    }
}

impl<T: HasLayout> Bbox for Cell<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox()
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<T: HasLayout> {
    pub(crate) cell: CacheHandle<Cell<T>, Error>,
}

impl<T: HasLayout> Clone for CellHandle<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasLayout> CellHandle<T> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<T>> {
        self.cell.try_get().map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if generation fails.
    pub fn cell(&self) -> &Cell<T> {
        self.try_cell().expect("cell generation failed")
    }
}

/// A transformed view of a cell, usually created by accessing the cell of an instance.
pub struct TransformedCell<'a, T: HasLayout> {
    /// Block whose layout this cell represents.
    block: &'a T,
    /// Extra data created during layout generation.
    data: Transformed<'a, T::Data>,
    /// The geometry of the cell's IO.
    io: Transformed<'a, <T::Io as LayoutType>::Data>,
    pub(crate) raw: Arc<RawCell>,
    pub(crate) transform: Transformation,
}

impl<'a, T: HasLayout> TransformedCell<'a, T> {
    /// Returns the block whose layout this cell represents.
    pub fn block(&self) -> &T {
        self.block
    }

    /// Returns extra data created by the cell's schematic generator.
    pub fn data(&'a self) -> &Transformed<'a, T::Data> {
        &self.data
    }

    /// Returns the geometry of the cell's IO.
    pub fn io(&'a self) -> &Transformed<'a, <T::Io as LayoutType>::Data> {
        &self.io
    }
}

impl<T: HasLayout> HasTransformedView for Cell<T> {
    type TransformedView<'a> = TransformedCell<'a, T>;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView<'_> {
        Self::TransformedView {
            block: &self.block,
            data: self.data.transformed_view(trans),
            io: self.io.transformed_view(trans),
            raw: self.raw.clone(),
            transform: trans,
        }
    }
}

impl<'a, T: HasLayout> Bbox for TransformedCell<'a, T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox().transform(self.transform)
    }
}

/// A generic layout instance.
///
/// Stores a pointer to its underlying cell and its instantiated location and orientation.
#[allow(dead_code)]
pub struct Instance<T: HasLayout> {
    cell: CacheHandle<Cell<T>, Error>,
    pub(crate) loc: Point,
    pub(crate) orientation: Orientation,
}

impl<T: HasLayout> Clone for Instance<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            ..*self
        }
    }
}

impl<T: HasLayout> Instance<T> {
    pub(crate) fn new(cell: CacheHandle<Cell<T>, Error>) -> Self {
        Instance {
            cell,
            loc: Point::default(),
            orientation: Orientation::default(),
        }
    }

    /// Tries to access a transformed view of the underlying [`Cell`], blocking on generation.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<Transformed<'_, Cell<T>>> {
        self.cell
            .try_get()
            .map(|cell| {
                cell.transformed_view(Transformation::from_offset_and_orientation(
                    self.loc,
                    self.orientation,
                ))
            })
            .map_err(|err| err.clone())
    }

    /// Returns a transformed view of the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> Transformed<'_, Cell<T>> {
        self.try_cell().expect("cell generation failed")
    }

    /// Tries to access extra data created by the cell's schematic generator.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<Transformed<'_, T::Data>> {
        Ok(self.try_cell()?.data)
    }

    /// Tries to access extra data created by the cell's schematic generator.
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> Transformed<'_, T::Data> {
        self.cell().data
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        Ok(self.try_cell()?.block)
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        self.cell().block
    }

    /// Returns a transformed view of the underlying [`Cell`]'s IO.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_io(&self) -> Result<Transformed<'_, <T::Io as LayoutType>::Data>> {
        Ok(self.try_cell()?.io)
    }

    /// Returns a transformed view of the underlying [`Cell`]'s IO.
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn io(&self) -> Transformed<'_, <T::Io as LayoutType>::Data> {
        self.cell().io
    }

    /// Returns the current transformation of `self`.
    pub fn transformation(&self) -> Transformation {
        Transformation::from_offset_and_orientation(self.loc, self.orientation)
    }
}

impl<T: HasLayout> Bbox for Instance<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.cell().bbox()
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

impl<T: HasLayout> HasTransformedView for Instance<T> {
    type TransformedView<'a> = Instance<T>;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView<'_> {
        (*self).clone().transform(trans)
    }
}

impl<PDK: Pdk, I: HasLayoutImpl<PDK>> Draw<PDK> for Instance<I> {
    fn draw<T>(self, cell: &mut CellBuilder<PDK, T>) -> Result<()> {
        cell.draw_instance(self);
        Ok(())
    }
}

/// A layout cell builder.
///
/// Constructed once for each invocation of [`HasLayoutImpl::layout`].
pub struct CellBuilder<PDK: Pdk, T> {
    phantom: PhantomData<T>,
    instances: Vec<Receiver<Option<RawInstance>>>,
    cell: RawCell,
    /// The current global context.
    pub ctx: Context<PDK>,
}

impl<PDK: Pdk, T> CellBuilder<PDK, T> {
    pub(crate) fn new(id: CellId, name: ArcStr, ctx: Context<PDK>) -> Self {
        Self {
            phantom: PhantomData,
            instances: Vec::new(),
            cell: RawCell::new(id, name),
            ctx,
        }
    }

    pub(crate) fn finish(mut self) -> RawCell {
        for instance in self
            .instances
            .into_iter()
            .map(|instance| instance.recv().unwrap().unwrap())
        {
            self.cell.add_element(instance);
        }
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
    #[doc = include_str!("../../build/docs/prelude.rs.hidden")]
    #[doc = include_str!("../../build/docs/pdk/layers.rs.hidden")]
    #[doc = include_str!("../../build/docs/pdk/pdk.rs.hidden")]
    #[doc = include_str!("../../build/docs/block/inverter.rs.hidden")]
    #[doc = include_str!("../../build/docs/layout/inverter.rs.hidden")]
    #[doc = include_str!("../../build/docs/block/buffer.rs.hidden")]
    #[doc = include_str!("../../build/docs/layout/buffer.rs")]
    /// ```
    pub fn generate<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_layout(block);
        Instance::new(cell.cell)
    }

    /// Generate an instance of `block`.
    ///
    /// Blocks on generation, returning only once the instance's cell is populated. Useful for
    /// handling errors thrown by the generation of a cell immediately.
    pub fn generate_blocking<I: HasLayoutImpl<PDK>>(&mut self, block: I) -> Result<Instance<I>> {
        let cell = self.ctx.generate_layout(block);
        cell.try_cell()?;
        Ok(Instance::new(cell.cell))
    }

    pub(crate) fn draw_instance<I: HasLayoutImpl<PDK>>(&mut self, inst: Instance<I>) {
        let (send, recv) = mpsc::channel();

        self.instances.push(recv);

        let cell = inst.cell.clone();
        thread::spawn(move || {
            if let Ok(cell) = cell.try_get() {
                send.send(Some(RawInstance {
                    cell: cell.raw.clone(),
                    loc: inst.loc,
                    orientation: inst.orientation,
                }))
                .unwrap();
            } else {
                send.send(None).unwrap();
            }
        });
    }

    pub(crate) fn draw_element(&mut self, element: Element) {
        self.cell.add_element(element);
    }

    /// Draw a blockage.
    pub fn draw_blockage(&mut self, shape: Shape) {
        self.cell.add_blockage(shape);
    }

    /// Draw layout object `obj`.
    ///
    /// For instances, a new thread is spawned to add the instance once the underlying cell has
    /// been generated. If generation fails, the spawned thread may panic after this function has
    /// been called.
    ///
    /// For error recovery, instance generation results should be checked using [`Instance::try_cell`]
    /// before calling `draw`.
    ///
    /// # Panics
    ///
    /// May cause a panic if generation of an underlying instance fails.
    pub fn draw(&mut self, obj: impl Draw<PDK>) -> Result<()> {
        obj.draw(self)
    }

    /// Draw layout object `obj` from its reference.
    ///
    /// For instances, a new thread is spawned to add the instance once the underlying cell has
    /// been generated. If generation fails, the spawned thread may panic after this function has
    /// been called.
    ///
    /// For error recovery, instance generation results should be checked using [`Instance::try_cell`]
    /// before calling `draw`.
    ///
    /// # Panics
    ///
    /// May cause a panic if generation of an underlying instance fails.
    pub fn draw_ref(&mut self, obj: &impl DrawRef<PDK>) -> Result<()> {
        obj.draw_ref(self)
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context<PDK> {
        &self.ctx
    }
}

/// An object that can be drawn in a [`CellBuilder`].
pub trait Draw<PDK: Pdk> {
    /// Draws `self` inside `cell`.
    fn draw<T>(self, cell: &mut CellBuilder<PDK, T>) -> Result<()>;
}

/// An object that can be drawn in a [`CellBuilder`] from its reference.
pub trait DrawRef<PDK: Pdk> {
    /// Draws `self` inside `cell` from its reference.
    fn draw_ref<T>(&self, cell: &mut CellBuilder<PDK, T>) -> Result<()>;
}

impl<E: Into<Element>, PDK: Pdk> Draw<PDK> for E {
    fn draw<T>(self, cell: &mut CellBuilder<PDK, T>) -> Result<()> {
        cell.draw_element(self.into());
        Ok(())
    }
}
