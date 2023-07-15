use crate::shared::buffer::BufferIo;
use serde::{Deserialize, Serialize};
use substrate::Block;

#[derive(Block, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[substrate(
    io = "BufferIo",
    // schematic(path = "/path/to/schematic", fmt = "spectre")
)]
pub struct BufferHardMacro;
