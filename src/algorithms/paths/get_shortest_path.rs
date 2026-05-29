//! Single-pair shortest path (`ALGO-SP-035`).
//!
//! Counterpart of `igraph_get_shortest_path()` from
//! `references/igraph/src/paths/unweighted.c:662-703`, the convenience
//! wrapper over `igraph_get_shortest_paths()` for the special case of a
//! single target. It returns one shortest path from `from` to `to` as
//! both the vertex sequence (including both endpoints) and the edge
//! sequence along it.
//!
//! Like upstream, the backend is selected by the `weights` argument
//! (`igraph_get_shortest_paths`, `unweighted.c:404-426`):
//! - `None` → unweighted BFS ([`igraph_i_get_shortest_paths_unweighted`]),
//! - `Some` with all-non-negative weights → Dijkstra,
//! - `Some` with at least one negative weight → Bellman-Ford.
//!
//! When several shortest paths exist an arbitrary one is returned; for
//! the unweighted case the choice is determined by incident-edge order,
//! matching upstream's BFS exactly (it records the edge each vertex was
//! first reached through, scanning `igraph_incident` in edge-id order).

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::bellman_ford::bellman_ford_path_to_with_mode;
use super::dijkstra::{DijkstraMode, dijkstra_path_to_with_mode};

/// Per-vertex mode-aware incident edges, mirroring upstream's
/// `igraph_incident(_, _, v, mode, IGRAPH_LOOPS)`. For undirected graphs
/// every mode collapses to [`Graph::incident`]'s merged adjacency.
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

/// A single shortest path: the vertex sequence (including `from` and
/// `to`) and the edge sequence along it. For a path of `k` vertices the
/// edge vector has `k - 1` entries.
///
/// Both vectors are empty when `to` is unreachable from `from`,
/// matching upstream (which leaves the output vectors cleared). When
/// `from == to` the vertex vector is `[from]` and the edge vector is
/// empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortestPath {
    /// Vertex IDs along the path, source first and target last.
    pub vertices: Vec<VertexId>,
    /// Edge IDs along the path; `vertices.len().saturating_sub(1)` long.
    pub edges: Vec<EdgeId>,
}

/// Faithful single-target port of
/// `igraph_i_get_shortest_paths_unweighted` (`unweighted.c:428-619`).
///
/// `parent_eids` follows the upstream encoding:
/// - `0` — vertex is not the target and not yet reached,
/// - `-1` — vertex is the target and not yet reached,
/// - `1` — the start vertex,
/// - `e + 2` — reached via edge `e`.
fn bfs_single_path(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    mode: DijkstraMode,
) -> IgraphResult<ShortestPath> {
    let n = graph.vcount() as usize;
    let mut parent_eids: Vec<i64> = vec![0; n];
    parent_eids[to as usize] = -1;

    let to_reach = 1_i64;
    let mut reached = 0_i64;

    let mut q: VecDeque<VertexId> = VecDeque::new();
    q.push_back(from);
    if parent_eids[from as usize] < 0 {
        reached += 1; // from == to
    }
    parent_eids[from as usize] = 1;

    while reached < to_reach {
        let Some(act) = q.pop_front() else { break };
        for edge in incident_for_mode(graph, act, mode)? {
            let neighbor = graph.edge_other(edge, act)?;
            let pe = parent_eids[neighbor as usize];
            // pe > 0 means already reached (or the start) — skip it.
            if pe <= 0 {
                if pe < 0 {
                    reached += 1; // a wanted target reached for the first time
                }
                // edge + 2 cannot overflow i64: edge fits in u32.
                parent_eids[neighbor as usize] = i64::from(edge) + 2;
                q.push_back(neighbor);
            }
        }
    }

    // Reconstruct the single target path, walking parent edges back to
    // the start, then filling the output vectors front-to-back.
    if parent_eids[to as usize] <= 0 {
        // `to` was never reached.
        return Ok(ShortestPath {
            vertices: Vec::new(),
            edges: Vec::new(),
        });
    }

    let mut size = 0_usize;
    let mut act = to;
    while parent_eids[act as usize] > 1 {
        size += 1;
        let edge = decode_edge(parent_eids[act as usize])?;
        act = graph.edge_other(edge, act)?;
    }

    let mut vertices = vec![0 as VertexId; size + 1];
    let mut edges = vec![0 as EdgeId; size];
    vertices[size] = to;

    let mut idx = size;
    let mut act = to;
    while parent_eids[act as usize] > 1 {
        idx -= 1;
        let edge = decode_edge(parent_eids[act as usize])?;
        act = graph.edge_other(edge, act)?;
        vertices[idx] = act;
        edges[idx] = edge;
    }

    Ok(ShortestPath { vertices, edges })
}

/// Recover the edge id from a `parent_eids` entry (`e + 2`).
fn decode_edge(parent_eid: i64) -> IgraphResult<EdgeId> {
    EdgeId::try_from(parent_eid - 2)
        .map_err(|_| IgraphError::Internal("get_shortest_path: edge id overflow"))
}

/// Calculate a single shortest path from `from` to `to`.
///
/// Returns the path as both its vertex sequence (source first, target
/// last) and the edge sequence along it. If several shortest paths
/// exist, an arbitrary one is returned.
///
/// The backend mirrors `igraph_get_shortest_paths`:
/// - `weights == None`: unweighted breadth-first search,
/// - `weights == Some(w)` with all `w[e] >= 0`: Dijkstra,
/// - `weights == Some(w)` with any `w[e] < 0`: Bellman-Ford.
///
/// On directed graphs `mode` selects which adjacency is followed
/// ([`DijkstraMode::Out`] / [`DijkstraMode::In`] / [`DijkstraMode::All`]);
/// it is ignored on undirected graphs.
///
/// # Errors
///
/// Returns [`IgraphError::VertexOutOfRange`] if `from` or `to` is not a
/// valid vertex, or [`IgraphError::InvalidArgument`] if `weights` is
/// provided and its length differs from the edge count or contains NaN.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_path, DijkstraMode};
///
/// let mut g = Graph::new(4, false).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 2).unwrap(); // edge 1
/// g.add_edge(2, 3).unwrap(); // edge 2
/// let p = get_shortest_path(&g, 0, 3, None, DijkstraMode::Out).unwrap();
/// assert_eq!(p.vertices, vec![0, 1, 2, 3]);
/// assert_eq!(p.edges, vec![0, 1, 2]);
///
/// // Unreachable target → empty path.
/// let mut h = Graph::new(2, true).unwrap();
/// h.add_edge(0, 1).unwrap();
/// let q = get_shortest_path(&h, 1, 0, None, DijkstraMode::Out).unwrap();
/// assert!(q.vertices.is_empty());
/// assert!(q.edges.is_empty());
/// ```
pub fn get_shortest_path(
    graph: &Graph,
    from: VertexId,
    to: VertexId,
    weights: Option<&[f64]>,
    mode: DijkstraMode,
) -> IgraphResult<ShortestPath> {
    let n = graph.vcount();
    if from >= n {
        return Err(IgraphError::VertexOutOfRange { id: from, n });
    }
    if to >= n {
        return Err(IgraphError::VertexOutOfRange { id: to, n });
    }

    let Some(w) = weights else {
        return bfs_single_path(graph, from, to, mode);
    };

    let m = graph.ecount();
    if w.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "get_shortest_path: weights length {} != edge count {m}",
            w.len()
        )));
    }
    let mut has_negative = false;
    for (e, &x) in w.iter().enumerate() {
        if x.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "get_shortest_path: weight at edge {e} is NaN"
            )));
        }
        if x < 0.0 {
            has_negative = true;
        }
    }

    let result = if has_negative {
        bellman_ford_path_to_with_mode(graph, from, to, w, mode)?
    } else {
        dijkstra_path_to_with_mode(graph, from, to, w, mode)?
    };

    Ok(match result {
        Some((vertices, edges)) => ShortestPath { vertices, edges },
        None => ShortestPath {
            vertices: Vec::new(),
            edges: Vec::new(),
        },
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn from_equals_to_unweighted() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let p = get_shortest_path(&g, 1, 1, None, DijkstraMode::Out).unwrap();
        assert_eq!(p.vertices, vec![1]);
        assert!(p.edges.is_empty());
    }

    #[test]
    fn simple_path_unweighted_undirected() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        let p = get_shortest_path(&g, 0, 3, None, DijkstraMode::Out).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 2, 3]);
        assert_eq!(p.edges, vec![0, 1, 2]);
    }

    #[test]
    fn picks_shorter_of_two_routes() {
        // 0-1-3 (len 2) vs 0-2-... ; direct shortcut 0-3.
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 3).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2  (shortcut)
        let p = get_shortest_path(&g, 0, 3, None, DijkstraMode::Out).unwrap();
        assert_eq!(p.vertices, vec![0, 3]);
        assert_eq!(p.edges, vec![2]);
    }

    #[test]
    fn unreachable_returns_empty() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let p = get_shortest_path(&g, 0, 2, None, DijkstraMode::Out).unwrap();
        assert!(p.vertices.is_empty());
        assert!(p.edges.is_empty());
    }

    #[test]
    fn directed_mode_in_follows_reverse() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        // OUT: 0 -> 2 works.
        let out = get_shortest_path(&g, 0, 2, None, DijkstraMode::Out).unwrap();
        assert_eq!(out.vertices, vec![0, 1, 2]);
        // OUT: 2 -> 0 impossible.
        let none = get_shortest_path(&g, 2, 0, None, DijkstraMode::Out).unwrap();
        assert!(none.vertices.is_empty());
        // IN: 2 -> 0 follows reversed edges.
        let inp = get_shortest_path(&g, 2, 0, None, DijkstraMode::In).unwrap();
        assert_eq!(inp.vertices, vec![2, 1, 0]);
        assert_eq!(inp.edges, vec![1, 0]);
    }

    #[test]
    fn directed_mode_all_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let p = get_shortest_path(&g, 2, 0, None, DijkstraMode::All).unwrap();
        assert_eq!(p.vertices, vec![2, 1, 0]);
    }

    #[test]
    fn weighted_dijkstra_prefers_cheaper_route() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 3).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2 direct but expensive
        let w = vec![1.0, 1.0, 10.0];
        let p = get_shortest_path(&g, 0, 3, Some(&w), DijkstraMode::Out).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 3]);
        assert_eq!(p.edges, vec![0, 1]);
    }

    #[test]
    fn weighted_negative_uses_bellman_ford() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(0, 2).unwrap(); // 2 direct
        // Direct edge cheaper only thanks to a negative leg on 0->1->2.
        let w = vec![1.0, -5.0, 1.0];
        let p = get_shortest_path(&g, 0, 2, Some(&w), DijkstraMode::Out).unwrap();
        assert_eq!(p.vertices, vec![0, 1, 2]);
        assert_eq!(p.edges, vec![0, 1]);
    }

    #[test]
    fn weighted_unreachable_returns_empty() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![1.0];
        let p = get_shortest_path(&g, 0, 2, Some(&w), DijkstraMode::Out).unwrap();
        assert!(p.vertices.is_empty());
        assert!(p.edges.is_empty());
    }

    #[test]
    fn invalid_source_errors() {
        let g = Graph::new(2, false).unwrap();
        assert!(get_shortest_path(&g, 5, 0, None, DijkstraMode::Out).is_err());
        assert!(get_shortest_path(&g, 0, 5, None, DijkstraMode::Out).is_err());
    }

    #[test]
    fn weights_length_mismatch_errors() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![1.0, 2.0];
        assert!(get_shortest_path(&g, 0, 1, Some(&w), DijkstraMode::Out).is_err());
    }

    #[test]
    fn weights_nan_errors() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        let w = vec![f64::NAN];
        assert!(get_shortest_path(&g, 0, 1, Some(&w), DijkstraMode::Out).is_err());
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
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
        /// A non-empty result is a genuine simple walk from `from` to
        /// `to`: endpoints match, each edge joins consecutive vertices,
        /// and no vertex repeats (shortest paths are loopless).
        #[test]
        fn path_is_a_valid_simple_walk(
            g in arb_graph(6),
            from in 0u32..6,
            to in 0u32..6,
        ) {
            let n = g.vcount();
            prop_assume!(from < n && to < n);
            let p = get_shortest_path(&g, from, to, None, DijkstraMode::All).expect("ok");
            if p.vertices.is_empty() {
                return Ok(());
            }
            prop_assert_eq!(p.vertices[0], from);
            prop_assert_eq!(*p.vertices.last().expect("non-empty"), to);
            prop_assert_eq!(p.edges.len() + 1, p.vertices.len());

            // No repeated vertices.
            let mut seen = vec![false; n as usize];
            for &v in &p.vertices {
                prop_assert!(!seen[v as usize], "vertex {} repeats", v);
                seen[v as usize] = true;
            }

            // Each edge joins the two consecutive vertices (undirected).
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

        /// On an unweighted graph the BFS backend and the Dijkstra
        /// backend (all weights 1.0) must agree on path length.
        #[test]
        fn bfs_and_unit_dijkstra_agree_on_length(
            g in arb_graph(6),
            from in 0u32..6,
            to in 0u32..6,
        ) {
            let n = g.vcount();
            prop_assume!(from < n && to < n);
            let unweighted = get_shortest_path(&g, from, to, None, DijkstraMode::All)
                .expect("ok");
            let ones = vec![1.0_f64; g.ecount()];
            let weighted = get_shortest_path(&g, from, to, Some(&ones), DijkstraMode::All)
                .expect("ok");
            prop_assert_eq!(unweighted.vertices.is_empty(), weighted.vertices.is_empty());
            prop_assert_eq!(unweighted.edges.len(), weighted.edges.len());
        }
    }
}
