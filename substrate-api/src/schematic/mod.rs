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
    Connect, FlatLen, Flatten, HasNameTree, NameBuf, Node, NodeContext, NodeUf, Port,
    SchematicData, SchematicType,
};
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
            let root = uf.find(node);
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
    /// Instantiate a schematic view of the given block.
    pub fn instantiate<I: HasSchematicImpl<PDK>>(&mut self, block: I) -> Instance<I> {
        let cell = self.ctx.generate_schematic(block.clone());
        let io = block.io();

        let ids = self.node_ctx.nodes(io.len());
        let (io_data, ids_rest) = block.io().instantiate(&ids);
        assert!(ids_rest.is_empty());

        let connections = io_data.flatten_vec();
        let names = io.flat_names(arcstr::format!("xinst{}", self.instances.len()));
        assert_eq!(connections.len(), names.len());

        self.node_names
            .extend(connections.iter().copied().zip(names));

        let inst = Instance { cell, io: io_data };

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
pub struct CellId(usize);

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

/// An instance of a schematic cell.
#[allow(dead_code)]
pub struct Instance<T: HasSchematic> {
    /// The cell's input/output interface.
    pub io: <T::Io as SchematicType>::Data,
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
