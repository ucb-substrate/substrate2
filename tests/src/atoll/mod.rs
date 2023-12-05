use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
use substrate::block::Block;
use substrate::io::LayoutType;
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};

#[test]
fn atoll_debug_routing_grid() {
    let gds_path = get_path("atoll_debug_routing_grid", "layout.gds");
    let ctx = sky130_open_ctx();

    // Imports a hard macro from a GDS file.
    ctx.write_layout(Sky130DebugRoutingGrid, gds_path)
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
        _io: &mut <<Self as Block>::Io as LayoutType>::Builder,
        cell: &mut CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
        use atoll::grid::*;
        let stack = cell.ctx.get_installation::<LayerStack<PdkLayer>>().unwrap();
        let grid = DebugRoutingGrid::new(RoutingGrid::new((*stack).clone(), 0..stack.len(), 10, 2));
        cell.draw(grid)?;
        Ok(())
    }
}
