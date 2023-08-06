//! Tests for ensuring that `#[derive(Io)]` works.

use substrate::io::Io;
use substrate::io::{Input, Output, Signal};

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    /// Takes an IO type.
    ///
    /// Used to validate that a given type implements `Io`.
    fn takes_io<T: Io>() -> usize {
        std::mem::size_of::<T>()
    }

    #[test]
    fn generic_io_implements_io() {
        takes_io::<GenericIo<Signal>>();
        takes_io::<GenericIo<NamedStructIo>>();
        takes_io::<GenericIo<TupleIo>>();
    }

    #[test]
    fn named_struct_io_implements_io() {
        takes_io::<NamedStructIo>();
    }

    #[test]
    fn tuple_io_implements_io() {
        takes_io::<TupleIo>();
    }
}
