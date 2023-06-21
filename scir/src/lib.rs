//! Schematic cell intermediate representation (SCIR).
//!
//! An intermediate-level representation of schematic cells and instances.
//!
//! Unlike higher-level Substrate APIs, the structures in this crate use
//! strings, rather than generics, to specify ports, connections, and parameters.
//!
//! This format is designed to be easy to generate from high-level APIs and
//! easy to parse from lower-level formats, such as SPICE or structural Verilog.
//!
//! SCIR modules are very simple: each node is a single net.
//! There are no buses/arrays.

use std::collections::HashMap;
use std::fmt::Display;

use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

pub(crate) mod validation;

/// An expression, often used in parameter assignments.
pub enum Expr {
    /// A numeric literal.
    NumericLiteral(Decimal),
    /// A boolean literal.
    BoolLiteral(bool),
    /// A string literal.
    StringLiteral(ArcStr),
    /// A variable/identifier in an expression.
    Var(ArcStr),
    /// A binary operation.
    BinOp {
        /// The operation type.
        op: BinOp,
        /// The left operand.
        left: Box<Expr>,
        /// The right operand.
        right: Box<Expr>,
    },
}

/// Binary operation types.
pub enum BinOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
}

/// A cell parameter.
pub enum Param {
    /// A string parameter.
    String {
        /// The default value.
        default: Option<ArcStr>,
    },
    /// A numeric parameter.
    Numeric {
        /// The default value.
        default: Option<Decimal>,
    },
    /// A boolean parameter.
    Bool {
        /// The default value.
        default: Option<bool>,
    },
}

impl Param {
    /// Whether or not the parameter has a default value.
    pub fn has_default(&self) -> bool {
        match self {
            Self::String { default } => default.is_some(),
            Self::Numeric { default } => default.is_some(),
            Self::Bool { default } => default.is_some(),
        }
    }
}

/// An opaque node identifier.
///
/// A node ID created in the context of one cell must
/// *not* be used in the context of another cell.
/// You should instead create a new node ID in the second cell.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(u64);

/// An opaque node identifier.
///
/// A cell ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new cell ID in the second library.
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

/// An enumeration of supported primitive devices.
pub enum PrimitiveDevice {
    /// An ideal 2-terminal resistor.
    Res2 {
        pos: NodeId,
        neg: NodeId,
        value: Expr,
    },
    /// A 3-terminal resistor.
    Res3 {
        pos: NodeId,
        neg: NodeId,
        sub: NodeId,
        value: Expr,
        model: Option<ArcStr>,
    },
}

impl PrimitiveDevice {
    /// An iterator over the nodes referenced in the device.
    pub(crate) fn nodes(&self) -> impl IntoIterator<Item = NodeId> {
        match self {
            Self::Res2 { pos, neg, .. } => vec![*pos, *neg],
            Self::Res3 { pos, neg, sub, .. } => vec![*pos, *neg, *sub],
        }
    }
}

/// A library of SCIR cells.
pub struct Library {
    /// The last ID assigned.
    ///
    /// Initialized to 0 when the library is created.
    cell_id: u64,

    /// A map of the cells in the library.
    cells: HashMap<CellId, Cell>,
}

/// A node exposed by a cell.
pub struct Port {
    node: NodeId,
}

/// Information about a node in a cell.
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

    /// A map mapping parameter names to expressions indicating their values.
    params: HashMap<ArcStr, Expr>,
}

/// A cell.
pub struct Cell {
    pub(crate) name: ArcStr,
    pub(crate) ports: Vec<Port>,
    pub(crate) nodes: HashMap<NodeId, NodeInfo>,
    pub(crate) instances: Vec<Instance>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
    pub(crate) params: HashMap<ArcStr, Param>,
}

impl Library {
    /// Creates a new, empty library.
    pub fn new() -> Self {
        Self {
            cell_id: 0,
            cells: HashMap::new(),
        }
    }

    /// Adds the given cell to the library.
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
    /// Creates a new cell with the given name.
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
