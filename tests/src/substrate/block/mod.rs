use serde::{Deserialize, Serialize};

use geometry::{prelude::Bbox, rect::Rect};

use substrate::{block::Block, context::Context};

use crate::substrate::pdk::{ExamplePdkA, ExamplePdkB};

pub mod layout;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    strength: usize,
}

impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Inverter {
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Buffer {
    strength: usize,
}

impl Buffer {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

impl Block for Buffer {
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("buffer")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}", self.strength)
    }
}

#[test]
fn test_layout_generation_and_data_propagation() {
    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_layout(Buffer::new(5));
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
}
