//! Substrate's schematic generator framework.

pub mod conv;

use cache::error::TryInnerError;
use cache::mem::TypeCache;
use cache::CacheHandle;
use opacity::Opacity;
use pathtree::PathTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;

use arcstr::ArcStr;
use rust_decimal::Decimal;

use crate::block::Block;
use crate::context::Context;
use crate::error::{Error, Result};
use crate::io::{
    Connect, FlatLen, Flatten, HasNameTree, NameBuf, Node, NodeContext, NodePriority, NodeUf, Port,
    SchematicData, SchematicType,
};
use crate::pdk::Pdk;
use crate::simulation::{HasSimSchematic, Simulator};

/// A block that has associated schematic data.
pub trait HasSchematicData: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Data;
}

/// A block that has a schematic for process design kit `PDK`.
pub trait HasSchematic<PDK: Pdk>: HasSchematicData {
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Data,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> Result<Self::Data>;
}

/// A builder for creating a schematic cell.
#[allow(dead_code)]
pub struct CellBuilder<PDK: Pdk, T: Block> {
    /// The current global context.
    pub ctx: Context<PDK>,
    pub(crate) id: CellId,
    pub(crate) next_instance_id: InstanceId,
    /// The root instance path that all nested paths should be relative to.
    pub(crate) root: InstancePath,
    pub(crate) node_ctx: NodeContext,
    pub(crate) instances: Vec<Receiver<Option<RawInstance>>>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) node_names: HashMap<Node, NameBuf>,
    pub(crate) cell_name: ArcStr,
    pub(crate) phantom: PhantomData<T>,
    pub(crate) ports: Vec<Port>,
    pub(crate) blackbox: Option<BlackboxContents>,
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

/// A builder for creating a simulation schematic cell.
pub struct SimCellBuilder<PDK: Pdk, S: Simulator, T: Block> {
    pub(crate) simulator: Arc<S>,
    pub(crate) inner: CellBuilder<PDK, T>,
}

impl<PDK: Pdk, T: Block> CellBuilder<PDK, T> {
    pub(crate) fn finish(self) -> RawCell {
        let mut roots = HashMap::with_capacity(self.node_names.len());
        let mut uf = self.node_ctx.into_inner();
        for &node in self.node_names.keys() {
            let root = uf.probe_value(node).unwrap().source;
            roots.insert(node, root);
        }
        let contents = if let Some(contents) = self.blackbox {
            RawCellContents::Opaque(contents)
        } else {
            let instances = self
                .instances
                .into_iter()
                .map(|instance| instance.recv().unwrap().unwrap())
                .collect::<Vec<_>>();
            RawCellContents::Clear(RawCellInner {
                primitives: self.primitives,
                instances,
            })
        };
        RawCell {
            id: self.id,
            name: self.cell_name,
            node_names: self.node_names,
            ports: self.ports,
            uf,
            roots,
            contents,
            flatten: T::FLATTEN,
        }
    }

    /// Create a new signal with the given name and hardware type.
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Data {
        let ids = self.node_ctx.nodes(ty.len(), NodePriority::Named);
        let (data, ids_rest) = ty.instantiate(&ids);
        assert!(ids_rest.is_empty());

        let nodes = data.flatten_vec();
        let names = ty.flat_names(Some(name.into().into()));
        assert_eq!(nodes.len(), names.len());

        self.node_names.extend(nodes.iter().copied().zip(names));

        data
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generated results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate<I: HasSchematic<PDK>>(&mut self, block: I) -> CellHandle<I> {
        self.ctx.generate_schematic(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<I: HasSchematic<PDK>>(&mut self, block: I) -> Result<CellHandle<I>> {
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
    pub fn add<I: HasSchematic<PDK>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        assert!(
            self.blackbox.is_none(),
            "cannot add instances to a blackbox cell"
        );

        self.post_instantiate(cell)
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
    pub fn instantiate<I: HasSchematic<PDK>>(&mut self, block: I) -> Instance<I> {
        assert!(
            self.blackbox.is_none(),
            "cannot add instances to a blackbox cell"
        );

        let cell = self.ctx.generate_schematic(block);
        self.post_instantiate(cell)
    }

    /// Creates nodes for the newly-instantiated block's IOs.
    fn post_instantiate<I: HasSchematicData>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        let io = cell.block.io();

        let ids = self.node_ctx.nodes(io.len(), NodePriority::Auto);
        let (io_data, ids_rest) = io.instantiate(&ids);
        assert!(ids_rest.is_empty());

        let connections = io_data.flatten_vec();
        let names = io.flat_names(Some(
            arcstr::format!("xinst{}", self.instances.len()).into(),
        ));
        assert_eq!(connections.len(), names.len());

        self.node_names
            .extend(connections.iter().copied().zip(names));

        self.next_instance_id.increment();

        let inst = Instance {
            id: self.next_instance_id,
            parent: self.root.clone(),
            path: self.root.append_segment(self.next_instance_id, cell.id),
            cell: cell.clone(),
            io: io_data,
        };

        let (send, recv) = mpsc::channel();

        self.instances.push(recv);

        thread::spawn(move || {
            if let Ok(cell) = cell.try_cell() {
                let raw = RawInstance {
                    id: inst.id,
                    name: arcstr::literal!("unnamed"),
                    child: cell.raw.clone(),
                    connections,
                };
                send.send(Some(raw)).unwrap();
            } else {
                send.send(None).unwrap();
            }
        });

        inst
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: HasSchematic<PDK>,
        C: SchematicData,
        <I::Io as SchematicType>::Data: Connect<C>,
    {
        let inst = self.instantiate(block);
        self.connect(inst.io, io);
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: SchematicData,
        D2: SchematicData,
        D1: Connect<D2>,
    {
        let s1f = s1.flatten_vec();
        let s2f = s2.flatten_vec();
        assert_eq!(s1f.len(), s2f.len());
        s1f.into_iter().zip(s2f).for_each(|(a, b)| {
            self.node_ctx.connect(a, b);
        });
    }

    /// Add a primitive device to the schematic of the current cell.
    ///
    /// # Panics
    ///
    /// Panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        assert!(
            self.blackbox.is_none(),
            "cannot add primitives to a blackbox cell"
        );
        self.primitives.push(device);
    }

    /// Marks this cell as a blackbox containing the given content.
    ///
    /// # Panics
    ///
    /// Panics if any instances or primitive devices have already been
    /// added to this cell. A blackbox cell cannot contain instances or
    /// primitive devices.
    pub fn set_blackbox(&mut self, contents: impl Into<BlackboxContents>) {
        self.blackbox = Some(contents.into());
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context<PDK> {
        &self.ctx
    }
}

impl<PDK: Pdk, S: Simulator, T: Block> SimCellBuilder<PDK, S, T> {
    /// Get a reference to the simulator being used.
    pub fn simulator(&self) -> &S {
        &self.simulator
    }

    pub(crate) fn finish(self) -> RawCell {
        self.inner.finish()
    }

    /// Create a new signal with the given name and hardware type.
    pub fn signal<TY: SchematicType>(
        &mut self,
        name: impl Into<ArcStr>,
        ty: TY,
    ) -> <TY as SchematicType>::Data {
        self.inner.signal(name, ty)
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generation results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate<I: HasSchematic<PDK>>(&mut self, block: I) -> CellHandle<I> {
        self.inner.generate(block)
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_blocking<I: HasSchematic<PDK>>(&mut self, block: I) -> Result<CellHandle<I>> {
        self.inner.generate_blocking(block)
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
    pub fn add<I: HasSchematic<PDK>>(&mut self, cell: CellHandle<I>) -> Instance<I> {
        assert!(
            self.inner.blackbox.is_none(),
            "cannot add instances to a blackbox cell"
        );
        self.inner.post_instantiate(cell)
    }

    /// Instantiate a schematic view of the given block.
    ///
    /// # Panics
    ///
    /// Panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    pub fn instantiate<I: HasSchematic<PDK>>(&mut self, block: I) -> Instance<I> {
        self.inner.instantiate(block)
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: HasSchematic<PDK>,
        C: SchematicData,
        <I::Io as SchematicType>::Data: Connect<C>,
    {
        self.inner.instantiate_connected(block, io)
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: SchematicData,
        D2: SchematicData,
        D1: Connect<D2>,
    {
        self.inner.connect(s1, s2)
    }

    /// Add a primitive device to the schematic of the current cell.
    ///
    /// # Panics
    ///
    /// Panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.inner.add_primitive(device)
    }

    /// Marks this cell as a blackbox containing the given content.
    ///
    /// # Panics
    ///
    /// Panics if any instances or primitive devices have already been
    /// added to this cell. A blackbox cell cannot contain instances or
    /// primitive devices.
    pub fn set_blackbox(&mut self, contents: impl Into<BlackboxContents>) {
        self.inner.set_blackbox(contents)
    }

    /// Starts generating a block in a new thread and returns a handle to its cell.
    ///
    /// Can be used to check data stored in the cell or other generation results before adding the
    /// cell to the current schematic with [`CellBuilder::add`].
    ///
    /// To generate and add the block simultaneously, use [`CellBuilder::instantiate`]. However,
    /// error recovery and other checks are not possible when using
    /// [`instantiate`](CellBuilder::instantiate).
    pub fn generate_tb<I: HasSimSchematic<PDK, S>>(&mut self, block: I) -> SimCellHandle<I> {
        self.inner.ctx.generate_testbench_schematic(Arc::new(block))
    }

    /// Generates a cell corresponding to `block` and returns a handle to it.
    ///
    /// Blocks on generation. Useful for handling errors thrown by the generation of a cell immediately.
    ///
    /// As with [`CellBuilder::generate`], the resulting handle must be added to the schematic with
    /// [`CellBuilder::add`] before it can be connected as an instance.
    pub fn generate_tb_blocking<I: HasSimSchematic<PDK, S>>(
        &mut self,
        block: I,
    ) -> Result<SimCellHandle<I>> {
        let cell = self.inner.ctx.generate_testbench_schematic(Arc::new(block));
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
    pub fn add_tb<I: HasSimSchematic<PDK, S>>(&mut self, cell: SimCellHandle<I>) -> Instance<I> {
        assert!(
            self.inner.blackbox.is_none(),
            "cannot add instances to a blackbox cell"
        );
        self.inner.post_instantiate(cell.0)
    }

    /// Instantiate a testbench schematic view of the given block.
    ///
    /// # Panics
    ///
    /// Panics if this cell has been marked as a blackbox.
    /// A blackbox cell cannot contain instances or primitive devices.
    pub fn instantiate_tb<I: HasSimSchematic<PDK, S>>(&mut self, block: I) -> Instance<I> {
        assert!(
            self.inner.blackbox.is_none(),
            "cannot add instances to a blackbox cell"
        );

        let cell = self.inner.ctx.generate_testbench_schematic(Arc::new(block));
        self.inner.post_instantiate(cell.0)
    }

    /// Gets the global context.
    pub fn ctx(&self) -> &Context<PDK> {
        &self.inner.ctx
    }
}

/// A schematic cell.
#[derive(Clone)]
pub struct Cell<T: HasSchematicData> {
    /// The block from which this cell was generated.
    block: Arc<T>,
    /// Data returned by the cell's schematic generator.
    pub(crate) data: T::Data,
    pub(crate) raw: Arc<RawCell>,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    /// The path corresponding to this cell.
    path: InstancePath,
}

impl<T: HasSchematicData> Cell<T> {
    pub(crate) fn new(
        io: <T::Io as SchematicType>::Data,
        block: Arc<T>,
        data: T::Data,
        raw: Arc<RawCell>,
    ) -> Self {
        let id = raw.id;
        Self {
            io,
            block,
            data,
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
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Data> {
        self.io.nested_view(&self.path)
    }

    fn nested_view(&self, parent: &InstancePath) -> NestedCellView<'_, T> {
        NestedCellView {
            block: &self.block,
            data: self.data.nested_view(parent),
            raw: self.raw.clone(),
            io: self.io.nested_view(parent),
        }
    }
}

/// A handle to a schematic cell that is being generated.
pub struct CellHandle<T: HasSchematicData> {
    pub(crate) id: CellId,
    pub(crate) block: Arc<T>,
    pub(crate) cell: CacheHandle<Result<Cell<T>>>,
}

impl<T: HasSchematicData> Clone for CellHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            block: self.block.clone(),
            cell: self.cell.clone(),
        }
    }
}

/// A handle to a testbench schematic cell that is being generated.
#[derive(Clone)]
pub struct SimCellHandle<T: HasSchematicData>(pub(crate) CellHandle<T>);

impl<T: HasSchematicData> CellHandle<T> {
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

impl<T: HasSchematicData> SimCellHandle<T> {
    /// Tries to access the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes and returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<&Cell<T>> {
        self.0.try_cell()
    }

    /// Returns the underlying [`Cell`].
    ///
    /// Blocks until cell generation completes.
    ///
    /// # Panics
    ///
    /// Panics if generation fails.
    pub fn cell(&self) -> &Cell<T> {
        self.0.cell()
    }
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawInstance {
    id: InstanceId,
    name: ArcStr,
    child: Arc<RawCell>,
    connections: Vec<Node>,
}

/// A context-wide unique identifier for a cell.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CellId(u64);

impl CellId {
    pub(crate) fn increment(&mut self) {
        *self = CellId(self.0 + 1)
    }
}

/// A raw (weakly-typed) cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawCell {
    id: CellId,
    name: ArcStr,
    ports: Vec<Port>,
    uf: NodeUf,
    node_names: HashMap<Node, NameBuf>,
    roots: HashMap<Node, Node>,
    contents: RawCellContents,
    /// Whether this cell should be flattened when being exported.
    flatten: bool,
}

/// The (possibly blackboxed) contents of a raw cell.
pub(crate) type RawCellContents = Opacity<BlackboxContents, RawCellInner>;

/// The inner contents of a not-blackboxed [`RawCell`].
#[derive(Debug, Clone)]
pub(crate) struct RawCellInner {
    primitives: Vec<PrimitiveDevice>,
    instances: Vec<RawInstance>,
}

/// A cell-wide unique identifier for an instance.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InstanceId(pub(crate) u64);

impl InstanceId {
    pub(crate) fn increment(&mut self) {
        *self = InstanceId(self.0 + 1)
    }
}

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<T: HasSchematicData> {
    id: InstanceId,
    /// Path of the parent cell.
    parent: InstancePath,
    /// Path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    cell: CellHandle<T>,
}

impl<T: HasSchematicData> Instance<T> {
    /// The ports of this instance.
    pub fn io(&self) -> &<T::Io as SchematicType>::Data {
        &self.io
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<T> {
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

/// A primitive device.
#[derive(Debug, Clone)]
pub enum PrimitiveDevice {
    /// An ideal 2-terminal resistor.
    Res2 {
        /// The positive node.
        pos: Node,
        /// The negative node.
        neg: Node,
        /// The value of the resistor, in Ohms.
        value: Decimal,
    },
    /// A raw instance.
    ///
    /// This can be an instance of a subcircuit defined outside of Substrate.
    RawInstance {
        /// The ports of the instance, as an ordered list.
        ports: Vec<Node>,
        /// The name of the cell being instantiated.
        cell: ArcStr,
        /// Parameters to the cell being instantiated.
        params: HashMap<ArcStr, scir::Expr>,
    },
    /// An instance of a SCIR cell.
    ///
    /// Substrate does not support creating
    /// SCIR instances with parameters.
    ScirInstance {
        /// The SCIR library containing the child cell.
        lib: Arc<scir::Library>,
        /// The ID of the child cell.
        cell: scir::CellId,
        /// The name of the instance.
        ///
        /// This need not be related to the name of the child cell.
        name: ArcStr,
        /// The connections to the ports of the child cell.
        connections: HashMap<ArcStr, Vec<Node>>,
    },
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

/// Data that can be stored in [`HasSchematicData::Data`](crate::schematic::HasSchematicData::Data).
pub trait Data: HasNestedView + Send + Sync {}
impl<T: HasNestedView + Send + Sync> Data for T {}

/// An object that can be nested in the data of a cell.
///
/// Stores a path of instances up to the current cell using a linked list.
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

/// A view of a nested cell.
///
/// Created when accessing a cell from one of its instantiations in another cell.
pub struct NestedCellView<'a, T: HasSchematicData> {
    /// The block from which this cell was generated.
    block: &'a T,
    /// Data returned by the cell's schematic generator.
    data: NestedView<'a, T::Data>,
    #[allow(dead_code)]
    pub(crate) raw: Arc<RawCell>,
    /// The cell's input/output interface.
    io: NestedView<'a, <T::Io as SchematicType>::Data>,
}

impl<'a, T: HasSchematicData> NestedCellView<'a, T> {
    /// Returns the block whose schematic this cell represents.
    pub fn block(&'a self) -> &'a T {
        self.block
    }

    /// Returns the data of `self`.
    pub fn data(&'a self) -> &'a NestedView<'a, T::Data> {
        &self.data
    }

    /// Returns this cell's IO.
    pub fn io(&'a self) -> &'a NestedView<<T::Io as SchematicType>::Data> {
        &self.io
    }
}

/// A view of a nested instance.
///
/// Created when accessing an instance stored in the data of a nested cell.
pub struct NestedInstanceView<'a, T: HasSchematicData> {
    id: InstanceId,
    /// Path to the parent cell of this instance.
    parent: InstancePath,
    /// Head of linked list path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: &'a <T::Io as SchematicType>::Data,
    cell: CellHandle<T>,
}

/// An owned nested instance created by cloning the instance referenced by a
/// [`NestedInstanceView`].
///
/// A [`NestedInstance`] can be used to store a nested instance directly in a cell's data for
/// easier access.
pub struct NestedInstance<T: HasSchematicData> {
    id: InstanceId,
    /// Path to the parent cell of this instance.
    parent: InstancePath,
    /// Path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    cell: CellHandle<T>,
}

impl HasNestedView for () {
    type NestedView<'a> = ();

    fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView<'_> {}
}

impl<T: HasSchematicData> HasNestedView for Instance<T> {
    type NestedView<'a> = NestedInstanceView<'a, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            parent: self.parent.prepend(parent),
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<'a, T: HasSchematicData> NestedInstanceView<'a, T> {
    /// The ports of this instance.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Data> {
        self.io.nested_view(&self.parent)
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<T> {
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
    pub fn to_owned(&self) -> NestedInstance<T> {
        NestedInstance {
            id: self.id,
            parent: self.parent.clone(),
            path: self.path.clone(),
            io: (*self.io).clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasSchematicData> HasNestedView for NestedInstance<T> {
    type NestedView<'a> = NestedInstanceView<'a, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            parent: self.parent.prepend(parent),
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasSchematicData> NestedInstance<T> {
    /// The ports of this instance.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Data> {
        self.io.nested_view(&self.parent)
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedCellView<'_, T>> {
        self.cell
            .try_cell()
            .map(|cell| cell.nested_view(&self.path))
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedCellView<'_, T> {
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
