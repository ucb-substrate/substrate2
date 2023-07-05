use approx::relative_eq;
use rust_decimal_macros::dec;
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::context::Context;
use substrate::pdk::corner::Pvt;
use test_log::test;

use crate::shared::inverter::tb::InverterTb;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::VdividerArrayTb;
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir);

    println!("{:?}", output.vdd);
    println!("{:?}", output.out);
}

#[test]
fn spectre_vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerArrayTb, sim_dir);

    for (expected, (out, out_nested)) in output
        .expected
        .iter()
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(val, expected)));
        assert_eq!(out, out_nested);
    }

    assert!(output.vdd.iter().all(|val| *val > 1.7));
}

#[test]
pub fn inv_tb() {
    let test_name = "inv_tb";
    let sim_dir = get_path(test_name, "sim/");
    let mut ctx = sky130_commercial_ctx();
    ctx.simulate(
        InverterTb::new(Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25))),
        sim_dir,
    );
}
