//! Common IO types.

use substrate_api::block::Block;

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

/// A trait indicating that a block is a standard 4 terminal MOSFET.
pub trait Mos: Block<Io = MosIo> {}

impl<T> Mos for T where T: Block<Io = MosIo> {}

/// The interface to which simulation testbenches should conform.
#[derive(Debug, Default, Clone, Io)]
pub struct TestbenchIo {
    /// The global ground net.
    pub vss: InOut<Signal>,
}

/// The interface for 2-terminal voltage sources.
#[derive(Debug, Default, Clone, Io)]
pub struct VsourceIo {
    /// The positive terminal.
    pub p: InOut<Signal>,
    /// The negative terminal.
    pub n: InOut<Signal>,
}
