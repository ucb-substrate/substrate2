use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::relative_eq;
use cache::multi::MultiCache;
use rust_decimal_macros::dec;
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130CommercialPdk;
use spectre::Spectre;
use substrate::cache::Cache;
use substrate::context::Context;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::pdk::corner::Pvt;
use test_log::test;

use crate::shared::inverter::tb::InverterTb;
use crate::shared::inverter::Inverter;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::VdividerArrayTb;
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};

#[test]
fn spectre_vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir);

    println!("{:?}", output.vdd);
    println!("{:?}", output.out);
}

#[test]
fn spectre_vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerArrayTb, sim_dir);

    for (expected, (out, out_nested)) in output
        .expected
        .iter()
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(val, expected)));
        assert_eq!(out, out_nested);
    }

    assert!(output.vdd.iter().all(|val| relative_eq!(*val, 1.8)));
}

#[test]
fn inv_tb() {
    let test_name = "inv_tb";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    ctx.simulate(
        InverterTb::new(
            Pvt::new(Sky130Corner::Tt, dec!(1.8), dec!(25)),
            Inverter {
                nw: 1_200,
                pw: 2_000,
                lch: 150,
            },
        ),
        sim_dir,
    );
}

#[test]
fn spectre_caches_simulations() {
    #[derive(Clone, Debug, Default)]
    struct CountExecutor {
        executor: LocalExecutor,
        count: Arc<Mutex<u64>>,
    }

    impl Executor for CountExecutor {
        fn execute(&self, command: Command, opts: ExecOpts) -> Result<(), substrate::error::Error> {
            *self.count.lock().unwrap() += 1;
            self.executor.execute(command, opts)
        }
    }

    let test_name = "spectre_caches_simulations";
    let sim_dir = get_path(test_name, "sim/");
    let executor = CountExecutor::default();
    let count = executor.count.clone();

    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    let ctx = Context::builder()
        .pdk(Sky130CommercialPdk::new(pdk_root))
        .with_simulator(Spectre::default())
        .cache(Cache::new(MultiCache::builder().build()))
        .executor(executor)
        .build();

    ctx.simulate(VdividerTb, &sim_dir);
    ctx.simulate(VdividerTb, &sim_dir);

    assert_eq!(*count.lock().unwrap(), 1);
}
