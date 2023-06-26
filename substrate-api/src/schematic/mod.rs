//! Substrate's schematic generator framework.

use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use arcstr::ArcStr;
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;

use crate::block::Block;
use crate::context::Context;
use crate::error::Result;
use crate::generator::Generator;
use crate::pdk::Pdk;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An input port of hardware type `T`.
pub struct Input<T: Undirected>(T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An output port of hardware type `T`.
pub struct Output<T: Undirected>(T);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
/// An inout port of hardware type `T`.
pub struct InOut<T: Undirected>(T);

/// Indicates that a hardware type specifies signal directions all of its fields.
pub trait Directed: Flatten<Direction> {}
impl<T: Flatten<Direction>> Directed for T {}

/// A marker trait indicating that a hardware type does not specify signal directions.
pub trait Undirected {}

/// Flatten a structure into a list.
pub trait Flatten<T> {
    /// Flatten a structure into a list.
    fn flatten(&self) -> Vec<T>;
}

/// The length of the flattened list.
pub trait FlatLen {
    /// The length of the flattened list.
    fn len(&self) -> usize;
    /// Whether or not the flattened representation is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A hardware type.
pub trait HardwareType: FlatLen + Clone {
    /// The **Rust** type representing instances of this **hardware** type.
    type Data: HardwareData;

    /// Must consume exactly [`HardwareType::num_signals`] elements of the node list.
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]);
}

/// Hardware data.
///
/// An instance of a [`HardwareType`].
pub trait HardwareData: FlatLen + Flatten<Node> {}
impl<T> HardwareData for T where T: FlatLen + Flatten<Node> {}
impl<T> FlatLen for &T
where
    T: FlatLen,
{
    fn len(&self) -> usize {
        (*self).len()
    }
}
impl<T> Flatten<Node> for &T
where
    T: Flatten<Node>,
{
    fn flatten(&self) -> Vec<Node> {
        (*self).flatten()
    }
}

// BEGIN DIRECTIONS
impl<T: Undirected> AsRef<T> for Input<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Undirected> Deref for Input<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Undirected> Borrow<T> for Input<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}
impl<T> HardwareType for Input<T>
where
    T: Undirected + HardwareType,
    T::Data: Undirected,
{
    type Data = Input<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (Input(data), ids)
    }
}
impl<T: Undirected + FlatLen> FlatLen for Input<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Undirected + FlatLen> Flatten<Direction> for Input<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::Input; self.0.len()]
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for Input<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
    }
}

impl<T> HardwareType for Output<T>
where
    T: Undirected + HardwareType,
    T::Data: Undirected,
{
    type Data = Output<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (Output(data), ids)
    }
}
impl<T: Undirected + FlatLen> FlatLen for Output<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Undirected + FlatLen> Flatten<Direction> for Output<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::Output; self.0.len()]
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for Output<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
    }
}
impl<T: Undirected> AsRef<T> for Output<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Undirected> Deref for Output<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Undirected> Borrow<T> for Output<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> HardwareType for InOut<T>
where
    T: Undirected + HardwareType,
    T::Data: Undirected,
{
    type Data = InOut<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (InOut(data), ids)
    }
}
impl<T: Undirected + FlatLen> FlatLen for InOut<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Undirected + FlatLen> Flatten<Direction> for InOut<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::InOut; self.0.len()]
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for InOut<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
    }
}
impl<T: Undirected> AsRef<T> for InOut<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Undirected> Deref for InOut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Undirected> Borrow<T> for InOut<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}
// END DIRECTIONS

/// A type representing a single hardware wire.
#[derive(Debug, Default, Clone, Copy)]
pub struct Signal;

/// A single node in a circuit.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Node(u32);

/// A collection of [`Node`]s.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct NodeSet(HashSet<Node>);

pub(crate) struct NodeContext {
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

impl FlatLen for Signal {
    fn len(&self) -> usize {
        1
    }
}
impl HardwareType for Signal {
    type Data = Node;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
}

impl FlatLen for Node {
    fn len(&self) -> usize {
        1
    }
}
impl Flatten<Node> for Node {
    fn flatten(&self) -> Vec<Node> {
        vec![*self]
    }
}

impl FlatLen for () {
    fn len(&self) -> usize {
        0
    }
}

impl Flatten<Direction> for () {
    fn flatten(&self) -> Vec<Direction> {
        vec![]
    }
}
impl Undirected for () {}

impl HardwareType for () {
    type Data = ();
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        ((), ids)
    }
}

impl Undirected for Signal {}
impl Undirected for Node {}
impl Flatten<Node> for () {
    fn flatten(&self) -> Vec<Node> {
        vec![]
    }
}

/// A builder for creating a schematic cell.
#[allow(dead_code)]
pub struct CellBuilder<PDK: Pdk, T: Block> {
    pub(crate) id: CellId,
    pub(crate) ctx: Context<PDK>,
    pub(crate) node_ctx: NodeContext,
    pub(crate) instances: Vec<RawInstance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) phantom: PhantomData<T>,
}

impl<PDK: Pdk, T: Block> CellBuilder<PDK, T> {
    pub(crate) fn finish(self) -> RawCell {
        RawCell {
            id: self.id,
            primitives: self.primitives,
            instances: self.instances,
            ports: Default::default(),
            uf: self.node_ctx.into_inner(),
        }
    }
    /// Instantiate a schematic view of the given block.
    pub fn instantiate<I: HasSchematicImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_schematic(block.clone());
        let io = block.io();

        let ids = self.node_ctx.nodes(io.len());
        let (io, ids) = block.io().instantiate(&ids);
        assert!(ids.is_empty());

        let connections = io.flatten();

        let inst = Instance { cell, io };

        let raw = RawInstance {
            name: arcstr::literal!("unnamed"),
            child: inst.cell().raw.clone(),
            connections,
        };
        self.instances.push(raw);

        inst
    }

    /// Connect all signals in the given data instances.
    pub fn connect<D1, D2>(&mut self, s1: D1, s2: D2)
    where
        D1: HardwareData,
        D2: HardwareData,
        D1: Connect<D2>,
    {
        let s1f = s1.flatten();
        let s2f = s2.flatten();
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

/// A trait indicating that this type can be connected to T.
pub trait Connect<T> {}
impl<T> Connect<T> for T {}
impl<T: Undirected> Connect<T> for Input<T> {}
impl<T: Undirected> Connect<T> for Output<T> {}
impl<T: Undirected> Connect<T> for InOut<T> {}

// For analog circuits, we don't check directionality of connections.
impl<T: Undirected> Connect<Input<T>> for Output<T> {}
impl<T: Undirected> Connect<Input<T>> for InOut<T> {}
impl<T: Undirected> Connect<Output<T>> for Input<T> {}
impl<T: Undirected> Connect<Output<T>> for InOut<T> {}
impl<T: Undirected> Connect<InOut<T>> for Input<T> {}
impl<T: Undirected> Connect<InOut<T>> for Output<T> {}

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

/// Port directions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Input.
    Input,
    /// Output.
    Output,
    /// Input or output.
    ///
    /// Represents ports whose direction is not known
    /// at generator elaboration time.
    #[default]
    InOut,
}

/// A signal exposed by a cell.
#[allow(dead_code)]
pub struct Port {
    direction: Direction,
    node: Node,
}

type NodeUf = ena::unify::InPlaceUnificationTable<Node>;

/// A context-wide unique identifier for a cell.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CellId(usize);

/// A raw (weakly-typed) cell.
#[allow(dead_code)]
pub(crate) struct RawCell {
    // TODO CellId
    id: CellId,
    primitives: Vec<PrimitiveDevice>,
    instances: Vec<RawInstance>,

    // TODO: directions
    ports: Vec<Port>,
    uf: NodeUf,
}

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<T: HasSchematic> {
    /// The cell's input/output interface.
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
