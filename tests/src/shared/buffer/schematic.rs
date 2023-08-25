use crate::shared::buffer::{Buffer, BufferNxM};
use substrate::block::Block;
use substrate::io::{SchematicType, Terminal};
use substrate::pdk::{ExportsPdkSchematicData, Pdk, PdkSchematic, ToSchema};
use substrate::schematic::schema::Schema;
use substrate::schematic::CellBuilder;
use substrate::{
    io::Signal,
    schematic::{ExportsSchematicData, Instance, NestedInstance, Schematic, SchematicData},
};

use crate::shared::pdk::{ExamplePdkA, NmosA, PmosA};

use super::{BufferN, Inverter};

#[derive(SchematicData)]
pub struct InverterData<PDK: Pdk, S: Schema> {
    pub pmos_g: Terminal,
    pub pmos: Instance<PDK, S, PmosA>,
}

impl<PDK: Pdk> ExportsPdkSchematicData<PDK> for Inverter {
    type Data<S> = InverterData<PDK, S> where PDK: ToSchema<S>;
}

impl PdkSchematic<ExamplePdkA> for Inverter {
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA, S>,
    ) -> substrate::error::Result<Self::Data<S>>
    where
        ExamplePdkA: ToSchema<S>,
    {
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
pub struct BufferData<PDK: Pdk, S: Schema> {
    #[substrate(nested)]
    pub inv1: Instance<PDK, S, Inverter>,
    #[substrate(nested)]
    pub inv2: Instance<PDK, S, Inverter>,
}

impl<PDK: Pdk> ExportsPdkSchematicData<PDK> for Buffer {
    type Data<S> = BufferData<PDK, S> where PDK: ToSchema<S>;
}

impl PdkSchematic<ExamplePdkA> for Buffer {
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA, S>,
    ) -> substrate::error::Result<Self::Data<S>>
    where
        ExamplePdkA: ToSchema<S>,
    {
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
pub struct BufferNData<PDK: Pdk, S: Schema> {
    pub bubbled_pmos_g: Terminal,
    pub bubbled_inv1: NestedInstance<PDK, S, Inverter>,
    pub buffers: Vec<Instance<PDK, S, Buffer>>,
}

impl<PDK: Pdk> ExportsPdkSchematicData<PDK> for BufferN {
    type Data<S> = BufferNData<PDK, S> where PDK: ToSchema<S>;
}

impl PdkSchematic<ExamplePdkA> for BufferN {
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA, S>,
    ) -> substrate::error::Result<Self::Data<S>>
    where
        ExamplePdkA: ToSchema<S>,
    {
        let mut buffers = Vec::new();
        for _ in 0..self.n {
            buffers.push(cell.instantiate(Buffer::new(self.strength)));
        }

        cell.connect(io.din, buffers[0].io().din);
        cell.connect(io.dout, buffers[self.n - 1].io().dout);

        for i in 1..self.n {
            cell.connect(buffers[i].io().din, buffers[i - 1].io().dout);
        }

        let bubbled_pmos_g = buffers[0].data().inv1.data().pmos_g;
        let bubbled_inv1 = buffers[0].data().inv1.to_owned();

        Ok(BufferNData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffers,
        })
    }
}

#[derive(SchematicData)]
pub struct BufferNxMData<PDK: Pdk, S: Schema> {
    pub bubbled_pmos_g: Terminal,
    pub bubbled_inv1: NestedInstance<PDK, S, Inverter>,
    pub buffer_chains: Vec<Instance<PDK, S, BufferN>>,
}

impl<PDK: Pdk> ExportsPdkSchematicData<PDK> for BufferNxM {
    type Data<S> = BufferNxMData<PDK, S> where PDK: ToSchema<S>;
}

impl PdkSchematic<ExamplePdkA> for BufferNxM {
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA, S>,
    ) -> substrate::error::Result<Self::Data<S>>
    where
        ExamplePdkA: ToSchema<S>,
    {
        let mut buffer_chains = Vec::new();
        for i in 0..self.n {
            let buffer = cell.instantiate(BufferN::new(self.strength, self.n));
            cell.connect(io.din[i], buffer.io().din);
            cell.connect(io.dout[i], buffer.io().dout);
            cell.connect(io.vdd, buffer.io().vdd);
            cell.connect(io.vss, buffer.io().vss);
            buffer_chains.push(buffer);
        }

        let bubbled_pmos_g = buffer_chains[0].data().bubbled_pmos_g;
        let bubbled_inv1 = buffer_chains[0].data().bubbled_inv1.to_owned();

        Ok(BufferNxMData {
            bubbled_pmos_g,
            bubbled_inv1,
            buffer_chains,
        })
    }
}
