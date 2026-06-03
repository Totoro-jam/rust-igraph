//! Betweenness centrality (ALGO-PR-008).
//!
//! Counterpart of `igraph_betweenness()` from
//! `references/igraph/src/centrality/betweenness.c:504+`.
//!
//! Implements Brandes' (2001) BFS-based algorithm for unweighted graphs.
//! For each source vertex `s`:
//! 1. BFS that records `sigma[v]` = number of shortest paths from `s` to
//!    `v`, and `P[v]` = predecessor list of `v` on shortest paths.
//! 2. Process vertices in non-increasing distance from `s` (LIFO stack
//!    `S`); accumulate dependencies `delta[v] = Σ (sigma[v]/sigma[w]) *
//!    (1 + delta[w])` over each child `w` of `v`.
//! 3. For every `w != s`, add `delta[w]` to the running betweenness sum.
//!
//! For undirected graphs the result is divided by 2 (each unordered
//! pair is counted from both endpoints).
//!
//! Phase-1 minimal slice: undirected / `IGRAPH_OUT`, unweighted, raw counts
//! (`normalized = false`). Weighted version (Dijkstra) and per-vertex
//! selection ship in PR-008b.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Per-vertex (unweighted) betweenness centrality.
///
/// `result[v]` is the raw number of shortest paths between *all*
/// pairs `(s, t)` (`s != v != t`) that pass through `v`. For
/// undirected graphs the result is halved (each unordered pair `(s, t)`
/// is counted once).
///
/// Counterpart of
/// `igraph_betweenness(_, _, vss_all(), /*directed=*/g.is_directed(), NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, betweenness};
///
/// // Path 0-1-2-3-4: only inner vertices have nonzero betweenness.
/// // Vertex 1 lies on shortest paths (0,2),(0,3),(0,4) → 3 pairs.
/// // Vertex 2: (0,3),(0,4),(1,3),(1,4) → 4 pairs.
/// // Vertex 3: 3 pairs. Endpoints 0, 4: 0 pairs.
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i + 1).unwrap(); }
/// let b = betweenness(&g).unwrap();
/// assert_eq!(b, vec![0.0, 3.0, 4.0, 3.0, 0.0]);
/// ```
pub fn betweenness(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut betweenness = vec![0.0_f64; n_us];

    if n == 0 {
        return Ok(betweenness);
    }

    // Per-vertex BFS using `neighbors()` (out-edges for directed, all for undirected).
    // sigma[v] = number of shortest paths from s to v
    // dist[v] = -1 (unvisited) initially, else the BFS layer index from s
    // pred[v] = list of predecessors of v on a shortest path from s
    let mut sigma = vec![0.0_f64; n_us];
    let mut dist = vec![-1i64; n_us];
    let mut pred: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    let mut stack: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut delta = vec![0.0_f64; n_us];

    for s in 0..n {
        // Reset per-source state.
        sigma.fill(0.0);
        dist.fill(-1);
        for slot in &mut pred {
            slot.clear();
        }
        stack.clear();
        delta.fill(0.0);

        sigma[s as usize] = 1.0;
        dist[s as usize] = 0;
        let mut queue: VecDeque<VertexId> = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for w in graph.neighbors_iter(v)? {
                if dist[w as usize] < 0 {
                    queue.push_back(w);
                    dist[w as usize] = dist[v as usize] + 1;
                }
                if dist[w as usize] == dist[v as usize] + 1 {
                    sigma[w as usize] += sigma[v as usize];
                    pred[w as usize].push(v);
                }
            }
        }

        // Accumulate dependencies in reverse-BFS order.
        while let Some(w) = stack.pop() {
            for &v in &pred[w as usize] {
                delta[v as usize] +=
                    (sigma[v as usize] / sigma[w as usize]) * (1.0 + delta[w as usize]);
            }
            if w != s {
                betweenness[w as usize] += delta[w as usize];
            }
        }
    }

    // Undirected: divide by 2 (every unordered pair (s,t) was counted twice).
    if !graph.is_directed() {
        for slot in &mut betweenness {
            *slot /= 2.0;
        }
    }

    Ok(betweenness)
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
        assert!(betweenness(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_all_zero() {
        let g = Graph::with_vertices(5);
        let b = betweenness(&g).unwrap();
        assert_eq!(b, vec![0.0; 5]);
    }

    #[test]
    fn k3_triangle_all_zero() {
        // Every pair of vertices has a direct edge — no intermediates.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let b = betweenness(&g).unwrap();
        close(&b, &[0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn path_5_betweenness() {
        // Pairs through vertex 1: (0,2),(0,3),(0,4) = 3.
        // Through 2: (0,3),(0,4),(1,3),(1,4) = 4.
        // Through 3: (0,4),(1,4),(2,4) = 3.
        // Endpoints 0, 4 = 0.
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let b = betweenness(&g).unwrap();
        close(&b, &[0.0, 3.0, 4.0, 3.0, 0.0], 1e-12);
    }

    #[test]
    fn star_centre_has_max() {
        // 4-star: centre on every (leaf, leaf) shortest path → 3 pairs (C(3,2)).
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let b = betweenness(&g).unwrap();
        close(&b, &[3.0, 0.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn k4_complete_all_zero() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let b = betweenness(&g).unwrap();
        close(&b, &[0.0; 4], 1e-12);
    }

    #[test]
    fn cycle_4_all_one() {
        // 0-1-2-3-0. Pairs (0,2) and (1,3) are at distance 2 — there are two
        // shortest paths each (clockwise + counterclockwise). Each interior
        // vertex on one of those paths counts 0.5. Each vertex sits on
        // exactly one antipodal path → betweenness 0.5.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let b = betweenness(&g).unwrap();
        close(&b, &[0.5, 0.5, 0.5, 0.5], 1e-12);
    }

    #[test]
    fn directed_path_3_uses_out_edges() {
        // 0->1->2: only (0,2) has a path; goes through 1.
        // For directed unnormalized: betweenness[1] = 1.0 (1 ordered pair
        // (0,2) with the single shortest path passing through 1).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let b = betweenness(&g).unwrap();
        close(&b, &[0.0, 1.0, 0.0], 1e-12);
    }
}
