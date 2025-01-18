#![allow(dead_code)]

use std::collections::HashSet;

use anyhow::anyhow;
use arcstr::ArcStr;
use codegen::Io;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::{Instance, NestedInstance};
use crate::context::Context;
use crate::schematic::CellBuilder;
use crate::tests::{Buffer, BufferN, BufferNxM, Inverter, InverterMos};
use crate::types::schematic::{DataView, IoNodeBundle, NestedTerminal, NodeBundle, Terminal};
use crate::types::{Array, Flipped, HasBundleKind, Input, PowerIo};
use crate::{
    block::Block,
    schematic::{conv::RawLib, NestedData, PrimitiveBinding, Schematic},
    types::{HasNameTree, InOut, NameTree, Output, Signal},
};

#[derive(Default, NestedData)]
pub struct SchematicInstances<T: Schematic> {
    pub instances: Vec<Instance<T>>,
}
#[derive(NestedData)]
pub enum EnumInstances<T: Schematic> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}
#[derive(Default, NestedData)]
pub struct SchematicInstancesWithWhereClause<T>
where
    T: Schematic,
{
    pub instances: Vec<Instance<T>>,
}

#[derive(NestedData)]
pub struct TwoInstances<T: Schematic>(pub Instance<T>, pub Instance<T>);

pub struct Schema;

#[derive(Clone, Debug, Copy)]
pub enum Primitive {
    Resistor(Decimal),
    Pmos,
    Nmos,
}

impl scir::schema::Schema for Schema {
    type Primitive = Primitive;
}

#[derive(Io, Clone, Default, Debug)]
pub struct ResistorIo {
    pub p: InOut<Signal>,
    pub n: InOut<Signal>,
}

#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "ResistorIo")]
pub struct Resistor(pub Decimal);

impl Schematic for Resistor {
    type Schema = Schema;
    type NestedData = ();

    fn schematic(
        &self,
        io: &IoNodeBundle<Resistor>,
        cell: &mut super::CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::Resistor(self.0));
        prim.connect("p", io.p);
        prim.connect("n", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

#[derive(Io, Clone, Default, Debug)]
pub struct VdividerIo {
    pub pwr: PowerIo,
    pub out: Output<Signal>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Vdivider {
    r1: Decimal,
    r2: Decimal,
    flatten: bool,
}

impl Vdivider {
    pub fn new(r1: impl Into<Decimal>, r2: impl Into<Decimal>) -> Self {
        Vdivider {
            r1: r1.into(),
            r2: r2.into(),
            flatten: false,
        }
    }
    pub fn flattened(mut self) -> Self {
        self.flatten = true;
        self
    }
}

impl Block for Vdivider {
    type Io = VdividerIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_{}_{}", self.r1, self.r2)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(NestedData)]
pub struct VdividerData {
    r1: Instance<Resistor>,
    r2: Instance<Resistor>,
}

impl Schematic for Vdivider {
    type Schema = Schema;
    type NestedData = VdividerData;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        if self.flatten {
            cell.flatten();
        }

        let r1 = cell.instantiate(Resistor(self.r1));
        let r2 = cell.instantiate(Resistor(self.r2));

        cell.connect(io.pwr.vdd, r1.io().p);
        cell.connect(io.out, r1.io().n);
        cell.connect(io.out, r2.io().p);
        cell.connect(io.pwr.vss, r2.io().n);
        Ok(VdividerData { r1, r2 })
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct VdividerArray {
    pub vdividers: Vec<Vdivider>,
}

#[derive(Debug, Clone, Io)]
pub struct VdividerArrayIo {
    pub elements: Array<PowerIo>,
}

impl Block for VdividerArray {
    type Io = VdividerArrayIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_array_{}", self.vdividers.len())
    }

    fn io(&self) -> Self::Io {
        VdividerArrayIo {
            elements: Array::new(self.vdividers.len(), Default::default()),
        }
    }
}
impl Schematic for VdividerArray {
    type Schema = Schema;
    type NestedData = Vec<Instance<Vdivider>>;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let mut vdividers = Vec::new();

        for (i, vdivider) in self.vdividers.iter().enumerate() {
            let vdiv = cell.instantiate(*vdivider);

            cell.connect(&vdiv.io().pwr, &io.elements[i]);

            vdividers.push(vdiv);
        }

        Ok(vdividers)
    }
}

#[derive(Io, Clone, Debug)]
pub struct DecoupledIo {
    pub ready: Input<Signal>,
    pub valid: Output<Signal>,
    pub data: Output<Array<Signal>>,
}

impl DecoupledIo {
    fn new(width: usize) -> Self {
        Self {
            ready: Default::default(),
            valid: Default::default(),
            data: Output(Array::new(width, Default::default())),
        }
    }
}

#[derive(Io, Clone, Debug)]
pub struct MultiDecoupledIo {
    pub d1: DecoupledIo,
    pub d2: Flipped<DecoupledIo>,
    pub d3: Input<DecoupledIo>,
    pub d4: Output<DecoupledIo>,
    pub d5: InOut<DecoupledIo>,
    pub ready: Input<Signal>,
    pub valid: Output<Signal>,
    pub data: Output<Array<Signal>>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MultiDecoupledBlock;

impl Block for MultiDecoupledBlock {
    type Io = MultiDecoupledIo;
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("multi_decoupled_block")
    }
    fn io(&self) -> Self::Io {
        MultiDecoupledIo {
            d1: DecoupledIo::new(5),
            d2: Flipped(DecoupledIo::new(4)),
            d3: Input(DecoupledIo::new(5)),
            d4: Output(DecoupledIo::new(4)),
            d5: InOut(DecoupledIo::new(5)),
            ready: Default::default(),
            valid: Default::default(),
            data: Output(Array::new(4, Default::default())),
        }
    }
}

impl DataView<DecoupledIoKind> for MultiDecoupledIoKind {
    fn view_nodes_as(nodes: &NodeBundle<Self>) -> NodeBundle<DecoupledIo> {
        NodeBundle::<DecoupledIo> {
            ready: nodes.ready,
            valid: nodes.valid,
            data: nodes.data.clone(),
        }
    }
}

impl Schematic for MultiDecoupledBlock {
    type Schema = Schema;
    type NestedData = ();

    fn schematic(
        &self,
        _io: &IoNodeBundle<Self>,
        _cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        Ok(())
    }
}

#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "()")]
pub struct SuperBlock;

impl Schematic for SuperBlock {
    type Schema = Schema;
    type NestedData = ();
    fn schematic(
        &self,
        _io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let b1 = cell.instantiate(MultiDecoupledBlock);
        let b2 = cell.instantiate(MultiDecoupledBlock);
        let wire = cell.signal(
            "abc",
            MultiDecoupledIo {
                d1: DecoupledIo::new(4),
                d2: Flipped(DecoupledIo::new(5)),
                d3: Input(DecoupledIo::new(4)),
                d4: Output(DecoupledIo::new(5)),
                d5: InOut(DecoupledIo::new(4)),
                ready: Default::default(),
                valid: Default::default(),
                data: Output(Array::new(5, Default::default())),
            },
        );

        assert!(b1.io().kind() == b2.io().kind());
        assert!(b1.io().d1.kind() == b2.io().d1.kind());
        assert!(b1.io().d1.kind() != b2.io().d2.kind());
        assert!(b1.io().d1.kind() == b2.io().d3.kind());
        assert!(b1.io().d1.kind() != b2.io().d4.kind());
        assert!(b1.io().d1.kind() == b2.io().d5.kind());
        assert!(b1.io().d2.kind() == b2.io().d4.kind());
        assert!(b1.io().d2.kind() == b2.io().view_as::<DecoupledIoKind>().kind());

        cell.connect(&b1.io().d1, &b2.io().d1);
        cell.connect(&b1.io().d1, &b2.io().d1);
        cell.connect(&b1.io().d1, &b2.io().d1);
        cell.connect(&b1.io().d1, &b2.io().d3);
        cell.connect(&b1.io().d1, &b2.io().d5);
        cell.connect(&b1.io().d2, b2.io().view_as::<DecoupledIoKind>());
        cell.connect(wire.d2, &b1.io().d1);

        Ok(())
    }
}

#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "()")]
pub struct NestedBlock<T>(T);

impl<S: crate::schematic::schema::Schema, T: Schematic<Schema = S> + Clone> Schematic
    for NestedBlock<T>
{
    type Schema = S;
    type NestedData = ();
    fn schematic(
        &self,
        _io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let _b1 = cell.instantiate(self.0.clone());

        Ok(())
    }
}

impl Schematic for InverterMos {
    type Schema = Schema;
    type NestedData = ();

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(match self {
            Self::Pmos => Primitive::Pmos,
            Self::Nmos => Primitive::Nmos,
        });
        prim.connect("d", io.d);
        prim.connect("g", io.g);
        prim.connect("s", io.s);
        prim.connect("b", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

#[derive(NestedData)]
pub struct InverterData {
    pub pmos_g: Terminal,
    pub pmos: Instance<InverterMos>,
}

impl Schematic for Inverter {
    type Schema = Schema;
    type NestedData = InverterData;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(InverterMos::Nmos);
        let nmos_io = nmos.io();

        let pmos = cell.instantiate(InverterMos::Pmos);
        let pmos_io = pmos.io();

        for mos in [nmos_io, pmos_io] {
            cell.connect(io.dout, mos.d);
            cell.connect(io.din, mos.g);
        }

        cell.connect(io.vdd, pmos_io.s);
        cell.connect(io.vss, nmos_io.s);
        Ok(InverterData {
            pmos_g: pmos.io().g,
            pmos,
        })
    }
}

#[derive(NestedData)]
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl Schematic for Buffer {
    type Schema = Schema;
    type NestedData = BufferData;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
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

#[derive(NestedData)]
pub struct BufferNData {
    pub bubbled_pmos_g: NestedTerminal,
    pub bubbled_inv1: NestedInstance<Inverter>,
    pub buffers: Vec<Instance<Buffer>>,
}

impl Schematic for BufferN {
    type Schema = Schema;
    type NestedData = BufferNData;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
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

#[derive(NestedData)]
pub struct BufferNxMData {
    pub bubbled_pmos_g: NestedTerminal,
    pub bubbled_inv1: NestedInstance<Inverter>,
    pub buffer_chains: Vec<Instance<BufferN>>,
}

impl Schematic for BufferNxM {
    type Schema = Schema;
    type NestedData = BufferNxMData;

    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
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

#[crate::test]
fn can_generate_vdivider_schematic() {
    let ctx = Context::new();
    let RawLib { scir, conv: _ } = ctx
        .export_scir(Vdivider::new(dec!(300), dec!(100)))
        .unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[crate::test]
fn can_generate_multi_top_scir() {
    let ctx = Context::new();
    let vdivider1 = Vdivider::new(dec!(300), dec!(100));
    let vdivider2 = Vdivider::new(dec!(500), dec!(600));
    let vdiv1 = ctx.generate_schematic(vdivider1);
    let vdiv2 = ctx.generate_schematic(vdivider2);
    let RawLib { scir, conv: _ } = ctx.export_scir_all(&[&vdiv1.raw(), &vdiv2.raw()]).unwrap();
    assert_eq!(scir.cells().count(), 2);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);

    let vdiv = scir.cell_named("vdivider_500_600");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[test]
fn can_generate_flattened_vdivider_schematic() {
    let ctx = Context::new();
    let vdivider = Vdivider::new(300, 100).flattened();
    let RawLib { scir, conv: _ } = ctx.export_scir(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[test]
fn can_generate_flattened_vdivider_array_schematic() {
    let ctx = Context::new();
    let vdiv1 = Vdivider::new(300, 100).flattened();
    let vdiv2 = Vdivider::new(600, 800).flattened();
    let vdiv3 = Vdivider::new(20, 20).flattened();
    let vdivs = vec![vdiv1, vdiv2, vdiv3];
    let vdivider = VdividerArray { vdividers: vdivs };
    let RawLib { scir, conv: _ } = ctx.export_scir(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_array_3");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 6);
    assert!(port_names.contains("elements_0_vdd"));
    assert!(port_names.contains("elements_0_vss"));
    assert!(port_names.contains("elements_1_vdd"));
    assert!(port_names.contains("elements_1_vss"));
    assert!(port_names.contains("elements_2_vdd"));
    assert!(port_names.contains("elements_2_vss"));
    assert_eq!(vdiv.ports().count(), 6);
    assert_eq!(vdiv.instances().count(), 6);
}

#[test]
fn nested_io_naming() {
    let io = VdividerIo {
        pwr: PowerIo {
            vdd: InOut(Signal),
            vss: InOut(Signal),
        },
        out: Output(Signal),
    };

    let actual = NameTree::new("io", io.kind().names().unwrap());
    let expected = NameTree::new(
        "io",
        vec![
            NameTree::new(
                "pwr",
                vec![NameTree::new("vdd", vec![]), NameTree::new("vss", vec![])],
            ),
            NameTree::new("out", vec![]),
        ],
    );
    assert_eq!(actual, expected);
}

#[crate::test]
fn connection_semantics_work() {
    let ctx = Context::new();
    // TODO: Run checks on internal signals.
    ctx.export_scir(NestedBlock(NestedBlock(SuperBlock)))
        .expect("failed to generate raw lib");
}

#[test]
fn internal_signal_names_preserved() {
    let ctx = Context::new();
    let RawLib { scir, conv: _ } = ctx.export_scir(Buffer::new(5)).unwrap();
    assert_eq!(scir.cells().count(), 2);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("buffer_5");
    let sigs: HashSet<ArcStr> = vdiv.signals().map(|p| p.1.name.clone()).collect();
    assert_eq!(sigs.len(), 5);
    assert!(sigs.contains("vdd"));
    assert!(sigs.contains("vss"));
    assert!(sigs.contains("din"));
    assert!(sigs.contains("dout"));
    assert!(sigs.contains("x"));
}

#[test]
fn nested_node_naming() {
    let ctx = Context::new();
    let handle = ctx.generate_schematic(BufferNxM::new(5, 5, 5));
    let cell = handle.cell();

    assert_eq!(
        cell.bubbled_inv1.pmos.io().g.path(),
        cell.bubbled_pmos_g.path()
    );

    assert_eq!(
        cell.bubbled_inv1.io().din.path(),
        cell.buffer_chains[0].bubbled_inv1.io().din.path()
    );
    assert_eq!(
        cell.bubbled_inv1.io().din.path(),
        cell.buffer_chains[0].buffers[0].inv1.io().din.path()
    );

    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].bubbled_pmos_g.path()
    );
    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].bubbled_inv1.pmos_g.path()
    );
    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].buffers[0].inv1.pmos_g.path()
    );
}

#[derive(Block, Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[substrate(io = "()")]
pub struct Block1;

impl Schematic for Block1 {
    type Schema = Schema;
    type NestedData = ();

    fn schematic(
        &self,
        _io: &IoNodeBundle<Self>,
        _cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        Err(substrate::error::Error::Anyhow(
            anyhow!("failed to generate block 1").into(),
        ))
    }
}

#[derive(Block, Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[substrate(io = "()")]
pub struct Block2;

impl Schematic for Block2 {
    type Schema = Schema;
    type NestedData = ();

    fn schematic(
        &self,
        _io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> crate::error::Result<Self::NestedData> {
        let handle = cell.generate(Block1);
        handle.try_cell()?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

#[test]
fn error_propagation_works() {
    let ctx = Context::new();
    let handle = ctx.generate_schematic(Block2);
    assert!(handle.try_cell().is_err());
}
