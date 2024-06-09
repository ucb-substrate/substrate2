//! Short propagation analysis.

use crate::parser::conv::{map_subckts, SubcktName};
use crate::parser::{Ast, Component, Node, Subckt};
use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct NodeKey(u32);

type NodeUf = ena::unify::InPlaceUnificationTable<NodeKey>;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
enum NodePriority {
    Io = 2,
    #[default]
    Default = 1,
}

/// The value associated to a node in a schematic builder's union find data structure.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[doc(hidden)]
struct NodeUfValue {
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

impl ena::unify::UnifyKey for NodeKey {
    type Value = NodeUfValue;
    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "NodeKey"
    }
}

impl ena::unify::UnifyValue for NodeUfValue {
    type Error = ena::unify::NoError;

    fn unify_values(value1: &Self, value2: &Self) -> std::result::Result<Self, Self::Error> {
        Ok(std::cmp::max(value1.clone(), value2.clone()))
    }
}

/// Analyzes shorts in a SPICE netlist.
#[derive(Clone)]
pub struct ShortPropagator {
    cells: HashMap<SubcktName, CellShortManager>,
}

/// Stores information about shorts within a cell.
#[derive(Clone)]
pub struct CellShortManager {
    node_to_key: HashMap<Node, NodeKey>,
    uf: NodeUf,
}

impl ShortPropagator {
    /// Construct a [`ShortPropagator`] from the given SPICE AST.
    pub fn analyze(ast: &Ast, blackbox: &HashSet<SubcktName>) -> Self {
        let mut val = Self::new();
        let subckts = map_subckts(ast);
        let order = dfs_postorder(&subckts, blackbox);

        for name in order.iter() {
            let subckt = subckts[name];
            let mut manager = CellShortManager::new();
            for p in subckt.ports.iter() {
                manager.register_node(p.clone(), NodePriority::Io);
            }

            for c in subckt.components.iter() {
                match c {
                    Component::Mos(m) => {
                        for node in [&m.d, &m.g, &m.s, &m.b] {
                            manager.register_node(node.clone(), NodePriority::Default);
                        }
                    }
                    Component::Res(r) => {
                        for node in [&r.pos, &r.neg] {
                            manager.register_node(node.clone(), NodePriority::Default);
                        }
                    }
                    Component::Diode(d) => {
                        for node in [&d.pos, &d.neg] {
                            manager.register_node(node.clone(), NodePriority::Default);
                        }
                    }
                    Component::Cap(c) => {
                        for node in [&c.pos, &c.neg] {
                            manager.register_node(node.clone(), NodePriority::Default);
                        }
                    }
                    Component::Instance(inst) => {
                        for node in inst.ports.iter() {
                            manager.register_node(node.clone(), NodePriority::Default);
                        }
                        // We do not support propagation of shorts in blackbox/missing subcircuits.
                        if !blackbox.contains(&inst.child) {
                            let child_subckt = match subckts.get(&inst.child) {
                                None => continue,
                                Some(&s) => s,
                            };
                            let mut port_to_connected_node = HashMap::new();
                            assert_eq!(inst.ports.len(), child_subckt.ports.len());
                            for (node, cport) in inst.ports.iter().zip(child_subckt.ports.iter()) {
                                port_to_connected_node.insert(cport, node);
                            }
                            let child_shorts = val.get_or_add_cell(inst.child.clone());
                            for (node, cport) in inst.ports.iter().zip(child_subckt.ports.iter()) {
                                let croot = child_shorts.root(cport);
                                manager.connect(node, port_to_connected_node[&croot]);
                            }
                        }
                    }
                }
            }

            for (n1, n2) in subckt.connects.iter() {
                manager.connect(n1, n2);
            }

            val.set_cell(name.clone(), manager);
        }

        val
    }

    fn new() -> Self {
        Self {
            cells: Default::default(),
        }
    }

    fn get_or_add_cell(&mut self, name: SubcktName) -> &mut CellShortManager {
        self.cells.entry(name).or_insert(CellShortManager::new())
    }

    /// Get the set of shorts within a cell.
    pub fn get_cell(&mut self, name: &SubcktName) -> &mut CellShortManager {
        self.cells.get_mut(name).unwrap()
    }

    fn set_cell(&mut self, name: SubcktName, manager: CellShortManager) {
        self.cells.insert(name, manager);
    }
}

impl CellShortManager {
    fn new() -> Self {
        Self {
            node_to_key: HashMap::new(),
            uf: NodeUf::new(),
        }
    }

    /// Adds a node with the given priority.
    /// Does nothing if the node has already been registered.
    ///
    /// Consequently, IO nodes must be registered first.
    fn register_node(&mut self, node: Node, priority: NodePriority) {
        if self.node_to_key.contains_key(&node) {
            return;
        }
        let key = self.uf.new_key(NodeUfValue {
            priority,
            source: node.clone(),
        });
        self.node_to_key.insert(node, key);
    }

    fn connect(&mut self, node1: &Node, node2: &Node) {
        let k1 = self.node_to_key[node1];
        let k2 = self.node_to_key[node2];
        self.uf.union(k1, k2);
    }

    /// The node to which the given node is shorted.
    pub fn root(&mut self, node: &Node) -> Node {
        let k = self.node_to_key[node];
        let v = self.uf.probe_value(k);
        v.source
    }
}

fn dfs_postorder(
    subckts: &HashMap<SubcktName, &Subckt>,
    blackbox: &HashSet<SubcktName>,
) -> Vec<SubcktName> {
    let mut visited = HashSet::new();
    let mut order = Vec::new();

    for name in subckts.keys() {
        dfs_postorder_inner(name, subckts, blackbox, &mut visited, &mut order);
    }

    order
}

fn dfs_postorder_inner(
    name: &SubcktName,
    subckts: &HashMap<SubcktName, &Subckt>,
    blackbox: &HashSet<SubcktName>,
    visited: &mut HashSet<SubcktName>,
    out: &mut Vec<SubcktName>,
) {
    if visited.contains(name) || blackbox.contains(name) {
        return;
    }

    let subckt = match subckts.get(name) {
        None => {
            return;
        }
        Some(&s) => s,
    };
    for c in subckt.components.iter() {
        if let Component::Instance(ref inst) = c {
            dfs_postorder_inner(&inst.child, subckts, blackbox, visited, out);
        }
    }

    out.push(name.clone());
    visited.insert(name.clone());
}
