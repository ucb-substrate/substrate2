use super::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::pdk::Pdk;
use substrate::schematic::{CellBuilder, HasSchematic, HasSchematicData, Instance};
use substrate::Block;
use substrate::SchematicData;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Block)]
#[substrate(io = "ResistorIo", flatten)]
pub struct Resistor {
    pub value: Decimal,
}

impl Resistor {
    #[inline]
    pub fn new(value: impl Into<Decimal>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Block)]
#[substrate(io = "VdividerIo", flatten)]
pub struct Vdivider {
    pub r1: Resistor,
    pub r2: Resistor,
}

impl Vdivider {
    #[inline]
    pub fn new(r1: impl Into<Decimal>, r2: impl Into<Decimal>) -> Self {
        Self {
            r1: Resistor::new(r1),
            r2: Resistor::new(r2),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VdividerArray {
    pub vdividers: Vec<Vdivider>,
}

impl Block for VdividerArray {
    type Io = VdividerArrayIo;

    fn id() -> ArcStr {
        arcstr::literal!("flattened_vdivider_array")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("flattened_vdivider_array_{}", self.vdividers.len())
    }

    fn io(&self) -> Self::Io {
        VdividerArrayIo {
            elements: Array::new(self.vdividers.len(), Default::default()),
        }
    }
}

impl HasSchematicData for Resistor {
    type Data = ();
}

impl HasSchematicData for Vdivider {
    type Data = VdividerData;
}

impl HasSchematicData for VdividerArray {
    type Data = Vec<Instance<Vdivider>>;
}

#[derive(SchematicData)]
pub struct VdividerData {
    #[substrate(nested)]
    pub r1: Instance<Resistor>,
    #[substrate(nested)]
    pub r2: Instance<Resistor>,
}

impl<PDK: Pdk> HasSchematic<PDK> for Resistor {
    fn schematic(
        &self,
        io: &ResistorIoSchematic,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(
            PrimitiveDeviceKind::Res2 {
                pos: io.p,
                neg: io.n,
                value: self.value,
            }
            .into(),
        );
        Ok(())
    }
}

impl<PDK: Pdk> HasSchematic<PDK> for Vdivider {
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
        Ok(VdividerData { r1, r2 })
    }
}

impl<PDK: Pdk> HasSchematic<PDK> for VdividerArray {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut vdividers = Vec::new();

        for (i, vdivider) in self.vdividers.iter().enumerate() {
            let vdiv = cell.instantiate(*vdivider);

            cell.connect(&vdiv.io().pwr, &io.elements[i]);

            vdividers.push(vdiv);
        }

        Ok(vdividers)
    }
}
