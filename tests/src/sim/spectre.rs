use arcstr::ArcStr;
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::{context::Context, simulation::data::HasNodeData};
use test_log::test;

use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = Context::builder()
        .pdk(Sky130Pdk::new())
        .with_simulator(Spectre::default())
        .build();
    let output = ctx.simulate(VdividerTb, sim_dir);

    println!("{:?}", output.values);

    println!("{:?}", output.get_data("out"));
    println!(
        "{:?}",
        output.get_data(&scir::NodePath {
            signal: arcstr::literal!("out"),
            index: None,
            instances: Vec::new(),
            top: arcstr::literal!("vdivider"),
        })
    );
}
