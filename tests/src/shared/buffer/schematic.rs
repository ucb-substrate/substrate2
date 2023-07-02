use crate::shared::buffer::Buffer;
use substrate::{
    io::{Node, NodePathView, Signal},
    schematic::{HasSchematic, HasSchematicImpl, Instance, OwnedInstancePathView, PathView},
    SchematicData,
};

use crate::shared::pdk::{ExamplePdkA, NmosA, PmosA};

use super::{BufferN, Inverter};

#[derive(SchematicData)]
pub struct InverterData {
    #[path_view]
    pub din: Node,
    #[path_view]
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
    #[path_view]
    pub inv1: Instance<Inverter>,
    #[path_view]
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
    #[path_view]
    pub bubbled_din: NodePathView,
    #[path_view]
    pub bubbled_inv1: OwnedInstancePathView<Inverter>,
    #[path_view]
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
        cell.connect(io.dout, buffers[self.n - 1].io().din);

        for i in 1..self.n {
            cell.connect(buffers[i].io().din, buffers[i - 1].io().dout);
        }

        let bubbled_din = buffers[0].cell().data.inv1.cell().data.din;
        let bubbled_inv1 = buffers[0].cell().data.inv1.to_owned();

        Ok(BufferNData {
            bubbled_din,
            bubbled_inv1,
            buffers,
        })
    }
}
