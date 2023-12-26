use crate::paths::get_path;
use crate::shared::pdk::sky130_open_ctx;
use atoll::AtollBuilder;
use geometry::align::{AlignBboxMut, AlignMode};
use geometry::bbox::Bbox;
use serde::{Deserialize, Serialize};
use sky130pdk::atoll::{MosLength, MosTileIo, NmosTile, Sky130AtollLayer};
use sky130pdk::{Sky130CommercialSchema, Sky130Pdk};
use spice::netlist::NetlistOptions;
use spice::Spice;
use substrate::block::Block;
use substrate::io::layout::HardwareType;
use substrate::io::schematic::Bundle;
use substrate::io::{ArrayData, Signal};
use substrate::layout::tiling::{GridTiler, Tile};
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::schematic;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::{ExportsNestedData, Schematic};

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
}

#[derive(Block, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
#[substrate(io = "()")]
pub struct Sky130NmosTileAutoroute;

impl ExportsNestedData for Sky130NmosTileAutoroute {
    type NestedData = Vec<schematic::Instance<NmosTile>>;
}
impl Schematic<Sky130Pdk> for Sky130NmosTileAutoroute {
    fn schematic(
        &self,
        io: &Bundle<<Self as Block>::Io>,
        cell: &mut substrate::schematic::CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let block = sky130pdk::atoll::NmosTile {
            w: 1_680,
            nf: 3,
            len: MosLength::L150,
        };
        let sd = cell.signal("sd", Signal::new());
        let g = cell.signal("g", Signal::new());
        let b = cell.signal("b", Signal::new());
        Ok((0..6)
            .map(|_| {
                let instance = cell.instantiate(block.clone());
                for i in 0..4 {
                    cell.connect(sd, instance.io().sd[i]);
                }
                cell.connect(g, instance.io().g);
                cell.connect(b, instance.io().b);
                instance
            })
            .collect())
    }
}

impl ExportsLayoutData for Sky130NmosTileAutoroute {
    type LayoutData = ();
}

impl Layout<Sky130Pdk> for Sky130NmosTileAutoroute {
    fn layout(
        &self,
        _io: &mut <<Self as Block>::Io as HardwareType>::Builder,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let mut cell = AtollBuilder::new(*self, cell)?;
        let mut tiler = GridTiler::new();
        let mut keys = Vec::new();

        for (i, inst) in cell.cell().data().into_iter().enumerate() {
            let mut atoll_inst = cell.linked_generate(inst);
            keys.push(tiler.push(Tile::from_bbox(atoll_inst)));
            if i == 2 {
                tiler.end_row();
            }
        }

        let tiler = tiler.tile();

        for key in keys {
            cell.draw((*tiler.get(key).unwrap()).clone())?;
        }

        cell.route()?;

        Ok(())
    }
}

#[test]
fn sky130_atoll_nmos_tile_autoroute() {
    let gds_path = get_path("sky130_atoll_nmos_tile_autoroute", "layout.gds");
    let ctx = sky130_open_ctx();

    ctx.write_layout(Sky130NmosTileAutoroute, gds_path)
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
        let stack = cell
            .ctx
            .get_installation::<LayerStack<Sky130AtollLayer>>()
            .unwrap();
        let grid = DebugRoutingGrid::new(RoutingGrid::new((*stack).clone(), 0..stack.len(), 10, 2));
        cell.draw(grid)?;
        Ok(())
    }
}
