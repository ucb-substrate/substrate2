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
    /// Device width, in nm.
    pub w: i64,
    /// Device channel length, in nm.
    pub l: i64,
    /// Number of fingers.
    pub nf: i64,
}

impl From<(i64, i64, i64)> for MosParams {
    fn from(value: (i64, i64, i64)) -> Self {
        Self {
            w: value.0,
            l: value.1,
            nf: value.2,
        }
    }
}

impl From<(i64, i64)> for MosParams {
    fn from(value: (i64, i64)) -> Self {
        Self {
            w: value.0,
            l: value.1,
            nf: 1,
        }
    }
}

impl Display for MosParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}x{}", self.w, self.l, self.nf)
    }
}

macro_rules! define_mos {
    ($typ:ident, $name:ident, $doc:literal, $subckt:ident) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
        #[doc = $doc]
        #[doc = ""]
        #[doc = concat!("Produces an instance of `", stringify!($subckt), "`.")]
        pub struct $typ {
            params: MosParams,
        }

        impl $typ {
            #[inline]
            pub fn new(params: impl Into<MosParams>) -> Self {
                Self {
                    params: params.into(),
                }
            }
        }

        impl Block for $typ {
            type Io = MosIo;
            fn id() -> substrate::arcstr::ArcStr {
                arcstr::literal!(stringify!($name))
            }
            fn name(&self) -> substrate::arcstr::ArcStr {
                arcstr::format!(concat!(stringify!($name), "_{}"), self.params)
            }
            fn io(&self) -> Self::Io {
                Default::default()
            }
        }

        impl HasSchematic for $typ {
            type Data = ();
        }

        impl HasSchematicImpl<Sky130Pdk> for $typ {
            fn schematic(
                &self,
                io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
                cell: &mut substrate::schematic::CellBuilder<Sky130Pdk, Self>,
            ) -> substrate::error::Result<Self::Data> {
                cell.add_primitive(substrate::schematic::PrimitiveDevice::RawInstance {
                    ports: vec![*io.d, *io.g, *io.s, *io.b],
                    cell: arcstr::literal!(stringify!($subckt)),
                    params: HashMap::from_iter([
                        (
                            arcstr::literal!("w"),
                            substrate::scir::Expr::NumericLiteral(self.params.w.into()),
                        ),
                        (
                            arcstr::literal!("l"),
                            substrate::scir::Expr::NumericLiteral(self.params.l.into()),
                        ),
                        (
                            arcstr::literal!("nf"),
                            substrate::scir::Expr::NumericLiteral(self.params.nf.into()),
                        ),
                    ]),
                });
                Ok(())
            }
        }
    };
}

define_mos!(
    Nfet01v8,
    nfet_01v8,
    "A core NMOS device.",
    sky130_fd_pr__nfet_01v8
);
define_mos!(
    Nfet01v8Lvt,
    nfet_01v8_lvt,
    "A core low-threshold NMOS device.",
    sky130_fd_pr__nfet_01v8_lvt
);
define_mos!(
    Nfet03v3Nvt,
    nfet_03v3_nvt,
    "A 3.3V native-threshold NMOS device.",
    sky130_fd_pr__nfet_03v3_nvt
);
define_mos!(
    Nfet05v0Nvt,
    nfet_05v0_nvt,
    "A 5.0V native-threshold NMOS device.",
    sky130_fd_pr__nfet_05v0_nvt
);
define_mos!(
    Nfet20v0,
    nfet_20v0,
    "A 20.0V NMOS device.",
    sky130_fd_pr__nfet_20v0
);

define_mos!(
    SpecialNfetLatch,
    special_nfet_latch,
    "A special latch NMOS, used as the pull down device in SRAM cells.",
    sky130_fd_pr__special_nfet_latch
);
define_mos!(
    SpecialNfetPass,
    special_nfet_pass,
    "A special pass NMOS, used as the access device in SRAM cells.",
    sky130_fd_pr__special_nfet_pass
);
define_mos!(
    SpecialPfetPass,
    special_pfet_pass,
    "A special pass PMOS, used as the pull-up device in SRAM cells.",
    sky130_fd_pr__special_pfet_pass
);

define_mos!(
    Pfet01v8,
    pfet_01v8,
    "A core PMOS device.",
    sky130_fd_pr__pfet_01v8
);
define_mos!(
    Pfet01v8Hvt,
    pfet_01v8_hvt,
    "A core high-threshold PMOS device.",
    sky130_fd_pr__pfet_01v8_hvt
);
define_mos!(
    Pfet01v8Lvt,
    pfet_01v8_lvt,
    "A core low-threshold PMOS device.",
    sky130_fd_pr__pfet_01v8_lvt
);
define_mos!(
    Pfet01v8Mvt,
    pfet_01v8_mvt,
    "A core MVT PMOS device.",
    sky130_fd_pr__pfet_01v8_mvt
);
define_mos!(
    Pfet20v0,
    pfet_20v0,
    "A 20.0V PMOS device.",
    sky130_fd_pr__pfet_20v0
);
