use indexmap::IndexMap;
use ngspice::Ngspice;
use scir::Expr;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use substrate::block;
use substrate::block::{Block, PdkPrimitive};
use substrate::context::Context;
use substrate::io::MosIo;
use substrate::pdk::{HasPdkPrimitive, Pdk};
use substrate::schematic::{ExportsNestedData, Schematic};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

#[derive(Copy, Clone, Debug)]
pub enum ExamplePrimitive {
    Pmos { w: i64, l: i64 },
    Nmos { w: i64, l: i64 },
}

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Primitive = ExamplePrimitive;
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Primitive = ExamplePrimitive;
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
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
    type Kind = block::PdkPrimitive;
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

impl HasPdkPrimitive<NmosA> for ExamplePdkA {
    fn primitive(block: &NmosA) -> Self::Primitive {
        ExamplePrimitive::Nmos {
            w: block.w,
            l: block.l,
        }
    }
}

/// An PMOS in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PmosA {
    pub w: i64,
    pub l: i64,
}

impl Block for PmosA {
    type Kind = PdkPrimitive;
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

impl HasPdkPrimitive<PmosA> for ExamplePdkA {
    fn primitive(block: &PmosA) -> Self::Primitive {
        ExamplePrimitive::Pmos {
            w: block.w,
            l: block.l,
        }
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
pub fn sky130_commercial_ctx() -> Context<Sky130Pdk> {
    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    Context::builder()
        .pdk(Sky130Pdk::commercial(pdk_root))
        .with_simulator(Spectre::default())
        .build()
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
pub fn sky130_open_ctx() -> Context<Sky130Pdk> {
    let pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    Context::builder()
        .pdk(Sky130Pdk::open(pdk_root))
        .with_simulator(Ngspice::default())
        .build()
}
