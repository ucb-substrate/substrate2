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
//! SCIR supports single-bit wires and 1-dimensional buses.
//! Higher-dimensional buses should be flattened to 1-dimensional buses or single bits
//! when converting to SCIR.
//!
//! Single-bit wires are not exactly the same as single-bit buses:
//! A single bit wire named `x` will typically be exported to netlists as `x`,
//! unless the name contains reserved characters or is a keyword in the target
//! netlist format.
//! On the other hand, a bus named `x` with width 1
//! will typically be exported as `x[0]`.
//! Furthermore, whenever a 1-bit bus is used, a zero index must be specified.
//! However, single bit wires require that no index is specified.
//!
//! Zero-width buses are not supported.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::Display;

use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use slice::Slice;
use tracing::{span, Level};

use crate::slice::SliceRange;

pub mod slice;
pub(crate) mod validation;

#[cfg(test)]
pub(crate) mod tests;

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

/// An opaque signal identifier.
///
/// A signal ID created in the context of one cell must
/// *not* be used in the context of another cell.
/// You should instead create a new signal ID in the second cell.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignalId(u64);

impl From<Slice> for SignalId {
    #[inline]
    fn from(value: Slice) -> Self {
        value.signal()
    }
}

/// An opaque cell identifier.
///
/// A cell ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new cell ID in the second library.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellId(u64);

impl Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signal{}", self.0)
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
        /// The positive terminal.
        pos: Slice,
        /// The negative terminal.
        neg: Slice,
        /// The value of the resistance, in Ohms.
        value: Expr,
    },
    /// A 3-terminal resistor.
    Res3 {
        /// The positive terminal.
        pos: Slice,
        /// The negative terminal.
        neg: Slice,
        /// The substrate/body terminal.
        sub: Slice,
        /// The value of the resistance, in Ohms.
        value: Expr,
        /// The name of the resistor model to use.
        ///
        /// The available resistor models are usually specified by a PDK.
        model: Option<ArcStr>,
    },
}

impl PrimitiveDevice {
    /// An iterator over the nodes referenced in the device.
    pub(crate) fn nodes(&self) -> impl IntoIterator<Item = Slice> {
        match self {
            Self::Res2 { pos, neg, .. } => vec![*pos, *neg],
            Self::Res3 { pos, neg, sub, .. } => vec![*pos, *neg, *sub],
        }
    }
}

/// A concatenation of multiple slices.
pub struct Concat {
    parts: Vec<Slice>,
}

impl Concat {
    /// Creates a new concatenation from the given list of slices.
    #[inline]
    pub fn new(parts: Vec<Slice>) -> Self {
        Self { parts }
    }

    /// The width of this concatenation.
    ///
    /// Equal to the sum of the widths of all constituent slices.
    pub fn width(&self) -> usize {
        self.parts.iter().map(Slice::width).sum()
    }
}

impl From<Vec<Slice>> for Concat {
    #[inline]
    fn from(value: Vec<Slice>) -> Self {
        Self::new(value)
    }
}

impl From<Slice> for Concat {
    #[inline]
    fn from(value: Slice) -> Self {
        Self { parts: vec![value] }
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

/// A signal exposed by a cell.
pub struct Port {
    signal: SignalId,
}

/// Information about a signal in a cell.
pub struct SignalInfo {
    name: ArcStr,
    width: Option<usize>,
}

/// An instance of a child cell placed inside a parent cell.
pub struct Instance {
    /// The ID of the child cell.
    cell: CellId,
    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    name: ArcStr,

    /// A map mapping port names to connections.
    ///
    /// The ports are the ports of the **child** cell.
    /// The signal identifiers are signals of the **parent** cell.
    connections: HashMap<ArcStr, Concat>,

    /// A map mapping parameter names to expressions indicating their values.
    params: HashMap<ArcStr, Expr>,
}

/// A cell.
pub struct Cell {
    /// The last signal ID used.
    ///
    /// Initialized to 0 upon cell creation.
    signal_id: u64,
    pub(crate) name: ArcStr,
    pub(crate) ports: Vec<Port>,
    pub(crate) signals: HashMap<SignalId, SignalInfo>,
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
    ///
    /// Returns the ID of the newly added cell.
    pub fn add_cell(&mut self, cell: Cell) -> CellId {
        self.cell_id += 1;
        let id = CellId(self.cell_id);
        self.cells.insert(id, cell);
        id
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
            signal_id: 0,
            name: name.into(),
            ports: Vec::new(),
            signals: HashMap::new(),
            instances: Vec::new(),
            primitives: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Creates a new 1-bit signal in this cell.
    pub fn add_node(&mut self, name: impl Into<ArcStr>) -> Slice {
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signals.insert(
            id,
            SignalInfo {
                name: name.into(),
                width: None,
            },
        );
        Slice::new(id, None)
    }

    /// Creates a new 1-dimensional bus in this cell.
    pub fn add_bus(&mut self, name: impl Into<ArcStr>, width: usize) -> Slice {
        assert!(width > 0);
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signals.insert(
            id,
            SignalInfo {
                name: name.into(),
                width: Some(width),
            },
        );
        Slice::new(id, Some(SliceRange::with_width(width)))
    }

    /// Exposes the given signal as a port.
    ///
    /// If the signal is a bus, the entire bus is exposed.
    /// It is not possible to expose only a portion of a bus.
    /// Create two separate buses instead.
    pub fn expose_port(&mut self, signal: impl Into<SignalId>) {
        self.ports.push(Port {
            signal: signal.into(),
        });
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) {
        self.instances.push(instance);
    }

    /// Add the given [`PrimitiveDevice`] to the cell.
    #[inline]
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.primitives.push(device);
    }

    /// Add the given parameter to the cell.
    #[inline]
    pub fn add_param(&mut self, name: impl Into<ArcStr>, param: Param) {
        self.params.insert(name.into(), param);
    }
}

impl Instance {
    /// Create an instance of the given cell with the given name.
    pub fn new(name: impl Into<ArcStr>, cell: CellId) -> Self {
        Self {
            cell,
            name: name.into(),
            connections: HashMap::new(),
            params: HashMap::new(),
        }
    }

    /// Connect the given port of the child cell to the given node in the parent cell.
    #[inline]
    pub fn connect(&mut self, name: impl Into<ArcStr>, conn: impl Into<Concat>) {
        self.connections.insert(name.into(), conn.into());
    }

    /// Set the value of the given parameter.
    #[inline]
    pub fn set_param(&mut self, param: impl Into<ArcStr>, value: Expr) {
        self.params.insert(param.into(), value);
    }
}

impl From<Decimal> for Expr {
    fn from(value: Decimal) -> Self {
        Self::NumericLiteral(value)
    }
}

impl From<ArcStr> for Expr {
    fn from(value: ArcStr) -> Self {
        Self::StringLiteral(value)
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::BoolLiteral(value)
    }
}