//! Bellman-Ford single-source shortest paths (ALGO-SP-002).
//!
//! Counterpart of `igraph_distances_bellman_ford()` from
//! `references/igraph/src/paths/bellman_ford.c:69`. Returns the
//! shortest-path distance from a single source to every vertex,
//! allowing **negative edge weights** (which Dijkstra forbids).
//! Detects negative cycles reachable from the source.
//!
//! Algorithm: SPFA (Shortest Path Faster Algorithm) — the
//! queue-based Bellman-Ford variant the upstream `bellman_ford.c`
//! uses. Each vertex starts in the queue; popping a vertex marks it
//! "clean" and relaxes its outgoing edges; relaxing an edge marks
//! the target "dirty" and re-queues it if it was clean. A vertex
//! popped more than `vcount` times signals a negative cycle.
//!
//! Time complexity: `O(V · E)` worst case (the SPFA optimisation
//! often beats this in practice).

use std::collections::VecDeque;

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate_weights(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} is NaN"
            )));
        }
    }
    Ok(())
}

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

/// Single-source Bellman-Ford shortest distances on `graph` from
/// `source`, following out-edges on directed graphs.
///
/// Returns `distances[v]`: `Some(d)` if `v` is reachable from
/// `source`, `None` otherwise. `distances[source] == Some(0.0)`.
///
/// `weights[e]` is the weight of edge `e`; length must equal
/// `graph.ecount()`. Negative weights are allowed; NaN weights are
/// rejected ([`IgraphError::InvalidArgument`]). If a negative cycle
/// is reachable from the source, returns
/// [`IgraphError::InvalidArgument`] with a "negative cycle"
/// message (matches upstream's `IGRAPH_ENEGCYCLE` semantics).
///
/// Use this instead of [`crate::dijkstra_distances`] when edge
/// weights can be negative. For non-negative weights Dijkstra is
/// asymptotically faster (`O((V+E) log V)` vs Bellman-Ford's
/// `O(V·E)`).
///
/// Counterpart of `igraph_distances_bellman_ford(_, _, vss(source),
/// vss_all(), weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bellman_ford_distances};
///
/// // Directed graph 0 → 1 → 2 with weights 3, -1 — Bellman-Ford
/// // handles the negative edge that would break Dijkstra's
/// // monotonicity assumption.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();  // edge 0, weight 3
/// g.add_edge(1, 2).unwrap();  // edge 1, weight -1
/// let d = bellman_ford_distances(&g, 0, &[3.0, -1.0]).unwrap();
/// assert_eq!(d, vec![Some(0.0), Some(3.0), Some(2.0)]);
/// ```
pub fn bellman_ford_distances(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<Vec<Option<f64>>> {
    bellman_ford_distances_with_mode(graph, source, weights, DijkstraMode::Out)
}

/// Single-source Bellman-Ford with directed-mode selection.
///
/// `mode` selects how directed edges are followed:
/// - [`DijkstraMode::Out`] follows out-edges (default for
///   [`bellman_ford_distances`]),
/// - [`DijkstraMode::In`] follows in-edges (i.e. shortest paths
///   *into* `source`),
/// - [`DijkstraMode::All`] ignores edge direction.
///
/// On undirected graphs the mode is ignored.
///
/// Counterpart of `igraph_distances_bellman_ford(_, _, vss(source),
/// vss_all(), weights, mode)`.
pub fn bellman_ford_distances_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Vec<Option<f64>>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    validate_weights(graph, weights)?;

    let n_usize = n as usize;
    let mut dist: Vec<f64> = vec![f64::INFINITY; n_usize];
    dist[source as usize] = 0.0;

    // SPFA: queue all vertices initially; pop, relax edges of clean
    // vertices, re-queue targets that get relaxed.
    let mut queue: VecDeque<VertexId> = VecDeque::with_capacity(n_usize);
    let mut in_queue: Vec<bool> = vec![true; n_usize];
    let mut num_queued: Vec<u32> = vec![0; n_usize];
    for v in 0..n {
        queue.push_back(v);
    }

    let n_u32 = n;
    while let Some(j) = queue.pop_front() {
        let j_idx = j as usize;
        in_queue[j_idx] = false;
        num_queued[j_idx] = num_queued[j_idx]
            .checked_add(1)
            .ok_or(IgraphError::Internal("num_queued overflow"))?;
        // A vertex queued more than vcount times indicates a negative
        // cycle (upstream uses the same `> n` threshold).
        if num_queued[j_idx] > n_u32 {
            return Err(IgraphError::InvalidArgument(
                "negative cycle reachable from source while running Bellman-Ford".to_string(),
            ));
        }

        // No point relaxing edges from an unreachable vertex.
        if !dist[j_idx].is_finite() {
            continue;
        }

        let incidents = incident_for_mode(graph, j, mode)?;
        for eid in incidents {
            let w = weights[eid as usize];
            // Positive-infinite weights are ignored (matches upstream).
            if w == f64::INFINITY {
                continue;
            }
            let target = graph.edge_other(eid, j)?;
            let target_idx = target as usize;
            let altdist = dist[j_idx] + w;
            if altdist < dist[target_idx] {
                dist[target_idx] = altdist;
                if !in_queue[target_idx] {
                    in_queue[target_idx] = true;
                    queue.push_back(target);
                }
            }
        }
    }

    Ok(dist
        .into_iter()
        .map(|d| if d.is_finite() { Some(d) } else { None })
        .collect())
}

/// Returns the shortest path from `source` to `target` using
/// Bellman-Ford, following out-edges on directed graphs.
///
/// Returns `Some((vertices, edges))` if a finite-weight path exists,
/// `None` if `target` is unreachable. When `source == target`, returns
/// `Some((vec![source], vec![]))`.
///
/// Supports negative edge weights; detects negative cycles reachable
/// from the source.
///
/// Counterpart of `igraph_get_shortest_path_bellman_ford(_, vertices,
/// edges, from, to, weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bellman_ford_path_to};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0, weight 3
/// g.add_edge(1, 2).unwrap(); // edge 1, weight -1
/// g.add_edge(0, 2).unwrap(); // edge 2, weight 5
/// g.add_edge(2, 3).unwrap(); // edge 3, weight 1
/// let (vs, es) = bellman_ford_path_to(&g, 0, 3, &[3.0, -1.0, 5.0, 1.0])
///     .unwrap()
///     .unwrap();
/// assert_eq!(vs, vec![0, 1, 2, 3]);
/// assert_eq!(es, vec![0, 1, 3]);
/// ```
pub fn bellman_ford_path_to(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    weights: &[f64],
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    bellman_ford_path_to_with_mode(graph, source, target, weights, DijkstraMode::Out)
}

/// Returns the shortest path from `source` to `target` using
/// Bellman-Ford with directed-mode selection.
///
/// `mode` selects how directed edges are followed:
/// - [`DijkstraMode::Out`] follows out-edges,
/// - [`DijkstraMode::In`] follows in-edges,
/// - [`DijkstraMode::All`] ignores edge direction.
///
/// Returns `Some((vertices, edges))` if a finite-weight path exists,
/// `None` if `target` is unreachable.
pub fn bellman_ford_path_to_with_mode(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<Option<(Vec<VertexId>, Vec<EdgeId>)>> {
    let n = graph.vcount();
    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    validate_weights(graph, weights)?;

    if source == target {
        return Ok(Some((vec![source], vec![])));
    }

    let n_usize = n as usize;
    let mut dist: Vec<f64> = vec![f64::INFINITY; n_usize];
    dist[source as usize] = 0.0;

    // Parent edge for reconstructing the path.
    let mut parent_edge: Vec<Option<EdgeId>> = vec![None; n_usize];

    let mut queue: VecDeque<VertexId> = VecDeque::with_capacity(n_usize);
    let mut in_queue: Vec<bool> = vec![true; n_usize];
    let mut num_queued: Vec<u32> = vec![0; n_usize];
    for v in 0..n {
        queue.push_back(v);
    }

    while let Some(j) = queue.pop_front() {
        let j_idx = j as usize;
        in_queue[j_idx] = false;
        num_queued[j_idx] = num_queued[j_idx]
            .checked_add(1)
            .ok_or(IgraphError::Internal("num_queued overflow"))?;
        if num_queued[j_idx] > n {
            return Err(IgraphError::InvalidArgument(
                "negative cycle reachable from source while running Bellman-Ford".to_string(),
            ));
        }

        if !dist[j_idx].is_finite() {
            continue;
        }

        let incidents = incident_for_mode(graph, j, mode)?;
        for eid in incidents {
            let w = weights[eid as usize];
            if w == f64::INFINITY {
                continue;
            }
            let neighbor = graph.edge_other(eid, j)?;
            let neighbor_idx = neighbor as usize;
            let altdist = dist[j_idx] + w;
            if altdist < dist[neighbor_idx] {
                dist[neighbor_idx] = altdist;
                parent_edge[neighbor_idx] = Some(eid);
                if !in_queue[neighbor_idx] {
                    in_queue[neighbor_idx] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // If target is unreachable, return None.
    if !dist[target as usize].is_finite() {
        return Ok(None);
    }

    // Reconstruct path from target back to source.
    let mut edges_rev: Vec<EdgeId> = Vec::new();
    let mut vertices_rev: Vec<VertexId> = Vec::new();
    let mut cur = target;
    vertices_rev.push(cur);
    while cur != source {
        let eid = parent_edge[cur as usize].ok_or(IgraphError::Internal(
            "bellman_ford_path_to: missing parent edge in path reconstruction",
        ))?;
        edges_rev.push(eid);
        cur = graph.edge_other(eid, cur)?;
        vertices_rev.push(cur);
    }

    vertices_rev.reverse();
    edges_rev.reverse();
    Ok(Some((vertices_rev, edges_rev)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_weights_match_dijkstra_triangle() {
        // Triangle 0-1-2 with positive weights — Bellman-Ford and
        // Dijkstra must agree.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = [1.0, 4.0, 2.0];
        let bf = bellman_ford_distances(&g, 0, &weights).unwrap();
        assert_eq!(bf, vec![Some(0.0), Some(1.0), Some(3.0)]);
    }

    #[test]
    fn negative_edge_directed_no_cycle() {
        // Directed 0 → 1 → 2 with weights 3, -1.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = bellman_ford_distances(&g, 0, &[3.0, -1.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(3.0), Some(2.0)]);
    }

    #[test]
    fn unreachable_vertex_is_none() {
        // Two components, source in the first.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = bellman_ford_distances(&g, 0, &[1.0, 1.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None, None]);
    }

    #[test]
    fn negative_cycle_directed_errors() {
        // 0 → 1 → 2 → 0 with weights summing to -1 (negative cycle).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let err = bellman_ford_distances(&g, 0, &[1.0, 1.0, -3.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn negative_cycle_unreachable_does_not_error() {
        // Negative cycle 2 → 3 → 2 unreachable from source 0.
        // Although SPFA initially queues every vertex, vertex 2's
        // distance stays at infinity, so relaxation is skipped and
        // the cycle never gets explored. Distances for the
        // unreachable component come back as None.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 2).unwrap();
        let d = bellman_ford_distances(&g, 0, &[1.0, -1.0, -1.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None, None]);
    }

    #[test]
    fn zero_weights_ok() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = bellman_ford_distances(&g, 0, &[0.0, 0.0]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(0.0), Some(0.0)]);
    }

    #[test]
    fn source_out_of_range_errors() {
        let g = Graph::with_vertices(3);
        let err = bellman_ford_distances(&g, 99, &[]).unwrap_err();
        assert!(matches!(
            err,
            IgraphError::VertexOutOfRange { id: 99, n: 3 }
        ));
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = bellman_ford_distances(&g, 0, &[1.0, 2.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = bellman_ford_distances(&g, 0, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn in_mode_walks_reverse_edges() {
        // Directed 0 → 1 → 2: from 2 with IN mode, paths reach 1 then 0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = bellman_ford_distances_with_mode(&g, 2, &[3.0, -1.0], DijkstraMode::In).unwrap();
        // dist(2→2)=0, dist(2→1)=-1, dist(2→0)=-1+3=2
        assert_eq!(d, vec![Some(2.0), Some(-1.0), Some(0.0)]);
    }

    #[test]
    fn all_mode_ignores_direction() {
        // Directed 0 → 1, asking ALL from 1 reaches 0 via the reverse.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = bellman_ford_distances_with_mode(&g, 1, &[5.0], DijkstraMode::All).unwrap();
        assert_eq!(d, vec![Some(5.0), Some(0.0)]);
    }

    #[test]
    fn infinity_weight_ignored() {
        // Edge with positive-infinite weight should be skipped (upstream behaviour).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let d = bellman_ford_distances(&g, 0, &[1.0, f64::INFINITY]).unwrap();
        assert_eq!(d, vec![Some(0.0), Some(1.0), None]);
    }

    // --- bellman_ford_path_to tests ---

    #[test]
    fn path_to_simple_directed() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(0, 2).unwrap(); // 2, w=5
        g.add_edge(2, 3).unwrap(); // 3
        let w = [3.0, -1.0, 5.0, 1.0];
        let (vs, es) = bellman_ford_path_to(&g, 0, 3, &w).unwrap().unwrap();
        assert_eq!(vs, vec![0, 1, 2, 3]);
        assert_eq!(es, vec![0, 1, 3]);
    }

    #[test]
    fn path_to_source_equals_target() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let (vs, es) = bellman_ford_path_to(&g, 0, 0, &[1.0]).unwrap().unwrap();
        assert_eq!(vs, vec![0]);
        assert!(es.is_empty());
    }

    #[test]
    fn path_to_unreachable() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let result = bellman_ford_path_to(&g, 0, 2, &[1.0]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn path_to_negative_cycle_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let err = bellman_ford_path_to(&g, 0, 2, &[1.0, 1.0, -3.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn path_to_prefers_negative_shortcut() {
        // 0→1→2 (w=1,1) and 0→2 (w=5); via negative: 0→1→2 is 2 < 5
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0, w=1
        g.add_edge(1, 2).unwrap(); // 1, w=-1
        g.add_edge(0, 2).unwrap(); // 2, w=5
        let (vs, es) = bellman_ford_path_to(&g, 0, 2, &[1.0, -1.0, 5.0])
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![0, 1, 2]);
        assert_eq!(es, vec![0, 1]);
    }

    #[test]
    fn path_to_with_in_mode() {
        // Directed 0→1→2; from 2 in IN mode finds path 2←1←0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        let (vs, es) = bellman_ford_path_to_with_mode(&g, 2, 0, &[3.0, -1.0], DijkstraMode::In)
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![2, 1, 0]);
        assert_eq!(es, vec![1, 0]);
    }

    #[test]
    fn path_to_undirected_negative_cycle() {
        // Undirected graph with a negative edge creates a negative cycle
        // (traverse edge back-and-forth indefinitely), so BF reports it.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // 0, w=2
        g.add_edge(1, 2).unwrap(); // 1, w=-1
        g.add_edge(0, 2).unwrap(); // 2, w=5
        let err = bellman_ford_path_to(&g, 0, 2, &[2.0, -1.0, 5.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn path_to_multiple_hops() {
        // 0→1→2→3 all weight 1, direct 0→3 weight 10
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(0, 3).unwrap(); // 3, w=10
        let (vs, es) = bellman_ford_path_to(&g, 0, 3, &[1.0, 1.0, 1.0, 10.0])
            .unwrap()
            .unwrap();
        assert_eq!(vs, vec![0, 1, 2, 3]);
        assert_eq!(es, vec![0, 1, 2]);
    }
}
