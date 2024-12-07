//! Substrate's layout generator framework.
//!
//! # Examples
//!
//! ## Simple
#![doc = examples::get_snippets!("core", "inverter_layout")]
//!
//! ## With data
#![doc = examples::get_snippets!("core", "buffer_layout")]

use std::fmt::Debug;
use std::{marker::PhantomData, sync::Arc, thread};

use arcstr::ArcStr;
use cache::{error::TryInnerError, mem::TypeCache, CacheHandle};
pub use codegen::{Layout, LayoutData};
use examples::get_snippets;
use geometry::prelude::Rect;
use geometry::transform::{TransformRef, TranslateRef};
use geometry::{
    prelude::{Bbox, Point},
    transform::{Transform, TransformMut, Transformation, Translate, TranslateMut},
    union::BoundingUnion,
};
use layir::LayerBbox;
use once_cell::sync::OnceCell;
use schema::Schema;

use crate::context::Context;
use crate::error::Error;
use crate::error::Result;
use crate::types::layout::{Builder, Bundle, BundleKind, HasBundleKind, Io};

use self::element::{CellId, Element, RawCell, RawInstance};

pub mod conv;
pub mod element;
pub mod error;
pub mod schema;
pub mod tiling;
pub mod tracks;

use crate::block::Block;

/// Data exported from a generated layout.
///
/// Contained data is transformed with the containing instance
/// according to its [`TransformRef`] implementation.
pub trait LayoutData: TransformRef + Send + Sync {}
impl<T: TransformRef + Send + Sync> LayoutData for T {}

type CellLayer<T: Layout> = <T::Schema as Schema>::Layer;
pub type IoBuilder<T> =
    <<<T as Block>::Io as HasBundleKind<<T as Layout>::Schema>>::BundleKind as BundleKind<
        <T as Layout>::Schema,
    >>::Builder;
pub type IoBundleKind<T> = <<T as Block>::Io as HasBundleKind<<T as Layout>::Schema>>::BundleKind;
pub type IoBundle<T> = <IoBundleKind<T> as BundleKind<<T as Layout>::Schema>>::Bundle;

/// A block that can be laid out in a given layout [`Schema`].
pub trait Layout: Block<Io: HasBundleKind<Self::Schema>> {
    type Schema: Schema;
    type Data: LayoutData;
    /// Generates the block's layout.
    fn layout(
        &self,
        io: &mut IoBuilder<Self>,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> Result<Self::Data>;
}

impl<T: Layout> Layout for Arc<T> {
    type Schema = T::Schema;
    type Data = T::Data;
    fn layout(
        &self,
        io: &mut IoBuilder<Self>,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> Result<Self::Data> {
        T::layout(self.as_ref(), io, cell)
    }
}

/// Layout-specific context data.
///
/// Stores generated layout cells as well as state used for assigning unique cell IDs.
#[derive(Debug, Default)]
pub struct LayoutContext {
    next_id: CellId,
    pub(crate) cell_cache: TypeCache,
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
#[doc = get_snippets!("core", "generate")]
#[allow(dead_code)]
pub struct Cell<T: Layout> {
    /// Block whose layout this cell represents.
    block: Arc<T>,
    /// Extra data created during layout generation.
    data: T::Data,
    pub(crate) io: IoBundle<T>,
    pub(crate) raw: Arc<RawCell<<T::Schema as Schema>::Layer>>,
}

impl<T: Layout> Cell<T> {
    pub(crate) fn new(
        block: Arc<T>,
        data: T::Data,
        io: IoBundle<T>,
        raw: Arc<RawCell<T::Schema>>,
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
    pub fn io(&self) -> &IoBundle<T> {
        &self.io
    }

    /// The raw layout geometry contained by this cell.
    pub fn raw(&self) -> &Arc<RawCell<CellLayer<T>>> {
        &self.raw
    }
}

impl<T: Layout> Bbox for Cell<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox()
    }
}

impl<T: Layout> LayerBbox<<T::Schema as Schema>::Layer> for Cell<T>
where
    <T::Schema as Schema>::Layer: PartialEq,
{
    fn layer_bbox(&self, layer: &<T::Schema as Schema>::Layer) -> Option<Rect> {
        self.raw.layer_bbox(layer)
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<T: Layout> {
    pub(crate) block: Arc<T>,
    pub(crate) cell: CacheHandle<Result<Cell<T>>>,
}

impl<T: Layout> Clone for CellHandle<T> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T: Layout> CellHandle<T> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<T>> {
        self.cell.try_inner().map_err(|e| match e {
            TryInnerError::CacheError(e) => Error::CacheError(e.clone()),
            TryInnerError::GeneratorError(e) => e.clone(),
        })
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
pub struct TransformedCell<T: Layout> {
    /// Block whose layout this cell represents.
    block: Arc<T>,
    /// Extra data created during layout generation.
    ///
    /// This is the result of applying `trans` to the original cell's data.
    /// If `trans` changes, this field must be updated.
    data: T::Data,
    /// The geometry of the cell's IO.
    ///
    /// This is the result of applying `trans` to the original cell's IO.
    /// If `trans` changes, this field must be updated.
    io: IoBundle<T>,
    /// The underlying raw cell.
    ///
    /// This field should NOT be modified if `trans` changes.
    raw: Arc<RawCell<CellLayer<T>>>,
    /// The transformation applied to all geometry stored in the raw cell (`raw`).
    trans: Transformation,
}

impl<T: Layout> TransformedCell<T> {
    /// Creates a new transformed cell from the given cell and transformation.
    pub fn new(cell: &Cell<T>, trans: Transformation) -> Self {
        Self {
            block: cell.block.clone(),
            data: cell.data.transform_ref(trans),
            io: cell.io.transform_ref(trans),
            raw: cell.raw.clone(),
            trans,
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
}

impl<T: Layout> TranslateRef for TransformedCell<T> {
    fn translate_ref(&self, p: Point) -> Self {
        Self {
            block: self.block.clone(),
            data: self.data.translate_ref(p),
            io: self.io.translate_ref(p),
            raw: self.raw.clone(),
            trans: self.trans.translate_ref(p.x, p.y),
        }
    }
}

impl<T: Layout> TransformRef for TransformedCell<T> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        Self {
            block: self.block.clone(),
            data: self.data.transform_ref(trans),
            io: self.io.transform_ref(trans),
            raw: self.raw.clone(),
            trans: Transformation::cascade(trans, self.trans),
        }
    }
}

impl<T: Layout> Bbox for TransformedCell<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox().transform(self.trans)
    }
}

impl<T: Layout> LayerBbox<<T::Schema as Schema>::Layer> for TransformedCell<T>
where
    CellLayer<T>: PartialEq,
{
    fn layer_bbox(&self, layer: &<T::Schema as Schema>::Layer) -> Option<Rect> {
        self.raw.layer_bbox(layer).transform(self.trans)
    }
}

/// A generic layout instance.
///
/// Stores a pointer to its underlying cell and its instantiated transformation.
#[allow(dead_code)]
pub struct Instance<T: Layout> {
    cell: CellHandle<T>,
    trans: Transformation,
}

impl<T: Layout> Clone for Instance<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            ..*self
        }
    }
}

impl<T: Layout> Instance<T> {
    pub(crate) fn new(cell: CellHandle<T>) -> Self {
        Instance {
            cell,
            trans: Transformation::default(),
        }
    }

    /// Tries to access a transformed view of the underlying [`Cell`], blocking on generation.
    ///
    /// Blocks until cell generation completes.
    ///
    /// The returned object provides coordinates in the parent cell's coordinate system.
    /// If you want coordinates in the child cell's coordinate system,
    /// consider using [`Instance::try_raw_cell`] instead.
    ///
    /// Returns an error if one was thrown during generation.
    // TODO: this recomputes transformations every time it is called.
    pub fn try_cell(&self) -> Result<TransformedCell<T>> {
        self.cell
            .try_cell()
            .map(|cell| TransformedCell::new(cell, self.trans))
    }

    /// Returns a transformed view of the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// The returned object provides coordinates in the parent cell's coordinate system.
    /// If you want coordinates in the child cell's coordinate system,
    /// consider using [`Instance::raw_cell`] instead.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> TransformedCell<T> {
        self.try_cell().expect("cell generation failed")
    }

    /// Tries to access a transformed view of the underlying [`Cell`], blocking on generation.
    ///
    /// Blocks until cell generation completes.
    ///
    /// The returned cell does not store any information related
    /// to this instance's transformation.
    /// Consider using [`Instance::try_cell`] instead.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_raw_cell(&self) -> Result<&Cell<T>> {
        self.cell.try_cell()
    }

    /// Returns a transformed view of the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// The returned cell does not store any information related
    /// to this instance's transformation.
    /// Consider using [`Instance::cell`] instead.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn raw_cell(&self) -> &Cell<T> {
        self.try_raw_cell().expect("cell generation failed")
    }

    /// Tries to access extra data created by the cell's schematic generator.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<T::Data> {
        Ok(self.try_cell()?.data)
    }

    /// Tries to access extra data created by the cell's schematic generator.
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> T::Data {
        self.cell().data
    }

    /// Returns the underlying block used to create this instance's cell.
    pub fn block(&self) -> &T {
        self.cell.block.as_ref()
    }

    /// Returns a transformed view of the underlying [`Cell`]'s IO.
    ///
    /// Blocks until cell generation completes.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_io(&self) -> Result<IoBundle<T>> {
        Ok(self.try_cell()?.io)
    }

    /// Returns a transformed view of the underlying [`Cell`]'s IO.
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn io(&self) -> IoBundle<T> {
        self.cell().io
    }

    /// The transformation of this instance.
    pub fn transformation(&self) -> &Transformation {
        &self.trans
    }
}

impl<T: Layout> Bbox for Instance<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.cell().bbox()
    }
}

impl<T: Layout> LayerBbox<<T::Schema as Schema>::Layer> for Instance<T>
where
    CellLayer<T>: PartialEq,
{
    fn layer_bbox(&self, layer: &<T::Schema as Schema>::Layer) -> Option<Rect> {
        self.cell().layer_bbox(layer)
    }
}

impl<T: Layout> TranslateMut for Instance<T> {
    fn translate_mut(&mut self, p: Point) {
        self.transform_mut(Transformation::from_offset(p))
    }
}

impl<T: Layout> TransformMut for Instance<T> {
    fn transform_mut(&mut self, trans: Transformation) {
        self.trans = Transformation::cascade(trans, self.trans);
    }
}

impl<T: Layout> TranslateRef for Instance<T> {
    fn translate_ref(&self, p: Point) -> Self {
        self.clone().translate(p)
    }
}

impl<T: Layout> TransformRef for Instance<T> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        self.clone().transform(trans)
    }
}

impl<I: Layout> Draw<I::Schema> for Instance<I> {
    fn draw(self, recv: &mut DrawReceiver<I::Schema>) -> Result<()> {
        recv.draw_instance(self);
        Ok(())
    }
}

impl<I: Layout> Draw<I::Schema> for &Instance<I> {
    fn draw(self, recv: &mut DrawReceiver<I::Schema>) -> Result<()> {
        recv.draw_instance((*self).clone());
        Ok(())
    }
}

/// A layout cell builder.
///
/// Constructed once for each invocation of [`Layout::layout`].
pub struct CellBuilder<S: Schema> {
    container: Container<S>,
    /// The current global context.
    pub ctx: Context,
}

impl<S: Schema> CellBuilder<S> {
    /// Creates a new layout builder.
    pub fn new(ctx: Context) -> Self {
        Self {
            container: Container::new(),
            ctx,
        }
    }

    pub(crate) fn finish(self, id: CellId, name: ArcStr) -> RawCell<S::Layer> {
        let mut cell = RawCell::new(id, name);

        self.container.finish(&mut cell.elements);

        cell
    }

    /// Generate an instance of `block`.
    ///
    /// Returns immediately, allowing generation to complete in the background. Attempting to
    /// access the generated instance's cell will block until generation is complete.
    ///
    /// # Examples
    ///
    #[doc = get_snippets!("core", "cell_builder_generate")]
    pub fn generate<I: Layout>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_layout(block);
        Instance::new(cell)
    }

    /// Generate an instance of `block`.
    ///
    /// Blocks on generation, returning only once the instance's cell is populated. Useful for
    /// handling errors thrown by the generation of a cell immediately.
    pub fn generate_blocking<I: Layout>(&mut self, block: I) -> Result<Instance<I>> {
        let cell = self.ctx.generate_layout(block);
        cell.try_cell()?;
        Ok(Instance::new(cell))
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
    pub fn draw(&mut self, obj: impl Draw<S>) -> Result<()> {
        Container::draw(&mut self.container, obj)
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
}

impl<S: Schema> Bbox for CellBuilder<S> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.container.bbox()
    }
}

impl<S: Schema> LayerBbox<S::Layer> for CellBuilder<S> {
    fn layer_bbox(&self, layer: &S::Layer) -> Option<Rect> {
        self.container.layer_bbox(layer)
    }
}

/// A receiver for drawing layout objects.
///
/// Implements the primitive functions that layout objects need to implement [`Draw`].
pub struct DrawReceiver<S: Schema> {
    phantom: PhantomData<S>,
    containers: Vec<Container<S>>,
    instances: Vec<Arc<OnceCell<Option<RawInstance<S::Layer>>>>>,
    elements: Vec<Element<S::Layer>>,
    trans: Transformation,
}

impl<S: Schema> Debug for DrawReceiver<S>
where
    S::Layer: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawReceiver")
            .field("phantom", &self.phantom)
            .field("containers", &self.containers)
            .field("instances", &self.instances)
            .field("elements", &self.elements)
            .field("trans", &self.trans)
            .finish()
    }
}

impl<S: Schema> Clone for DrawReceiver<S>
where
    S::Layer: Clone,
{
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            containers: self.containers.clone(),
            instances: self.instances.clone(),
            elements: self.elements.clone(),
            trans: self.trans.clone(),
        }
    }
}

impl<S: Schema> DrawReceiver<S> {
    pub(crate) fn new() -> Self {
        Self {
            phantom: PhantomData,
            containers: Vec::new(),
            instances: Vec::new(),
            elements: Vec::new(),
            trans: Transformation::default(),
        }
    }

    /// Blocks on instances and returns pointers to them.
    fn get_instances(&self) -> Vec<&RawInstance<S::Layer>> {
        self.instances
            .iter()
            .map(|instance| instance.wait().as_ref().unwrap())
            .collect()
    }

    pub(crate) fn finish(self, elements: &mut Vec<Element<S::Layer>>) {
        for instance in self
            .instances
            .into_iter()
            .map(|instance| instance.wait().clone().unwrap())
        {
            elements.push(instance.transform(self.trans).into());
        }

        elements.extend(
            self.elements
                .into_iter()
                .map(|element| element.transform(self.trans)),
        );

        for mut container in self.containers {
            container.transform_mut(self.trans);
            container.finish(elements);
        }
    }

    pub(crate) fn draw_container(&mut self, container: Container<S>) {
        self.containers.push(container);
    }

    pub(crate) fn draw_element(&mut self, element: impl Into<Element<S::Layer>>) {
        let element = element.into();
        self.elements.push(element);
    }
}

impl<S: Schema> DrawReceiver<S> {
    pub(crate) fn draw_instance<I: Layout<Schema = S>>(&mut self, inst: Instance<I>) {
        let instance = Arc::new(OnceCell::new());
        self.instances.push(instance.clone());

        let cell = inst.cell.clone();
        thread::spawn(move || {
            instance.set(cell.try_cell().ok().map(|cell| RawInstance {
                cell: cell.raw.clone(),
                trans: inst.trans,
            }))
        });
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
    pub fn draw(&mut self, obj: impl Draw<S>) -> Result<()> {
        obj.draw(self)
    }
}

impl<S: Schema> Draw<S> for DrawReceiver<S> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        recv.containers.extend(self.containers);
        recv.instances.extend(self.instances);
        recv.elements.extend(self.elements);
        Ok(())
    }
}

impl<S: Schema> Bbox for DrawReceiver<S> {
    // TODO: process containers?
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.get_instances()
            .bbox()
            .bounding_union(&self.elements.bbox())
    }
}

impl<S: Schema> LayerBbox<S::Layer> for DrawReceiver<S> {
    fn layer_bbox(&self, layer: &S::Layer) -> Option<Rect> {
        self.get_instances()
            .layer_bbox(layer)
            .bounding_union(&self.elements.layer_bbox(layer))
    }
}

/// An object that can be drawn in a [`CellBuilder`].
pub trait Draw<S: Schema>: DrawBoxed<S> {
    /// Draws `self` inside `recv`.
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()>;
}

/// An object where `Box<Self>` can be drawn.
pub trait DrawBoxed<S: Schema> {
    /// Draws `self` inside `recv`.
    fn draw_boxed(self: Box<Self>, recv: &mut DrawReceiver<S>) -> Result<()>;
}

impl<S: Schema, T: Draw<S>> DrawBoxed<S> for T {
    fn draw_boxed(self: Box<Self>, recv: &mut DrawReceiver<S>) -> Result<()> {
        (*self).draw(recv)
    }
}

/// Draws an object into a new [`Container`].
// TODO: Decide if this trait should be made public.
#[allow(dead_code)]
pub(crate) trait DrawContainer<S: Schema>: Draw<S> {
    /// Draws `self` into a new [`Container`].
    fn draw_container(self) -> Result<Container<S>>;
}

impl<S: Schema, T: Draw<S>> DrawContainer<S> for T {
    fn draw_container(self) -> Result<Container<S>> {
        let mut container = Container::new();
        Container::draw(&mut container, self)?;
        Ok(container)
    }
}

impl<S: Schema, T: Draw<S> + ?Sized> Draw<S> for Box<T> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        self.draw_boxed(recv)
    }
}

/// TODO: Temporarily private until we decide whether it is worth exposing.
pub(crate) struct Container<S: Schema> {
    recvs: Vec<DrawReceiver<S>>,
    trans: Transformation,
}

impl<S: Schema> Clone for Container<S>
where
    S::Layer: Clone,
{
    fn clone(&self) -> Self {
        Self {
            recvs: self.recvs.clone(),
            trans: self.trans,
        }
    }
}

impl<S: Schema> Debug for Container<S>
where
    S::Layer: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("recvs", &self.recvs)
            .field("trans", &self.trans)
            .finish()
    }
}

impl<S: Schema> Default for Container<S> {
    fn default() -> Self {
        Self {
            recvs: vec![DrawReceiver::new()],
            trans: Transformation::default(),
        }
    }
}

impl<S: Schema> Container<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn finish(self, elements: &mut Vec<Element<S::Layer>>) {
        for mut recv in self.recvs {
            recv.trans = Transformation::cascade(self.trans, recv.trans);
            recv.finish(elements);
        }
    }

    pub(crate) fn recv_mut(&mut self) -> &mut DrawReceiver<S> {
        self.recvs.last_mut().unwrap()
    }
}

impl<S: Schema> Container<S> {
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
    pub fn draw(&mut self, obj: impl Draw<S>) -> Result<()> {
        self.recv_mut().draw(obj)
    }
}

impl<S: Schema> Bbox for Container<S> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.recvs.bbox().transform(self.trans)
    }
}

impl<S: Schema> LayerBbox<S::Layer> for Container<S> {
    fn layer_bbox(&self, layer: &S::Layer) -> Option<Rect> {
        self.recvs.layer_bbox(layer).transform(self.trans)
    }
}

impl<S: Schema> Draw<S> for Container<S> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        recv.draw_container(self);
        Ok(())
    }
}

impl<S: Schema> TranslateMut for Container<S> {
    fn translate_mut(&mut self, p: Point) {
        self.transform_mut(Transformation::from_offset(p))
    }
}

impl<S: Schema> TransformMut for Container<S> {
    fn transform_mut(&mut self, trans: Transformation) {
        self.trans = Transformation::cascade(trans, self.trans);
        let mut recv = DrawReceiver::new();
        recv.trans = self.trans.inv();
        self.recvs.push(recv);
    }
}

pub struct LayoutLibrary<S: Schema> {
    inner: layir::Library<S::Layer>,
    data: S::Data,
}
