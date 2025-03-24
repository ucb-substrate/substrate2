//! StrongARM latch layout generators.

use crate::tiles::{MosKind, MosTileParams, TapIo, TapTileParams, TileKind};
use atoll::route::{GreedyRouter, ViaMaker};
use atoll::{Orientation, Tile, TileBuilder};
use std::any::Any;
use std::marker::PhantomData;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::error::Result;
use substrate::geometry::align::AlignMode;
use substrate::geometry::bbox::Bbox;
use substrate::types::codegen::{PortGeometryBundle, View};
use substrate::types::layout::PortGeometryBuilder;
use substrate::types::schematic::NodeBundle;
use substrate::types::{DiffPair, DiffPairView, InOut, Input, Io, MosIo, Output, Signal};
use substrate::{layout, schematic};

// pub mod tb;
pub mod tech;
pub mod tiles;

/// The interface to a clocked differential comparator.
#[derive(Debug, Default, Clone, Io)]
pub struct ClockedDiffComparatorIo {
    /// The input differential pair.
    pub input: Input<DiffPair>,
    /// The output differential pair.
    pub output: Output<DiffPair>,
    /// The clock signal.
    pub clock: Input<Signal>,
    /// The VDD rail.
    pub vdd: InOut<Signal>,
    /// The VSS rail.
    pub vss: InOut<Signal>,
}

/// The input pair device kind of the comparator.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum InputKind {
    /// A comparator with an NMOS input pair.
    N,
    /// A comparator with a PMOS input pair.
    P,
}

impl InputKind {
    /// Returns true if the input kind is NMOS.
    pub fn is_n(&self) -> bool {
        matches!(self, InputKind::N)
    }

    /// Returns true if the input kind is PMOS.
    pub fn is_p(&self) -> bool {
        matches!(self, InputKind::P)
    }
}

/// The parameters of the [`StrongArm`] layout generator.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct StrongArmParams {
    /// The NMOS device flavor.
    pub nmos_kind: MosKind,
    /// The PMOS device flavor.
    pub pmos_kind: MosKind,
    /// The width of one half of the tail MOS device.
    pub half_tail_w: i64,
    /// The width of an input pair MOS device.
    pub input_pair_w: i64,
    /// The width of the inverter MOS devices connected to the input pair.
    pub inv_input_w: i64,
    /// The width of the inverter MOS devices connected to the precharge devices.
    pub inv_precharge_w: i64,
    /// The width of the precharge MOS devices.
    pub precharge_w: i64,
    /// The kind of the input pair MOS devices.
    pub input_kind: InputKind,
}

/// A StrongARM latch implementation.
pub trait StrongArmImpl: Any {
    type Schema: layout::schema::Schema + schematic::schema::Schema;
    /// The MOS tile.
    type MosTile: Tile<Schema = Self::Schema, LayoutBundle = View<MosIo, PortGeometryBundle<Self::Schema>>>
        + Block<Io = MosIo>
        + Clone;
    /// The tap tile.
    type TapTile: Tile<Schema = Self::Schema, LayoutBundle = View<TapIo, PortGeometryBundle<Self::Schema>>>
        + Block<Io = TapIo>
        + Clone;
    /// A PDK-specific via maker.
    type ViaMaker: ViaMaker<<Self::Schema as layout::schema::Schema>::Layer>;

    /// Creates an instance of the MOS tile.
    fn mos(params: MosTileParams) -> Self::MosTile;
    /// Creates an instance of the tap tile.
    fn tap(params: TapTileParams) -> Self::TapTile;
    /// Creates a PDK-specific via maker.
    fn via_maker() -> Self::ViaMaker;
    /// Additional layout hooks to run after the strongARM layout is complete.
    fn post_layout_hooks(_cell: &mut TileBuilder<'_, Self::Schema>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Io)]
struct StrongArmHalfIo {
    /// Ports that are exposed at the top level of a StrongARM.
    top_io: InOut<ClockedDiffComparatorIo>,
    /// Drains of input pair.
    input_d: InOut<DiffPair>,
    /// Drain of tail.
    tail_d: InOut<Signal>,
}

#[derive_where::derive_where(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct StrongArmHalf<T>(StrongArmParams, PhantomData<fn() -> T>);

impl<T> StrongArmHalf<T> {
    fn new(params: StrongArmParams) -> Self {
        Self(params, PhantomData)
    }
}

impl<T: Any> Block for StrongArmHalf<T> {
    type Io = StrongArmHalfIo;

    // todo: include parameters in name
    fn name(&self) -> ArcStr {
        substrate::arcstr::literal!("strong_arm_half")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl<T: StrongArmImpl> Tile for StrongArmHalf<T> {
    type Schema = <T as StrongArmImpl>::Schema;
    type NestedData = ();
    type LayoutBundle = View<StrongArmHalfIo, PortGeometryBundle<Self::Schema>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<(
        Self::NestedData,
        Self::LayoutBundle,
        Self::LayoutData,
        substrate::geometry::prelude::Rect,
    )> {
        let (
            input_kind,
            precharge_kind,
            input_flavor,
            precharge_flavor,
            input_rail,
            precharge_rail,
        ) = match self.0.input_kind {
            InputKind::N => (
                TileKind::N,
                TileKind::P,
                self.0.nmos_kind,
                self.0.pmos_kind,
                io.top_io.vss,
                io.top_io.vdd,
            ),
            InputKind::P => (
                TileKind::P,
                TileKind::N,
                self.0.pmos_kind,
                self.0.nmos_kind,
                io.top_io.vdd,
                io.top_io.vss,
            ),
        };
        let half_tail_params = MosTileParams::new(input_flavor, input_kind, self.0.half_tail_w);
        let input_pair_params = MosTileParams::new(input_flavor, input_kind, self.0.input_pair_w);
        let inv_input_params = MosTileParams::new(input_flavor, input_kind, self.0.inv_input_w);
        let inv_precharge_params =
            MosTileParams::new(precharge_flavor, precharge_kind, self.0.inv_precharge_w);
        let precharge_params =
            MosTileParams::new(precharge_flavor, precharge_kind, self.0.precharge_w);

        let tail = io.tail_d;
        let intn = io.input_d.n;
        let intp = cell.signal("intp", Signal);

        let mut tail_dummy = cell.generate_connected(
            T::mos(half_tail_params),
            NodeBundle::<MosIo> {
                d: input_rail,
                g: input_rail,
                s: input_rail,
                b: input_rail,
            },
        );
        let mut tail_pair = (0..2)
            .map(|_| {
                cell.generate_connected(
                    T::mos(half_tail_params),
                    NodeBundle::<MosIo> {
                        d: tail,
                        g: io.top_io.clock,
                        s: input_rail,
                        b: input_rail,
                    },
                )
            })
            .collect::<Vec<_>>();

        let mut ptap = cell.generate(T::tap(TapTileParams::new(TileKind::P, 3)));
        let ntap = cell.generate(T::tap(TapTileParams::new(TileKind::N, 3)));
        cell.connect(ptap.io().x, io.top_io.vss);
        cell.connect(ntap.io().x, io.top_io.vdd);

        let mut input_pair = (0..2)
            .map(|i| {
                cell.generate_connected(
                    T::mos(input_pair_params),
                    NodeBundle::<MosIo> {
                        d: if i == 0 { intn } else { intp },
                        g: if i == 0 {
                            io.top_io.input.p
                        } else {
                            io.top_io.input.n
                        },
                        s: tail,
                        b: input_rail,
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut input_dummy = cell.generate_connected(
            T::mos(input_pair_params),
            NodeBundle::<MosIo> {
                d: input_rail,
                g: input_rail,
                s: input_rail,
                b: input_rail,
            },
        );
        let mut inv_input_pair = (0..2)
            .map(|i| {
                cell.generate_connected(
                    T::mos(inv_input_params),
                    if i == 0 {
                        NodeBundle::<MosIo> {
                            d: io.top_io.output.n,
                            g: io.top_io.output.p,
                            s: intn,
                            b: input_rail,
                        }
                    } else {
                        NodeBundle::<MosIo> {
                            d: io.top_io.output.p,
                            g: io.top_io.output.n,
                            s: intp,
                            b: input_rail,
                        }
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut inv_input_dummy = cell.generate_connected(
            T::mos(inv_input_params),
            NodeBundle::<MosIo> {
                d: input_rail,
                g: input_rail,
                s: input_rail,
                b: input_rail,
            },
        );
        let mut inv_precharge_pair = (0..2)
            .map(|i| {
                cell.generate_connected(
                    T::mos(inv_precharge_params),
                    NodeBundle::<MosIo> {
                        d: if i == 0 {
                            io.top_io.output.n
                        } else {
                            io.top_io.output.p
                        },
                        g: if i == 0 {
                            io.top_io.output.p
                        } else {
                            io.top_io.output.n
                        },
                        s: precharge_rail,
                        b: precharge_rail,
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut inv_precharge_dummy = cell.generate_connected(
            T::mos(inv_precharge_params),
            NodeBundle::<MosIo> {
                d: precharge_rail,
                g: precharge_rail,
                s: precharge_rail,
                b: precharge_rail,
            },
        );
        let mut precharge_pair_a = (0..2)
            .map(|i| {
                cell.generate_connected(
                    T::mos(precharge_params),
                    NodeBundle::<MosIo> {
                        d: if i == 0 {
                            io.top_io.output.n
                        } else {
                            io.top_io.output.p
                        },
                        g: io.top_io.clock,
                        s: precharge_rail,
                        b: precharge_rail,
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut precharge_pair_a_dummy = cell.generate_connected(
            T::mos(precharge_params),
            NodeBundle::<MosIo> {
                d: precharge_rail,
                g: precharge_rail,
                s: precharge_rail,
                b: precharge_rail,
            },
        );
        let mut precharge_pair_b = (0..2)
            .map(|i| {
                cell.generate_connected(
                    T::mos(precharge_params),
                    NodeBundle::<MosIo> {
                        d: if i == 0 { intn } else { intp },
                        g: io.top_io.clock,
                        s: precharge_rail,
                        b: precharge_rail,
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut precharge_pair_b_dummy = cell.generate_connected(
            T::mos(precharge_params),
            NodeBundle::<MosIo> {
                d: precharge_rail,
                g: precharge_rail,
                s: precharge_rail,
                b: precharge_rail,
            },
        );

        let mut prev = ntap.lcm_bounds();

        let mut rows = [
            (&mut precharge_pair_a_dummy, &mut precharge_pair_a),
            (&mut precharge_pair_b_dummy, &mut precharge_pair_b),
            (&mut inv_precharge_dummy, &mut inv_precharge_pair),
            (&mut inv_input_dummy, &mut inv_input_pair),
            (&mut input_dummy, &mut input_pair),
            (&mut tail_dummy, &mut tail_pair),
        ];

        if self.0.input_kind == InputKind::P {
            rows.reverse();
        }

        for (dummy, mos_pair) in rows {
            dummy.align_rect_mut(prev, AlignMode::Left, 0);
            dummy.align_rect_mut(prev, AlignMode::Beneath, 0);
            prev = dummy.lcm_bounds();
            mos_pair[0].align_rect_mut(prev, AlignMode::Bottom, 0);
            mos_pair[0].align_rect_mut(prev, AlignMode::ToTheRight, 0);
            let left_rect = mos_pair[0].lcm_bounds();
            mos_pair[1].align_rect_mut(left_rect, AlignMode::Bottom, 0);
            mos_pair[1].align_rect_mut(left_rect, AlignMode::ToTheRight, 0);
        }

        ptap.align_rect_mut(prev, AlignMode::Left, 0);
        ptap.align_rect_mut(prev, AlignMode::Beneath, 0);

        let ptap = cell.draw(ptap)?;
        let ntap = cell.draw(ntap)?;
        let tail_pair = tail_pair
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _tail_dummy = cell.draw(tail_dummy)?;
        let input_pair = input_pair
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _input_dummy = cell.draw(input_dummy)?;
        let inv_nmos_pair = inv_input_pair
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _inv_nmos_dummy = cell.draw(inv_input_dummy)?;
        let _inv_pmos_pair = inv_precharge_pair
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _inv_pmos_dummy = cell.draw(inv_precharge_dummy)?;
        let _precharge_pair_a = precharge_pair_a
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _precharge_pair_a_dummy = cell.draw(precharge_pair_a_dummy)?;
        let _precharge_pair_b = precharge_pair_b
            .into_iter()
            .map(|inst| cell.draw(inst))
            .collect::<Result<Vec<_>>>()?;
        let _precharge_pair_b_dummy = cell.draw(precharge_pair_b_dummy)?;

        cell.set_top_layer(2);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(T::via_maker());

        Ok((
            (),
            StrongArmHalfIoView {
                top_io: ClockedDiffComparatorIoView {
                    vdd: ntap.layout.io().x,
                    vss: ptap.layout.io().x,
                    clock: tail_pair[0].layout.io().g,
                    input: DiffPairView {
                        p: input_pair[0].layout.io().g,
                        n: input_pair[1].layout.io().g,
                    },
                    output: DiffPairView {
                        p: inv_nmos_pair[1].layout.io().d,
                        n: inv_nmos_pair[0].layout.io().d,
                    },
                },
                input_d: DiffPairView {
                    p: input_pair[0].layout.io().d,
                    n: input_pair[1].layout.io().d,
                },
                tail_d: tail_pair[0].layout.io().d,
            },
            (),
            cell.layout.bbox_rect(),
        ))
    }
}

/// A StrongARM latch.
// Layout assumes that PDK layer stack has a vertical layer 0.
#[derive_where::derive_where(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct StrongArm<T>(StrongArmParams, PhantomData<fn() -> T>);

impl<T> StrongArm<T> {
    /// Creates a new [`StrongArm`].
    pub const fn new(params: StrongArmParams) -> Self {
        Self(params, PhantomData)
    }
}

impl<T: Any> Block for StrongArm<T> {
    type Io = ClockedDiffComparatorIo;

    // todo: include parameters in name
    fn name(&self) -> ArcStr {
        substrate::arcstr::literal!("strong_arm")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl<T: StrongArmImpl> Tile for StrongArm<T> {
    type Schema = <T as StrongArmImpl>::Schema;
    type NestedData = ();
    type LayoutBundle = View<ClockedDiffComparatorIo, PortGeometryBundle<Self::Schema>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<(
        Self::NestedData,
        Self::LayoutBundle,
        Self::LayoutData,
        substrate::geometry::prelude::Rect,
    )> {
        let tail_d = cell.signal("tail_d", Signal::new());
        let input_d = cell.signal("input_d", DiffPair::default());
        let mut vdd = PortGeometryBuilder::new();
        let mut vss = PortGeometryBuilder::new();
        let mut clock = PortGeometryBuilder::new();
        let mut input_p = PortGeometryBuilder::new();
        let mut input_n = PortGeometryBuilder::new();
        let mut output_p = PortGeometryBuilder::new();
        let mut output_n = PortGeometryBuilder::new();

        let conn = NodeBundle::<StrongArmHalfIo> {
            top_io: io.clone(),
            input_d,
            tail_d,
        };
        let left_half = cell.generate_connected(StrongArmHalf::<T>::new(self.0), conn.clone());

        let right_half = cell
            .generate_connected(StrongArmHalf::<T>::new(self.0), conn)
            .orient(Orientation::ReflectHoriz)
            .align(&left_half, AlignMode::ToTheRight, 0);

        let left_half = cell.draw(left_half)?;
        let right_half = cell.draw(right_half)?;

        cell.set_top_layer(2);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(T::via_maker());

        vdd.merge(left_half.layout.io().top_io.vdd);
        vdd.merge(right_half.layout.io().top_io.vdd);
        vss.merge(left_half.layout.io().top_io.vss);
        vss.merge(right_half.layout.io().top_io.vss);
        clock.merge(left_half.layout.io().top_io.clock);
        clock.merge(right_half.layout.io().top_io.clock);

        input_p.merge(left_half.layout.io().top_io.input.p);
        input_p.merge(right_half.layout.io().top_io.input.p);
        input_n.merge(left_half.layout.io().top_io.input.n);
        input_n.merge(right_half.layout.io().top_io.input.n);
        output_p.merge(left_half.layout.io().top_io.output.p);
        output_p.merge(right_half.layout.io().top_io.output.p);
        output_n.merge(left_half.layout.io().top_io.output.n);
        output_n.merge(right_half.layout.io().top_io.output.n);

        T::post_layout_hooks(cell)?;

        Ok((
            (),
            ClockedDiffComparatorIoView {
                input: DiffPairView {
                    p: input_p.build()?,
                    n: input_n.build()?,
                },
                output: DiffPairView {
                    p: output_p.build()?,
                    n: output_n.build()?,
                },
                clock: clock.build()?,
                vdd: vdd.build()?,
                vss: vss.build()?,
            },
            (),
            cell.layout.bbox_rect(),
        ))
    }
}
