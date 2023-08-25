//! Substrate's schematic generator framework.

pub mod conv;
pub mod primitives;
pub mod schema;

use cache::error::TryInnerError;
use cache::mem::TypeCache;
use cache::CacheHandle;
pub use codegen::{Schematic, SchematicData};
use pathtree::PathTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

use arcstr::ArcStr;
use schema::Schema;
use scir::Library;
use substrate::pdk::PdkScirSchematic;
use type_dispatch::impl_dispatch;

use crate::block::{self, Block, Opaque, PdkPrimitive, ScirBlock};
use crate::context::Context;
use crate::diagnostics::SourceInfo;
use crate::error::{Error, Result};
use crate::io::{
    Connect, HasNameTree, HasTerminalView, NameBuf, Node, NodeContext, NodePriority, NodeUf, Port,
    SchematicBundle, SchematicType, TerminalView,
};
use crate::pdk::{ExportsPdkSchematicData, Pdk, PdkSchematic, ToSchema};

/// A block with a schematic specified using SCIR.
pub trait ScirSchematic<PDK: Pdk, S: Schema, K = <Self as Block>::Kind>: ScirBlock {
    /// Returns the library containing the SCIR cell and its ID.
    fn schematic(&self) -> Result<(Library<S::Primitive>, scir::CellId)>;
}

#[impl_dispatch({block::Scir; block::InlineScir})]
impl<T, PDK: ToSchema<S>, S: Schema, B: Block<Kind = T> + PdkScirSchematic<PDK>>
    ScirSchematic<PDK, S, T> for B
{
    fn schematic(&self) -> Result<(Library<S::Primitive>, scir::CellId)> {
        let (lib, cell) = PdkScirSchematic::schematic(self)?;
        Ok((
            lib.convert_primitives(PDK::convert_primitive)
                .ok_or(Error::UnsupportedPrimitive)?,
            cell,
        ))
    }
}

/// A block whose contents are opaque to Substrate.
pub trait Blackbox<PDK: Pdk, S: Schema>: Block<Kind = Opaque> {
    /// Returns the contents of the blackbox.
    fn contents(&self, io: &<<Self as Block>::Io as SchematicType>::Bundle) -> BlackboxContents;
}

/// A block that exports data from its schematic.
///
/// All blocks that have a schematic implementation must export data.
pub trait ExportsSchematicData<PDK: Pdk, S: Schema, K = <Self as Block>::Kind>: Block {
    /// Extra schematic data to be stored with the block's generated cell.
    ///
    /// When the block is instantiated, all contained data will be nested
    /// within that instance.
    type Data: SchematicData;
}

/// A block that has a schematic associated with the given PDK and schema.
pub trait Schematic<PDK: Pdk, S: Schema, K = <Self as Block>::Kind>:
    ExportsSchematicData<PDK, S, K>
{
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data>;
}

#[impl_dispatch({block::Scir; block::InlineScir})]
impl<T, PDK: Pdk, S: Schema, B: Block<Kind = T> + ScirSchematic<PDK, S>>
    ExportsSchematicData<PDK, S, T> for B
{
    type Data = ();
}

#[impl_dispatch({block::Scir; block::InlineScir})]
impl<T, PDK: Pdk, S: Schema, B: Block<Kind = T> + ScirSchematic<PDK, S>> Schematic<PDK, S, T>
    for B
{
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data> {
        let (lib, id) = ScirSchematic::schematic(self)?;
        cell.set_scir(ScirCellInner {
            lib: Arc::new(lib),
            cell: id,
        });
        Ok(())
    }
}

#[impl_dispatch({crate::block::PdkPrimitive; crate::block::Cell; crate::block::InlineCell})]
impl<T, PDK: ToSchema<S>, S: Schema, B: PdkSchematic<PDK, T> + Block<Kind = T>>
    ExportsSchematicData<PDK, S, T> for B
{
    type Data = <Self as ExportsPdkSchematicData<PDK>>::Data<S>;
}
#[impl_dispatch({PdkPrimitive; crate::block::Cell; crate::block::InlineCell})]
impl<T, PDK: ToSchema<S>, S: Schema, B: PdkSchematic<PDK, T> + Block<Kind = T>> Schematic<PDK, S, T>
    for B
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data> {
        PdkSchematic::schematic(self, io, cell)
    }
}
impl<PDK: Pdk, S: Schema, B: Blackbox<PDK, S>> ExportsSchematicData<PDK, S, Opaque> for B {
    type Data = ();
}
impl<PDK: Pdk, S: Schema, B: Blackbox<PDK, S>> Schematic<PDK, S, Opaque> for B {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data> {
        cell.set_blackbox(Blackbox::contents(self, io));
        Ok(())
    }
}

/// A builder for creating a schematic cell.
pub struct CellBuilder<PDK: Pdk, S: Schema> {
    /// The current global context.
    pub ctx: Context<PDK>,
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
    pub(crate) contents: CellBuilderContents<S::Primitive>,
}

pub(crate) type CellBuilderContents<P> =
    RawCellKind<CellInner<P>, ScirCellInner<P>, P, BlackboxContents>;

pub(crate) struct CellInner<P> {
    pub(crate) next_instance_id: InstanceId,
    pub(crate) instances: Vec<Receiver<Option<RawInstance<P>>>>,
}

/// The contents of a blackbox cell.
#[derive(Debug, Default, Clone)]
pub struct BlackboxContents {
    /// The list of [`BlackboxElement`]s comprising this cell.
    ///
    /// During netlisting, each blackbox element will be
    /// injected into the final netlist.
    /// Netlister implementations should add spaces before each element
    /// in the list, except for the first element.
    pub elems: Vec<BlackboxElement>,
}

/// An element in the contents of a blackbox cell.
#[derive(Debug, Clone)]
pub enum BlackboxElement {
    /// A reference to a [`Node`].
    Node(Node),
    /// A raw, opaque [`String`].
    RawString(String),
}

impl BlackboxContents {
    /// Creates a new, empty [`BlackboxContents`].
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds the given element to the list of blackbox elements.
    pub fn push(&mut self, elem: impl Into<BlackboxElement>) {
        self.elems.push(elem.into());
    }
}

impl FromIterator<BlackboxElement> for BlackboxContents {
    fn from_iter<T: IntoIterator<Item = BlackboxElement>>(iter: T) -> Self {
        Self {
            elems: iter.into_iter().collect(),
        }
    }
}

impl From<String> for BlackboxElement {
    #[inline]
    fn from(value: String) -> Self {
        Self::RawString(value)
    }
}

impl From<&str> for BlackboxElement {
    #[inline]
    fn from(value: &str) -> Self {
        Self::RawString(value.to_string())
    }
}

impl From<Node> for BlackboxElement {
    #[inline]
    fn from(value: Node) -> Self {
        Self::Node(value)
    }
}

impl From<&Node> for BlackboxElement {
    #[inline]
    fn from(value: &Node) -> Self {
        Self::Node(*value)
    }
}

impl From<String> for BlackboxContents {
    fn from(value: String) -> Self {
        Self {
            elems: vec![BlackboxElement::RawString(value)],
        }
    }
}

impl From<&str> for BlackboxContents {
    fn from(value: &str) -> Self {
        Self {
            elems: vec![BlackboxElement::RawString(value.to_string())],
        }
    }
}

impl<PDK: Pdk, S: Schema> CellBuilder<PDK, S> {
    pub(crate) fn finish(self) -> RawCell<S::Primitive> {
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
            uf,
            roots,
            contents: self.contents.into(),
            flatten: self.flatten,
        }
    }

    /// Create a new signal with the given name and hardware type.
    #[track_caller]
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Bundle {
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
        D1: SchematicBundle,
        D2: SchematicBundle,
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

    /// Marks this cell as a SCIR cell.
    pub(crate) fn set_scir(&mut self, scir: ScirCellInner<S::Primitive>) {
        self.contents = CellBuilderContents::Scir(scir);
    }

    /// Marks this cell as a primitive.
    pub(crate) fn set_primitive(&mut self, primitive: S::Primitive) {
        self.contents = CellBuilderContents::Primitive(primitive);
    }

    /// Marks this cell as a blackbox containing the given content.
    pub(crate) fn set_blackbox(&mut self, contents: impl Into<BlackboxContents>) {
        self.contents = CellBuilderContents::Blackbox(contents.into());
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context<PDK> {
        &self.ctx
    }
}

impl<PDK: Pdk, S: Schema> CellBuilder<PDK, S> {
    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate<I: Schematic<PDK, S>>(&mut self, block: I) -> CellHandle<PDK, S, I> {
        self.ctx.generate_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<I: Schematic<PDK, S>>(
        &mut self,
        block: I,
    ) -> Result<CellHandle<PDK, S, I>> {
        let cell = self.ctx.generate_schematic(block);
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
    pub fn add<I: Schematic<PDK, S>>(
        &mut self,
        cell: CellHandle<PDK, S, I>,
    ) -> Instance<PDK, S, I> {
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
    pub fn instantiate<I: Schematic<PDK, S>>(&mut self, block: I) -> Instance<PDK, S, I> {
        let cell = self.ctx.generate_schematic(block);
        self.post_instantiate(cell, SourceInfo::from_caller())
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: Schematic<PDK, S>,
        C: SchematicBundle,
        <I::Io as SchematicType>::Bundle: Connect<C>,
    {
        let inst = self.instantiate(block);
        self.connect(inst.io, io);
    }

    /// Creates nodes for the newly-instantiated block's IOs.
    fn post_instantiate<I: ExportsSchematicData<PDK, S>>(
        &mut self,
        cell: CellHandle<PDK, S, I>,
        source_info: SourceInfo,
    ) -> Instance<PDK, S, I> {
        let io = cell.block.io();
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
                .append_segment(cell_contents.next_instance_id, cell.id),
            cell: cell.clone(),
            io: io_data,
        };

        let (send, recv) = mpsc::channel();

        cell_contents.instances.push(recv);

        thread::spawn(move || {
            if let Ok(cell) = cell.try_cell() {
                let raw = RawInstance {
                    id: inst.id,
                    name: arcstr::literal!("unnamed"),
                    child: cell.raw.clone(),
                    connections: nodes,
                };
                send.send(Some(raw)).unwrap();
            } else {
                send.send(None).unwrap();
            }
        });

        inst
    }
}

/// A schematic cell.
pub struct Cell<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    /// The block from which this cell was generated.
    block: Arc<T>,
    /// Data returned by the cell's schematic generator.
    pub(crate) data: Arc<T::Data>,
    pub(crate) raw: Arc<RawCell<S::Primitive>>,
    /// The cell's input/output interface.
    io: Arc<<T::Io as SchematicType>::Bundle>,
    /// The path corresponding to this cell.
    path: InstancePath,
}
impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> Clone for Cell<PDK, S, T> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            data: self.data.clone(),
            raw: self.raw.clone(),
            io: self.io.clone(),
            path: self.path.clone(),
        }
    }
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> Cell<PDK, S, T> {
    pub(crate) fn new(
        io: <T::Io as SchematicType>::Bundle,
        block: Arc<T>,
        data: T::Data,
        raw: Arc<RawCell<S::Primitive>>,
    ) -> Self {
        let id = raw.id;
        Self {
            io: Arc::new(io),
            block,
            data: Arc::new(data),
            raw,
            path: InstancePath::new(id),
        }
    }

    /// Returns the block whose schematic this cell represents.
    pub fn block(&self) -> &T {
        &self.block
    }

    /// Returns extra data created by the cell's schematic generator.
    pub fn data(&self) -> NestedView<T::Data> {
        self.data.nested_view(&self.path)
    }

    /// Returns this cell's IO.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Bundle> {
        self.io.nested_view(&self.path)
    }

    fn nested_view(&self, parent: &InstancePath) -> NestedCellView<'_, PDK, S, T> {
        NestedCellView {
            block: &self.block,
            data: self.data.nested_view(parent),
            raw: self.raw.clone(),
            io: self.io.terminal_view(parent),
        }
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    pub(crate) id: CellId,
    pub(crate) block: Arc<T>,
    pub(crate) cell: CacheHandle<Result<Cell<PDK, S, T>>>,
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> Clone for CellHandle<PDK, S, T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            block: self.block.clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> CellHandle<PDK, S, T> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<PDK, S, T>> {
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
    pub fn cell(&self) -> &Cell<PDK, S, T> {
        self.try_cell().expect("cell generation failed")
    }
}

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    id: InstanceId,
    /// Path of the parent cell.
    parent: InstancePath,
    /// Path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Bundle,
    cell: CellHandle<PDK, S, T>,
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> Instance<PDK, S, T> {
    /// The ports of this instance.
    ///
    /// Used for node connection purposes.
    pub fn io(&self) -> &<T::Io as SchematicType>::Bundle {
        &self.io
    }

    /// Tries to access this instance's terminals.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_terminals(&self) -> Result<TerminalView<<T::Io as SchematicType>::Bundle>> {
        self.try_cell().map(|cell| cell.io)
    }

    /// Tries to access this instance's terminals
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn terminals(&self) -> TerminalView<<T::Io as SchematicType>::Bundle> {
        self.cell().io
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, PDK, S, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<PDK, S, T> {
        self.try_cell().expect("cell generation failed")
    }

    /// Tries to access the underlying cell data.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<NestedView<'_, T::Data>> {
        self.try_cell().map(|cell| cell.data)
    }

    /// Tries to access the underlying cell data.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> NestedView<'_, T::Data> {
        self.cell().data
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        self.try_cell().map(|cell| cell.block)
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        self.cell().block
    }
}

/// A wrapper around schematic-specific context data.
#[derive(Debug, Clone)]
pub struct SchematicContext {
    next_id: CellId,
    pub(crate) cell_cache: TypeCache,
}

impl Default for SchematicContext {
    fn default() -> Self {
        Self {
            next_id: CellId(0),
            cell_cache: Default::default(),
        }
    }
}

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

/// A path to an instance from a top level cell.
///
/// Inexpensive to clone as it only clones an ID and a reference counted pointer.
#[derive(Debug, Clone)]
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

/// Data that can be stored in [`HasSchematicData::Data`](crate::schematic::ExportsSchematicData::Data).
pub trait SchematicData: HasNestedView + Send + Sync {}
impl<T: HasNestedView + Send + Sync> SchematicData for T {}

/// An object that can be nested in the data of a cell.
///
/// Stores a path of instances up to the current cell using an [`InstancePath`].
pub trait HasNestedView {
    /// A view of the nested object.
    type NestedView<'a>
    where
        Self: 'a;

    /// Creates a nested view of the object given a parent node.
    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_>;
}

impl<T> HasNestedView for &T
where
    T: HasNestedView,
{
    type NestedView<'a>
    = T::NestedView<'a> where Self: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        (*self).nested_view(parent)
    }
}

// TODO: Potentially use lazy evaluation instead of cloning.
impl<T: HasNestedView> HasNestedView for Vec<T> {
    type NestedView<'a> = Vec<NestedView<'a, T>> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        self.iter().map(|elem| elem.nested_view(parent)).collect()
    }
}

/// The associated nested view of an object.
pub type NestedView<'a, T> = <T as HasNestedView>::NestedView<'a>;

impl HasNestedView for () {
    type NestedView<'a> = ();

    fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView<'_> {}
}

/// A view of a nested cell.
///
/// Created when accessing a cell from one of its instantiations in another cell.
pub struct NestedCellView<'a, PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    /// The block from which this cell was generated.
    block: &'a T,
    /// Data returned by the cell's schematic generator.
    data: NestedView<'a, T::Data>,
    #[allow(dead_code)]
    pub(crate) raw: Arc<RawCell<S::Primitive>>,
    /// The cell's input/output interface.
    io: TerminalView<'a, <T::Io as SchematicType>::Bundle>,
}

impl<'a, PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> NestedCellView<'a, PDK, S, T> {
    /// Returns the block whose schematic this cell represents.
    pub fn block(&'a self) -> &'a T {
        self.block
    }

    /// Returns the data of `self`.
    pub fn data(&'a self) -> &'a NestedView<'a, T::Data> {
        &self.data
    }
}

/// A view of a nested instance.
///
/// Created when accessing an instance stored in the data of a nested cell.
pub struct NestedInstanceView<'a, PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    id: InstanceId,
    /// The path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: &'a <T::Io as SchematicType>::Bundle,
    cell: CellHandle<PDK, S, T>,
}

/// An owned nested instance created by cloning the instance referenced by a
/// [`NestedInstanceView`].
///
/// A [`NestedInstance`] can be used to store a nested instance directly in a cell's data for
/// easier access.
pub struct NestedInstance<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    id: InstanceId,
    /// Path to this instance relative to the current cell. path: InstancePath,
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Bundle,
    cell: CellHandle<PDK, S, T>,
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> HasNestedView for Instance<PDK, S, T> {
    type NestedView<'a> = NestedInstanceView<'a, PDK, S, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<'a, PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> NestedInstanceView<'a, PDK, S, T> {
    /// Tries to access this instance's terminals.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_terminals(&self) -> Result<TerminalView<<T::Io as SchematicType>::Bundle>> {
        self.try_cell().map(|cell| cell.io)
    }

    /// Tries to access this instance's terminals
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn terminals(&self) -> TerminalView<<T::Io as SchematicType>::Bundle> {
        self.cell().io
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, PDK, S, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<PDK, S, T> {
        self.try_cell().expect("cell generation failed")
    }

    /// Tries to access the underlying cell data.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<NestedView<'_, T::Data>> {
        self.try_cell().map(|cell| cell.data)
    }

    /// Tries to access the underlying cell data.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> NestedView<'_, T::Data> {
        self.cell().data
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        self.try_cell().map(|cell| cell.block)
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        self.cell().block
    }

    /// Creates an owned [`NestedInstance`] that can be stored in propagated schematic data.
    pub fn to_owned(&self) -> NestedInstance<PDK, S, T> {
        NestedInstance {
            id: self.id,
            path: self.path.clone(),
            io: (*self.io).clone(),
            cell: self.cell.clone(),
        }
    }

    /// Returns the path of this instance relative to the top cell.
    pub fn path(&self) -> &InstancePath {
        &self.path
    }
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> HasNestedView
    for NestedInstance<PDK, S, T>
{
    type NestedView<'a> = NestedInstanceView<'a, PDK, S, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> NestedInstance<PDK, S, T> {
    /// Tries to access this instance's terminals.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_terminals(&self) -> Result<TerminalView<<T::Io as SchematicType>::Bundle>> {
        self.try_cell().map(|cell| cell.io)
    }

    /// Tries to access this instance's terminals
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn terminals(&self) -> TerminalView<<T::Io as SchematicType>::Bundle> {
        self.cell().io
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, PDK, S, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<'_, PDK, S, T> {
        self.try_cell().expect("cell generation failed")
    }

    /// Tries to access the underlying cell data.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_data(&self) -> Result<NestedView<'_, T::Data>> {
        self.try_cell().map(|cell| cell.data)
    }

    /// Tries to access the underlying cell data.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn data(&self) -> NestedView<'_, T::Data> {
        self.cell().data
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_block(&self) -> Result<&T> {
        self.try_cell().map(|cell| cell.block)
    }

    /// Tries to access the underlying block used to create this instance's cell.
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn block(&self) -> &T {
        self.cell().block
    }

    /// Returns the path of this instance relative to the top cell.
    pub fn path(&self) -> &InstancePath {
        &self.path
    }
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawInstance<P> {
    id: InstanceId,
    name: ArcStr,
    child: Arc<RawCell<P>>,
    connections: Vec<Node>,
}

/// A raw (weakly-typed) cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawCell<P> {
    id: CellId,
    name: ArcStr,
    ports: Vec<Port>,
    uf: NodeUf,
    node_names: HashMap<Node, NameBuf>,
    roots: HashMap<Node, Node>,
    contents: RawCellContents<P>,
    /// Whether this cell should be flattened when being exported.
    flatten: bool,
}

/// The contents of a raw cell.
pub(crate) type RawCellContents<P> =
    RawCellKind<RawCellInner<P>, ScirCellInner<P>, P, BlackboxContents>;

impl<P> From<CellBuilderContents<P>> for RawCellContents<P> {
    fn from(value: CellBuilderContents<P>) -> Self {
        match value {
            CellBuilderContents::Cell(CellInner { instances, .. }) => {
                RawCellContents::Cell(RawCellInner {
                    instances: instances
                        .into_iter()
                        .map(|instance| instance.recv().unwrap().unwrap())
                        .collect::<Vec<_>>(),
                })
            }
            CellBuilderContents::Scir(s) => RawCellContents::Scir(s),
            CellBuilderContents::Primitive(p) => RawCellContents::Primitive(p),
            CellBuilderContents::Blackbox(b) => RawCellContents::Blackbox(b),
        }
    }
}

/// An enumeration of raw cell kinds.
///
/// Can be used to store data associated with each kind of raw cell.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[enumify::enumify]
pub(crate) enum RawCellKind<C, S, P, B> {
    Cell(C),
    Scir(S),
    Primitive(P),
    Blackbox(B),
}

#[derive(Debug, Clone)]
pub(crate) struct RawCellInner<P> {
    instances: Vec<RawInstance<P>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ScirCellInner<P> {
    pub(crate) lib: Arc<scir::Library<P>>,
    pub(crate) cell: scir::CellId,
}

/// A context-wide unique identifier for a cell.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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
