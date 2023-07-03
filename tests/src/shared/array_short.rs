use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::io::*;
use substrate::schematic::*;

use substrate::pdk::Pdk;
use substrate::Io;
use substrate::{block::Block, schematic::HasSchematic};

#[derive(Debug, Default, Clone, Io)]
pub struct ArrayIo {
    pub inputs: Input<Array<Signal>>,
    pub out: Output<Signal>,
}

/// Shorts all input signals to an output node.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayShort {
    width: usize,
}

impl Block for ArrayShort {
    type Io = ArrayIo;

    fn id() -> ArcStr {
        arcstr::literal!("array_shorter")
    }
    fn name(&self) -> ArcStr {
        arcstr::format!("array_shorter_{}", self.width)
    }
    fn io(&self) -> Self::Io {
        Self::Io {
            inputs: Input(Array::new(self.width, Signal)),
            out: Output(Signal),
        }
    }
}

impl HasSchematic for ArrayShort {
    type Data = ();
}

impl<PDK: Pdk> HasSchematicImpl<PDK> for ArrayShort {
    fn schematic(
        &self,
        io: &ArrayIoSchematic,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        for i in 0..self.width {
            cell.connect(io.inputs[i], io.out)
        }
        Ok(())
    }
}