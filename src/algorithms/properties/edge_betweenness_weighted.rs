//! Weighted edge betweenness centrality (ALGO-PR-010b).
//!
//! Counterpart of `igraph_edge_betweenness(_, _, all_eids, directed,
//! &weights)` from `references/igraph/src/centrality/betweenness.c`.
//! Same Brandes-Dijkstra framework as
//! [`crate::betweenness_weighted`] (PR-008b) but the dependency is
//! deposited on edges rather than vertices.
//!
//! Phase-1 minimal slice: undirected/directed-OUT, raw (unnormalised)
//! counts. Weights must be non-negative + finite.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Min-heap entry. Reversed ordering so `BinaryHeap` (max-heap) pops
/// the smallest distance first. NaN/negative weights are rejected by
/// the public-API entry point so `total_cmp` is safe.
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

/// Per-edge weighted betweenness centrality (`Vec<f64>`).
///
/// `result[e]` is the raw count of shortest paths between all `(s, t)`
/// pairs that use edge `e`. Undirected results are halved.
///
/// `weights[e]` must be non-negative and finite; `weights.len()` must
/// equal `graph.ecount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness_weighted};
///
/// // 4-path with unit weights → matches PR-010 unweighted result.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3u32 { g.add_edge(i, i + 1).unwrap(); }
/// let eb = edge_betweenness_weighted(&g, &[1.0, 1.0, 1.0]).unwrap();
/// assert_eq!(eb, vec![3.0, 4.0, 3.0]);
/// ```
pub fn edge_betweenness_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    let mut bc = vec![0.0_f64; m];

    if n == 0 || m == 0 {
        return Ok(bc);
    }

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

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![f64::INFINITY; n_us];
    let mut visited = vec![false; n_us];
    // Predecessors store (vertex, edge-id pairs along shortest paths into w).
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];

    for s in 0..n {
        sigma.fill(0.0);
        dist.fill(f64::INFINITY);
        visited.fill(false);
        for slot in &mut pred {
            slot.clear();
        }
        stack.clear();
        delta_v.fill(0.0);

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
                        pred[other_us].push((v, eid as EdgeId));
                        heap.push(Frontier(nd, other));
                    }
                    Some(Ordering::Equal) => {
                        sigma[other_us] += sigma[v_us];
                        pred[other_us].push((v, eid as EdgeId));
                    }
                    _ => {}
                }
            }
        }

        // Reverse final-distance order; deposit dependency on edges.
        while let Some(w) = stack.pop() {
            let w_us = w as usize;
            for &(v, e) in &pred[w_us] {
                let c = (sigma[v as usize] / sigma[w_us]) * (1.0 + delta_v[w_us]);
                delta_v[v as usize] += c;
                bc[e as usize] += c;
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
            assert!((a - e).abs() < tol, "edge {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_betweenness_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn no_edges_yields_empty() {
        let g = Graph::with_vertices(5);
        assert!(edge_betweenness_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn unit_weights_match_unweighted_path_4() {
        // 0-1-2-3 → expected [3, 4, 3] (matches PR-010 fixture).
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let eb = edge_betweenness_weighted(&g, &[1.0, 1.0, 1.0]).unwrap();
        close(&eb, &[3.0, 4.0, 3.0], 1e-12);
    }

    #[test]
    fn weighted_triangle_swaps_route() {
        // Triangle 0-1-2 + chord 0-2 with weights (1, 1, 5). Path 0-1-2
        // (cost 2) is shorter than chord 0-2 (cost 5), so chord stays
        // unused. Brandes raw count: edges (0,1) and (1,2) each pick up
        // pair (0,2) plus their own direct pair = 2 ordered each direction,
        // halved → 2.0. Chord = 0.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let eb = edge_betweenness_weighted(&g, &[1.0, 1.0, 5.0]).unwrap();
        close(&eb, &[2.0, 2.0, 0.0], 1e-12);
    }

    #[test]
    fn weighted_triangle_keeps_direct() {
        // Same triangle but chord wins (1.0 vs 5+5). Each unordered pair
        // sits on its single direct edge → every edge has betweenness 1.0
        // (matches K3 unweighted PR-010 semantics).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let eb = edge_betweenness_weighted(&g, &[5.0, 5.0, 1.0]).unwrap();
        close(&eb, &[1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn directed_unit_weights_match_unweighted_path_3() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let eb = edge_betweenness_weighted(&g, &[1.0, 1.0]).unwrap();
        close(&eb, &[2.0, 2.0], 1e-12);
    }

    #[test]
    fn k4_unit_weights_each_edge_one() {
        // K4 with unit weights: every direct pair sits on its own edge
        // only → each edge betweenness = 1.0 (raw=2 over both endpoints,
        // halved to 1.0).
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let eb = edge_betweenness_weighted(&g, &[1.0; 6]).unwrap();
        close(&eb, &[1.0; 6], 1e-12);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(edge_betweenness_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(edge_betweenness_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(edge_betweenness_weighted(&g, &[f64::NAN]).is_err());
    }
}
