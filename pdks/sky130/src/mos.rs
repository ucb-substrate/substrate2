//! MOS devices and parameters.

use std::fmt::Display;

use crate::layers::Sky130Layer;
use crate::{Sky130, Sky130Schema};
use arcstr::ArcStr;
use geometry_macros::{TransformMut, TransformRef, TranslateMut, TranslateRef};
use layir::Shape;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::dir::Dir;
use substrate::geometry::rect::Rect;
use substrate::geometry::span::Span;
use substrate::layout::tracks::{RoundingMode, Tracks, UniformTracks};
use substrate::layout::Layout;
use substrate::schematic::CellBuilder;
use substrate::types::codegen::PortGeometryBundle;
use substrate::types::layout::PortGeometry;
use substrate::types::{Array, ArrayBundle, FlatLen, InOut, Input, Io, MosIo, Signal};

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
    ($({$typ:ident, $name:ident, $doc:literal, $opensubckt:ident, $srcndasubckt:ident, $cdssubckt:ident}),*) => {
        /// An enumeration of Sky 130 MOSFET varieties.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum MosKind {
            $(
                #[doc = $doc]
                #[doc = ""]
                #[doc = concat!("In the open-source PDK, produces an instance of `", stringify!($opensubckt), "`.")]
                #[doc = concat!("In the SRC NDA PDK, produces an instance of `", stringify!($srcndasubckt), "`.")]
                #[doc = concat!("In the CDS PDK, produces an instance of `", stringify!($cdssubckt), "`.")]
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

            pub(crate) fn src_nda_subckt(&self) -> arcstr::ArcStr {
                match self {
                    $(
                        MosKind::$typ => arcstr::literal!(stringify!($srcndasubckt))
                    ),*
                }
            }

            pub(crate) fn cds_subckt(&self) -> arcstr::ArcStr {
                match self {
                    $(
                        MosKind::$typ => arcstr::literal!(stringify!($cdssubckt))
                    ),*
                }
            }

            pub(crate) fn try_from_str(kind: &str) -> Option<Self> {
                match kind {
                    $(
                        stringify!($opensubckt) | stringify!($srcndasubckt) | stringify!($cdssubckt) => Some(MosKind::$typ),
                    )*
                    _ => None,
                }
            }

            pub(crate) fn schema(kind: &str) -> Option<Sky130Schema> {
                match kind {
                    $(
                        stringify!($opensubckt) => Some(Sky130Schema::Open),
                        stringify!($srcndasubckt) => Some(Sky130Schema::SrcNda),
                        stringify!($cdssubckt) => Some(Sky130Schema::Cds),
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
        #[doc = concat!("In the SRC NDA PDK, produces an instance of `", stringify!($srcndasubckt), "`.")]
        #[doc = concat!("In the CDS PDK, produces an instance of `", stringify!($cdssubckt), "`.")]
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

            fn name(&self) -> substrate::arcstr::ArcStr {
                arcstr::format!(concat!(stringify!($name), "_{}"), self.params)
            }
            fn io(&self) -> Self::Io {
                Default::default()
            }
        }

        impl substrate::schematic::Schematic for $typ {
            type Schema = crate::Sky130;
            type NestedData = ();
            fn schematic(
                    &self,
                    io: &substrate::types::schematic::IoNodeBundle<Self>,
                    cell: &mut CellBuilder<<Self as substrate::schematic::Schematic>::Schema>,
                ) -> substrate::error::Result<Self::NestedData> {
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
        nshort,
        nfet_01v8
    },
    {
        Nfet01v8Lvt,
        nfet_01v8_lvt,
        "A core low-threshold NMOS device.",
        sky130_fd_pr__nfet_01v8_lvt,
        nlowvt,
        nfet_01v8_lvt
    },
    {
        Nfet03v3Nvt,
        nfet_03v3_nvt,
        "A 3.3V native-threshold NMOS device.",
        sky130_fd_pr__nfet_03v3_nvt,
        ntvnative,
        nfet_03v3_nvt
    },
    {
        Nfet05v0Nvt,
        nfet_05v0_nvt,
        "A 5.0V native-threshold NMOS device.",
        sky130_fd_pr__nfet_05v0_nvt,
        nhvnative,
        nfet_05v0_nvt
    },
    {
        Nfet20v0,
        nfet_20v0,
        "A 20.0V NMOS device.",
        sky130_fd_pr__nfet_20v0,
        nvhv,
        nfet_20v0
    },
    {
        SpecialNfetLatch,
        special_nfet_latch,
        "A special latch NMOS, used as the pull down device in SRAM cells.",
        sky130_fd_pr__special_nfet_latch,
        npd,
        special_nfet_latch
    },
    {
        SpecialNfetPass,
        special_nfet_pass,
        "A special pass NMOS, used as the access device in SRAM cells.",
        sky130_fd_pr__special_nfet_pass,
        npass,
        special_nfet_pass
    },
    {
        SpecialPfetPass,
        special_pfet_pass,
        "A special pass PMOS, used as the pull-up device in SRAM cells.",
        sky130_fd_pr__special_pfet_pass,
        ppu,
        special_pfet_pass
    },
    {
        Pfet01v8,
        pfet_01v8,
        "A core PMOS device.",
        sky130_fd_pr__pfet_01v8,
        pshort,
        pfet_01v8
    },
    {
        Pfet01v8Hvt,
        pfet_01v8_hvt,
        "A core high-threshold PMOS device.",
        sky130_fd_pr__pfet_01v8_hvt,
        phighvt,
        pfet_01v8_hvt
    },
    {
        Pfet01v8Lvt,
        pfet_01v8_lvt,
        "A core low-threshold PMOS device.",
        sky130_fd_pr__pfet_01v8_lvt,
        plowvt,
        pfet_01v8_lvt
    },
    {
        Pfet20v0,
        pfet_20v0,
        "A 20.0V PMOS device.",
        sky130_fd_pr__pfet_20v0,
        pvhv,
        pfet_20v0
    }
);

/// Determines the connection direction of a transistor gate.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateDir {
    /// Connects the gate towards the right.
    #[default]
    Right,
    /// Connects the gate towards the left.
    Left,
}

/// The IO of an NMOS or PMOS tile.
#[derive(Debug, Clone, Io)]
pub(crate) struct BareMosTileIo {
    /// `NF + 1` source/drain contacts on li1, where `NF` is the number of fingers.
    pub sd: InOut<Array<Signal>>,
    /// `NF` gate contacts on li1, where `NF` is the number of fingers.
    pub g: Input<Array<Signal>>,
}

/// The IO of an NMOS or PMOS tile.
#[derive(Debug, Clone, Io)]
pub struct MosTileIo {
    /// `NF + 1` source/drain contacts on li1, where `NF` is the number of fingers.
    pub sd: InOut<Array<Signal>>,
    /// `NF` gate contacts on li1, where `NF` is the number of fingers.
    pub g: Input<Array<Signal>>,
    /// The body connection on nwell (PMOS) or pwell (NMOS).
    pub b: InOut<Signal>,
}

/// The set of supported gate lengths.
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Serialize, Deserialize,
)]
pub enum MosLength {
    /// 150nm.
    ///
    /// This is the minimum length supported by the SKY130 technology.
    #[default]
    L150,
}

impl MosLength {
    /// The length in nanometers.
    fn nm(&self) -> i64 {
        match *self {
            Self::L150 => 150,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MosTile {
    pub(crate) w: i64,
    pub(crate) len: MosLength,
    pub(crate) nf: i64,
    pub(crate) gate_dir: GateDir,
}

impl MosTile {
    /// Create a new MOS tile with the given physical transistor dimensions.
    pub fn new(w: i64, len: MosLength, nf: i64) -> Self {
        Self {
            w,
            len,
            nf,
            gate_dir: Default::default(),
        }
    }
}

impl Block for MosTile {
    type Io = BareMosTileIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("mos_tile_w{}_l{}_nf{}", self.w, self.len.nm(), self.nf)
    }

    fn io(&self) -> Self::Io {
        BareMosTileIo {
            sd: InOut(Array::new(self.nf as usize + 1, Signal::new())),
            g: Input(Array::new(
                match self.gate_dir {
                    GateDir::Left => self.nf / 2 + 1,
                    GateDir::Right => (self.nf - 1) / 2 + 1,
                } as usize,
                Signal::new(),
            )),
        }
    }
}

#[derive(TransformRef, TranslateRef, TransformMut, TranslateMut)]
pub(crate) struct MosTileData {
    diff: Rect,
    bbox: Rect,
}

impl MosTile {
    fn layout_inner(
        &self,
        cell: &mut substrate::layout::CellBuilder<<Self as Layout>::Schema>,
    ) -> substrate::error::Result<(<Self as Layout>::Bundle, <Self as Layout>::Data)> {
        let m0tracks = UniformTracks::new(170, 260);
        let m1tracks = UniformTracks::new(400, 140);

        let top_m1 = m1tracks.to_track_idx(self.w + 10, RoundingMode::Up);
        let bot_m1 = m1tracks.to_track_idx(-10, RoundingMode::Down);
        let gate_top_m1 = bot_m1 - 1;
        let gate_vspan = m1tracks
            .track(gate_top_m1)
            .union(m1tracks.track(gate_top_m1 - 1))
            .shrink_all(45);

        let tracks = (1..self.nf + 2)
            .map(|i| {
                let span = m0tracks.track(i);
                Rect::from_spans(span, Span::new(-10, self.w + 10))
            })
            .collect::<Vec<_>>();

        let gate_spans = tracks
            .windows(2)
            .map(|tracks| {
                let (left, right) = (tracks[0], tracks[1]);
                Span::new(left.right(), right.left()).shrink_all(55)
            })
            .collect::<Vec<_>>();

        let mut sd = Vec::new();
        for rect in tracks.iter() {
            let sd_rect = rect.with_vspan(
                m1tracks
                    .track(bot_m1)
                    .union(m1tracks.track(top_m1))
                    .shrink_all(45),
            );
            sd.push(PortGeometry::new(Shape::new(Sky130Layer::Li1, sd_rect)));
            cell.draw(Shape::new(Sky130Layer::Li1, sd_rect))?;
            let num_cuts = (self.w + 20 - 160 + 170) / 340;
            for j in 0..num_cuts {
                let base = rect.bot() + 10 + 80 + 340 * j;
                let cut = Rect::from_spans(rect.hspan(), Span::with_start_and_length(base, 170));
                cell.draw(Shape::new(Sky130Layer::Licon1, cut))?;
            }
        }

        let diff = Rect::from_sides(
            tracks[0].left() - 130,
            0,
            tracks.last().unwrap().right() + 130,
            self.w,
        );
        cell.draw(Shape::new(Sky130Layer::Diff, diff))?;

        let mut g = vec![None; self.io().g.len()];
        for i in 0..self.nf as usize {
            let li_track = tracks[match (i % 2 == 0, self.gate_dir) {
                (true, GateDir::Left) | (false, GateDir::Right) => i,
                _ => i + 1,
            }];

            let gate_idx = |idx| match self.gate_dir {
                GateDir::Left => (idx + 1) / 2,
                GateDir::Right => idx / 2,
            };
            let poly_li = Rect::from_spans(li_track.hspan(), gate_vspan);
            if i == 0 || gate_idx(i) != gate_idx(i - 1) {
                cell.draw(Shape::new(Sky130Layer::Li1, poly_li))?;
                g[gate_idx(i)] = Some(PortGeometry::new(Shape::new(Sky130Layer::Li1, poly_li)));

                let cut = Rect::from_spans(
                    li_track.hspan(),
                    Span::new(poly_li.top() - 90, poly_li.top() - 260),
                );
                cell.draw(Shape::new(Sky130Layer::Licon1, cut))?;

                let npc = Rect::from_spans(
                    poly_li.hspan(),
                    Span::new(poly_li.top(), poly_li.top() - 350),
                )
                .expand_dir(Dir::Vert, 10)
                .expand_dir(Dir::Horiz, 100);
                cell.draw(Shape::new(Sky130Layer::Npc, npc))?;
            }
            let poly = Rect::from_spans(
                gate_spans[i]
                    .union(li_track.hspan())
                    .union(poly_li.hspan().expand_all(50)),
                Span::new(poly_li.top() - 350, poly_li.top()),
            );
            cell.draw(Shape::new(Sky130Layer::Poly, poly))?;
        }
        let g = g.into_iter().map(|x| x.unwrap()).collect();

        for &span in gate_spans.iter() {
            cell.draw(Shape::new(
                Sky130Layer::Poly,
                Rect::from_spans(span, Span::new(gate_vspan.stop() - 350, self.w + 130)),
            ))?;
        }

        let bbox = cell.bbox_rect();
        let lcm_bbox = bbox;
        cell.draw(Shape::new(Sky130Layer::Outline, lcm_bbox))?;

        Ok((
            BareMosTileIoView {
                g: ArrayBundle::new(Signal, g),
                sd: ArrayBundle::new(Signal, sd),
            },
            MosTileData {
                diff,
                bbox: cell.bbox_rect(),
            },
        ))
    }
}

impl Layout for MosTile {
    type Schema = Sky130;
    type Bundle = BareMosTileIoView<substrate::types::codegen::PortGeometryBundle<Sky130>>;
    type Data = MosTileData;
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        self.layout_inner(cell)
    }
}

/// A tile containing a set of NMOS transistors.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NmosTile {
    tile: MosTile,
}

impl NmosTile {
    /// Create a new NMOS tile with the given physical transistor dimensions.
    pub fn new(w: i64, len: MosLength, nf: i64) -> Self {
        Self {
            tile: MosTile::new(w, len, nf),
        }
    }

    /// Sets the connection direction of the left-most gate in the tile.
    ///
    /// Connection directions alternate for each adjacent gate.
    /// A gate will always be connected with the gate adjacent to it
    /// in its connection direction.
    pub fn with_gate_dir(mut self, gate_dir: GateDir) -> Self {
        self.tile.gate_dir = gate_dir;
        self
    }
}

impl Block for NmosTile {
    type Io = MosTileIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("n{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        let io = self.tile.io();
        MosTileIo {
            sd: io.sd,
            g: io.g,
            b: InOut(Signal),
        }
    }
}

impl Layout for NmosTile {
    type Schema = Sky130;
    type Bundle = MosTileIoView<PortGeometryBundle<Sky130>>;
    type Data = ();

    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let (tio, tdata) = self.tile.layout_inner(cell)?;
        let nsdm = tdata.diff.expand_all(130);
        let nsdm = nsdm.with_hspan(tdata.bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Nsdm, nsdm))?;

        let pwell = Shape::new(Sky130Layer::Pwell, tdata.bbox);
        cell.draw(pwell.clone())?;
        let b = PortGeometry::new(pwell);

        Ok((
            MosTileIoView {
                g: tio.g,
                sd: tio.sd,
                b,
            },
            (),
        ))
    }
}

/// A tile containing a set of PMOS transistors.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PmosTile {
    tile: MosTile,
}

impl PmosTile {
    /// Create a new PMOS tile with the given physical transistor dimensions.
    pub fn new(w: i64, len: MosLength, nf: i64) -> Self {
        Self {
            tile: MosTile::new(w, len, nf),
        }
    }

    /// Sets the connection direction of the left-most gate in the tile.
    ///
    /// Connection directions alternate for each adjacent gate.
    /// A gate will always be connected with the gate adjacent to it
    /// in its connection direction.
    pub fn with_gate_dir(mut self, gate_dir: GateDir) -> Self {
        self.tile.gate_dir = gate_dir;
        self
    }
}

impl Block for PmosTile {
    type Io = MosTileIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        let io = self.tile.io();
        MosTileIo {
            sd: io.sd,
            g: io.g,
            b: InOut(Signal),
        }
    }
}

impl Layout for PmosTile {
    type Schema = Sky130;
    type Bundle = MosTileIoView<PortGeometryBundle<Sky130>>;
    type Data = ();

    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let (tio, tdata) = self.tile.layout_inner(cell)?;
        let psdm = tdata.diff.expand_all(130);
        let bbox = tdata.bbox;
        let psdm = psdm.with_hspan(bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Psdm, psdm))?;

        let nwell = tdata.diff.expand_all(180).union(bbox);
        let nwell = Shape::new(Sky130Layer::Nwell, nwell);
        cell.draw(nwell.clone())?;
        let b = PortGeometry::new(nwell);

        Ok((
            MosTileIoView {
                g: tio.g,
                sd: tio.sd,
                b,
            },
            (),
        ))
    }
}
