//! DSPF netlists.

use arcstr::ArcStr;
use scir::{Library, NamedSliceOne, NetlistLibConversion, SliceOnePath};
use spice::Spice;
use substrate::io::schematic::{NestedNode, Node};
use substrate::schematic::conv::{ConvertedNodePath, RawLib};
use substrate::schematic::{HasNestedView, InstancePath};

pub struct DspfNodes<T> {
    instances: InstancePath,
    /// The source spice file for this DSPF extracted view.
    lib: RawLib<Spice>,
    inner: T,
}

pub struct DspfNestedNodes<T> {
    instances: InstancePath,
    inner: T,
}

pub trait HasNestedDspfView: Sized {
    type Strings: ReconstructDspfView<Self>;

    fn flatten(&self) -> (Vec<Node>, Vec<NestedNode>);
}

pub trait ReconstructDspfView<T> {
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
            instances: parent.clone(),
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
            instances: self.instances.prepend(parent),
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
    fn unflatten(source: &(), nodes: Vec<String>, nested_nodes: Vec<String>) -> Self {}
}
