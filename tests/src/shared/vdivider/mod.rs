use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::Io;
use substrate::io::{Array, InOut, Output, PowerIo, Signal};
use substrate::schematic::primitives::Resistor;
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, ExportsNestedData, Instance, NestedData, Schematic};

pub mod flattened;
pub mod tb;

#[derive(Debug, Default, Clone, Io)]
pub struct VdividerIo {
    pub pwr: PowerIo,
    pub out: Output<Signal>,
}

#[derive(Debug, Default, Clone, Io)]
pub struct VdividerFlatIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub out: Output<Signal>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vdivider {
    pub r1: Resistor,
    pub r2: Resistor,
}

impl Vdivider {
    #[inline]
    fn new(r1: impl Into<Decimal>, r2: impl Into<Decimal>) -> Self {
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

impl Block for Vdivider {
    type Io = VdividerIo;

    fn id() -> ArcStr {
        arcstr::literal!("vdivider")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_{}_{}", self.r1.value(), self.r2.value())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Debug, Clone, Io)]
pub struct VdividerArrayIo {
    pub elements: Array<PowerIo>,
}

impl Block for VdividerArray {
    type Io = VdividerArrayIo;

    fn id() -> ArcStr {
        arcstr::literal!("vdivider_array")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_array_{}", self.vdividers.len())
    }

    fn io(&self) -> Self::Io {
        VdividerArrayIo {
            elements: Array::new(self.vdividers.len(), Default::default()),
        }
    }
}

#[derive(NestedData)]
pub struct VdividerData {
    r1: Instance<Resistor>,
    r2: Instance<Resistor>,
}

impl ExportsNestedData for Vdivider {
    type NestedData = VdividerData;
}

impl ExportsNestedData for VdividerArray {
    type NestedData = Vec<Instance<Vdivider>>;
}

impl<S: Schema> Schematic<S> for Vdivider
where
    Resistor: Schematic<S>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<S>,
    ) -> substrate::error::Result<Self::NestedData> {
        let r1 = cell.instantiate(self.r1);
        let r2 = cell.instantiate(self.r2);

        cell.connect(io.pwr.vdd, r1.io().p);
        cell.connect(io.out, r1.io().n);
        cell.connect(io.out, r2.io().p);
        cell.connect(io.pwr.vss, r2.io().n);
        Ok(VdividerData { r1, r2 })
    }
}

impl<S: Schema> Schematic<S> for VdividerArray
where
    Resistor: Schematic<S>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<S>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut vdividers = Vec::new();

        for (i, vdivider) in self.vdividers.iter().enumerate() {
            let vdiv = cell.instantiate(*vdivider);

            cell.connect(&vdiv.io().pwr, &io.elements[i]);

            vdividers.push(vdiv);
        }

        Ok(vdividers)
    }
}
