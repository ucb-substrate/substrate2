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
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use arcstr::ArcStr;
use opacity::Opacity;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

pub mod merge;
mod slice;

pub use slice::{IndexOwned, Slice, SliceRange};
pub(crate) mod drivers;
pub(crate) mod validation;

#[cfg(test)]
pub(crate) mod tests;

/// An expression, often used in parameter assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A path to a node in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignalPath {
    /// The signal ID.
    pub signal: SignalId,
    /// The signal index.
    ///
    /// [`None`] for single-wire signals.
    pub index: Option<usize>,
    /// The path to the containing instance.
    pub path: InstancePath,
}

/// A path of strings to a node in a SCIR library.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedSignalPath {
    /// The signal name.
    pub signal: ArcStr,
    /// The signal index.
    ///
    /// [`None`] for single-wire signals.
    pub index: Option<usize>,
    /// The path to the containing instance.
    pub instances: NamedInstancePath,
}

/// A path to an instance in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstancePath {
    /// Path of instance names.
    pub instances: Vec<InstanceId>,
    /// Name of the top cell.
    pub top: CellId,
}

/// A path of strings to an instance in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamedInstancePath(pub Vec<ArcStr>);

impl Deref for NamedInstancePath {
    type Target = Vec<ArcStr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An opaque cell identifier.
///
/// A cell ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new cell ID in the second library.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellId(u64);

/// An opaque instance identifier.
///
/// A instance ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new instance ID in the second library.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstanceId(u64);

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

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inst{}", self.0)
    }
}

/// A primitive device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveDevice {
    /// The kind (resistor, capacitor, etc.) of this primitive device.
    pub kind: PrimitiveDeviceKind,
    /// An unordered set of parameters, represented as key-value pairs.
    pub params: HashMap<ArcStr, Expr>,
}

impl PrimitiveDevice {
    /// Create a new primitive device with the given parameters.
    #[inline]
    pub fn from_params(
        kind: PrimitiveDeviceKind,
        params: impl Into<HashMap<ArcStr, Expr>>,
    ) -> Self {
        Self {
            kind,
            params: params.into(),
        }
    }

    /// Create a new primitive device with no parameters.
    #[inline]
    pub fn new(kind: PrimitiveDeviceKind) -> Self {
        Self {
            kind,
            params: Default::default(),
        }
    }
}

impl From<PrimitiveDeviceKind> for PrimitiveDevice {
    #[inline]
    fn from(value: PrimitiveDeviceKind) -> Self {
        Self::new(value)
    }
}

impl From<PrimitiveDevice> for PrimitiveDeviceKind {
    fn from(value: PrimitiveDevice) -> Self {
        value.kind
    }
}

/// An enumeration of supported primitive kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveDeviceKind {
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
    /// A raw instance.
    ///
    /// This can be an instance of a subcircuit defined outside a SCIR library.
    RawInstance {
        /// The ports of the instance, as an ordered list.
        ports: Vec<Slice>,
        /// The name of the cell being instantiated.
        cell: ArcStr,
    },
}

impl PrimitiveDevice {
    /// An iterator over the nodes referenced in the device.
    pub(crate) fn nodes(&self) -> impl IntoIterator<Item = Slice> {
        match &self.kind {
            PrimitiveDeviceKind::Res2 { pos, neg, .. } => vec![*pos, *neg],
            PrimitiveDeviceKind::Res3 { pos, neg, sub, .. } => vec![*pos, *neg, *sub],
            PrimitiveDeviceKind::RawInstance { ports, .. } => ports.clone(),
        }
    }
}

/// A concatenation of multiple slices.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Iterate over the parts of this concatenation.
    #[inline]
    pub fn parts(&self) -> impl Iterator<Item = &Slice> {
        self.parts.iter()
    }
}

impl FromIterator<Slice> for Concat {
    fn from_iter<T: IntoIterator<Item = Slice>>(iter: T) -> Self {
        let parts = iter.into_iter().collect();
        Self { parts }
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

/// Information about the top-level cell in a SCIR library.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Top {
    /// The ID of the top-level cell.
    cell: CellId,
    /// Whether or not to inline the top-level cell during netlisting.
    inline: bool,
}

/// A library of SCIR cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// The current ID counter.
    ///
    /// Initialized to 0 when the library is created.
    /// Should be incremented before assigning a new ID.
    cell_id: u64,

    /// The name of the library.
    name: ArcStr,

    /// A map of the cells in the library.
    cells: HashMap<CellId, Cell>,

    /// A map of cell name to cell ID.
    ///
    /// SCIR makes no attempt to prevent duplicate cell names.
    name_map: HashMap<ArcStr, CellId>,

    /// Information about the top cell, if there is one.
    top: Option<Top>,

    /// The order in which cells were added to this library.
    order: Vec<CellId>,
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

impl Direction {
    /// Returns the flipped direction.
    ///
    /// [`Direction::InOut`] is unchanged by flipping.
    ///
    /// # Examples
    ///
    /// ```
    /// use scir::Direction;
    /// assert_eq!(Direction::Input.flip(), Direction::Output);
    /// assert_eq!(Direction::Output.flip(), Direction::Input);
    /// assert_eq!(Direction::InOut.flip(), Direction::InOut);
    /// ```
    #[inline]
    pub fn flip(&self) -> Self {
        match *self {
            Self::Input => Self::Output,
            Self::Output => Self::Input,
            Self::InOut => Self::InOut,
        }
    }

    /// Test if two nodes of the respective directions are allowed be connected
    /// to each other.
    ///
    /// # Examples
    ///
    /// ```
    /// use scir::Direction;
    /// assert_eq!(Direction::Input.is_compatible_with(Direction::Output), true);
    /// assert_eq!(Direction::Output.is_compatible_with(Direction::Output), false);
    /// assert_eq!(Direction::Output.is_compatible_with(Direction::InOut), true);
    /// ```
    pub fn is_compatible_with(&self, other: Direction) -> bool {
        use Direction::*;

        #[allow(clippy::match_like_matches_macro)]
        match (*self, other) {
            (Output, Output) => false,
            _ => true,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Output => write!(f, "output"),
            Self::Input => write!(f, "input"),
            Self::InOut => write!(f, "inout"),
        }
    }
}

/// A signal exposed by a cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    signal: SignalId,
    direction: Direction,
}

/// Information about a signal in a cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInfo {
    /// The name of this signal.
    pub name: ArcStr,

    /// The width of this signal, if this signal is a bus.
    ///
    /// For single-wire signals, this will be [`None`].
    pub width: Option<usize>,

    /// Set to `true` if this signal corresponds to a port.
    port: bool,
}

/// An instance of a child cell placed inside a parent cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// The (possibly blackboxed) contents of a SCIR cell.
pub type CellContents = Opacity<BlackboxContents, CellInner>;

/// The contents of a blackbox cell.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlackboxElement {
    /// A reference to a [`Slice`].
    Slice(Slice),
    /// A raw, opaque [`String`].
    RawString(String),
}

/// A cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// The last signal ID used.
    ///
    /// Initialized to 0 upon cell creation.
    signal_id: u64,
    pub(crate) name: ArcStr,
    pub(crate) ports: Ports,
    pub(crate) signals: HashMap<SignalId, SignalInfo>,
    pub(crate) params: HashMap<ArcStr, Param>,
    pub(crate) contents: CellContents,
}

/// A set of signals exposed by a cell.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Ports {
    /// Signals exposed by a cell.
    ports: Vec<Port>,
    /// Mapping from a port name to its index in `ports`.
    name_map: HashMap<ArcStr, usize>,
}

/// The inner contents of a non-blackbox cell.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CellInner {
    /// The last instance ID assigned.
    ///
    /// Initialized to 0 upon cell creation.
    instance_id: u64,
    pub(crate) instances: HashMap<InstanceId, Instance>,
    /// The order in which instances are added to this cell.
    pub(crate) order: Vec<InstanceId>,
    pub(crate) primitives: Vec<PrimitiveDevice>,
}

impl Library {
    /// Creates a new, empty library.
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            cell_id: 0,
            name: name.into(),
            cells: HashMap::new(),
            name_map: HashMap::new(),
            top: None,
            order: Vec::new(),
        }
    }

    /// Adds the given cell to the library.
    ///
    /// Returns the ID of the newly added cell.
    pub fn add_cell(&mut self, cell: Cell) -> CellId {
        let id = self.alloc_id();
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
        self.order.push(id);
        id
    }

    #[inline]
    pub(crate) fn alloc_id(&mut self) -> CellId {
        self.cell_id += 1;
        self.curr_id()
    }

    #[inline]
    pub(crate) fn curr_id(&self) -> CellId {
        CellId(self.cell_id)
    }

    /// Adds the given cell to the library with the given cell ID.
    ///
    /// Returns the ID of the newly added cell.
    ///
    /// # Panics
    ///
    /// Panics if the ID is already in use.
    pub(crate) fn add_cell_with_id(&mut self, id: impl Into<CellId>, cell: Cell) {
        let id = id.into();
        assert!(!self.cells.contains_key(&id));
        self.cell_id = std::cmp::max(id.0, self.cell_id);
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
        self.order.push(id);
    }

    /// Adds the given cell to the library with the given cell ID,
    /// overwriting an existing cell with the same ID.
    ///
    /// This can lead to unintended effects.
    /// This method is intended for use only by Substrate libraries.
    ///
    /// # Panics
    ///
    /// Panics if the ID is **not** already in use.
    #[doc(hidden)]
    pub fn overwrite_cell_with_id(&mut self, id: impl Into<CellId>, cell: Cell) {
        let id = id.into();
        assert!(self.cells.contains_key(&id));
        self.cell_id = std::cmp::max(id.0, self.cell_id);
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
    }

    /// Sets the top cell to the given cell ID.
    ///
    /// If `inline` is set to `true`, the top cell will
    /// be flattened during netlisting.
    pub fn set_top(&mut self, cell: CellId, inline: bool) {
        self.top = Some(Top { cell, inline });
    }

    /// The ID of the top-level cell, if there is one.
    #[inline]
    pub fn top_cell(&self) -> Option<CellId> {
        self.top.map(|t| t.cell)
    }

    /// Whether or not to inline the top-level cell.
    ///
    /// If no top cell has been set, returns `false`.
    #[inline]
    pub fn inline_top(&self) -> bool {
        self.top.map(|t| t.inline).unwrap_or_default()
    }

    /// Returns `true` if the given cell should be emitted inline during netlisting.
    ///
    /// At the moment, the only cell that may be inlined is the top-level cell.
    /// However, this is subject to change.
    pub fn should_inline(&self, cell: CellId) -> bool {
        self.inline_top() && self.top_cell().map(|c| c == cell).unwrap_or_default()
    }

    /// The name of the library.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Gets the cell with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given ID.
    /// For a non-panicking alternative, see [`try_cell`](Library::try_cell).
    pub fn cell(&self, id: CellId) -> &Cell {
        self.cells.get(&id).unwrap()
    }

    /// Gets the cell with the given ID.
    #[inline]
    pub fn try_cell(&self, id: CellId) -> Option<&Cell> {
        self.cells.get(&id)
    }

    /// Gets the cell with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given name.
    pub fn cell_named(&self, name: &str) -> &Cell {
        self.cell(*self.name_map.get(name).unwrap())
    }

    /// Gets the cell ID corresponding to the given name.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given name.
    /// For a non-panicking alternative, see [`try_cell_id_named`](Library::try_cell_id_named).
    pub fn cell_id_named(&self, name: &str) -> CellId {
        match self.name_map.get(name) {
            Some(&cell) => cell,
            None => {
                tracing::error!("no cell named `{}` in SCIR library `{}`", name, self.name);
                panic!("no cell named `{}` in SCIR library `{}`", name, self.name);
            }
        }
    }

    /// Gets the cell ID corresponding to the given name.
    pub fn try_cell_id_named(&self, name: &str) -> Option<CellId> {
        self.name_map.get(name).copied()
    }

    /// Iterates over the `(id, cell)` pairs in this library.
    pub fn cells(&self) -> impl Iterator<Item = (CellId, &Cell)> {
        self.order.iter().map(|&id| (id, self.cell(id)))
    }

    fn convert_instance_path_inner(&self, path: &InstancePath) -> (NamedInstancePath, &Cell) {
        let mut instances = Vec::new();
        let mut cell = self.cell(path.top);
        for instance in &path.instances {
            let inst = cell.instance(*instance);
            instances.push(inst.name().clone());
            cell = self.cell(inst.cell());
        }
        (NamedInstancePath(instances), cell)
    }

    /// Converts a [`SignalPath`] to a [`NamedSignalPath`].
    pub fn convert_signal_path(&self, path: &SignalPath) -> NamedSignalPath {
        let (instances, cell) = self.convert_instance_path_inner(&path.path);
        NamedSignalPath {
            signal: cell.signal(path.signal).name.clone(),
            index: path.index,
            instances,
        }
    }

    /// Converts an [`InstancePath`] to a [`NamedInstancePath`].
    pub fn convert_instance_path(&self, path: &InstancePath) -> NamedInstancePath {
        self.convert_instance_path_inner(path).0
    }

    /// Returns a simplified path to the provided node, bubbling up through IOs.
    ///
    /// # Panics
    ///
    /// Panics if the provided path does not exist.
    pub fn simplify_path(&self, mut path: SignalPath) -> SignalPath {
        if path.path.instances.is_empty() {
            return path;
        }

        let mut cells = Vec::with_capacity(path.path.instances.len());
        let mut cell = self.cell(path.path.top);

        for inst in path.path.instances.iter() {
            let inst = &cell.contents().as_ref().unwrap_clear().instances[inst];
            cells.push(inst.cell);
            cell = self.cell(inst.cell);
        }

        assert_eq!(cells.len(), path.path.instances.len());

        for i in (0..cells.len()).rev() {
            let cell = self.cell(cells[i]);
            let info = cell.signal(path.signal);
            if !info.port {
                path.path.instances.truncate(i + 1);
                return path;
            } else {
                let parent = if i == 0 {
                    self.cell(path.path.top)
                } else {
                    self.cell(cells[i - 1])
                };
                let inst =
                    &parent.contents().as_ref().unwrap_clear().instances[&path.path.instances[i]];
                let idx = if let Some(idx) = path.index { idx } else { 0 };
                let slice = inst.connection(info.name.as_ref()).index(idx);
                path.signal = slice.signal();
                path.index = slice.range().map(|range| range.start);
            }
        }

        path.path.instances = Vec::new();
        path
    }
}

impl Cell {
    /// Creates a new whitebox cell with the given name.
    pub fn new_whitebox(name: impl Into<ArcStr>) -> Self {
        Self {
            signal_id: 0,
            name: name.into(),
            ports: Ports::new(),
            signals: HashMap::new(),
            params: HashMap::new(),
            contents: CellContents::Clear(Default::default()),
        }
    }

    /// Creates a new blackbox cell with the given name and contents.
    ///
    /// This does not automatically expose ports.
    /// See [`Cell::expose_port`] for more information on exposing ports.
    pub fn new_blackbox(name: impl Into<ArcStr>) -> Self {
        Self {
            signal_id: 0,
            name: name.into(),
            ports: Ports::new(),
            signals: HashMap::new(),
            params: HashMap::new(),
            contents: CellContents::Opaque(Default::default()),
        }
    }

    /// Creates a new 1-bit signal in this cell.
    pub fn add_node(&mut self, name: impl Into<ArcStr>) -> Slice {
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signals.insert(
            id,
            SignalInfo {
                port: false,
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
                port: false,
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
    ///
    /// # Panics
    ///
    /// Panics if the provided signal does not exist.
    pub fn expose_port(&mut self, signal: impl Into<SignalId>, direction: Direction) {
        let signal = signal.into();
        let info = self.signals.get_mut(&signal).unwrap();
        info.port = true;
        self.ports.push(info.name.clone(), signal, direction);
    }

    /// Returns a reference to the contents of this cell.
    #[inline]
    pub fn contents(&self) -> &CellContents {
        &self.contents
    }

    /// Returns a mutable reference to the contents of this cell.
    #[inline]
    pub fn contents_mut(&mut self) -> &mut CellContents {
        &mut self.contents
    }

    /// Add the given parameter to the cell.
    #[inline]
    pub fn add_param(&mut self, name: impl Into<ArcStr>, param: Param) {
        self.params.insert(name.into(), param);
    }

    /// The name of the cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Iterate over the ports of this cell.
    #[inline]
    pub fn ports(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter()
    }

    /// Get a port of this cell by name.
    #[inline]
    pub fn port(&self, name: &str) -> &Port {
        self.ports.get_named(name)
    }

    /// Iterate over the signals of this cell.
    #[inline]
    pub fn signals(&self) -> impl Iterator<Item = (SignalId, &SignalInfo)> {
        self.signals.iter().map(|x| (*x.0, x.1))
    }

    /// Get the signal associated with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no signal with the given ID exists.
    #[inline]
    pub fn signal(&self, id: SignalId) -> &SignalInfo {
        self.signals.get(&id).unwrap()
    }

    /// Get the instance associated with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no instance with the given ID exists (including if the cell is a blackbox).
    #[inline]
    pub fn instance(&self, id: InstanceId) -> &Instance {
        self.contents()
            .as_ref()
            .unwrap_clear()
            .instances
            .get(&id)
            .unwrap()
    }

    /// Sets the contents of the cell.
    #[inline]
    pub fn set_contents(&mut self, contents: CellContents) {
        self.contents = contents;
    }

    /// Add the given instance to the cell.
    ///
    /// # Panics
    ///
    /// Panics if this cell is a blackbox.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) -> InstanceId {
        self.contents.as_mut().unwrap_clear().add_instance(instance)
    }

    /// Add the given [`PrimitiveDevice`] to the cell.
    ///
    /// # Panics
    ///
    /// Panics if this cell is a blackbox.
    #[inline]
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.contents
            .as_mut()
            .unwrap_clear()
            .primitives
            .push(device);
    }

    /// Add the given [`BlackboxElement`] to the cell.
    ///
    /// # Panics
    ///
    /// Panics if this cell is **not** blackbox.
    #[inline]
    pub fn add_blackbox_elem(&mut self, elem: impl Into<BlackboxElement>) {
        self.contents
            .as_mut()
            .unwrap_opaque()
            .elems
            .push(elem.into());
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

    /// The ID of the child cell.
    ///
    /// This instance represents an instantiation of the child cell in a parent cell.
    #[inline]
    pub fn cell(&self) -> CellId {
        self.cell
    }

    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Iterate over the connections of this instance.
    #[inline]
    pub fn connections(&self) -> impl Iterator<Item = (&ArcStr, &Concat)> {
        self.connections.iter()
    }

    /// The connection to the given port.
    ///
    /// # Panics
    ///
    /// Panics if there is no connection for the given port.
    #[inline]
    pub fn connection<'a>(&'a self, port: &str) -> &'a Concat {
        &self.connections[port]
    }
}

impl Ports {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    /// Pushes a port to the set of ports.
    pub(crate) fn push(&mut self, name: impl Into<ArcStr>, signal: SignalId, direction: Direction) {
        self.ports.push(Port { signal, direction });
        self.name_map.insert(name.into(), self.ports.len() - 1);
    }

    pub(crate) fn get_named(&self, name: &str) -> &Port {
        let idx = self.name_map.get(name).unwrap();
        &self.ports[*idx]
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.ports.len()
    }
}

impl Port {
    /// The ID of the signal this port exposes.
    #[inline]
    pub fn signal(&self) -> SignalId {
        self.signal
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

impl CellInner {
    /// Returns a new, empty inner cell.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) -> InstanceId {
        self.instance_id += 1;
        let id = InstanceId(self.instance_id);
        self.instances.insert(id, instance);
        self.order.push(id);
        id
    }

    /// Add the given [`PrimitiveDevice`] to the cell.
    #[inline]
    pub fn add_primitive(&mut self, device: PrimitiveDevice) {
        self.primitives.push(device);
    }

    /// Iterate over the primitive devices of this cell.
    #[inline]
    pub fn primitives(&self) -> impl Iterator<Item = &PrimitiveDevice> {
        self.primitives.iter()
    }

    /// Iterate over the instances of this cell.
    #[inline]
    pub fn instances(&self) -> impl Iterator<Item = (InstanceId, &Instance)> {
        self.order.iter().map(|x| (*x, &self.instances[x]))
    }

    /// Iterate over mutable references to the instances of this cell.
    #[inline]
    pub fn instances_mut(&mut self) -> impl Iterator<Item = (InstanceId, &mut Instance)> {
        self.instances.iter_mut().map(|x| (*x.0, x.1))
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

impl From<Slice> for BlackboxElement {
    #[inline]
    fn from(value: Slice) -> Self {
        Self::Slice(value)
    }
}

impl From<String> for BlackboxContents {
    fn from(value: String) -> Self {
        Self {
            elems: vec![BlackboxElement::RawString(value)],
        }
    }
}
