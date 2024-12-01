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

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use arcstr::ArcStr;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uniquify::Names;

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
            .map(|inst| inst.child)
            .or_else(|| self.top.get_id().copied())
    }
}

/// An element of an [`AnnotatedInstancePath`].
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AnnotatedInstancePathElement {
    /// The underlying instance path element.
    pub elem: InstancePathElement,
    /// The child associated with this element.
    pub child: CellId,
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

/// A path of strings to an instance or object in a SCIR library.
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

pub trait Ir {
    type LibraryData;
    type CellData;
    type InstanceData;
    /// Data associated to a port (e.g. input[7:0]).
    type PortData;
    /// Data associated to one element of a port (e.g. input[2]).
    type PortElementData;
}

impl Ir for () {
    type LibraryData = ();
    type CellData = ();
    type InstanceData = ();
    type PortData = ();
    type PortElementData = ();
}

/// A library of SCIR cells with schema `S`.
pub struct LibraryBuilder<IR: Ir + ?Sized = ()> {
    /// The current cell ID counter.
    ///
    /// Initialized to 0 when the library is created.
    /// Should be incremented before assigning a new ID.
    cell_id: u64,

    /// A map of the cells in the library.
    cells: IndexMap<CellId, Cell<IR>>,

    /// A map of cell name to cell ID.
    ///
    /// Cell names are only guaranteed to be unique in a validated [`Library`].
    name_map: HashMap<ArcStr, CellId>,

    /// Assigned names for purposes of auto-assigning names to new cells.
    names: Names<CellId>,

    data: IR::LibraryData,

    /// The ID of the top cell, if there is one.
    top: Option<CellId>,
}

impl<IR: Ir + ?Sized> Default for LibraryBuilder<IR>
where
    IR::LibraryData: Default,
{
    fn default() -> Self {
        Self {
            cell_id: 0,
            cells: IndexMap::new(),
            name_map: HashMap::new(),
            names: Names::new(),
            top: None,
            data: Default::default(),
        }
    }
}

impl<IR> Clone for LibraryBuilder<IR>
where
    IR: Ir + ?Sized,
    IR::LibraryData: Clone,
    Cell<IR>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cell_id: self.cell_id,
            cells: self.cells.clone(),
            name_map: self.name_map.clone(),
            names: self.names.clone(),
            top: self.top,
            data: self.data.clone(),
        }
    }
}

impl<IR: Ir + ?Sized> Debug for LibraryBuilder<IR>
where
    IR::LibraryData: Debug,
    Cell<IR>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("LibraryBuilder");
        let _ = builder.field("cell_id", &self.cell_id);
        let _ = builder.field("cells", &self.cells);
        let _ = builder.field("name_map", &self.name_map);
        let _ = builder.field("names", &self.names);
        let _ = builder.field("top", &self.top);
        let _ = builder.field("data", &self.data);
        builder.finish()
    }
}

/// A SCIR library that is guaranteed to be valid.
///
/// The contents of the library cannot be mutated.
pub struct Library<IR: Ir + ?Sized>(LibraryBuilder<IR>);

impl<IR> Debug for Library<IR>
where
    IR: Ir + ?Sized,
    LibraryBuilder<IR>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Library");
        let _ = builder.field("0", &self.0);
        builder.finish()
    }
}

impl<IR: Ir + ?Sized> Clone for Library<IR>
where
    LibraryBuilder<IR>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<IR: Ir + ?Sized> Deref for Library<IR> {
    type Target = LibraryBuilder<IR>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<IR: Ir + ?Sized> Library<IR> {
    /// Converts this library into a [`LibraryBuilder`] that can be modified.
    pub fn into_builder(self) -> LibraryBuilder<IR> {
        self.0
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
    /// use commonir::Direction;
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Port<IR: Ir + ?Sized> {
    /// The name of this port.
    pub name: ArcStr,
    direction: Direction,
    data: IR::PortData,
    #[serde(bound(
        deserialize = "IR::PortElementData: Deserialize<'de>",
        serialize = "IR::PortElementData: Serialize"
    ))]
    elems: Vec<IR::PortElementData>,
}

impl<IR: Ir + ?Sized> Clone for Port<IR>
where
    IR::PortData: Clone,
    IR::PortElementData: Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            direction: self.direction,
            data: self.data.clone(),
            elems: self.elems.clone(),
        }
    }
}

/// An instance of a child cell placed inside a parent cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance<IR: Ir + ?Sized> {
    /// The ID of the child.
    child: CellId,
    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    name: ArcStr,
    data: IR::InstanceData,
}

/// A cell.
#[derive(Serialize, Deserialize)]
pub struct Cell<IR: Ir + ?Sized> {
    pub(crate) name: ArcStr,
    #[serde(bound(
        deserialize = "Port<IR>: Deserialize<'de>",
        serialize = "Port<IR>: Serialize"
    ))]
    pub(crate) ports: IndexMap<ArcStr, Port<IR>>,
    /// The last instance ID assigned.
    ///
    /// Initialized to 0 upon cell creation.
    instance_id: u64,
    #[serde(bound(
        deserialize = "Instance<IR>: Deserialize<'de>",
        serialize = "Instance<IR>: Serialize"
    ))]
    pub(crate) instances: IndexMap<InstanceId, Instance<IR>>,
    /// A map of instance name to instance ID.
    ///
    /// Instance names are only guaranteed to be unique in a validated [`Library`].
    instance_name_map: HashMap<ArcStr, InstanceId>,
}

impl<IR: Ir + ?Sized> Clone for Cell<IR>
where
    Port<IR>: Clone,
    Instance<IR>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            instance_id: self.instance_id,
            ports: self.ports.clone(),
            instances: self.instances.clone(),
            instance_name_map: self.instance_name_map.clone(),
        }
    }
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

impl<IR: Ir + ?Sized> LibraryBuilder<IR> {
    /// Creates a new, empty library.
    pub fn new(data: IR::LibraryData) -> Self {
        Self {
            cell_id: 0,
            cells: IndexMap::new(),
            name_map: HashMap::new(),
            names: Names::new(),
            top: None,
            data,
        }
    }

    /// Adds the given cell to the library.
    ///
    /// Returns the ID of the newly added cell.
    pub fn add_cell(&mut self, cell: Cell<IR>) -> CellId {
        let id = self.alloc_cell_id();
        self.name_map.insert(cell.name.clone(), id);
        self.names.reserve_name(id, cell.name.clone());
        self.cells.insert(id, cell);
        id
    }

    /// Merges the given cell into the library.
    ///
    /// Returns the ID of the newly added cell. May rename the cell if the name is already taken.
    pub fn merge_cell(&mut self, mut cell: Cell<IR>) -> CellId {
        let id = self.alloc_cell_id();
        let n_name = self.names.assign_name(id, &cell.name);
        cell.name = n_name;
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
    pub(crate) fn add_cell_with_id(&mut self, id: impl Into<CellId>, cell: Cell<IR>) {
        let id = id.into();
        assert!(!self.cells.contains_key(&id));
        self.cell_id = std::cmp::max(id.0, self.cell_id);
        self.name_map.insert(cell.name.clone(), id);
        self.names.reserve_name(id, cell.name.clone());
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
    pub fn overwrite_cell_with_id(&mut self, id: impl Into<CellId>, cell: Cell<IR>) {
        let id = id.into();
        assert!(self.cells.contains_key(&id));
        self.cell_id = std::cmp::max(id.0, self.cell_id);
        self.name_map.insert(cell.name.clone(), id);
        self.cells.insert(id, cell);
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
    pub fn cell(&self, id: CellId) -> &Cell<IR> {
        self.cells.get(&id).unwrap()
    }

    /// Gets the cell with the given ID.
    #[inline]
    pub fn try_cell(&self, id: CellId) -> Option<&Cell<IR>> {
        self.cells.get(&id)
    }

    /// Gets the cell with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no cell has the given name.
    pub fn cell_named(&self, name: &str) -> &Cell<IR> {
        self.cell(*self.name_map.get(name).unwrap())
    }

    /// Gets the cell with the given name.
    pub fn try_cell_named(&self, name: &str) -> Option<&Cell<IR>> {
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
    pub fn cells(&self) -> impl Iterator<Item = (CellId, &Cell<IR>)> {
        self.cells.iter().map(|(id, cell)| (*id, cell))
    }

    /// The list of cell IDs instantiated by the given root cells.
    ///
    /// The list returned will include the root cell IDs.
    pub(crate) fn cells_used_by(&self, roots: impl IntoIterator<Item = CellId>) -> Vec<CellId> {
        let mut stack = VecDeque::new();
        let mut visited = HashSet::new();
        for root in roots {
            stack.push_back(root);
        }

        while let Some(id) = stack.pop_front() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            let cell = self.cell(id);
            for (_, inst) in cell.instances() {
                stack.push_back(inst.child);
            }
        }

        visited.drain().collect()
    }

    fn convert_instance_path_cell(&self, top: &InstancePathCell) -> Option<(CellId, &Cell<IR>)> {
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
                .map(|inst| inst.child)
                .unwrap();

                annotated_elems.push(AnnotatedInstancePathElement {
                    elem: instance,
                    child,
                });

                cell = self.try_cell(child);
            } else {
                panic!("cannot find cell");
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
                        self.try_cell(path.instances[i - 1].child)
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
                                        path.instances[i - 1].child
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
    pub fn build(self) -> Result<Library<IR>, ()> {
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
    pub fn try_build(self) -> Result<(Library<IR>, ()), ()> {
        Ok((Library(self), ()))
    }
}

impl<IR: Ir + ?Sized> Cell<IR> {
    /// Creates a new cell with the given name.
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            name: name.into(),
            ports: IndexMap::new(),
            instance_id: 0,
            instances: IndexMap::new(),
            instance_name_map: HashMap::new(),
        }
    }

    /// The name of the cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    /// Iterate over the ports of this cell.
    #[inline]
    pub fn ports(&self) -> impl Iterator<Item = &Port<IR>> {
        self.ports.iter().map(|(_, port)| port)
    }

    /// Get a port of this cell by name.
    ///
    /// # Panics
    ///
    /// Panics if the provided port does not exist.
    #[inline]
    pub fn port(&self, name: &str) -> &Port<IR> {
        self.ports.get(name).unwrap()
    }

    /// Get the instance associated with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if no instance with the given ID exists.
    #[inline]
    pub fn instance(&self, id: InstanceId) -> &Instance<IR> {
        self.instances.get(&id).unwrap()
    }

    /// Get the instance associated with the given ID.
    #[inline]
    pub fn try_instance(&self, id: InstanceId) -> Option<&Instance<IR>> {
        self.instances.get(&id)
    }

    /// Gets the instance with the given name.
    ///
    /// # Panics
    ///
    /// Panics if no instance has the given name.
    pub fn instance_named(&self, name: &str) -> &Instance<IR> {
        self.instance(*self.instance_name_map.get(name).unwrap())
    }

    /// Gets the instance with the given name.
    pub fn try_instance_named(&self, name: &str) -> Option<&Instance<IR>> {
        self.try_instance(*self.instance_name_map.get(name)?)
    }

    /// Gets the instance associated with the given path element.
    pub fn instance_from_path_element(&self, elem: &InstancePathElement) -> &Instance<IR> {
        match elem {
            InstancePathElement::Id(id) => self.instance(*id),
            InstancePathElement::Name(name) => self.instance_named(name),
        }
    }

    /// Add the given instance to the cell.
    #[inline]
    pub fn add_instance(&mut self, instance: Instance<IR>) -> InstanceId {
        self.instance_id += 1;
        let id = InstanceId(self.instance_id);
        self.instance_name_map.insert(instance.name.clone(), id);
        self.instances.insert(id, instance);
        id
    }

    /// Iterate over the instances of this cell.
    #[inline]
    pub fn instances(&self) -> impl Iterator<Item = (InstanceId, &Instance<IR>)> {
        self.instances.iter().map(|x| (*x.0, x.1))
    }
}

impl<IR: Ir + ?Sized> Instance<IR> {
    /// Create an instance of the given cell with the given name.
    pub fn new(name: impl Into<ArcStr>, child: CellId, data: IR::InstanceData) -> Self {
        Self {
            child: child.into(),
            name: name.into(),
            data,
        }
    }

    /// The ID of the child cell.
    ///
    /// This instance represents an instantiation of the child cell in a parent cell.
    #[inline]
    pub fn child(&self) -> CellId {
        self.child
    }

    /// The name of this instance.
    ///
    /// This is not necessarily the name of the child cell.
    #[inline]
    pub fn name(&self) -> &ArcStr {
        &self.name
    }
}

impl<IR: Ir + ?Sized> Port<IR> {
    /// The direction of this port.
    #[inline]
    pub fn direction(&self) -> Direction {
        self.direction
    }
}
