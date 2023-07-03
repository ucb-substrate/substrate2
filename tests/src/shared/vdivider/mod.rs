use arcstr::ArcStr;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::{InOut, Output, Signal};
use substrate::pdk::Pdk;
use substrate::schematic::{CellBuilder, HasSchematic, HasSchematicImpl, Instance, PrimitiveDevice};
use substrate::{Io, SchematicData};

pub mod tb;

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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resistor {
    pub r: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vdivider {
    pub r1: Resistor,
    pub r2: Resistor,
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
        arcstr::format!("vdivider_{}_{}", self.r1.r, self.r2.r)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Resistor {
    type Data = ();
}

#[derive(SchematicData)]
pub struct VdividerData {
    r1: Instance<Resistor>,
    r2: Instance<Resistor>,
}

impl HasSchematic for Vdivider {
    type Data = VdividerData;
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
        Ok(VdividerData {
            r1, r2
        })
    }
}
