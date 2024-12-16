//! Traits and types for schematic IOs.

use crate::block::Block;
use crate::diagnostics::SourceInfo;
use crate::schematic::{CellId, HasNestedView, InstanceId, InstancePath, NestedView};
use crate::types::{FlatLen, Flatten, HasNameTree};
use scir::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;

use super::{BundleKind, HasBundleKind, Io, Signal, Unflatten};

/// A schematic bundle kind.
pub trait SchematicBundleKind: BundleKind {
    /// The associated node bundle.
    type NodeBundle: HasNestedView<NestedView: HasBundleKind<BundleKind = Self>>
        + HasBundleKind<BundleKind = Self>
        + Unflatten<Self, Node>
        + Flatten<Node>;

    /// The associated terminal bundle.
    type TerminalBundle: HasNestedView<NestedView: HasBundleKind<BundleKind = Self>>
        + HasBundleKind<BundleKind = Self>
        + Unflatten<Self, Terminal>
        + Flatten<Terminal>
        + Flatten<Node>;

    /// Creates a terminal view of the object given a parent node, the cell IO, and the instance IO.
    fn terminal_view(
        cell: CellId,
        cell_io: &NodeBundle<Self>,
        instance: InstanceId,
        instance_io: &NodeBundle<Self>,
    ) -> TerminalBundle<Self>;
}

/// A schematic bundle kind that can be viewed as another bundle kind `T`.
pub trait DataView<T: SchematicBundleKind>: SchematicBundleKind {
    /// Views a node bundle as a node bundle of a different kind.
    fn view_nodes_as(nodes: &NodeBundle<Self>) -> NodeBundle<T>;

    /// Views a terminal bundle as a terminal bundle of a different kind.
    fn view_terminals_as(terminals: &TerminalBundle<Self>) -> TerminalBundle<T> {
        // TODO: Do some sanity checking/error handling.
        let kind = terminals.kind();
        let flat_terminals = Flatten::<Terminal>::flatten_vec(terminals);
        let terminal_map = flat_terminals
            .iter()
            .map(|terminal| (terminal.instance_node, terminal))
            .collect::<HashMap<_, _>>();
        let mut flat_nodes = flat_terminals.iter().map(|terminal| terminal.instance_node);
        let nodes = NodeBundle::<Self>::unflatten(&kind, &mut flat_nodes).unwrap();
        let nodes_view = Self::view_nodes_as(&nodes);
        let nodes_view_kind = nodes_view.kind();
        let flat_nodes_view = Flatten::<Node>::flatten_vec(&nodes_view);
        let mut flat_terminals_view = flat_nodes_view.iter().map(|node| *terminal_map[node]);
        TerminalBundle::<T>::unflatten(&nodes_view_kind, &mut flat_terminals_view).unwrap()
    }
}

/// The type of a bundle associated with an IO.
pub type IoNodeBundle<T> = NodeBundle<<T as Block>::Io>;
/// The type of a terminal bundle associated with an IO.
pub type IoTerminalBundle<T> = TerminalBundle<<T as Block>::Io>;
/// The type of a node bundle associated with [`SchematicBundleKind`] `T`.
pub type NodeBundle<T> = <<T as HasBundleKind>::BundleKind as SchematicBundleKind>::NodeBundle;
/// The type of a terminal bundle associated with [`SchematicBundleKind`] `T`.
pub type TerminalBundle<T> =
    <<T as HasBundleKind>::BundleKind as SchematicBundleKind>::TerminalBundle;

/// The priority a node has in determining the name of a merged node.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) enum NodePriority {
    /// An IO / externally-visible signal name.
    ///
    /// Has the highest priority in determining node names.
    Io = 3,
    /// An explicitly named signal.
    Named = 2,
    /// A signal with an automatically-generated name.
    ///
    /// Has the lowest priority in determining node names.
    Auto = 1,
}

/// A node unification table for connectivity management.
pub type NodeUf = ena::unify::InPlaceUnificationTable<Node>;

#[derive(Clone, Debug)]
pub(crate) struct NodeConnectDirectionError {
    #[allow(dead_code)]
    data: Vec<[(Direction, NodeDriverData); 2]>,
}

/// A single node in a circuit.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node(u32);

impl FlatLen for Node {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for Node {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        output.extend(std::iter::once(*self));
    }
}

impl Unflatten<Signal, Node> for Node {
    fn unflatten<I>(_data: &Signal, source: &mut I) -> Option<Self>
    where
        I: Iterator<Item = Node>,
    {
        source.next()
    }
}

impl HasBundleKind for Node {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

impl HasNestedView for Node {
    type NestedView = NestedNode;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedNode {
            node: *self,
            instances: parent.clone(),
        }
    }
}

/// A nested node within a cell.
///
/// Created when accessing nodes from instances propagated through data.
#[derive(Clone, Debug)]
pub struct NestedNode {
    pub(crate) instances: InstancePath,
    pub(crate) node: Node,
}

/// A path from a top level cell to a nested node.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePath {
    pub(crate) top: CellId,
    pub(crate) instances: Vec<InstanceId>,
    pub(crate) node: Node,
}

impl NestedNode {
    /// Returns the path to this node.
    pub fn path(&self) -> NodePath {
        NodePath {
            top: self.instances.top,
            instances: self.instances.path.iter().copied().collect(),
            node: self.node,
        }
    }
}

impl FlatLen for NestedNode {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<NestedNode> for NestedNode {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<NestedNode>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl HasBundleKind for NestedNode {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

impl HasNestedView for NestedNode {
    type NestedView = NestedNode;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedNode {
            node: self.node,
            instances: self.instances.prepend(parent),
        }
    }
}

impl FlatLen for Vec<Node> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Flatten<Node> for Vec<Node> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        output.extend(self.iter().copied());
    }
}

impl From<NestedNode> for NodePath {
    fn from(value: NestedNode) -> Self {
        value.path()
    }
}

impl From<&NestedNode> for NodePath {
    fn from(value: &NestedNode) -> Self {
        value.path()
    }
}

/// A terminal of an instance.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terminal {
    pub(crate) cell_id: CellId,
    pub(crate) cell_node: Node,
    pub(crate) instance_id: InstanceId,
    pub(crate) instance_node: Node,
}

impl Deref for Terminal {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.instance_node
    }
}

impl AsRef<Node> for Terminal {
    fn as_ref(&self) -> &Node {
        self
    }
}

impl FlatLen for Terminal {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for Terminal {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        output.extend(std::iter::once(self.instance_node));
    }
}

impl Flatten<Terminal> for Terminal {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Terminal>,
    {
        output.extend(std::iter::once(*self));
    }
}

impl Unflatten<Signal, Terminal> for Terminal {
    fn unflatten<I>(_data: &Signal, source: &mut I) -> Option<Self>
    where
        I: Iterator<Item = Terminal>,
    {
        source.next()
    }
}

impl HasBundleKind for Terminal {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

impl HasNestedView for Terminal {
    type NestedView = NestedTerminal;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedTerminal(NestedNode {
            instances: parent.append_segment(self.instance_id, self.cell_id),
            node: self.cell_node,
        })
    }
}

/// A nested instance terminal.
#[derive(Clone, Debug)]
pub struct NestedTerminal(NestedNode);

impl Deref for NestedTerminal {
    type Target = NestedNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<NestedNode> for NestedTerminal {
    fn as_ref(&self) -> &NestedNode {
        self
    }
}

impl NestedTerminal {
    /// Returns the path to this [`NestedTerminal`].
    pub fn path(&self) -> TerminalPath {
        TerminalPath(self.0.path())
    }
}

impl FlatLen for NestedTerminal {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<NestedTerminal> for NestedTerminal {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<NestedTerminal>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl HasBundleKind for NestedTerminal {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

impl HasNestedView for NestedTerminal {
    type NestedView = NestedTerminal;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedTerminal(<NestedNode as HasNestedView>::nested_view(&self.0, parent))
    }
}

/// A path to an instance's terminal.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalPath(NodePath);

impl Deref for TerminalPath {
    type Target = NodePath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<NodePath> for TerminalPath {
    fn as_ref(&self) -> &NodePath {
        self
    }
}

impl From<NestedTerminal> for TerminalPath {
    fn from(value: NestedTerminal) -> Self {
        value.path()
    }
}

impl From<&NestedTerminal> for TerminalPath {
    fn from(value: &NestedTerminal) -> Self {
        value.path()
    }
}

/// The value associated to a node in a schematic builder's union find data structure.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[doc(hidden)]
pub struct NodeUfValue {
    /// The overall priority of a set of merged nodes.
    ///
    /// Taken to be the highest among priorities of all nodes
    /// in the merged set.
    priority: NodePriority,

    /// The node that provides `priority`.
    ///
    /// For example, if priority is NodePriority::Io, `node`
    /// should be the node identifier representing the IO node.
    pub(crate) source: Node,
}

impl ena::unify::UnifyValue for NodeUfValue {
    type Error = ena::unify::NoError;

    fn unify_values(value1: &Self, value2: &Self) -> std::result::Result<Self, Self::Error> {
        if value1.priority == NodePriority::Io
            && value2.priority == NodePriority::Io
            && value1.source != value2.source
        {
            panic!("shorted IOs are not supported")
        }
        Ok(if value1.priority >= value2.priority {
            *value1
        } else {
            *value2
        })
    }
}

impl ena::unify::UnifyKey for Node {
    type Value = Option<NodeUfValue>;
    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "Node"
    }
}

#[derive(Clone)]
pub(crate) struct NodeContext {
    uf: NodeUf,
    connections_data: Vec<Option<NodeConnectionsData>>,
}

#[derive(Clone, Debug)]
struct NodeConnectionsData {
    /// Info about all attached nodes on the net, grouped by direction
    drivers: BTreeMap<Direction, NodeDriverData>,
}

impl NodeConnectionsData {
    fn merge_from(&mut self, other: Self) {
        for (direction, data) in other.drivers {
            use std::collections::btree_map::Entry;
            match self.drivers.entry(direction) {
                Entry::Vacant(entry) => {
                    entry.insert(data);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().merge_from(data);
                }
            }
        }
    }

    fn from_single(direction: Direction, source_info: SourceInfo) -> Self {
        Self {
            drivers: [(direction, NodeDriverData::from_single(source_info))].into(),
        }
    }

    fn empty() -> Self {
        Self { drivers: [].into() }
    }
}

impl Default for NodeConnectionsData {
    fn default() -> Self {
        Self::empty()
    }
}

/// Information about all nodes on a net of a particular [`Direction`]
#[derive(Clone, Debug)]
struct NodeDriverData {
    // FIXME: come up with some mechanism for representing root cell IO
    // locations (there's no call-site source info that would make sense)
    /// Locations at which nodes on this net were instantiated
    sources: Vec<SourceInfo>,
}

impl NodeDriverData {
    fn merge_from(&mut self, other: Self) {
        self.sources.extend(other.sources);
    }

    fn from_single(source_info: SourceInfo) -> Self {
        Self {
            sources: vec![source_info],
        }
    }
}

impl NodeContext {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            uf: Default::default(),
            connections_data: vec![],
        }
    }

    fn connections_data(&self, node: Node) -> &Option<NodeConnectionsData> {
        &self.connections_data[usize::try_from(ena::unify::UnifyKey::index(&node)).unwrap()]
    }

    fn connections_data_mut(&mut self, node: Node) -> &mut Option<NodeConnectionsData> {
        &mut self.connections_data[usize::try_from(ena::unify::UnifyKey::index(&node)).unwrap()]
    }

    pub(crate) fn node(
        &mut self,
        direction: Option<Direction>,
        priority: NodePriority,
        source_info: SourceInfo,
    ) -> Node {
        let id = self.uf.new_key(Default::default());

        assert_eq!(
            usize::try_from(ena::unify::UnifyKey::index(&id)).unwrap(),
            self.connections_data.len()
        );
        self.connections_data.push(Some(
            direction
                .map(|direction| NodeConnectionsData::from_single(direction, source_info))
                .unwrap_or_default(),
        ));
        // scuffed self-consistency check - false negatives possible
        debug_assert!(self.connections_data_mut(id).is_some());

        self.uf.union_value(
            id,
            Some(NodeUfValue {
                priority,
                source: id,
            }),
        );

        id
    }

    #[inline]
    pub fn into_uf(self) -> NodeUf {
        self.uf
    }

    fn nodes_directed(
        &mut self,
        directions: &[Direction],
        priority: NodePriority,
        source_info: SourceInfo,
    ) -> Vec<Node> {
        directions
            .iter()
            .map(|dir| self.node(Some(*dir), priority, source_info.clone()))
            .collect()
    }

    fn nodes_undirected(
        &mut self,
        n: usize,
        priority: NodePriority,
        source_info: SourceInfo,
    ) -> Vec<Node> {
        (0..n)
            .map(|_| self.node(None, priority, source_info.clone()))
            .collect()
    }

    pub fn instantiate_directed<IO: Io + HasBundleKind<BundleKind: SchematicBundleKind>>(
        &mut self,
        io: &IO,
        priority: NodePriority,
        source_info: SourceInfo,
    ) -> (Vec<Node>, NodeBundle<IO>) {
        let nodes = self.nodes_directed(&io.flatten_vec(), priority, source_info);
        let data = NodeBundle::<IO>::unflatten(&io.kind(), &mut nodes.iter().copied()).unwrap();
        (nodes, data)
    }

    pub fn instantiate_undirected<K: HasBundleKind<BundleKind: SchematicBundleKind>>(
        &mut self,
        kind: &K,
        priority: NodePriority,
        source_info: SourceInfo,
    ) -> (Vec<Node>, NodeBundle<K>) {
        let kind = kind.kind();
        let nodes = self.nodes_undirected(kind.flat_names(None).len(), priority, source_info);
        let data = NodeBundle::<K>::unflatten(&kind, &mut nodes.iter().copied()).unwrap();
        (nodes, data)
    }

    pub(crate) fn connect(&mut self, n1: Node, n2: Node, source_info: SourceInfo) {
        fn get_root(this: &mut NodeContext, n: Node) -> Node {
            this.uf
                .probe_value(n)
                .expect("node should be populated")
                .source
        }

        let n1_root = get_root(self, n1);
        let n2_root = get_root(self, n2);

        if n1_root == n2_root {
            tracing::info!(?source_info, "connecting nodes that are already connected");
            return;
        }

        let n1_connections_data = self
            .connections_data(n1_root)
            .as_ref()
            .expect("n1 should be populated");
        let n2_connections_data = self
            .connections_data(n2_root)
            .as_ref()
            .expect("n1 should be populated");

        // TODO: potentially use an algorithm better than n^2?
        let incompatible_drivers: Vec<_> = n1_connections_data
            .drivers
            .iter()
            .flat_map(|e1| n2_connections_data.drivers.iter().map(move |e2| [e1, e2]))
            .filter(|[(&k1, _), (&k2, _)]| !k1.is_compatible_with(k2))
            .collect();
        if !incompatible_drivers.is_empty() {
            // If drivers are not compatible, return an error but connect them
            // anyways, because (1) we would like to detect further errors
            // that may be caused by the connection being made and (2) the
            // error might be spurious and waived by the user.
            let err = NodeConnectDirectionError {
                data: incompatible_drivers
                    .iter()
                    .map(|&[(&k1, v1), (&k2, v2)]| [(k1, v1.clone()), (k2, v2.clone())])
                    .collect(),
            };
            tracing::warn!(?err, "incompatible drivers");
        }

        self.uf.union(n1, n2);

        let new_root = get_root(self, n1);
        let old_root = match new_root {
            x if x == n1_root => n2_root,
            x if x == n2_root => n1_root,
            _ => panic!(
                "connect: new root isn't any of the old roots? (got {:?}, had {:?} {:?})",
                new_root, n1_root, n2_root
            ),
        };

        let old_connections_data = self
            .connections_data_mut(old_root)
            .take()
            .expect("old root should be populated");
        self.connections_data_mut(new_root)
            .as_mut()
            .expect("new root should be populated")
            .merge_from(old_connections_data);
    }
}

/// A signal exposed by a cell.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    direction: Direction,
    node: Node,
}

impl Port {
    #[inline]
    pub(crate) fn new(node: Node, direction: Direction) -> Self {
        Self { node, direction }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn direction(&self) -> Direction {
        self.direction
    }

    #[inline]
    pub(crate) fn node(&self) -> Node {
        self.node
    }
}
