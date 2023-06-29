use serde::{Deserialize, Serialize};

use geometry::{prelude::Bbox, rect::Rect};
use substrate::io::{
    CustomLayoutType, InOut, Input, LayoutPort, LayoutType, NameTree, Output, ShapePort, Signal,
};
use substrate::{Io, LayoutIo};
use test_log::test;

use substrate::{block::Block, context::Context};

use crate::{
    paths::get_path,
    substrate::pdk::{ExamplePdkA, ExamplePdkB},
};

use self::schematic::{Resistor, Vdivider};

pub mod layout;
pub mod schematic;

#[derive(Io, Clone, Default)]
pub struct BufferIo {
    #[io(layout_type = "ShapePort")]
    vdd: InOut<Signal>,
    #[io(layout_type = "ShapePort")]
    vss: InOut<Signal>,
    #[io(layout_type = "ShapePort")]
    din: Input<Signal>,
    #[io(layout_type = "ShapePort")]
    dout: Output<Signal>,
}

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
    type Io = BufferIo;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
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
    type Io = BufferIo;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("buffer")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Io, Clone, Default)]
#[io(layout_type = "CustomBufferNIoLayout")]
pub struct BufferNIo {
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
    din: Input<Signal>,
    dout: Output<Signal>,
}

#[derive(LayoutIo, Clone)]
pub struct CustomBufferNIoLayout {
    vdd: Signal,
    vss: Signal,
    din: ShapePort,
    dout: ShapePort,
}

impl CustomLayoutType<BufferNIo> for CustomBufferNIoLayout {
    fn builder(other: &BufferNIo) -> Self::Builder {
        Self {
            vdd: Signal,
            vss: Signal,
            din: ShapePort,
            dout: ShapePort,
        }
        .builder()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BufferN {
    strength: usize,
    n: usize,
}

impl BufferN {
    pub fn new(strength: usize, n: usize) -> Self {
        Self { strength, n }
    }
}

impl Block for BufferN {
    type Io = BufferNIo;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("buffer_n")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}_{}", self.strength, self.n)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[test]
fn layout_generation_and_data_propagation_work() {
    let test_name = "layout_generation_and_data_propagation_work";

    let block = Buffer::new(5);

    let mut ctx = Context::new(ExamplePdkA);
    let handle = ctx.generate_layout(block);
    let cell = handle.wait().as_ref().unwrap();

    assert_eq!(cell.block, Buffer::new(5));
    assert_eq!(cell.data.inv1.cell().block, &Inverter::new(5));
    assert_eq!(cell.data.inv2.cell().block, &Inverter::new(5));

    assert_eq!(
        cell.data.inv1.bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data.inv1.cell().bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data.inv2.bbox(),
        Some(Rect::from_sides(110, 0, 210, 200))
    );

    assert_eq!(
        cell.data.inv2.cell().bbox(),
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
fn nested_transform_views_work() {
    let test_name = "nested_transform_views_work";

    let block = BufferN::new(5, 10);

    let mut ctx = Context::new(ExamplePdkA);
    ctx.write_layout(block, get_path(test_name, "layout.gds"))
        .expect("failed to write layout");

    let handle = ctx.generate_layout(block);
    let cell = handle.wait().as_ref().unwrap();

    assert_eq!(
        cell.data.buffers[9].cell().data.inv2.bbox(),
        Some(Rect::from_sides(2090, 0, 2190, 200))
    );
}

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

    let vdiv = lib.cell_named("vdivider_resistor_300_resistor_100");
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
    use crate::substrate::block::schematic::{PowerIo, VdividerIo};
    use substrate::io::HasNameTree;

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
