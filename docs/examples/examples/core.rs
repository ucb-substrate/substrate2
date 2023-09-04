use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use substrate::block::{self, Block};
use substrate::context::Context;
use substrate::geometry::prelude::*;
use substrate::io::{
    CustomLayoutType, InOut, Input, Io, IoShape, LayoutPort, LayoutType, Node, Output,
    PortGeometry, ShapePort, Signal,
};
use substrate::layout::{element::Shape, Cell, ExportsLayoutData, Instance, Layout, LayoutData};
use substrate::pdk::layers::{DerivedLayerFamily, DerivedLayers, LayerFamily, Layers};
use substrate::pdk::{HasLayout, Pdk, PdkLayers};
use substrate::schematic::Schematic;

// begin-code-snippet pdk
pub struct ExamplePdk;

impl Pdk for ExamplePdk {
    type Primitive = ExamplePrimitive;
    type Layers = ExamplePdkLayers;
    type Corner = ExamplePdkCorner;
}
// end-code-snippet pdk

#[derive(Clone)]
pub enum ExamplePrimitive {}

// begin-code-snippet layers
#[derive(Layers)]
pub struct ExamplePdkLayers {
    #[layer(gds = "66/20")]
    pub poly: Poly,
    #[layer_family]
    pub met1: Met1,
    #[layer(name = "met2", gds = "69/20")]
    pub met2: Met2,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1 {
    #[layer(gds = "68/20", primary)]
    pub met1_drawing: Met1Drawing,
    #[layer(gds = "68/16", pin)]
    pub met1_pin: Met1Pin,
    #[layer(gds = "68/5", label)]
    pub met1_label: Met1Label,
}
// end-code-snippet layers

// begin-code-snippet derive_corner
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExamplePdkCorner {
    Tt,
    Ss,
    Ff,
}
// end-code-snippet derive_corner

#[derive(Clone)]
pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Primitive = ExamplePrimitive;
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
}

#[derive(Clone)]
pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Primitive = ExamplePrimitive;
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExampleCorner;

#[derive(Layers)]
pub struct ExamplePdkALayers {
    #[layer(gds = "66/20")]
    pub polya: PolyA,
    #[layer_family]
    pub met1a: Met1A,
    #[layer(name = "met2", gds = "69/20")]
    pub met2a: Met2A,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1A {
    #[layer(gds = "68/20", primary)]
    pub drawing: Met1ADrawing,
    #[layer(gds = "68/16", pin)]
    pub pin: Met1APin,
    #[layer(gds = "68/5", label)]
    pub label: Met1ALabel,
}

#[derive(Layers)]
pub struct ExamplePdkBLayers {
    #[layer(gds = "66/20")]
    pub polyb: PolyB,
    #[layer_family]
    pub met1b: Met1B,
    #[layer(name = "met2", gds = "69/20")]
    pub met2b: Met2B,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1B {
    #[layer(gds = "68/20", primary)]
    pub drawing: Met1BDrawing,
    #[layer(gds = "68/16", pin)]
    pub pin: Met1BPin,
    #[layer(gds = "68/5", label)]
    pub label: Met1BLabel,
}

#[allow(dead_code)]
// begin-code-snippet derived_layers
#[derive(DerivedLayers)]
pub struct DerivedLayers {
    #[layer_family]
    m1: M1,
    m2: M2,
}

#[derive(DerivedLayerFamily, Clone, Copy)]
pub struct M1 {
    #[layer(primary)]
    pub drawing: M1Drawing,
    #[layer(pin)]
    pub pin: M1Pin,
    #[layer(label)]
    pub label: M1Label,
}

impl From<&PdkLayers<ExamplePdkA>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkA>) -> Self {
        Self {
            m1: M1 {
                drawing: M1Drawing::new(value.met1a.drawing),
                pin: M1Pin::new(value.met1a.pin),
                label: M1Label::new(value.met1a.label),
            },
            m2: M2::new(value.met2a),
        }
    }
}

impl From<&PdkLayers<ExamplePdkB>> for DerivedLayers {
    fn from(value: &PdkLayers<ExamplePdkB>) -> Self {
        Self {
            m1: M1 {
                drawing: M1Drawing::new(value.met1b.drawing),
                pin: M1Pin::new(value.met1b.pin),
                label: M1Label::new(value.met1b.label),
            },
            m2: M2::new(value.met2b),
        }
    }
}
// end-code-snippet derived_layers

// begin-code-snippet inverter
#[derive(Io, Clone, Default, Debug)]
pub struct InverterIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    strength: usize,
}

// begin-hidden-code
impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}

// end-hidden-code
impl Block for Inverter {
    type Io = InverterIo;
    type Kind = block::Cell;

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
// end-code-snippet inverter

// begin-code-snippet inverter_layout
impl ExportsLayoutData for Inverter {
    type Data = ();
}

impl Layout<ExamplePdk> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        io.vss.push(IoShape::with_layers(
            cell.ctx.layers.met1,
            Rect::from_sides(25, 0, 75, 25),
        ));
        io.vdd.push(IoShape::with_layers(
            cell.ctx.layers.met1,
            Rect::from_sides(25, 175, 75, 200),
        ));
        io.din.push(IoShape::with_layers(
            cell.ctx.layers.met1,
            Rect::from_sides(0, 50, 25, 150),
        ));
        io.dout.push(IoShape::with_layers(
            cell.ctx.layers.met1,
            Rect::from_sides(75, 50, 100, 150),
        ));
        cell.draw(Shape::new(
            cell.ctx.layers.met1,
            Rect::from_sides(0, 0, 100, 200),
        ))?;
        Ok(())
    }
}
// end-code-snippet inverter_layout

// begin-code-snippet inverter_multiprocess
impl Layout<ExamplePdkA> for Inverter {
    // begin-ellipses inverter_multiprocess
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        io.vss.push(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(25, 0, 75, 25),
        ));
        io.vdd.push(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(25, 175, 75, 200),
        ));
        io.din.push(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(0, 50, 25, 150),
        ));
        io.dout.push(IoShape::with_layers(
            cell.ctx.layers.met1a,
            Rect::from_sides(75, 50, 100, 150),
        ));
        cell.draw(Shape::new(
            cell.ctx.layers.met1a,
            Rect::from_sides(0, 0, 100, 200),
        ))?;
        Ok(())
    }
    // end-ellipses inverter_multiprocess
}

impl Layout<ExamplePdkB> for Inverter {
    // begin-ellipses inverter_multiprocess
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdkB, Self>,
    ) -> substrate::error::Result<Self::Data> {
        io.vss.push(IoShape::with_layers(
            cell.ctx.layers.met1b,
            Rect::from_sides(50, 0, 150, 25),
        ));
        io.vdd.push(IoShape::with_layers(
            cell.ctx.layers.met1b,
            Rect::from_sides(50, 75, 150, 100),
        ));
        io.din.push(IoShape::with_layers(
            cell.ctx.layers.met1b,
            Rect::from_sides(0, 25, 25, 75),
        ));
        io.dout.push(IoShape::with_layers(
            cell.ctx.layers.met1b,
            Rect::from_sides(175, 25, 200, 75),
        ));
        cell.draw(Shape::new(
            cell.ctx.layers.met1b,
            Rect::from_sides(0, 0, 200, 100),
        ))?;
        Ok(())
    }
    // end-ellipses inverter_multiprocess
}
// end-code-snippet inverter_multiprocess

// begin-code-snippet buffer_io_simple
#[derive(Io, Clone, Default)]
pub struct BufferIo {
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
    din: Input<Signal>,
    dout: Output<Signal>,
}
// end-code-snippet buffer_io_simple

#[allow(dead_code)]
mod __buffer_io_autogenerated {
    use super::*;

    // begin-code-snippet buffer_io_autogenerated
    // Autogenerated by `#[derive(Io)]`.
    pub struct BufferIoSchematic {
        vdd: InOut<Node>,
        vss: InOut<Node>,
        din: Input<Node>,
        dout: Output<Node>,
    }

    pub struct BufferIoLayout {
        vdd: PortGeometry,
        vss: PortGeometry,
        din: PortGeometry,
        dout: PortGeometry,
    }
    // end-code-snippet buffer_io_autogenerated
}

mod __buffer_io_signal_override {
    use super::*;

    // begin-code-snippet buffer_io
    #[derive(Io, Clone, Default)]
    pub struct BufferIo {
        vdd: InOut<Signal>,
        vss: InOut<Signal>,
        #[substrate(layout_type = "ShapePort")]
        din: Input<Signal>,
        #[substrate(layout_type = "ShapePort")]
        dout: Output<Signal>,
    }
    // end-code-snippet buffer_io
}

mod __buffer_io_custom_layout {
    use super::*;
    use substrate::io::Io;

    // begin-code-snippet buffer_io_custom_layout
    #[derive(Io, Clone, Default)]
    #[substrate(layout_type = "BufferIoLayout")]
    pub struct BufferIo {
        vdd: InOut<Signal>,
        vss: InOut<Signal>,
        din: Input<Signal>,
        dout: Output<Signal>,
    }

    #[derive(LayoutType, Clone)]
    pub struct BufferIoLayout {
        vdd: LayoutPort,
        vss: LayoutPort,
        din: ShapePort,
        dout: ShapePort,
    }

    impl CustomLayoutType<BufferIo> for BufferIoLayout {
        fn from_layout_type(_other: &BufferIo) -> Self {
            Self {
                vdd: LayoutPort,
                vss: LayoutPort,
                din: ShapePort,
                dout: ShapePort,
            }
        }
    }
    // end-code-snippet buffer_io_custom_layout
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
    type Io = BufferIo;
    type Kind = block::Cell;

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

#[derive(LayoutData)]
pub struct BufferData {
    pub inv1: Instance<Inverter>,
    pub inv2: Instance<Inverter>,
}

impl ExportsLayoutData for Buffer {
    type Data = BufferData;
}

mod single_process_buffer {
    use crate::{BufferData, BufferIo, ExamplePdk, Inverter};
    use serde::{Deserialize, Serialize};
    use substrate::block::{Block, Cell};
    use substrate::geometry::align::{AlignBbox, AlignMode};
    use substrate::io::LayoutType;
    use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};

    #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Block)]
    #[substrate(io = "BufferIo", kind = "Cell")]
    pub struct Buffer {
        strength: usize,
    }

    // begin-code-snippet buffer_layout
    impl ExportsLayoutData for Buffer {
        type Data = BufferData;
    }

    // begin-code-snippet cell_builder_generate
    impl Layout<ExamplePdk> for Buffer {
        fn layout(
            // begin-ellipses cell_builder_generate
            &self,
            io: &mut <<Self as Block>::Io as LayoutType>::Builder,
            cell: &mut CellBuilder<ExamplePdk, Self>,
            // end-ellipses cell_builder_generate
        ) -> substrate::error::Result<Self::Data> {
            let inv1 = cell.generate(Inverter::new(self.strength));
            let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

            cell.draw(inv1.clone())?;
            cell.draw(inv2.clone())?;

            io.vdd.merge(inv1.io().vdd);
            io.vdd.merge(inv2.io().vdd);
            io.vss.merge(inv1.io().vss);
            io.vss.merge(inv2.io().vss);
            io.din.merge(inv1.io().din);
            io.dout.merge(inv2.io().dout);

            Ok(BufferData { inv1, inv2 })
        }
    }
    // end-code-snippet cell_builder_generate
    // end-code-snippet buffer_layout
}

// begin-code-snippet buffer_multiprocess
pub trait BufferSupportedPdk: Pdk + HasLayout<Inverter> {}
impl<PDK: Pdk + HasLayout<Inverter>> BufferSupportedPdk for PDK {}

impl<PDK: BufferSupportedPdk> Layout<PDK> for Buffer {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let inv1 = cell.generate(Inverter::new(self.strength));
        let inv2 = inv1.clone().align_bbox(AlignMode::ToTheRight, &inv1, 10);

        cell.draw(inv1.clone())?;
        cell.draw(inv2.clone())?;

        io.vdd.merge(inv1.io().vdd);
        io.vdd.merge(inv2.io().vdd);
        io.vss.merge(inv1.io().vss);
        io.vss.merge(inv2.io().vss);
        io.din.merge(inv1.io().din);
        io.dout.merge(inv2.io().dout);

        Ok(BufferData { inv1, inv2 })
    }
}
// end-code-snippet buffer_multiprocess

// begin-code-snippet buffer_hard_macro
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Schematic)]
#[substrate(io = "BufferIo", kind = "PdkScir")]
#[substrate(schematic(
    source = "r###\"
        * CMOS buffer

        .subckt buffer din dout vdd vss
        X0 din dinb vdd vss inverter
        X1 dinb dout vdd vss inverter
        .ends

        .subckt inverter din dout vdd vss
        X0 dout din vss vss sky130_fd_pr__nfet_01v8 w=2 l=0.15
        X1 dout din vdd vdd sky130_fd_pr__pfet_01v8 w=4 l=0.15
        .ends
    \"###",
    name = "buffer",
    fmt = "inline-spice",
    pdk = "Sky130Pdk"
))]
pub struct BufferInlineHardMacro;
// end-code-snippet buffer_hard_macro

// begin-code-snippet buffern_data
#[derive(Default, LayoutData)]
pub struct BufferNData {
    pub buffers: Vec<Instance<Buffer>>,
}
// end-code-snippet buffern_data

fn generate_layout() {
    // begin-code-snippet generate
    let ctx = Context::new(ExamplePdk);
    let handle = ctx.generate_layout(Buffer::new(5));
    let cell: &Cell<Buffer> = handle.cell();

    assert_eq!(cell.block(), &Buffer::new(5));
    assert_eq!(cell.data().inv1.block(), &Inverter::new(5));
    assert_eq!(cell.data().inv2.block(), &Inverter::new(5));

    assert_eq!(
        cell.data().inv1.bbox(),
        Some(Rect::from_sides(0, 0, 100, 200))
    );

    assert_eq!(
        cell.data().inv2.bbox(),
        Some(Rect::from_sides(110, 0, 210, 200))
    );

    assert_eq!(cell.bbox(), Some(Rect::from_sides(0, 0, 210, 200)));
    // end-code-snippet generate
}

fn main() {
    generate_layout();
}

/// Demonstrates how to save simulator output.
mod sim {
    use spectre::tran::{TranCurrent, TranVoltage};
    use substrate::simulation::data::FromSaved;

    // begin-code-snippet sim_from_saved
    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub enum SavedEnum {
        Fields {
            vout: TranVoltage,
            iout: TranCurrent,
        },
        Tuple(TranVoltage, TranCurrent),
        Unit,
    }

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct NamedFields {
        vout: TranVoltage,
        iout: TranCurrent,
    }

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct NewType(NamedFields);

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct Tuple(NamedFields, SavedEnum);
    // end-code-snippet sim_from_saved
}
