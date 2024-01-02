/// Routing interfaces and implementations.
use crate::abs::{Abstract, GridCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use pathfinding::prelude::dijkstra;
use std::collections::HashMap;

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
        // build roots map
        let mut roots = HashMap::new();
        for seq in to_connect.iter() {
            for node in seq.iter() {
                roots.insert(*node, seq[0]);
            }
        }
        state.roots = roots;

        for group in to_connect.iter() {
            for node in group.iter().skip(1) {
                let start = state.find(*node).unwrap();
                let (path, _) = dijkstra(
                    &start,
                    |s| state.successors(*s, *node),
                    |s| state.is_routed_for_net(*s, *node),
                )
                .expect("no path found");
                for coord in path {
                    state[coord] = PointState::Routed { net: *node };
                }
            }
        }

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
