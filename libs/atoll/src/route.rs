/// Routing interfaces and implementations.
use crate::abs::{Abstract, GridCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::NetId;

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
                x: 0,
                y: 0,
            },
            GridCoord {
                layer: 0,
                x: 0,
                y: 4,
            },
        ]]
    }
}
