#![allow(dead_code)]
use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::schematic::{
    CellBuilder, HasNestedView, InstancePath, NestedData, NestedView, Schematic,
};
use substrate::types::schematic::Node;
use substrate::types::{Array, Flipped, InOut, Input, Io, Output, Signal};

#[derive(Clone)]
pub enum ExamplePrimitive {}

// begin-code-snippet derive_corner
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExamplePdkCorner {
    Tt,
    Ss,
    Ff,
}
// end-code-snippet derive_corner

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

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}
// end-code-snippet inverter

// begin-code-snippet buffer_io_simple
#[derive(Io, Clone, Copy, Default)]
pub struct BufferIo {
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
    din: Input<Signal>,
    dout: Output<Signal>,
}
// end-code-snippet buffer_io_simple

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

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("buffer_{}", self.strength)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
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

    // TODO: replace with data view API
    // begin-code-snippet mos-io-from
    // impl<T: BundlePrimitive> From<ThreePortMosIoBundle<T>> for FourPortMosIoBundle<T> {
    //     fn from(value: ThreePortMosIoBundle<T>) -> Self {
    //         Self {
    //             d: value.d,
    //             g: value.g,
    //             s: value.s.clone(),
    //             b: value.s,
    //         }
    //     }
    // }
    // end-code-snippet mos-io-from

    // begin-code-snippet mos-io-body
    //impl<T: BundlePrimitive> ThreePortMosIoBundle<T> {
    //    fn with_body(&self, b: T) -> FourPortMosIoBundle<T> {
    //        FourPortMosIoBundle {
    //            d: self.d.clone(),
    //            g: self.g.clone(),
    //            s: self.s.clone(),
    //            b,
    //        }
    //    }
    //}
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
    use super::*;
    use ::scir::schema::StringSchema;
    use substrate::schematic::Instance;
    use substrate::types::codegen::{FromSelf, ViewSource};
    use substrate::types::schematic::IoNodeBundle;

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "()")]
    pub struct Inverter;

    impl Schematic for Inverter {
        type Schema = StringSchema;
        type NestedData = ();
        fn schematic(
            &self,
            _io: &IoNodeBundle<Self>,
            _cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            Ok(())
        }
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
    // end-code-snippet buffer-nested-data

    // begin-code-snippet custom-nested-view
    #[derive(Clone, Copy)]
    pub struct MyMetadata {
        my_calculated_value: i64,
    }

    impl ViewSource for MyMetadata {
        type Kind = FromSelf;
        type Source = Self;
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

    impl HasNestedView for NestedBufferDataWithMetadataV2 {
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

#[derive(Io, Clone, Default, Debug)]
pub struct ResistorIo {
    pub p: InOut<Signal>,
    pub n: InOut<Signal>,
}

#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "ResistorIo")]
pub struct Resistor(Decimal);

impl Resistor {
    pub fn new(val: impl Into<Decimal>) -> Self {
        Self(val.into())
    }
}

mod try_data {
    use ::scir::schema::StringSchema;
    use substrate::types::schematic::IoNodeBundle;

    use super::*;

    impl Schematic for Resistor {
        type Schema = StringSchema;
        type NestedData = ();

        fn schematic(
            &self,
            _io: &IoNodeBundle<Resistor>,
            _cell: &mut super::CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    // begin-code-snippet vdivider-try-data-error-handling
    impl Schematic for Vdivider {
        type Schema = StringSchema;
        type NestedData = ();

        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
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
    use ::scir::schema::StringSchema;
    use substrate::types::schematic::IoNodeBundle;

    use super::*;

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    // begin-code-snippet vdivider-instantiate-blocking-error-handling
    impl Schematic for Vdivider {
        type Schema = StringSchema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
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
    use ::scir::schema::StringSchema;

    use super::*;

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    // begin-code-snippet vdivider-instantiate-blocking-bad
    impl Schematic for Vdivider {
        type Schema = StringSchema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &substrate::types::schematic::IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
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
    use ::scir::schema::StringSchema;
    use substrate::types::schematic::IoNodeBundle;

    use super::*;

    #[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "super::VdividerIo")]
    pub struct Vdivider {
        /// The top resistance.
        pub r1: Decimal,
        /// The bottom resistance.
        pub r2: Decimal,
    }

    // begin-code-snippet vdivider-generate-add-error-handling
    impl Schematic for Vdivider {
        type Schema = StringSchema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
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
    use scir::schema::{Schema, StringSchema};
    use scir::{Cell, Direction, Instance, LibraryBuilder};
    use serde::{Deserialize, Serialize};
    use substrate::block::Block;
    use substrate::schematic::{CellBuilder, PrimitiveBinding, Schematic, ScirBinding};
    use substrate::types::schematic::IoNodeBundle;
    use substrate::types::TwoTerminalIo;

    // begin-code-snippet scir-schema
    pub struct MySchema;

    #[derive(Debug, Copy, Clone)]
    pub enum MyPrimitive {
        Resistor(i64),
        Capacitor(i64),
    }

    impl Schema for MySchema {
        type Primitive = MyPrimitive;
    }
    // end-code-snippet scir-schema

    // begin-code-snippet scir-primitive-binding
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct Resistor(i64);

    impl Schematic for Resistor {
        type Schema = MySchema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            let mut prim = PrimitiveBinding::new(MyPrimitive::Resistor(self.0));

            prim.connect("p", io.p);
            prim.connect("n", io.n);

            cell.set_primitive(prim);
            Ok(())
        }
    }
    // end-code-snippet scir-primitive-binding

    // begin-code-snippet scir-scir-binding
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo")]
    pub struct ParallelResistors(i64, i64);

    impl Schematic for ParallelResistors {
        type Schema = MySchema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> substrate::error::Result<Self::NestedData> {
            // Creates a SCIR library containing the desired cell.
            let mut lib = LibraryBuilder::<MySchema>::new();
            let r1 = lib.add_primitive(MyPrimitive::Resistor(self.0));
            let r2 = lib.add_primitive(MyPrimitive::Resistor(self.1));
            let mut parallel_resistors = Cell::new("parallel_resistors");
            let p = parallel_resistors.add_node("p");
            let n = parallel_resistors.add_node("n");
            parallel_resistors.expose_port(p, Direction::InOut);
            parallel_resistors.expose_port(n, Direction::InOut);
            let mut r1 = Instance::new("r1", r1);
            r1.connect("p", p);
            r1.connect("n", n);
            parallel_resistors.add_instance(r1);
            let mut r2 = Instance::new("r2", r2);
            r2.connect("p", p);
            r2.connect("n", n);
            parallel_resistors.add_instance(r2);
            let cell_id = lib.add_cell(parallel_resistors);

            // Binds to the desired cell in the SCIR library.
            let mut scir = ScirBinding::new(lib.build().unwrap(), cell_id);

            scir.connect("p", io.p);
            scir.connect("n", io.n);

            cell.set_scir(scir);
            Ok(())
        }
    }
    // end-code-snippet scir-scir-binding

    #[allow(unused_variables)]
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
        // begin-code-snippet scir-library-signals
        let mut vdivider = Cell::new("vdivider");

        let vdd = vdivider.add_node("vdd");
        let vout = vdivider.add_node("vout");
        let vss = vdivider.add_node("vss");

        vdivider.expose_port(vdd, Direction::InOut);
        vdivider.expose_port(vout, Direction::Output);
        vdivider.expose_port(vss, Direction::InOut);
        // end-code-snippet scir-library-signals
        // begin-code-snippet scir-library-primitive-instances
        let mut r1 = Instance::new("r1", resistor_id);

        r1.connect("p", vdd);
        r1.connect("n", vout);

        vdivider.add_instance(r1);

        let mut r2 = Instance::new("r2", resistor_id);

        r2.connect("p", vout);
        r2.connect("n", vss);

        vdivider.add_instance(r2);

        let vdivider_id = lib.add_cell(vdivider);
        // end-code-snippet scir-library-primitive-instances
        // begin-code-snippet scir-library-instances
        let mut stacked_vdivider = Cell::new("stacked_vdivider");

        let vdd = stacked_vdivider.add_node("vdd");
        let v1 = stacked_vdivider.add_node("v1");
        let v2 = stacked_vdivider.add_node("v2");
        let v3 = stacked_vdivider.add_node("v3");
        let vss = stacked_vdivider.add_node("vss");

        let mut vdiv1 = Instance::new("vdiv1", vdivider_id);

        vdiv1.connect("vdd", vdd);
        vdiv1.connect("vout", v1);
        vdiv1.connect("vss", v2);

        stacked_vdivider.add_instance(vdiv1);

        let mut vdiv2 = Instance::new("vdiv2", vdivider_id);

        vdiv2.connect("vdd", v2);
        vdiv2.connect("vout", v3);
        vdiv2.connect("vss", vss);

        stacked_vdivider.add_instance(vdiv2);

        let stacked_vdivider_id = lib.add_cell(stacked_vdivider);
        // end-code-snippet scir-library-instances
    }
}

fn main() {}
