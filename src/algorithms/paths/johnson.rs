//! Johnson's all-pairs shortest paths (ALGO-SP-003).
//!
//! Counterpart of `igraph_distances_johnson()` from
//! `references/igraph/src/paths/johnson.c:83`. Computes the
//! shortest-path distance matrix between every pair of vertices,
//! supporting **negative edge weights** in directed graphs.
//!
//! Algorithm:
//!
//! Fast path: if no edge weight is negative, fall through to
//! [`dijkstra_distances`] from every source — that is asymptotically
//! the same (`O(V (V+E) log V)`) and avoids the reweighting overhead.
//!
//! Slow path (negative weights present on a *directed* graph): first
//! compute potentials `h[v]` via a "virtual source" Bellman-Ford.
//! The standard trick — initialising every vertex's distance to 0
//! in SPFA is equivalent to attaching a virtual vertex with
//! zero-weight outgoing edges to every real vertex and running BF
//! from it (matches upstream's `bfres` row computation). Reweight
//! each edge `(u, v, w)` to `w' = w + h[u] - h[v]`. By the triangle
//! inequality on `h`, every `w'` is ≥ 0 (modulo float roundoff,
//! which we snap to 0). Run [`dijkstra_distances`] from every source
//! on the reweighted graph to get `d'`. Recover the true distance:
//! `d[u][v] = d'[u][v] - h[u] + h[v]`.
//!
//! Time complexity: `O(V·E + V·(V+E)·log V)` — V Dijkstras plus one
//! BF.
//!
//! Constraints (matching upstream):
//! - Undirected graphs with negative weights are rejected
//!   ([`IgraphError::InvalidArgument`]). This isn't a Johnson
//!   limitation per se — an undirected negative edge is itself a
//!   2-cycle of negative weight.
//! - Negative cycles reachable from anywhere are detected by the
//!   inner BF and surfaced as [`IgraphError::InvalidArgument`].

use std::collections::VecDeque;

use crate::algorithms::paths::dijkstra::dijkstra_distances;
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

/// SPFA with every vertex initialised to distance 0. Returns the
/// Johnson potential vector `h[v]`. Equivalent to running
/// Bellman-Ford from a virtual source with zero-weight outgoing
/// edges to every real vertex.
///
/// Surfaces a negative cycle (any vertex popped more than `vcount`
/// times) as [`IgraphError::InvalidArgument`].
fn compute_potentials(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_usize = n as usize;
    let mut dist: Vec<f64> = vec![0.0; n_usize];
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
                "negative cycle detected while computing Johnson potentials".to_string(),
            ));
        }

        // OUT-mode SPFA: traverse outgoing incident edges.
        let incidents = graph.incident(j)?;
        for eid in incidents {
            let w = weights[eid as usize];
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

    Ok(dist)
}

/// All-pairs shortest-path distances via Johnson's algorithm.
///
/// Returns a `vcount × vcount` matrix where `result[u][v]` is the
/// distance from `u` to `v`: `Some(d)` if reachable, `None`
/// otherwise. `result[u][u]` is `Some(0.0)` for every valid `u`.
///
/// Use this when (a) you need many shortest paths and (b) at least
/// one edge weight is negative. For non-negative weights this
/// function transparently falls through to V independent Dijkstra
/// runs — slightly wasteful but always correct.
///
/// `weights[e]` is the weight of edge `e`; length must equal
/// `graph.ecount()`. NaN weights are rejected. Positive-infinite
/// weights are treated as "edge ignored" (matches upstream).
///
/// **Constraint**: undirected graphs with any negative weight are
/// rejected ([`IgraphError::InvalidArgument`]) — an undirected
/// negative edge is itself a length-2 negative cycle.
///
/// Counterpart of `igraph_distances_johnson(_, _, vss_all(),
/// vss_all(), weights, IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, johnson_distances};
///
/// // Directed diamond 0→1 (3), 0→2 (1), 1→3 (-2), 2→3 (4).
/// // The negative edge 1→3 makes 0→1→3 cheaper than 0→2→3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let d = johnson_distances(&g, &[3.0, 1.0, -2.0, 4.0]).unwrap();
/// // Row 0 (distances from 0): [0, 3, 1, 1]
/// assert_eq!(d[0], vec![Some(0.0), Some(3.0), Some(1.0), Some(1.0)]);
/// ```
pub fn johnson_distances(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<Vec<Option<f64>>>> {
    validate_weights(graph, weights)?;
    let vcount = graph.vcount();

    let has_negative = weights.iter().any(|&w| w < 0.0);

    // Fast path: no negative weights ⇒ Dijkstra from every source.
    if !has_negative {
        let mut out = Vec::with_capacity(vcount as usize);
        for v in 0..vcount {
            out.push(dijkstra_distances(graph, v, weights)?);
        }
        return Ok(out);
    }

    // Negative weights but undirected: an undirected negative edge
    // is a length-2 cycle of negative weight.
    if !graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "Johnson's algorithm cannot be applied to undirected graphs with negative weights (would form a negative cycle)".to_string(),
        ));
    }

    // 1. Compute potentials h[v] via virtual-source SPFA.
    let potentials = compute_potentials(graph, weights)?;

    // 2. Reweight: w'(u, v) = w(u, v) + h[u] - h[v]. Snap roundoff
    //    negatives to 0 (matches upstream johnson.c:194).
    let ecount = graph.ecount();
    let mut reweighted: Vec<f64> = Vec::with_capacity(ecount);
    for (e, &orig_w) in weights.iter().enumerate() {
        let eid = u32::try_from(e)
            .map_err(|_| IgraphError::Internal("edge id exceeds u32::MAX in johnson"))?;
        let (from, to) = graph.edge(eid)?;
        let mut new_w = orig_w + potentials[from as usize] - potentials[to as usize];
        if new_w < 0.0 {
            new_w = 0.0;
        }
        reweighted.push(new_w);
    }

    // 3. Run Dijkstra from every source on the reweighted graph.
    // 4. Recover: d[u][v] = d'[u][v] - h[u] + h[v].
    let mut out = Vec::with_capacity(vcount as usize);
    for src in 0..vcount {
        let dprime = dijkstra_distances(graph, src, &reweighted)?;
        let mut row: Vec<Option<f64>> = Vec::with_capacity(vcount as usize);
        for (target, dp) in dprime.into_iter().enumerate() {
            let recovered = dp.map(|d| d - potentials[src as usize] + potentials[target]);
            row.push(recovered);
        }
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_returns_empty_matrix() {
        let g = Graph::with_vertices(0);
        let d = johnson_distances(&g, &[]).unwrap();
        assert!(d.is_empty());
    }

    #[test]
    fn no_edges_diagonal_zero_off_diagonal_none() {
        let g = Graph::with_vertices(3);
        let d = johnson_distances(&g, &[]).unwrap();
        assert_eq!(d.len(), 3);
        for (u, row) in d.iter().enumerate() {
            for (v, entry) in row.iter().enumerate() {
                if u == v {
                    assert_eq!(entry, &Some(0.0));
                } else {
                    assert_eq!(entry, &None);
                }
            }
        }
    }

    #[test]
    fn positive_weights_match_pairwise_dijkstra() {
        // Triangle 0-1-2 with positive weights — both fast paths.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = [1.0, 4.0, 2.0];
        let d = johnson_distances(&g, &weights).unwrap();
        // Row 0: 0, 1, 3 (via shortcut)
        assert_eq!(d[0], vec![Some(0.0), Some(1.0), Some(3.0)]);
        assert_eq!(d[1], vec![Some(1.0), Some(0.0), Some(2.0)]);
        assert_eq!(d[2], vec![Some(3.0), Some(2.0), Some(0.0)]);
    }

    #[test]
    fn directed_with_negative_edge_diamond() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = [3.0, 1.0, -2.0, 4.0];
        let d = johnson_distances(&g, &weights).unwrap();
        // From 0: 0→1=3, 0→2=1, 0→3=min(3-2, 1+4)=1
        assert_eq!(d[0], vec![Some(0.0), Some(3.0), Some(1.0), Some(1.0)]);
        // From 1: only out-edge is 1→3=-2; 0,2 unreachable
        assert_eq!(d[1], vec![None, Some(0.0), None, Some(-2.0)]);
        // From 2: only out-edge is 2→3=4
        assert_eq!(d[2], vec![None, None, Some(0.0), Some(4.0)]);
        // From 3: sink, only itself
        assert_eq!(d[3], vec![None, None, None, Some(0.0)]);
    }

    #[test]
    fn unreachable_components_yield_none() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = johnson_distances(&g, &[1.0, 2.0]).unwrap();
        // Two components: 0,1 reach each other; 2,3 reach each other; cross None.
        assert_eq!(d[0][0], Some(0.0));
        assert_eq!(d[0][1], Some(1.0));
        assert_eq!(d[0][2], None);
        assert_eq!(d[0][3], None);
        assert_eq!(d[2][3], Some(2.0));
    }

    #[test]
    fn undirected_with_negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = johnson_distances(&g, &[-1.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn negative_cycle_in_directed_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // 1 + 1 - 3 = -1 < 0 ⇒ negative cycle
        let err = johnson_distances(&g, &[1.0, 1.0, -3.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = johnson_distances(&g, &[1.0, 2.0]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let err = johnson_distances(&g, &[f64::NAN]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn zero_weights_ok() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = johnson_distances(&g, &[0.0, 0.0]).unwrap();
        assert_eq!(d[0], vec![Some(0.0), Some(0.0), Some(0.0)]);
    }

    #[test]
    fn directed_negative_edges_no_cycle_complex() {
        // 4 vertices: 0→1 (5), 0→2 (-3), 1→3 (1), 2→1 (4), 2→3 (2).
        // From 0: 0→2→3 = -3 + 2 = -1; 0→2→1→3 = -3+4+1 = 2; so 0→3 = -1.
        // From 0→1: direct 5, or 0→2→1 = -3+4 = 1, so 0→1 = 1.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let weights = [5.0, -3.0, 1.0, 4.0, 2.0];
        let d = johnson_distances(&g, &weights).unwrap();
        assert_eq!(d[0][0], Some(0.0));
        assert_eq!(d[0][1], Some(1.0));
        assert_eq!(d[0][2], Some(-3.0));
        assert_eq!(d[0][3], Some(-1.0));
    }
}
