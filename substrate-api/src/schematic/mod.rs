//! Substrate's schematic generator framework.

pub mod conv;

use opacity::Opacity;
use pathtree::PathTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use arcstr::ArcStr;
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;

use crate::block::Block;
use crate::context::Context;
use crate::error::Result;
use crate::generator::Generator;
use crate::io::{
    Connect, FlatLen, Flatten, HasNameTree, NameBuf, Node, NodeContext, NodePriority, NodeUf, Port,
    SchematicData, SchematicType,
};
use crate::pdk::Pdk;

/// A block that has a schematic.
pub trait HasSchematic: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Data;
}

/// A block that has a schematic for process design kit `PDK`.
pub trait HasSchematicImpl<PDK: Pdk>: HasSchematic {
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
    pub(crate) id: CellId,
    /// Dummy path stub containing just this builder's cell ID to ensure that paths are correctly propagated.
    pub(crate) path: InstancePath,
    pub(crate) next_instance_id: InstanceId,
    pub(crate) ctx: Context<PDK>,
    pub(crate) node_ctx: NodeContext,
    pub(crate) instances: Vec<RawInstance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) node_names: HashMap<Node, NameBuf>,
    pub(crate) cell_name: ArcStr,
    pub(crate) phantom: PhantomData<T>,
    pub(crate) ports: Vec<Port>,
    pub(crate) blackbox: Option<ArcStr>,
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
            RawCellContents::Clear(RawCellInner {
                primitives: self.primitives,
                instances: self.instances,
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
        let names = ty.flat_names(name.into());
        assert_eq!(nodes.len(), names.len());

        self.node_names.extend(nodes.iter().copied().zip(names));

        data
    }

    /// Instantiate a schematic view of the given block.
    pub fn instantiate<I: HasSchematicImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_schematic(block.clone());
        let io = block.io();

        let ids = self.node_ctx.nodes(io.len(), NodePriority::Auto);
        let (io_data, ids_rest) = block.io().instantiate(&ids);
        assert!(ids_rest.is_empty());

        let connections = io_data.flatten_vec();
        let names = io.flat_names(arcstr::format!("xinst{}", self.instances.len()));
        assert_eq!(connections.len(), names.len());

        self.node_names
            .extend(connections.iter().copied().zip(names));

        self.next_instance_id.increment();

        let inst = Instance {
            id: self.next_instance_id,
            path: self.path.clone(),
            cell,
            io: io_data,
        };

        let raw = RawInstance {
            name: arcstr::literal!("unnamed"),
            child: inst.cell().raw,
            connections,
        };
        self.instances.push(raw);

        inst
    }

    /// Create an instance and immediately connect its ports.
    pub fn instantiate_connected<I, C>(&mut self, block: I, io: C)
    where
        I: HasSchematicImpl<PDK>,
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
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.primitives.push(device);
    }
}

/// A schematic cell.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Cell<T: HasSchematic> {
    /// The block from which this cell was generated.
    pub block: T,
    /// Data returned by the cell's schematic generator.
    pub data: T::Data,
    pub(crate) raw: Arc<RawCell>,
}

impl<T: HasSchematic> Cell<T> {
    pub(crate) fn new(block: T, data: T::Data, raw: Arc<RawCell>) -> Self {
        Self { block, data, raw }
    }
}

/// A raw (weakly-typed) instance of a cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RawInstance {
    name: ArcStr,
    child: Arc<RawCell>,
    connections: Vec<Node>,
}

/// A context-wide unique identifier for a cell.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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
}

/// The (possibly blackboxed) contents of a raw cell.
pub(crate) type RawCellContents = Opacity<ArcStr, RawCellInner>;

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
pub struct Instance<T: HasSchematic> {
    id: InstanceId,
    /// Head of linked list path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

impl<T: HasSchematic> Instance<T> {
    /// The ports of this instance.
    pub fn io(&self) -> &<T::Io as SchematicType>::Data {
        &self.io
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| cell.nested_view(&self.path.append_segment((cell.raw.id, self.id))))
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedView<Cell<T>> {
        self.try_cell().unwrap()
    }
}

/// A primitive device.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// A wrapper around schematic-specific context data.
#[derive(Debug, Default, Clone)]
pub struct SchematicContext {
    next_id: CellId,
    pub(crate) gen: Generator,
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
pub type InstancePath = PathTree<(CellId, InstanceId)>;

/// Data that can be stored in [`HasSchematic::Data`](crate::schematic::HasSchematic::Data).
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
pub struct NestedCellView<'a, T: HasSchematic> {
    /// The block from which this cell was generated.
    pub block: &'a T,
    /// Data returned by the cell's schematic generator.
    pub data: NestedView<'a, T::Data>,
    pub(crate) raw: Arc<RawCell>,
}

/// A view of a nested instance.
///
/// Created when accessing an instance stored in the data of a nested cell.
pub struct NestedInstanceView<'a, T: HasSchematic> {
    id: InstanceId,
    /// Head of linked list path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: &'a <T::Io as SchematicType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

/// An owned nested instance created by cloning the instance referenced by a
/// [`NestedInstanceView`].
///
/// A [`NestedInstance`] can be used to store a nested instance directly in a cell's data for
/// easier access.
pub struct NestedInstance<T: HasSchematic> {
    id: InstanceId,
    /// Head of linked list path to this instance relative to the current cell.
    path: InstancePath,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

impl HasNestedView for () {
    type NestedView<'a> = ();

    fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView<'_> {}
}

impl<T: HasSchematic> HasNestedView for Cell<T> {
    type NestedView<'a> = NestedCellView<'a, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            block: &self.block,
            data: self.data.nested_view(parent),
            raw: self.raw.clone(),
        }
    }
}

impl<T: HasSchematic> HasNestedView for Instance<T> {
    type NestedView<'a> = NestedInstanceView<'a, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<'a, T: HasSchematic> NestedInstanceView<'a, T> {
    /// The ports of this instance.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Data> {
        self.io.nested_view(&self.path)
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| cell.nested_view(&self.path.append_segment((cell.raw.id, self.id))))
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedView<Cell<T>> {
        self.try_cell().unwrap()
    }

    /// Creates an owned [`NestedInstance`] that can be stored in propagated schematic data.
    pub fn to_owned(&self) -> NestedInstance<T> {
        NestedInstance {
            id: self.id,
            path: self.path.clone(),
            io: (*self.io).clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasSchematic> HasNestedView for NestedInstance<T> {
    type NestedView<'a> = NestedInstanceView<'a, T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        Self::NestedView {
            id: self.id,
            path: self.path.prepend(parent),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasSchematic> NestedInstance<T> {
    /// The ports of this instance.
    pub fn io(&self) -> NestedView<<T::Io as SchematicType>::Data> {
        self.io.nested_view(&self.path)
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<NestedView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| cell.nested_view(&self.path.append_segment((cell.raw.id, self.id))))
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> NestedView<Cell<T>> {
        self.try_cell().unwrap()
    }
}
