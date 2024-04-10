//! DSPF netlists.

use scir::{NamedSliceOne, NetlistLibConversion, SliceOnePath};
use spice::Spice;
use std::sync::Arc;
use substrate::io::schematic::{NestedNode, Node};
use substrate::schematic::conv::{ConvertedNodePath, RawLib};
use substrate::schematic::{HasNestedView, InstancePath};

/// A set of nodes in a DSPF netlist.
pub struct DspfNodes<T> {
    /// The source spice file for this DSPF extracted view.
    pub lib: RawLib<Spice>,
    /// The inner saved nodes.
    pub inner: Arc<T>,
}

/// A set of nodes in a nested DSPF netlist instantiation.
pub struct DspfNestedNodes<T> {
    pub dspf_instance: InstancePath,
    /// The inner saved nodes.
    pub inner: T,
}

pub struct DspfNode {
    pub dspf_instance: InstancePath,
    pub path: String,
}

/// Indicates that a type has a nested DSPF view.
pub trait HasNestedDspfView: Sized {
    /// The node container type, where nodes are stored as strings.
    type Strings: ReconstructDspfView<Self>;

    /// Flatten the container into a set of nodes and nested nodes.
    fn flatten(&self) -> (Vec<Node>, Vec<NestedNode>);
}

/// A type that can reconstruct a DSPF view.
pub trait ReconstructDspfView<T> {
    /// Unflatten the container from a set of nodes and nested nodes.
    fn unflatten(source: &T, nodes: Vec<String>, nested_nodes: Vec<String>) -> Self;
}

impl<T> HasNestedView for DspfNodes<T>
where
    T: HasNestedDspfView,
    <T as HasNestedDspfView>::Strings: Send + Sync,
{
    type NestedView = DspfNestedNodes<<T as HasNestedDspfView>::Strings>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        let (nodes, nested_nodes) = self.inner.flatten();
        let nodes = nodes
            .into_iter()
            .map(|n| {
                let path = self.lib.convert_node(&n).unwrap();
                let path = match path {
                    ConvertedNodePath::Cell(path) => path,
                    ConvertedNodePath::Primitive {
                        instances, port, ..
                    } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
                };
                Spice::node_voltage_path(&self.lib.scir, &NetlistLibConversion::new(), &path)
                    .to_uppercase()
            })
            .collect();
        let nested_nodes = nested_nodes
            .into_iter()
            .map(|n| {
                let path = self.lib.convert_node_path(&n.path()).unwrap();
                let path = match path {
                    ConvertedNodePath::Cell(path) => path,
                    ConvertedNodePath::Primitive {
                        instances, port, ..
                    } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
                };
                Spice::node_voltage_path(&self.lib.scir, &NetlistLibConversion::new(), &path)
                    .to_uppercase()
            })
            .collect();
        let inner = <T as HasNestedDspfView>::Strings::unflatten(&self.inner, nodes, nested_nodes);
        DspfNestedNodes {
            dspf_instance: parent.clone(),
            inner,
        }
    }
}

impl<T> HasNestedView for DspfNestedNodes<T>
where
    T: Send + Sync + Clone,
{
    type NestedView = DspfNestedNodes<T>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        Self {
            dspf_instance: self.dspf_instance.prepend(parent),
            inner: self.inner.clone(),
        }
    }
}

impl HasNestedDspfView for () {
    type Strings = ();

    fn flatten(&self) -> (Vec<Node>, Vec<NestedNode>) {
        (Vec::new(), Vec::new())
    }
}

impl ReconstructDspfView<()> for () {
    fn unflatten(_source: &(), _nodes: Vec<String>, _nested_nodes: Vec<String>) -> Self {}
}
