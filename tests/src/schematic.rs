use std::collections::HashSet;

use arcstr::ArcStr;
use substrate::{
    context::Context,
    io::{HasNameTree, InOut, NameTree, Output, Signal},
};

use crate::shared::{
    buffer::Buffer,
    pdk::ExamplePdkA,
    vdivider::{PowerIo, Resistor, Vdivider, VdividerIo},
};
use crate::shared::buffer::BufferN;

#[test]
fn can_generate_vdivider_schematic() {
    let mut ctx = Context::new(ExamplePdkA);
    let vdivider = Vdivider {
        r1: Resistor { r: 300 },
        r2: Resistor { r: 100 },
    };
    let handle = ctx.generate_schematic(vdivider);
    let _cell = handle.wait().as_ref().unwrap();

    let lib = ctx.export_scir(vdivider);
    assert_eq!(lib.cells().count(), 3);
    let issues = lib.validate();
    println!("Library:\n{:#?}", lib);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = lib.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("io_pwr_vdd"));
    assert!(port_names.contains("io_pwr_vss"));
    assert!(port_names.contains("io_out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.primitives().count(), 0);
    assert_eq!(vdiv.instances().count(), 2);

    let res300 = lib.cell_named("resistor_300");
    assert_eq!(res300.ports().count(), 2);
    assert_eq!(res300.primitives().count(), 1);
    assert_eq!(res300.instances().count(), 0);

    let res100 = lib.cell_named("resistor_100");
    assert_eq!(res100.ports().count(), 2);
    assert_eq!(res100.primitives().count(), 1);
    assert_eq!(res100.instances().count(), 0);
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
    let lib = ctx.export_scir(Buffer::new(5));
    assert_eq!(lib.cells().count(), 4);
    let issues = lib.validate();
    println!("Library:\n{:#?}", lib);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = lib.cell_named("buffer_5");
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
    let handle = ctx.generate_schematic(BufferN::new(5, 5));
    let cell = handle.wait().as_ref().unwrap();

    println!("{:?}", cell.data.buffers[1].cell().data.inv1.cell().data.din.path());
    println!("{:?}", cell.data.bubbled_inv1.cell().data.din.path());
    println!("{:?}", cell.data.bubbled_din.path());
}
