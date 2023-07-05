use rust_decimal_macros::dec;
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130CommercialPdk;
use spectre::Spectre;
use substrate::context::Context;
use substrate::pdk::corner::Pvt;
use test_log::test;

use crate::shared::inverter::tb::InverterTb;
use crate::shared::vdivider::tb::VdividerArrayTb;
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new("/path/to/sky130pdk"))
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
        .pdk(Sky130CommercialPdk::new("/path/to/sky130pdk"))
        .with_simulator(Spectre::default())
        .build();
    let output = ctx.simulate(VdividerArrayTb, sim_dir);

    for out in output.out {
        println!("{:?}", out);
    }
}

#[test]
pub fn inv_tb() {
    let test_name = "inv_tb";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new("/path/to/sky130pdk"))
        .with_simulator(Spectre::default())
        .build();
    ctx.simulate(
        InverterTb::new(Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25))),
        sim_dir,
    );
}
