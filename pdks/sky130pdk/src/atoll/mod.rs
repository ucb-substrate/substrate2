use crate::layers::Sky130Layers;
use crate::Sky130Pdk;
use arcstr::ArcStr;
use atoll::grid::{AbstractLayer, AtollLayer, DebugRoutingGrid, LayerStack, PdkLayer, RoutingGrid};
use atoll::RoutingDir;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::dir::Dir;
use substrate::geometry::rect::Rect;
use substrate::geometry::span::Span;
use substrate::io::{Array, InOut, Input, Io, IoShape, LayoutType, MosIoSchematic, SchematicType, Signal};
use substrate::layout::element::Shape;
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::pdk::layers::{Layer, LayerId};
use substrate::schematic::{ExportsNestedData, Schematic};
use crate::mos::{MosParams, Nfet01v8};

#[derive(Clone)]
pub struct Sky130AtollLayer(PdkLayer);

impl AsRef<LayerId> for Sky130AtollLayer {
    fn as_ref(&self) -> &LayerId {
        self.0.as_ref()
    }
}

impl Deref for Sky130AtollLayer {
    type Target = PdkLayer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sky130AtollLayer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AtollLayer for Sky130AtollLayer {
    fn dir(&self) -> RoutingDir {
        self.0.dir()
    }

    fn line(&self) -> i64 {
        self.0.line()
    }

    fn space(&self) -> i64 {
        self.0.space()
    }

    fn offset(&self) -> i64 {
        self.0.offset()
    }
}

impl From<PdkLayer> for Sky130AtollLayer {
    fn from(value: PdkLayer) -> Self {
        Self(value)
    }
}

impl From<Sky130AtollLayer> for PdkLayer {
    fn from(value: Sky130AtollLayer) -> Self {
        value.0
    }
}

impl Sky130Layers {
    /// Returns the ATOLL-compatible routing layer stack.
    pub fn atoll_layer_stack(&self) -> LayerStack<Sky130AtollLayer> {
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
                        endcap: 0,
                    },
                }
                .into(),
                PdkLayer {
                    id: self.met1.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 260,
                        space: 140,
                        offset: 130,
                        endcap: 100,
                    },
                }
                .into(),
                PdkLayer {
                    id: self.met2.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 400,
                        space: 460,
                        offset: 150,
                        endcap: 130,
                    },
                }
                .into(),
                PdkLayer {
                    id: self.met3.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 400,
                        space: 400,
                        offset: 200,
                        endcap: 150,
                    },
                }
                .into(),
                PdkLayer {
                    id: self.met4.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 1_200,
                        space: 950,
                        offset: 600,
                        endcap: 200,
                    },
                }
                .into(),
                PdkLayer {
                    id: self.met5.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 1_800,
                        space: 1_800,
                        offset: 900,
                        endcap: 600,
                    },
                }
                .into(),
            ],
            offset_x: 0,
            offset_y: 0,
        }
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

/// The IO of a [`MosTile`].
#[derive(Debug, Clone, Io)]
pub struct MosTileIo {
    /// `NF + 1` source/drain contacts on li1, where `NF` is the number of fingers.
    pub sd: InOut<Array<Signal>>,
    /// `NF` gate contacts on li1, where `NF` is the number of fingers.
    pub g: Input<Signal>,
    /// A body port on either nwell or pwell.
    pub b: InOut<Signal>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MosTile {
    pub w: i64,
    pub len: MosLength,
    pub nf: i64,
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

impl Layout<Sky130Pdk> for MosTile {
    fn layout(
        &self,
        io: &mut <<Self as Block>::Io as LayoutType>::Builder,
        cell: &mut CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let stack = cell
            .ctx
            .get_installation::<LayerStack<Sky130AtollLayer>>()
            .unwrap();
        let grid = RoutingGrid::new((*stack).clone(), 0..2, self.nf + 3, 4);
        let debug = DebugRoutingGrid::new(grid.clone());
        // cell.draw(debug)?;

        let tracks = (0..self.nf + 1)
            .map(|i| {
                let span = grid.track_span(0, i);
                Rect::from_spans(span, Span::new(-20, self.w + 20))
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
            let num_cuts = (self.w + 40 - 160 + 170) / 340;
            for j in 0..num_cuts {
                let base = rect.bot() + 20 + 80 + 340 * j;
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
        let nsdm = diff.expand_all(130);
        cell.draw(Shape::new(cell.ctx.layers.nsdm, nsdm))?;

        let poly = Rect::from_sides(
            gates[0].left(),
            -200 - 350,
            gates.last().unwrap().right(),
            -200,
        );
        cell.draw(Shape::new(cell.ctx.layers.poly, poly))?;

        let poly_li = Rect::from_sides(
            tracks[1].left(),
            poly.bot(),
            tracks[tracks.len() - 2].right(),
            poly.top(),
        );
        cell.draw(Shape::new(cell.ctx.layers.li1, poly_li))?;
        io.g.push(IoShape::with_layers(cell.ctx.layers.li1, poly_li));
        let npc = poly_li
            .expand_dir(Dir::Vert, 10)
            .expand_dir(Dir::Horiz, 100);
        cell.draw(Shape::new(cell.ctx.layers.npc, npc))?;

        for i in 1..self.nf as usize {
            let cut = Rect::from_spans(tracks[i].hspan(), poly_li.shrink_all(90).unwrap().vspan());
            cell.draw(Shape::new(cell.ctx.layers.licon1, cut))?;
        }

        let pwell = cell.bbox().unwrap();
        cell.draw(Shape::new(cell.ctx.layers.pwell, pwell))?;
        io.b.push(IoShape::with_layers(cell.ctx.layers.pwell, pwell));

        Ok(())
    }
}

impl ExportsNestedData for MosTile { type NestedData = (); }

impl Schematic<Sky130Pdk> for MosTile {
    fn schematic(&self, io: &<<Self as Block>::Io as SchematicType>::Bundle, cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>) -> substrate::error::Result<Self::NestedData> {
        for i in 0..self.nf as usize {
            cell.instantiate_connected(Nfet01v8::new(MosParams {
                w: self.w,
                nf: 1,
                l: self.len.nm(),
            }), MosIoSchematic {
                d: io.sd[i],
                g: io.g,
                s: io.sd[i+1],
                b: io.b,
            })
        }
        Ok(())
    }
}