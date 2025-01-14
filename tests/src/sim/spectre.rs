// TODO: Move to examples folder

use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use approx::{assert_relative_eq, relative_eq};
use cache::multi::MultiCache;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use spectre::analysis::tran::Tran;
use spectre::blocks::Vsource;
use spectre::{Options, Primitive, Spectre};
use spice::{BlackboxContents, BlackboxElement, Spice};
use substrate::block::Block;
use substrate::cache::Cache;
use substrate::context::Context;
use substrate::execute::{ExecOpts, Executor, LocalExecutor};
use substrate::io::schematic::HardwareType;
use substrate::io::{InOut, Signal, TestbenchIo};
use substrate::io::{Io, TwoTerminalIo};
use substrate::pdk::corner::Pvt;
use substrate::schematic::{
    Cell, CellBuilder, ExportsNestedData, Instance, PrimitiveBinding, Schematic,
};
use substrate::simulation::data::{tran, FromSaved, Save, SaveTb};
use substrate::simulation::{SimController, SimulationContext, Simulator, Testbench};
use test_log::test;

use crate::paths::test_data;
use crate::shared::inverter::tb::InverterTb;
use crate::shared::inverter::Inverter;
use crate::shared::pdk::sky130_commercial_ctx;
use crate::shared::vdivider::tb::{VdividerArrayTb, VdividerDuplicateSubcktTb};
use crate::{paths::get_path, shared::vdivider::tb::VdividerTb};
use substrate::schematic::primitives::{RawInstance, Resistor};

#[test]
fn vdivider_tran() {
    let test_name = "spectre_vdivider_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerTb, sim_dir).unwrap();

    for (actual, expected) in [
        (&*output.current, 1.8 / 40.),
        (&*output.iprobe, 1.8 / 40.),
        (&*output.vdd, 1.8),
        (&*output.out, 0.9),
    ] {
        assert!(actual
            .iter()
            .cloned()
            .all(|val| relative_eq!(val, expected)));
    }
}

#[test]
fn vdivider_duplicate_subckt() {
    let test_name = "spectre_vdivider_duplicate_subckt";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerDuplicateSubcktTb, sim_dir).unwrap();

    // There are 2 subcircuits with the name `resistor`.
    // The first has a value of 100; the second has a value of 200.
    // We expect the second one to be used.
    let expected = 1.8 * 200.0 / (200.0 + 600.0);
    assert!(output.out.iter().all(|&val| relative_eq!(val, expected)));
}

#[test]
fn vdivider_array_tran() {
    let test_name = "spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx.simulate(VdividerArrayTb, sim_dir).unwrap();

    let cell = ctx.generate_schematic(crate::shared::vdivider::tb::VdividerArrayTb);

    for (expected, (out, out_nested)) in cell
        .cell()
        .iter()
        .map(|inst| {
            (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                .to_f64()
                .unwrap()
                * 1.8f64
        })
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(*val, expected)));
        assert!(out_nested.iter().all(|val| relative_eq!(*val, expected)));
    }

    assert!(output.vdd.iter().all(|val| relative_eq!(*val, 1.8)));
}

#[test]
fn flattened_vdivider_array_tran() {
    let test_name = "flattened_spectre_vdivider_array_tran";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_commercial_ctx();
    let output = ctx
        .simulate(
            crate::shared::vdivider::tb::FlattenedVdividerArrayTb,
            sim_dir,
        )
        .unwrap();

    let cell = ctx.generate_schematic(crate::shared::vdivider::tb::FlattenedVdividerArrayTb);

    for (expected, (out, out_nested)) in cell
        .cell()
        .iter()
        .map(|inst| {
            (inst.block().r2.value() / (inst.block().r1.value() + inst.block().r2.value()))
                .to_f64()
                .unwrap()
                * 1.8f64
        })
        .zip(output.out.iter().zip(output.out_nested.iter()))
    {
        assert!(out.iter().all(|val| relative_eq!(*val, expected)));
        assert!(out_nested.iter().all(|val| relative_eq!(*val, expected)));
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
    )
    .unwrap();
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

    let ctx = Context::builder()
        .install(Spectre::default())
        .cache(Cache::new(MultiCache::builder().build()))
        .executor(executor)
        .build();

    ctx.simulate(VdividerTb, &sim_dir).unwrap();
    ctx.simulate(VdividerTb, &sim_dir).unwrap();

    assert_eq!(*count.lock().unwrap(), 1);
}
