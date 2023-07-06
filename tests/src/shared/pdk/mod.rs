use std::collections::HashMap;

use scir::Expr;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130CommercialPdk;
use spectre::Spectre;
use substrate::block::Block;
use substrate::context::Context;
use substrate::ios::MosIo;
use substrate::pdk::corner::Corner;
use substrate::pdk::Pdk;
use substrate::schematic::{HasSchematic, HasSchematicImpl, PrimitiveDevice};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
}

#[derive(Debug, Clone, Copy)]
pub struct ExampleCorner;

impl Corner for ExampleCorner {
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("example_corner")
    }
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

impl HasSchematic for NmosA {
    type Data = ();
}

impl HasSchematicImpl<ExamplePdkA> for NmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(PrimitiveDevice::RawInstance {
            ports: vec![*io.d, *io.g, *io.s, *io.b],
            cell: arcstr::literal!("example_pdk_nmos_a"),
            params: HashMap::from_iter([
                (arcstr::literal!("w"), Expr::NumericLiteral(self.w.into())),
                (arcstr::literal!("l"), Expr::NumericLiteral(self.l.into())),
            ]),
        });
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

impl HasSchematic for PmosA {
    type Data = ();
}

impl HasSchematicImpl<ExamplePdkA> for PmosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(PrimitiveDevice::RawInstance {
            ports: vec![*io.d, *io.g, *io.s, *io.b],
            cell: arcstr::literal!("example_pdk_pmos_a"),
            params: HashMap::from_iter([
                (arcstr::literal!("w"), Expr::NumericLiteral(self.w.into())),
                (arcstr::literal!("l"), Expr::NumericLiteral(self.l.into())),
            ]),
        });
        Ok(())
    }
}

// begin-code-snippet sky130-commercial-ctx
/// Create a new Substrate context for the SKY130 commercial PDK.
///
/// Sets the PDK root to the value of the `SKY130_COMMERCIAL_PDK_ROOT`
/// environment variable and installs Spectre with default configuration.
///
/// # Panics
///
/// Panics if the `SKY130_COMMERCIAL_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_commercial_ctx() -> Context<Sky130CommercialPdk> {
    let pdk_root = std::env::var("SKY130_COMMERCIAL_PDK_ROOT")
        .expect("the SKY130_COMMERCIAL_PDK_ROOT environment variable must be set");
    Context::builder()
        .pdk(Sky130CommercialPdk::new(pdk_root))
        .with_simulator(Spectre::default())
        .build()
}
// end-code-snippet sky130-commercial-ctx
