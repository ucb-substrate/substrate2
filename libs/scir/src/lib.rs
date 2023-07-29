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
use opacity::Opacity;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

pub mod merge;
mod slice;

pub use slice::{IndexOwned, Slice, SliceRange};
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
pub struct NodePath {
    /// The signal name.
    pub signal: SignalId,
    /// The signal index.
    ///
    /// [`None`] for single-wire signals.
    pub index: Option<usize>,
    /// Path of instance names.
    pub instances: Vec<InstanceId>,
    /// Name of the top cell.
    pub top: CellId,
}

/// An opaque cell identifier.
///
/// A cell ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new cell ID in the second library.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
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

/// An enumeration of supported primitive devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// A raw instance.
    ///
    /// This can be an instance of a subcircuit defined outside a SCIR library.
    RawInstance {
        /// The ports of the instance, as an ordered list.
        ports: Vec<Slice>,
        /// The name of the cell being instantiated.
        cell: ArcStr,
        /// Parameters to the cell being instantiated.
        params: HashMap<ArcStr, Expr>,
    },
}

impl PrimitiveDevice {
    /// An iterator over the nodes referenced in the device.
    pub(crate) fn nodes(&self) -> impl IntoIterator<Item = Slice> {
        match self {
            Self::Res2 { pos, neg, .. } => vec![*pos, *neg],
            Self::Res3 { pos, neg, sub, .. } => vec![*pos, *neg, *sub],
            Self::RawInstance { ports, .. } => ports.clone(),
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

/// A signal exposed by a cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    signal: SignalId,
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
    pub(crate) ports: Vec<Port>,
    pub(crate) signals: HashMap<SignalId, SignalInfo>,
    pub(crate) params: HashMap<ArcStr, Param>,
    pub(crate) contents: CellContents,
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
        self.cell_id = id.0;
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
        self.order.push(id);
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
    pub fn cell(&self, id: CellId) -> &Cell {
        self.cells.get(&id).unwrap()
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
    pub fn cell_id_named(&self, name: &str) -> CellId {
        match self.name_map.get(name) {
            Some(&cell) => cell,
            None => {
                tracing::error!("no cell named `{}` in SCIR library `{}`", name, self.name);
                panic!("no cell named `{}` in SCIR library `{}`", name, self.name);
            }
        }
    }

    /// Iterates over the `(id, cell)` pairs in this library.
    pub fn cells(&self) -> impl Iterator<Item = (CellId, &Cell)> {
        self.order.iter().map(|&id| (id, self.cell(id)))
    }

    /// Returns a simplified path to the provided node, bubbling up through IOs.
    ///
    /// # Panics
    ///
    /// Panics if the provided path does not exist.
    pub fn simplify_path(&self, mut path: NodePath) -> NodePath {
        if path.instances.is_empty() {
            return path;
        }

        let mut cells = Vec::with_capacity(path.instances.len());
        let mut cell = self.cell(path.top);

        for inst in path.instances.iter() {
            let inst = &cell.contents().as_ref().unwrap_clear().instances[inst];
            cells.push(inst.cell);
            cell = self.cell(inst.cell);
        }

        assert_eq!(cells.len(), path.instances.len());

        for i in (0..cells.len()).rev() {
            let cell = self.cell(cells[i]);
            let info = cell.signal(path.signal);
            if !info.port {
                path.instances.truncate(i + 1);
                return path;
            } else {
                let parent = if i == 0 {
                    self.cell(path.top)
                } else {
                    self.cell(cells[i - 1])
                };
                let inst = &parent.contents().as_ref().unwrap_clear().instances[&path.instances[i]];
                let idx = if let Some(idx) = path.index { idx } else { 0 };
                let slice = inst.connection(info.name.as_ref()).index(idx);
                path.signal = slice.signal();
                path.index = slice.range().map(|range| range.start);
            }
        }

        NodePath {
            instances: Vec::new(),
            ..path
        }
    }
}

impl Cell {
    /// Creates a new whitebox cell with the given name.
    pub fn new_whitebox(name: impl Into<ArcStr>) -> Self {
        Self {
            signal_id: 0,
            name: name.into(),
            ports: Vec::new(),
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
            ports: Vec::new(),
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
    pub fn expose_port(&mut self, signal: impl Into<SignalId>) {
        let signal = signal.into();
        self.signals.get_mut(&signal).unwrap().port = true;
        self.ports.push(Port { signal });
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