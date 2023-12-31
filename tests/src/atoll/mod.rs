use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;
use atoll::abs::{DebugAbstract, AtollAbstract};
use atoll::grid::{LayerStack, PdkLayer};
use atoll::{AtollIo, AtollTile, AtollTileBuilder, AtollTileWrapper};
use geometry::point::Point;
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
    let netlist_path = get_path("sky130_atoll_nmos_tile", "schematic.sp");
    let ctx = sky130_open_ctx();

    let block = sky130pdk::atoll::NmosTile::new(1_680, MosLength::L150, 3);

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

    // todo: add mechanism to have multiple ATOLL layer stacks (one per PDK)
    let stack = ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();

    let abs = AtollAbstract::generate(&ctx, handle.cell());
    ctx.write_layout(
        DebugAbstract {
            abs,
            stack: (*stack).clone(),
        },
        abs_path,
    )
    .expect("failed to write abstract");
}

#[test]
fn sky130_atoll_pmos_tile() {
    let gds_path = get_path("sky130_atoll_pmos_tile", "layout.gds");
    let abs_path = get_path("sky130_atoll_pmos_tile", "abs.gds");
    let netlist_path = get_path("sky130_atoll_pmos_tile", "schematic.sp");
    let ctx = sky130_open_ctx();

    let block = sky130pdk::atoll::PmosTile::new(1_680, MosLength::L150, 3);

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

    let abs = AtollAbstract::generate(&ctx, handle.cell());
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

impl AtollTile<Sky130Pdk> for Sky130NmosTileAutoroute {
    fn tile<'a>(
        &self,
        io: AtollIo<'a, Self>,
        cell: &mut AtollTileBuilder<'a, Sky130Pdk>,
    ) -> substrate::error::Result<(
        <Self as ExportsNestedData>::NestedData,
        <Self as ExportsLayoutData>::LayoutData,
    )> {
        let block = sky130pdk::atoll::NmosTile::new(1_680, MosLength::L150, 3);

        let mut instances = Vec::new();

        for i in 0..3 {
            let mut inst = cell.generate_primitive(block.clone());
            inst.translate_mut(Point::new(5 * i, 0));
            cell.draw(&inst)?;

            let (schematic, layout) = inst.into_instances();

            for i in 0..4 {
                cell.connect(io.schematic.sd, schematic.io().sd[i]);
                io.layout.sd.merge(layout.io().sd[i].clone());
            }
            cell.connect(io.schematic.g, schematic.io().g);
            cell.connect(io.schematic.b, schematic.io().b);
            io.layout.g.merge(layout.io().g.clone());
            io.layout.b.merge(layout.io().b.clone());

            instances.push(schematic);
        }

        Ok((instances, ()))
    }
}

#[test]
fn sky130_atoll_nmos_tile_autoroute() {
    let gds_path = get_path("sky130_atoll_nmos_tile_autoroute", "layout.gds");
    let abs_path = get_path("sky130_atoll_nmos_tile_autoroute", "abstract.gds");
    let ctx = sky130_open_ctx();
    let netlist_path = get_path("sky130_atoll_nmos_tile_autoroute", "schematic.sp");

    let block = AtollTileWrapper::new(Sky130NmosTileAutoroute);

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

    let abs = AtollAbstract::generate(&ctx, handle.cell());
    ctx.write_layout(
        DebugAbstract {
            abs,
            stack: (*stack).clone(),
        },
        abs_path,
    )
    .expect("failed to write abstract");
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
        let grid = DebugRoutingGrid::new(RoutingGrid::new((*stack).clone(), 0..stack.len()), 10, 2);
        cell.draw(grid)?;
        Ok(())
    }
}
