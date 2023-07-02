//! Substrate's schematic generator framework.

pub mod conv;

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
    pub(crate) next_instance_id: InstanceId,
    pub(crate) ctx: Context<PDK>,
    pub(crate) node_ctx: NodeContext,
    pub(crate) instances: Vec<RawInstance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) node_names: HashMap<Node, NameBuf>,
    pub(crate) cell_name: ArcStr,
    pub(crate) phantom: PhantomData<T>,
    pub(crate) ports: Vec<Port>,
}

impl<PDK: Pdk, T: Block> CellBuilder<PDK, T> {
    pub(crate) fn finish(self) -> RawCell {
        let mut roots = HashMap::with_capacity(self.node_names.len());
        let mut uf = self.node_ctx.into_inner();
        for &node in self.node_names.keys() {
            let root = uf.probe_value(node).unwrap().source;
            roots.insert(node, root);
        }
        RawCell {
            id: self.id,
            name: self.cell_name,
            primitives: self.primitives,
            instances: self.instances,
            node_names: self.node_names,
            ports: self.ports,
            uf,
            roots,
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
            head: None,
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
pub(crate) struct RawCell {
    id: CellId,
    name: ArcStr,
    primitives: Vec<PrimitiveDevice>,
    instances: Vec<RawInstance>,
    ports: Vec<Port>,
    uf: NodeUf,
    node_names: HashMap<Node, NameBuf>,
    roots: HashMap<Node, Node>,
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
    head: Option<Arc<RetrogradeEntry>>,
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
    pub fn try_cell(&self) -> Result<PathView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| {
                cell.path_view(Some(Arc::new(RetrogradeEntry {
                    entry: Arc::new(InstanceEntry {
                        id: self.id,
                        parent: self.head.clone(),
                    }),
                    child: None,
                })))
            })
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> PathView<Cell<T>> {
        self.try_cell().unwrap()
    }
}

/// A primitive device.
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

/// Grows to the left.
#[derive(Debug, Clone)]
pub struct RetrogradeEntry {
    pub(crate) entry: Arc<InstanceEntry>,
    pub(crate) child: Option<Arc<RetrogradeEntry>>,
}

/// Grows to the right.
#[derive(Debug, Clone)]
pub struct InstanceEntry {
    pub(crate) id: InstanceId,
    pub(crate) parent: Option<Arc<RetrogradeEntry>>,
}

pub trait Data: HasPathView + Send + Sync {}
impl<T: HasPathView + Send + Sync> Data for T {}

pub trait HasPathView {
    type PathView<'a>
    where
        Self: 'a;

    fn path_view<'a>(&'a self, parent: Option<Arc<RetrogradeEntry>>) -> Self::PathView<'a>;
}

// TODO: Potentially use lazy evaluation instead of cloning.
impl<T: HasPathView> HasPathView for Vec<T> {
    type PathView<'a> = Vec<PathView<'a, T>> where T: 'a;

    fn path_view<'a>(&'a self, parent: Option<Arc<RetrogradeEntry>>) -> Self::PathView<'a> {
        self.iter()
            .map(|elem| elem.path_view(parent.clone()))
            .collect()
    }
}

pub type PathView<'a, T: HasPathView> = T::PathView<'a>;

pub struct CellPathView<'a, T: HasSchematic> {
    /// The block from which this cell was generated.
    pub block: &'a T,
    /// Data returned by the cell's schematic generator.
    pub data: PathView<'a, T::Data>,
    pub(crate) raw: Arc<RawCell>,
}

pub struct InstancePathView<'a, T: HasSchematic> {
    id: InstanceId,
    /// Head of linked list path to this instance relative to the current cell.
    head: Option<Arc<RetrogradeEntry>>,
    /// The cell's input/output interface.
    io: &'a <T::Io as SchematicType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

pub struct OwnedInstancePathView<T: HasSchematic> {
    id: InstanceId,
    /// Head of linked list path to this instance relative to the current cell.
    head: Option<Arc<RetrogradeEntry>>,
    /// The cell's input/output interface.
    io: <T::Io as SchematicType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

impl HasPathView for () {
    type PathView<'a> = ();

    fn path_view<'a>(&'a self, _parent: Option<Arc<RetrogradeEntry>>) -> Self::PathView<'a> {}
}

impl<T: HasSchematic> HasPathView for Cell<T> {
    type PathView<'a> = CellPathView<'a, T>;

    fn path_view<'a>(&'a self, parent: Option<Arc<RetrogradeEntry>>) -> Self::PathView<'a> {
        Self::PathView {
            block: &self.block,
            data: self.data.path_view(parent),
            raw: self.raw.clone(),
        }
    }
}

impl<T: HasSchematic> HasPathView for Instance<T> {
    type PathView<'a> = InstancePathView<'a, T>;

    fn path_view<'a>(&'a self, parent: Option<Arc<RetrogradeEntry>>) -> Self::PathView<'a> {
        Self::PathView {
            id: self.id,
            head: parent.map(|parent| {
                Arc::new(RetrogradeEntry {
                    entry: parent.entry.clone(),
                    child: self.head.clone(),
                })
            }),
            io: &self.io,
            cell: self.cell.clone(),
        }
    }
}

impl<'a, T: HasSchematic> InstancePathView<'a, T> {
    /// The ports of this instance.
    pub fn io(&self) -> PathView<<T::Io as SchematicType>::Data> {
        self.io.path_view(self.head.clone())
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<PathView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| {
                cell.path_view(Some(Arc::new(RetrogradeEntry {
                    entry: Arc::new(InstanceEntry {
                        id: self.id,
                        parent: self.head.clone(),
                    }),
                    child: None,
                })))
            })
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> PathView<Cell<T>> {
        self.try_cell().unwrap()
    }

    pub fn to_owned(&self) -> OwnedInstancePathView<T> {
        OwnedInstancePathView {
            id: self.id,
            head: self.head.clone(),
            io: (*self.io).clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T: HasSchematic> OwnedInstancePathView<T> {
    /// The ports of this instance.
    pub fn io(&self) -> PathView<<T::Io as SchematicType>::Data> {
        self.io.path_view(self.head.clone())
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> Result<PathView<'_, Cell<T>>> {
        self.cell
            .wait()
            .as_ref()
            .map(|cell| {
                cell.path_view(Some(Arc::new(RetrogradeEntry {
                    entry: Arc::new(InstanceEntry {
                        id: self.id,
                        parent: self.head.clone(),
                    }),
                    child: None,
                })))
            })
            .map_err(|e| e.clone())
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> PathView<Cell<T>> {
        self.try_cell().unwrap()
    }
}
