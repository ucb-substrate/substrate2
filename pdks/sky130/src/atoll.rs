//! SKY130 primitives for [ATOLL](atoll).

use crate::layers::Sky130Layer;
use crate::mos::{MosParams, Nfet01v8, Pfet01v8};
use crate::Sky130;
use arcstr::ArcStr;
use atoll::abs::TrackCoord;
use atoll::grid::{AbstractLayer, LayerStack, PdkLayer, RoutingGrid, TrackOffset};
use atoll::route::ViaMaker;
use atoll::{AtollPrimitive, RoutingDir};
use geometry_macros::{TransformRef, TranslateRef};
use serde::{Deserialize, Serialize};

use layir::Shape;
use substrate::block::Block;
use substrate::context::Context;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::dir::Dir;
use substrate::geometry::rect::Rect;
use substrate::geometry::span::Span;
use substrate::geometry::transform::Translate;
use substrate::layout::tracks::RoundingMode;
use substrate::layout::{CellBuilder, Layout};
use substrate::schematic::Schematic;
use substrate::types::codegen::{PortGeometryBundle, View};
use substrate::types::layout::{PortGeometry, PortGeometryBuilder};
use substrate::types::schematic::NodeBundle;
use substrate::types::{Array, ArrayBundle, InOut, Input, Io, MosIo, Signal};

impl Sky130 {
    /// Returns the ATOLL-compatible routing layer stack.
    pub fn atoll_layer_stack() -> LayerStack<PdkLayer<Sky130Layer>> {
        LayerStack {
            layers: vec![
                PdkLayer {
                    layer: Sky130Layer::Li1,
                    inner: AbstractLayer {
                        dir: RoutingDir::Any {
                            track_dir: Dir::Vert,
                        },
                        line: 170,
                        space: 260,
                        offset: TrackOffset::None,
                        endcap: 85,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
                PdkLayer {
                    layer: Sky130Layer::Met1,
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 400,
                        space: 140,
                        offset: TrackOffset::None,
                        endcap: 85,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
                PdkLayer {
                    layer: Sky130Layer::Met2,
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 260,
                        space: 170,
                        offset: TrackOffset::None,
                        endcap: 130,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
                PdkLayer {
                    layer: Sky130Layer::Met3,
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 400,
                        space: 410,
                        offset: TrackOffset::None,
                        endcap: 200,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
                PdkLayer {
                    layer: Sky130Layer::Met4,
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 1_200,
                        space: 950,
                        offset: TrackOffset::None,
                        endcap: 200,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
                PdkLayer {
                    layer: Sky130Layer::Met5,
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 1_800,
                        space: 1_800,
                        offset: TrackOffset::None,
                        endcap: 600,
                        via_spacing: 1,
                        strap_via_spacing: 1,
                    },
                },
            ],
            offset_x: 0,
            offset_y: 0,
        }
    }
}

/// A [`ViaMaker`] for SKY 130's ATOLL layer stack.
pub struct Sky130ViaMaker;

impl ViaMaker<Sky130Layer> for Sky130ViaMaker {
    fn draw_via(&self, ctx: Context, track_coord: TrackCoord) -> Vec<Shape<Sky130Layer>> {
        let stack = ctx
            .get_installation::<LayerStack<PdkLayer<Sky130Layer>>>()
            .unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..track_coord.layer + 1);
        let top_layer = stack.layer(track_coord.layer);
        let bot_layer = stack.layer(track_coord.layer - 1);

        let via_center = grid.xy_track_point(track_coord.layer, track_coord.x, track_coord.y);

        let (via_layer, bot_rect, via_rect, top_rect) = match track_coord.layer {
            1 => (
                Sky130Layer::Mcon,
                Rect::from_sides(0, 0, 170, 170),
                Rect::from_sides(0, 0, 170, 170),
                Rect::from_sides(-60, -115, 230, 285),
            ),
            2 => (
                Sky130Layer::Via,
                Rect::from_sides(-55, -125, 205, 275),
                Rect::from_sides(0, 0, 150, 150),
                Rect::from_sides(-55, -125, 205, 275),
            ),
            _ => todo!(),
        };
        let translation = via_center - via_rect.center();

        [
            (bot_layer.layer, bot_rect),
            (via_layer, via_rect),
            (top_layer.layer, top_rect),
        ]
        .into_iter()
        .map(|(layer, shape)| Shape::new(layer, shape).translate(translation))
        .collect()
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
    pub sd: Array<InOut<Signal>>,
    /// `NF` gate contacts on li1, where `NF` is the number of fingers.
    pub g: Array<Input<Signal>>,
    /// A body port on either nwell or pwell.
    pub b: InOut<Signal>,
}

/// Determines the connection direction of a transistor gate.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateDir {
    /// Connects the gate towards the right.
    #[default]
    Right,
    /// Connects the gate towards the left.
    Left,
}

/// A tile containing a set of MOS transistors.
///
/// There are `nf` transistors, each of length `len` and width `w`.
/// The gates of all transistors are connected.
/// The `nf+1` sources and drains are not connected to anything else.
///
/// This tile does not contain internal taps.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MosTile {
    /// Transistor width.
    w: i64,

    /// Transistor length.
    len: MosLength,

    /// Number of fingers.
    nf: i64,

    /// The connection direction of the left-most gate in the tile.
    ///
    /// A gate will always be connected with the gate adjacent to it
    /// in its connection direction.
    gate_dir: GateDir,
}

impl MosTile {
    /// Create a new MOS tile with the given physical transistor dimensions.
    pub fn new(w: i64, len: MosLength, nf: i64) -> Self {
        Self {
            w,
            len,
            nf,
            gate_dir: GateDir::default(),
        }
    }
}

impl Block for MosTile {
    type Io = MosTileIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("mos_tile_w{}_l{}_nf{}", self.w, self.len.nm(), self.nf)
    }

    fn io(&self) -> Self::Io {
        MosTileIo {
            sd: Array::new(self.nf as usize + 1, InOut(Signal::new())),
            g: Array::new(
                match self.gate_dir {
                    GateDir::Left => self.nf / 2 + 1,
                    GateDir::Right => (self.nf - 1) / 2 + 1,
                } as usize,
                Input(Signal::new()),
            ),

            b: InOut(Signal::new()),
        }
    }
}

/// MOS tile geometry.
#[derive(TranslateRef, TransformRef)]
pub struct MosTileData {
    diff: Rect,
    lcm_bbox: Rect,
}

impl MosTile {
    fn layout(
        &self,
        sd: &mut Vec<PortGeometry<Sky130Layer>>,
        g: &mut Vec<PortGeometry<Sky130Layer>>,
        cell: &mut CellBuilder<Sky130>,
    ) -> substrate::error::Result<MosTileData> {
        let stack = cell
            .ctx
            .get_installation::<LayerStack<PdkLayer<Sky130Layer>>>()
            .unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..2);

        let top_m1 = grid.tracks(1).to_track_idx(self.w + 10, RoundingMode::Up);
        let bot_m1 = grid.tracks(1).to_track_idx(-10, RoundingMode::Down);
        let gate_top_m1 = bot_m1 - 1;
        let gate_vspan = grid
            .track_span(1, gate_top_m1)
            .union(grid.track_span(1, gate_top_m1 - 1))
            .shrink_all(45);

        let tracks = (1..self.nf + 2)
            .map(|i| {
                let span = grid.track_span(0, i);
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

        for (i, &rect) in tracks.iter().enumerate() {
            let sd_rect = rect.with_vspan(
                grid.track_span(1, bot_m1)
                    .union(grid.track_span(1, top_m1))
                    .shrink_all(45),
            );
            let sd_shape = Shape::new(Sky130Layer::Li1, sd_rect);
            cell.draw(sd_shape.clone())?;
            sd.push(PortGeometry::new(sd_shape));
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
                let g_shape = Shape::new(Sky130Layer::Li1, poly_li);
                cell.draw(g_shape.clone())?;
                g.push(PortGeometry::new(g_shape));

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
                gate_spans[i].union(li_track.hspan()),
                Span::new(poly_li.top() - 350, poly_li.top()),
            );
            cell.draw(Shape::new(Sky130Layer::Poly, poly))?;
        }

        for &span in gate_spans.iter() {
            cell.draw(Shape::new(
                Sky130Layer::Poly,
                Rect::from_spans(span, Span::new(gate_vspan.stop() - 350, self.w + 130)),
            ))?;
        }

        let slice = stack.slice(0..2);
        let bbox = cell.bbox().unwrap();
        let lcm_bbox = slice.lcm_to_physical_rect(slice.expand_to_lcm_units(bbox));

        Ok(MosTileData { diff, lcm_bbox })
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
        self.tile.io()
    }
}

impl Layout for NmosTile {
    type Schema = Sky130;
    type Bundle = View<MosTileIo, PortGeometryBundle<Sky130>>;
    type Data = MosTileData;

    fn layout(
        &self,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let mut sd = Vec::new();
        let mut g = Vec::new();
        let data = self.tile.layout(&mut sd, &mut g, cell)?;
        let nsdm = data.diff.expand_all(130);
        let nsdm = nsdm.with_hspan(data.lcm_bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Nsdm, nsdm))?;

        let pwell_shape = Shape::new(Sky130Layer::Pwell, data.lcm_bbox);
        cell.draw(pwell_shape.clone())?;

        Ok((
            MosTileIoView {
                sd: ArrayBundle::new(Signal, sd),
                g: ArrayBundle::new(Signal, g),
                b: PortGeometry::new(pwell_shape),
            },
            data,
        ))
    }
}

impl AtollPrimitive for NmosTile {
    type Schema = Sky130;
    fn outline(cell: &substrate::layout::TransformedCell<Self>) -> Rect {
        cell.data().lcm_bbox
    }
}

impl Schematic for NmosTile {
    type Schema = Sky130;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.tile.nf as usize {
            cell.instantiate_connected(
                Nfet01v8::new(MosParams {
                    w: self.tile.w,
                    nf: 1,
                    l: self.tile.len.nm(),
                }),
                NodeBundle::<MosIo> {
                    d: io.sd[i],
                    g: io.g[match self.tile.gate_dir {
                        GateDir::Left => (i + 1) / 2,
                        GateDir::Right => i / 2,
                    }],
                    s: io.sd[i + 1],
                    b: io.b,
                },
            );
        }
        Ok(())
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
}

impl Block for PmosTile {
    type Io = MosTileIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        self.tile.io()
    }
}

impl Layout for PmosTile {
    type Schema = Sky130;
    type Bundle = View<MosTileIo, PortGeometryBundle<Sky130>>;
    type Data = MosTileData;

    fn layout(
        &self,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let mut sd = Vec::new();
        let mut g = Vec::new();
        let data = self.tile.layout(&mut sd, &mut g, cell)?;
        let psdm = data.diff.expand_all(130);
        let psdm = psdm.with_hspan(data.lcm_bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Psdm, psdm))?;

        let nwell = data.diff.expand_all(180).union(data.lcm_bbox);
        let nwell = nwell
            .with_hspan(data.lcm_bbox.hspan().union(nwell.hspan()))
            .with_vspan(data.lcm_bbox.vspan().union(nwell.vspan()));
        let nwell_shape = Shape::new(Sky130Layer::Nwell, nwell);
        cell.draw(nwell_shape.clone())?;

        Ok((
            MosTileIoView {
                sd: ArrayBundle::new(Signal, sd),
                g: ArrayBundle::new(Signal, g),
                b: PortGeometry::new(nwell_shape),
            },
            data,
        ))
    }
}

impl AtollPrimitive for PmosTile {
    type Schema = Sky130;
    fn outline(cell: &substrate::layout::TransformedCell<Self>) -> Rect {
        cell.data().lcm_bbox
    }
}

impl Schematic for PmosTile {
    type Schema = Sky130;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.tile.nf as usize {
            cell.instantiate_connected(
                Pfet01v8::new(MosParams {
                    w: self.tile.w,
                    nf: 1,
                    l: self.tile.len.nm(),
                }),
                NodeBundle::<MosIo> {
                    d: io.sd[i],
                    g: io.g[match self.tile.gate_dir {
                        GateDir::Left => (i + 1) / 2,
                        GateDir::Right => i / 2,
                    }],
                    s: io.sd[i + 1],
                    b: io.b,
                },
            );
        }
        Ok(())
    }
}

/// Tap tile geometry.
#[derive(TranslateRef, TransformRef)]
pub struct TapTileData {
    li: Rect,
    tap: Rect,
    lcm_bbox: Rect,
}

/// A tile containing taps.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TapTile {
    /// x dimension, in number of li1 tracks
    xtracks: i64,
    /// y dimension, in number of m1 tracks
    ytracks: i64,
}

impl TapTile {
    /// Create a new tap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self { xtracks, ytracks }
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("tap_tile_x{}_y{}", self.xtracks, self.ytracks)
    }
}

impl TapTile {
    fn layout(&self, cell: &mut CellBuilder<Sky130>) -> substrate::error::Result<TapTileData> {
        let stack = cell
            .ctx
            .get_installation::<LayerStack<PdkLayer<Sky130Layer>>>()
            .unwrap();
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
        cell.draw(Shape::new(Sky130Layer::Li1, li))?;

        for x in 0..self.xtracks {
            for y in 0..self.ytracks {
                let cut = Rect::from_spans(
                    grid.track_span(0, x),
                    Span::from_center_span(grid.track_span(1, y).center(), 170),
                );
                cell.draw(Shape::new(Sky130Layer::Licon1, cut))?;
            }
        }

        let tap = inner.expand_dir(Dir::Vert, 65).expand_dir(Dir::Horiz, 120);
        cell.draw(Shape::new(Sky130Layer::Tap, tap))?;

        let slice = stack.slice(0..2);
        let bbox = cell.bbox().unwrap();
        let lcm_bbox = slice.lcm_to_physical_rect(slice.expand_to_lcm_units(bbox));

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
    /// The n-well net.
    pub vpb: InOut<Signal>,
}

impl Block for NtapTile {
    type Io = NtapIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("n{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Layout for NtapTile {
    type Schema = Sky130;
    type Bundle = View<NtapIo, PortGeometryBundle<Sky130>>;
    type Data = TapTileData;

    fn layout(
        &self,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let data = self.tile.layout(cell)?;
        let li_shape = Shape::new(Sky130Layer::Li1, data.li);
        cell.draw(li_shape.clone())?;

        let nsdm = data.tap.expand_all(130);
        let nsdm = nsdm.with_hspan(data.lcm_bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Nsdm, nsdm))?;

        let nwell = data.tap.expand_all(180);
        let nwell = nwell
            .with_hspan(data.lcm_bbox.hspan().union(nwell.hspan()))
            .with_vspan(data.lcm_bbox.vspan().union(nwell.vspan()));
        let nwell_shape = Shape::new(Sky130Layer::Nwell, nwell);
        cell.draw(nwell_shape.clone())?;

        Ok((
            NtapIoView {
                vpb: PortGeometry::new(li_shape),
            },
            data,
        ))
    }
}

impl AtollPrimitive for NtapTile {
    type Schema = Sky130;
    fn outline(cell: &substrate::layout::TransformedCell<Self>) -> Rect {
        cell.data().lcm_bbox
    }
}

impl Schematic for NtapTile {
    type Schema = Sky130;
    type NestedData = ();

    fn schematic(
        &self,
        _io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
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
    /// The p-well net.
    pub vnb: InOut<Signal>,
}

impl Block for PtapTile {
    type Io = PtapIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Layout for PtapTile {
    type Schema = Sky130;
    type Bundle = View<PtapIo, PortGeometryBundle<Sky130>>;
    type Data = TapTileData;

    fn layout(
        &self,
        cell: &mut CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let data = self.tile.layout(cell)?;
        let li_shape = Shape::new(Sky130Layer::Li1, data.li);
        cell.draw(li_shape.clone())?;

        let psdm = data.tap.expand_all(130);
        let psdm = psdm.with_hspan(data.lcm_bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Psdm, psdm))?;

        Ok((
            PtapIoView {
                vnb: PortGeometry::new(li_shape),
            },
            data,
        ))
    }
}

impl AtollPrimitive for PtapTile {
    type Schema = Sky130;
    fn outline(cell: &substrate::layout::TransformedCell<Self>) -> Rect {
        cell.data().lcm_bbox
    }
}

impl Schematic for PtapTile {
    type Schema = Sky130;
    type NestedData = ();

    fn schematic(
        &self,
        _io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        Ok(())
    }
}
