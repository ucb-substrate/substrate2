//! Routing interfaces and implementations.

use crate::abs::{GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use pathfinding::prelude::dijkstra;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use substrate::geometry::side::Side;
use substrate::layout;
use substrate::pdk::Pdk;

/// A path of grid-coordinates.
pub type Path = Vec<GridSegment>;

pub type GridSegment = (GridCoord, GridCoord);

/// An ATOLL router.
pub trait Router {
    // todo: perhaps add way to translate nodes to net IDs
    /// Returns routes that connect the given nets.
    fn route(
        &self,
        routing_state: &mut RoutingState<PdkLayer>,
        to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path>;
}

/// A router that greedily routes net groups one at a time.
pub struct GreedyRouter;

/// A node in the traversal of a [`GreedyRouter`].
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RoutingNode {
    pub(crate) coord: GridCoord,
    pub(crate) has_via: bool,
    /// The side from which we got to this routing node.
    ///
    /// Do not want to go back to where we came from, especially
    /// after skipping invalid via placements.
    pub(crate) prev_side: Option<Side>,
}

impl Router for GreedyRouter {
    fn route(
        &self,
        state: &mut RoutingState<PdkLayer>,
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
        // and relabel them to ones that are on the grid.
        for group in to_connect.iter_mut() {
            *group = group
                .iter()
                .copied()
                .filter(|&n| state.find(n).is_some())
                .collect::<Vec<_>>();
            if let Some(first_on_grid) = group.first_mut() {
                state.relabel_net(*first_on_grid, state.roots[first_on_grid]);
                *first_on_grid = state.roots[first_on_grid];
            }
        }

        let mut paths = Vec::new();
        for group in to_connect.iter() {
            if group.len() <= 1 {
                // skip empty or one node groups
                continue;
            }
            let group_root = state.roots[&group[0]];

            let locs = group
                .iter()
                .filter_map(|n| state.find(*n))
                .collect::<Vec<_>>();

            let mut remaining_nets: HashSet<_> = group[1..].iter().collect();

            while !remaining_nets.is_empty() {
                let start = RoutingNode {
                    coord: locs[0],
                    has_via: false,
                    prev_side: None,
                };
                let path = dijkstra(
                    &start,
                    |s| state.successors(*s, group_root).into_iter(),
                    |node| {
                        if let PointState::Routed { net, .. } = state[node.coord] {
                            remaining_nets.contains(&net)
                        } else {
                            false
                        }
                    },
                )
                .unwrap_or_else(|| panic!("cannot connect all nodes in group {:?}", group_root))
                .0;

                let mut to_remove = HashSet::new();

                let mut segment_path = Vec::new();
                for nodes in path.windows(2) {
                    if state.are_routed_for_same_net(nodes[0].coord, nodes[1].coord) {
                        continue;
                    }
                    segment_path.push((nodes[0].coord, nodes[1].coord));
                }

                for node in path.iter() {
                    if let PointState::Routed { net, .. } = state[node.coord] {
                        to_remove.insert(net);
                    }
                }

                for nodes in path.windows(2) {
                    match nodes[0].coord.layer.cmp(&nodes[1].coord.layer) {
                        Ordering::Less => {
                            let ilt = state.ilt_up(nodes[0].coord).unwrap();
                            state[nodes[1].coord] = PointState::Routed {
                                net: group_root,
                                has_via: true,
                            };
                            if let Some(requires) = ilt.requires {
                                state[requires] = PointState::Reserved { net: group_root };
                            }
                        }
                        Ordering::Greater => {
                            let ilt = state.ilt_down(nodes[0].coord).unwrap();
                            state[nodes[0].coord] = PointState::Routed {
                                net: group_root,
                                has_via: true,
                            };
                            if let Some(requires) = ilt.requires {
                                state[requires] = PointState::Reserved { net: group_root };
                            }
                        }
                        Ordering::Equal => {
                            for x in std::cmp::min(nodes[0].coord.x, nodes[1].coord.x)
                                ..=std::cmp::min(nodes[0].coord.x, nodes[1].coord.x)
                            {
                                for y in std::cmp::min(nodes[0].coord.y, nodes[1].coord.y)
                                    ..=std::cmp::min(nodes[0].coord.y, nodes[1].coord.y)
                                {
                                    let next = GridCoord {
                                        x,
                                        y,
                                        layer: nodes[0].coord.layer,
                                    };
                                    if let PointState::Routed { net, .. } = state[next] {
                                        to_remove.insert(net);
                                    }
                                    state[next] = PointState::Routed {
                                        net: group_root,
                                        has_via: state.has_via(next),
                                    };
                                }
                            }
                        }
                    }
                }

                for net in to_remove {
                    state.relabel_net(net, group_root);
                    remaining_nets.remove(&net);
                }
                paths.push(segment_path);
            }
        }

        paths
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
