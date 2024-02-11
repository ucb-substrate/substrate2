//! Routing interfaces and implementations.

use crate::abs::{GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use pathfinding::prelude::{build_path, dijkstra_all};
use std::cmp::Ordering;
use std::collections::HashMap;
use substrate::layout;
use substrate::pdk::Pdk;

/// A path of grid-coordinates.
pub type Path = Vec<GridCoord>;

/// An ATOLL router.
pub trait Router {
    // todo: perhaps add way to translate nodes to net IDs
    /// Returns routes that connect the given nets.
    fn route(
        &self,
        routing_state: RoutingState<PdkLayer>,
        to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path>;
}

/// A router that greedily routes net groups one at a time.
pub struct GreedyRouter;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct RoutingNode {
    pub(crate) coord: GridCoord,
    pub(crate) has_via: bool,
}

impl Router for GreedyRouter {
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

                let has_via = state.has_via(locs[0]);
                let start = RoutingNode {
                    coord: locs[0],
                    has_via,
                };
                let mut spt = dijkstra_all(&start, |s| state.successors(*s, group[0]).into_iter());

                // a bit of a hack: insert this now for making the next line easier
                // remove it when we go to building a path.
                assert!(!spt.contains_key(&start));
                spt.insert(start, (start, 0));

                let (_cost, nearest_loc, _node) = match group
                    .iter()
                    .zip(locs.iter())
                    .flat_map(|(node, loc)| {
                        let mut nodes = Vec::new();
                        if let Some(spt_loc) = spt.get(&RoutingNode {
                            coord: *loc,
                            has_via: false,
                        }) {
                            if spt_loc.1 != 0 {
                                nodes.push((
                                    spt_loc.1,
                                    RoutingNode {
                                        coord: *loc,
                                        has_via: false,
                                    },
                                    node,
                                ));
                            }
                        } else if let Some(spt_loc) = spt.get(&RoutingNode {
                            coord: *loc,
                            has_via: true,
                        }) {
                            if spt_loc.1 != 0 {
                                nodes.push((
                                    spt_loc.1,
                                    RoutingNode {
                                        coord: *loc,
                                        has_via: true,
                                    },
                                    node,
                                ));
                            }
                        } else {
                            panic!(
                                "node {node:?} (group {:?}) was unreachable for state {state:#?}",
                                group[0]
                            );
                        }
                        nodes
                    })
                    .min()
                {
                    None => {
                        // all node fragments have been connected
                        break;
                    }
                    Some(x) => x,
                };

                spt.remove(&start);
                let path = build_path(&nearest_loc, &spt);
                if path.len() <= 1 {
                    panic!("node was unreachable");
                }
                for node in path.iter() {
                    state[node.coord] = PointState::Routed {
                        net: group[0],
                        has_via: false,
                    };
                }
                for x in path.windows(2) {
                    match x[0].coord.layer.cmp(&x[1].coord.layer) {
                        Ordering::Less => {
                            let ilt = state.ilt_up(x[0].coord).unwrap();
                            state[x[1].coord] = PointState::Routed {
                                net: group[0],
                                has_via: true,
                            };
                            if let Some(requires) = ilt.requires {
                                state[requires] = PointState::Reserved { net: group[0] };
                            }
                        }
                        Ordering::Greater => {
                            let ilt = state.ilt_down(x[0].coord).unwrap();
                            state[x[0].coord] = PointState::Routed {
                                net: group[0],
                                has_via: true,
                            };
                            if let Some(requires) = ilt.requires {
                                state[requires] = PointState::Reserved { net: group[0] };
                            }
                        }
                        Ordering::Equal => {}
                    }
                }
                paths.push(path);
            }
        }

        paths
            .into_iter()
            .map(|path| path.into_iter().map(|node| node.coord).collect())
            .collect()
    }
}

/// An type capable of drawing vias.
pub trait ViaMaker<PDK: Pdk> {
    /// Draws a via from the given track coordinate to the layer below.
    fn draw_via(
        &self,
        cell: &mut layout::CellBuilder<PDK>,
        track_coord: TrackCoord,
    ) -> substrate::error::Result<()>;
}
