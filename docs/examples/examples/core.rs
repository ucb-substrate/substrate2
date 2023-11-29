#![allow(dead_code)]
use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use spice::Spice;
use substrate::block::Block;
use substrate::context::{ContextBuilder, Installation, PdkContext};
use substrate::geometry::prelude::*;
use substrate::io::{
    Array, CustomLayoutType, Flipped, InOut, Input, Io, IoShape, LayoutPort, LayoutType, Node,
    Output, PortGeometry, SchematicType, ShapePort, Signal,
};
use substrate::layout::{element::Shape, Cell, ExportsLayoutData, Instance, Layout, LayoutData};
use substrate::pdk::layers::{DerivedLayerFamily, DerivedLayers, LayerFamily, Layers};
use substrate::pdk::{Pdk, PdkLayers};
use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

// begin-code-snippet pdk
pub struct ExamplePdk;

impl Installation for ExamplePdk {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdk {
    type Layers = ExamplePdkLayers;
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

pub struct ExamplePdkA;

impl Installation for ExamplePdkA {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
}

pub struct ExamplePdkB;

impl Installation for ExamplePdkB {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
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
    type LayoutData = ();
}

impl Layout<ExamplePdk> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdk, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
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
    ) -> substrate::error::Result<Self::LayoutData> {
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
    ) -> substrate::error::Result<Self::LayoutData> {
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
    type LayoutData = BufferData;
}

mod single_process_buffer {
    use crate::{BufferData, BufferIo, ExamplePdk, Inverter};
    use serde::{Deserialize, Serialize};
    use substrate::block::Block;
    use substrate::geometry::align::{AlignBbox, AlignMode};
    use substrate::io::LayoutType;
    use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};

    #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Block)]
    #[substrate(io = "BufferIo")]
    pub struct Buffer {
        strength: usize,
    }

    // begin-code-snippet buffer_layout
    impl ExportsLayoutData for Buffer {
        type LayoutData = BufferData;
    }

    // begin-code-snippet cell_builder_generate
    impl Layout<ExamplePdk> for Buffer {
        fn layout(
            // begin-ellipses cell_builder_generate
            &self,
            io: &mut <<Self as Block>::Io as LayoutType>::Builder,
            cell: &mut CellBuilder<ExamplePdk, Self>,
            // end-ellipses cell_builder_generate
        ) -> substrate::error::Result<Self::LayoutData> {
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
impl<PDK: Pdk> Layout<PDK> for Buffer
where
    Inverter: Layout<PDK>,
{
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
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
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "BufferIo")]
pub struct BufferInlineHardMacro;

impl ExportsNestedData for BufferInlineHardMacro {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for BufferInlineHardMacro {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut scir = Spice::scir_cell_from_str(
            r#"
                * CMOS buffer

                .subckt buffer din dout vdd vss
                X0 din dinb vdd vss inverter
                X1 dinb dout vdd vss inverter
                .ends

                .subckt inverter din dout vdd vss
                X0 dout din vss vss sky130_fd_pr__nfet_01v8 w=2 l=0.15
                X1 dout din vdd vdd sky130_fd_pr__pfet_01v8 w=4 l=0.15
                .ends
            "#,
            "buffer",
        )
        .convert_schema::<Sky130Pdk>()?;

        scir.connect("din", io.din);
        scir.connect("dout", io.dout);
        scir.connect("vss", io.vss);
        scir.connect("vdd", io.vdd);

        cell.set_scir(scir);
        Ok(())
    }
}
// end-code-snippet buffer_hard_macro

// begin-code-snippet buffern_data
#[derive(Default, LayoutData)]
pub struct BufferNData {
    pub buffers: Vec<Instance<Buffer>>,
}
// end-code-snippet buffern_data

fn generate_layout() {
    // begin-code-snippet generate
    let ctx = PdkContext::new(ExamplePdk);
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
    use substrate::simulation::data::{tran, FromSaved};

    // begin-code-snippet sim_from_saved
    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub enum SavedEnum {
        Fields {
            vout: tran::Voltage,
            iout: tran::Current,
        },
        Tuple(tran::Voltage, tran::Current),
        Unit,
    }

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct NamedFields {
        vout: tran::Voltage,
        iout: tran::Current,
    }

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct NewType(NamedFields);

    #[derive(Debug, Clone, FromSaved)]
    #[allow(unused)]
    pub struct Tuple(NamedFields, SavedEnum);
    // end-code-snippet sim_from_saved
}

fn io() {
    // begin-code-snippet array-io
    #[derive(Io, Clone, Debug)]
    pub struct ArrayIo {
        pub in_bus: Input<Array<Signal>>,
        pub out_bus: Output<Array<Signal>>,
    }

    let io_type = ArrayIo {
        in_bus: Input(Array::new(5, Signal::new())),
        out_bus: Output(Array::new(5, Signal::new())),
    };
    // end-code-snippet array-io

    // begin-code-snippet array-io-constructor
    impl ArrayIo {
        pub fn new(in_size: usize, m: usize) -> Self {
            Self {
                in_bus: Input(Array::new(in_size, Signal::new())),
                out_bus: Output(Array::new(in_size * m, Signal::new())),
            }
        }
    }
    // end-code-snippet array-io-constructor

    // begin-code-snippet mos-io
    #[derive(Io, Clone, Default, Debug)]
    pub struct ThreePortMosIo {
        pub d: InOut<Signal>,
        pub g: Input<Signal>,
        pub s: InOut<Signal>,
    }

    #[derive(Io, Clone, Default, Debug)]
    pub struct FourPortMosIo {
        pub d: InOut<Signal>,
        pub g: Input<Signal>,
        pub s: InOut<Signal>,
        pub b: InOut<Signal>,
    }
    // end-code-snippet mos-io

    // begin-code-snippet mos-io-from
    impl From<ThreePortMosIoSchematic> for FourPortMosIoSchematic {
        fn from(value: ThreePortMosIoSchematic) -> Self {
            Self {
                d: value.d,
                g: value.g,
                s: value.s,
                b: value.s,
            }
        }
    }
    // end-code-snippet mos-io-from

    // begin-code-snippet mos-io-body
    impl ThreePortMosIoSchematic {
        fn with_body(&self, b: Node) -> FourPortMosIoSchematic {
            FourPortMosIoSchematic {
                d: self.d,
                g: self.g,
                s: self.s,
                b,
            }
        }
    }
    // end-code-snippet mos-io-body

    // begin-code-snippet sram-io
    #[derive(Io, Clone, Debug)]
    pub struct SramIo {
        pub clk: Input<Signal>,
        pub we: Input<Signal>,
        pub addr: Input<Array<Signal>>,
        pub din: Input<Array<Signal>>,
        pub dout: Output<Array<Signal>>,
    }

    pub type SramObserverIo = Input<SramIo>;
    // end-code-snippet sram-io

    // begin-code-snippet sram-driver-io
    pub type SramDriverIo = Flipped<SramIo>;
    // end-code-snippet sram-driver-io

    // begin-code-snippet sram-block
    #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Sram {
        num_words: usize,
        data_width: usize,
    }

    impl Block for Sram {
        type Io = SramIo;

        fn id() -> ArcStr {
            arcstr::literal!("sram")
        }

        fn name(&self) -> ArcStr {
            arcstr::format!("sram{}x{}", self.num_words, self.data_width)
        }

        fn io(&self) -> Self::Io {
            Self::Io {
                clk: Default::default(),
                we: Default::default(),
                addr: Input(Array::new(
                    (self.num_words - 1).ilog2() as usize + 1,
                    Signal::new(),
                )),
                din: Input(Array::new(self.data_width, Signal::new())),
                dout: Output(Array::new(self.data_width, Signal::new())),
            }
        }
    }
    // end-code-snippet sram-block

    let _ = io_type;
}

#[derive(Io, Clone, Default, Debug)]
pub struct VdividerIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub dout: Output<Signal>,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, Eq)]
#[substrate(io = "()")]
pub struct Vdivider {
    /// The top resistance.
    pub r1: Decimal,
    /// The bottom resistance.
    pub r2: Decimal,
}

// begin-code-snippet vdivider-bad-eq
impl PartialEq<Self> for Vdivider {
    fn eq(&self, other: &Self) -> bool {
        self.r1 == other.r1
    }
}
// end-code-snippet vdivider-bad-eq

pub mod nested_data {
    use serde::{Deserialize, Serialize};
    use substrate::block::Block;
    use substrate::io::Node;
    use substrate::schematic::{
        ExportsNestedData, HasNestedView, Instance, InstancePath, NestedData, NestedView,
    };

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "()")]
    pub struct Inverter;

    impl ExportsNestedData for Inverter {
        type NestedData = ();
    }

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "()")]
    pub struct Buffer;

    // begin-code-snippet buffer-nested-data
    #[derive(NestedData)]
    pub struct BufferData {
        inv1: Instance<Inverter>,
        inv2: Instance<Inverter>,
        x: Node,
    }

    impl ExportsNestedData for Buffer {
        type NestedData = BufferData;
    }
    // end-code-snippet buffer-nested-data

    // begin-code-snippet custom-nested-view
    #[derive(Clone, Copy)]
    pub struct MyMetadata {
        my_calculated_value: i64,
    }

    impl HasNestedView for MyMetadata {
        type NestedView = Self;

        fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView {
            *self
        }
    }

    #[derive(NestedData)]
    pub struct BufferDataWithMetadata {
        inv1: Instance<Inverter>,
        inv2: Instance<Inverter>,
        metadata: MyMetadata,
    }
    // end-code-snippet custom-nested-view

    // begin-code-snippet custom-nested-view-2
    pub struct BufferDataWithMetadataV2 {
        inv1: Instance<Inverter>,
        inv2: Instance<Inverter>,
        metadata: i64,
    }

    pub struct NestedBufferDataWithMetadataV2 {
        inv1: NestedView<Instance<Inverter>>,
        inv2: NestedView<Instance<Inverter>>,
        metadata: i64,
    }

    impl HasNestedView for BufferDataWithMetadataV2 {
        type NestedView = NestedBufferDataWithMetadataV2;

        fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
            Self::NestedView {
                inv1: self.inv1.nested_view(parent),
                inv2: self.inv2.nested_view(parent),
                metadata: self.metadata,
            }
        }
    }
    // end-code-snippet custom-nested-view-2
}

mod try_data {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use spice::Spice;
    use substrate::block::Block;
    use substrate::io::SchematicType;
    use substrate::schematic::primitives::Resistor;
    use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    impl ExportsNestedData for Vdivider {
        type NestedData = ();
    }

    // begin-code-snippet vdivider-try-data-error-handling
    impl Schematic<Spice> for Vdivider {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut CellBuilder<Spice>,
        ) -> substrate::error::Result<Self::NestedData> {
            let r1 = cell.instantiate(Resistor::new(self.r1));
            let r2 = cell.instantiate(Resistor::new(self.r2));
            r1.try_data()?;
            r2.try_data()?;

            cell.connect(io.vdd, r1.io().p);
            cell.connect(io.dout, r1.io().n);
            cell.connect(io.dout, r2.io().p);
            cell.connect(io.vss, r2.io().n);

            Ok(())
        }
    }
    // end-code-snippet vdivider-try-data-error-handling
}

mod instantiate_blocking {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use spice::Spice;
    use substrate::block::Block;
    use substrate::io::SchematicType;
    use substrate::schematic::primitives::Resistor;
    use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    impl ExportsNestedData for Vdivider {
        type NestedData = ();
    }

    // begin-code-snippet vdivider-instantiate-blocking-error-handling
    impl Schematic<Spice> for Vdivider {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut CellBuilder<Spice>,
        ) -> substrate::error::Result<Self::NestedData> {
            let r1 = cell.instantiate_blocking(Resistor::new(self.r1))?;
            let r2 = cell.instantiate_blocking(Resistor::new(self.r2))?;

            cell.connect(io.vdd, r1.io().p);
            cell.connect(io.dout, r1.io().n);
            cell.connect(io.dout, r2.io().p);
            cell.connect(io.vss, r2.io().n);

            Ok(())
        }
    }
    // end-code-snippet vdivider-instantiate-blocking-error-handling
}

mod instantiate_blocking_bad {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use spice::Spice;
    use substrate::block::Block;
    use substrate::io::SchematicType;
    use substrate::schematic::primitives::Resistor;
    use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    impl ExportsNestedData for Vdivider {
        type NestedData = ();
    }

    // begin-code-snippet vdivider-instantiate-blocking-bad
    impl Schematic<Spice> for Vdivider {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut CellBuilder<Spice>,
        ) -> substrate::error::Result<Self::NestedData> {
            if let Ok(r1) = cell.instantiate_blocking(Resistor::new(self.r1)) {
                cell.connect(io.vdd, r1.io().p);
                cell.connect(io.dout, r1.io().n);
            } else {
                cell.connect(io.vdd, io.dout);
            }
            let r2 = cell.instantiate_blocking(Resistor::new(self.r1))?;
            cell.connect(io.dout, r2.io().p);
            cell.connect(io.vss, r2.io().n);

            Ok(())
        }
    }
    // end-code-snippet vdivider-instantiate-blocking-bad
}

mod generate {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use spice::Spice;
    use substrate::block::Block;
    use substrate::io::SchematicType;
    use substrate::schematic::primitives::Resistor;
    use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    impl ExportsNestedData for Vdivider {
        type NestedData = ();
    }

    // begin-code-snippet vdivider-generate-add-error-handling
    impl Schematic<Spice> for Vdivider {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as SchematicType>::Bundle,
            cell: &mut CellBuilder<Spice>,
        ) -> substrate::error::Result<Self::NestedData> {
            let r1_cell = cell.generate(Resistor::new(self.r1));
            let r2 = cell.instantiate_blocking(Resistor::new(self.r2))?;

            // Block on generator to see if it succeeds.
            if r1_cell.try_cell().is_ok() {
                let r1 = cell.add(r1_cell);
                cell.connect(io.vdd, r1.io().p);
                cell.connect(io.dout, r1.io().n);
            } else {
                cell.connect(io.vdd, io.dout);
            }

            cell.connect(io.dout, r2.io().p);
            cell.connect(io.vss, r2.io().n);

            Ok(())
        }
    }
    // end-code-snippet vdivider-generate-add-error-handling
}

mod scir {
    use substrate::scir::schema::{Schema, StringSchema};
    use substrate::scir::{Cell, LibraryBuilder};

    fn library() {
        // begin-code-snippet scir-library-builder
        let mut lib = LibraryBuilder::<StringSchema>::new();
        // end-code-snippet scir-library-builder
        // begin-code-snippet scir-library-cell
        let empty_cell = Cell::new("empty");
        let empty_cell_id = lib.add_cell(empty_cell);
        // end-code-snippet scir-library-cell
        // begin-code-snippet scir-library-primitive
        let resistor_id = lib.add_primitive(arcstr::literal!("resistor"));
        // end-code-snippet scir-library-primitive
        let composite_cell = Cell::new("composite_cell");
    }
}
