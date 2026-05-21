//! Global efficiency (ALGO-PR-029).
//!
//! Counterpart of `igraph_global_efficiency()` from
//! `references/igraph/src/paths/shortest_paths.c:392` (and the underlying
//! `igraph_i_average_path_length_unweighted` helper at line 38, called
//! with `invert = true, unconn = false`).
//!
//! Definition: `E_g = 1/(N*(N-1)) * sum_{i != j} 1/d(i, j)`. Pairs that
//! are unreachable contribute 0 (treated as `1/inf`). Returns `None`
//! when `vcount < 2` (no ordered pairs to average over — upstream
//! returns NaN; we model this as `Option<f64>` to match the rest of the
//! Phase-1 averaging APIs).
//!
//! Phase-1 minimal slice: unweighted only. Edge directions are followed
//! for directed graphs (`distances()` walks OUT edges) — that matches
//! upstream's `directed = true` default.
//!
//! Reference: V. Latora and M. Marchiori, "Efficient Behavior of
//! Small-World Networks", Phys. Rev. Lett. 87, 198701 (2001).

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

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
}
