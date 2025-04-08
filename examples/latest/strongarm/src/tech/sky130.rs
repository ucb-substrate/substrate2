//! SKY130-specific implementations.

use crate::tiles::{MosTileParams, TapIo, TapIoView, TapTileParams, TileKind};
use crate::StrongArmImpl;
use atoll::resizing::ResizableInstance;
use atoll::route::GreedyRouter;
use atoll::{Tile, TileBuilder, TileData};
use layir::Shape;
use sky130::atoll::{MosLength, NmosTile, PmosTile, Sky130ViaMaker};
use sky130::Sky130;
use substrate::arcstr;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::geometry::dims::Dims;
use substrate::types::codegen::{PortGeometryBundle, View};
use substrate::types::layout::PortGeometryBuilder;
use substrate::types::{FlatLen, MosIo, MosIoView};

/// A SKY130 implementation.
pub struct Sky130Impl;

impl StrongArmImpl for Sky130Impl {
    const TAP_FREQ: i64 = 6_000;
    type Schema = Sky130;
    type MosTile = ResizeableMosTile;
    type TapTile = TapTile;
    type ViaMaker = Sky130ViaMaker;

    fn mos(params: MosTileParams) -> Self::MosTile {
        ResizeableMosTile::new(params.w, MosLength::L150, params.tile_kind)
    }
    fn tap(params: TapTileParams) -> Self::TapTile {
        TapTile::new(params)
    }
    fn via_maker() -> Self::ViaMaker {
        Sky130ViaMaker
    }
}

pub struct ResizeableMosTile {
    wxnf: i64,
    l: MosLength,
    kind: TileKind,
}

fn max_nf(w_max: i64) -> i64 {
    (w_max - 860) / 430
}

fn max_w(h_max: i64) -> i64 {
    (h_max / 540 - 1) * 540 - 20
}

impl ResizeableMosTile {
    fn new(wxnf: i64, l: MosLength, kind: TileKind) -> Self {
        Self { wxnf, l, kind }
    }
    fn max_nf(&self, w_max: i64) -> Option<i64> {
        let mut nf = max_nf(w_max);
        if nf > 0 {
            loop {
                if self.wxnf % nf == 0 {
                    break Some(nf);
                }
                nf -= 1;
            }
        } else {
            None
        }
    }
}

impl ResizableInstance for ResizeableMosTile {
    type Tile = MosTile;

    fn wh_increments(&self) -> substrate::geometry::prelude::Dims {
        Dims::new(430, 540)
    }

    fn tile(&self, dims: substrate::geometry::prelude::Dims) -> Self::Tile {
        let nf = self.max_nf(dims.w()).unwrap();
        let w = self.wxnf / nf;
        MosTile::new(w, nf, self.l, self.kind)
    }

    fn max_min_width(&self) -> i64 {
        (self.wxnf / 420 + 1) * 430 + 860
    }

    fn min_height(&self, w_max: i64) -> Option<i64> {
        let mut nf = max_nf(w_max);
        if nf > 0 {
            let nf = loop {
                if self.wxnf % nf == 0 {
                    break nf;
                }
                nf -= 1;
            };
            let w = self.wxnf / nf;
            Some(((w + 20 - 1) / 540 + 2) * 540)
        } else {
            None
        }
    }
}

/// A MOS tile.
#[derive(Block, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[substrate(io = "MosIo")]
pub struct MosTile {
    w: i64,
    nf: i64,
    l: MosLength,
    kind: TileKind,
}

impl MosTile {
    /// Creates a new [`TwoFingerMosTile`].
    pub fn new(w: i64, nf: i64, l: MosLength, kind: TileKind) -> Self {
        Self { w, nf, l, kind }
    }
}

impl Tile for MosTile {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<MosIo, PortGeometryBundle<Sky130>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<TileData<Self>> {
        cell.flatten();
        let (d, g, s, b) = match self.kind {
            TileKind::P => {
                let pmos = cell.generate_primitive(PmosTile::new(self.w, self.l, self.nf));
                let pmos = cell.draw(pmos)?;
                cell.connect(pmos.schematic.io().b, io.b);
                let mut d = PortGeometryBuilder::new();
                let mut g = PortGeometryBuilder::new();
                let mut s = PortGeometryBuilder::new();
                for i in 0..pmos.schematic.io().g.len() {
                    cell.connect(pmos.schematic.io().g[i], io.g);
                    g.merge(pmos.layout.io().g[i].clone());
                }
                for i in 0..pmos.schematic.io().sd.len() {
                    cell.connect(
                        pmos.schematic.io().sd[i],
                        if i % 2 == 0 { io.s } else { io.d },
                    );
                    if i % 2 == 0 {
                        s.merge(pmos.layout.io().sd[i].clone());
                    } else {
                        d.merge(pmos.layout.io().sd[i].clone());
                    }
                }
                (d.build()?, g.build()?, s.build()?, pmos.layout.io().b)
            }
            TileKind::N => {
                let nmos = cell.generate_primitive(NmosTile::new(self.w, self.l, self.nf));
                let nmos = cell.draw(nmos)?;
                cell.connect(nmos.schematic.io().b, io.b);
                let mut d = PortGeometryBuilder::new();
                let mut g = PortGeometryBuilder::new();
                let mut s = PortGeometryBuilder::new();
                for i in 0..nmos.schematic.io().g.len() {
                    cell.connect(nmos.schematic.io().g[i], io.g);
                    g.merge(nmos.layout.io().g[i].clone());
                }
                for i in 0..nmos.schematic.io().sd.len() {
                    cell.connect(
                        nmos.schematic.io().sd[i],
                        if i % 2 == 0 { io.s } else { io.d },
                    );
                    if i % 2 == 0 {
                        s.merge(nmos.layout.io().sd[i].clone());
                    } else {
                        d.merge(nmos.layout.io().sd[i].clone());
                    }
                }
                (d.build()?, g.build()?, s.build()?, nmos.layout.io().b)
            }
        };

        cell.set_top_layer(1);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(Sky130ViaMaker);

        Ok(TileData {
            nested_data: (),
            layout_bundle: MosIoView { d, g, s, b },
            layout_data: (),
            outline: cell.layout.bbox_rect(),
        })
    }
}

/// A tile containing a N/P tap for biasing an N-well or P-substrate.
/// These can be used to connect to the body terminals of MOS devices.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TapTile(TapTileParams);

impl TapTile {
    /// Creates a new [`TapTile`].
    pub fn new(params: TapTileParams) -> Self {
        Self(params)
    }
}

impl Block for TapTile {
    type Io = TapIo;

    fn name(&self) -> ArcStr {
        arcstr::format!(
            "{}tap_tile",
            match self.0.kind {
                TileKind::N => "n",
                TileKind::P => "p",
            }
        )
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Tile for TapTile {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<TapIo, PortGeometryBundle<Sky130>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<atoll::TileData<Self>> {
        cell.flatten();
        let x = match self.0.kind {
            TileKind::N => {
                let inst = cell.generate_primitive(sky130::atoll::NtapTile::new(
                    self.0.hspan - 1,
                    self.0.vspan,
                ));
                cell.connect(io.x, inst.io().vpb);
                let inst = cell.draw(inst)?;
                inst.layout.io().vpb
            }
            TileKind::P => {
                let inst = cell.generate_primitive(sky130::atoll::PtapTile::new(
                    self.0.hspan - 1,
                    self.0.vspan,
                ));
                cell.connect(io.x, inst.io().vnb);
                let inst = cell.draw(inst)?;
                inst.layout.io().vnb
            }
        };
        cell.set_router(GreedyRouter::new());
        Ok(TileData {
            nested_data: (),
            layout_bundle: TapIoView { x },
            layout_data: (),
            outline: cell.layout.bbox_rect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tb::{ComparatorDecision, StrongArmTranTb};
    use crate::tech::sky130::Sky130Impl;
    use crate::tiles::MosKind;
    use crate::{InputKind, StrongArm, StrongArmParams};
    use atoll::TileWrapper;
    use pegasus::lvs::LvsParams;
    use pegasus::{
        drc::{run_drc, DrcParams},
        lvs::LvsStatus,
        RuleCheck,
    };
    use quantus::pex::Pex;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use scir::netlist::ConvertibleNetlister;
    use sky130::corner::Sky130Corner;
    use sky130::{layout::to_gds, Sky130, Sky130CdsSchema};
    use spice::{netlist::NetlistOptions, Spice};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use substrate::context::Context;
    use substrate::geometry::dir::Dir;
    use substrate::simulation::Pvt;
    use substrate::{block::Block, schematic::ConvertSchema};

    pub const SKY130_DRC: &str = concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_DRC");
    pub const SKY130_DRC_RULES_PATH: &str = concat!(
        env!("SKY130_CDS_PDK_ROOT"),
        "/Sky130_DRC/sky130_rev_0.0_1.0.drc.pvl",
    );
    pub const SKY130_LVS: &str = concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_LVS");
    pub const SKY130_LVS_RULES_PATH: &str =
        concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_LVS/sky130.lvs.pvl",);
    pub const SKY130_TECHNOLOGY_DIR: &str =
        concat!(env!("SKY130_CDS_PDK_ROOT"), "/quantus/extraction/typical",);

    pub const STRONGARM_PARAMS: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 1_000,
        input_pair_w: 1_000,
        inv_input_w: 500,
        inv_precharge_w: 500,
        precharge_w: 500,
        input_kind: InputKind::P,
        h_max: 10_000,
        dir: Dir::Vert,
    };

    pub const STRONGARM_PARAMS_1: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 2_048,
        inv_input_w: 8_192,
        inv_precharge_w: 4096,
        precharge_w: 2_048,
        input_kind: InputKind::P,
        h_max: 40_000,
        dir: Dir::Vert,
    };

    pub const STRONGARM_PARAMS_2: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 2_048,
        inv_input_w: 8_192,
        inv_precharge_w: 4096,
        precharge_w: 2_048,
        input_kind: InputKind::P,
        h_max: 20_000,
        dir: Dir::Vert,
    };

    pub const STRONGARM_PARAMS_3: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 2_048,
        inv_input_w: 8_192,
        inv_precharge_w: 4096,
        precharge_w: 2_048,
        input_kind: InputKind::P,
        h_max: 15_000,
        dir: Dir::Vert,
    };

    pub const STRONGARM_PARAMS_4: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Lvt,
        pmos_kind: MosKind::Lvt,
        half_tail_w: 8_192,
        input_pair_w: 8_192,
        inv_input_w: 8_192,
        inv_precharge_w: 2_048,
        precharge_w: 2_048,
        input_kind: InputKind::N,
        h_max: 20_000,
        dir: Dir::Vert,
    };

    pub const STRONGARM_PARAMS_5: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 2_048,
        inv_input_w: 8_192,
        inv_precharge_w: 4096,
        precharge_w: 2_048,
        input_kind: InputKind::P,
        h_max: 15_000,
        dir: Dir::Horiz,
    };

    pub const STRONGARM_PARAMS_6: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 2_048,
        inv_input_w: 8_192,
        inv_precharge_w: 4096,
        precharge_w: 2_048,
        input_kind: InputKind::P,
        h_max: 40_000,
        dir: Dir::Horiz,
    };

    pub const STRONGARM_PARAMS_7: StrongArmParams = StrongArmParams {
        nmos_kind: MosKind::Nom,
        pmos_kind: MosKind::Nom,
        half_tail_w: 8_192,
        input_pair_w: 8_192,
        inv_input_w: 8_192,
        inv_precharge_w: 8_192,
        precharge_w: 8_192,
        input_kind: InputKind::N,
        h_max: 8_000,
        dir: Dir::Vert,
    };

    pub fn sky130_cds_ctx() -> Context {
        let pdk_root = std::env::var("SKY130_CDS_PDK_ROOT")
            .expect("the SKY130_CDS_PDK_ROOT environment variable must be set");
        Context::builder()
            .install(spectre::Spectre::default())
            .install(Sky130::cds_only(pdk_root))
            .build()
    }

    fn test_check_filter(check: &RuleCheck) -> bool {
        !["licon.12", "hvnwell.8"].contains(&check.name.as_ref())
    }

    fn test_strongarm(work_dir: impl AsRef<Path>, extracted: bool) {
        let work_dir = work_dir.as_ref();
        let input_kind = InputKind::P;
        let dut = TileWrapper::new(StrongArm::<Sky130Impl>::new(STRONGARM_PARAMS));
        let pvt = Pvt {
            corner: Sky130Corner::Tt,
            voltage: dec!(1.8),
            temp: dec!(25.0),
        };
        let ctx = sky130_cds_ctx();

        for i in 0..=10 {
            for j in [dec!(-0.1), dec!(-0.05), dec!(0.05), dec!(0.1)] {
                let vinn = dec!(0.18) * Decimal::from(i);
                let vinp = vinn + j;

                if vinp < dec!(0) || vinp > dec!(1.8) {
                    continue;
                }

                match input_kind {
                    InputKind::P => {
                        if (vinp + vinn) / dec!(2) > dec!(1.2) {
                            continue;
                        }
                    }
                    InputKind::N => {
                        if (vinp + vinn) / dec!(2) < dec!(0.6) {
                            continue;
                        }
                    }
                }
                let work_dir = work_dir.join(format!("ofs_{i}_{j}"));
                let decision = if extracted {
                    let layout_path = work_dir.join("layout.gds");
                    ctx.write_layout(dut, to_gds, &layout_path)
                        .expect("failed to write layout");
                    let tb = StrongArmTranTb::new(
                        Pex {
                            schematic: Arc::new(ConvertSchema::new(ConvertSchema::<
                                _,
                                Sky130CdsSchema,
                            >::new(
                                dut
                            ))),
                            gds_path: work_dir.join("layout.gds"),
                            layout_cell_name: dut.name(),
                            work_dir: work_dir.clone(),
                            lvs_rules_dir: PathBuf::from(SKY130_LVS),
                            lvs_rules_path: PathBuf::from(SKY130_LVS_RULES_PATH),
                            technology_dir: PathBuf::from(SKY130_TECHNOLOGY_DIR),
                        },
                        vinp,
                        vinn,
                        input_kind.is_p(),
                        pvt,
                    );
                    let sim = ctx
                        .get_sim_controller(tb.clone(), work_dir)
                        .expect("failed to get sim controller");
                    tb.run(sim).expect("comparator output did not rail")
                } else {
                    let tb = StrongArmTranTb::new(
                        ConvertSchema::<_, Sky130CdsSchema>::new(dut),
                        vinp,
                        vinn,
                        input_kind.is_p(),
                        pvt,
                    );
                    let sim = ctx
                        .get_sim_controller(tb.clone(), work_dir)
                        .expect("failed to get sim controller");
                    tb.run(sim).expect("comparator output did not rail")
                };
                assert_eq!(
                    decision,
                    if j > dec!(0) {
                        ComparatorDecision::Pos
                    } else {
                        ComparatorDecision::Neg
                    },
                    "comparator produced incorrect decision"
                );
            }
        }
    }

    #[test]
    fn sky130_strongarm_schematic_sim() {
        let work_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/sky130_strongarm_schematic_sim"
        );
        test_strongarm(work_dir, false);
    }

    #[test]
    #[ignore = "long"]
    fn sky130_strongarm_extracted_sim() {
        let work_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/sky130_strongarm_extracted_sim"
        );
        test_strongarm(work_dir, true);
    }

    fn test_sky130_strongarm_lvs(test_name: &'static str, params: StrongArmParams) {
        let work_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/build")).join(test_name);
        let gds_path = work_dir.join("layout.gds");
        let netlist_path = work_dir.join("netlist.sp");
        let ctx = sky130_cds_ctx();

        let block = TileWrapper::new(StrongArm::<Sky130Impl>::new(params));
        let scir = ctx
            .export_scir(block)
            .unwrap()
            .scir
            .convert_schema::<Sky130CdsSchema>()
            .unwrap()
            .convert_schema::<Spice>()
            .unwrap()
            .build()
            .unwrap();
        Spice
            .write_scir_netlist_to_file(&scir, &netlist_path, NetlistOptions::default())
            .expect("failed to write netlist");

        ctx.write_layout(block, to_gds, &gds_path)
            .expect("failed to write layout");

        // Run DRC.
        let drc_dir = work_dir.join("drc");
        let data = run_drc(&DrcParams {
            work_dir: &drc_dir,
            layout_path: &gds_path,
            cell_name: &block.name(),
            rules_dir: &PathBuf::from(SKY130_DRC),
            rules_path: &PathBuf::from(SKY130_DRC_RULES_PATH),
        })
        .expect("failed to run drc");

        assert_eq!(
            data.rule_checks
                .into_iter()
                .filter(test_check_filter)
                .count(),
            0,
            "layout was not DRC clean"
        );

        // Run LVS.
        let lvs_dir = work_dir.join("lvs");
        let output = pegasus::lvs::run_lvs(&LvsParams {
            work_dir: &lvs_dir,
            layout_path: &gds_path,
            layout_cell_name: &block.name(),
            source_paths: &[netlist_path],
            source_cell_name: &block.name(),
            rules_dir: &PathBuf::from(SKY130_LVS),
            rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
        })
        .expect("failed to run lvs");

        assert_eq!(
            output.status,
            LvsStatus::Correct,
            "layout does not match netlist"
        );
    }

    #[test]
    fn sky130_strongarm_lvs() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs", STRONGARM_PARAMS);
    }

    #[test]
    fn sky130_strongarm_lvs_resizing_1() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_1", STRONGARM_PARAMS_1);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_2() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_2", STRONGARM_PARAMS_2);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_3() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_3", STRONGARM_PARAMS_3);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_4() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_4", STRONGARM_PARAMS_4);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_5() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_5", STRONGARM_PARAMS_5);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_6() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_6", STRONGARM_PARAMS_6);
    }
    #[test]
    fn sky130_strongarm_lvs_resizing_7() {
        test_sky130_strongarm_lvs("sky130_strongarm_lvs_7", STRONGARM_PARAMS_7);
    }
}
