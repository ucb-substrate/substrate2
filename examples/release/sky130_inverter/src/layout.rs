// begin-code-snippet imports
use layir::Shape;
use sky130::{
    layers::Sky130Layer,
    layout::{NtapTile, PtapTile},
    mos::{GateDir, MosLength, NmosTile, PmosTile},
    Sky130,
};
use substrate::{
    error::Result,
    geometry::{
        align::{AlignMode, AlignRectMut},
        bbox::Bbox,
        prelude::Transformation,
        rect::Rect,
        span::Span,
        transform::TransformMut,
        union::BoundingUnion,
    },
    layout::{CellBuilder, CellBundle, Layout},
    types::{
        codegen::{PortGeometryBundle, View},
        layout::PortGeometry,
    },
};

use crate::{Inverter, InverterIo};
// end-code-snippet imports

// begin-code-snippet layout
impl Layout for Inverter {
    type Schema = Sky130;
    type Bundle = View<InverterIo, PortGeometryBundle<Sky130>>;
    type Data = ();
    fn layout(&self, cell: &mut CellBuilder<Self::Schema>) -> Result<(Self::Bundle, Self::Data)> {
        // begin-code-replace layout-body
        // begin-code-snippet generate-mos
        let mut nmos =
            cell.generate(NmosTile::new(self.nw, MosLength::L150, 1).with_gate_dir(GateDir::Left));
        let mut pmos =
            cell.generate(PmosTile::new(self.pw, MosLength::L150, 1).with_gate_dir(GateDir::Left));
        // end-code-snippet generate-mos

        // begin-code-snippet transform-mos
        nmos.transform_mut(Transformation::reflect_vert());
        pmos.align_mut(AlignMode::Above, pmos.bbox_rect(), nmos.bbox_rect(), 600);
        // end-code-snippet transform-mos

        // begin-code-snippet inverter-conns
        let dout = Shape::new(
            Sky130Layer::Li1,
            nmos.io().sd[1]
                .primary
                .bounding_union(&pmos.io().sd[1].primary),
        );
        cell.draw(dout.clone())?;

        let din = Shape::new(
            Sky130Layer::Li1,
            nmos.io().g[0]
                .primary
                .bounding_union(&pmos.io().g[0].primary),
        );
        cell.draw(din.clone())?;
        // end-code-snippet inverter-conns

        // begin-code-snippet taps
        let mut ntap = cell.generate(NtapTile::new(2, 2));
        ntap.align_mut(AlignMode::Above, ntap.bbox_rect(), pmos.bbox_rect(), 0);
        ntap.align_mut(
            AlignMode::CenterHorizontal,
            ntap.bbox_rect(),
            pmos.bbox_rect(),
            0,
        );

        let mut ptap = cell.generate(PtapTile::new(2, 2));
        ptap.align_mut(AlignMode::Beneath, ptap.bbox_rect(), nmos.bbox_rect(), -20);
        ptap.align_mut(
            AlignMode::CenterHorizontal,
            ptap.bbox_rect(),
            nmos.bbox_rect(),
            0,
        );

        let vdd = ntap.io().vpb.primary.clone();
        let vss = ptap.io().vnb.primary.clone();

        let nmos_s = nmos.io().sd[0].bbox_rect();
        let vss_conn = Rect::from_spans(
            nmos_s.hspan(),
            Span::new(vss.bbox_rect().bot(), nmos_s.top()),
        );
        cell.draw(Shape::new(Sky130Layer::Li1, vss_conn))?;

        let pmos_s = pmos.io().sd[0].bbox_rect();
        let vdd_conn = Rect::from_spans(
            pmos_s.hspan(),
            Span::new(pmos_s.bot(), vdd.bbox_rect().top()),
        );
        cell.draw(Shape::new(Sky130Layer::Li1, vdd_conn))?;
        // end-code-snippet taps

        // begin-code-snippet finalize-layout
        cell.draw(ntap)?;
        cell.draw(ptap)?;
        cell.draw(nmos)?;
        cell.draw(pmos)?;

        Ok((
            CellBundle::<Inverter> {
                vdd: PortGeometry::new(vdd),
                vss: PortGeometry::new(vss),
                din: PortGeometry::new(din),
                dout: PortGeometry::new(dout),
            },
            (),
        ))
        // end-code-snippet finalize-layout
        // end-code-replace layout-body
    }
}
// end-code-snippet layout

mod open {
    // begin-code-snippet open-tests
    #[cfg(test)]
    mod tests {
        use std::{path::PathBuf, sync::Arc};

        use magic::drc::{run_drc, DrcParams};
        use sky130::{layout::to_gds, Sky130OpenSchema};
        use substrate::{block::Block, schematic::ConvertSchema};

        use crate::{sky130_magic_tech_file, sky130_netgen_setup_file, sky130_open_ctx, Inverter};

        #[test]
        fn inverter_layout_open() {
            use magic_netgen::LvsParams;
            let work_dir = PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/inverter_layout_open"
            ));
            let layout_path = work_dir.join("layout.gds");
            let ctx = sky130_open_ctx();

            let dut = Inverter {
                nw: 1_200,
                pw: 2_400,
            };

            ctx.write_layout(dut, to_gds, &layout_path).unwrap();

            // Run DRC.
            let drc_dir = work_dir.join("drc");
            let drc_report_path = drc_dir.join("drc_results.rpt");
            let data = run_drc(&DrcParams {
                work_dir: &drc_dir,
                gds_path: &layout_path,
                cell_name: &dut.name(),
                tech_file_path: &sky130_magic_tech_file(),
                drc_report_path: &drc_report_path,
            })
            .expect("failed to run drc");

            assert_eq!(data.rule_checks.len(), 0, "layout was not DRC clean");

            // Run LVS.
            let lvs_dir = work_dir.join("lvs");
            let output = magic_netgen::run_lvs(LvsParams {
                schematic: Arc::new(ConvertSchema::new(
                    ConvertSchema::<_, Sky130OpenSchema>::new(dut),
                )),
                ctx: ctx.clone(),
                gds_path: layout_path,
                work_dir: lvs_dir,
                layout_cell_name: dut.name(),
                magic_tech_file_path: sky130_magic_tech_file(),
                netgen_setup_file_path: sky130_netgen_setup_file(),
            })
            .expect("failed to run lvs");

            assert!(output.matches, "layout does not match netlist");
        }
    }
    // end-code-snippet open-tests
}

mod cds {
    // begin-code-snippet cds-tests
    #[cfg(test)]
    mod tests {
        use std::path::PathBuf;

        use pegasus::{
            drc::{run_drc, DrcParams},
            lvs::LvsStatus,
            RuleCheck,
        };
        use sky130::{layout::to_gds, Sky130CdsSchema};
        use spice::{netlist::NetlistOptions, Spice};
        use substrate::{block::Block, schematic::ConvertSchema};

        use crate::{
            sky130_cds_ctx, sky130_drc, sky130_drc_rules_path, sky130_lvs, sky130_lvs_rules_path,
            Inverter,
        };

        fn test_check_filter(check: &RuleCheck) -> bool {
            !["licon.12", "hvnwell.8"].contains(&check.name.as_ref())
        }

        #[test]
        fn inverter_layout_cds() {
            use pegasus::lvs::LvsParams;
            use scir::netlist::ConvertibleNetlister;

            let work_dir = PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/inverter_layout_cds"
            ));
            let layout_path = work_dir.join("layout.gds");
            let ctx = sky130_cds_ctx();

            let dut = Inverter {
                nw: 1_200,
                pw: 2_400,
            };

            ctx.write_layout(dut, to_gds, &layout_path).unwrap();

            // Run DRC.
            let drc_dir = work_dir.join("drc");
            let data = run_drc(&DrcParams {
                work_dir: &drc_dir,
                layout_path: &layout_path,
                cell_name: &dut.name(),
                rules_dir: &sky130_drc(),
                rules_path: &sky130_drc_rules_path(),
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
                >::new(dut)))
                .unwrap();

            Spice
                .write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())
                .expect("failed to write netlist");
            let output = pegasus::lvs::run_lvs(&LvsParams {
                work_dir: &lvs_dir,
                layout_path: &layout_path,
                layout_cell_name: &dut.name(),
                source_paths: &[source_path],
                source_cell_name: &dut.name(),
                rules_dir: &sky130_lvs(),
                rules_path: &sky130_lvs_rules_path(),
            })
            .expect("failed to run lvs");

            assert_eq!(
                output.status,
                LvsStatus::Correct,
                "layout does not match netlist"
            );
        }
    }
    // end-code-snippet cds-tests
}
