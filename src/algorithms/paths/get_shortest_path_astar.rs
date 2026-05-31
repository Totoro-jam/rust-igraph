//! Single-pair shortest path via A* (`ALGO-SP-036`).
//!
//! Port of `igraph_get_shortest_path_astar()` from
//! `references/igraph/src/paths/astar.c`. A* finds one shortest path from
//! `from` to `to` by exploring vertices in order of an *estimated* total
//! path length: the actual distance from the source plus a heuristic
//! estimate of the remaining distance to the target. When the heuristic is
//! admissible (never overestimates the true remaining distance) the result
//! is a genuine shortest path; with the null heuristic A* degenerates to
//! Dijkstra's algorithm.
//!
//! The path is returned as both the vertex sequence (including `from` and
//! `to`) and the edge sequence along it, sharing the [`ShortestPath`]
//! struct with [`super::get_shortest_path`]. Empty vectors signal an
//! unreachable target; `from == to` yields `[from]` with no edges, exactly
//! as upstream.
//!
//! Unlike [`super::get_shortest_path`], A* supports only **non-negative**
//! weights (there is no Bellman-Ford fallback): a negative weight is an
//! error. Edges with positive-infinite weight are ignored, matching
//! upstream.
//!
//! When several shortest paths of equal length exist, an arbitrary one is
//! returned; the exact choice depends on heap tie-breaking and is not a
//! stable cross-implementation observable. The total path *length* is.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::dijkstra::DijkstraMode;
use super::get_shortest_path::ShortestPath;

/// A heuristic estimating the distance from a candidate vertex (first
/// argument) to the target vertex (second argument). Must be admissible
/// (never overestimate the true remaining distance) for the result to be
/// a true shortest path.
pub type AstarHeuristic<'a> = dyn Fn(VertexId, VertexId) -> f64 + 'a;

/// Mode-aware incident edges, mirroring upstream's lazy incidence list
/// (`igraph_lazy_inclist_init(_, _, mode, IGRAPH_LOOPS_TWICE)`). For
/// undirected graphs every mode collapses to the merged adjacency.
fn incident_for_mode(graph: &Graph, v: VertexId, mode: DijkstraMode) -> IgraphResult<Vec<EdgeId>> {
    if !graph.is_directed() {
        return graph.incident(v);
    }
    match mode {
        DijkstraMode::Out => graph.incident(v),
        DijkstraMode::In => graph.incident_in(v),
        DijkstraMode::All => {
            let mut out = graph.incident(v)?;
            out.extend(graph.incident_in(v)?);
            Ok(out)
        }
    }
}

/// An open-set entry: the priority is `est` (g + heuristic) ordered so the
/// smallest estimate pops first; `g` is the actual `from -> vertex`
/// distance recorded at push time, used to drop stale entries.
struct State {
    est: f64,
    g: f64,
    vertex: VertexId,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.est.total_cmp(&other.est) == Ordering::Equal
    }
}
impl Eq for State {}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so `BinaryHeap` (a max-heap) yields the smallest estimate.
        other.est.total_cmp(&self.est)
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Calculate a single shortest path from `from` to `to` using A*.
///
/// Returns the path as both its vertex sequence (source first, target
/// last) and the edge sequence along it. If several shortest paths exist,
/// an arbitrary one is returned. The path is empty when `to` is
/// unreachable; when `from == to` the vertex sequence is `[from]` and the
/// edge sequence is empty.
///
/// `weights` are optional edge weights; `None` treats every edge as weight
/// `1` (an unweighted graph). All weights must be non-negative and
/// non-NaN; positive-infinite weights mark unusable edges. On directed
/// graphs `mode` selects the followed adjacency
/// ([`DijkstraMode::Out`] / [`DijkstraMode::In`] / [`DijkstraMode::All`]);
/// it is ignored on undirected graphs.
///
/// `heuristic`, if given, estimates the remaining distance from a
/// candidate vertex to `to`. It must be *admissible* (never overestimate)
/// for the returned path to be a true shortest path. `None` uses the null
/// heuristic (always `0`), making A* equivalent to Dijkstra.
///
/// # Errors
///
/// Returns [`IgraphError::VertexOutOfRange`] if `from` or `to` is not a
/// valid vertex, or [`IgraphError::InvalidArgument`] if `weights` is
/// provided and its length differs from the edge count, contains NaN, or
/// contains a negative value.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_path_astar, DijkstraMode};
///
/// let mut g = Graph::new(4, false).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 2).unwrap(); // edge 1
/// g.add_edge(2, 3).unwrap(); // edge 2
///
/// // Null heuristic (None) behaves exactly like Dijkstra.
/// let p = get_shortest_path_astar(&g, 0, 3, None, DijkstraMode::Out, None).unwrap();
/// assert_eq!(p.vertices, vec![0, 1, 2, 3]);
/// assert_eq!(p.edges, vec![0, 1, 2]);
///
/// // An admissible heuristic (here: |target - v|, a lower bound on the
/// // number of hops) yields the same shortest path.
/// let h = |v: u32, to: u32| f64::from(to.abs_diff(v));
/// let q = get_shortest_path_astar(&g, 0, 3, None, DijkstraMode::Out, Some(&h)).unwrap();
/// assert_eq!(q.vertices, vec![0, 1, 2, 3]);
/// ```
pub fn get_shortest_path_astar(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
    heuristic: Option<&AstarHeuristic<'_>>,
) -> IgraphResult<ShortestPath> {
    let n = graph.vcount();
    if from >= n {
        return Err(IgraphError::VertexOutOfRange { id: from, n });
    }
    if to >= n {
        return Err(IgraphError::VertexOutOfRange { id: to, n });
    }

    if let Some(w) = weights {
        let m = graph.ecount();
        if w.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "get_shortest_path_astar: weights length {} != edge count {m}",
                w.len()
            )));
        }
        for (e, &x) in w.iter().enumerate() {
            if x.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "get_shortest_path_astar: weight at edge {e} is NaN"
                )));
            }
            if x < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "get_shortest_path_astar: weight at edge {e} is negative ({x})"
                )));
            }
        }
    }

    let h = |v: VertexId| heuristic.map_or(0.0, |f| f(v, to));

    let n_usize = n as usize;
    let mut dists = vec![f64::INFINITY; n_usize];
    // parent_eids[v] = 1 + edge id of v's inbound tree edge; 0 = unreached.
    let mut parent_eids: Vec<EdgeId> = vec![0; n_usize];

    dists[from as usize] = 0.0;
    let mut queue: BinaryHeap<State> = BinaryHeap::new();
    queue.push(State {
        est: h(from),
        g: 0.0,
        vertex: from,
    });

    let mut found = false;
    while let Some(State { g, vertex: u, .. }) = queue.pop() {
        // Stale entry: a cheaper route to `u` was found after this was
        // pushed. Skip it (lazy deletion in place of decrease-key).
        if g > dists[u as usize] {
            continue;
        }
        if u == to {
            found = true;
            break;
        }
        for edge in incident_for_mode(graph, u, mode)? {
            let weight = match weights {
                Some(w) => {
                    let x = w[edge as usize];
                    if x.is_infinite() {
                        continue;
                    }
                    x
                }
                None => 1.0,
            };
            let v = graph.edge_other(edge, u)?;
            let altdist = dists[u as usize] + weight;
            if altdist < dists[v as usize] {
                dists[v as usize] = altdist;
                parent_eids[v as usize] = edge + 1;
                queue.push(State {
                    est: altdist + h(v),
                    g: altdist,
                    vertex: v,
                });
            }
        }
    }

    if !found {
        // `to` unreachable from `from` (and `from != to`): empty path.
        return Ok(ShortestPath {
            vertices: Vec::new(),
            edges: Vec::new(),
        });
    }

    // Reconstruct by walking parent edges back from `to`.
    let mut size = 0_usize;
    let mut act = to;
    while parent_eids[act as usize] != 0 {
        size += 1;
        let edge = parent_eids[act as usize] - 1;
        act = graph.edge_other(edge, act)?;
    }

    let mut vertices = vec![0 as VertexId; size + 1];
    let mut edges = vec![0 as EdgeId; size];
    vertices[size] = to;

    let mut idx = size;
    let mut act = to;
    while parent_eids[act as usize] != 0 {
        idx -= 1;
        let edge = parent_eids[act as usize] - 1;
        act = graph.edge_other(edge, act)?;
        vertices[idx] = act;
        edges[idx] = edge;
    }

    Ok(ShortestPath { vertices, edges })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    /// Admissible heuristic for a path/grid where vertex ids are laid out
    /// so `|to - v|` lower-bounds the hop count.
    fn linear_h(to: VertexId) -> impl Fn(VertexId, VertexId) -> f64 {
        move |v: VertexId, _to: VertexId| f64::from(to.abs_diff(v))
    }

    #[test]
    fn from_equals_to() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let p = get_shortest_path_astar(&g, 1, 1, None, DijkstraMode::Out, None).unwrap();
        assert_eq!(p.vertices, vec![1]);
        assert!(p.edges.is_empty());
    }

    #[test]
    fn simple_path_unweighted_null_heuristic() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        let p = get_shortest_path_astar(&g, 0, 3, None, DijkstraMode::Out, None).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 2, 3]);
        assert_eq!(p.edges, vec![0, 1, 2]);
    }

    #[test]
    fn admissible_heuristic_matches_dijkstra() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        let h = linear_h(3);
        let p = get_shortest_path_astar(&g, 0, 3, None, DijkstraMode::Out, Some(&h)).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 2, 3]);
        assert_eq!(p.edges, vec![0, 1, 2]);
    }

    #[test]
    fn picks_shortcut() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 3).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2 shortcut
        let p = get_shortest_path_astar(&g, 0, 3, None, DijkstraMode::Out, None).unwrap();
        assert_eq!(p.vertices, vec![0, 3]);
        assert_eq!(p.edges, vec![2]);
    }

    #[test]
    fn weighted_prefers_cheaper_route() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 3).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2 direct but expensive
        let w = vec![1.0, 1.0, 10.0];
        let p = get_shortest_path_astar(&g, 0, 3, Some(&w), DijkstraMode::Out, None).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 3]);
        assert_eq!(p.edges, vec![0, 1]);
    }

    #[test]
    fn infinite_weight_edge_ignored() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 2).unwrap(); // 0 direct, but infinite
        g.add_edge(0, 1).unwrap(); // 1
        g.add_edge(1, 2).unwrap(); // 2
        let w = vec![f64::INFINITY, 1.0, 1.0];
        let p = get_shortest_path_astar(&g, 0, 2, Some(&w), DijkstraMode::Out, None).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 2]);
        assert_eq!(p.edges, vec![1, 2]);
    }

    #[test]
    fn unreachable_returns_empty() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let p = get_shortest_path_astar(&g, 0, 2, None, DijkstraMode::Out, None).unwrap();
        assert!(p.vertices.is_empty());
        assert!(p.edges.is_empty());
    }

    #[test]
    fn directed_mode_in_follows_reverse() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        let out = get_shortest_path_astar(&g, 0, 2, None, DijkstraMode::Out, None).unwrap();
        assert_eq!(out.vertices, vec![0, 1, 2]);
        let none = get_shortest_path_astar(&g, 2, 0, None, DijkstraMode::Out, None).unwrap();
        assert!(none.vertices.is_empty());
        let inp = get_shortest_path_astar(&g, 2, 0, None, DijkstraMode::In, None).unwrap();
        assert_eq!(inp.vertices, vec![2, 1, 0]);
        assert_eq!(inp.edges, vec![1, 0]);
    }

    #[test]
    fn directed_mode_all_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let p = get_shortest_path_astar(&g, 2, 0, None, DijkstraMode::All, None).unwrap();
        assert_eq!(p.vertices, vec![2, 1, 0]);
    }

    #[test]
    fn invalid_endpoints_error() {
        let g = Graph::new(2, false).unwrap();
        assert!(get_shortest_path_astar(&g, 5, 0, None, DijkstraMode::Out, None).is_err());
        assert!(get_shortest_path_astar(&g, 0, 5, None, DijkstraMode::Out, None).is_err());
    }

    #[test]
    fn weights_length_mismatch_errors() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![1.0, 2.0];
        assert!(get_shortest_path_astar(&g, 0, 1, Some(&w), DijkstraMode::Out, None).is_err());
    }

    #[test]
    fn weights_nan_errors() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![f64::NAN];
        assert!(get_shortest_path_astar(&g, 0, 1, Some(&w), DijkstraMode::Out, None).is_err());
    }

    #[test]
    fn weights_negative_errors() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![-1.0];
        assert!(get_shortest_path_astar(&g, 0, 1, Some(&w), DijkstraMode::Out, None).is_err());
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::algorithms::paths::get_shortest_path::get_shortest_path;
    use crate::create;
    use proptest::prelude::*;

    fn arb_graph(max_v: u32) -> impl Strategy<Value = Graph> {
        (2..=max_v).prop_flat_map(|n| {
            let max_e = (n as usize)
                .saturating_mul(n.saturating_sub(1) as usize)
                .min(20);
            proptest::collection::vec((0..n, 0..n), 0..=max_e).prop_map(move |edges| {
                let edge_tuples: Vec<(u32, u32)> = edges.into_iter().collect();
                create(&edge_tuples, n, false).expect("arb graph")
            })
        })
    }

    proptest! {
        /// A non-empty A* result is a genuine simple walk from `from` to
        /// `to`: endpoints match, edges join consecutive vertices, and no
        /// vertex repeats.
        #[test]
        fn path_is_a_valid_simple_walk(
            g in arb_graph(6),
            from in 0u32..6,
            to in 0u32..6,
        ) {
            let n = g.vcount();
            prop_assume!(from < n && to < n);
            let p = get_shortest_path_astar(&g, from, to, None, DijkstraMode::All, None)
                .expect("ok");
            if p.vertices.is_empty() {
                return Ok(());
            }
            prop_assert_eq!(p.vertices[0], from);
            prop_assert_eq!(*p.vertices.last().expect("non-empty"), to);
            prop_assert_eq!(p.edges.len() + 1, p.vertices.len());

            let mut seen = vec![false; n as usize];
            for &v in &p.vertices {
                prop_assert!(!seen[v as usize], "vertex {} repeats", v);
                seen[v as usize] = true;
            }
            for (i, &e) in p.edges.iter().enumerate() {
                let (a, b) = g.edge(e).expect("edge id valid");
                let (u, v) = (p.vertices[i], p.vertices[i + 1]);
                prop_assert!(
                    (a == u && b == v) || (a == v && b == u),
                    "edge {} = ({},{}) does not join {} and {}",
                    e, a, b, u, v
                );
            }
        }

        /// A* with the null heuristic must agree with Dijkstra
        /// (`get_shortest_path`) on path length, and on reachability.
        #[test]
        fn astar_null_matches_dijkstra_length(
            g in arb_graph(6),
            from in 0u32..6,
            to in 0u32..6,
        ) {
            let n = g.vcount();
            prop_assume!(from < n && to < n);
            let ones = vec![1.0_f64; g.ecount()];
            let astar = get_shortest_path_astar(&g, from, to, Some(&ones), DijkstraMode::All, None)
                .expect("astar");
            let dij = get_shortest_path(&g, from, to, Some(&ones), DijkstraMode::All)
                .expect("dijkstra");
            prop_assert_eq!(astar.vertices.is_empty(), dij.vertices.is_empty());
            prop_assert_eq!(astar.edges.len(), dij.edges.len());
        }

        /// An admissible heuristic must not change the shortest-path
        /// length found with the null heuristic.
        #[test]
        fn admissible_heuristic_preserves_length(
            g in arb_graph(6),
            from in 0u32..6,
            to in 0u32..6,
        ) {
            let n = g.vcount();
            prop_assume!(from < n && to < n);
            let null = get_shortest_path_astar(&g, from, to, None, DijkstraMode::All, None)
                .expect("null");
            // h(v,to) = 1 for v!=to, 0 for v==to: always admissible on
            // unweighted graphs since any path uses at least one edge.
            let h = |v: VertexId, t: VertexId| if v == t { 0.0 } else { 1.0 };
            let heur = get_shortest_path_astar(&g, from, to, None, DijkstraMode::All, Some(&h))
                .expect("heur");
            prop_assert_eq!(null.vertices.is_empty(), heur.vertices.is_empty());
            prop_assert_eq!(null.edges.len(), heur.edges.len());
        }
    }
}
