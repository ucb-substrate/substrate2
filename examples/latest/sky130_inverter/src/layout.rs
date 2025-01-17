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
        point::Point,
        prelude::Transformation,
        rect::Rect,
        span::Span,
        transform::{TransformMut, TranslateMut},
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
        ptap.align_mut(AlignMode::Beneath, ptap.bbox_rect(), nmos.bbox_rect(), 0);
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
    use gds::GdsUnits;
    use gdsconv::export::GdsExportOpts;
    use sky130pdk::layout::to_gds;

    use crate::{tb::sky130_open_ctx, Inverter};

    #[test]
    fn inverter_layout() {
        let layout_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/inverter_layout/layout.gds"
        );
        let ctx = sky130_open_ctx();

        let layir = ctx
            .export_layir(Inverter {
                nw: 1_200,
                pw: 2_400,
                lch: 150,
            })
            .unwrap();
        let layir = to_gds(&layir.layir);
        let gds = gdsconv::export::export_gds(
            layir,
            GdsExportOpts {
                name: arcstr::literal!("pfet_01v8_layout"),
                units: Some(GdsUnits::new(1., 1e-9)),
            },
        );
        gds.save(layout_path).unwrap();
    }
}
