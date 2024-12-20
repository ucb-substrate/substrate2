use std::collections::HashSet;

use anyhow::anyhow;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use spice::Spice;
use substrate::context::Context;
use substrate::io::schematic::HardwareType;
use substrate::io::PowerIo;
use substrate::schematic::primitives::Resistor;
use substrate::schematic::CellBuilder;
use substrate::type_dispatch::impl_dispatch;
use substrate::{
    block::Block,
    context::PdkContext,
    io::{HasNameTree, InOut, NameTree, Output, Signal},
    schematic::{conv::RawLib, ExportsNestedData, Schematic},
};

use crate::shared::{
    buffer::Buffer,
    pdk::ExamplePdkA,
    vdivider::{Vdivider, VdividerIo},
};
use crate::shared::{buffer::BufferNxM, pdk::ExamplePdkB};

#[test]
fn can_generate_vdivider_schematic() {
    let ctx = Context::new();
    let vdivider = Vdivider {
        r1: Resistor::new(300),
        r2: Resistor::new(100),
    };
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
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
fn can_generate_multi_top_scir() {
    let ctx = Context::new();
    let vdivider1 = Vdivider {
        r1: Resistor::new(300),
        r2: Resistor::new(100),
    };
    let vdivider2 = Vdivider {
        r1: Resistor::new(500),
        r2: Resistor::new(600),
    };
    let vdiv1 = ctx.generate_schematic::<Spice, _>(vdivider1);
    let vdiv2 = ctx.generate_schematic::<Spice, _>(vdivider2);
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
    let vdivider = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider");
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
    let vdiv1 = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let vdiv2 = crate::shared::vdivider::flattened::Vdivider::new(600, 800);
    let vdiv3 = crate::shared::vdivider::flattened::Vdivider::new(20, 20);
    let vdivs = vec![vdiv1, vdiv2, vdiv3];
    let vdivider = crate::shared::vdivider::flattened::VdividerArray { vdividers: vdivs };
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("flattened_vdivider_array_3");
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

    let actual = NameTree::new("io", io.names().unwrap());
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

#[test]
fn internal_signal_names_preserved() {
    let ctx = PdkContext::new(ExamplePdkA);
    let RawLib { scir, conv: _ } = ctx.export_scir::<ExamplePdkA, _>(Buffer::new(5)).unwrap();
    assert_eq!(scir.cells().count(), 4);
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
    let ctx = PdkContext::new(ExamplePdkA);
    let handle = ctx.generate_schematic::<ExamplePdkA, _>(BufferNxM::new(5, 5, 5));
    let cell = handle.cell();

    assert_eq!(
        cell.bubbled_inv1.pmos.as_ref().unwrap().io().g.path(),
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

    assert_ne!(
        cell.bubbled_inv1.pmos.as_ref().unwrap().data().path(),
        cell.bubbled_inv1.pmos.as_ref().unwrap().path()
    );
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Block1;

impl Block for Block1 {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("block1")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("block1")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Block1 {
    type NestedData = ();
}

#[impl_dispatch({ExamplePdkA; ExamplePdkB})]
impl<PDK> Schematic<PDK> for Block1 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        _cell: &mut CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        Err(substrate::error::Error::Anyhow(
            anyhow!("failed to generate block 1").into(),
        ))
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Block2;

impl Block for Block2 {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("block2")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("block2")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Block2 {
    type NestedData = ();
}

impl Schematic<ExamplePdkA> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let handle = cell.generate(Block1);
        handle.try_cell()?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

impl Schematic<ExamplePdkB> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::NestedData> {
        let handle = cell.generate_blocking(Block1)?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

#[test]
fn error_propagation_works() {
    let ctx = PdkContext::new(ExamplePdkA);
    let handle = ctx.generate_schematic::<ExamplePdkA, _>(Block2);
    assert!(handle.try_cell().is_err());

    let ctx = PdkContext::new(ExamplePdkB);
    let handle = ctx.generate_schematic::<ExamplePdkB, _>(Block2);
    assert!(handle.try_cell().is_err());
}
