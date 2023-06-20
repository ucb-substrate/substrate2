use std::collections::HashMap;

use arcstr::ArcStr;
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
    Literal(ArcStr),
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

pub enum ParamType {
    String,
    Number,
    Bool,
}

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(u64);
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellId(u64);

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

pub struct Library {
    cell_id: u64,
    cells: HashMap<CellId, Cell>,
}

pub enum Direction {
    Input,
    Output,
    InOut,
}

pub struct Port {
    name: ArcStr,
    node: NodeId,
    dir: Direction,
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
    name: ArcStr,
    ports: Vec<Port>,
    nodes: HashMap<NodeId, NodeInfo>,
    instances: Vec<Instance>,
    primitives: Vec<PrimitiveDevice>,
    params: HashMap<ArcStr, ParamType>,
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
