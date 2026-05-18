//! Weighted betweenness centrality (ALGO-PR-008b).
//!
//! Counterpart of `igraph_betweenness(_, _, vss_all(), directed,
//! &weights)` from
//! `references/igraph/src/centrality/betweenness.c`. Brandes' (2001)
//! algorithm with Dijkstra in place of BFS:
//!
//! 1. For each source `s`, run Dijkstra. While relaxing edge
//!    `(v, w, weight)`:
//!    - if `d[v] + weight < d[w]`: `pred[w] = [v]`, `sigma[w] = sigma[v]`
//!    - if `d[v] + weight == d[w]`: `pred[w].push(v)`, `sigma[w] += sigma[v]`
//! 2. Record vertices in the order they're popped (final-distance order)
//!    in a stack `S`.
//! 3. Process `S` in reverse to accumulate dependencies the same way as
//!    the unweighted variant.
//!
//! Phase-1 minimal slice: directed/OUT or undirected (divided by 2),
//! raw counts. Weights must be non-negative + finite (forwarded from
//! the Dijkstra building blocks).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Min-heap entry. Reversed so `BinaryHeap` (max-heap) pops the
/// smallest distance first. `total_cmp` is fine because `dijkstra`-
/// style entry points reject NaN / negative / non-finite weights.
#[derive(Copy, Clone)]
struct Frontier(f64, VertexId);

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Per-vertex weighted betweenness centrality.
///
/// Returns `Vec<f64>` of length `vcount`. Raw (unnormalised) counts:
/// for undirected graphs the result is divided by 2 to match the
/// unweighted variant (each unordered pair is counted once).
///
/// `weights[e]` is the weight of edge `e`; `weights.len()` must equal
/// `graph.ecount()`. All weights must be `>= 0` and finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, betweenness_weighted};
///
/// // Path 0-1-2-3-4, all weights 1.0 → matches unweighted result.
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i + 1).unwrap(); }
/// let b = betweenness_weighted(&g, &[1.0; 4]).unwrap();
/// // Same as unweighted PR-008 path test.
/// assert_eq!(b, vec![0.0, 3.0, 4.0, 3.0, 0.0]);
/// ```
pub fn betweenness_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut bc = vec![0.0_f64; n_us];

    if n == 0 {
        return Ok(bc);
    }

    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() || w < 0.0 || !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} must be non-negative and finite (got {w})"
            )));
        }
    }

    // Per-source state, allocated once and reused.
    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![f64::INFINITY; n_us];
    let mut visited = vec![false; n_us];
    let mut pred: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta = vec![0.0_f64; n_us];

    // Tolerance for "equal distance" comparison: zero-weight edges can
    // produce numerically-identical alternate paths. Brandes' algorithm
    // is robust under exact equality; we don't add a fudge factor here.

    for s in 0..n {
        // Reset per-source state.
        sigma.fill(0.0);
        dist.fill(f64::INFINITY);
        visited.fill(false);
        for slot in &mut pred {
            slot.clear();
        }
        stack.clear();
        delta.fill(0.0);

        sigma[s as usize] = 1.0;
        dist[s as usize] = 0.0;
        let mut heap: BinaryHeap<Frontier> = BinaryHeap::new();
        heap.push(Frontier(0.0, s));

        while let Some(Frontier(d, v)) = heap.pop() {
            let v_us = v as usize;
            if visited[v_us] {
                continue;
            }
            visited[v_us] = true;
            stack.push(v);

            for eid in graph.incident(v)? {
                let w_edge = weights[eid as usize];
                let other = graph.edge_other(eid as EdgeId, v)?;
                let other_us = other as usize;
                let nd = d + w_edge;
                match nd.partial_cmp(&dist[other_us]) {
                    Some(Ordering::Less) => {
                        dist[other_us] = nd;
                        sigma[other_us] = sigma[v_us];
                        pred[other_us].clear();
                        pred[other_us].push(v);
                        heap.push(Frontier(nd, other));
                    }
                    Some(Ordering::Equal) => {
                        sigma[other_us] += sigma[v_us];
                        pred[other_us].push(v);
                    }
                    _ => {} // strictly greater, or NaN (already rejected)
                }
            }
        }

        // Accumulate dependencies in reverse final-distance order.
        while let Some(w) = stack.pop() {
            let w_us = w as usize;
            for &v in &pred[w_us] {
                delta[v as usize] += (sigma[v as usize] / sigma[w_us]) * (1.0 + delta[w_us]);
            }
            if w != s {
                bc[w_us] += delta[w_us];
            }
        }
    }

    if !graph.is_directed() {
        for slot in &mut bc {
            *slot /= 2.0;
        }
    }

    Ok(bc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "vertex {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(betweenness_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_all_zero() {
        let g = Graph::with_vertices(5);
        let b = betweenness_weighted(&g, &[]).unwrap();
        assert_eq!(b, vec![0.0; 5]);
    }

    #[test]
    fn unit_weights_match_unweighted_path_5() {
        // Path 0-1-2-3-4 with unit weights → same as PR-008 path test.
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let b = betweenness_weighted(&g, &[1.0; 4]).unwrap();
        close(&b, &[0.0, 3.0, 4.0, 3.0, 0.0], 1e-12);
    }

    #[test]
    fn weighted_path_swaps_route_via_higher_weight() {
        // 3-vertex graph (0,1), (1,2), (0,2); weights make the direct
        // (0,2) edge longer than 0->1->2: 0-1 w=1, 1-2 w=1, 0-2 w=5.
        // Shortest path 0-2 is via 1, so vertex 1 is a betweenness
        // intermediary. Brandes raw count: 2 (one ordered pair from
        // each direction); undirected halve → 1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let b = betweenness_weighted(&g, &[1.0, 1.0, 5.0]).unwrap();
        close(&b, &[0.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn weighted_path_keeps_direct_when_cheaper() {
        // Same 3-vertex graph but direct (0,2) cheaper. Vertex 1
        // betweenness drops to 0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let b = betweenness_weighted(&g, &[5.0, 5.0, 1.0]).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn directed_unit_weights_match_unweighted() {
        // 0->1->2: betweenness[1] = 1 (one ordered pair (0,2)).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let b = betweenness_weighted(&g, &[1.0, 1.0]).unwrap();
        close(&b, &[0.0, 1.0, 0.0], 1e-12);
    }

    #[test]
    fn k4_complete_unit_weights_all_zero() {
        // K4 + unit weights → no vertex on a 2-hop shortest path.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let b = betweenness_weighted(&g, &[1.0; 6]).unwrap();
        close(&b, &[0.0; 4], 1e-12);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(betweenness_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(betweenness_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(betweenness_weighted(&g, &[f64::NAN]).is_err());
    }
}
