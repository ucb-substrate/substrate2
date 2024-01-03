/// Routing interfaces and implementations.
use crate::abs::{Abstract, GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use pathfinding::prelude::{build_path, dijkstra, dijkstra_all};
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
    fn route(
        &self,
        mut state: RoutingState<PdkLayer>,
        mut to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path> {
        // build roots map
        let mut roots = HashMap::new();
        for seq in to_connect.iter() {
            for node in seq.iter() {
                roots.insert(*node, seq[0]);
            }
        }
        state.roots = roots;

        // remove nodes from the to connect list that are not on the grid
        for group in to_connect.iter_mut() {
            *group = group
                .iter()
                .copied()
                .filter(|&n| state.find(n).is_some())
                .collect::<Vec<_>>();
        }

        let mut paths = Vec::new();
        for group in to_connect.iter() {
            if group.len() <= 1 {
                // skip empty or one node groups
                continue;
            }

            loop {
                let locs = group
                    .iter()
                    .filter_map(|n| state.find(*n))
                    .collect::<Vec<_>>();

                let mut spt = dijkstra_all(&locs[0], |s| state.successors(*s, group[0]));

                // a bit of a hack: insert this now for making the next line easier
                // remove it when we go to building a path.
                assert!(!spt.contains_key(&locs[0]));
                spt.insert(locs[0], (locs[0], 0));

                let (cost, nearest_loc, node) = match group
                    .iter()
                    .zip(locs.iter())
                    .filter_map(|(node, loc)| {
                        if !spt.contains_key(loc) {
                            panic!(
                                "node {node:?} (group {:?}) was unreachable for state {state:#?}",
                                group[0]
                            );
                        }
                        if spt[loc].1 == 0 {
                            None
                        } else {
                            Some((spt[loc].1, loc, node))
                        }
                    })
                    .min()
                {
                    None => {
                        // all node fragments have been connected
                        break;
                    }
                    Some(x) => x,
                };

                spt.remove(&locs[0]);
                let path = build_path(nearest_loc, &spt);
                if path.len() <= 1 {
                    panic!("node was unreachable");
                }
                for coord in path.iter() {
                    state[*coord] = PointState::Routed { net: group[0] };
                }
                for x in path.windows(2) {
                    if x[0].layer < x[1].layer {
                        let ilt = state.ilt_up(x[0]).unwrap();
                        if let Some(requires) = ilt.requires {
                            state[requires] = PointState::Reserved { net: group[0] };
                        }
                    } else if x[0].layer > x[1].layer {
                        let ilt = state.ilt_down(x[0]).unwrap();
                        if let Some(requires) = ilt.requires {
                            state[requires] = PointState::Reserved { net: group[0] };
                        }
                    }
                }
                paths.push(path);
            }
        }

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
