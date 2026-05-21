//! Global + local efficiency (ALGO-PR-029, ALGO-PR-030).
//!
//! Counterpart of `igraph_global_efficiency()` from
//! `references/igraph/src/paths/shortest_paths.c:392` (and the underlying
//! `igraph_i_average_path_length_unweighted` helper at line 38, called
//! with `invert = true, unconn = false`); plus `igraph_local_efficiency()`
//! at line 688 and `igraph_average_local_efficiency()` at line 842.
//!
//! Definition (global): `E_g = 1/(N*(N-1)) * sum_{i != j} 1/d(i, j)`.
//! Pairs that are unreachable contribute 0 (treated as `1/inf`).
//! Returns `None` when `vcount < 2` (no ordered pairs to average over —
//! upstream returns NaN; we model this as `Option<f64>` to match the
//! rest of the Phase-1 averaging APIs).
//!
//! Definition (local): for each vertex `v`, let `N(v)` be the unique
//! neighbours (self-loops dropped). The local efficiency around `v` is
//! the average inverse shortest-path distance between every ordered
//! pair `(s, t)` of distinct neighbours, computed in the *induced
//! subgraph* `G \ {v}` — i.e. paths must not pass through `v`.
//! `local_efficiency[v] = 0` when `|N(v)| < 2`. The average over all
//! vertices is `average_local_efficiency`.
//!
//! Phase-1 minimal slice: unweighted only. Edge directions are followed
//! for directed graphs (`distances()` walks OUT edges) — that matches
//! upstream's `directed = true` default.
//!
//! Reference: V. Latora and M. Marchiori, "Efficient Behavior of
//! Small-World Networks", Phys. Rev. Lett. 87, 198701 (2001); and
//! I. Vragović, E. Louis, A. Díaz-Guilera, "Efficiency of informational
//! transfer in regular and complex networks", Phys. Rev. E 71, 036122
//! (2005).

use std::collections::VecDeque;

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Global efficiency of `graph` — average inverse pairwise shortest
/// distance over all `N*(N-1)` ordered vertex pairs. Pairs that are
/// unreachable contribute 0.
///
/// Returns `None` when `vcount() < 2` (no pairs).
///
/// For undirected graphs each unordered pair contributes twice (once
/// per direction); the divisor `N*(N-1)` mirrors that, so the formula
/// is the standard Latora–Marchiori definition.
///
/// Counterpart of
/// `igraph_global_efficiency(_, NULL_weights, _, /*directed=*/true)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, global_efficiency};
///
/// // K3: every ordered pair is at distance 1 → mean inverse distance = 1.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(global_efficiency(&g).unwrap(), Some(1.0));
///
/// // Path 0-1-2-3: 12 ordered pairs. d=1 ×6 → 6; d=2 ×4 → 2; d=3 ×2 → 2/3.
/// // Sum = 26/3; /12 = 13/18.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3u32 { g.add_edge(i, i + 1).unwrap(); }
/// let e = global_efficiency(&g).unwrap().unwrap();
/// assert!((e - 13.0 / 18.0).abs() < 1e-12);
/// ```
pub fn global_efficiency(graph: &Graph) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    if n < 2 {
        return Ok(None);
    }
    let mut sum_inv: f64 = 0.0;
    for v in 0..n {
        let d = distances(graph, v)?;
        let v_us = v as usize;
        for (target, &val) in d.iter().enumerate() {
            if target == v_us {
                continue;
            }
            if let Some(dist) = val {
                if dist > 0 {
                    sum_inv += 1.0 / f64::from(dist);
                }
            }
        }
    }
    let n_f = f64::from(n);
    let denom = n_f * (n_f - 1.0);
    Ok(Some(sum_inv / denom))
}

/// Per-vertex local efficiency. For each vertex `v`, computes the
/// average inverse distance between every ordered pair of distinct
/// vertices in `N(v)` (the unique non-self neighbours of `v`),
/// **measured in the subgraph obtained by removing `v`** — paths must
/// not pass through `v`. Pairs unreachable in `G \ {v}` contribute 0.
///
/// `local_efficiency[v] = 0` whenever `|N(v)| < 2`. For directed graphs,
/// `N(v)` is the set of OUT-neighbours and BFS follows OUT edges (mirrors
/// the simple `directed=true, mode=OUT` slice we expose for [`distances`]
/// and [`global_efficiency`]).
///
/// Counterpart of
/// `igraph_local_efficiency(_, NULL_weights, _, igraph_vss_all(),
///  /*directed=*/true, /*mode=*/IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_efficiency};
///
/// // K4: each vertex has 3 neighbours forming a K3, all at distance 1
/// // in G \ {v} → local efficiency = 1.0 at every vertex.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i + 1)..4u32 { g.add_edge(i, j).unwrap(); }
/// }
/// assert_eq!(local_efficiency(&g).unwrap(), vec![1.0, 1.0, 1.0, 1.0]);
/// ```
pub fn local_efficiency(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    let mut result = vec![0.0_f64; n_us];

    if n < 3 {
        return Ok(result);
    }

    let mut nei_mask = vec![false; n_us];

    for v in 0..n {
        let raw = graph.neighbors(v)?;
        let mut neighbors: Vec<VertexId> = raw.into_iter().filter(|&u| u != v).collect();
        neighbors.sort_unstable();
        neighbors.dedup();
        let k = neighbors.len();
        if k < 2 {
            continue;
        }

        for &u in &neighbors {
            nei_mask[u as usize] = true;
        }

        let mut sum_inv: f64 = 0.0;
        for &source in &neighbors {
            let mut dist: Vec<Option<u32>> = vec![None; n_us];
            dist[source as usize] = Some(0);
            let mut queue: VecDeque<VertexId> = VecDeque::new();
            queue.push_back(source);
            let mut reached = 0_usize;

            'bfs: while let Some(node) = queue.pop_front() {
                let cur = dist[node as usize].ok_or(IgraphError::Internal(
                    "dequeued unvisited vertex in local_efficiency BFS",
                ))?;
                if node != source && nei_mask[node as usize] && cur > 0 {
                    sum_inv += 1.0 / f64::from(cur);
                    reached += 1;
                    if reached + 1 == k {
                        break 'bfs;
                    }
                }
                let next = cur.checked_add(1).ok_or(IgraphError::Internal(
                    "distance overflow in local_efficiency",
                ))?;
                for w in graph.neighbors(node)? {
                    if w == v {
                        continue;
                    }
                    if dist[w as usize].is_none() {
                        dist[w as usize] = Some(next);
                        queue.push_back(w);
                    }
                }
            }
        }

        for &u in &neighbors {
            nei_mask[u as usize] = false;
        }

        let k_u32 = u32::try_from(k)
            .map_err(|_| IgraphError::Internal("neighbour count exceeds u32::MAX"))?;
        let k_f = f64::from(k_u32);
        result[v as usize] = sum_inv / (k_f * (k_f - 1.0));
    }

    Ok(result)
}

/// Average of [`local_efficiency`] over all `N` vertices. By upstream
/// convention, returns `0.0` when `vcount < 3` (no vertex can have two
/// distinct neighbours, so every per-vertex value is trivially 0).
///
/// Counterpart of
/// `igraph_average_local_efficiency(_, NULL_weights, _,
///  /*directed=*/true, /*mode=*/IGRAPH_OUT)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, average_local_efficiency};
///
/// // K4: every vertex has local efficiency 1.0, so the average is 1.0.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 {
///     for j in (i + 1)..4u32 { g.add_edge(i, j).unwrap(); }
/// }
/// assert_eq!(average_local_efficiency(&g).unwrap(), 1.0);
/// ```
pub fn average_local_efficiency(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n < 3 {
        return Ok(0.0);
    }
    let local = local_efficiency(graph)?;
    let n_f = f64::from(n);
    Ok(local.iter().sum::<f64>() / n_f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} vs {b}");
    }

    #[test]
    fn empty_graph_returns_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(global_efficiency(&g).unwrap(), None);
    }

    #[test]
    fn singleton_returns_none() {
        let g = Graph::with_vertices(1);
        assert_eq!(global_efficiency(&g).unwrap(), None);
    }

    #[test]
    fn no_edges_two_vertices_zero() {
        // No reachable pairs → sum 0 → 0/2 = 0.
        let g = Graph::with_vertices(2);
        assert_eq!(global_efficiency(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn complete_graph_one() {
        // K_n: every ordered pair at distance 1 → mean = 1.
        for n in 2..=5u32 {
            let mut g = Graph::with_vertices(n);
            for u in 0..n {
                for v in (u + 1)..n {
                    g.add_edge(u, v).unwrap();
                }
            }
            close(global_efficiency(&g).unwrap().unwrap(), 1.0, 1e-12);
        }
    }

    #[test]
    fn path_3_two_thirds() {
        // 0-1-2: distances among 6 ordered pairs: (0,1)=1, (0,2)=2,
        // (1,0)=1, (1,2)=1, (2,0)=2, (2,1)=1. Inverses sum = 4*1 + 2*0.5 = 5.
        // /6 = 5/6.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let e = global_efficiency(&g).unwrap().unwrap();
        close(e, 5.0 / 6.0, 1e-12);
    }

    #[test]
    fn path_4_thirteen_eighteenths() {
        // 0-1-2-3 ordered pairs:
        //   d=1: 6 pairs → contrib 6.
        //   d=2: 4 pairs → contrib 2.
        //   d=3: 2 pairs → contrib 2/3.
        // Sum = 26/3. /12 = 13/18.
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let e = global_efficiency(&g).unwrap().unwrap();
        close(e, 13.0 / 18.0, 1e-12);
    }

    #[test]
    fn isolated_vertices_zero() {
        // Three isolated vertices: no reachable pairs → 0.
        let g = Graph::with_vertices(3);
        assert_eq!(global_efficiency(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn disconnected_two_components() {
        // {0-1}, {2}: ordered pairs (0,1) and (1,0) at d=1 → contrib 2.
        // Other 4 pairs unreachable → 0. Sum = 2; /6 = 1/3.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let e = global_efficiency(&g).unwrap().unwrap();
        close(e, 1.0 / 3.0, 1e-12);
    }

    #[test]
    fn directed_path_uses_out_edges() {
        // 0->1->2: reachable pairs (0,1)=1, (0,2)=2, (1,2)=1.
        // Inverses sum = 1 + 0.5 + 1 = 2.5. /6 = 5/12.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let e = global_efficiency(&g).unwrap().unwrap();
        close(e, 5.0 / 12.0, 1e-12);
    }

    #[test]
    fn star_efficiency() {
        // Star K_{1,3}: centre 0; leaves 1,2,3.
        // Pairs at d=1: (0,1)(0,2)(0,3) ×2 = 6 → contrib 6.
        // Pairs at d=2 (between leaves): 3 unordered, ×2 = 6 → contrib 3.
        // Sum = 9. N=4 → /12 = 0.75.
        let mut g = Graph::with_vertices(4);
        for v in 1..4u32 {
            g.add_edge(0, v).unwrap();
        }
        let e = global_efficiency(&g).unwrap().unwrap();
        close(e, 0.75, 1e-12);
    }

    #[test]
    fn matches_harmonic_centrality_average() {
        // Identity: global_efficiency = sum(harmonic_centrality) / n.
        // Verify on a small graph.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let e = global_efficiency(&g).unwrap().unwrap();
        let h = crate::algorithms::properties::harmonic::harmonic_centrality(&g).unwrap();
        let avg: f64 = h.iter().sum::<f64>() / f64::from(u32::try_from(h.len()).unwrap());
        close(e, avg, 1e-12);
    }

    #[test]
    fn efficiency_in_range() {
        // For any unweighted graph: 0 ≤ E_g ≤ 1.
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(0, 5).unwrap(); // 6-cycle
        let e = global_efficiency(&g).unwrap().unwrap();
        assert!((0.0..=1.0).contains(&e), "{e}");
    }

    // ----- local_efficiency / average_local_efficiency tests (PR-030) -----

    #[test]
    fn local_eff_empty_graph_empty_vec() {
        let g = Graph::with_vertices(0);
        assert!(local_efficiency(&g).unwrap().is_empty());
    }

    #[test]
    fn local_eff_singleton_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(local_efficiency(&g).unwrap(), vec![0.0]);
    }

    #[test]
    fn local_eff_two_vertices_zero() {
        // n < 3 → all per-vertex values are 0 by convention.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert_eq!(local_efficiency(&g).unwrap(), vec![0.0, 0.0]);
    }

    #[test]
    fn local_eff_isolated_three_vertices_zero() {
        let g = Graph::with_vertices(3);
        assert_eq!(local_efficiency(&g).unwrap(), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn local_eff_path_three_zero() {
        // 0-1-2: vertex 1 has neighbours {0,2}; in G\{1} they are
        // disconnected → no path → contribution 0. Vertices 0 and 2
        // have one neighbour each → 0 by convention.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(local_efficiency(&g).unwrap(), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn local_eff_triangle_zero() {
        // K3: every vertex has 2 neighbours; in G\{v} those two
        // neighbours are disconnected (only the edge through v is gone,
        // but the direct edge between them remains).
        // Wait: in K3, vertices 1 and 2 are adjacent. So in G\{0},
        // 1-2 is still an edge → distance 1 → local[0] = 2 / (2*1) = 1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let local = local_efficiency(&g).unwrap();
        for v in &local {
            close(*v, 1.0, 1e-12);
        }
    }

    #[test]
    fn local_eff_k4_all_one() {
        // K4: each vertex's neighbour set is K3, all at distance 1 in
        // G\{v} → local efficiency 1 at every vertex.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        let local = local_efficiency(&g).unwrap();
        for v in &local {
            close(*v, 1.0, 1e-12);
        }
    }

    #[test]
    fn local_eff_star_zero() {
        // K_{1,3}: centre 0 has 3 neighbours but in G\{0} they are
        // mutually unreachable → local[0] = 0. Each leaf has 1
        // neighbour → 0 by convention.
        let mut g = Graph::with_vertices(4);
        for v in 1..4u32 {
            g.add_edge(0, v).unwrap();
        }
        assert_eq!(local_efficiency(&g).unwrap(), vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn local_eff_diamond() {
        // Diamond: 0-1, 0-2, 0-3, 1-2, 2-3 (so vertex 2 is adjacent to
        // 0,1,3; vertex 0 to 1,2,3; vertices 1 and 3 are adjacent to
        // 0 and 2 each).
        // Vertex 0's neighbours = {1,2,3}. In G\{0}: 1-2 (edge), 2-3
        // (edge), 1-3 (no edge, but reachable via 2 at distance 2).
        // Ordered pair contributions: (1,2)=(2,1)=1; (2,3)=(3,2)=1;
        // (1,3)=(3,1)=1/2. Sum = 4 + 1 = 5. /(3*2)=6 → 5/6.
        // Vertex 2 is symmetric to vertex 0 → also 5/6.
        // Vertex 1's neighbours = {0,2}: edge 0-2 in G\{1} → distance
        // 1. Sum=2; /(2*1)=2 → 1.0.
        // Vertex 3 symmetric to vertex 1 → 1.0.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let local = local_efficiency(&g).unwrap();
        close(local[0], 5.0 / 6.0, 1e-12);
        close(local[1], 1.0, 1e-12);
        close(local[2], 5.0 / 6.0, 1e-12);
        close(local[3], 1.0, 1e-12);
    }

    #[test]
    fn local_eff_ignores_self_loops_and_parallel_edges() {
        // Self-loops should not count as neighbours; parallel edges
        // collapse to one neighbour.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // self-loop on 0
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        // Vertex 0 has neighbours {1,2}; 1-2 is edge → local[0] = 1.0.
        // Vertex 1 has neighbours {0,2}; 0-2 is edge → local[1] = 1.0.
        // Vertex 2 has neighbours {0,1}; 0-1 is edge → local[2] = 1.0.
        let local = local_efficiency(&g).unwrap();
        for v in &local {
            close(*v, 1.0, 1e-12);
        }
    }

    #[test]
    fn average_local_eff_empty_zero() {
        let g = Graph::with_vertices(0);
        close(average_local_efficiency(&g).unwrap(), 0.0, 1e-12);
    }

    #[test]
    fn average_local_eff_lt_three_zero() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        close(average_local_efficiency(&g).unwrap(), 0.0, 1e-12);
    }

    #[test]
    fn average_local_eff_k4_one() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        close(average_local_efficiency(&g).unwrap(), 1.0, 1e-12);
    }

    #[test]
    fn average_local_eff_diamond() {
        // Diamond has local = [5/6, 1, 5/6, 1] → avg = (5/6+1+5/6+1)/4 = 11/12.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        close(average_local_efficiency(&g).unwrap(), 11.0 / 12.0, 1e-12);
    }

    #[test]
    fn average_local_eff_path_zero() {
        // Path 0-1-2-3: vertex 1's neighbours {0,2} disconnected in
        // G\{1}; vertex 2 symmetric. All others <2 neighbours → all 0.
        let mut g = Graph::with_vertices(4);
        for i in 0..3u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        close(average_local_efficiency(&g).unwrap(), 0.0, 1e-12);
    }
}
