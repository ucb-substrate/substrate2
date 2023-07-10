use crate::shared::buffer::{Buffer, BufferNxM};
use substrate::{
    io::{NestedNode, Node, Signal},
    schematic::{HasSchematic, HasSchematicImpl, Instance, NestedInstance},
    SchematicData,
};

use crate::shared::pdk::{ExamplePdkA, NmosA, PmosA};

use super::{BufferN, Inverter};

#[derive(SchematicData)]
pub struct InverterData {
    #[substrate(nested)]
    pub din: Node,
    #[substrate(nested)]
    pub pmos: Instance<PmosA>,
}

impl HasSchematic for Inverter {
    type Data = InverterData;
}

impl HasSchematicImpl<ExamplePdkA> for Inverter {
    fn schematic(
        &self,
        io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
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
        Ok(InverterData { din: *io.din, pmos })
    }
}

#[derive(SchematicData)]
pub struct BufferData {
    #[substrate(nested)]
    pub inv1: Instance<Inverter>,
    #[substrate(nested)]
    pub inv2: Instance<Inverter>,
}

impl HasSchematic for Buffer {
    type Data = BufferData;
}

impl HasSchematicImpl<ExamplePdkA> for Buffer {
    fn schematic(
        &self,
        io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
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
    #[substrate(nested)]
    pub bubbled_din: NestedNode,
    #[substrate(nested)]
    pub bubbled_inv1: NestedInstance<Inverter>,
    #[substrate(nested)]
    pub buffers: Vec<Instance<Buffer>>,
}

impl HasSchematic for BufferN {
    type Data = BufferNData;
}

impl HasSchematicImpl<ExamplePdkA> for BufferN {
    fn schematic(
        &self,
        io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut buffers = Vec::new();
        for _ in 0..self.n {
            buffers.push(cell.instantiate(Buffer::new(self.strength)));
        }

        cell.connect(io.din, buffers[0].io().din);
        cell.connect(io.dout, buffers[self.n - 1].io().dout);

        for i in 1..self.n {
            cell.connect(buffers[i].io().din, buffers[i - 1].io().dout);
        }

        let bubbled_din = buffers[0].data().inv1.data().din;
        let bubbled_inv1 = buffers[0].data().inv1.to_owned();

        Ok(BufferNData {
            bubbled_din,
            bubbled_inv1,
            buffers,
        })
    }
}

#[derive(SchematicData)]
pub struct BufferNxMData {
    #[substrate(nested)]
    pub bubbled_din: NestedNode,
    #[substrate(nested)]
    pub bubbled_inv1: NestedInstance<Inverter>,
    #[substrate(nested)]
    pub buffer_chains: Vec<Instance<BufferN>>,
}

impl HasSchematic for BufferNxM {
    type Data = BufferNxMData;
}

impl HasSchematicImpl<ExamplePdkA> for BufferNxM {
    fn schematic(
        &self,
        io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut buffer_chains = Vec::new();
        for i in 0..self.n {
            let buffer = cell.instantiate(BufferN::new(self.strength, self.n));
            cell.connect(io.din[i], buffer.io().din);
            cell.connect(io.dout[i], buffer.io().dout);
            cell.connect(io.vdd, buffer.io().vdd);
            cell.connect(io.vss, buffer.io().vss);
            buffer_chains.push(buffer);
        }

        let bubbled_din = buffer_chains[0].data().bubbled_din;
        let bubbled_inv1 = buffer_chains[0].data().bubbled_inv1.to_owned();

        Ok(BufferNxMData {
            bubbled_din,
            bubbled_inv1,
            buffer_chains,
        })
    }
}
