//! Weighted harmonic centrality (ALGO-PR-009b).
//!
//! Counterpart of `igraph_harmonic_centrality(_, _, vss_all(),
//! IGRAPH_OUT, &weights, /*normalized=*/true)` from
//! `references/igraph/src/centrality/closeness.c`. Same shape as
//! [`crate::harmonic_centrality`] but uses weighted (Dijkstra)
//! shortest distances instead of BFS hops.
//!
//! Phase-1 minimal slice: undirected/OUT, normalized = true.

use crate::algorithms::paths::dijkstra::dijkstra_distances;
use crate::core::{Graph, IgraphResult};

/// Per-vertex weighted harmonic centrality.
///
/// `result[v] = (1/(n-1)) * sum_{u != v, reachable} 1/d_w(v, u)`,
/// where `d_w` is the Dijkstra weighted distance. Unreachable vertices
/// contribute 0 (defining `1/inf = 0`), so harmonic centrality is
/// well-defined on disconnected graphs (unlike closeness).
///
/// `weights[e]` is the weight of edge `e`; weights must be
/// non-negative and finite (forwarded from
/// [`crate::dijkstra_distances`]). Zero-weight edges that produce a
/// distance of `0` between distinct vertices are silently skipped to
/// avoid division by zero.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, harmonic_centrality_weighted};
///
/// // Path 0-1-2 with weights (1, 1) recovers the unweighted answer.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let h = harmonic_centrality_weighted(&g, &[1.0, 1.0]).unwrap();
/// assert!((h[0] - 0.75).abs() < 1e-12);
/// assert!((h[1] - 1.00).abs() < 1e-12);
/// assert!((h[2] - 0.75).abs() < 1e-12);
/// ```
pub fn harmonic_centrality_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n < 2 {
        return Ok(vec![0.0; n_us]);
    }
    let mut out = Vec::with_capacity(n_us);
    let denom = f64::from(n - 1);
    for v in 0..n {
        let d = dijkstra_distances(graph, v, weights)?;
        let mut sum_inv: f64 = 0.0;
        let v_us = v as usize;
        for (target, val) in d.iter().enumerate() {
            if target == v_us {
                continue;
            }
            if let Some(dist) = val {
                if *dist > 0.0 {
                    sum_inv += 1.0 / *dist;
                }
            }
        }
        out.push(sum_inv / denom);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} vs {b}");
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(harmonic_centrality_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(harmonic_centrality_weighted(&g, &[]).unwrap(), vec![0.0]);
    }

    #[test]
    fn isolated_vertices_all_zero() {
        let g = Graph::with_vertices(3);
        assert_eq!(
            harmonic_centrality_weighted(&g, &[]).unwrap(),
            vec![0.0, 0.0, 0.0]
        );
    }

    #[test]
    fn unit_weights_match_unweighted() {
        // Path 0-1-2.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let hw = harmonic_centrality_weighted(&g, &[1.0, 1.0]).unwrap();
        let hu = crate::harmonic_centrality(&g).unwrap();
        for (a, b) in hw.iter().zip(hu.iter()) {
            close(*a, *b, 1e-12);
        }
    }

    #[test]
    fn weighted_path_3_with_doubled_middle_edge() {
        // 0-1 (w=1), 1-2 (w=2). Distances from 0: {1@1, 2@3}.
        // Vertex 0: 1/1 + 1/3 = 1.333...; /2 ≈ 0.6667.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality_weighted(&g, &[1.0, 2.0]).unwrap();
        // 0: 1 + 1/3 = 4/3; /2 = 2/3.
        close(h[0], 2.0 / 3.0, 1e-12);
        // 1: 1/1 + 1/2 = 1.5; /2 = 0.75.
        close(h[1], 0.75, 1e-12);
        // 2: 1/2 + 1/3 = 5/6; /2 = 5/12.
        close(h[2], 5.0 / 12.0, 1e-12);
    }

    #[test]
    fn directed_path_uses_out_edges() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality_weighted(&g, &[1.0, 1.0]).unwrap();
        close(h[0], 0.75, 1e-12);
        close(h[1], 0.5, 1e-12);
        close(h[2], 0.0, 1e-12);
    }

    #[test]
    fn disconnected_components_well_defined() {
        // {0-1-2} and isolated 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality_weighted(&g, &[1.0, 1.0]).unwrap();
        // 0: 1 + 0.5 = 1.5; /3 = 0.5.
        close(h[0], 0.5, 1e-12);
        // 1: 1 + 1 = 2; /3 = 2/3.
        close(h[1], 2.0 / 3.0, 1e-12);
        close(h[2], 0.5, 1e-12);
        close(h[3], 0.0, 1e-12);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(harmonic_centrality_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(harmonic_centrality_weighted(&g, &[-1.0]).is_err());
    }
}
