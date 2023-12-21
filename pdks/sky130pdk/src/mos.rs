//! MOS devices and parameters.

use std::fmt::Display;

use crate::Sky130Pdk;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::MosIo;
use substrate::schematic::CellBuilder;

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

macro_rules! define_mosfets {
    ($({$typ:ident, $name:ident, $doc:literal, $opensubckt:ident, $comsubckt:ident}),*) => {
        /// An enumeration of Sky 130 MOSFET varieties.
        #[derive(Clone, Copy, Debug)]
        pub enum MosKind {
            $(
                #[doc = $doc]
                #[doc = ""]
                #[doc = concat!("In the open-source PDK, produces an instance of `", stringify!($opensubckt), "`.")]
                #[doc = concat!("In the commercial PDK, produces an instance of `", stringify!($comsubckt), "`.")]
                $typ,
            )*
        }

        impl MosKind {
            pub(crate) fn open_subckt(&self) -> arcstr::ArcStr {
                match self {
                    $(
                        MosKind::$typ => arcstr::literal!(stringify!($opensubckt))
                    ),*
                }
            }
            pub(crate) fn commercial_subckt(&self) -> arcstr::ArcStr {
                match self {
                    $(
                        MosKind::$typ => arcstr::literal!(stringify!($comsubckt))
                    ),*
                }
            }

            pub(crate) fn try_from_str(kind: &str) -> Option<Self> {
                match kind {
                    $(
                        stringify!($opensubckt) | stringify!($comsubckt) => Some(MosKind::$typ),
                    )*
                    _ => None,
                }
            }
        }
        $(
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
        #[doc = $doc]
        #[doc = ""]
        #[doc = concat!("In the open-source PDK, produces an instance of `", stringify!($opensubckt), "`.")]
        #[doc = concat!("In the commercial PDK, produces an instance of `", stringify!($comsubckt), "`.")]
        pub struct $typ {
            params: MosParams,
        }

        impl $typ {
            /// Creates a new [`$typ`].
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

        impl substrate::schematic::ExportsNestedData for $typ {
            type NestedData = ();
        }

        impl substrate::schematic::Schematic<crate::Sky130Pdk> for $typ {
            fn schematic(&self, io: &<<Self as Block>::Io as HardwareType>::Bundle, cell: &mut CellBuilder<Sky130Pdk>) -> substrate::error::Result<Self::NestedData> {
                let mut prim = substrate::schematic::PrimitiveBinding::new(crate::Primitive::Mos {
                    kind: MosKind::$typ,
                    params: self.params.clone(),
                });
                prim.connect("D", io.d);
                prim.connect("G", io.g);
                prim.connect("S", io.s);
                prim.connect("B", io.b);
                cell.set_primitive(prim);
                Ok(())
            }
        }
        )*
    };
}

define_mosfets!(
    {
        Nfet01v8,
        nfet_01v8,
        "A core NMOS device.",
        sky130_fd_pr__nfet_01v8,
        nshort
    },
    {
        Nfet01v8Lvt,
        nfet_01v8_lvt,
        "A core low-threshold NMOS device.",
        sky130_fd_pr__nfet_01v8_lvt,
        nlowvt
    },
    {
        Nfet03v3Nvt,
        nfet_03v3_nvt,
        "A 3.3V native-threshold NMOS device.",
        sky130_fd_pr__nfet_03v3_nvt,
        ntvnative
    },
    {
        Nfet05v0Nvt,
        nfet_05v0_nvt,
        "A 5.0V native-threshold NMOS device.",
        sky130_fd_pr__nfet_05v0_nvt,
        nhvnative
    },
    {
        Nfet20v0,
        nfet_20v0,
        "A 20.0V NMOS device.",
        sky130_fd_pr__nfet_20v0,
        nvhv
    },
    {
        SpecialNfetLatch,
        special_nfet_latch,
        "A special latch NMOS, used as the pull down device in SRAM cells.",
        sky130_fd_pr__special_nfet_latch,
        npd
    },
    {
        SpecialNfetPass,
        special_nfet_pass,
        "A special pass NMOS, used as the access device in SRAM cells.",
        sky130_fd_pr__special_nfet_pass,
        npass
    },
    {
        SpecialPfetPass,
        special_pfet_pass,
        "A special pass PMOS, used as the pull-up device in SRAM cells.",
        sky130_fd_pr__special_pfet_pass,
        ppu
    },
    {
        Pfet01v8,
        pfet_01v8,
        "A core PMOS device.",
        sky130_fd_pr__pfet_01v8,
        pshort
    },
    {
        Pfet01v8Hvt,
        pfet_01v8_hvt,
        "A core high-threshold PMOS device.",
        sky130_fd_pr__pfet_01v8_hvt,
        phighvt
    },
    {
        Pfet01v8Lvt,
        pfet_01v8_lvt,
        "A core low-threshold PMOS device.",
        sky130_fd_pr__pfet_01v8_lvt,
        plowvt
    },
    {
        Pfet20v0,
        pfet_20v0,
        "A 20.0V PMOS device.",
        sky130_fd_pr__pfet_20v0,
        pvhv
    }
);
