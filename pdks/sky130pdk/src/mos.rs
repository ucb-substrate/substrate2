use std::collections::HashMap;
use std::fmt::Display;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::MosIo;

use super::{Sky130CommercialPdk, Sky130OpenPdk};

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
    ($typ:ident, $name:ident, $doc:literal, $opensubckt:ident, $comsubckt:ident) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
        #[doc = $doc]
        #[doc = ""]
        #[doc = concat!("In the open-source PDK, produces an instance of `", stringify!($opensubckt), "`.")]
        #[doc = concat!("In the commercial PDK, produces an instance of `", stringify!($comsubckt), "`.")]
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

        impl substrate::schematic::HasSchematicData for $typ {
            type Data = ();
        }

        impl substrate::schematic::HasSchematic<Sky130OpenPdk> for $typ {
            fn schematic(
                &self,
                io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
                cell: &mut substrate::schematic::CellBuilder<Sky130OpenPdk, Self>,
            ) -> substrate::error::Result<Self::Data> {
                // Convert from DB units to microns.
                let w = Decimal::new(self.params.w, 3);
                let l = Decimal::new(self.params.l, 3);
                cell.add_primitive(substrate::schematic::PrimitiveDevice::from_params(
                    substrate::schematic::PrimitiveDeviceKind::RawInstance {
                        ports: vec![
                            substrate::schematic::PrimitiveNode::new("d", io.d),
                            substrate::schematic::PrimitiveNode::new("g", io.g),
                            substrate::schematic::PrimitiveNode::new("s", io.s),
                            substrate::schematic::PrimitiveNode::new("b", io.b),
                        ],
                        cell: arcstr::literal!(stringify!($opensubckt)),
                    },
                    HashMap::from_iter([
                        (
                            arcstr::literal!("w"),
                            substrate::scir::Expr::NumericLiteral(w),
                        ),
                        (
                            arcstr::literal!("l"),
                            substrate::scir::Expr::NumericLiteral(l),
                        ),
                        (
                            arcstr::literal!("nf"),
                            substrate::scir::Expr::NumericLiteral(self.params.nf.into()),
                        ),
                    ]),
                ));
                Ok(())
            }
        }

        impl substrate::schematic::HasSchematic<Sky130CommercialPdk> for $typ {
            fn schematic(
                &self,
                io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
                cell: &mut substrate::schematic::CellBuilder<Sky130CommercialPdk, Self>,
            ) -> substrate::error::Result<Self::Data> {
                // Convert from DB units to microns.
                let w = Decimal::new(self.params.w, 3);
                let l = Decimal::new(self.params.l, 3);
                cell.add_primitive(substrate::schematic::PrimitiveDevice::from_params(
                    substrate::schematic::PrimitiveDeviceKind::RawInstance {
                        ports: vec![
                            substrate::schematic::PrimitiveNode::new("d", io.d),
                            substrate::schematic::PrimitiveNode::new("g", io.g),
                            substrate::schematic::PrimitiveNode::new("s", io.s),
                            substrate::schematic::PrimitiveNode::new("b", io.b),
                        ],
                        cell: arcstr::literal!(stringify!($comsubckt)),
                    },
                    HashMap::from_iter([
                        (
                            arcstr::literal!("w"),
                            substrate::scir::Expr::NumericLiteral(w),
                        ),
                        (
                            arcstr::literal!("l"),
                            substrate::scir::Expr::NumericLiteral(l),
                        ),
                        (
                            arcstr::literal!("nf"),
                            substrate::scir::Expr::NumericLiteral(self.params.nf.into()),
                        ),
                    ]),
                ));
                Ok(())
            }
        }
    };
}

define_mos!(
    Nfet01v8,
    nfet_01v8,
    "A core NMOS device.",
    sky130_fd_pr__nfet_01v8,
    nshort
);
define_mos!(
    Nfet01v8Lvt,
    nfet_01v8_lvt,
    "A core low-threshold NMOS device.",
    sky130_fd_pr__nfet_01v8_lvt,
    nlowvt
);
define_mos!(
    Nfet03v3Nvt,
    nfet_03v3_nvt,
    "A 3.3V native-threshold NMOS device.",
    sky130_fd_pr__nfet_03v3_nvt,
    ntvnative
);
define_mos!(
    Nfet05v0Nvt,
    nfet_05v0_nvt,
    "A 5.0V native-threshold NMOS device.",
    sky130_fd_pr__nfet_05v0_nvt,
    nhvnative
);
define_mos!(
    Nfet20v0,
    nfet_20v0,
    "A 20.0V NMOS device.",
    sky130_fd_pr__nfet_20v0,
    nvhv
);

define_mos!(
    SpecialNfetLatch,
    special_nfet_latch,
    "A special latch NMOS, used as the pull down device in SRAM cells.",
    sky130_fd_pr__special_nfet_latch,
    npd
);
define_mos!(
    SpecialNfetPass,
    special_nfet_pass,
    "A special pass NMOS, used as the access device in SRAM cells.",
    sky130_fd_pr__special_nfet_pass,
    npass
);
define_mos!(
    SpecialPfetPass,
    special_pfet_pass,
    "A special pass PMOS, used as the pull-up device in SRAM cells.",
    sky130_fd_pr__special_pfet_pass,
    ppu
);

define_mos!(
    Pfet01v8,
    pfet_01v8,
    "A core PMOS device.",
    sky130_fd_pr__pfet_01v8,
    pshort
);
define_mos!(
    Pfet01v8Hvt,
    pfet_01v8_hvt,
    "A core high-threshold PMOS device.",
    sky130_fd_pr__pfet_01v8_hvt,
    phighvt
);
define_mos!(
    Pfet01v8Lvt,
    pfet_01v8_lvt,
    "A core low-threshold PMOS device.",
    sky130_fd_pr__pfet_01v8_lvt,
    plowvt
);
define_mos!(
    Pfet20v0,
    pfet_20v0,
    "A 20.0V PMOS device.",
    sky130_fd_pr__pfet_20v0,
    pvhv
);
