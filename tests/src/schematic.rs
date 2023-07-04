use std::collections::HashSet;

use arcstr::ArcStr;
use substrate::{
    context::Context,
    io::{HasNameTree, InOut, NameTree, Output, Signal},
    schematic::conv::RawLib,
};

use crate::shared::buffer::BufferNxM;
use crate::shared::{
    buffer::Buffer,
    pdk::ExamplePdkA,
    vdivider::{PowerIo, Resistor, Vdivider, VdividerIo},
};

#[test]
fn can_generate_vdivider_schematic() {
    let mut ctx = Context::new(ExamplePdkA);
    let vdivider = Vdivider {
        r1: Resistor { r: 300 },
        r2: Resistor { r: 100 },
    };
    let handle = ctx.generate_schematic(vdivider);
    let _cell = handle.wait().as_ref().unwrap();

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
    assert!(port_names.contains("io_pwr_vdd"));
    assert!(port_names.contains("io_pwr_vss"));
    assert!(port_names.contains("io_out"));
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
    let mut ctx = Context::new(ExamplePdkA);
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
    assert!(sigs.contains("io_vdd"));
    assert!(sigs.contains("io_vss"));
    assert!(sigs.contains("io_din"));
    assert!(sigs.contains("io_dout"));
    assert!(sigs.contains("x"));
}

#[test]
fn nested_node_naming() {
    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_schematic(BufferNxM::new(5, 5, 5));
    let cell = handle.wait().as_ref().unwrap();

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
