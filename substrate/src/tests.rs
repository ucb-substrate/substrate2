use std::path::PathBuf;

use crate::block::Block;
use crate::types::{Array, InOut, Input, Io, MosIo, Output, Signal};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

#[inline]
pub(crate) fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "MosIo")]
pub enum InverterMos {
    Nmos,
    Pmos,
}

#[derive(Io, Clone, Default)]
pub struct BufferIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    pub(crate) strength: usize,
}

impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Inverter {
    type Io = BufferIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Buffer {
    pub(crate) strength: usize,
}

impl Buffer {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Buffer {
    type Io = BufferIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BufferN {
    pub(crate) strength: usize,
    pub(crate) n: usize,
}

impl BufferN {
    pub fn new(strength: usize, n: usize) -> Self {
        Self { strength, n }
    }
}

impl Block for BufferN {
    type Io = BufferIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}_{}", self.strength, self.n)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Io, Clone)]
pub struct BufferNxMIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Array<Signal>>,
    pub dout: Output<Array<Signal>>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BufferNxM {
    pub(crate) strength: usize,
    pub(crate) n: usize,
    pub(crate) m: usize,
}

impl BufferNxM {
    pub fn new(strength: usize, n: usize, m: usize) -> Self {
        Self { strength, n, m }
    }
}

impl Block for BufferNxM {
    type Io = BufferNxMIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}_{}x{}", self.strength, self.n, self.m)
    }

    fn io(&self) -> Self::Io {
        Self::Io {
            din: Input(Array::new(self.m, Default::default())),
            dout: Output(Array::new(self.m, Default::default())),
            vdd: Default::default(),
            vss: Default::default(),
        }
    }
}
