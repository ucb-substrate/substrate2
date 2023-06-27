use serde::{Deserialize, Serialize};

use geometry::{prelude::Bbox, rect::Rect};
use test_log::test;

use substrate::{block::Block, context::Context};

use crate::{
    paths::get_path,
    substrate::pdk::{ExamplePdkA, ExamplePdkB},
};

use self::schematic::{Resistor, Vdivider};

pub mod layout;
pub mod schematic;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    strength: usize,
}

impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Inverter {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }

    fn io(&self) -> Self::Io {}
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Buffer {
    strength: usize,
}

impl Buffer {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Buffer {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("buffer")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}", self.strength)
    }

    fn io(&self) -> Self::Io {}
}

#[test]
fn layout_generation_and_data_propagation_work() {
    let test_name = "layout_generation_and_data_propagation_work";

    let block = Buffer::new(5);

    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_layout(block);
    let cell = handle.wait().as_ref().unwrap();

    assert_eq!(cell.block, Buffer::new(5));
    assert_eq!(cell.data.inv1.cell().block, Inverter::new(5));
    assert_eq!(cell.data.inv2.cell().block, Inverter::new(5));

    assert_eq!(
        cell.data.inv1.bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data.inv2.bbox(),
        Some(Rect::from_sides(110, 0, 210, 200))
    );

    assert_eq!(cell.bbox(), Some(Rect::from_sides(0, 0, 210, 200)));

    ctx.write_layout(block, get_path(test_name, "layout_pdk_a.gds"))
        .expect("failed to write layout");

    let mut ctx = Context::new(ExamplePdkB);
    let handle = ctx.generate_layout(Buffer::new(5));
    let cell = handle.wait().as_ref().unwrap();

    assert_eq!(
        cell.data.inv1.bbox(),
        Some(Rect::from_sides(0, 0, 200, 100))
    );

    assert_eq!(
        cell.data.inv2.bbox(),
        Some(Rect::from_sides(210, 0, 410, 100))
    );

    assert_eq!(cell.bbox(), Some(Rect::from_sides(0, 0, 410, 100)));

    ctx.write_layout(block, get_path(test_name, "layout_pdk_b.gds"))
        .expect("failed to write layout");
}

#[test]
fn can_generate_vdivider_schematic() {
    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_schematic(Vdivider {
        r1: Resistor { r: 300 },
        r2: Resistor { r: 100 },
    });
    let _cell = handle.wait().as_ref().unwrap();
}
