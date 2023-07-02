use crate::shared::buffer::Buffer;
use substrate::{
    io::{Node, NodePathView, Signal},
    schematic::{
        HasPathView, HasSchematic, HasSchematicImpl, Instance, OwnedInstancePathView, PathView,
    },
};

use crate::shared::pdk::{ExamplePdkA, NmosA, PmosA};

use super::{BufferN, Inverter};

pub struct InverterData {
    pub din: Node,
    pub pmos: Instance<PmosA>,
}

pub struct InverterDataPathView<'a> {
    pub din: PathView<'a, Node>,
    pub pmos: PathView<'a, Instance<PmosA>>,
}

impl HasPathView for InverterData {
    type PathView<'a>
    where
        Self: 'a,
    = InverterDataPathView<'a>;

    fn path_view<'a>(
        &'a self,
        parent: Option<std::sync::Arc<substrate::schematic::RetrogradeEntry>>,
    ) -> Self::PathView<'a> {
        Self::PathView {
            din: self.din.path_view(parent.clone()),
            pmos: self.pmos.path_view(parent.clone()),
        }
    }
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

pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

pub struct BufferDataPathView<'a> {
    pub inv1: PathView<'a, Instance<Inverter>>,
}

impl HasPathView for BufferData {
    type PathView<'a> = BufferDataPathView<'a>;

    fn path_view<'a>(
        &'a self,
        parent: Option<std::sync::Arc<substrate::schematic::RetrogradeEntry>>,
    ) -> Self::PathView<'a> {
        Self::PathView {
            inv1: self.inv1.path_view(parent),
        }
    }
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

pub struct BufferNData {
    pub bubbled_din: NodePathView,
    pub bubbled_inv1: OwnedInstancePathView<Inverter>,
    pub buffers: Vec<Instance<Buffer>>,
}

pub struct BufferNDataPathView<'a> {
    pub buffers: PathView<'a, Vec<Instance<Buffer>>>,
}

impl HasPathView for BufferNData {
    type PathView<'a> = BufferNDataPathView<'a>;

    fn path_view<'a>(
        &'a self,
        parent: Option<std::sync::Arc<substrate::schematic::RetrogradeEntry>>,
    ) -> Self::PathView<'a> {
        Self::PathView {
            buffers: self.buffers.path_view(parent),
        }
    }
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
        for i in 0..self.n {
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
