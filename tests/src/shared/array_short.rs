use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::io::*;
use substrate::schematic::*;

use substrate::pdk::{ExportsPdkSchematicData, Pdk, PdkSchematic, ToSchema};
use substrate::schematic::schema::Schema;
use substrate::{block, block::Block, schematic::Schematic};

#[derive(Debug, Clone, Io)]
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
    type Kind = block::Cell;
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

impl<PDK: Pdk> ExportsPdkSchematicData<PDK> for ArrayShort {
    type Data<S> = () where PDK: ToSchema<S>;
}

impl<PDK: Pdk> PdkSchematic<PDK> for ArrayShort {
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> substrate::error::Result<Self::Data<S>>
    where
        PDK: ToSchema<S>,
    {
        for i in 0..self.width {
            cell.connect(io.inputs[i], io.out)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::pdk::sky130_open_ctx;
    use test_log::test;

    #[test]
    #[should_panic]
    fn panics_when_shorting_ios() {
        let ctx = sky130_open_ctx();
        let _ = ctx.export_scir(ArrayShort { width: 5 });
    }
}
