//! SKY130 primitives for [ATOLL](atoll).

use crate::layers::Sky130Layers;
use crate::mos::{MosParams, Nfet01v8, Pfet01v8};
use crate::Sky130Pdk;
use arcstr::ArcStr;
use atoll::abs::TrackCoord;
use atoll::grid::{AbstractLayer, LayerStack, PdkLayer, RoutingGrid};
use atoll::route::ViaMaker;
use atoll::RoutingDir;
use serde::{Deserialize, Serialize};

use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::dir::Dir;
use substrate::geometry::rect::Rect;
use substrate::geometry::span::Span;
use substrate::geometry::transform::Translate;
use substrate::io::layout::IoShape;
use substrate::io::schematic::Bundle;
use substrate::io::{Array, InOut, Input, Io, MosIoSchematic, Signal};
use substrate::layout::element::Shape;
use substrate::layout::tracks::{RoundingMode, Tracks};
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::pdk::layers::Layer;
use substrate::schematic::{ExportsNestedData, Schematic};

impl Sky130Layers {
    /// Returns the ATOLL-compatible routing layer stack.
    pub fn atoll_layer_stack(&self) -> LayerStack<PdkLayer> {
        LayerStack {
            layers: vec![
                PdkLayer {
                    id: self.li1.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Any {
                            track_dir: Dir::Vert,
                        },
                        line: 170,
                        space: 260,
                        offset: 85,
                        endcap: 85,
                    },
                },
                PdkLayer {
                    id: self.met1.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 260,
                        space: 140,
                        offset: 130,
                        endcap: 100,
                    },
                },
                PdkLayer {
                    id: self.met2.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 400,
                        space: 460,
                        offset: 150,
                        endcap: 130,
                    },
                },
                PdkLayer {
                    id: self.met3.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 400,
                        space: 400,
                        offset: 200,
                        endcap: 150,
                    },
                },
                PdkLayer {
                    id: self.met4.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 1_200,
                        space: 950,
                        offset: 600,
                        endcap: 200,
                    },
                },
                PdkLayer {
                    id: self.met5.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 1_800,
                        space: 1_800,
                        offset: 900,
                        endcap: 600,
                    },
                },
            ],
            offset_x: 0,
            offset_y: 0,
        }
    }
}

pub struct Sky130ViaMaker;

impl ViaMaker<Sky130Pdk> for Sky130ViaMaker {
    fn draw_via(
        &self,
        cell: &mut CellBuilder<Sky130Pdk>,
        track_coord: TrackCoord,
    ) -> substrate::error::Result<()> {
        let stack = cell.ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..track_coord.layer + 1);
        let top_layer = stack.layer(track_coord.layer);
        let bot_layer = stack.layer(track_coord.layer - 1);

        let via_center = grid.xy_track_point(track_coord.layer, track_coord.x, track_coord.y);

        let (via_layer, bot_rect, via_rect, top_rect) = match track_coord.layer {
            1 => (
                cell.ctx.layers.mcon.drawing.id(),
                Rect::from_sides(0, 0, 170, 170),
                Rect::from_sides(0, 0, 170, 170),
                Rect::from_sides(-60, -45, 230, 215),
            ),
            2 => (
                cell.ctx.layers.via.drawing.id(),
                Rect::from_sides(-125, -55, 275, 205),
                Rect::from_sides(0, 0, 150, 150),
                Rect::from_sides(-125, -55, 275, 205),
            ),
            _ => todo!(),
        };
        let translation = via_center - via_rect.center();

        for (layer, shape) in [
            (bot_layer.id, bot_rect),
            (via_layer, via_rect),
            (top_layer.id, top_rect),
        ] {
            cell.draw(Shape::new(layer, shape).translate(translation))?;
        }
        Ok(())
    }
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

/// The IO of an NMOS or PMOS tile.
#[derive(Debug, Clone, Io)]
pub struct MosTileIo {
    /// `NF + 1` source/drain contacts on li1, where `NF` is the number of fingers.
    pub sd: InOut<Array<Signal>>,
    /// `NF` gate contacts on li1, where `NF` is the number of fingers.
    pub g: Input<Signal>,
    /// A body port on either nwell or pwell.
    pub b: InOut<Signal>,
}

/// A tile containing a set of MOS transistors.
///
/// There are `nf` transistors, each of length `len` and width `w`.
/// The gates of all transistors are connected.
/// The `nf+1` sources and drains are not connected to anything else.
///
/// This tile does not contain internal taps.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct MosTile {
    /// Transistor width.
    w: i64,

    /// Transistor length.
    len: MosLength,

    /// Number of fingers.
    nf: i64,
}

impl MosTile {
    /// Create a new MOS tile with the given physical transistor dimensions.
    fn new(w: i64, len: MosLength, nf: i64) -> Self {
        Self { w, len, nf }
    }
}

impl Block for MosTile {
    type Io = MosTileIo;
    fn id() -> ArcStr {
        arcstr::literal!("mos_tile")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("mos_tile_w{}_l{}_nf{}", self.w, self.len.nm(), self.nf)
    }

    fn io(&self) -> Self::Io {
        MosTileIo {
            sd: InOut(Array::new(self.nf as usize + 1, Signal::new())),
            g: Input(Signal::new()),
            b: InOut(Signal::new()),
        }
    }
}

impl ExportsLayoutData for MosTile {
    type LayoutData = ();
}

struct MosTileData {
    diff: Rect,
    lcm_bbox: Rect,
}

impl MosTile {
    fn layout(
        &self,
        io: &mut substrate::io::layout::Builder<MosTileIo>,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<MosTileData> {
        let stack = cell.ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..2);

        let tracks = (0..self.nf + 1)
            .map(|i| {
                let span = grid.track_span(0, i);
                Rect::from_spans(span, Span::new(-10, self.w + 10))
            })
            .collect::<Vec<_>>();

        let gates = tracks
            .windows(2)
            .map(|tracks| {
                let (left, right) = (tracks[0], tracks[1]);
                Rect::from_spans(
                    Span::new(left.right(), right.left()),
                    Span::new(-200, self.w + 130),
                )
                .shrink_dir(Dir::Horiz, 55)
                .unwrap()
            })
            .collect::<Vec<_>>();

        for &rect in gates.iter() {
            cell.draw(Shape::new(cell.ctx.layers.poly, rect))?;
        }

        for (i, &rect) in tracks.iter().enumerate() {
            io.sd[i].push(IoShape::with_layers(cell.ctx.layers.li1, rect));
            cell.draw(Shape::new(cell.ctx.layers.li1, rect))?;
            let num_cuts = (self.w + 20 - 160 + 170) / 340;
            for j in 0..num_cuts {
                let base = rect.bot() + 10 + 80 + 340 * j;
                let cut = Rect::from_spans(rect.hspan(), Span::with_start_and_length(base, 170));
                cell.draw(Shape::new(cell.ctx.layers.licon1, cut))?;
            }
        }

        let diff = Rect::from_sides(
            tracks[0].left() - 130,
            0,
            tracks.last().unwrap().right() + 130,
            self.w,
        );
        cell.draw(Shape::new(cell.ctx.layers.diff, diff))?;

        let poly = Rect::from_sides(
            gates[0].left(),
            -200 - 350,
            gates.last().unwrap().right(),
            -200,
        );
        cell.draw(Shape::new(cell.ctx.layers.poly, poly))?;

        let trk = grid.tracks(1).to_track_idx(poly.bot(), RoundingMode::Down);
        let bot = grid.tracks(1).track(trk).center() - 100;
        let poly_li = Rect::from_sides(
            tracks[1].left(),
            bot,
            tracks[tracks.len() - 2].right(),
            poly.top(),
        );
        cell.draw(Shape::new(cell.ctx.layers.li1, poly_li))?;
        io.g.push(IoShape::with_layers(cell.ctx.layers.li1, poly_li));
        let npc = Rect::from_spans(
            poly_li.hspan(),
            Span::new(poly_li.top(), poly_li.top() - 350),
        )
        .expand_dir(Dir::Vert, 10)
        .expand_dir(Dir::Horiz, 100);
        cell.draw(Shape::new(cell.ctx.layers.npc, npc))?;

        #[allow(clippy::needless_range_loop)]
        for i in 1..self.nf as usize {
            let cut = Rect::from_spans(
                tracks[i].hspan(),
                Span::new(poly_li.top() - 90, poly_li.top() - 260),
            );
            cell.draw(Shape::new(cell.ctx.layers.licon1, cut))?;
        }

        let virtual_layers = cell.ctx.install_layers::<atoll::VirtualLayers>();
        let slice = stack.slice(0..2);
        let bbox = cell.bbox().unwrap();
        let lcm_bbox = slice.lcm_to_physical_rect(slice.expand_to_lcm_units(bbox));
        cell.draw(Shape::new(virtual_layers.outline, lcm_bbox))?;

        Ok(MosTileData { diff, lcm_bbox })
    }
}

impl ExportsNestedData for MosTile {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for MosTile {
    fn schematic(
        &self,
        io: &substrate::io::schematic::Bundle<MosTileIo>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.nf as usize {
            cell.instantiate_connected(
                Nfet01v8::new(MosParams {
                    w: self.w,
                    nf: 1,
                    l: self.len.nm(),
                }),
                MosIoSchematic {
                    d: io.sd[i],
                    g: io.g,
                    s: io.sd[i + 1],
                    b: io.b,
                },
            )
        }
        Ok(())
    }
}

/// A tile containing a set of NMOS transistors.
///
/// See [`MosTile`] for more information.
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
}

impl Block for NmosTile {
    type Io = MosTileIo;
    fn id() -> ArcStr {
        arcstr::literal!("nmos_tile")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("n{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        self.tile.io()
    }
}

impl ExportsLayoutData for NmosTile {
    type LayoutData = ();
}

impl Layout<Sky130Pdk> for NmosTile {
    fn layout(
        &self,
        io: &mut substrate::io::layout::Builder<MosTileIo>,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let data = self.tile.layout(io, cell)?;
        let nsdm = data.diff.expand_all(130);
        let nsdm = nsdm.with_hspan(data.lcm_bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(cell.ctx.layers.nsdm, nsdm))?;

        cell.draw(Shape::new(cell.ctx.layers.pwell, data.lcm_bbox))?;
        io.b.push(IoShape::with_layers(cell.ctx.layers.pwell, data.lcm_bbox));

        Ok(())
    }
}

impl ExportsNestedData for NmosTile {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for NmosTile {
    fn schematic(
        &self,
        io: &substrate::io::schematic::Bundle<MosTileIo>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.tile.nf as usize {
            cell.instantiate_connected(
                Nfet01v8::new(MosParams {
                    w: self.tile.w,
                    nf: 1,
                    l: self.tile.len.nm(),
                }),
                MosIoSchematic {
                    d: io.sd[i],
                    g: io.g,
                    s: io.sd[i + 1],
                    b: io.b,
                },
            )
        }
        Ok(())
    }
}

/// A tile containing a set of PMOS transistors.
///
/// See [`MosTile`] for more information.
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
}

impl Block for PmosTile {
    type Io = MosTileIo;
    fn id() -> ArcStr {
        arcstr::literal!("pmos_tile")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        self.tile.io()
    }
}

impl ExportsLayoutData for PmosTile {
    type LayoutData = ();
}

impl Layout<Sky130Pdk> for PmosTile {
    fn layout(
        &self,
        io: &mut substrate::io::layout::Builder<MosTileIo>,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let data = self.tile.layout(io, cell)?;
        let psdm = data.diff.expand_all(130);
        let psdm = psdm.with_hspan(data.lcm_bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(cell.ctx.layers.psdm, psdm))?;

        let nwell = data.diff.expand_all(180).union(data.lcm_bbox);
        cell.draw(Shape::new(cell.ctx.layers.nwell, nwell))?;
        io.b.push(IoShape::with_layers(cell.ctx.layers.nwell, nwell));

        Ok(())
    }
}

impl ExportsNestedData for PmosTile {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for PmosTile {
    fn schematic(
        &self,
        io: &substrate::io::schematic::Bundle<MosTileIo>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.tile.nf as usize {
            cell.instantiate_connected(
                Pfet01v8::new(MosParams {
                    w: self.tile.w,
                    nf: 1,
                    l: self.tile.len.nm(),
                }),
                MosIoSchematic {
                    d: io.sd[i],
                    g: io.g,
                    s: io.sd[i + 1],
                    b: io.b,
                },
            )
        }
        Ok(())
    }
}

struct TapTileData {
    li: Rect,
    tap: Rect,
    lcm_bbox: Rect,
}

/// A tile containing taps.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct TapTile {
    /// x dimension, in number of li1 tracks
    xtracks: i64,
    /// y dimension, in number of m1 tracks
    ytracks: i64,
}

impl TapTile {
    /// Create a new tap tile with the given dimensions.
    fn new(xtracks: i64, ytracks: i64) -> Self {
        Self { xtracks, ytracks }
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("tap_tile_x{}_y{}", self.xtracks, self.ytracks)
    }
}

impl TapTile {
    fn layout(&self, cell: &mut CellBuilder<Sky130Pdk>) -> substrate::error::Result<TapTileData> {
        let stack = cell.ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..2);

        let li_hspan = grid
            .track_span(0, 0)
            .union(grid.track_span(0, self.xtracks - 1));
        let li_vspan = Span::new(
            grid.track_span(1, 0).center(),
            grid.track_span(1, self.ytracks - 1).center(),
        )
        .expand_all(85);
        let inner = Rect::from_spans(li_hspan, li_vspan);
        let li = inner.expand_dir(Dir::Horiz, 80);
        cell.draw(Shape::new(cell.ctx.layers.li1, li))?;

        for x in 0..self.xtracks {
            for y in 0..self.ytracks {
                let cut = Rect::from_spans(
                    grid.track_span(0, x),
                    Span::from_center_span(grid.track_span(1, y).center(), 170),
                );
                cell.draw(Shape::new(cell.ctx.layers.licon1, cut))?;
            }
        }

        let tap = inner.expand_dir(Dir::Vert, 65).expand_dir(Dir::Horiz, 120);
        cell.draw(Shape::new(cell.ctx.layers.tap, tap))?;

        let virtual_layers = cell.ctx.install_layers::<atoll::VirtualLayers>();
        let slice = stack.slice(0..2);
        let bbox = cell.bbox().unwrap();
        let lcm_bbox = slice.lcm_to_physical_rect(slice.expand_to_lcm_units(bbox));
        cell.draw(Shape::new(virtual_layers.outline, lcm_bbox))?;

        Ok(TapTileData { li, tap, lcm_bbox })
    }
}

/// A tile containing an N+ tap for biasing an N-well.
/// These can be used to connect to the body terminals of PMOS devices.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NtapTile {
    tile: TapTile,
}

impl NtapTile {
    /// Create a new ntap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self {
            tile: TapTile::new(xtracks, ytracks),
        }
    }
}

/// The IO of an [`NtapTile`].
#[derive(Io, Clone, Default, Debug)]
pub struct NtapIo {
    pub vpb: InOut<Signal>,
}

impl Block for NtapTile {
    type Io = NtapIo;

    fn id() -> ArcStr {
        arcstr::literal!("ntap_tile")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("n{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for NtapTile {
    type NestedData = ();
}

impl ExportsLayoutData for NtapTile {
    type LayoutData = ();
}

impl Schematic<Sky130Pdk> for NtapTile {
    fn schematic(
        &self,
        _io: &Bundle<<Self as Block>::Io>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        Ok(())
    }
}

impl Layout<Sky130Pdk> for NtapTile {
    fn layout(
        &self,
        io: &mut substrate::io::layout::Builder<NtapIo>,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let data = self.tile.layout(cell)?;
        io.vpb
            .push(IoShape::with_layers(cell.ctx.layers.li1, data.li));

        let nsdm = data.tap.expand_all(130);
        let nsdm = nsdm.with_hspan(data.lcm_bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(cell.ctx.layers.nsdm, nsdm))?;

        let nwell = data.tap.expand_all(180);
        let nwell = nwell
            .with_hspan(data.lcm_bbox.hspan().union(nwell.hspan()))
            .with_vspan(data.lcm_bbox.vspan().union(nwell.vspan()));
        cell.draw(Shape::new(cell.ctx.layers.nwell, nwell))?;

        Ok(())
    }
}

/// A tile containing a P+ tap for biasing an P-well or P-substrate.
/// These can be used to connect to the body terminals of NMOS devices.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PtapTile {
    tile: TapTile,
}

impl PtapTile {
    /// Create a new ntap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self {
            tile: TapTile::new(xtracks, ytracks),
        }
    }
}

/// The IO of a [`PtapTile`].
#[derive(Io, Clone, Default, Debug)]
pub struct PtapIo {
    pub vnb: InOut<Signal>,
}

impl Block for PtapTile {
    type Io = crate::atoll::PtapIo;

    fn id() -> ArcStr {
        arcstr::literal!("ptap_tile")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for PtapTile {
    type NestedData = ();
}

impl ExportsLayoutData for PtapTile {
    type LayoutData = ();
}

impl Schematic<Sky130Pdk> for PtapTile {
    fn schematic(
        &self,
        _io: &Bundle<<Self as Block>::Io>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        Ok(())
    }
}

impl Layout<Sky130Pdk> for PtapTile {
    fn layout(
        &self,
        io: &mut substrate::io::layout::Builder<PtapIo>,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let data = self.tile.layout(cell)?;
        io.vnb
            .push(IoShape::with_layers(cell.ctx.layers.li1, data.li));

        let psdm = data.tap.expand_all(130);
        let psdm = psdm.with_hspan(data.lcm_bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(cell.ctx.layers.psdm, psdm))?;
        Ok(())
    }
}
