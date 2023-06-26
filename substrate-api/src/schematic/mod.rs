use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;

use arcstr::ArcStr;
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;

use crate::block::{AnalogIo, Block};
use crate::context::Context;
use crate::error::Result;
use crate::generator::Generator;
use crate::pdk::Pdk;

pub(crate) mod example;

/// A block that has a schematic.
pub trait HasSchematic: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Send + Sync;
}

/// A block that has a schematic for process design kit `PDK`.
pub trait HasSchematicImpl<PDK: Pdk>: HasSchematic {
    /// Generates the block's layout.
    fn schematic(
        &self,
        io: <<Self as Block>::Io as HardwareType>::Data,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> Result<Self::Data>;
}

#[derive(Debug, Clone, Copy)]
pub struct Signal;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Node(u32);

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct NodeSet(HashSet<Node>);

pub struct NodeContext {
    uf: NodeUf,
}

impl NodeContext {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            uf: Default::default(),
        }
    }
    pub(crate) fn node(&mut self) -> Node {
        let id = self.uf.new_key(Default::default());
        self.uf.union_value(id, NodeSet([id].into()));
        id
    }
    #[inline]
    pub fn into_inner(self) -> NodeUf {
        self.uf
    }
    pub fn nodes(&mut self, n: usize) -> Vec<Node> {
        (0..n).map(|_| self.node()).collect()
    }
    pub(crate) fn connect(&mut self, n1: Node, n2: Node) {
        self.uf.union(n1, n2);
    }
}

pub trait HardwareType: Clone {
    type Data: HardwareData;

    /// Returns the number of nodes used to represent this type.
    fn num_signals(&self) -> u64;

    /// Must consume exactly [`HardwareType::num_signals`] elements of the node list.
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]);
}

pub trait HardwareData {
    /// Must have a length equal to the corresponding [`HardwareType`]'s `num_signals`.
    fn flatten(&self) -> Vec<Node>;
    fn flatten_hierarchical(&self) -> Vec<Vec<Node>>;
}

impl HardwareType for Signal {
    type Data = Node;
    fn num_signals(&self) -> u64 {
        1
    }
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
}

impl HardwareData for Node {
    fn flatten(&self) -> Vec<Node> {
        vec![*self]
    }
    fn flatten_hierarchical(&self) -> Vec<Vec<Node>> {
        vec![vec![*self]]
    }
    // Provide suggested node names.
    // fn names(&self, base: Option<&str>) -> Vec<String>;
}

impl HardwareType for () {
    type Data = ();
    fn num_signals(&self) -> u64 {
        0
    }
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        ((), ids)
    }
}

impl HardwareData for () {
    fn flatten(&self) -> Vec<Node> {
        Vec::new()
    }
    fn flatten_hierarchical(&self) -> Vec<Vec<Node>> {
        Vec::new()
    }
}

pub struct CellBuilder<PDK, T: Block> {
    pub(crate) id: CellId,
    pub(crate) ctx: Context<PDK>,
    pub(crate) node_ctx: NodeContext,
    pub(crate) instances: Vec<RawInstance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) phantom: PhantomData<T>,
}

impl<PDK: Pdk, T: Block> CellBuilder<PDK, T> {
    pub fn finish(self) -> RawCell {
        RawCell {
            id: self.id,
            primitives: self.primitives,
            instances: self.instances,
            ports: Default::default(),
            uf: self.node_ctx.into_inner(),
        }
    }
    pub fn instantiate<I: HasSchematicImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_schematic(block.clone());
        let io = block.io();

        let ids = self.node_ctx.nodes(io.num_signals() as usize);
        let (io, ids) = block.io().instantiate(&ids);
        assert!(ids.is_empty());

        let connections = io.flatten_hierarchical();

        let inst = Instance { cell, io };

        let raw = RawInstance {
            name: arcstr::literal!("unnamed"),
            child: inst.cell().raw.clone(),
            connections,
        };
        self.instances.push(raw);

        inst
    }

    pub fn connect<D: HardwareData>(&mut self, s1: &D, s2: &D) {
        let s1f = s1.flatten();
        let s2f = s2.flatten();
        assert_eq!(s1f.len(), s2f.len());
        s1f.into_iter().zip(s2f).for_each(|(a, b)| {
            self.node_ctx.connect(a, b);
        });
    }

    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.primitives.push(device);
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Cell<T: HasSchematic> {
    pub block: T,
    pub data: T::Data,
    pub(crate) raw: Arc<RawCell>,
}

impl<T: HasSchematic> Cell<T> {
    pub fn new(block: T, data: T::Data, raw: Arc<RawCell>) -> Self {
        Self { block, data, raw }
    }
}

pub struct RawInstance {
    name: ArcStr,
    child: Arc<RawCell>,
    connections: Vec<Vec<Node>>,
}

pub enum Direction {
    Input,
    Output,
    InOut,
}

pub struct Port {
    direction: Direction,
    nodes: Vec<Node>,
}

type NodeUf = ena::unify::InPlaceUnificationTable<Node>;

/// A context-wide unique identifier for a cell.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CellId(usize);

pub struct RawCell {
    // TODO CellId
    id: CellId,
    primitives: Vec<PrimitiveDevice>,
    instances: Vec<RawInstance>,

    // TODO: directions
    ports: Vec<Port>,
    uf: NodeUf,
}

#[allow(dead_code)]
pub struct Instance<T: HasSchematic> {
    pub io: <T::Io as HardwareType>::Data,
    cell: Arc<OnceCell<Result<Cell<T>>>>,
}

impl<T: HasSchematic> Instance<T> {
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
}

pub enum PrimitiveDevice {
    Res2 {
        pos: Node,
        neg: Node,
        value: Decimal,
    },
}

impl ena::unify::UnifyValue for NodeSet {
    type Error = ena::unify::NoError;

    fn unify_values(value1: &Self, value2: &Self) -> std::result::Result<Self, Self::Error> {
        Ok(Self(&value1.0 | &value2.0))
    }
}

impl ena::unify::UnifyKey for Node {
    type Value = NodeSet;
    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "Node"
    }
}

/// A wrapper around schematic-specific context data.
#[derive(Debug, Default, Clone)]
pub struct SchematicContext {
    next_id: CellId,
    pub(crate) gen: Generator,
}

impl std::ops::Add<usize> for CellId {
    type Output = CellId;

    fn add(self, rhs: usize) -> Self::Output {
        CellId(self.0 + rhs)
    }
}

impl std::ops::AddAssign<usize> for CellId {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl SchematicContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_id(&mut self) -> CellId {
        let tmp = self.next_id;
        self.next_id += 1;
        tmp
    }
}
