use std::collections::HashSet;

use anyhow::anyhow;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::type_dispatch::impl_dispatch;
use substrate::{
    block::Block,
    context::Context,
    io::{HasNameTree, InOut, NameTree, Output, Signal},
    schematic::{conv::RawLib, HasSchematic, HasSchematicData},
};

use crate::shared::{
    buffer::Buffer,
    pdk::ExamplePdkA,
    vdivider::{PowerIo, Resistor, Vdivider, VdividerIo},
};
use crate::shared::{buffer::BufferNxM, pdk::ExamplePdkB};

#[test]
fn can_generate_vdivider_schematic() {
    let ctx = Context::new(ExamplePdkA);
    let vdivider = Vdivider {
        r1: Resistor::new(300),
        r2: Resistor::new(100),
    };
    let RawLib { scir, conv: _ } = ctx.export_scir(vdivider);
    assert_eq!(scir.cells().count(), 3);
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
    let contents = vdiv.contents().as_ref().unwrap_clear();
    assert_eq!(contents.primitives().count(), 0);
    assert_eq!(contents.instances().count(), 2);

    let res300 = scir.cell_named("resistor_300");
    let contents = res300.contents().as_ref().unwrap_clear();
    assert_eq!(res300.ports().count(), 2);
    assert_eq!(contents.primitives().count(), 1);
    assert_eq!(contents.instances().count(), 0);

    let res100 = scir.cell_named("resistor_100");
    let contents = res100.contents().as_ref().unwrap_clear();
    assert_eq!(res100.ports().count(), 2);
    assert_eq!(contents.primitives().count(), 1);
    assert_eq!(contents.instances().count(), 0);
}

#[test]
fn can_generate_flattened_vdivider_schematic() {
    let ctx = Context::new(ExamplePdkA);
    let vdivider = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let RawLib { scir, conv: _ } = ctx.export_scir(vdivider);
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
    let contents = vdiv.contents().as_ref().unwrap_clear();
    assert_eq!(contents.primitives().count(), 2);
    assert_eq!(contents.instances().count(), 0);
}

#[test]
fn can_generate_flattened_vdivider_array_schematic() {
    let ctx = Context::new(ExamplePdkA);
    let vdiv1 = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let vdiv2 = crate::shared::vdivider::flattened::Vdivider::new(600, 800);
    let vdiv3 = crate::shared::vdivider::flattened::Vdivider::new(20, 20);
    let vdivs = vec![vdiv1, vdiv2, vdiv3];
    let vdivider = crate::shared::vdivider::flattened::VdividerArray { vdividers: vdivs };
    let RawLib { scir, conv: _ } = ctx.export_scir(vdivider);
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
    let contents = vdiv.contents().as_ref().unwrap_clear();
    assert_eq!(contents.primitives().count(), 6);
    assert_eq!(contents.instances().count(), 0);
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
    let ctx = Context::new(ExamplePdkA);
    let RawLib { scir, conv: _ } = ctx.export_scir(Buffer::new(5));
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
    let ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_schematic(BufferNxM::new(5, 5, 5));
    let cell = handle.cell();

    assert_ne!(
        cell.data().bubbled_inv1.io().din.path(),
        cell.data().bubbled_din.path()
    );

    assert_eq!(
        cell.data().bubbled_inv1.io().din.path(),
        cell.data().buffer_chains[0]
            .data()
            .bubbled_inv1
            .io()
            .din
            .path()
    );
    assert_eq!(
        cell.data().bubbled_inv1.io().din.path(),
        cell.data().buffer_chains[0].data().buffers[0]
            .data()
            .inv1
            .io()
            .din
            .path()
    );

    assert_eq!(
        cell.data().bubbled_din.path(),
        cell.data().buffer_chains[0].data().bubbled_din.path()
    );
    assert_eq!(
        cell.data().bubbled_din.path(),
        cell.data().buffer_chains[0]
            .data()
            .bubbled_inv1
            .data()
            .din
            .path()
    );
    assert_eq!(
        cell.data().bubbled_din.path(),
        cell.data().buffer_chains[0].data().buffers[0]
            .data()
            .inv1
            .data()
            .din
            .path()
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

impl HasSchematicData for Block1 {
    type Data = ();
}

#[impl_dispatch({ExamplePdkA; ExamplePdkB})]
impl<PDK> HasSchematic<PDK> for Block1 {
    fn schematic(
        &self,
        _io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        _cell: &mut substrate::schematic::CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
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

impl HasSchematicData for Block2 {
    type Data = ();
}

impl HasSchematic<ExamplePdkA> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let handle = cell.generate(Block1);
        handle.try_cell()?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

impl HasSchematic<ExamplePdkB> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkB, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let handle = cell.generate_blocking(Block1)?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

#[test]
fn error_propagation_works() {
    let ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_schematic(Block2);
    assert!(handle.try_cell().is_err());

    let ctx = Context::new(ExamplePdkB);
    let handle = ctx.generate_schematic(Block2);
    assert!(handle.try_cell().is_err());
}
