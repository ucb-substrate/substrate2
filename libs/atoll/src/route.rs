/// Routing interfaces and implementations.
use crate::abs::{Abstract, GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use pathfinding::prelude::dijkstra;
use std::collections::HashMap;
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
        println!("state = {state:?}, to_connect = {to_connect:?}");
        // build roots map
        let mut roots = HashMap::new();
        for seq in to_connect.iter() {
            for node in seq.iter() {
                roots.insert(*node, seq[0]);
            }
        }
        state.roots = roots;

        let mut paths = Vec::new();
        for group in to_connect.iter() {
            for node in group.iter().skip(1) {
                let start = match state.find(*node) {
                    Some(c) => c,
                    None => {
                        println!("no starting point found for {node:?}; skipping");
                        continue;
                    }
                };
                let (path, _) = dijkstra(
                    &start,
                    |s| state.successors(*s, *node),
                    |s| state.forms_new_connection_for_net(*s, *node),
                )
                .expect("no path found");
                for coord in path.iter() {
                    state[*coord] = PointState::Routed { net: *node };
                }
                paths.push(path);
            }
        }

        println!("paths = {paths:?}");
        paths
    }
}

pub trait ViaMaker<PDK: Pdk> {
    fn draw_via(
        &self,
        cell: &mut layout::CellBuilder<PDK>,
        track_coord: TrackCoord,
    ) -> substrate::error::Result<()>;
}
