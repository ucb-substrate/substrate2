//! SKY130-specific implementations.

use crate::tiles::{MosTileParams, TapIo, TapIoView, TapTileParams, TileKind};
use crate::StrongArmImpl;
use atoll::route::GreedyRouter;
use atoll::{Tile, TileBuilder};
use sky130::atoll::{MosLength, NmosTile, PmosTile, Sky130ViaMaker};
use sky130::Sky130;
use substrate::arcstr;
use substrate::arcstr::ArcStr;
use substrate::block::Block;
use substrate::geometry::bbox::Bbox;
use substrate::types::codegen::{PortGeometryBundle, View};
use substrate::types::layout::PortGeometryBuilder;
use substrate::types::{MosIo, MosIoView};

/// A SKY130 implementation.
pub struct Sky130Impl;

impl StrongArmImpl for Sky130Impl {
    type Schema = Sky130;
    type MosTile = TwoFingerMosTile;
    type TapTile = TapTile;
    type ViaMaker = Sky130ViaMaker;

    fn mos(params: MosTileParams) -> Self::MosTile {
        TwoFingerMosTile::new(params.w, MosLength::L150, params.tile_kind)
    }
    fn tap(params: TapTileParams) -> Self::TapTile {
        TapTile::new(params)
    }
    fn via_maker() -> Self::ViaMaker {
        Sky130ViaMaker
    }
}

/// A two-finger MOS tile.
#[derive(Block, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[substrate(io = "MosIo")]
pub struct TwoFingerMosTile {
    w: i64,
    l: MosLength,
    kind: TileKind,
}

impl TwoFingerMosTile {
    /// Creates a new [`TwoFingerMosTile`].
    pub fn new(w: i64, l: MosLength, kind: TileKind) -> Self {
        Self { w, l, kind }
    }
}

impl Tile for TwoFingerMosTile {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<MosIo, PortGeometryBundle<Sky130>>;
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
        cell.flatten();
        let (d, g, s, b) = match self.kind {
            TileKind::P => {
                let pmos = cell.generate_primitive(PmosTile::new(self.w, self.l, 2));
                cell.connect(pmos.io().g[0], io.g);
                cell.connect(pmos.io().b, io.b);
                cell.connect(pmos.io().sd[0], io.s);
                cell.connect(pmos.io().sd[1], io.d);
                cell.connect(pmos.io().sd[2], io.s);
                let pmos = cell.draw(pmos)?;
                let mut s = PortGeometryBuilder::new();
                s.merge(pmos.layout.io().sd[0].clone());
                s.merge(pmos.layout.io().sd[2].clone());
                (
                    pmos.layout.io().sd[1].clone(),
                    pmos.layout.io().g[0].clone(),
                    s.build()?,
                    pmos.layout.io().b,
                )
            }
            TileKind::N => {
                let nmos = cell.generate_primitive(NmosTile::new(self.w, self.l, 2));
                cell.connect(nmos.io().g[0], io.g);
                cell.connect(nmos.io().b, io.b);
                cell.connect(nmos.io().sd[0], io.s);
                cell.connect(nmos.io().sd[1], io.d);
                cell.connect(nmos.io().sd[2], io.s);
                let nmos = cell.draw(nmos)?;
                let mut s = PortGeometryBuilder::new();
                s.merge(nmos.layout.io().sd[0].clone());
                s.merge(nmos.layout.io().sd[2].clone());
                (
                    nmos.layout.io().sd[1].clone(),
                    nmos.layout.io().g[0].clone(),
                    s.build()?,
                    nmos.layout.io().b,
                )
            }
        };

        cell.set_top_layer(1);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(Sky130ViaMaker);

        Ok(((), MosIoView { d, g, s, b }, (), cell.layout.bbox_rect()))
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
    ) -> substrate::error::Result<(
        Self::NestedData,
        Self::LayoutBundle,
        Self::LayoutData,
        substrate::geometry::prelude::Rect,
    )> {
        cell.flatten();
        let x = match self.0.kind {
            TileKind::N => {
                let inst = cell
                    .generate_primitive(sky130::atoll::NtapTile::new(4 * self.0.mos_span - 1, 2));
                cell.connect(io.x, inst.io().vpb);
                let inst = cell.draw(inst)?;
                inst.layout.io().vpb
            }
            TileKind::P => {
                let inst = cell
                    .generate_primitive(sky130::atoll::PtapTile::new(4 * self.0.mos_span - 1, 2));
                cell.connect(io.x, inst.io().vnb);
                let inst = cell.draw(inst)?;
                inst.layout.io().vnb
            }
        };
        cell.set_router(GreedyRouter::new());
        Ok(((), TapIoView { x }, (), cell.layout.bbox_rect()))
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
                println!("{i} {j}");
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
    fn sky130_strongarm_extracted_sim() {
        let work_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/sky130_strongarm_extracted_sim"
        );
        test_strongarm(work_dir, true);
    }

    #[test]
    fn sky130_strongarm_lvs() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/sky130_strongarm_lvs"
        ));
        let gds_path = work_dir.join("layout.gds");
        let netlist_path = work_dir.join("netlist.sp");
        let ctx = sky130_cds_ctx();

        let block = TileWrapper::new(StrongArm::<Sky130Impl>::new(STRONGARM_PARAMS));

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
}
