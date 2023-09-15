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
use std::ops::{Deref, DerefMut};

use arcstr::ArcStr;
use diagnostics::IssueSet;
use drivers::DriverIssue;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

pub mod blackbox;
pub mod merge;
pub mod netlist;
pub mod schema;
mod slice;

use crate::netlist::NetlistLibConversion;
use crate::schema::{NoSchema, NoSchemaError, Schema, ToSchema};
use crate::slice::Concat;
use crate::validation::ValidatorIssue;
pub use slice::{IndexOwned, Slice, SliceOne, SliceRange};

pub(crate) mod drivers;
pub(crate) mod validation;

#[cfg(test)]
pub(crate) mod tests;

/// An expression, often used in parameter assignments.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(
    Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct SignalId(u64);

impl From<Slice> for SignalId {
    #[inline]
    fn from(value: Slice) -> Self {
        value.signal()
    }
}

impl From<SliceOne> for SignalId {
    #[inline]
    fn from(value: SliceOne) -> Self {
        value.signal()
    }
}

/// A path to a nested [`Slice`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlicePath(SignalPath<Slice>);

/// A path to a nested [`SliceOne`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SliceOnePath(SignalPath<SliceOne>);

/// A path to a signal of type `S` in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
struct SignalPath<S> {
    path: InstancePath,
    tail: SignalPathTail<S>,
}

/// The end of a signal path.
pub type SignalPathTail<S> = PathElementKind<S, ArcStr>;

/// A path to an instance in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InstancePath {
    /// ID of the top cell.
    top: InstancePathTop,
    /// Path of SCIR instance IDs.
    instances: Vec<InstancePathElement>,
}

/// The top cell that an [`InstancePath`] is relative to.
pub type InstancePathTop = PathElementKind<CellId, ArcStr>;

/// An element of an [`InstancePath`].
pub type InstancePathElement = PathElementKind<InstanceIdElement, ArcStr>;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify]
pub enum PathElementKind<I, N> {
    /// An element addressed by SCIR identifiers.
    Scir(I),
    /// An element addressed by a name.
    Name(N),
}

/// An instance ID element within an [`InstancePath`] with additional metadata.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InstanceIdElement {
    id: InstanceId,
    child: ChildId,
}

impl Deref for InstanceIdElement {
    type Target = InstanceId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

/// A path of strings to an instance or signal in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamedPath(Vec<ArcStr>);

impl Deref for NamedPath {
    type Target = Vec<ArcStr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NamedPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl NamedPath {
    /// Consumes the [`NamedPath`], returning the path as a [`Vec<ArcStr>`].
    pub fn into_vec(self) -> Vec<ArcStr> {
        self.0
    }
}

/// An opaque cell identifier.
///
/// A cell ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new cell ID in the second library.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CellId(u64);

/// An opaque instance identifier.
///
/// An instance ID created in the context of one cell must
/// *not* be used in the context of another cell.
/// You should instead create a new instance ID in the second cell.
#[derive(
    Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct InstanceId(u64);

/// An opaque primitive identifier.
///
/// A primitive ID created in the context of one library must
/// *not* be used in the context of another library.
/// You should instead create a new primitive ID in the second library.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PrimitiveId(u64);

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

impl Display for PrimitiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "primitive{}", self.0)
    }
}
impl Display for ChildId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChildId::Cell(c) => c.fmt(f),
            ChildId::Primitive(p) => p.fmt(f),
        }
    }
}

/// A library of SCIR cells with primitive type P.
#[derive(Debug)]
pub struct LibraryBuilder<S: Schema = NoSchema> {
    /// The current cell ID counter.
    ///
    /// Initialized to 0 when the library is created.
    /// Should be incremented before assigning a new ID.
    cell_id: u64,

    /// The current primitive ID counter.
    ///
    /// Initialized to 0 when the library is created.
    /// Should be incremented before assigning a new ID.
    primitive_id: u64,

    /// The name of the library.
    name: ArcStr,

    /// A map of the cells in the library.
    cells: IndexMap<CellId, Cell>,

    /// A map of cell name to cell ID.
    ///
    /// SCIR makes no attempt to prevent duplicate cell names.
    name_map: HashMap<ArcStr, CellId>,

    /// A map of the primitives in the library.
    primitives: IndexMap<PrimitiveId, S::Primitive>,

    /// The ID of the top cell, if there is one.
    top: Option<CellId>,
}

impl<S: Schema<Primitive = impl Clone>> Clone for LibraryBuilder<S> {
    fn clone(&self) -> Self {
        Self {
            cell_id: self.cell_id,
            primitive_id: self.primitive_id,
            name: self.name.clone(),
            cells: self.cells.clone(),
            name_map: self.name_map.clone(),
            primitives: self.primitives.clone(),
            top: self.top,
        }
    }
}

/// A SCIR library that is guaranteed to be valid (with the exception of primitives,
/// which are opaque to SCIR).
///
/// The contents of the library cannot be mutated.
pub struct Library<S: Schema = NoSchema>(LibraryBuilder<S>);

impl<S: Schema> Clone for Library<S>
where
    LibraryBuilder<S>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: Schema> Deref for Library<S> {
    type Target = LibraryBuilder<S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Schema> Library<S> {
    /// Converts a [`Library<S>`] into a [`Library<C>`].
    pub fn convert_schema<C: Schema>(self) -> Result<Library<C>, S::Error>
    where
        S: ToSchema<C>,
    {
        Ok(Library(self.0.convert_schema()?))
    }
}

/// Issues encountered when validating a SCIR library.
#[derive(Debug, Clone)]
pub struct Issues {
    /// Correctness issues.
    pub correctness: IssueSet<ValidatorIssue>,
    /// Driver connectivity issues.
    pub drivers: IssueSet<DriverIssue>,
}

impl Display for Issues {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.correctness.is_empty() && self.drivers.is_empty() {
            write!(f, "no issues")?;
        }
        if !self.correctness.is_empty() {
            writeln!(f, "correctness issues:\n{}", self.correctness)?;
        }
        if !self.drivers.is_empty() {
            writeln!(f, "driver issues:\n{}", self.drivers)?;
        }
        Ok(())
    }
}

impl Issues {
    /// Returns `true` if there are warnings.
    pub fn has_warning(&self) -> bool {
        self.correctness.has_warning() || self.drivers.has_warning()
    }

    /// Returns `true` if there are errors.
    pub fn has_error(&self) -> bool {
        self.correctness.has_error() || self.drivers.has_error()
    }
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
    /// The ID representing this signal.
    pub id: SignalId,

    /// The name of this signal.
    pub name: ArcStr,

    /// The width of this signal, if this signal is a bus.
    ///
    /// For single-wire signals, this will be [`None`].
    pub width: Option<usize>,

    /// Set to `Some(..)` if this signal corresponds to a port.
    ///
    /// The contained `usize` represents the index at which the port
    /// corresponding to this signal starts.
    pub port: Option<usize>,
}

impl SignalInfo {
    /// The [`Slice`] representing this entire signal.
    #[inline]
    pub fn slice(&self) -> Slice {
        Slice::new(self.id, self.width.map(SliceRange::with_width))
    }
}

/// An instance of a child cell placed inside a parent cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// The ID of the child.
    child: ChildId,
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

/// The ID of an instance's child.
pub type ChildId = ChildKind<CellId, PrimitiveId>;

/// An enumeration of instance child types.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify]
pub enum ChildKind<C, P> {
    /// An instance of a cell.
    ///
    /// Indicates that the instance is an instantiation of a cell.
    Cell(C),
    /// A primitive.
    ///
    /// Indicates that the instance is an instantiation of a primitive.
    Primitive(P),
}

impl From<CellId> for ChildId {
    fn from(value: CellId) -> Self {
        Self::Cell(value)
    }
}

impl From<PrimitiveId> for ChildId {
    fn from(value: PrimitiveId) -> Self {
        Self::Primitive(value)
    }
}

/// A cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// The last signal ID used.
    ///
    /// Initialized to 0 upon cell creation.
    signal_id: u64,
    port_idx: usize,
    pub(crate) name: ArcStr,
    pub(crate) ports: IndexMap<ArcStr, Port>,
    pub(crate) signals: HashMap<SignalId, SignalInfo>,
    pub(crate) params: IndexMap<ArcStr, Param>,
    /// The last instance ID assigned.
    ///
    /// Initialized to 0 upon cell creation.
    instance_id: u64,
    pub(crate) instances: IndexMap<InstanceId, Instance>,
    /// A map of instance name to instance ID.
    ///
    /// SCIR makes no attempt to prevent duplicate instance names.
    name_map: HashMap<ArcStr, InstanceId>,
}

impl<S: Schema> LibraryBuilder<S> {
    /// Creates a new, empty library.
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            cell_id: 0,
            primitive_id: 0,
            name: name.into(),
            cells: IndexMap::new(),
            primitives: IndexMap::new(),
            name_map: HashMap::new(),
            top: None,
        }
    }

    /// Adds the given cell to the library.
    ///
    /// Returns the ID of the newly added cell.
    pub fn add_cell(&mut self, cell: Cell) -> CellId {
        let id = self.alloc_cell_id();
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
        id
    }

    #[inline]
    pub(crate) fn alloc_cell_id(&mut self) -> CellId {
        self.cell_id += 1;
        self.curr_cell_id()
    }

    #[inline]
    pub(crate) fn curr_cell_id(&self) -> CellId {
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

    /// Adds the given primitive to the library.
    ///
    /// Returns the ID of the newly added primitive.
    pub fn add_primitive(&mut self, primitive: S::Primitive) -> PrimitiveId {
        let id = self.alloc_primitive_id();
        self.primitives.insert(id, primitive);
        id
    }

    #[inline]
    pub(crate) fn alloc_primitive_id(&mut self) -> PrimitiveId {
        self.primitive_id += 1;
        self.curr_primitive_id()
    }

    #[inline]
    pub(crate) fn curr_primitive_id(&self) -> PrimitiveId {
        PrimitiveId(self.primitive_id)
    }

    /// Adds the given primitive to the library with the given primitive ID.
    ///
    /// Returns the ID of the newly added primitive.
    ///
    /// # Panics
    ///
    /// Panics if the ID is already in use.
    pub(crate) fn add_primitive_with_id(
        &mut self,
        id: impl Into<PrimitiveId>,
        primitive: S::Primitive,
    ) {
        let id = id.into();
        assert!(!self.primitives.contains_key(&id));
        self.primitive_id = std::cmp::max(id.0, self.primitive_id);
        self.primitives.insert(id, primitive);
    }

    /// Adds the given primitive to the library with the given primitive ID,
    /// overwriting an existing primitive with the same ID.
    ///
    /// This can lead to unintended effects.
    /// This method is intended for use only by Substrate libraries.
    ///
    /// # Panics
    ///
    /// Panics if the ID is **not** already in use.
    #[doc(hidden)]
    pub fn overwrite_primitive_with_id(
        &mut self,
        id: impl Into<PrimitiveId>,
        primitive: S::Primitive,
    ) {
        let id = id.into();
        assert!(self.primitives.contains_key(&id));
        self.primitive_id = std::cmp::max(id.0, self.primitive_id);
        self.primitives.insert(id, primitive);
    }

    /// Sets the top cell to the given cell ID.
    pub fn set_top(&mut self, cell: CellId) {
        self.top = Some(cell);
    }

    /// The ID of the top-level cell, if there is one.
    #[inline]
    pub fn top_cell(&self) -> Option<CellId> {
        self.top
    }

    /// Returns `true` if the given cell is the top cell.
    pub fn is_top(&self, cell: CellId) -> bool {
        self.top_cell().map(|c| c == cell).unwrap_or_default()
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
    /// For a non-panicking alternative, see [`try_cell`](LibraryBuilder::try_cell).
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

    /// Gets the cell with the given name.
    pub fn try_cell_named(&self, name: &str) -> Option<&Cell> {
        self.try_cell(*self.name_map.get(name)?)
    }

    /// Gets the cell ID corresponding to the given name.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given name.
    /// For a non-panicking alternative, see [`try_cell_id_named`](LibraryBuilder::try_cell_id_named).
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
        self.cells.iter().map(|(id, cell)| (*id, cell))
    }

    /// Gets the primitive with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no primitive has the given ID.
    /// For a non-panicking alternative, see [`try_primitive`](LibraryBuilder::try_primitive).
    pub fn primitive(&self, id: PrimitiveId) -> &S::Primitive {
        self.primitives.get(&id).unwrap()
    }

    /// Gets the primitive with the given ID.
    #[inline]
    pub fn try_primitive(&self, id: PrimitiveId) -> Option<&S::Primitive> {
        self.primitives.get(&id)
    }

    /// Iterates over the `(id, primitive)` pairs in this library.
    pub fn primitives(&self) -> impl Iterator<Item = (PrimitiveId, &S::Primitive)> {
        self.primitives
            .iter()
            .map(|(id, primitive)| (*id, primitive))
    }

    fn convert_instance_path_head(
        &self,
        conv: &NetlistLibConversion,
        top: CellId,
        instances: &[InstanceId],
    ) -> (Vec<ArcStr>, CellId) {
        let mut instance_names = Vec::new();
        let mut id = top;
        for instance in instances {
            let cell = self.cell(id);
            let inst = cell.instance(*instance);
            let inst_conv = conv.cells.get(&id).and_then(|c| c.instances.get(instance));
            if let Some(name) = inst_conv {
                instance_names.push(name.clone());
            } else {
                instance_names.push(inst.name().clone());
            }
            todo!()
            // id = inst.cell();
        }
        (instance_names, id)
    }

    fn convert_instance_path_top(&self, top: &InstancePathTop) -> Option<&Cell> {
        Some(match top {
            InstancePathTop::Id(id) => self.cell(*id),
            InstancePathTop::Name(name) => self.try_cell_named(name.as_ref())?,
        })
    }

    /// Converts an [`InstancePath`] to a [`NamedPath`] and the [`CellId`] of the final
    /// instance in the path.
    ///
    /// Uses `conv` to convert SCIR instance names to netlisted instances if a corresponding
    /// entry is found.
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    pub fn convert_instance_path(
        &self,
        conv: &NetlistLibConversion,
        path: &InstancePath,
    ) -> NamedPath {
        let mut named_path = NamedPath::new();

        let mut cell = self.convert_instance_path_top(&path.top);

        let mut ctr = 0;
        while let Some(cell) = cell {
            if ctr >= path.instances.len() {
                break;
            }
            let inst = match &path.instances[ctr] {
                InstancePathElement::Id(id) => cell.try_instance(id.id),
                InstancePathElement::Name(name) => cell.try_instance_named(&name),
            };

            let name = inst.map(|inst| inst.name()).unwrap_or_else(|| path.instances[ctr].)
            ctr += 1;
        }

        named_path
    }

    /// Converts an [`SignalPath`] to a [`NamedSignalPath`].
    ///
    /// Uses `conv` to convert SCIR instance names to netlisted instances if a corresponding
    /// entry is found.
    ///
    /// # Panics
    ///
    /// Panics if the path does not exist.
    pub fn convert_signal_path(&self) -> () {
        todo!();
    }

    /// Returns a simplified path to the provided node, bubbling up through IOs.
    ///
    /// # Panics
    ///
    /// Panics if the provided path does not exist or the path is terminated with
    /// [`SignalPathTail::Primitive`].
    pub fn simplify_path(&self) -> () {
        todo!()
        //if path.instances.is_empty() {
        //    return path;
        //}

        //let slice = if let SignalPathTail::Scir { slice, .. } = &mut path.tail {
        //    slice
        //} else {
        //    panic!("path is terminated with a primitive instance and cannot be simplified")
        //};
        //let mut cells = Vec::with_capacity(path.instances.len());
        //let mut cell = self.cell(path.top);

        //for inst in path.instances.iter() {
        //    let inst = &cell.contents().as_ref().unwrap_cell().instances[inst];
        //    todo!()
        //}

        //assert_eq!(cells.len(), path.instances.len());

        //for i in (0..cells.len()).rev() {
        //    let cell = self.cell(cells[i]);
        //    let info = cell.signal(slice.signal());
        //    if info.port.is_none() {
        //        path.instances.truncate(i + 1);
        //        return path;
        //    } else {
        //        let parent = if i == 0 {
        //            self.cell(path.top)
        //        } else {
        //            self.cell(cells[i - 1])
        //        };
        //        let inst = &parent.contents().as_ref().unwrap_cell().instances[&path.instances[i]];
        //        let idx = slice.index().unwrap_or_default();
        //        *slice = inst.connection(info.name.as_ref()).index(idx);
        //    }
        //}

        //path.instances = Vec::new();
        //path
    }

    /// Validate and construct a SCIR [`Library`].
    ///
    /// If errors are encountered during validation,
    /// returns an `Err(_)` containing the set of issues found.
    /// If no errors are encountered, returns an `Ok(_)` containing
    /// the SCIR library. Warnings and infos are discarded.
    ///
    /// If you want to inspect warnings/infos, consider using
    /// [`try_build`](LibraryBuilder::try_build) instead.
    #[inline]
    pub fn build(self) -> Result<Library<S>, Issues> {
        self.try_build().map(|ok| ok.0)
    }

    /// Validate and construct a SCIR [`Library`].
    ///
    /// If errors are encountered during validation,
    /// returns an `Err(_)` containing the set of issues found.
    /// If no errors are encountered, returns `Ok((library, issues))`.
    /// The issues returned will not have any errors, but may have
    /// warnings or infos.
    ///
    /// If you do not want to inspect warnings/infos, consider using
    /// [`build`](LibraryBuilder::build) instead.
    ///
    pub fn try_build(self) -> Result<(Library<S>, Issues), Issues> {
        let correctness = self.validate();
        let drivers = self.validate_drivers();
        let issues = Issues {
            correctness,
            drivers,
        };
        if issues.has_error() {
            Err(issues)
        } else {
            Ok((Library(self), issues))
        }
    }

    fn convert_inner<C: Schema, E>(
        self,
        convert_primitive: fn(<S as Schema>::Primitive) -> Result<<C as Schema>::Primitive, E>,
        convert_instance: fn(&mut Instance, &<S as Schema>::Primitive) -> Result<(), E>,
    ) -> Result<LibraryBuilder<C>, E> {
        let LibraryBuilder {
            cell_id,
            primitive_id,
            name,
            mut cells,
            name_map,
            primitives,
            top,
        } = self;

        for (_, cell) in cells.iter_mut() {
            for (_, instance) in cell.instances.iter_mut() {
                if let ChildId::Primitive(p) = instance.child {
                    if let Some(primitive) = primitives.get(&p) {
                        convert_instance(instance, primitive)?;
                    }
                }
            }
        }

        Ok(LibraryBuilder {
            cell_id,
            primitive_id,
            name,
            cells,
            name_map,
            primitives: primitives
                .into_iter()
                .map(|(k, v)| Ok((k, convert_primitive(v)?)))
                .collect::<Result<_, _>>()?,
            top,
        })
    }

    /// Converts a [`LibraryBuilder<S>`] to a [`LibraryBuilder<NoSchema>`], throwing an error if there
    /// are any primitives.
    pub fn drop_schema(self) -> Result<LibraryBuilder<NoSchema>, NoSchemaError> {
        self.convert_inner(|_| Err(NoSchemaError), |_, _| Err(NoSchemaError))
    }

    /// Converts a [`LibraryBuilder<S>`] into a [`LibraryBuilder<C>`].
    ///
    /// Instances associated with non-existent primitives will remain unchanged.
    pub fn convert_schema<C: Schema>(self) -> Result<LibraryBuilder<C>, S::Error>
    where
        S: ToSchema<C>,
    {
        self.convert_inner(S::convert_primitive, S::convert_instance)
    }
}

impl Cell {
    /// Creates a new cell with the given name.
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            signal_id: 0,
            port_idx: 0,
            name: name.into(),
            ports: IndexMap::new(),
            signals: HashMap::new(),
            params: IndexMap::new(),
            instance_id: 0,
            instances: IndexMap::new(),
        }
    }

    /// Creates a new 1-bit signal in this cell.
    pub fn add_node(&mut self, name: impl Into<ArcStr>) -> SliceOne {
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signals.insert(
            id,
            SignalInfo {
                id,
                port: None,
                name: name.into(),
                width: None,
            },
        );
        SliceOne::new(id, None)
    }

    /// Creates a new 1-dimensional bus in this cell.
    pub fn add_bus(&mut self, name: impl Into<ArcStr>, width: usize) -> Slice {
        assert!(width > 0);
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signals.insert(
            id,
            SignalInfo {
                id,
                port: None,
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
        info.port = Some(self.port_idx);
        self.port_idx += info.width.unwrap_or(1);
        self.ports
            .insert(info.name.clone(), Port { signal, direction });
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
        self.ports.iter().map(|(_, port)| port)
    }

    /// Get a port of this cell by name.
    ///
    /// # Panics
    ///
    /// Panics if the provided port does not exist.
    #[inline]
    pub fn port(&self, name: &str) -> &Port {
        self.ports.get(name).unwrap()
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
    /// Panics if no instance with the given ID exists.
    #[inline]
    pub fn instance(&self, id: InstanceId) -> &Instance {
        self.instances.get(&id).unwrap()
    }

    /// Get the instance associated with the given ID.
    #[inline]
    pub fn try_instance(&self, id: InstanceId) -> Option<&Instance> {
        self.instances.get(&id)
    }

    /// Gets the instance with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no instance has the given name.
    pub fn instance_named(&self, name: &str) -> &Instance {
        self.instance(*self.name_map.get(name).unwrap())
    }

    /// Gets the cell with the given name.
    pub fn try_instance_named(&self, name: &str) -> Option<&Instance> {
        self.try_instance(*self.name_map.get(name)?)
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) -> InstanceId {
        self.instance_id += 1;
        let id = InstanceId(self.instance_id);
        self.instances.insert(id, instance);
        id
    }
}

impl Instance {
    /// Create an instance of the given cell with the given name.
    pub fn new(name: impl Into<ArcStr>, child: impl Into<ChildId>) -> Self {
        Self {
            child: child.into(),
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
    pub fn child(&self) -> ChildId {
        self.child
    }

    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Returns a reference to this instance's connection map.
    #[inline]
    pub fn connections(&self) -> &HashMap<ArcStr, Concat> {
        &self.connections
    }

    /// Returns a mutable reference to this instance's connection map.
    #[inline]
    pub fn connections_mut(&mut self) -> &mut HashMap<ArcStr, Concat> {
        &mut self.connections
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

    /// Maps the connections to this instance to new port names.
    ///
    /// Exhibits undefined behavior if two connections map to the same port name.
    pub fn map_connections(&mut self, map_fn: impl Fn(ArcStr) -> ArcStr) {
        self.connections = self
            .connections
            .drain()
            .map(|(k, v)| (map_fn(k), v))
            .collect();
    }

    /// Returns a reference to this instance's parameter map.
    #[inline]
    pub fn params(&self) -> &HashMap<ArcStr, Expr> {
        &self.params
    }

    /// Returns a mutable reference to this instance's connection map.
    #[inline]
    pub fn params_mut(&mut self) -> &mut HashMap<ArcStr, Expr> {
        &mut self.params
    }
}

impl Port {
    /// The ID of the signal this port exposes.
    #[inline]
    pub fn signal(&self) -> SignalId {
        self.signal
    }
}
