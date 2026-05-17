//! Edge betweenness centrality (ALGO-PR-010).
//!
//! Counterpart of `igraph_edge_betweenness()` from
//! `references/igraph/src/centrality/betweenness.c:766+`. Same Brandes
//! framework as vertex betweenness ([`super::betweenness::betweenness`]) but
//! accumulating the dependency on edge ids rather than vertices.
//!
//! For each source `s`:
//! 1. BFS that records `sigma[v]` (number of shortest paths from `s`)
//!    and `pred_edges[w]` = `(predecessor_vertex, edge_id)` pairs along
//!    shortest paths into `w`.
//! 2. Reverse-BFS dependency accumulation:
//!    `c = (sigma[v] / sigma[w]) * (1 + delta_v[w])`,
//!    then `delta_v[v] += c` and `betweenness_e[e] += c`.
//!
//! For undirected graphs the result is halved (each unordered pair is
//! discovered from both endpoints).
//!
//! Phase-1 minimal slice: undirected / `IGRAPH_OUT`, unweighted, raw
//! counts (`normalized = false`).

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Per-edge unweighted betweenness centrality.
///
/// `result[e]` is the number of shortest paths between all vertex
/// pairs `(s, t)` (`s != t`) that use edge `e`. For undirected graphs
/// the result is halved (each unordered pair counted once).
///
/// Counterpart of
/// `igraph_edge_betweenness(_, NULL_weights, _, /*eids=*/all, /*directed=*/g.is_directed(), /*normalized=*/false)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_betweenness};
///
/// // Path 0-1-2-3 (3 edges): every edge sits on every pair through it.
/// // Edge 0 (0-1) carries (0,1),(0,2),(0,3) = 3 pairs.
/// // Edge 1 (1-2) carries (0,2),(0,3),(1,2),(1,3) = 4 pairs.
/// // Edge 2 (2-3) carries (0,3),(1,3),(2,3) = 3 pairs.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3u32 { g.add_edge(i, i + 1).unwrap(); }
/// let eb = edge_betweenness(&g).unwrap();
/// assert_eq!(eb, vec![3.0, 4.0, 3.0]);
/// ```
pub fn edge_betweenness(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    let mut betweenness_e = vec![0.0_f64; m];

    if n == 0 || m == 0 {
        return Ok(betweenness_e);
    }

    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1i64; n_us];
    let mut pred: Vec<Vec<(VertexId, EdgeId)>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta_v = vec![0.0_f64; n_us];

    for s in 0..n {
        sigma.fill(0.0);
        dist.fill(-1);
        for slot in &mut pred {
            slot.clear();
        }
        stack.clear();
        delta_v.fill(0.0);

        sigma[s as usize] = 1.0;
        dist[s as usize] = 0;
        let mut queue: VecDeque<VertexId> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for e in graph.incident(v)? {
                let w = graph.edge_other(e, v)?;
                if dist[w as usize] < 0 {
                    queue.push_back(w);
                    dist[w as usize] = dist[v as usize] + 1;
                }
                if dist[w as usize] == dist[v as usize] + 1 {
                    sigma[w as usize] += sigma[v as usize];
                    pred[w as usize].push((v, e));
                }
            }
        }

        // Reverse-BFS dependency accumulation, deposited on edges.
        while let Some(w) = stack.pop() {
            for &(v, e) in &pred[w as usize] {
                let c = (sigma[v as usize] / sigma[w as usize]) * (1.0 + delta_v[w as usize]);
                delta_v[v as usize] += c;
                betweenness_e[e as usize] += c;
            }
        }
    }

    // Undirected: every unordered pair was counted twice (from both endpoints).
    if !graph.is_directed() {
        for slot in &mut betweenness_e {
            *slot /= 2.0;
        }
    }

    Ok(betweenness_e)
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
        assert!(edge_betweenness(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_no_edges_empty() {
        let g = Graph::with_vertices(5);
        assert!(edge_betweenness(&g).unwrap().is_empty());
    }

    #[test]
    fn path_3_edge_betweenness() {
        // 0-1-2: edge 0 (0-1) carries (0,1),(0,2)=2; edge 1 (1-2) carries (0,2),(1,2)=2.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[2.0, 2.0], 1e-12);
    }

    #[test]
    fn path_4_edge_betweenness() {
        // 0-1-2-3: edges 0:(0,1), 1:(1,2), 2:(2,3).
        // Edge 0 = pairs through {0-1}: (0,1),(0,2),(0,3) = 3.
        // Edge 1 = (0,2),(0,3),(1,2),(1,3) = 4.
        // Edge 2 = (0,3),(1,3),(2,3) = 3.
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[3.0, 4.0, 3.0], 1e-12);
    }

    #[test]
    fn k3_triangle_uniform_one() {
        // Each edge sits on exactly one pair-shortest-path of length 1 →
        // 3 pairs, 1 edge each direct → each edge has betweenness 1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[1.0, 1.0, 1.0], 1e-12);
    }

    #[test]
    fn star_4_each_edge_three() {
        // 0-1, 0-2, 0-3: each edge connects centre to one leaf;
        // each edge serves (centre,leaf) + 2 (leaf,other-leaf) pairs = 3.
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[3.0, 3.0, 3.0], 1e-12);
    }

    #[test]
    fn cycle_4_uniform_two() {
        // 0-1, 1-2, 2-3, 3-0. Each direct adjacent pair contributes 1.0
        // to its edge. Antipodal pairs (0,2) and (1,3) each have two
        // shortest paths of length 2; each pair contributes 0.5 to all 4
        // edges. Per-edge total: 1.0 + 2 * 0.5 = 2.0. (Verified vs
        // python-igraph 0.11.)
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[2.0, 2.0, 2.0, 2.0], 1e-12);
    }

    #[test]
    fn directed_path_3_edge_betweenness() {
        // 0->1->2: edge 0 = (0,1),(0,2) = 2 (ordered); edge 1 = (0,2),(1,2) = 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let eb = edge_betweenness(&g).unwrap();
        close(&eb, &[2.0, 2.0], 1e-12);
    }
}
