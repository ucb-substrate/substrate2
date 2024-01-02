/// Routing interfaces and implementations.
use crate::abs::{Abstract, GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::NetId;
use substrate::layout;
use substrate::pdk::Pdk;

pub type Path = Vec<GridCoord>;

pub trait Router {
    // todo: perhaps add way to translate nodes to net IDs
    fn route(
        &self,
        routing_state: RoutingState<PdkLayer>,
        to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path>;
}

pub struct GreedyBfsRouter;

impl Router for GreedyBfsRouter {
    fn route(&self, mut state: RoutingState<PdkLayer>, to_connect: Vec<Vec<NetId>>) -> Vec<Path> {
        vec![vec![
            GridCoord {
                layer: 0,
                x: 2,
                y: 0,
            },
            GridCoord {
                layer: 0,
                x: 0,
                y: 0,
            },
            GridCoord {
                layer: 0,
                x: 0,
                y: 4,
            },
            GridCoord {
                layer: 1,
                x: 0,
                y: 4,
            },
            GridCoord {
                layer: 1,
                x: 3,
                y: 4,
            },
            GridCoord {
                layer: 2,
                x: 3,
                y: 4,
            },
            GridCoord {
                layer: 2,
                x: 3,
                y: 0,
            },
        ]]
    }
}

pub trait ViaMaker<PDK: Pdk> {
    fn draw_via(
        &self,
        cell: &mut layout::CellBuilder<PDK>,
        track_coord: TrackCoord,
    ) -> substrate::error::Result<()>;
}
