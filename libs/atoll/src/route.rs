//! Routing interfaces and implementations.

use crate::abs::{GridCoord, TrackCoord};
use crate::grid::{PdkLayer, RoutingState};
use crate::{NetId, PointState};
use indexmap::{map::Entry, IndexMap};
use num::Zero;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rustc_hash::FxHasher;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::hash::{BuildHasherDefault, Hash};
use substrate::context::PdkContext;
use substrate::layout::element::Shape;
use substrate::pdk::Pdk;

/// A path of grid-coordinates.
pub type Path = Vec<GridSegment>;

/// A segment of a path.
pub type GridSegment = (GridCoord, GridCoord);

/// An ATOLL router.
pub trait Router: Send + Sync {
    /// Returns routes that connect the given nets.
    fn route(
        &self,
        routing_state: &mut RoutingState<PdkLayer>,
        to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path>;
}

/// A router that greedily routes net groups one at a time.
#[derive(Clone, Debug, Copy, Default)]
pub struct GreedyRouter {
    seed: [u8; 32],
}

impl GreedyRouter {
    /// Creates a new [`GreedyRouter`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new [`GreedyRouter`] with the given seed.
    pub fn with_seed(seed: [u8; 32]) -> Self {
        Self { seed }
    }
}

/// A node in the traversal of a [`GreedyRouter`].
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RoutingNode {
    pub(crate) coord: GridCoord,
    pub(crate) has_via: bool,
}

// BEGIN DIJKSTRA IMPL (taken from https://docs.rs/pathfinding/latest/src/pathfinding/directed/dijkstra.rs.html)
type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

struct SmallestHolder<K> {
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl<K: PartialEq> Eq for SmallestHolder<K> {}

impl<K: Ord> PartialOrd for SmallestHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

fn reverse_path<N, V, F>(parents: &FxIndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i).map(|(node, value)| {
            i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();
    // Collecting the going through the vector is needed to revert the path because the
    // unfold iterator is not double-ended due to its iterative nature.
    path.into_iter().rev().cloned().collect()
}

fn dijkstra<'a, N, C, FN, IN, FS>(
    start: impl IntoIterator<Item = &'a N>,
    mut successors: FN,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone + 'a,
    C: Zero + Ord + Copy,
    FN: FnMut(&N, &[N]) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    dijkstra_internal(start, &mut successors, &mut success)
}

pub(crate) fn dijkstra_internal<'a, N, C, FN, IN, FS>(
    start: impl IntoIterator<Item = &'a N>,
    successors: &mut FN,
    success: &mut FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone + 'a,
    C: Zero + Ord + Copy,
    FN: FnMut(&N, &[N]) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    let (parents, reached) = run_dijkstra(start, successors, success);
    reached.map(|target| {
        (
            reverse_path(&parents, |&(p, _)| p, target),
            parents.get_index(target).unwrap().1 .1,
        )
    })
}

fn run_dijkstra<'a, N, C, FN, IN, FS>(
    start: impl IntoIterator<Item = &'a N>,
    successors: &mut FN,
    stop: &mut FS,
) -> (FxIndexMap<N, (usize, C)>, Option<usize>)
where
    N: Eq + Hash + Clone + 'a,
    C: Zero + Ord + Copy,
    FN: FnMut(&N, &[N]) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    for (i, node) in start.into_iter().enumerate() {
        to_see.push(SmallestHolder {
            cost: Zero::zero(),
            index: i,
        });
        parents.insert(node.clone(), (usize::max_value(), Zero::zero()));
    }
    let mut target_reached = None;
    while let Some(SmallestHolder { cost, index }) = to_see.pop() {
        let successors = {
            let (node, _) = parents.get_index(index).unwrap();
            if stop(node) {
                target_reached = Some(index);
                break;
            }
            let path = reverse_path(&parents, |&(p, _)| p, index);
            successors(node, &path)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let n;
            match parents.entry(successor) {
                Entry::Vacant(e) => {
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Entry::Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestHolder {
                cost: new_cost,
                index: n,
            });
        }
    }
    (parents, target_reached)
}
// END DIJKSTRA IMPL

impl Router for GreedyRouter {
    fn route(
        &self,
        state: &mut RoutingState<PdkLayer>,
        mut to_connect: Vec<Vec<NetId>>,
    ) -> Vec<Path> {
        let mut rng = StdRng::from_seed(self.seed);
        to_connect.shuffle(&mut rng);
        // remove nodes from the to connect list that are not on the grid
        // and relabel them to ones that are on the grid.
        for group in to_connect.iter_mut() {
            group.shuffle(&mut rng);
            *group = group
                .iter()
                .copied()
                .filter(|&n| state.find(n).is_some())
                .collect::<Vec<_>>();

            // Router assumes that the first element of list is the root element.
            if let Some(first_on_grid) = group.first_mut().copied() {
                if let Some(root_idx) = group
                    .iter()
                    .position(|net| *net == state.roots[&first_on_grid])
                {
                    group.swap(0, root_idx);
                } else {
                    state.relabel_net(first_on_grid, state.roots[&first_on_grid]);
                    group[0] = state.roots[&first_on_grid];
                }
            }
        }

        let mut paths = Vec::new();
        for group in to_connect.iter() {
            if group.len() <= 1 {
                // skip empty or one node groups
                continue;
            }
            let group_root = state.roots[&group[0]];

            let mut remaining_nets: HashSet<_> = group[1..].iter().collect();

            while !remaining_nets.is_empty() {
                let start = state
                    .find_all(group_root)
                    .into_iter()
                    .map(|coord| RoutingNode {
                        coord,
                        has_via: state.has_via(coord),
                    })
                    .collect::<Vec<_>>();
                let path = dijkstra(
                    start.iter(),
                    |s, path| state.successors(*s, path, group_root).into_iter(),
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
                            state[nodes[0].coord] = PointState::Routed {
                                net: group_root,
                                has_via: true,
                            };
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
                            state[nodes[1].coord] = PointState::Routed {
                                net: group_root,
                                has_via: true,
                            };
                            if let Some(requires) = ilt.requires {
                                state[requires] = PointState::Reserved { net: group_root };
                            }
                        }
                        Ordering::Equal => {
                            for x in std::cmp::min(nodes[0].coord.x, nodes[1].coord.x)
                                ..=std::cmp::max(nodes[0].coord.x, nodes[1].coord.x)
                            {
                                for y in std::cmp::min(nodes[0].coord.y, nodes[1].coord.y)
                                    ..=std::cmp::max(nodes[0].coord.y, nodes[1].coord.y)
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
pub trait ViaMaker<PDK: Pdk>: Send + Sync {
    /// Draws a via from the given track coordinate to the layer below.
    fn draw_via(&self, ctx: PdkContext<PDK>, track_coord: TrackCoord) -> Vec<Shape>;
}
