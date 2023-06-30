use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::ios::MosIo;
use substrate::schematic::{HasSchematic, HasSchematicImpl};

use super::Sky130Pdk;

/// MOSFET sizing parameters.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MosParams {
    pub w: i64,
    pub l: i64,
}

impl Display for MosParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.w, self.l)
    }
}

/// A core NMOS device (`sky130_fd_pr__nfet_01v8`).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nfet {
    params: MosParams,
}

impl Nfet {
    #[inline]
    pub fn new(params: impl Into<MosParams>) -> Self {
        Self {
            params: params.into(),
        }
    }
}

impl Block for Nfet {
    type Io = MosIo;
    fn id() -> substrate::arcstr::ArcStr {
        arcstr::literal!("nfet")
    }
    fn name(&self) -> substrate::arcstr::ArcStr {
        arcstr::format!("nfet_{}", self.params)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Nfet {
    type Data = ();
}

impl HasSchematicImpl<Sky130Pdk> for Nfet {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(substrate::schematic::PrimitiveDevice::RawInstance {
            ports: vec![*io.d, *io.g, *io.s, *io.b],
            cell: arcstr::literal!("sky130_fd_pr__nfet_01v8"),
            params: HashMap::from_iter([
                (
                    arcstr::literal!("w"),
                    substrate::scir::Expr::NumericLiteral(self.params.w.into()),
                ),
                (
                    arcstr::literal!("l"),
                    substrate::scir::Expr::NumericLiteral(self.params.l.into()),
                ),
            ]),
        });
        Ok(())
    }
}

/// A core PMOS device (`sky130_fd_pr__pfet_01v8`).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pfet {
    params: MosParams,
}

impl Pfet {
    #[inline]
    pub fn new(params: impl Into<MosParams>) -> Self {
        Self {
            params: params.into(),
        }
    }
}

impl Block for Pfet {
    type Io = MosIo;
    fn id() -> substrate::arcstr::ArcStr {
        arcstr::literal!("pfet")
    }
    fn name(&self) -> substrate::arcstr::ArcStr {
        arcstr::format!("pfet_{}", self.params)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Pfet {
    type Data = ();
}

impl HasSchematicImpl<Sky130Pdk> for Pfet {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(substrate::schematic::PrimitiveDevice::RawInstance {
            ports: vec![*io.d, *io.g, *io.s, *io.b],
            cell: arcstr::literal!("sky130_fd_pr__pfet_01v8"),
            params: HashMap::from_iter([
                (
                    arcstr::literal!("w"),
                    substrate::scir::Expr::NumericLiteral(self.params.w.into()),
                ),
                (
                    arcstr::literal!("l"),
                    substrate::scir::Expr::NumericLiteral(self.params.l.into()),
                ),
            ]),
        });
        Ok(())
    }
}
