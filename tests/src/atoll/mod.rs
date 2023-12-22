use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;
use serde::{Deserialize, Serialize};
use sky130pdk::atoll::{MosLength, Sky130AtollLayer};
use sky130pdk::Sky130Pdk;
use spectre::Spectre;
use spice::netlist::NetlistOptions;
use spice::Spice;
use substrate::block::Block;
use substrate::io::layout::HardwareType;
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::schematic::netlist::ConvertibleNetlister;

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
    let spice_path = get_path("sky130_atoll_nmos_tile", "schematic.sp");
    let ctx = sky130_open_ctx();

    let block = sky130pdk::atoll::MosTile {
        w: 1_680,
        nf: 3,
        len: MosLength::L150,
    };

    ctx.write_layout(block, gds_path)
        .expect("failed to write layout");

    Spectre::default()
        .write_netlist_to_file(&ctx, block, spice_path, NetlistOptions::default())
        .expect("failed to write netlist");
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
        cell: &mut CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
        use atoll::grid::*;
        let stack = cell
            .ctx
            .get_installation::<LayerStack<Sky130AtollLayer>>()
            .unwrap();
        let grid = DebugRoutingGrid::new(RoutingGrid::new((*stack).clone(), 0..stack.len(), 10, 2));
        cell.draw(grid)?;
        Ok(())
    }
}
