use atoll::fold::Foldable;
use atoll::{Tile, TileData, route::GreedyRouter};
use layir::Shape;
use sky130::layers::Sky130Layer;
use sky130::{
    Sky130,
    atoll::{NmosTile, PtapTile, Sky130ViaMaker},
    res::PrecisionResistorCell,
};
use substrate::types::codegen::{PortGeometryBundle, View};
use substrate::{
    block::Block,
    geometry::align::AlignMode,
    geometry::bbox::Bbox,
    types::{FlatLen, InOut, Input, Io, Signal, layout::PortGeometryBuilder},
};

#[derive(Debug, Default, Clone, Io)]
pub struct TerminationSliceIo {
    pub din: Input<Signal>,
    pub en: Input<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "TerminationSliceIo")]
pub struct TerminationSlice {
    pub n: NmosTile,
    pub res: PrecisionResistorCell,
    pub tap: PtapTile,
}

impl Foldable for TerminationSlice {
    type ViaMaker = Sky130ViaMaker;
    fn via_maker() -> Self::ViaMaker {
        Sky130ViaMaker
    }
}

impl Tile for TerminationSlice {
    type Schema = Sky130;
    type NestedData = ();
    type LayoutBundle = View<TerminationSliceIo, PortGeometryBundle<Self::Schema>>;
    type LayoutData = ();

    fn tile<'a>(
        &self,
        io: &'a substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut atoll::TileBuilder<'a, Self::Schema>,
    ) -> substrate::error::Result<atoll::TileData<Self>> {
        let mut din = PortGeometryBuilder::new();
        let mut en = PortGeometryBuilder::new();
        let mut vss = PortGeometryBuilder::new();
        let x = cell.signal("x", Signal);
        let n = cell.generate_primitive_named(self.n, "nmos");
        for i in 0..n.io().g.len() {
            cell.connect(n.io().g[i], io.en);
        }
        for i in 0..n.io().sd.len() {
            if i % 2 == 0 {
                cell.connect(n.io().sd[i], io.din);
            } else {
                cell.connect(n.io().sd[i], x);
            }
        }
        cell.connect(n.io().b, io.vss);
        let mut tap = cell.generate_primitive_named(self.tap, "tap");
        cell.connect(tap.io().vnb, io.vss);
        let mut res = cell.generate_primitive_named(self.res, "res");
        cell.connect(res.io().p, x);
        cell.connect(res.io().n, io.vss);
        let nbbox = n.lcm_bounds();
        res.align_rect_mut(nbbox, AlignMode::CenterHorizontal, 0);
        res.align_rect_mut(nbbox, AlignMode::Beneath, 0);
        let resbbox = res.lcm_bounds();
        tap.align_rect_mut(nbbox, AlignMode::Left, 0);
        tap.align_rect_mut(resbbox, AlignMode::Beneath, 0);

        let n = cell.draw(n)?;
        let res = cell.draw(res)?;
        let tap = cell.draw(tap)?;

        let vss_rect = tap.layout.io().vnb.primary.bbox_rect();
        cell.layout.draw(Shape::new(Sky130Layer::Met1, vss_rect))?;
        cell.assign_grid_points(Some(*tap.schematic.io().vnb), 1, vss_rect);

        en.merge(n.layout.io().g[0].clone());
        din.merge(n.layout.io().sd[1].clone());
        vss.merge(n.layout.io().b);
        vss.merge(res.layout.io().n);
        vss.merge(tap.layout.io().vnb);

        cell.set_top_layer(2);
        cell.set_router(GreedyRouter::new());
        cell.set_via_maker(Sky130ViaMaker);

        Ok(TileData {
            nested_data: (),
            layout_bundle: TerminationSliceIoView {
                din: din.build()?,
                en: en.build()?,
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
    use atoll::fold::{FoldedArray, PinConfig};
    use scir::netlist::ConvertibleNetlister;
    use sky130::Sky130SrcNdaSchema;
    use sky130::atoll::MosLength;
    use sky130::res::{PrecisionResistor, PrecisionResistorWidth};
    use sky130::{Sky130, layout::to_gds};
    use spice::{Spice, netlist::NetlistOptions};
    use std::path::PathBuf;
    use substrate::context::Context;
    use substrate::geometry::dir::Dir;
    use substrate::geometry::side::Side;

    pub fn sky130_src_nda_ctx() -> Context {
        // Open PDK needed for standard cells.
        let open_pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
            .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
        let src_nda_pdk_root = std::env::var("SKY130_SRC_NDA_PDK_ROOT")
            .expect("the SKY130_SRC_NDA_PDK_ROOT environment variable must be set");
        Context::builder()
            .install(Sky130::src_nda(open_pdk_root, src_nda_pdk_root))
            .build()
    }

    #[test]
    fn termination_slice_lvs() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/termination_slice_lvs"
        ));
        let gds_path = work_dir.join("layout.gds");
        let netlist_path = work_dir.join("netlist.sp");
        let ctx = sky130_src_nda_ctx();

        let block = TileWrapper::new(TerminationSlice {
            tap: PtapTile::new(7, 4),
            res: PrecisionResistorCell {
                resistor: PrecisionResistor {
                    width: PrecisionResistorWidth::W285,
                    length: 4_000,
                },
                dir: Dir::Vert,
            },
            n: NmosTile::new(2_000, MosLength::L150, 6),
        });

        let scir = ctx
            .export_scir(block)
            .unwrap()
            .scir
            .convert_schema::<Sky130SrcNdaSchema>()
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

    #[test]
    fn termination_bank() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/termination_bank"
        ));
        let gds_path = work_dir.join("layout.gds");
        let netlist_path = work_dir.join("netlist.sp");
        let ctx = sky130_src_nda_ctx();

        let block = TileWrapper::new(FoldedArray {
            rows: 2,
            cols: 32,
            pins: vec![
                PinConfig::Parallel { layer: 1 },
                PinConfig::Escape {
                    layer: 0,
                    side: Side::Top,
                },
                PinConfig::Ignore,
            ],
            tile: TerminationSlice {
                tap: PtapTile::new(7, 4),
                res: PrecisionResistorCell {
                    resistor: PrecisionResistor {
                        width: PrecisionResistorWidth::W141,
                        length: 4_000,
                    },
                    dir: Dir::Vert,
                },
                n: NmosTile::new(2_000, MosLength::L150, 6),
            },
            top_layer: 3,
        });

        let scir = ctx
            .export_scir(block.clone())
            .unwrap()
            .scir
            .convert_schema::<Sky130SrcNdaSchema>()
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
