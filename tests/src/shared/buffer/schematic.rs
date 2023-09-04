use crate::shared::buffer::{Buffer, BufferNxM};
use substrate::block::Block;
use substrate::io::{SchematicType, Terminal};
use substrate::pdk::{Pdk, PdkSchematic, ToSchema};
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, InstanceData, PdkCellBuilder};
use substrate::{
    io::Signal,
    schematic::{ExportsNestedData, Instance, Schematic, SchematicData},
};

use crate::shared::pdk::{ExamplePdkA, NmosA, PmosA};

use super::{BufferN, Inverter};

#[derive(SchematicData)]
pub struct InverterData {
    pub pmos_g: Terminal,
    pub pmos: Instance<PmosA>,
}

impl ExportsNestedData for Inverter {
    type NestedData = InverterData;
}

impl PdkSchematic<ExamplePdkA> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(NmosA { w: 500, l: 150 });
        let nmos = nmos.io();

        let pmos = cell.instantiate(PmosA { w: 500, l: 150 });
        let pmos_io = pmos.io();

        for mos in [nmos, pmos_io] {
            cell.connect(io.dout, mos.d);
            cell.connect(io.din, mos.g);
        }

        cell.connect(io.vdd, pmos_io.s);
        cell.connect(io.vss, nmos.s);
        Ok(InverterData {
            pmos_g: pmos.terminals().g,
            pmos,
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

impl PdkSchematic<ExamplePdkA> for Buffer {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkA>,
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
    pub bubbled_pmos_g: Terminal,
    pub bubbled_inv1: Instance<Inverter>,
    pub buffers: Vec<Instance<Buffer>>,
}

impl ExportsNestedData for BufferN {
    type NestedData = BufferNData;
}

impl PdkSchematic<ExamplePdkA> for BufferN {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkA>,
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

        let bubbled_pmos_g = buffers[0].nodes().inv1.nodes().pmos_g;
        let bubbled_inv1 = buffers[0].nodes().inv1.to_owned();

        Ok(BufferNData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffers,
        })
    }
}

#[derive(SchematicData)]
pub struct BufferNxMData {
    pub bubbled_pmos_g: Terminal,
    pub bubbled_inv1: Instance<Inverter>,
    pub buffer_chains: Vec<Instance<BufferN>>,
}

impl ExportsNestedData for BufferNxM {
    type NestedData = BufferNxMData;
}

impl PdkSchematic<ExamplePdkA> for BufferNxM {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<ExamplePdkA>,
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

        let bubbled_pmos_g = buffer_chains[0].nodes().bubbled_pmos_g;
        let bubbled_inv1 = buffer_chains[0].nodes().bubbled_inv1.to_owned();

        Ok(BufferNxMData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffer_chains,
        })
    }
}
