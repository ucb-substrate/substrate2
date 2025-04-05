use atoll::{route::GreedyRouter, Tile, TileData};
use sky130::{
    atoll::{NmosTile, PmosTile, Sky130ViaMaker},
    Sky130,
};
use substrate::{
    block::Block,
    geometry::bbox::Bbox,
    types::{layout::PortGeometryBuilder, FlatLen, InOut, Input, Io, Output, Signal},
};
use substrate::{
    geometry::align::AlignMode,
    types::{
        codegen::{PortGeometryBundle, View},
        schematic::NodeBundle,
        MosIo,
    },
};

#[derive(Debug, Default, Clone, Io)]
pub struct DriverSliceIo {
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "DriverSliceIo")]
pub struct DriverSlice {
    pub p: PmosTile,
    pub n: NmosTile,
}

impl Tile for DriverSlice {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<DriverSliceIo, PortGeometryBundle<Self::Schema>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut atoll::TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<atoll::TileData<Self>> {
        let mut din = PortGeometryBuilder::new();
        let mut dout = PortGeometryBuilder::new();
        let mut vdd = PortGeometryBuilder::new();
        let mut vss = PortGeometryBuilder::new();
        let n = cell.generate_primitive_named(self.n, "nmos");
        for i in 0..n.io().g.len() {
            cell.connect(n.io().g[i], io.din);
        }
        for i in 0..n.io().sd.len() {
            if i % 2 == 0 {
                cell.connect(n.io().sd[i], io.vss);
            } else {
                cell.connect(n.io().sd[i], io.dout);
            }
        }
        cell.connect(n.io().b, io.vss);
        let mut p = cell.generate_primitive_named(self.p, "pmos");
        for i in 0..p.io().g.len() {
            cell.connect(p.io().g[i], io.din);
        }
        for i in 0..p.io().sd.len() {
            if i % 2 == 0 {
                cell.connect(p.io().sd[i], io.vdd);
            } else {
                cell.connect(p.io().sd[i], io.dout);
            }
        }
        cell.connect(p.io().b, io.vdd);
        let nbbox = n.lcm_bounds();
        p.align_rect_mut(nbbox, AlignMode::Left, 0);
        p.align_rect_mut(nbbox, AlignMode::Above, 0);

        let n = cell.draw(n)?;
        let p = cell.draw(p)?;

        din.merge(n.layout.io().g[0].clone());
        din.merge(p.layout.io().g[0].clone());
        dout.merge(n.layout.io().sd[1].clone());
        dout.merge(p.layout.io().sd[1].clone());
        vss.merge(n.layout.io().b);
        vss.merge(n.layout.io().sd[0].clone());
        vdd.merge(p.layout.io().b);
        vdd.merge(p.layout.io().sd[0].clone());

        cell.set_top_layer(2);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(Sky130ViaMaker);

        Ok(TileData {
            nested_data: (),
            layout_bundle: DriverSliceIoView {
                din: din.build()?,
                dout: dout.build()?,
                vdd: vdd.build()?,
                vss: vss.build()?,
            },
            layout_data: (),
            outline: cell.layout.bbox_rect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use atoll::TileWrapper;
    use scir::netlist::ConvertibleNetlister;
    use sky130::atoll::MosLength;
    use sky130::{layout::to_gds, Sky130, Sky130CdsSchema};
    use spice::{netlist::NetlistOptions, Spice};
    use std::path::PathBuf;
    use substrate::context::Context;

    pub fn sky130_cds_ctx() -> Context {
        let pdk_root = std::env::var("SKY130_CDS_PDK_ROOT")
            .expect("the SKY130_CDS_PDK_ROOT environment variable must be set");
        Context::builder()
            .install(Sky130::cds_only(pdk_root))
            .build()
    }

    #[test]
    fn driver_slice_lvs() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/driver_slice_lvs"
        ));
        let gds_path = work_dir.join("layout.gds");
        let netlist_path = work_dir.join("netlist.sp");
        let ctx = sky130_cds_ctx();

        let block = TileWrapper::new(DriverSlice {
            p: PmosTile::new(2_000, MosLength::L150, 4),
            n: NmosTile::new(2_000, MosLength::L150, 4),
        });

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
    }
}
