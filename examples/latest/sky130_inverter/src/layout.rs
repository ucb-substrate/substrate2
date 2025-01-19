use layir::Shape;
use sky130pdk::{
    layers::Sky130Layer,
    layout::{NtapTile, PtapTile},
    mos::{GateDir, MosLength, NmosTile, PmosTile},
    Sky130Pdk,
};
use substrate::{
    geometry::{
        align::{AlignMode, AlignRectMut},
        bbox::Bbox,
        prelude::Transformation,
        rect::Rect,
        span::Span,
        transform::TransformMut,
        union::BoundingUnion,
    },
    layout::Layout,
    types::layout::PortGeometry,
};

use crate::{Inverter, InverterIoView};

impl Layout for Inverter {
    type Schema = Sky130Pdk;
    type Bundle = InverterIoView<substrate::types::codegen::PortGeometryBundle<Sky130Pdk>>;
    type Data = ();
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let mut nmos =
            cell.generate(NmosTile::new(self.nw, MosLength::L150, 1).with_gate_dir(GateDir::Left));
        let mut pmos =
            cell.generate(PmosTile::new(self.pw, MosLength::L150, 1).with_gate_dir(GateDir::Left));

        nmos.transform_mut(Transformation::reflect_vert());
        pmos.align_mut(AlignMode::Above, pmos.bbox_rect(), nmos.bbox_rect(), 600);

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

        cell.draw(ntap)?;
        cell.draw(ptap)?;
        cell.draw(nmos)?;
        cell.draw(pmos)?;

        Ok((
            InverterIoView {
                vdd: PortGeometry::new(vdd),
                vss: PortGeometry::new(vss),
                din: PortGeometry::new(din),
                dout: PortGeometry::new(dout),
            },
            (),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use pegasus::{
        drc::{run_drc, DrcParams},
        lvs::LvsStatus,
        RuleCheck,
    };
    use sky130pdk::{layout::to_gds, Sky130CdsSchema, Sky130OpenSchema};
    use spice::{netlist::NetlistOptions, Spice};
    use substrate::{block::Block, schematic::ConvertSchema};

    use crate::{
        sky130_open_ctx, Inverter, SKY130_DRC, SKY130_DRC_RULES_PATH, SKY130_LVS,
        SKY130_LVS_RULES_PATH, SKY130_MAGIC_TECH_FILE, SKY130_NETGEN_SETUP_FILE,
    };

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
            lch: 150,
        };

        ctx.write_layout(dut, to_gds, &layout_path).unwrap();

        let lvs_dir = work_dir.join("lvs");
        let output = magic_netgen::run_lvs(LvsParams {
            schematic: Arc::new(ConvertSchema::new(
                ConvertSchema::<_, Sky130OpenSchema>::new(dut),
            )),
            ctx: ctx.clone(),
            gds_path: layout_path,
            work_dir: lvs_dir,
            layout_cell_name: dut.name(),
            magic_tech_file_path: PathBuf::from(SKY130_MAGIC_TECH_FILE),
            netgen_setup_file_path: PathBuf::from(SKY130_NETGEN_SETUP_FILE),
        })
        .expect("failed to run lvs");

        assert!(output.matches, "layout does not match netlist");
    }

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
        let ctx = sky130_open_ctx();

        let dut = Inverter {
            nw: 1_200,
            pw: 2_400,
            lch: 150,
        };

        ctx.write_layout(dut, to_gds, &layout_path).unwrap();

        // Run DRC.
        let drc_dir = work_dir.join("drc");
        let data = run_drc(&DrcParams {
            work_dir: &drc_dir,
            layout_path: &layout_path,
            cell_name: &dut.name(),
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
