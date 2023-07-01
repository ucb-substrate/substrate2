use arcstr::ArcStr;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use substrate::io::*;
use substrate::schematic::*;

use substrate::pdk::Pdk;
use substrate::Io;
use substrate::{block::Block, schematic::HasSchematic};

pub mod internal_signal;

#[derive(Debug, Default, Clone, Io)]
pub struct ResistorIo {
    pub p: InOut<Signal>,
    pub n: InOut<Signal>,
}

#[derive(Debug, Default, Clone, Io)]
pub struct PowerIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Debug, Default, Clone, Io)]
pub struct VdividerIo {
    pub pwr: PowerIo,
    pub out: Output<Signal>,
}

#[derive(Debug, Default, Clone, Io)]
pub struct ArrayIo {
    pub inputs: Input<Array<Signal>>,
    pub out: Output<Signal>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resistor {
    pub r: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vdivider {
    pub r1: Resistor,
    pub r2: Resistor,
}

/// Shorts all input signals to an output node.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayShorter {
    width: usize,
}

impl Block for Resistor {
    type Io = ResistorIo;

    fn id() -> ArcStr {
        arcstr::literal!("resistor")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("resistor_{}", self.r)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Block for Vdivider {
    type Io = VdividerIo;

    fn id() -> ArcStr {
        arcstr::literal!("vdivider")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_{}_{}", self.r1.name(), self.r2.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Block for ArrayShorter {
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

impl HasSchematic for Resistor {
    type Data = ();
}

impl HasSchematic for Vdivider {
    type Data = ();
}

impl HasSchematic for ArrayShorter {
    type Data = ();
}

impl<PDK: Pdk> HasSchematicImpl<PDK> for Resistor {
    fn schematic(
        &self,
        io: &ResistorIoSchematic,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(PrimitiveDevice::Res2 {
            pos: *io.p,
            neg: *io.n,
            value: dec!(1000),
        });
        Ok(())
    }
}

impl<PDK: Pdk> HasSchematicImpl<PDK> for Vdivider {
    fn schematic(
        &self,
        io: &VdividerIoSchematic,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let r1 = cell.instantiate(self.r1);
        let r2 = cell.instantiate(self.r2);

        cell.connect(io.pwr.vdd, r1.io().p);
        cell.connect(io.out, r1.io().n);
        cell.connect(io.out, r2.io().p);
        cell.connect(io.pwr.vss, r2.io().n);
        Ok(())
    }
}

impl<PDK: Pdk> HasSchematicImpl<PDK> for ArrayShorter {
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
