//! Tests for ensuring that `#[derive(Io)]` works.

use substrate::io::{HierarchicalBuildFrom, Input, Io, LayoutType, Output, SchematicType, Signal};
use substrate::layout::element::NamedPorts;

/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T>
where
    T: Clone + SchematicType + LayoutType + 'static,
    <T as LayoutType>::Builder: HierarchicalBuildFrom<NamedPorts>,
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
