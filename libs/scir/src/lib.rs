//! Schematic cell intermediate representation (SCIR).
//!
//! An intermediate-level representation of schematic cells and instances.
//!
//! Unlike higher-level Substrate APIs, the structures in this crate use
//! strings, rather than generics, to specify ports and connections.
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

pub mod merge;
pub mod schema;
mod slice;

use crate::schema::{FromSchema, NoSchema, NoSchemaError, Schema};
use crate::validation::ValidatorIssue;
pub use slice::{Concat, IndexOwned, NamedSlice, NamedSliceOne, Slice, SliceOne, SliceRange};

pub mod drivers;
pub mod validation;

#[cfg(test)]
pub(crate) mod tests;

/// A value of a parameter.
#[enumify::enumify(no_as_ref, no_as_mut)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamValue {
    /// A string parameter value.
    String(ArcStr),
    /// A numeric parameter value.
    Numeric(Decimal),
}

impl From<ArcStr> for ParamValue {
    fn from(value: ArcStr) -> Self {
        Self::String(value)
    }
}

impl From<Decimal> for ParamValue {
    fn from(value: Decimal) -> Self {
        Self::Numeric(value)
    }
}

impl Display for ParamValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamValue::String(s) => write!(f, "{}", s),
            ParamValue::Numeric(n) => write!(f, "{}", n),
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
pub struct SlicePath(SignalPath<Slice, NamedSlice>);

impl SlicePath {
    /// Returns the instance path associated with this path.
    pub fn instances(&self) -> &InstancePath {
        &self.0.instances
    }
    /// Returns a mutable pointer to the instance path associated with this path.
    pub fn instances_mut(&mut self) -> &mut InstancePath {
        &mut self.0.instances
    }
    /// Returns the tail of this path.
    ///
    /// The tail includes information on the signal that this path addresses.
    pub fn tail(&self) -> &SignalPathTail<Slice, NamedSlice> {
        &self.0.tail
    }
}

/// A path to a nested [`SliceOne`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SliceOnePath(SignalPath<SliceOne, NamedSliceOne>);

impl SliceOnePath {
    /// Creates a new [`SliceOnePath`].
    pub fn new(
        instances: InstancePath,
        tail: impl Into<SignalPathTail<SliceOne, NamedSliceOne>>,
    ) -> Self {
        Self(SignalPath {
            instances,
            tail: tail.into(),
        })
    }
    /// Returns the instance path associated with this path.
    pub fn instances(&self) -> &InstancePath {
        &self.0.instances
    }
    /// Returns a mutable pointer to the instance path associated with this path.
    pub fn instances_mut(&mut self) -> &mut InstancePath {
        &mut self.0.instances
    }
    /// Returns the tail of this path.
    ///
    /// The tail includes information on the signal that this path addresses.
    pub fn tail(&self) -> &SignalPathTail<SliceOne, NamedSliceOne> {
        &self.0.tail
    }
}

/// A path to a signal of type `I` or `N` in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
struct SignalPath<I, N> {
    instances: InstancePath,
    tail: SignalPathTail<I, N>,
}

/// The end of a signal path.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify(generics_only)]
pub enum SignalPathTail<I, N> {
    /// A signal addressed by ID.
    Id(I),
    /// A signal addressed by name.
    Name(N),
}

impl From<Slice> for SignalPathTail<Slice, NamedSlice> {
    fn from(value: Slice) -> Self {
        SignalPathTail::Id(value)
    }
}

impl From<NamedSlice> for SignalPathTail<Slice, NamedSlice> {
    fn from(value: NamedSlice) -> Self {
        SignalPathTail::Name(value)
    }
}

impl SignalPathTail<Slice, NamedSlice> {
    /// The range of indices indexed by this signal path tail.
    ///
    /// Returns [`None`] if this slice represents a single bit wire.
    pub fn range(&self) -> Option<SliceRange> {
        match self {
            SignalPathTail::Id(slice) => slice.range(),
            SignalPathTail::Name(slice) => slice.range(),
        }
    }
}

impl From<SliceOne> for SignalPathTail<SliceOne, NamedSliceOne> {
    fn from(value: SliceOne) -> Self {
        SignalPathTail::Id(value)
    }
}

impl From<NamedSliceOne> for SignalPathTail<SliceOne, NamedSliceOne> {
    fn from(value: NamedSliceOne) -> Self {
        SignalPathTail::Name(value)
    }
}

impl SignalPathTail<SliceOne, NamedSliceOne> {
    /// The index this single-bit signal path tail contains.
    pub fn index(&self) -> Option<usize> {
        match self {
            SignalPathTail::Id(slice) => slice.index(),
            SignalPathTail::Name(slice) => slice.index(),
        }
    }
}

/// A path to an instance in a SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InstancePath {
    /// The top cell of the path.
    top: InstancePathCell,
    /// Path of SCIR instances.
    elems: Vec<InstancePathElement>,
}

impl Deref for InstancePath {
    type Target = Vec<InstancePathElement>;

    fn deref(&self) -> &Self::Target {
        &self.elems
    }
}

impl DerefMut for InstancePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elems
    }
}

impl InstancePath {
    /// Creates a new empty [`InstancePath`] with the given reference point.
    pub fn new(top: impl Into<InstancePathCell>) -> Self {
        Self {
            top: top.into(),
            elems: Vec::new(),
        }
    }

    /// Pushes a new instance to the path.
    pub fn push(&mut self, elem: impl Into<InstancePathElement>) {
        self.elems.push(elem.into())
    }

    /// Pushes an iterator of instances to the path.
    pub fn push_iter<E: Into<InstancePathElement>>(&mut self, elems: impl IntoIterator<Item = E>) {
        for elem in elems {
            self.push(elem);
        }
    }

    /// Returns the top cell of the path.
    pub fn top(&self) -> &InstancePathCell {
        &self.top
    }

    /// Returns `true` if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// Creates a [`SliceOnePath`] by appending the provided `tail`.
    pub fn slice_one(
        self,
        tail: impl Into<SignalPathTail<SliceOne, NamedSliceOne>>,
    ) -> SliceOnePath {
        SliceOnePath(SignalPath {
            instances: self,
            tail: tail.into(),
        })
    }
}

/// The cell within an [`InstancePath`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify]
pub enum InstancePathCell {
    /// A cell addressed by ID.
    Id(CellId),
    /// A cell addressed by name.
    Name(ArcStr),
}

impl From<CellId> for InstancePathCell {
    fn from(value: CellId) -> Self {
        Self::Id(value)
    }
}

impl<S: Into<ArcStr>> From<S> for InstancePathCell {
    fn from(value: S) -> Self {
        Self::Name(value.into())
    }
}

/// An element of an [`InstancePath`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify]
pub enum InstancePathElement {
    /// An instance addressed by ID.
    Id(InstanceId),
    /// An instance addressed by name.
    Name(ArcStr),
}

impl From<InstanceId> for InstancePathElement {
    fn from(value: InstanceId) -> Self {
        Self::Id(value)
    }
}

impl<S: Into<ArcStr>> From<S> for InstancePathElement {
    fn from(value: S) -> Self {
        Self::Name(value.into())
    }
}

/// A path to an instance in a SCIR library with annotated metadata.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AnnotatedInstancePath {
    /// ID or name of the top cell.
    ///
    /// If the name corresponds to a SCIR cell, this should always be an ID.
    pub top: InstancePathCell,
    /// Path of SCIR instance IDs.
    pub instances: Vec<AnnotatedInstancePathElement>,
}

impl AnnotatedInstancePath {
    fn bot(&self) -> Option<CellId> {
        self.instances
            .last()
            .and_then(|inst| inst.child?.into_cell())
            .or_else(|| self.top.get_id().copied())
    }
}

/// An element of an [`AnnotatedInstancePath`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AnnotatedInstancePathElement {
    /// The underlying instance path element.
    pub elem: InstancePathElement,
    /// The child associated with this element, if it exists.
    pub child: Option<ChildId>,
}

impl From<AnnotatedInstancePath> for InstancePath {
    fn from(value: AnnotatedInstancePath) -> Self {
        let AnnotatedInstancePath { top, instances } = value;

        InstancePath {
            top,
            elems: instances
                .into_iter()
                .map(|instance| instance.elem)
                .collect(),
        }
    }
}

/// A path of strings to an instance or signal in a SCIR library.
#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
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
    fn new() -> Self {
        Self::default()
    }

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

/// A library of SCIR cells with schema `S`.
pub struct LibraryBuilder<S: Schema + ?Sized = NoSchema> {
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

    /// A map of the cells in the library.
    cells: IndexMap<CellId, Cell>,

    /// A map of cell name to cell ID.
    ///
    /// Cell names are only guaranteed to be unique in a validated [`Library`].
    name_map: HashMap<ArcStr, CellId>,

    /// A map of the primitives in the library.
    primitives: IndexMap<PrimitiveId, S::Primitive>,

    /// The ID of the top cell, if there is one.
    top: Option<CellId>,
}

impl<S: Schema + ?Sized> Default for LibraryBuilder<S> {
    fn default() -> Self {
        Self {
            cell_id: 0,
            primitive_id: 0,
            cells: IndexMap::new(),
            primitives: IndexMap::new(),
            name_map: HashMap::new(),
            top: None,
        }
    }
}

impl<S: Schema<Primitive = impl Clone> + ?Sized> Clone for LibraryBuilder<S> {
    fn clone(&self) -> Self {
        Self {
            cell_id: self.cell_id,
            primitive_id: self.primitive_id,
            cells: self.cells.clone(),
            name_map: self.name_map.clone(),
            primitives: self.primitives.clone(),
            top: self.top,
        }
    }
}

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for LibraryBuilder<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("LibraryBuilder");
        let _ = builder.field("cell_id", &self.cell_id);
        let _ = builder.field("primitive_id", &self.primitive_id);
        let _ = builder.field("cells", &self.cells);
        let _ = builder.field("name_map", &self.name_map);
        let _ = builder.field("primitives", &self.primitives);
        let _ = builder.field("top", &self.top);
        builder.finish()
    }
}

/// A SCIR library that is guaranteed to be valid (with the exception of primitives,
/// which are opaque to SCIR).
///
/// The contents of the library cannot be mutated.
pub struct Library<S: Schema + ?Sized = NoSchema>(LibraryBuilder<S>);

impl<S: Schema<Primitive = impl std::fmt::Debug> + ?Sized> std::fmt::Debug for Library<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Library");
        let _ = builder.field("0", &self.0);
        builder.finish()
    }
}

impl<S: Schema + ?Sized> Clone for Library<S>
where
    LibraryBuilder<S>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: Schema + ?Sized> Deref for Library<S> {
    type Target = LibraryBuilder<S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Schema + ?Sized> Library<S> {
    /// Converts a [`Library<S>`] to a [`Library<NoSchema>`], throwing an error if there
    /// are any primitives.
    pub fn drop_schema(self) -> Result<Library<NoSchema>, NoSchemaError> {
        Ok(Library(self.0.drop_schema()?))
    }

    /// Converts a [`Library<S>`] into a [`LibraryBuilder<C>`].
    ///
    /// A [`LibraryBuilder`] is created to indicate that validation must be done again
    /// to ensure errors were not introduced during the conversion.
    pub fn convert_schema<C: Schema + ?Sized>(self) -> Result<LibraryBuilder<C>, C::Error>
    where
        C: FromSchema<S>,
    {
        self.0.convert_schema()
    }

    /// Converts this library into a [`LibraryBuilder`] that can be modified.
    pub fn into_builder(self) -> LibraryBuilder<S> {
        self.0
    }
}

impl<S: Schema<Primitive = impl Clone> + ?Sized> Library<S> {
    /// Creates a new SCIR library containing only the named cell and its children
    /// from an existing library.
    pub fn from_cell_named(lib: &Self, cell: &str) -> Self {
        LibraryBuilder::from_cell_named(lib, cell).build().unwrap()
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

impl std::error::Error for Issues {}

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
    /// at generator elaboration time (e.g. the output of a tristate buffer).
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

    /// Returns `true` if this signal is exposed as a port.
    pub fn is_port(&self) -> bool {
        self.port.is_some()
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
    /// The connected signals are signals of the **parent** cell.
    connections: HashMap<ArcStr, Concat>,
}

/// The ID of an instance's child.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[enumify::enumify(no_as_ref, no_as_mut)]
pub enum ChildId {
    /// A child cell.
    Cell(CellId),
    /// A child primitive.
    ///
    /// The contents of instance's with a child primitive are opaque to SCIR.
    Primitive(PrimitiveId),
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
    /// A map of instance name to instance ID.
    ///
    /// Signal names are only guaranteed to be unique in a validated [`Library`].
    signal_name_map: HashMap<ArcStr, SignalId>,
    /// The last instance ID assigned.
    ///
    /// Initialized to 0 upon cell creation.
    instance_id: u64,
    pub(crate) instances: IndexMap<InstanceId, Instance>,
    /// A map of instance name to instance ID.
    ///
    /// Instance names are only guaranteed to be unique in a validated [`Library`].
    instance_name_map: HashMap<ArcStr, InstanceId>,
}

/// Metadata associated with the conversion from a SCIR library to a netlist.
#[derive(Debug, Clone, Default)]
pub struct NetlistLibConversion {
    /// Conversion metadata for each cell in the SCIR library.
    pub cells: HashMap<CellId, NetlistCellConversion>,
}

impl NetlistLibConversion {
    /// Creates a new [`NetlistLibConversion`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata associated with the conversion from a SCIR cell to a netlisted subcircuit.
#[derive(Debug, Clone, Default)]
pub struct NetlistCellConversion {
    /// The netlisted names of SCIR instances.
    pub instances: HashMap<InstanceId, ArcStr>,
}

impl NetlistCellConversion {
    /// Creates a new [`NetlistCellConversion`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: Schema + ?Sized> LibraryBuilder<S> {
    /// Creates a new, empty library.
    pub fn new() -> Self {
        Self::default()
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
                tracing::error!("no cell named `{}`", name);
                panic!("no cell named `{}`", name);
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

    fn convert_instance_path_cell(&self, top: &InstancePathCell) -> Option<(CellId, &Cell)> {
        Some(match top {
            InstancePathCell::Id(id) => (*id, self.cell(*id)),
            InstancePathCell::Name(name) => (
                *self.name_map.get(name)?,
                self.try_cell_named(name.as_ref())?,
            ),
        })
    }

    /// Annotates an [`InstancePath`] with additional metadata.
    ///
    /// Finds cell IDs associated with instances in the path if they are in the SCIR
    /// library, and converts the top of the path to a cell ID if possible.
    pub fn annotate_instance_path(&self, path: InstancePath) -> AnnotatedInstancePath {
        let mut annotated_elems = Vec::new();

        let (top, mut cell) = if let Some((id, cell)) = self.convert_instance_path_cell(&path.top) {
            (Some(id), Some(cell))
        } else {
            (None, None)
        };

        for instance in path.elems {
            if let Some(cell_inner) = cell {
                let child = match &instance {
                    InstancePathElement::Id(id) => cell_inner.try_instance(*id),
                    InstancePathElement::Name(name) => cell_inner.try_instance_named(name),
                }
                .map(|inst| inst.child);

                annotated_elems.push(AnnotatedInstancePathElement {
                    elem: instance,
                    child,
                });

                cell = match child {
                    Some(ChildId::Cell(c)) => self.try_cell(c),
                    _ => None,
                };
            } else {
                annotated_elems.push(AnnotatedInstancePathElement {
                    elem: instance.clone(),
                    child: None,
                });
            }
        }

        AnnotatedInstancePath {
            top: top.map(|top| top.into()).unwrap_or(path.top),
            instances: annotated_elems,
        }
    }

    /// Annotates an instance path with additional metadata, such as whether
    /// each instance in the path corresponds to an actual SCIR instance.
    pub fn convert_annotated_instance_path(
        &self,
        conv: Option<&NetlistLibConversion>,
        path: AnnotatedInstancePath,
    ) -> NamedPath {
        let mut named_path = NamedPath::new();

        let (top_id, top) = if let Some((top_id, top)) = self.convert_instance_path_cell(&path.top)
        {
            (Some(top_id), Some(top))
        } else {
            (None, None)
        };

        for (i, instance) in path.instances.iter().enumerate() {
            match &instance.elem {
                InstancePathElement::Id(id) => {
                    let inst = if i == 0 {
                        top
                    } else {
                        self.try_cell(path.instances[i - 1].child.unwrap().unwrap_cell())
                    }
                    .unwrap()
                    .instance(*id);

                    let name = conv
                        .and_then(|conv| {
                            Some(
                                conv.cells
                                    .get(&if i == 0 {
                                        top_id?
                                    } else {
                                        path.instances[i - 1].child.unwrap().unwrap_cell()
                                    })?
                                    .instances
                                    .get(id)?
                                    .clone(),
                            )
                        })
                        .unwrap_or(inst.name().clone());
                    named_path.push(name);
                }
                InstancePathElement::Name(name) => {
                    named_path.push(name.clone());
                }
            }
        }

        named_path
    }

    /// Converts an [`InstancePath`] to a [`NamedPath`].
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    pub fn convert_instance_path(&self, path: InstancePath) -> NamedPath {
        let annotated_path = self.annotate_instance_path(path);

        self.convert_annotated_instance_path(None, annotated_path)
    }

    /// Converts an [`InstancePath`] to a [`NamedPath`], using the provided `conv`
    /// to modify instance names that were converted during netlisting.
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    pub fn convert_instance_path_with_conv(
        &self,
        conv: &NetlistLibConversion,
        path: InstancePath,
    ) -> NamedPath {
        let annotated_path = self.annotate_instance_path(path);

        self.convert_annotated_instance_path(Some(conv), annotated_path)
    }

    /// Converts a [`SliceOnePath`] to a [`NamedPath`], using the provided `conv`
    /// to modify instance names that were converted during netlisting.
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    fn convert_slice_one_path_inner(
        &self,
        conv: Option<&NetlistLibConversion>,
        path: SliceOnePath,
        index_fmt: impl FnOnce(&ArcStr, Option<usize>) -> ArcStr,
    ) -> NamedPath {
        let SignalPath { instances, tail } = path.0;
        let annotated_path = self.annotate_instance_path(instances);

        let bot = self.cell(annotated_path.bot().unwrap());

        let (name, index) = match &tail {
            SignalPathTail::Id(id) => (&bot.signal(id.signal()).name, id.index()),
            SignalPathTail::Name(name) => (name.signal(), name.index()),
        };

        let mut name_path = self.convert_annotated_instance_path(conv, annotated_path);
        name_path.push(index_fmt(name, index));

        name_path
    }

    /// Converts a [`SliceOnePath`] to a [`NamedPath`].
    /// to modify instance names that were converted during netlisting.
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    pub fn convert_slice_one_path(
        &self,
        path: SliceOnePath,
        index_fmt: impl FnOnce(&ArcStr, Option<usize>) -> ArcStr,
    ) -> NamedPath {
        self.convert_slice_one_path_inner(None, path, index_fmt)
    }

    /// Converts a [`SliceOnePath`] to a [`NamedPath`], using the provided `conv`
    /// to modify instance names that were converted during netlisting.
    ///
    /// # Panics
    ///
    /// Panics if the path contains instance or cell IDs that do not exist.
    pub fn convert_slice_one_path_with_conv(
        &self,
        conv: &NetlistLibConversion,
        path: SliceOnePath,
        index_fmt: impl FnOnce(&ArcStr, Option<usize>) -> ArcStr,
    ) -> NamedPath {
        self.convert_slice_one_path_inner(Some(conv), path, index_fmt)
    }

    /// Returns a simplified path to the provided node, bubbling up through IOs.
    ///
    /// # Panics
    ///
    /// Panics if the provided path does not exist within the SCIR library.
    pub fn simplify_path(&self, path: SliceOnePath) -> SliceOnePath {
        if path.instances().is_empty() {
            return path;
        }
        let SignalPath {
            instances,
            mut tail,
        } = path.0;

        let mut annotated_instances = self.annotate_instance_path(instances);
        let top = self
            .convert_instance_path_cell(&annotated_instances.top)
            .map(|(_, cell)| cell);

        for i in (0..annotated_instances.instances.len()).rev() {
            let parent = if i == 0 {
                top.unwrap()
            } else {
                self.cell(
                    annotated_instances.instances[i - 1]
                        .child
                        .unwrap()
                        .unwrap_cell(),
                )
            };
            match annotated_instances.instances[i].child.unwrap() {
                ChildId::Cell(id) => {
                    let cell = self.cell(id);
                    let info = match &tail {
                        SignalPathTail::Id(id) => cell.signal(id.signal()),
                        SignalPathTail::Name(name) => cell.signal_named(name.signal()),
                    };
                    if info.port.is_none() {
                        annotated_instances.instances.truncate(i + 1);
                        return SliceOnePath(SignalPath {
                            instances: annotated_instances.into(),
                            tail,
                        });
                    } else {
                        let inst = parent
                            .instance_from_path_element(&annotated_instances.instances[i].elem);
                        let idx = tail.index().unwrap_or_default();
                        tail = SignalPathTail::Id(inst.connection(info.name.as_ref()).index(idx));
                    }
                }
                ChildId::Primitive(_) => {
                    let inst =
                        parent.instance_from_path_element(&annotated_instances.instances[i].elem);
                    tail = SignalPathTail::Id(match &tail {
                        SignalPathTail::Id(_) => {
                            panic!("only paths to named primitive ports can be simplified")
                        }
                        SignalPathTail::Name(name) => inst
                            .connection(name.signal())
                            .index(name.index().unwrap_or_default()),
                    });
                }
            }
        }

        annotated_instances.instances = Vec::new();
        SliceOnePath(SignalPath {
            instances: annotated_instances.into(),
            tail,
        })
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

    fn convert_inner<C: Schema + ?Sized, E>(
        self,
        convert_primitive: fn(<S as Schema>::Primitive) -> Result<<C as Schema>::Primitive, E>,
        convert_instance: fn(&mut Instance, &<S as Schema>::Primitive) -> Result<(), E>,
    ) -> Result<LibraryBuilder<C>, E> {
        let LibraryBuilder {
            cell_id,
            primitive_id,
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
    pub fn convert_schema<C: Schema + ?Sized>(self) -> Result<LibraryBuilder<C>, C::Error>
    where
        C: FromSchema<S>,
    {
        self.convert_inner(C::convert_primitive, C::convert_instance)
    }
}

impl<S: Schema<Primitive = impl Clone> + ?Sized> LibraryBuilder<S> {
    /// Creates a new SCIR library builder containing only the named cell and its children
    /// from an existing library builder.
    pub fn from_cell_named(lib: &Self, cell: &str) -> Self {
        let mut new_lib = LibraryBuilder::new();
        let mut cells = vec![(lib.cell_id_named(cell), lib.cell_named(cell))];
        while let Some((id, cell)) = cells.pop() {
            for (_, inst) in cell.instances() {
                match inst.child {
                    ChildId::Primitive(id) => {
                        let prim = lib.primitive(id);
                        new_lib.add_primitive_with_id(id, prim.clone());
                    }
                    ChildId::Cell(cell) => {
                        cells.push((cell, lib.cell(cell)));
                    }
                }
            }
            new_lib.add_cell_with_id(id, cell.clone());
        }
        new_lib
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
            signal_name_map: HashMap::new(),
            instance_id: 0,
            instances: IndexMap::new(),
            instance_name_map: HashMap::new(),
        }
    }

    fn add_signal(&mut self, name: ArcStr, width: Option<usize>) -> SignalId {
        self.signal_id += 1;
        let id = SignalId(self.signal_id);
        self.signal_name_map.insert(name.clone(), id);
        self.signals.insert(
            id,
            SignalInfo {
                id,
                port: None,
                name,
                width,
            },
        );
        id
    }

    /// Creates a new 1-bit signal in this cell.
    pub fn add_node(&mut self, name: impl Into<ArcStr>) -> SliceOne {
        let id = self.add_signal(name.into(), None);
        SliceOne::new(id, None)
    }

    /// Creates a new 1-dimensional bus in this cell.
    pub fn add_bus(&mut self, name: impl Into<ArcStr>, width: usize) -> Slice {
        assert!(width > 0);
        let id = self.add_signal(name.into(), Some(width));
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

        // If this signal was already marked as a port, no need to do anything.
        if info.port.is_none() {
            info.port = Some(self.port_idx);
            self.port_idx += info.width.unwrap_or(1);
            self.ports
                .insert(info.name.clone(), Port { signal, direction });
        }
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

    /// Get the signal associated with the given ID.
    #[inline]
    pub fn try_signal(&self, id: SignalId) -> Option<&SignalInfo> {
        self.signals.get(&id)
    }

    /// Get the signal associated with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no signal with the given name exists.
    #[inline]
    pub fn signal_named(&self, name: &str) -> &SignalInfo {
        self.signal(*self.signal_name_map.get(name).unwrap())
    }

    /// Get the signal associated with the given ID.
    #[inline]
    pub fn try_signal_named(&self, name: &str) -> Option<&SignalInfo> {
        self.try_signal(*self.signal_name_map.get(name)?)
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
        self.instance(*self.instance_name_map.get(name).unwrap())
    }

    /// Gets the instance with the given name.
    pub fn try_instance_named(&self, name: &str) -> Option<&Instance> {
        self.try_instance(*self.instance_name_map.get(name)?)
    }

    /// Gets the instance associated with the given path element.
    pub fn instance_from_path_element(&self, elem: &InstancePathElement) -> &Instance {
        match elem {
            InstancePathElement::Id(id) => self.instance(*id),
            InstancePathElement::Name(name) => self.instance_named(name),
        }
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance) -> InstanceId {
        self.instance_id += 1;
        let id = InstanceId(self.instance_id);
        self.instance_name_map.insert(instance.name.clone(), id);
        self.instances.insert(id, instance);
        id
    }

    /// Iterate over the instances of this cell.
    #[inline]
    pub fn instances(&self) -> impl Iterator<Item = (InstanceId, &Instance)> {
        self.instances.iter().map(|x| (*x.0, x.1))
    }
}

impl Instance {
    /// Create an instance of the given cell with the given name.
    pub fn new(name: impl Into<ArcStr>, child: impl Into<ChildId>) -> Self {
        Self {
            child: child.into(),
            name: name.into(),
            connections: HashMap::new(),
        }
    }

    /// Connect the given port of the child cell to the given node in the parent cell.
    #[inline]
    pub fn connect(&mut self, name: impl Into<ArcStr>, conn: impl Into<Concat>) {
        self.connections.insert(name.into(), conn.into());
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
}

impl Port {
    /// The ID of the signal this port exposes.
    #[inline]
    pub fn signal(&self) -> SignalId {
        self.signal
    }

    /// The direction of this port.
    #[inline]
    pub fn direction(&self) -> Direction {
        self.direction
    }
}
