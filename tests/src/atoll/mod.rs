use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;
use atoll::abs::{generate_abstract, DebugAbstract};
use atoll::grid::{LayerStack, PdkLayer};
use atoll::{AtollIo, AtollTile, AtollTileBuilder, AtollTileWrapper};
use geometry::rect::Rect;
use serde::{Deserialize, Serialize};
use sky130pdk::atoll::{MosLength, NmosTile};
use sky130pdk::{Sky130CommercialSchema, Sky130Pdk};
use spice::netlist::NetlistOptions;
use spice::Spice;
use substrate::block::Block;
use substrate::io::layout::{HardwareType, IoShape};
use substrate::io::{InOut, Io, Signal};
use substrate::layout::tiling::{GridTiler, Tile};
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::schematic;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::ExportsNestedData;

#[test]
fn sky130_atoll_debug_routing_grid() {
    let gds_path = get_path("sky130_atoll_debug_routing_grid", "layout.gds");
    let ctx = sky130_open_ctx();

    ctx.write_layout(Sky130DebugRoutingGrid, gds_path)
        .expect("failed to write layout");
}

#[test]
fn sky130_atoll_nmos_tile() {
    let gds_path = get_path("sky130_atoll_nmos_tile", "layout.gds");
    let abs_path = get_path("sky130_atoll_nmos_tile", "abs.gds");
    let netlist_path = get_path("sky130_atoll_nmos_tile", "schematic.scs");
    let ctx = sky130_open_ctx();

    let block = sky130pdk::atoll::NmosTile {
        w: 1_680,
        nf: 3,
        len: MosLength::L150,
    };

    ctx.write_layout(block, gds_path)
        .expect("failed to write layout");

    let scir = ctx
        .export_scir(block)
        .unwrap()
        .scir
        .convert_schema::<Sky130CommercialSchema>()
        .unwrap()
        .convert_schema::<Spice>()
        .unwrap()
        .build()
        .unwrap();
    Spice
        .write_scir_netlist_to_file(&scir, netlist_path, NetlistOptions::default())
        .expect("failed to write netlist");

    let handle = ctx.generate_layout(block);
    let stack = ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();

    let abs = generate_abstract(handle.cell(), &*stack);
    ctx.write_layout(
        DebugAbstract {
            abs,
            stack: (*stack).clone(),
        },
        abs_path,
    )
    .expect("failed to write abstract");
}

#[derive(Clone, Copy, Debug, Default, Io)]
pub struct Sky130NmosTileAutorouteIo {
    sd: InOut<Signal>,
    g: InOut<Signal>,
    b: InOut<Signal>,
}

#[derive(Block, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
#[substrate(io = "Sky130NmosTileAutorouteIo")]
pub struct Sky130NmosTileAutoroute;

impl ExportsNestedData for Sky130NmosTileAutoroute {
    type NestedData = Vec<schematic::Instance<NmosTile>>;
}

impl ExportsLayoutData for Sky130NmosTileAutoroute {
    type LayoutData = ();
}

impl AtollTile<Sky130Pdk, Sky130Pdk> for Sky130NmosTileAutoroute {
    fn tile<'a>(
        &self,
        io: AtollIo<'a, Self>,
        cell: &mut AtollTileBuilder<'a, Sky130Pdk, Sky130Pdk>,
    ) -> substrate::error::Result<(
        <Self as ExportsNestedData>::NestedData,
        <Self as ExportsLayoutData>::LayoutData,
    )> {
        let block = sky130pdk::atoll::NmosTile {
            w: 1_680,
            nf: 3,
            len: MosLength::L150,
        };

        let mut tiler = GridTiler::new();
        let mut keys = Vec::new();

        for i in 0..6 {
            let atoll_inst = cell.generate(block.clone());
            keys.push(tiler.push(Tile::from_bbox(atoll_inst)));
            if i == 2 {
                tiler.end_row();
            }
        }

        let tiler = tiler.tile();

        let mut instances = Vec::new();
        for key in keys {
            instances.push(cell.draw((*tiler.get(key).unwrap()).clone())?);
        }

        instances.iter().for_each(|inst| {
            for i in 0..4 {
                cell.connect(io.schematic.sd, inst.io().sd[i]);
            }
            cell.connect(io.schematic.g, inst.io().g);
            cell.connect(io.schematic.b, inst.io().b);
        });

        let sd = IoShape::with_layers(cell.ctx().layers.li1, Rect::from_sides(0, 0, 100, 100));
        let g = IoShape::with_layers(cell.ctx().layers.li1, Rect::from_sides(100, 0, 200, 100));
        let b = IoShape::with_layers(cell.ctx().layers.li1, Rect::from_sides(200, 0, 300, 100));

        cell.match_geometry(io.schematic.sd, sd.clone());
        cell.match_geometry(io.schematic.g, g.clone());
        cell.match_geometry(io.schematic.b, b.clone());

        io.layout.sd.set_primary(sd);
        io.layout.g.set_primary(g);
        io.layout.b.set_primary(b);

        Ok((instances, ()))
    }
}

#[test]
fn sky130_atoll_nmos_tile_autoroute() {
    let gds_path = get_path("sky130_atoll_nmos_tile_autoroute", "layout.gds");
    let ctx = sky130_open_ctx();

    ctx.write_layout(
        AtollTileWrapper::<Sky130NmosTileAutoroute, Sky130Pdk, Sky130Pdk>::new(
            Sky130NmosTileAutoroute,
        ),
        gds_path,
    )
    .expect("failed to write layout");
}

#[derive(Block, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
#[substrate(io = "()")]
pub struct Sky130DebugRoutingGrid;

impl ExportsLayoutData for Sky130DebugRoutingGrid {
    type LayoutData = ();
}

impl Layout<Sky130Pdk> for Sky130DebugRoutingGrid {
    fn layout(
        &self,
        _io: &mut <<Self as Block>::Io as HardwareType>::Builder,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        use atoll::grid::*;
        let stack = cell.ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();
        let grid = DebugRoutingGrid::new(RoutingGrid::new((*stack).clone(), 0..stack.len(), 10, 2));
        cell.draw(grid)?;
        Ok(())
    }
}
