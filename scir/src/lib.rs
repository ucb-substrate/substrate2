use std::collections::HashMap;
use std::fmt::Display;

use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

pub(crate) mod validation;

pub enum Value {
    String(StringValue),
    Numeric(NumericExpr),
    Bool(BoolValue),
}

pub enum StringValue {
    Literal(ArcStr),
    Param(ArcStr),
}

pub enum BoolValue {
    Literal(bool),
    Param(ArcStr),
}

pub enum NumericExpr {
    Literal(Decimal),
    Param(ArcStr),
    BinOp {
        op: BinOp,
        left: Box<NumericExpr>,
        right: Box<NumericExpr>,
    },
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum Param {
    String { default: Option<ArcStr> },
    Number { default: Option<Decimal> },
    Bool { default: Option<bool> },
}

impl Param {
    pub fn has_default(&self) -> bool {
        match self {
            Self::String { default } => default.is_some(),
            Self::Number { default } => default.is_some(),
            Self::Bool { default } => default.is_some(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(u64);
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellId(u64);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node{}", self.0)
    }
}

impl Display for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cell{}", self.0)
    }
}

pub enum PrimitiveDevice {
    Res2 {
        pos: NodeId,
        neg: NodeId,
        value: NumericExpr,
    },
    Res3 {
        pos: NodeId,
        neg: NodeId,
        sub: NodeId,
        value: NumericExpr,
        model: Option<ArcStr>,
    },
}

impl PrimitiveDevice {
    pub fn nodes(&self) -> impl IntoIterator<Item = NodeId> {
        match self {
            Self::Res2 { pos, neg, .. } => vec![*pos, *neg],
            Self::Res3 { pos, neg, sub, .. } => vec![*pos, *neg, *sub],
        }
    }
}

pub struct Library {
    cell_id: u64,
    cells: HashMap<CellId, Cell>,
}

pub struct Port {
    node: NodeId,
}

pub struct NodeInfo {
    name: ArcStr,
}

/// An instance of a child cell placed inside a parent cell.
pub struct Instance {
    /// The ID of the child cell.
    cell: CellId,
    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    name: ArcStr,

    /// A map mapping port names to nodes.
    ///
    /// The ports are the ports of the **child** cell.
    /// The node identifiers are nodes of the **parent** cell.
    connections: HashMap<ArcStr, NodeId>,

    params: HashMap<ArcStr, Value>,
}

pub struct Cell {
    pub(crate) name: ArcStr,
    pub(crate) ports: Vec<Port>,
    pub(crate) nodes: HashMap<NodeId, NodeInfo>,
    pub(crate) instances: Vec<Instance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) params: HashMap<ArcStr, Param>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            cell_id: 0,
            cells: HashMap::new(),
        }
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.cell_id += 1;
        self.cells.insert(CellId(self.cell_id), cell);
    }
}

impl Default for Library {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Cell {
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            name: name.into(),
            ports: Vec::new(),
            nodes: HashMap::new(),
            instances: Vec::new(),
            primitives: Vec::new(),
            params: HashMap::new(),
        }
    }
}
