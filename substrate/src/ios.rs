//! Common IO types.

use crate::io::{InOut, Input, Signal};
use crate::Io;

/// The interface to a standard 4-terminal MOSFET.
#[derive(Debug, Default, Clone, Io)]
pub struct MosIo {
    /// The drain.
    pub d: InOut<Signal>,
    /// The gate.
    pub g: Input<Signal>,
    /// The source.
    pub s: InOut<Signal>,
    /// The body.
    pub b: InOut<Signal>,
}
