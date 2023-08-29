use super::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block;
use substrate::block::Block;
use substrate::io::SchematicType;
use substrate::pdk::{Pdk, PdkSchematic, ToSchema};
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, ExportsNestedNodes, Instance, Schematic, SchematicData};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Block)]
#[substrate(io = "VdividerIo", kind = "block::Cell")]
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
    type Kind = block::Cell;
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

impl ExportsNestedNodes for Vdivider {
    type NestedNodes = VdividerData;
}

impl ExportsNestedNodes for VdividerArray {
    type NestedNodes = Vec<Instance<PDK, S, Vdivider>>;
}

#[derive(SchematicData)]
pub struct VdividerData {
    pub r1: InstanceData<Resistor>,
    pub r2: InstanceData<Resistor>,
}

impl<PDK: Pdk> PdkSchematic<PDK> for Vdivider {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedNodes> {
        let r1 = cell.instantiate(self.r1);
        let r2 = cell.instantiate(self.r2);

        cell.connect(io.pwr.vdd, r1.io().p);
        cell.connect(io.out, r1.io().n);
        cell.connect(io.out, r2.io().p);
        cell.connect(io.pwr.vss, r2.io().n);
        Ok(VdividerData { r1, r2 })
    }
}

impl<PDK: Pdk> PdkSchematic<PDK> for VdividerArray {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedNodes> {
        let mut vdividers = Vec::new();

        for (i, vdivider) in self.vdividers.iter().enumerate() {
            let vdiv = cell.instantiate(*vdivider);

            cell.connect(&vdiv.io().pwr, &io.elements[i]);

            vdividers.push(vdiv);
        }

        Ok(vdividers)
    }
}
