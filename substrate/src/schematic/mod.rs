//! Substrate's schematic generator framework.

pub mod conv;
pub mod netlist;
pub mod primitives;
pub mod schema;

use cache::mem::TypeCache;
use cache::CacheHandle;
pub use codegen::NestedData;
use pathtree::PathTree;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use arcstr::ArcStr;
use once_cell::sync::OnceCell;

use crate::block::Block;
use crate::context::Context;
use crate::diagnostics::SourceInfo;
use crate::error::{Error, Result};
use crate::io::schematic::{
    Bundle, Connect, HardwareType, HasTerminalView, IsBundle, Node, NodeContext, NodePriority,
    NodeUf, Port, TerminalView,
};
use crate::io::{Flatten, HasNameTree, NameBuf};
use crate::schematic::conv::ConvError;
use crate::schematic::schema::{FromSchema, Schema};

/// A block that exports nodes from its schematic.
///
/// All blocks that have a schematic implementation must export nodes.
pub trait ExportsNestedData: Block {
    /// Extra schematic data to be stored with the block's generated cell.
    ///
    /// When the block is instantiated, all contained data will be nested
    /// within that instance.
    type NestedData: NestedData;
}

/// A block that has a schematic associated with the given PDK and schema.
pub trait Schematic<S: Schema + ?Sized>: ExportsNestedData {
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &Bundle<<Self as Block>::Io>,
        cell: &mut CellBuilder<S>,
    ) -> Result<Self::NestedData>;
}

impl<T: ExportsNestedData> ExportsNestedData for Arc<T> {
    type NestedData = T::NestedData;
}

impl<S: Schema, T: Schematic<S>> Schematic<S> for Arc<T> {
    fn schematic(
        &self,
        io: &Bundle<<Self as Block>::Io>,
        cell: &mut CellBuilder<S>,
    ) -> Result<Self::NestedData> {
        T::schematic(self.as_ref(), io, cell)
    }
}

/// A builder for creating a schematic cell.
pub struct CellBuilder<S: Schema + ?Sized> {
    /// The current global context.
    pub(crate) ctx: Context,
    pub(crate) id: CellId,
    pub(crate) cell_name: ArcStr,
    pub(crate) flatten: bool,
    /// The root instance path that all nested paths should be relative to.
    pub(crate) root: InstancePath,
    pub(crate) node_ctx: NodeContext,
    pub(crate) node_names: HashMap<Node, NameBuf>,
    /// Outward-facing ports of this cell.
    ///
    /// Directions are as viewed by a parent cell instantiating this cell; these
    /// are the wrong directions to use when looking at connections to this
    /// cell's IO from *within* the cell.
    pub(crate) ports: Vec<Port>,
    pub(crate) contents: RawCellContentsBuilder<S>,
}

impl<S: Schema + ?Sized> CellBuilder<S> {
    pub(crate) fn finish(self) -> RawCell<S> {
        let mut roots = HashMap::with_capacity(self.node_names.len());
        let mut uf = self.node_ctx.into_uf();
        for &node in self.node_names.keys() {
            let root = uf.probe_value(node).unwrap().source;
            roots.insert(node, root);
        }

        RawCell {
            id: self.id,
            name: self.cell_name,
            node_names: self.node_names,
            ports: self.ports,
            flatten: self.flatten,
            uf,
            roots,
            contents: self.contents.build(),
        }
    }

    /// Marks this cell to be flattened.
    pub fn flatten(&mut self) {
        self.flatten = true;
    }

    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: HardwareType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as HardwareType>::Bundle {
        let (nodes, data) = self.node_ctx.instantiate_undirected(
            &ty,
            NodePriority::Named,
            SourceInfo::from_caller(),
        );

        let names = ty.flat_names(Some(name.into().into()));
        assert_eq!(nodes.len(), names.len());

        self.node_names.extend(nodes.iter().copied().zip(names));

        data
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: Flatten<Node>,
        D2: Flatten<Node>,
        D1: Connect<D2>,
    {
        let s1f: Vec<Node> = s1.flatten_vec();
        let s2f: Vec<Node> = s2.flatten_vec();
        assert_eq!(s1f.len(), s2f.len());
        s1f.into_iter().zip(s2f).for_each(|(a, b)| {
            // FIXME: proper error handling mechanism (collect all errors into
            // context and emit later)
            let res = self.node_ctx.connect(a, b);
            if let Err(err) = res {
                tracing::warn!(?err, "connection failed");
            }
        });
    }

    /// Connect all signals in the given data instances.
    pub fn connect_multiple<D>(&mut self, s2: &[D])
    where
        D: Flatten<Node>,
    {
        if s2.len() > 1 {
            for s in &s2[1..] {
                self.connect(&s2[0], s);
            }
        }
    }

    /// Marks this cell as a SCIR cell.
    pub fn set_scir(&mut self, scir: ScirBinding<S>) {
        self.contents = RawCellContentsBuilder::Scir(scir);
    }

    /// Marks this cell as a primitive.
    pub fn set_primitive(&mut self, primitive: PrimitiveBinding<S>) {
        self.contents = RawCellContentsBuilder::Primitive(primitive);
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`].
    pub fn generate<B: Schematic<S>>(&mut self, block: B) -> SchemaCellHandle<S, B> {
        self.ctx().generate_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<B: Schematic<S>>(
        &mut self,
        block: B,
    ) -> Result<SchemaCellHandle<S, B>> {
        let handle = self.ctx().generate_schematic(block);
        handle.cell.try_cell()?;
        Ok(handle)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. If immediate error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// If the instantiated cell fails to generate, this function will eventually cause a panic after
    /// the parent cell's generator completes. To avoid this, return errors using [`Instance::try_data`]
    /// before your generator returns.
    #[track_caller]
    pub fn add<B: ExportsNestedData>(&mut self, cell: SchemaCellHandle<S, B>) -> Instance<B> {
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Instantiates a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`CellBuilder::generate`] and
    /// [`CellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell. If immediate error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// If the instantiated cell fails to generate, this function will eventually cause a panic after
    /// the parent cell's generator completes. To avoid this, return errors using [`Instance::try_data`]
    /// before your generator returns.
    ///
    /// If an error is not returned from the enclosing generator, but this function returns
    /// an error, the enclosing generator will panic since the instantiation irrecoverably failed.
    #[track_caller]
    pub fn instantiate<B: Schematic<S>>(&mut self, block: B) -> Instance<B> {
        let cell = self.ctx().generate_schematic(block);
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Instantiates a schematic view of the given block, blocking on generator for underlying
    /// cell. Returns an error if the generator returned an error.
    ///
    /// See [`SubCellBuilder::instantiate`] for details.
    ///
    /// # Panics
    ///
    /// If an error is not returned from the enclosing generator, but this function returns
    /// an error, the enclosing generator will panic since the instantiation irrecoverably failed.
    #[track_caller]
    pub fn instantiate_blocking<B: Schematic<S>>(&mut self, block: B) -> Result<Instance<B>> {
        let inst = self.instantiate(block);
        inst.try_data()?;
        Ok(inst)
    }

    /// Creates an instance using [`CellBuilder::instantiate`] and immediately connects its ports.
    pub fn instantiate_connected<B, C>(&mut self, block: B, io: C)
    where
        B: Schematic<S>,
        C: IsBundle,
        <B::Io as HardwareType>::Bundle: Connect<C>,
    {
        let inst = self.instantiate(block);
        self.connect(inst.io, io);
    }

    /// Creates nodes for the newly-instantiated block's IOs.
    fn post_instantiate<B: ExportsNestedData>(
        &mut self,
        cell: SchemaCellHandle<S, B>,
        source_info: SourceInfo,
    ) -> Instance<B> {
        let io = cell.cell.block.io();
        let cell_contents = self.contents.as_mut().unwrap_cell();

        let (nodes, io_data) =
            self.node_ctx
                .instantiate_directed(&io, NodePriority::Auto, source_info);

        let names = io.flat_names(Some(
            arcstr::format!("xinst{}", cell_contents.instances.len()).into(),
        ));
        assert_eq!(nodes.len(), names.len());

        self.node_names.extend(nodes.iter().copied().zip(names));

        cell_contents.next_instance_id.increment();

        let inst = Instance {
            id: cell_contents.next_instance_id,
            parent: self.root.clone(),
            path: self
                .root
                .append_segment(cell_contents.next_instance_id, cell.cell.id),
            cell: cell.cell,
            io: io_data,

            terminal_view: OnceCell::new(),
            nested_data: OnceCell::new(),
        };

        cell_contents.instances.push(RawInstanceBuilder {
            id: inst.id,
            name: arcstr::literal!("unnamed"),
            connections: nodes,
            child: cell.handle.map(|handle| match handle {
                Ok(Ok(SchemaCellCacheValue { raw, .. })) => Ok(raw.clone()),
                Ok(Err(e)) => {
                    tracing::error!("{:?}", e);
                    panic!("cell generator failed")
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                    panic!("cache failed")
                }
            }),
        });

        inst
    }

    /// Creates a [`SubCellBuilder`] for instantiating blocks from schema `S2`.
    pub fn sub_builder<S2: Schema + ?Sized>(&mut self) -> SubCellBuilder<S, S2>
    where
        S: FromSchema<S2>,
    {
        SubCellBuilder(self, PhantomData)
    }
}

/// A cell builder for instantiating blocks from schema `S2` in schema `S`.
pub struct SubCellBuilder<'a, S: Schema + ?Sized, S2: Schema + ?Sized>(
    &'a mut CellBuilder<S>,
    PhantomData<S2>,
);

impl<'a, S: FromSchema<S2> + ?Sized, S2: Schema + ?Sized> SubCellBuilder<'a, S, S2> {
    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: HardwareType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as HardwareType>::Bundle {
        self.0.signal(name, ty)
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: Flatten<Node>,
        D2: Flatten<Node>,
        D1: Connect<D2>,
    {
        self.0.connect(s1, s2)
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context {
        &self.0.ctx
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`].
    pub fn generate<B: Schematic<S2>>(&mut self, block: B) -> SchemaCellHandle<S, B> {
        self.ctx().generate_cross_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<B: Schematic<S2>>(
        &mut self,
        block: B,
    ) -> Result<SchemaCellHandle<S, B>> {
        let handle = self.ctx().generate_cross_schematic(block);
        handle.cell.try_cell()?;
        Ok(handle)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. If immediate error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// If the instantiated cell fails to generate, this function will eventually cause a panic after
    /// the parent cell's generator completes. To avoid this, return errors using [`Instance::try_data`]
    /// before your generator returns.
    #[track_caller]
    pub fn add<B: ExportsNestedData>(&mut self, cell: SchemaCellHandle<S, B>) -> Instance<B> {
        self.0.add(cell)
    }

    /// Instantiates a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`SubCellBuilder::generate`] and
    /// [`SubCellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell. If immediate error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// If the instantiated cell fails to generate, this function will eventually cause a panic after
    /// the parent cell's generator completes. To avoid this, return errors using [`Instance::try_data`]
    /// before your generator returns.
    ///
    /// If an error is not returned from the enclosing generator, but this function returns
    /// an error, the enclosing generator will panic since the instantiation irrecoverably failed.
    #[track_caller]
    pub fn instantiate<B: Schematic<S2>>(&mut self, block: B) -> Instance<B> {
        let cell = self.ctx().generate_cross_schematic(block);
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Instantiates a schematic view of the given block, blocking on generator for underlying
    /// cell. Returns an error if the generator returned an error.
    ///
    /// See [`SubCellBuilder::instantiate`] for details.
    ///
    /// # Panics
    ///
    /// If an error is not returned from the enclosing generator, but this function returns
    /// an error, the enclosing generator will panic since the instantiation irrecoverably failed.
    #[track_caller]
    pub fn instantiate_blocking<B: Schematic<S2>>(&mut self, block: B) -> Result<Instance<B>> {
        let inst = self.instantiate(block);
        inst.try_data()?;
        Ok(inst)
    }

    /// Creates an instance using [`SubCellBuilder::instantiate`] and immediately connects its ports.
    pub fn instantiate_connected<B, C>(&mut self, block: B, io: C)
    where
        B: Schematic<S2>,
        C: IsBundle,
        <B::Io as HardwareType>::Bundle: Connect<C>,
    {
        let inst = self.instantiate(block);
        self.connect(inst.io, io);
    }

    /// Creates nodes for the newly-instantiated block's IOs.
    fn post_instantiate<B: ExportsNestedData>(
        &mut self,
        cell: SchemaCellHandle<S, B>,
        source_info: SourceInfo,
    ) -> Instance<B> {
        self.0.post_instantiate(cell, source_info)
    }
}

/// A schematic cell.
pub struct Cell<T: ExportsNestedData> {
    /// The block from which this cell was generated.
    block: Arc<T>,
    /// Data returned by the cell's schematic generator.
    nodes: Arc<T::NestedData>,
    /// The cell's input/output interface.
    io: Arc<<T::Io as HardwareType>::Bundle>,
    /// The path corresponding to this cell.
    path: InstancePath,

    /// Stored nested data for deref purposes.
    nested_data: OnceCell<Arc<NestedView<T::NestedData>>>,
}

impl<T: ExportsNestedData> Deref for Cell<T> {
    type Target = NestedView<T::NestedData>;

    fn deref(&self) -> &Self::Target {
        self.nested_data
            .get_or_init(|| Arc::new(self.data()))
            .as_ref()
    }
}

impl<T: ExportsNestedData> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            nodes: self.nodes.clone(),
            io: self.io.clone(),
            path: self.path.clone(),

            nested_data: self.nested_data.clone(),
        }
    }
}

impl<T: ExportsNestedData> Cell<T> {
    pub(crate) fn new(
        id: CellId,
        io: Arc<<T::Io as HardwareType>::Bundle>,
        block: Arc<T>,
        data: Arc<T::NestedData>,
    ) -> Self {
        Self {
            io,
            block,
            nodes: data,
            path: InstancePath::new(id),
            nested_data: OnceCell::new(),
        }
    }

    /// Returns the block whose schematic this cell represents.
    pub fn block(&self) -> &T {
        &self.block
    }

    /// Returns nested data propagated by the cell's schematic generator.
    pub fn data(&self) -> NestedView<T::NestedData> {
        self.nodes.nested_view(&self.path)
    }

    /// Returns the raw data propagated by the cell's schematic generator.
    pub fn raw_data(&self) -> &Arc<<T as ExportsNestedData>::NestedData> {
        &self.nodes
    }

    /// Returns this cell's IO.
    pub fn io(&self) -> NestedView<<T::Io as HardwareType>::Bundle> {
        self.io.nested_view(&self.path)
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<T: ExportsNestedData> {
    pub(crate) id: CellId,
    pub(crate) block: Arc<T>,
    pub(crate) io_data: Arc<<T::Io as HardwareType>::Bundle>,
    pub(crate) cell: CacheHandle<Result<Arc<Cell<T>>>>,
}

impl<T: ExportsNestedData> Clone for CellHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            block: self.block.clone(),
            io_data: self.io_data.clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T: ExportsNestedData> CellHandle<T> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<T>> {
        // TODO: Handle cache errors with more granularity.
        self.cell
            .try_get()
            .as_ref()
            .map_err(|_| Error::Internal)?
            .as_ref()
            .map(|cell| cell.as_ref())
            .map_err(|e| e.clone())
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

pub(crate) struct SchemaCellCacheValue<S: Schema + ?Sized, B: ExportsNestedData> {
    pub(crate) raw: Arc<RawCell<S>>,
    pub(crate) cell: Arc<Cell<B>>,
}

/// A cell handle associated with a schema `S`.
pub struct SchemaCellHandle<S: Schema + ?Sized, B: ExportsNestedData> {
    pub(crate) handle: CacheHandle<Result<SchemaCellCacheValue<S, B>>>,
    pub(crate) cell: CellHandle<B>,
}

impl<S: Schema, B: ExportsNestedData> SchemaCellHandle<S, B> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<B>> {
        // TODO: Handle cache errors with more granularity.
        self.cell.try_cell()
    }

    /// Returns the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if generation fails.
    pub fn cell(&self) -> &Cell<B> {
        self.cell.cell()
    }

    /// Returns the raw cell.
    pub fn raw(&self) -> Arc<RawCell<S>> {
        let val = self.handle.unwrap_inner();
        val.raw.clone()
    }
}

impl<S: Schema + ?Sized, B: ExportsNestedData> Deref for SchemaCellHandle<S, B> {
    type Target = CellHandle<B>;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl<S: Schema + ?Sized, B: ExportsNestedData> Clone for SchemaCellHandle<S, B> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            cell: self.cell.clone(),
        }
    }
}

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<T: ExportsNestedData> {
    id: InstanceId,
    /// Path of the parent cell.
    parent: InstancePath,
    /// Path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as HardwareType>::Bundle,
    cell: CellHandle<T>,

    /// Stored terminal view for io purposes.
    terminal_view: OnceCell<Arc<TerminalView<<T::Io as HardwareType>::Bundle>>>,
    /// Stored nested data for deref purposes.
    nested_data: OnceCell<Arc<NestedView<T::NestedData>>>,
}

impl<T: ExportsNestedData> Deref for Instance<T> {
    type Target = NestedView<T::NestedData>;

    fn deref(&self) -> &Self::Target {
        self.nested_data
            .get_or_init(|| Arc::new(self.data()))
            .as_ref()
    }
}

impl<B: ExportsNestedData> Clone for Instance<B> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            parent: self.parent.clone(),
            path: self.path.clone(),
            io: self.io.clone(),
            cell: self.cell.clone(),

            terminal_view: self.terminal_view.clone(),
            nested_data: self.nested_data.clone(),
        }
    }
}

impl<B: ExportsNestedData> HasNestedView for Instance<B> {
    type NestedView = NestedInstance<B>;
    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        let mut inst = (*self).clone();
        inst.path = self.path.prepend(parent);
        inst.parent = self.parent.prepend(parent);
        inst.nested_data = OnceCell::new();
        inst.terminal_view = OnceCell::new();
        NestedInstance(inst)
    }
}

impl<T: ExportsNestedData> Instance<T> {
    /// The ports of this instance.
    ///
    /// Used for node connection purposes.
    pub fn io(&self) -> &TerminalView<<T::Io as HardwareType>::Bundle> {
        self.terminal_view
            .get_or_init(|| {
                Arc::new(HasTerminalView::terminal_view(
                    self.cell.id,
                    self.cell.io_data.as_ref(),
                    self.id,
                    &self.io,
                ))
            })
            .as_ref()
    }

    /// Tries to access the underlying cell data.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<NestedView<T::NestedData>> {
        self.cell
            .try_cell()
            .map(|data| data.nodes.nested_view(&self.path))
    }

    /// Tries to access the underlying cell data.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> NestedView<T::NestedData> {
        self.cell.cell().nodes.nested_view(&self.path)
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        &self.cell.block
    }

    /// Returns the path to this [`Instance`].
    pub fn path(&self) -> &InstancePath {
        &self.path
    }
}

/// A nested view of an [`Instance`].
pub struct NestedInstance<T: ExportsNestedData>(Instance<T>);

impl<T: ExportsNestedData> Deref for NestedInstance<T> {
    type Target = NestedView<T::NestedData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B: ExportsNestedData> Clone for NestedInstance<B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<B: ExportsNestedData> HasNestedView for NestedInstance<B> {
    type NestedView = NestedInstance<B>;
    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        let mut inst = (*self).clone();
        inst.0.path = self.0.path.prepend(parent);
        inst.0.parent = self.0.parent.prepend(parent);
        inst.0.nested_data = OnceCell::new();
        inst.0.terminal_view = OnceCell::new();
        inst
    }
}

impl<T: ExportsNestedData> NestedInstance<T> {
    /// The ports of this instance.
    ///
    /// Used for node connection purposes.
    pub fn io(&self) -> NestedView<TerminalView<<T::Io as HardwareType>::Bundle>> {
        self.0.io().nested_view(&self.0.parent)
    }

    /// Tries to access the underlying cell data.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<NestedView<T::NestedData>> {
        self.0.try_data()
    }

    /// Tries to access the underlying cell data.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> NestedView<T::NestedData> {
        self.0.data()
    }

    /// Tries to access the underlying block used to create this instance's cell.
    pub fn block(&self) -> &T {
        self.0.block()
    }

    /// Returns the path to this [`NestedInstance`].
    pub fn path(&self) -> &InstancePath {
        &self.0.path
    }
}

/// A wrapper around schematic-specific context data.
#[derive(Debug, Default)]
pub struct SchematicContext {
    pub(crate) next_id: CellId,
    /// Cache from [`CellCacheKey`] and [`ConvCacheKey`]
    /// to `Result<(Arc<RawCell>, Arc<Cell>)>`.
    pub(crate) cell_cache: TypeCache,
}

impl SchematicContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// Cell metadata that can be generated quickly.
pub(crate) struct CellMetadata<B: Block> {
    pub(crate) id: CellId,
    pub(crate) io_data: Arc<<B::Io as HardwareType>::Bundle>,
}

impl<B: Block> Clone for CellMetadata<B> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            io_data: self.io_data.clone(),
        }
    }
}

pub(crate) struct CellCacheKey<B, S: ?Sized> {
    pub(crate) block: Arc<B>,
    pub(crate) phantom: PhantomData<S>,
}

impl<B, S: ?Sized> Clone for CellCacheKey<B, S> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            phantom: PhantomData,
        }
    }
}

impl<B: PartialEq, S: ?Sized> PartialEq for CellCacheKey<B, S> {
    fn eq(&self, other: &Self) -> bool {
        self.block.eq(&other.block)
    }
}

impl<B: Eq, S: ?Sized> Eq for CellCacheKey<B, S> {}

impl<B: Hash, S: ?Sized> Hash for CellCacheKey<B, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block.hash(state)
    }
}

/// A key for a block that was generated in schema `S1` and converted to schema `S2`.
pub(crate) type ConvCacheKey<B, S1, S2> = CellCacheKey<B, (PhantomData<S1>, S2)>;

/// A path to an instance from a top level cell.
///
/// Inexpensive to clone as it only clones an ID and a reference counted pointer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InstancePath {
    /// The ID of the top level cell that this path is relative to.
    pub(crate) top: CellId,
    /// The ID of the last instance's underlying cell.
    ///
    /// Allows for verification that two paths can be concatenated.
    /// `None` if path is empty.
    pub(crate) bot: Option<CellId>,
    /// A path of instance IDs.
    pub(crate) path: PathTree<InstanceId>,
}

impl InstancePath {
    pub(crate) fn new(top: CellId) -> Self {
        Self {
            top,
            bot: None,
            path: PathTree::empty(),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn append(&self, other: &Self) -> Self {
        if let Some(bot) = self.bot {
            assert_eq!(
                bot, other.top,
                "path to append must start with the cell ID that the current path ends with"
            );
        } else {
            assert_eq!(
                self.top, other.top,
                "path to append must start with the cell ID that the current path ends with"
            );
        }
        Self {
            top: self.top,
            bot: other.bot,
            path: self.path.append(&other.path),
        }
    }

    /// Prepend another path to this path.
    pub fn prepend(&self, other: &Self) -> Self {
        if let Some(bot) = other.bot {
            assert_eq!(
                bot, self.top,
                "path to prepend must end with the cell ID that the current path starts with"
            );
        } else {
            assert_eq!(
                other.top, self.top,
                "path to prepend must end with the cell ID that the current path starts with"
            );
        }
        Self {
            top: other.top,
            bot: self.bot,
            path: self.path.prepend(&other.path),
        }
    }

    pub(crate) fn append_segment(&self, id: InstanceId, cell_id: CellId) -> Self {
        Self {
            top: self.top,
            bot: Some(cell_id),
            path: self.path.append_segment(id),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.bot.is_none()
    }
}

/// Data that can be stored in [`ExportsNestedData::NestedData`].
pub trait NestedData: HasNestedView + Send + Sync {}
impl<T: HasNestedView + Send + Sync> NestedData for T {}

/// An object that can be nested in the data of a cell.
///
/// Stores a path of instances up to the current cell using an [`InstancePath`].
pub trait HasNestedView {
    /// A view of the nested object.
    type NestedView: Send + Sync;

    /// Creates a nested view of the object given a parent node.
    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView;
}

/// The associated nested view of an object.
pub type NestedView<T> = <T as HasNestedView>::NestedView;

impl HasNestedView for () {
    type NestedView = ();

    fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView {}
}

impl<T> HasNestedView for &T
where
    T: HasNestedView,
{
    type NestedView = T::NestedView;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        (*self).nested_view(parent)
    }
}

// TODO: Potentially use lazy evaluation instead of cloning.
impl<T: HasNestedView> HasNestedView for Vec<T> {
    type NestedView = Vec<NestedView<T>>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.iter().map(|elem| elem.nested_view(parent)).collect()
    }
}

impl<T: HasNestedView> HasNestedView for Option<T> {
    type NestedView = Option<NestedView<T>>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.as_ref().map(|inner| inner.nested_view(parent))
    }
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
pub(crate) struct RawInstanceBuilder<S: Schema + ?Sized> {
    id: InstanceId,
    name: ArcStr,
    connections: Vec<Node>,
    child: CacheHandle<Arc<RawCell<S>>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug
    for RawInstanceBuilder<S>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawInstanceBuilder");
        let _ = builder.field("id", &self.id);
        let _ = builder.field("name", &self.name);
        let _ = builder.field("connections", &self.connections);
        let _ = builder.field("child", &self.child);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> RawInstanceBuilder<S> {
    fn build(self) -> RawInstance<S> {
        RawInstance {
            id: self.id,
            name: self.name,
            connections: self.connections,
            child: self.child.get().clone(),
        }
    }
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
pub(crate) struct RawInstance<S: Schema + ?Sized> {
    id: InstanceId,
    name: ArcStr,
    connections: Vec<Node>,
    child: Arc<RawCell<S>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for RawInstance<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawInstance");
        let _ = builder.field("id", &self.id);
        let _ = builder.field("name", &self.name);
        let _ = builder.field("connections", &self.connections);
        let _ = builder.field("child", &self.child);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for RawInstance<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            connections: self.connections.clone(),
            child: self.child.clone(),
        }
    }
}

impl<S: Schema + ?Sized> RawInstance<S> {
    fn convert_schema<S2: FromSchema<S> + ?Sized>(self) -> Result<RawInstance<S2>> {
        Ok(RawInstance {
            id: self.id,
            name: self.name,
            connections: self.connections,
            child: Arc::new((*self.child).clone().convert_schema()?),
        })
    }
}

/// A raw (weakly-typed) cell.
///
/// Only public for the sake of making the [`Schematic`] trait public,
/// should not have any public methods.
#[allow(dead_code)]
#[doc(hidden)]
pub struct RawCell<S: Schema + ?Sized> {
    id: CellId,
    pub(crate) name: ArcStr,
    ports: Vec<Port>,
    uf: NodeUf,
    node_names: HashMap<Node, NameBuf>,
    roots: HashMap<Node, Node>,
    /// Whether this cell should be flattened when being exported.
    flatten: bool,
    contents: RawCellContents<S>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for RawCell<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawCell");
        let _ = builder.field("id", &self.id);
        let _ = builder.field("name", &self.name);
        let _ = builder.field("ports", &self.ports);
        let _ = builder.field("uf", &self.uf);
        let _ = builder.field("node_names", &self.node_names);
        let _ = builder.field("roots", &self.roots);
        let _ = builder.field("contents", &self.contents);
        let _ = builder.field("flatten", &self.flatten);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for RawCell<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            ports: self.ports.clone(),
            uf: self.uf.clone(),
            node_names: self.node_names.clone(),
            roots: self.roots.clone(),
            contents: self.contents.clone(),
            flatten: self.flatten,
        }
    }
}

impl<S: Schema + ?Sized> RawCell<S> {
    pub(crate) fn convert_schema<S2: FromSchema<S> + ?Sized>(self) -> Result<RawCell<S2>> {
        Ok(RawCell {
            id: self.id,
            name: self.name,
            ports: self.ports,
            uf: self.uf,
            node_names: self.node_names,
            roots: self.roots,
            flatten: self.flatten,
            contents: self.contents.convert_schema()?,
        })
    }
}

/// The contents of a raw cell.
pub(crate) type RawCellContentsBuilder<S> =
    RawCellKind<RawCellInnerBuilder<S>, ScirBinding<S>, PrimitiveBinding<S>, ConvertedPrimitive<S>>;

impl<S: Schema + ?Sized> RawCellContentsBuilder<S> {
    fn build(self) -> RawCellContents<S> {
        match self {
            RawCellContentsBuilder::Cell(b) => RawCellContents::Cell(b.build()),
            RawCellContentsBuilder::Scir(s) => RawCellContents::Scir(s),
            RawCellContentsBuilder::Primitive(s) => RawCellContents::Primitive(s),
            RawCellContentsBuilder::ConvertedPrimitive(s) => RawCellContents::ConvertedPrimitive(s),
        }
    }
}

/// The contents of a raw cell.
pub(crate) type RawCellContents<S> =
    RawCellKind<RawCellInner<S>, ScirBinding<S>, PrimitiveBinding<S>, ConvertedPrimitive<S>>;

impl<S: Schema + ?Sized> RawCellContents<S> {
    fn convert_schema<S2: FromSchema<S> + ?Sized>(self) -> Result<RawCellContents<S2>> {
        Ok(match self {
            RawCellContents::Cell(c) => RawCellContents::Cell(c.convert_schema()?),
            RawCellContents::Scir(s) => RawCellContents::Scir(ScirBinding {
                lib: s
                    .lib
                    .convert_schema()
                    .map_err(|_| Error::UnsupportedPrimitive)?
                    .build()
                    .map_err(ConvError::from)?,
                cell: s.cell,
                port_map: s.port_map,
            }),
            RawCellContents::Primitive(p) => {
                RawCellContents::ConvertedPrimitive(ConvertedPrimitive {
                    converted: <S2 as scir::schema::FromSchema<S>>::convert_primitive(
                        p.primitive.clone(),
                    )
                    .map_err(|_| Error::UnsupportedPrimitive)?,
                    original: Arc::new(p),
                })
            }
            RawCellContents::ConvertedPrimitive(p) => {
                RawCellContents::ConvertedPrimitive(ConvertedPrimitive {
                    converted: <S2 as scir::schema::FromSchema<S>>::convert_primitive(
                        p.converted.clone(),
                    )
                    .map_err(|_| Error::UnsupportedPrimitive)?,
                    original: Arc::new(p),
                })
            }
        })
    }
}

pub(crate) trait ConvertPrimitive<S: Schema + ?Sized>: Any + Send + Sync {
    fn convert_primitive(&self) -> Result<<S as Schema>::Primitive>;
    fn convert_instance(&self, inst: &mut scir::Instance) -> Result<()>;
    fn port_map(&self) -> &HashMap<ArcStr, Vec<Node>>;
}

impl<S1: FromSchema<S2> + ?Sized, S2: Schema + ?Sized> ConvertPrimitive<S1>
    for PrimitiveBinding<S2>
{
    // TODO: Improve error handling
    fn convert_primitive(&self) -> Result<<S1 as Schema>::Primitive> {
        <S1 as scir::schema::FromSchema<S2>>::convert_primitive(self.primitive.clone())
            .map_err(|_| Error::UnsupportedPrimitive)
    }
    fn convert_instance(&self, inst: &mut scir::Instance) -> Result<()> {
        <S1 as scir::schema::FromSchema<S2>>::convert_instance(inst, &self.primitive)
            .map_err(|_| Error::UnsupportedPrimitive)
    }
    fn port_map(&self) -> &HashMap<ArcStr, Vec<Node>> {
        &self.port_map
    }
}

impl<S1: FromSchema<S2> + ?Sized, S2: Schema + ?Sized> ConvertPrimitive<S1>
    for ConvertedPrimitive<S2>
{
    // TODO: Improve error handling
    fn convert_primitive(&self) -> Result<<S1 as Schema>::Primitive> {
        <S1 as scir::schema::FromSchema<S2>>::convert_primitive(self.original.convert_primitive()?)
            .map_err(|_| Error::UnsupportedPrimitive)
    }
    fn convert_instance(&self, inst: &mut scir::Instance) -> Result<()> {
        self.original.convert_instance(inst)?;
        <S1 as scir::schema::FromSchema<S2>>::convert_instance(inst, &self.converted)
            .map_err(|_| Error::UnsupportedPrimitive)
    }
    fn port_map(&self) -> &HashMap<ArcStr, Vec<Node>> {
        self.original.port_map()
    }
}

/// A binding to a schema primitive that can be used to define
/// a Substrate schematic.
pub struct PrimitiveBinding<S: Schema + ?Sized> {
    pub(crate) primitive: <S as Schema>::Primitive,
    pub(crate) port_map: HashMap<ArcStr, Vec<Node>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for PrimitiveBinding<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Primitive");
        let _ = builder.field("primitive", &self.primitive);
        let _ = builder.field("port_map", &self.port_map);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for PrimitiveBinding<S> {
    fn clone(&self) -> Self {
        Self {
            primitive: self.primitive.clone(),
            port_map: self.port_map.clone(),
        }
    }
}

impl<S: Schema> PrimitiveBinding<S> {
    /// Creates a new [`PrimitiveBinding`] corresponding to the given schema primitive.
    pub fn new(primitive: <S as Schema>::Primitive) -> Self {
        Self {
            primitive,
            port_map: Default::default(),
        }
    }

    /// Connects port `port` of the schema primitive to Substrate nodes `s`.
    pub fn connect(&mut self, port: impl Into<ArcStr>, s: impl Flatten<Node>) {
        self.port_map.insert(port.into(), s.flatten_vec());
    }
}

pub(crate) struct ConvertedPrimitive<S: Schema + ?Sized> {
    converted: <S as Schema>::Primitive,
    original: Arc<dyn ConvertPrimitive<S>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug
    for ConvertedPrimitive<S>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ConvertedPrimitive");
        let _ = builder.field("converted", &self.converted);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for ConvertedPrimitive<S> {
    fn clone(&self) -> Self {
        Self {
            converted: self.converted.clone(),
            original: self.original.clone(),
        }
    }
}

impl<S: Schema + ?Sized> ConvertedPrimitive<S> {
    pub(crate) fn port_map(&self) -> &HashMap<ArcStr, Vec<Node>> {
        self.original.port_map()
    }
}

/// An enumeration of raw cell kinds.
///
/// Can be used to store data associated with each kind of raw cell.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[enumify::enumify(generics_only)]
pub(crate) enum RawCellKind<C, S, P, CP> {
    Cell(C),
    Scir(S),
    Primitive(P),
    ConvertedPrimitive(CP),
}

pub(crate) struct RawCellInnerBuilder<S: Schema + ?Sized> {
    pub(crate) next_instance_id: InstanceId,
    pub(crate) instances: Vec<RawInstanceBuilder<S>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug
    for RawCellInnerBuilder<S>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawCellInnerBuilder");
        let _ = builder.field("next_instance_id", &self.next_instance_id);
        let _ = builder.field("instances", &self.instances);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Default for RawCellInnerBuilder<S> {
    fn default() -> Self {
        Self {
            next_instance_id: Default::default(),
            instances: Default::default(),
        }
    }
}

impl<S: Schema + ?Sized> RawCellInnerBuilder<S> {
    fn build(self) -> RawCellInner<S> {
        RawCellInner {
            instances: self
                .instances
                .into_iter()
                .map(|builder| builder.build())
                .collect(),
        }
    }
}

pub(crate) struct RawCellInner<S: Schema + ?Sized> {
    pub(crate) instances: Vec<RawInstance<S>>,
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for RawCellInner<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("RawCellInner");
        let _ = builder.field("instances", &self.instances);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for RawCellInner<S> {
    fn clone(&self) -> Self {
        Self {
            instances: self.instances.clone(),
        }
    }
}

impl<S: Schema + ?Sized> RawCellInner<S> {
    fn convert_schema<S2: FromSchema<S> + ?Sized>(self) -> Result<RawCellInner<S2>> {
        Ok(RawCellInner {
            instances: self
                .instances
                .into_iter()
                .map(|instance| instance.convert_schema())
                .collect::<Result<_>>()?,
        })
    }
}

/// A binding to a cell within a SCIR library that can be used to define a Substrate schematic.
pub struct ScirBinding<S: Schema + ?Sized> {
    pub(crate) lib: scir::Library<S>,
    pub(crate) cell: scir::CellId,
    pub(crate) port_map: HashMap<ArcStr, Vec<Node>>,
}

impl<S: Schema<Primitive = impl Clone> + ?Sized> Clone for ScirBinding<S> {
    fn clone(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            cell: self.cell,
            port_map: self.port_map.clone(),
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for ScirBinding<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ScirCellInner");
        let _ = builder.field("lib", &self.lib);
        let _ = builder.field("cell", &self.cell);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> ScirBinding<S> {
    /// Creates a new [`ScirBinding`] corresponding to the given cell in
    /// SCIR library `lib`.
    ///
    /// # Panics
    ///
    /// Panics if the provided cell does not exist in the SCIR library.
    pub fn new(lib: scir::Library<S>, cell: scir::CellId) -> Self {
        assert!(lib.try_cell(cell).is_some());
        Self {
            lib,
            cell,
            port_map: HashMap::new(),
        }
    }

    /// Connects port `port` of the SCIR cell to Substrate nodes `s`.
    pub fn connect(&mut self, port: impl Into<ArcStr>, s: impl Flatten<Node>) {
        self.port_map.insert(port.into(), s.flatten_vec());
    }

    /// Returns the SCIR cell that this Substrate translation corresponds to.
    pub fn cell(&self) -> &scir::Cell {
        self.lib.cell(self.cell)
    }

    /// Returns the ports of the underlying SCIR cell in order.
    pub fn ports(&self) -> impl Iterator<Item = &ArcStr> {
        let cell = self.cell();
        cell.ports().map(|port| &cell.signal(port.signal()).name)
    }

    fn port_map(&self) -> &HashMap<ArcStr, Vec<Node>> {
        &self.port_map
    }

    /// Converts the underlying SCIR library to schema `S2`.
    pub fn convert_schema<S2: FromSchema<S> + ?Sized>(
        self,
    ) -> substrate::error::Result<ScirBinding<S2>> {
        Ok(ScirBinding {
            //  TODO: More descriptive error.
            lib: self
                .lib
                .convert_schema::<S2>()
                .map_err(|_| Error::UnsupportedPrimitive)?
                .build()
                .unwrap(),
            cell: self.cell,
            port_map: self.port_map,
        })
    }
}

/// A context-wide unique identifier for a cell.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CellId(u64);

impl CellId {
    pub(crate) fn increment(&mut self) {
        *self = CellId(self.0 + 1)
    }
}

/// A cell-wide unique identifier for an instance.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InstanceId(pub(crate) u64);

impl InstanceId {
    pub(crate) fn increment(&mut self) {
        *self = InstanceId(self.0 + 1)
    }
}
