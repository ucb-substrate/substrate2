use atoll::{fold::Foldable, route::GreedyRouter, Orientation, Tile, TileData};
use layir::Shape;
use sky130::{
    atoll::{GateDir, MosLength, NmosTile, NtapTile, PmosTile, PtapTile, Sky130ViaMaker},
    layers::Sky130Layer,
    Sky130,
};
use substrate::{
    geometry::{align::AlignMode, bbox::Bbox, rect::Rect},
    types::{
        codegen::{PortGeometryBundle, View},
        layout::PortGeometry,
    },
};

use crate::{Inverter, InverterIo, InverterIoView};

impl Tile for Inverter {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<InverterIo, PortGeometryBundle<Sky130>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut atoll::TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<atoll::TileData<Self>> {
        let mut n = cell.generate_primitive_named(
            NmosTile::new(self.nw, MosLength::L150, 1).with_gate_dir(GateDir::Left),
            "nmos",
        );
        let mut p = cell.generate_primitive_named(
            PmosTile::new(self.pw, MosLength::L150, 1).with_gate_dir(GateDir::Left),
            "pmos",
        );
        let mut ptap = cell.generate_primitive(PtapTile::new(2, 2));
        let mut ntap = cell.generate_primitive(NtapTile::new(2, 2));
        n.orient_mut(Orientation::ReflectVert);
        ptap.align_mut(&n, AlignMode::Beneath, 0);
        ptap.align_mut(&n, AlignMode::CenterHorizontal, 0);
        p.align_mut(&n, AlignMode::Above, 0);
        p.align_mut(&n, AlignMode::CenterHorizontal, 0);
        ntap.align_mut(&p, AlignMode::Above, 0);
        ntap.align_mut(&p, AlignMode::CenterHorizontal, 0);
        cell.connect(n.io().g[0], io.din);
        cell.connect(p.io().g[0], io.din);
        cell.connect(n.io().sd[0], io.vss);
        cell.connect(p.io().sd[0], io.vdd);
        cell.connect(n.io().sd[1], io.dout);
        cell.connect(p.io().sd[1], io.dout);
        cell.connect(ntap.io().vpb, io.vdd);
        cell.connect(ptap.io().vnb, io.vss);
        cell.connect(n.io().b, io.vss);
        cell.connect(p.io().b, io.vdd);

        let n = cell.draw(n)?;
        let p = cell.draw(p)?;
        let ntap = cell.draw(ntap)?;
        let ptap = cell.draw(ptap)?;

        let din = n.layout.io().g[0]
            .bbox_rect()
            .union(p.layout.io().g[0].bbox_rect());
        let dout = n.layout.io().sd[1]
            .bbox_rect()
            .union(p.layout.io().sd[1].bbox_rect());
        let m1_tracks = cell.layer_stack.tracks(1);
        let mid_pin_track_idx = m1_tracks.to_track_idx(
            din.center().y,
            substrate::layout::tracks::RoundingMode::Nearest,
        );
        let mid_pin_track = m1_tracks.get(mid_pin_track_idx);

        let m0_tracks = cell.layer_stack.tracks(0);
        let left_pin_track_idx = m0_tracks.to_track_idx(
            din.center().x,
            substrate::layout::tracks::RoundingMode::Nearest,
        );
        let left_pin_track = m0_tracks.get(left_pin_track_idx);
        let right_pin_track_idx = m0_tracks.to_track_idx(
            dout.center().x,
            substrate::layout::tracks::RoundingMode::Nearest,
        );
        let right_pin_track = m0_tracks.get(right_pin_track_idx);
        let din_pin = Shape::new(
            Sky130Layer::Met1,
            Rect::from_spans(left_pin_track, mid_pin_track),
        );
        let dout_pin = Shape::new(
            Sky130Layer::Met1,
            Rect::from_spans(right_pin_track, mid_pin_track),
        );
        cell.layout.draw(din_pin.clone())?;
        cell.layout.draw(dout_pin.clone())?;
        cell.assign_grid_points(
            Some(io.din),
            1,
            Rect::from_xy(left_pin_track_idx, mid_pin_track_idx),
        );
        cell.assign_grid_points(
            Some(io.dout),
            1,
            Rect::from_xy(right_pin_track_idx, mid_pin_track_idx),
        );

        cell.set_router(GreedyRouter::new());
        cell.set_top_layer(2);
        cell.set_via_maker(Sky130ViaMaker);

        Ok(TileData {
            nested_data: (),
            layout_bundle: InverterIoView {
                vdd: ntap.layout.io().vpb,
                vss: ptap.layout.io().vnb,
                din: PortGeometry::new(din_pin),
                dout: PortGeometry::new(dout_pin),
            },
            layout_data: (),
            outline: cell.layout.bbox_rect(),
        })
    }
}

impl Foldable for Inverter {
    type ViaMaker = Sky130ViaMaker;

    fn via_maker() -> Self::ViaMaker {
        Sky130ViaMaker
    }
}

#[cfg(test)]
mod tests {
    use crate::sky130_cds_ctx;

    use super::*;

    use atoll::fold::{FoldedArray, PinConfig};
    use atoll::TileWrapper;
    use pegasus::drc::{run_drc, DrcParams};
    use pegasus::lvs::{LvsParams, LvsStatus};
    use pegasus::RuleCheck;
    use scir::netlist::ConvertibleNetlister;

    use sky130::layout::to_gds;
    use sky130::{
        Sky130CdsSchema, SKY130_DRC, SKY130_DRC_RULES_PATH, SKY130_LVS, SKY130_LVS_RULES_PATH,
    };
    use spice::{netlist::NetlistOptions, Spice};
    use std::path::PathBuf;
    use substrate::block::Block;

    use substrate::geometry::dir::Dir;

    use substrate::schematic::ConvertSchema;

    fn test_check_filter(check: &RuleCheck) -> bool {
        !["licon.12", "hvnwell.8"].contains(&check.name.as_ref())
    }

    #[test]
    fn inverter_layout_atoll() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/inverter_layout_atoll"
        ));
        let layout_path = work_dir.join("layout.gds");
        let ctx = sky130_cds_ctx();

        let block = TileWrapper::new(Inverter {
            nw: 1_200,
            pw: 2_400,
        });

        ctx.write_layout(block, to_gds, &layout_path).unwrap();

        // Run DRC.
        let drc_dir = work_dir.join("drc");
        let data = run_drc(&DrcParams {
            work_dir: &drc_dir,
            layout_path: &layout_path,
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
        let source_path = work_dir.join("schematic.spice");
        let rawlib = ctx
            .export_scir(ConvertSchema::<_, Spice>::new(ConvertSchema::<
                _,
                Sky130CdsSchema,
            >::new(block)))
            .unwrap();

        Spice
            .write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())
            .expect("failed to write netlist");
        let output = pegasus::lvs::run_lvs(&LvsParams {
            work_dir: &lvs_dir,
            layout_path: &layout_path,
            layout_cell_name: &block.name(),
            source_paths: &[source_path],
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
    fn inverter_chain_layout_atoll() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/inverter_chain_layout_atoll"
        ));
        let layout_path = work_dir.join("layout.gds");
        let ctx = sky130_cds_ctx();

        let block = TileWrapper::new(FoldedArray {
            tile: Inverter {
                nw: 1_200,
                pw: 2_400,
            },
            rows: 5,
            cols: 5,
            pins: vec![
                PinConfig::Ignore,
                PinConfig::Ignore,
                PinConfig::Ignore,
                PinConfig::Series {
                    partner: 2,
                    layer: 1,
                },
            ],
            top_layer: 2,
            dir: Dir::Horiz,
        });

        ctx.write_layout(block.clone(), to_gds, &layout_path)
            .unwrap();

        // Run DRC.
        let drc_dir = work_dir.join("drc");
        let data = run_drc(&DrcParams {
            work_dir: &drc_dir,
            layout_path: &layout_path,
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
        let source_path = work_dir.join("schematic.spice");
        let rawlib = ctx
            .export_scir(ConvertSchema::<_, Spice>::new(ConvertSchema::<
                _,
                Sky130CdsSchema,
            >::new(
                block.clone()
            )))
            .unwrap();

        Spice
            .write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())
            .expect("failed to write netlist");
        let output = pegasus::lvs::run_lvs(&LvsParams {
            work_dir: &lvs_dir,
            layout_path: &layout_path,
            layout_cell_name: &block.name(),
            source_paths: &[source_path],
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
