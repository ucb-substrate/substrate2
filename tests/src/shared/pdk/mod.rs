use ngspice::Ngspice;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::block;
use substrate::block::Block;
use substrate::context::{Context, PdkContext};
use substrate::io::{MosIo, SchematicType};
use substrate::pdk::Pdk;
use substrate::schematic::{Primitive, PrimitiveSchematic};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

#[derive(Copy, Clone, Debug)]
pub enum ExamplePrimitive {
    Pmos { w: i64, l: i64 },
    Nmos { w: i64, l: i64 },
}

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
}

impl scir::schema::Schema for ExamplePdkA {
    type Primitive = ExamplePrimitive;
}

pub struct ExamplePdkB;

impl scir::schema::Schema for ExamplePdkB {
    type Primitive = ExamplePrimitive;
}

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
}

pub struct ExamplePdkC;

impl Pdk for ExamplePdkC {
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
}

impl scir::schema::Schema for ExamplePdkC {
    type Primitive = ExamplePrimitive;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExampleCorner;

/// An NMOS in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosA {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosA {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkA> for NmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkA> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
    }
}

/// An PMOS in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosA {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosA {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkA> for PmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkA> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
    }
}

/// An NMOS in PDK B.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosB {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosB {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkB> for NmosB {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkB> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
    }
}

/// An PMOS in PDK B.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosB {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosB {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkB> for PmosB {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkB> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
    }
}

/// An NMOS in PDK C.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NmosC {
    pub w: i64,
    pub l: i64,
}

impl Block for NmosC {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkC> for NmosC {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkC> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
    }
}

/// An PMOS in PDK C.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosC {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosC {
    type Kind = block::Primitive;
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

impl PrimitiveSchematic<ExamplePdkC> for PmosC {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
    ) -> Primitive<ExamplePdkC> {
        let mut prim = Primitive::new(ExamplePrimitive::Nmos {
            w: self.w,
            l: self.l,
        });
        prim.connect("D", io.d);
        prim.connect("G", io.g);
        prim.connect("S", io.s);
        prim.connect("B", io.b);
        prim
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
    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    Context::builder()
        .with_simulator(Spectre::default())
        .build()
        .with_pdk(Sky130Pdk::commercial(pdk_root))
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
        .with_simulator(Ngspice::default())
        .build()
        .with_pdk(Sky130Pdk::open(pdk_root))
}
