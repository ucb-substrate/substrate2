use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::context::Context;
use test_log::test;

use crate::shared::vdivider::tb::VdividerArrayTb;
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = Context::builder()
        .pdk(Sky130Pdk::new("/path/to/sky130pdk"))
        .with_simulator(Spectre::default())
        .build();
    let output = ctx.simulate(VdividerTb, sim_dir);

    println!("{:?}", output.vdd);
    println!("{:?}", output.out);
}

#[test]
fn spectre_vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = Context::builder()
        .pdk(Sky130Pdk::new("/path/to/sky130pdk"))
        .with_simulator(Spectre::default())
        .build();
    let output = ctx.simulate(VdividerArrayTb, sim_dir);

    for out in output.out {
        println!("{:?}", out);
    }
}
