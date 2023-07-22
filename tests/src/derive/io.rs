//! Tests for ensuring that `#[derive(Io)]` works.

use substrate::io::{Input, LayoutType, Output, SchematicType, Signal, Undirected, HierarchicalBuildFrom};
use substrate::Io;
use substrate::layout::element::NamedPorts;

/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T>
where
    T: Clone + Undirected + SchematicType + LayoutType + 'static,
    <T as SchematicType>::Data: Undirected,
    <T as LayoutType>::Data: Undirected,
    <T as LayoutType>::Builder: Undirected + HierarchicalBuildFrom<NamedPorts>,
{
    /// A single input field.
    pub signal: Input<T>,
}

/// A named struct Io.
#[derive(Debug, Clone, Io)]
pub struct NamedStructIo {
    /// An input.
    pub first: Input<Signal>,
    /// An output.
    pub second: Output<Signal>,
}

/// A tuple struct Io.
#[derive(Debug, Clone, Io)]
pub struct TupleIo(pub Input<Signal>, pub Output<Signal>);
