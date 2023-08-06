//! Tests for ensuring that `#[derive(Io)]` works.

use substrate::io::{Input, Output, Signal};
use substrate::Io;

/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T> {
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
