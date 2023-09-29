//! Substrate's schematic generator framework.

pub mod conv;
pub mod primitives;
pub mod schema;

use cache::error::TryInnerError;
use cache::mem::TypeCache;
use cache::{CacheHandle, SecondaryCacheHandle};
pub use codegen::{Schematic, SchematicData};
use pathtree::PathTree;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

use arcstr::ArcStr;
use enumify::enumify;
use once_cell::sync::OnceCell;
use scir::schema::ToSchema;
use scir::Library;
use substrate::pdk::{PdkScirSchematic, SupportsSchema};
use type_dispatch::impl_dispatch;

use crate::block::{self, Block, PdkPrimitive, ScirBlock};
use crate::context::Context;
use crate::diagnostics::SourceInfo;
use crate::error::{Error, Result};
use crate::io::{
    Connect, Flatten, HasNameTree, HasTerminalView, Io, NameBuf, Node, NodeContext, NodePriority,
    NodeUf, Port, SchematicBundle, SchematicType, TerminalView,
};
use crate::pdk::{Pdk, PdkSchematic};
use crate::schematic::schema::Schema;
use crate::sealed;
use crate::sealed::Token;

/// A block with a schematic specified using SCIR.
pub trait ScirSchematic<PDK: Pdk, S: Schema, K = <Self as Block>::Kind>: ScirBlock {
    /// Returns the library containing the SCIR cell and its ID.
    fn schematic(&self) -> Result<(Library<S>, scir::CellId)>;
}

/// A block that exports nodes from its schematic.
///
/// All blocks that have a schematic implementation must export nodes.
pub trait ExportsNestedData<K = <Self as Block>::Kind>: Block {
    /// Extra schematic data to be stored with the block's generated cell.
    ///
    /// When the block is instantiated, all contained data will be nested
    /// within that instance.
    type NestedData: NestedData;
}

/// A block that has a schematic associated with the given PDK and schema.
pub trait CellSchematic<PDK: SupportsSchema<S>, S: Schema, K = <Self as Block>::Kind>:
    ExportsNestedData
{
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::NestedData>;
}

/// A block that has a schematic associated with the given PDK and schema.
pub trait Schematic<PDK: SupportsSchema<S>, S: Schema, K = <Self as Block>::Kind>:
    ExportsNestedData
{
    /// Generates the block's schematic.
    #[doc(hidden)]
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        cell: CellBuilder<PDK, S>,
        _: sealed::Token,
    ) -> Result<(RawCell<PDK, S>, Cell<Self>)>;
}

impl<B: Block<Kind = block::Scir>> ExportsNestedData<block::Scir> for B {
    type NestedData = ();
}

impl<
        PDK: SupportsSchema<S>,
        S: Schema,
        B: Block<Kind = block::Scir> + ScirSchematic<PDK, S, block::Scir>,
    > Schematic<PDK, S, block::Scir> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: CellBuilder<PDK, S>,
        _: Token,
    ) -> Result<(RawCell<PDK, S>, Cell<Self>)> {
        let (lib, id) = ScirSchematic::schematic(block.as_ref())?;
        cell.0.set_scir(ScirCellInner { lib, cell: id });
        let id = cell.0.metadata.id;
        Ok((cell.0.finish(), Cell::new(id, io, block, Arc::new(()))))
    }
}

#[impl_dispatch({block::PdkPrimitive; block::PdkScir; block::PdkCell})]
impl<T, PDK: SupportsSchema<S>, S: Schema, B: Block<Kind = T> + PdkSchematic<PDK, T>>
    Schematic<PDK, S, T> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: CellBuilder<PDK, S>,
        _: Token,
    ) -> Result<(RawCell<PDK, S>, Cell<Self>)> {
        let (raw_cell, handle) = cell.ctx().generate_pdk_schematic_inner(block.clone());
        cell.0.set_pdk(raw_cell);
        let id = cell.0.metadata.id;
        Ok((
            cell.0.finish(),
            Cell::new(id, io, block, handle.try_cell()?.nodes.clone()),
        ))
    }
}

impl<
        PDK: SupportsSchema<S>,
        S: Schema,
        B: Block<Kind = block::Cell> + CellSchematic<PDK, S, block::Cell>,
    > Schematic<PDK, S, block::Cell> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: CellBuilder<PDK, S>,
        _: Token,
    ) -> Result<(RawCell<PDK, S>, Cell<Self>)> {
        let data = CellSchematic::schematic(block.as_ref(), io.as_ref(), &mut cell);
        data.map(|data| {
            let id = cell.0.metadata.id;
            (cell.0.finish(), Cell::new(id, io, block, Arc::new(data)))
        })
    }
}

pub(crate) struct CellBuilderMetadata<PDK: Pdk> {
    /// The current global context.
    pub(crate) ctx: Context<PDK>,
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
}

impl<PDK: Pdk> Clone for CellBuilderMetadata<PDK> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            id: self.id,
            cell_name: self.cell_name.clone(),
            flatten: self.flatten,
            root: self.root.clone(),
            node_ctx: self.node_ctx.clone(),
            node_names: self.node_names.clone(),
            ports: self.ports.clone(),
        }
    }
}

/// A builder for creating a schematic cell.
pub(crate) struct CellBuilderInner<PDK: Pdk, S: Schema> {
    pub(crate) metadata: CellBuilderMetadata<PDK>,
    pub(crate) contents: RawCellContents<PDK, S>,
}

impl<PDK: SupportsSchema<S>, S: Schema> CellBuilderInner<PDK, S> {
    pub(crate) fn finish(self) -> RawCell<PDK, S> {
        let mut roots = HashMap::with_capacity(self.metadata.node_names.len());
        let mut uf = self.metadata.node_ctx.into_uf();
        for &node in self.metadata.node_names.keys() {
            let root = uf.probe_value(node).unwrap().source;
            roots.insert(node, root);
        }

        RawCell {
            id: self.metadata.id,
            name: self.metadata.cell_name,
            node_names: self.metadata.node_names,
            ports: self.metadata.ports,
            flatten: self.metadata.flatten,
            uf,
            roots,
            contents: self.contents,
        }
    }

    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Bundle {
        let (nodes, data) = self.metadata.node_ctx.instantiate_undirected(
            &ty,
            NodePriority::Named,
            SourceInfo::from_caller(),
        );

        let names = ty.flat_names(Some(name.into().into()));
        assert_eq!(nodes.len(), names.len());

        self.metadata
            .node_names
            .extend(nodes.iter().copied().zip(names));

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
            let res = self.metadata.node_ctx.connect(a, b);
            if let Err(err) = res {
                tracing::warn!(?err, "connection failed");
            }
        });
    }

    /// Marks this cell as a SCIR cell.
    pub(crate) fn set_scir(&mut self, scir: ScirCellInner<S>) {
        self.contents = RawCellContents::Scir(scir);
    }

    /// Marks this cell as a primitive.
    pub(crate) fn set_primitive(&mut self, primitive: <S as Schema>::Primitive) {
        self.contents = RawCellContents::Primitive(primitive);
    }

    /// Marks this cell as a PDK-specific cell with the given ID.
    pub(crate) fn set_pdk(
        &mut self,
        raw_cell: SecondaryCacheHandle<Arc<RawCell<PDK, PDK::Schema>>>,
    ) {
        self.contents = RawCellContents::PdkId(raw_cell);
    }

    /// Gets the global context.
    pub(crate) fn ctx(&self) -> &Context<PDK> {
        &self.metadata.ctx
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    fn generate<I: Schematic<PDK, S>>(&mut self, block: I) -> CellHandle<I> {
        self.ctx().generate_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    fn generate_blocking<I: Schematic<PDK, S>>(&mut self, block: I) -> Result<CellHandle<I>> {
        let cell = self.ctx().generate_schematic(block);
        cell.try_cell()?;
        Ok(cell)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. Spawns a thread that waits on the generation of
    /// the underlying cell and panics if generation fails. If error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    fn add<I: Schematic<PDK, S>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Instantiate a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`CellBuilder::generate`] and
    /// [`CellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell and panics if the generator fails. If error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    fn instantiate<I: Schematic<PDK, S>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx().generate_schematic(block);
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Create an instance and immediately connect its ports.
    fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: Schematic<PDK, S>,
        C: SchematicBundle,
        <I::Io as SchematicType>::Bundle: Connect<C>,
    {
        let inst = self.instantiate(block);
        self.connect(inst.io, io);
    }

    /// Creates nodes for the newly-instantiated block's IOs.
    fn post_instantiate<I: ExportsNestedData>(
        &mut self,
        cell: CellHandle<I>,
        source_info: SourceInfo,
    ) -> Instance<I> {
        let io = cell.block.io();
        let cell_contents = self.contents.as_mut().unwrap_cell();

        let (nodes, io_data) =
            self.metadata
                .node_ctx
                .instantiate_directed(&io, NodePriority::Auto, source_info);

        let names = io.flat_names(Some(
            arcstr::format!("xinst{}", cell_contents.instances.len()).into(),
        ));
        assert_eq!(nodes.len(), names.len());

        self.metadata
            .node_names
            .extend(nodes.iter().copied().zip(names));

        cell_contents.next_instance_id.increment();

        let inst = Instance {
            id: cell_contents.next_instance_id,
            parent: self.metadata.root.clone(),
            path: self
                .metadata
                .root
                .append_segment(cell_contents.next_instance_id, cell.id),
            cell: cell.clone(),
            io: io_data,

            terminal_view: OnceCell::new(),
            nested_data: OnceCell::new(),
        };

        cell_contents.instances.push(RawInstance {
            id: inst.id,
            name: arcstr::literal!("unnamed"),
            child: cell.id,
            connections: nodes,
            kind: RawInstanceKind::Pdk,
        });

        inst
    }
}

impl<PDK: Pdk<Schema = S>, S: Schema> CellBuilderInner<PDK, S> {
    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    fn generate_pdk<I: PdkSchematic<PDK>>(&mut self, block: I) -> CellHandle<I> {
        self.ctx().generate_pdk_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    fn generate_pdk_blocking<I: PdkSchematic<PDK>>(&mut self, block: I) -> Result<CellHandle<I>> {
        let cell = self.ctx().generate_pdk_schematic(block);
        cell.try_cell()?;
        Ok(cell)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. Spawns a thread that waits on the generation of
    /// the underlying cell and panics if generation fails. If error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    fn add_pdk<I: PdkSchematic<PDK>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Instantiate a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`CellBuilder::generate`] and
    /// [`CellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell and panics if the generator fails. If error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    fn instantiate_pdk<I: PdkSchematic<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx().generate_pdk_schematic(block);
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Create an instance and immediately connect its ports.
    fn instantiate_pdk_connected<I, C>(&mut self, block: I, io: C)
    where
        I: PdkSchematic<PDK>,
        C: SchematicBundle,
        <I::Io as SchematicType>::Bundle: Connect<C>,
    {
        let inst = self.instantiate_pdk(block);
        self.connect(inst.io, io);
    }
}

pub struct CellBuilder<PDK: Pdk, S: Schema>(pub(crate) CellBuilderInner<PDK, S>);

impl<PDK: SupportsSchema<S>, S: Schema> CellBuilder<PDK, S> {
    pub(crate) fn new(inner: CellBuilderInner<PDK, S>) -> Self {
        Self(inner)
    }
    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Bundle {
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
    pub fn ctx(&self) -> &Context<PDK> {
        self.0.ctx()
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate<I: Schematic<PDK, S>>(&mut self, block: I) -> CellHandle<I> {
        self.0.generate(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<I: Schematic<PDK, S>>(&mut self, block: I) -> Result<CellHandle<I>> {
        self.0.generate_blocking(block)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. Spawns a thread that waits on the generation of
    /// the underlying cell and panics if generation fails. If error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    pub fn add<I: Schematic<PDK, S>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        self.0.add(cell)
    }

    /// Instantiate a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`CellBuilder::generate`] and
    /// [`CellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell and panics if the generator fails. If error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    pub fn instantiate<I: Schematic<PDK, S>>(&mut self, block: I) -> Instance<I> {
        self.0.instantiate(block)
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: Schematic<PDK, S>,
        C: SchematicBundle,
        <I::Io as SchematicType>::Bundle: Connect<C>,
    {
        self.0.instantiate_connected(block, io)
    }
}

pub struct PdkCellBuilder<PDK: Pdk>(pub(crate) CellBuilderInner<PDK, PDK::Schema>);

impl<PDK: Pdk> PdkCellBuilder<PDK> {
    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Bundle {
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
    pub fn ctx(&self) -> &Context<PDK> {
        self.0.ctx()
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate<I: PdkSchematic<PDK>>(&mut self, block: I) -> CellHandle<I> {
        self.0.generate_pdk(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<I: PdkSchematic<PDK>>(&mut self, block: I) -> Result<CellHandle<I>> {
        self.0.generate_pdk_blocking(block)
    }

    /// Adds a cell generated with [`CellBuilder::generate`] to the current schematic.
    ///
    /// Does not block on generation. Spawns a thread that waits on the generation of
    /// the underlying cell and panics if generation fails. If error recovery is desired,
    /// check errors before calling this function using [`CellHandle::try_cell`].
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    pub fn add<I: PdkSchematic<PDK>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        self.0.add_pdk(cell)
    }

    /// Instantiate a schematic view of the given block.
    ///
    /// This function generates and adds the cell to the schematic. If checks need to be done on
    /// the generated cell before it is added to the schematic, use [`CellBuilder::generate`] and
    /// [`CellBuilder::add`].
    ///
    /// Spawns a thread that generates the underlying cell and panics if the generator fails. If error
    /// recovery is desired, use the generate and add workflow mentioned above.
    ///
    /// # Panics
    ///
    /// Immediately panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    ///
    /// The spawned thread may panic after this function returns if cell generation fails.
    #[track_caller]
    pub fn instantiate<I: PdkSchematic<PDK>>(&mut self, block: I) -> Instance<I> {
        self.0.instantiate_pdk(block)
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: PdkSchematic<PDK>,
        C: SchematicBundle,
        <I::Io as SchematicType>::Bundle: Connect<C>,
    {
        self.0.instantiate_pdk_connected(block, io)
    }
}

/// A schematic cell.
pub struct Cell<T: ExportsNestedData> {
    /// The block from which this cell was generated.
    block: Arc<T>,
    /// Data returned by the cell's schematic generator.
    nodes: Arc<T::NestedData>,
    /// The cell's input/output interface.
    io: Arc<<T::Io as SchematicType>::Bundle>,
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
        io: Arc<<T::Io as SchematicType>::Bundle>,
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

    /// Returns this cell's IO.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Bundle> {
        self.io.nested_view(&self.path)
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<T: ExportsNestedData> {
    pub(crate) id: CellId,
    pub(crate) block: Arc<T>,
    pub(crate) io_data: Arc<<T::Io as SchematicType>::Bundle>,
    pub(crate) cell: CacheHandle<Result<Cell<T>>>,
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
        self.cell.try_inner().map_err(|e| match e {
            // TODO: Handle cache errors with more granularity.
            TryInnerError::CacheError(_) => Error::Internal,
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

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<T: ExportsNestedData> {
    id: InstanceId,
    /// Path of the parent cell.
    parent: InstancePath,
    /// Path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Bundle,
    cell: CellHandle<T>,

    /// Stored terminal view for io purposes.
    terminal_view: OnceCell<Arc<TerminalView<<T::Io as SchematicType>::Bundle>>>,
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
        println!(
            "nesting instance {:?} {:?} {:?}",
            Block::name(self.block()),
            parent.top,
            parent.bot
        );
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
    pub fn io(&self) -> &TerminalView<<T::Io as SchematicType>::Bundle> {
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
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        self.cell.try_cell().map(|cell| cell.block.as_ref())
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        &self.cell.cell().block
    }

    pub fn path(&self) -> &InstancePath {
        &self.path
    }
}

pub struct NestedInstance<T: ExportsNestedData>(Instance<T>);

impl<T: ExportsNestedData> Deref for NestedInstance<T> {
    type Target = NestedView<T::NestedData>;

    fn deref(&self) -> &Self::Target {
        &*self.0
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
    pub fn io(&self) -> NestedView<TerminalView<<T::Io as SchematicType>::Bundle>> {
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
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        self.0.try_block()
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        &self.0.block()
    }

    pub fn path(&self) -> &InstancePath {
        &self.0.path
    }
}

/// A wrapper around schematic-specific context data.
#[derive(Debug, Default)]
pub struct SchematicContext {
    pub(crate) next_id: CellId,
    /// Cache from [`CellCacheKey`] and [`PdkCellCacheKey`] to
    /// [`CellMetadata`] and  [`CellData`].
    pub(crate) cell_cache: TypeCache,
    /// Map from `CellId` to `(CellHandle, Box<SecondaryCacheHandle<Arc<RawCell<S>>>>)`.
    pub(crate) id_to_cell: HashMap<CellId, Box<dyn Any + Send + Sync>>,
}

pub(crate) struct SchemaWrapper<S, T>(PhantomData<S>, T);

impl SchematicContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_id(&mut self) -> CellId {
        self.next_id.increment();
        self.next_id
    }
}

/// Cell metadata that can be generated quickly.
pub(crate) struct CellMetadata<B: Block> {
    pub(crate) id: CellId,
    pub(crate) io_data: Arc<<B::Io as SchematicType>::Bundle>,
}

impl<B: Block> Clone for CellMetadata<B> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            io_data: self.io_data.clone(),
        }
    }
}

/// Cell data that must run a user specified generator.
pub(crate) struct CellData<B: ExportsNestedData, PDK: Pdk, S: Schema> {
    cell: Cell<B>,
    raw: RawCell<PDK, S>,
}

pub(crate) struct CellCacheKey<B, PDK, S> {
    pub(crate) block: Arc<B>,
    pub(crate) phantom: PhantomData<(PDK, S)>,
}

impl<B, PDK, S> Clone for CellCacheKey<B, PDK, S> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            phantom: PhantomData,
        }
    }
}

impl<B: PartialEq, PDK, S> PartialEq for CellCacheKey<B, PDK, S> {
    fn eq(&self, other: &Self) -> bool {
        self.block.eq(&other.block)
    }
}

impl<B: Eq, PDK, S> Eq for CellCacheKey<B, PDK, S> {}

impl<B: Hash, PDK, S> Hash for CellCacheKey<B, PDK, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block.hash(state)
    }
}

pub(crate) struct PdkCellCacheKey<B, PDK> {
    pub(crate) block: Arc<B>,
    pub(crate) phantom: PhantomData<PDK>,
}

impl<B, PDK> Clone for PdkCellCacheKey<B, PDK> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            phantom: PhantomData,
        }
    }
}

impl<B: PartialEq, PDK> PartialEq for PdkCellCacheKey<B, PDK> {
    fn eq(&self, other: &Self) -> bool {
        self.block.eq(&other.block)
    }
}

impl<B: Eq, PDK> Eq for PdkCellCacheKey<B, PDK> {}

impl<B: Hash, PDK> Hash for PdkCellCacheKey<B, PDK> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block.hash(state)
    }
}

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

    pub(crate) fn prepend(&self, other: &Self) -> Self {
        println!(
            "{:?} {:?} {:?} {:?}",
            self.top, self.bot, other.top, other.bot
        );
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

/// Data that can be stored in [`HasSchematicData::Data`](crate::schematic::ExportsNestedData::NestedData).
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

/// Defines at runtime whether this instance is associated with a [`PdkCell`]
/// or a [`Cell`].
#[derive(Copy, Clone, Debug)]
#[enumify(no_as_ref, no_as_mut)]
pub(crate) enum RawInstanceKind {
    Pdk,
    Schema,
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawInstance {
    id: InstanceId,
    name: ArcStr,
    child: CellId,
    connections: Vec<Node>,
    kind: RawInstanceKind,
}

/// A raw (weakly-typed) cell.
///
/// Only public for the sake of making the [`Schematic`] trait public,
/// should not have any public methods.
#[allow(dead_code)]
#[doc(hidden)]
pub struct RawCell<PDK: Pdk, S: Schema> {
    id: CellId,
    pub(crate) name: ArcStr,
    ports: Vec<Port>,
    uf: NodeUf,
    node_names: HashMap<Node, NameBuf>,
    roots: HashMap<Node, Node>,
    contents: RawCellContents<PDK, S>,
    /// Whether this cell should be flattened when being exported.
    flatten: bool,
}

impl<PDK: Pdk, S: Schema<Primitive = impl std::fmt::Debug>> std::fmt::Debug for RawCell<PDK, S>
where
    <<PDK as Pdk>::Schema as Schema>::Primitive: std::fmt::Debug,
{
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

/// The contents of a raw cell.
pub(crate) type RawCellContents<PDK, S> = RawCellKind<
    RawCellInner,
    SecondaryCacheHandle<Arc<RawCell<PDK, <PDK as Pdk>::Schema>>>,
    ScirCellInner<S>,
    <S as Schema>::Primitive,
>;

/// An enumeration of raw cell kinds.
///
/// Can be used to store data associated with each kind of raw cell.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[enumify::enumify]
pub(crate) enum RawCellKind<C, I, S, P> {
    Cell(C),
    /// Points to a RawCell in the PDK schema.
    PdkId(I),
    Scir(S),
    Primitive(P),
}

#[derive(Debug, Clone)]
pub(crate) struct RawCellInner {
    pub(crate) next_instance_id: InstanceId,
    pub(crate) instances: Vec<RawInstance>,
}

pub(crate) struct ScirCellInner<S: Schema> {
    pub(crate) lib: scir::Library<S>,
    pub(crate) cell: scir::CellId,
}

impl<S: Schema<Primitive = impl Clone>> Clone for ScirCellInner<S> {
    fn clone(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            cell: self.cell,
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug>> std::fmt::Debug for ScirCellInner<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("ScirCellInner");
        let _ = builder.field("lib", &self.lib);
        let _ = builder.field("cell", &self.cell);
        builder.finish()
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
