use substrate::types::Io;
use substrate::types::{Input, Output, Signal};

use super::codegen::{HasView, NodeBundle, TerminalBundle};
use super::HasBundleKind;

// TODO: uncomment
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

/// An enum Io.
#[derive(Debug, Clone, Io)]
pub enum EnumIo {
    A,
    B { a: Input<Signal>, b: Output<Signal> },
    C(NamedStructIo),
}

/// Takes an IO type.
///
/// Used to validate that a given type implements `Io`.
fn takes_io<T: Io>() -> usize {
    std::mem::size_of::<T>()
}

#[crate::test]
fn generic_io_implements_io() {
    takes_io::<GenericIo<Signal>>();
    takes_io::<GenericIo<NamedStructIo>>();
    takes_io::<GenericIo<TupleIo>>();
}

#[crate::test]
fn named_struct_io_implements_io() {
    takes_io::<NamedStructIo>();
}

#[crate::test]
fn tuple_io_implements_io() {
    takes_io::<TupleIo>();
}
