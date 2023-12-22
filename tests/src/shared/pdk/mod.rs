use ngspice::Ngspice;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::block::Block;
use substrate::context::{Context, ContextBuilder, Installation, PdkContext};
use substrate::io::schematic::HardwareType;
use substrate::io::MosIo;
use substrate::pdk::Pdk;
use substrate::schematic::{CellBuilder, ExportsNestedData, PrimitiveBinding, Schematic};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

#[derive(Copy, Clone, Debug)]
pub enum ExamplePrimitive {
    Pmos { w: i64, l: i64 },
    Nmos { w: i64, l: i64 },
}

pub struct ExamplePdkA;

impl Installation for ExamplePdkA {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
    const LAYOUT_DB_UNITS: Decimal = dec!(1e-9);
}

impl scir::schema::Schema for ExamplePdkA {
    type Primitive = ExamplePrimitive;
}

pub struct ExamplePdkB;

impl scir::schema::Schema for ExamplePdkB {
    type Primitive = ExamplePrimitive;
}

impl Installation for ExamplePdkB {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
    const LAYOUT_DB_UNITS: Decimal = dec!(1e-9);
}

pub struct ExamplePdkC;

impl Installation for ExamplePdkC {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        ctx.install_pdk_layers::<Self>();
    }
}

impl Pdk for ExamplePdkC {
    type Layers = ExamplePdkBLayers;
    const LAYOUT_DB_UNITS: Decimal = dec!(1e-9);
}

impl scir::schema::Schema for ExamplePdkC {
    type Primitive = ExamplePrimitive;
}

/// An NMOS in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosA {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosA {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("nmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("nmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for NmosA {
    type NestedData = ();
}

impl Schematic<ExamplePdkA> for NmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An PMOS in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosA {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosA {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("pmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("pmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for PmosA {
    type NestedData = ();
}

impl Schematic<ExamplePdkA> for PmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An NMOS in PDK B.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosB {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosB {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("nmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("nmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for NmosB {
    type NestedData = ();
}

impl Schematic<ExamplePdkB> for NmosB {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An PMOS in PDK B.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosB {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosB {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("pmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("pmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for PmosB {
    type NestedData = ();
}

impl Schematic<ExamplePdkB> for PmosB {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An NMOS in PDK C.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosC {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosC {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("nmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("nmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for NmosC {
    type NestedData = ();
}

impl Schematic<ExamplePdkC> for NmosC {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkC>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An PMOS in PDK C.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosC {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosC {
    type Io = MosIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("pmos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("pmos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for PmosC {
    type NestedData = ();
}

impl Schematic<ExamplePdkC> for PmosC {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkC>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// Create a new Substrate context for the SKY130 commercial PDK.
///
/// Sets the PDK root to the value of the `SKY130_COMMERCIAL_PDK_ROOT`
/// environment variable and installs Spectre with default configuration.
///
/// # Panics
///
/// Panics if the `SKY130_COMMERCIAL_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_commercial_ctx() -> PdkContext<Sky130Pdk> {
    // Open PDK needed for standard cells.
    let open_pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    let commercial_pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(Spectre::default())
        .install(Sky130Pdk::new(open_pdk_root, commercial_pdk_root))
        .build()
        .with_pdk()
}

/// Create a new Substrate context for the SKY130 open-source PDK.
///
/// Sets the PDK root to the value of the `SKY130_OPEN_PDK_ROOT`
/// environment variable.
///
/// # Panics
///
/// Panics if the `SKY130_OPEN_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_open_ctx() -> PdkContext<Sky130Pdk> {
    let pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(Ngspice::default())
        .install(Sky130Pdk::open(pdk_root))
        .build()
        .with_pdk()
}
