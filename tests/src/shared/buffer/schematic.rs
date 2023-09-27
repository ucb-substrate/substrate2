use crate::shared::buffer::{Buffer, BufferNxM};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use substrate::block::{Block, PdkCell};
use substrate::io::{MosIo, NestedTerminal, SchematicType, Terminal, TerminalView};
use substrate::pdk::{Pdk, PdkCellSchematic, PdkSchematic};
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, InstancePath, NestedInstance, PdkCellBuilder};
use substrate::type_dispatch::impl_dispatch;
use substrate::{
    io::Signal,
    schematic::{ExportsNestedData, Instance, Schematic, SchematicData},
};

use crate::shared::pdk::{
    ExamplePdkA, ExamplePdkB, ExamplePdkC, NmosA, NmosB, NmosC, PmosA, PmosB, PmosC,
};

use super::{BufferN, Inverter};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "MosIo", kind = "PdkCell")]
pub enum InverterMos {
    Nmos,
    Pmos,
}

#[derive(SchematicData)]
pub enum InverterMosData {
    NmosA(Instance<NmosA>),
    PmosA(Instance<PmosA>),
    NmosB(Instance<NmosB>),
    PmosB(Instance<PmosB>),
}

impl From<Instance<NmosA>> for InverterMosData {
    fn from(value: Instance<NmosA>) -> Self {
        Self::NmosA(value)
    }
}

impl From<Instance<PmosA>> for InverterMosData {
    fn from(value: Instance<PmosA>) -> Self {
        Self::PmosA(value)
    }
}

impl From<Instance<NmosB>> for InverterMosData {
    fn from(value: Instance<NmosB>) -> Self {
        Self::NmosB(value)
    }
}

impl From<Instance<PmosB>> for InverterMosData {
    fn from(value: Instance<PmosB>) -> Self {
        Self::PmosB(value)
    }
}

impl InverterMosDataNestedView {
    pub fn path(&self) -> &InstancePath {
        match self {
            Self::NmosA(inst) => inst.path(),
            Self::PmosA(inst) => inst.path(),
            Self::NmosB(inst) => inst.path(),
            Self::PmosB(inst) => inst.path(),
        }
    }
}

impl ExportsNestedData for InverterMos {
    type NestedData = InverterMosData;
}

impl PdkCellSchematic<ExamplePdkA> for InverterMos {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        match self {
            Self::Pmos => {
                let inst = cell.instantiate(PmosA { w: 500, l: 150 });
                cell.connect(inst.io(), io);
                Ok(inst.into())
            }
            Self::Nmos => {
                let inst = cell.instantiate(NmosA { w: 500, l: 150 });
                cell.connect(inst.io(), io);
                Ok(inst.into())
            }
        }
    }
}

impl PdkCellSchematic<ExamplePdkB> for InverterMos {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::NestedData> {
        match self {
            Self::Pmos => {
                let inst = cell.instantiate(PmosB { w: 500, l: 150 });
                cell.connect(inst.io(), io);
                Ok(inst.into())
            }
            Self::Nmos => {
                let inst = cell.instantiate(NmosB { w: 500, l: 150 });
                cell.connect(inst.io(), io);
                Ok(inst.into())
            }
        }
    }
}

#[derive(SchematicData)]
pub struct InverterData {
    pub pmos_g: Terminal,
    pub pmos: Option<Instance<InverterMos>>,
}

impl ExportsNestedData for Inverter {
    type NestedData = InverterData;
}

#[impl_dispatch({ExamplePdkA; ExamplePdkB})]
impl<PDK> PdkCellSchematic<PDK> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(InverterMos::Nmos);
        let nmos = nmos.io();

        let pmos = cell.instantiate(InverterMos::Pmos);
        let pmos_io = pmos.io();

        for mos in [nmos, pmos_io] {
            cell.connect(io.dout, mos.d);
            cell.connect(io.din, mos.g);
        }

        cell.connect(io.vdd, pmos_io.s);
        cell.connect(io.vss, nmos.s);
        Ok(InverterData {
            pmos_g: pmos.io().g,
            pmos: Some(pmos),
        })
    }
}

impl PdkCellSchematic<ExamplePdkC> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkC>,
    ) -> substrate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(NmosC { w: 50, l: 50 });
        let nmos = nmos.io();

        let pmos = cell.instantiate(PmosC { w: 50, l: 50 });
        let pmos_io = pmos.io();

        for mos in [nmos, pmos_io] {
            cell.connect(io.dout, mos.d);
            cell.connect(io.din, mos.g);
        }

        cell.connect(io.vdd, pmos_io.s);
        cell.connect(io.vss, nmos.s);
        Ok(InverterData {
            pmos_g: pmos.io().g,
            pmos: None,
        })
    }
}

#[derive(SchematicData)]
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl ExportsNestedData for Buffer {
    type NestedData = BufferData;
}

#[impl_dispatch({ExamplePdkA; ExamplePdkB; ExamplePdkC})]
impl<PDK> PdkCellSchematic<PDK> for Buffer {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        let inv1 = cell.instantiate(Inverter::new(self.strength));
        let inv1_io = inv1.io();

        let inv2 = cell.instantiate(Inverter::new(self.strength));
        let inv2_io = inv2.io();

        let x = cell.signal("x", Signal);

        cell.connect(x, inv1_io.dout);
        cell.connect(x, inv2_io.din);

        cell.connect(io.din, inv1_io.din);
        cell.connect(io.dout, inv2_io.dout);

        for inv in [inv1_io, inv2_io] {
            cell.connect(io.vdd, inv.vdd);
            cell.connect(io.vss, inv.vss);
        }

        Ok(BufferData { inv1, inv2 })
    }
}

#[derive(SchematicData)]
pub struct BufferNData {
    pub bubbled_pmos_g: NestedTerminal,
    pub bubbled_inv1: NestedInstance<Inverter>,
    pub buffers: Vec<Instance<Buffer>>,
}

impl ExportsNestedData for BufferN {
    type NestedData = BufferNData;
}

impl<PDK: Pdk> PdkCellSchematic<PDK> for BufferN
where
    Buffer: PdkSchematic<PDK>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut buffers = Vec::new();
        for _ in 0..self.n {
            buffers.push(cell.instantiate(Buffer::new(self.strength)));
        }

        cell.connect(io.din, buffers[0].io().din);
        cell.connect(io.dout, buffers[self.n - 1].io().dout);

        for i in 1..self.n {
            cell.connect(buffers[i].io().din, buffers[i - 1].io().dout);
        }

        let bubbled_pmos_g = buffers[0].inv1.pmos_g.clone();
        let bubbled_inv1 = buffers[0].inv1.clone();

        Ok(BufferNData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffers,
        })
    }
}

#[derive(SchematicData)]
pub struct BufferNxMData {
    pub bubbled_pmos_g: NestedTerminal,
    pub bubbled_inv1: NestedInstance<Inverter>,
    pub buffer_chains: Vec<Instance<BufferN>>,
}

impl ExportsNestedData for BufferNxM {
    type NestedData = BufferNxMData;
}

impl<PDK: Pdk> PdkCellSchematic<PDK> for BufferNxM
where
    BufferN: PdkSchematic<PDK>,
{
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut buffer_chains = Vec::new();
        for i in 0..self.n {
            let buffer = cell.instantiate(BufferN::new(self.strength, self.n));
            cell.connect(io.din[i], buffer.io().din);
            cell.connect(io.dout[i], buffer.io().dout);
            cell.connect(io.vdd, buffer.io().vdd);
            cell.connect(io.vss, buffer.io().vss);
            buffer_chains.push(buffer);
        }

        let bubbled_pmos_g = buffer_chains[0].bubbled_pmos_g.clone();
        let bubbled_inv1 = buffer_chains[0].bubbled_inv1.clone();

        Ok(BufferNxMData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffer_chains,
        })
    }
}
