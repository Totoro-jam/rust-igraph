//! Harmonic centrality (ALGO-PR-009).
//!
//! Counterpart of `igraph_harmonic_centrality()` from
//! `references/igraph/src/centrality/closeness.c:800-805` (and the
//! underlying `igraph_harmonic_centrality_cutoff`). Defined as the mean
//! inverse distance to all other vertices, with `1/inf == 0` for
//! unreachable pairs.
//!
//! Phase-1 minimal slice: undirected / `IGRAPH_OUT`, unweighted, normalized.
//! Returns `f64` per vertex (no `Option`); a vertex with no reachable
//! neighbours simply gets 0.0 (matching python-igraph's behaviour).

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

/// Per-vertex harmonic centrality.
///
/// `result[v] = (1/(n-1)) * sum_{u != v, reachable} 1/d(v, u)`. The
/// inverse distance to unreachable vertices is taken as 0, so harmonic
/// centrality is well-defined on disconnected graphs (unlike closeness).
///
/// For graphs with `vcount < 2` returns an empty / single-zero vector
/// (the normalization factor `n - 1` is 0 in that case; we report 0
/// rather than NaN to match python-igraph 0.11.x).
///
/// Counterpart of
/// `igraph_harmonic_centrality(_, _, vss_all(), IGRAPH_OUT, NULL, /*normalized=*/true)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, harmonic_centrality};
///
/// // Path 0-1-2: vertex 1 (centre) has harmonic 1.0; ends 0.75.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let h = harmonic_centrality(&g).unwrap();
/// assert_eq!(h, vec![0.75, 1.0, 0.75]);
/// ```
pub fn harmonic_centrality(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n < 2 {
        return Ok(vec![0.0; n_us]);
    }
    let mut out = Vec::with_capacity(n_us);
    let denom = f64::from(n - 1);
    for v in 0..n {
        let d = distances(graph, v)?;
        let mut sum_inv: f64 = 0.0;
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
        assert!(harmonic_centrality(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_zero() {
        let g = Graph::with_vertices(1);
        assert_eq!(harmonic_centrality(&g).unwrap(), vec![0.0]);
    }

    #[test]
    fn isolated_vertices_all_zero() {
        let g = Graph::with_vertices(3);
        assert_eq!(harmonic_centrality(&g).unwrap(), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn path_3_centre_one_ends_three_quarters() {
        // 0-1-2: centre has 1/1 + 1/1 = 2; / (3-1) = 1.0.
        // End: 1/1 + 1/2 = 1.5; / 2 = 0.75.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality(&g).unwrap();
        assert_eq!(h, vec![0.75, 1.0, 0.75]);
    }

    #[test]
    fn k3_uniform_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let h = harmonic_centrality(&g).unwrap();
        for &x in &h {
            close(x, 1.0, 1e-12);
        }
    }

    #[test]
    fn star_centre_one_leaves_two_thirds() {
        // Centre: 3 leaves at distance 1 → sum = 3, /3 = 1.0.
        // Leaf: centre at 1 + 2 leaves at 2 → 1 + 0.5 + 0.5 = 2; /3 = 2/3.
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let h = harmonic_centrality(&g).unwrap();
        close(h[0], 1.0, 1e-12);
        for &leaf in &h[1..4] {
            close(leaf, 2.0 / 3.0, 1e-12);
        }
    }

    #[test]
    fn disconnected_components_well_defined() {
        // {0-1-2} and {3} (isolated). Vertex 0: reach {1,2} sum=1+0.5 = 1.5; /3 = 0.5.
        // Vertex 3: no reach → 0/3 = 0.
        // Unlike closeness, harmonic is defined here.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality(&g).unwrap();
        close(h[0], 0.5, 1e-12);
        close(h[1], 2.0 / 3.0, 1e-12); // 1/1 + 1/1 = 2; /3.
        close(h[2], 0.5, 1e-12);
        close(h[3], 0.0, 1e-12);
    }

    #[test]
    fn directed_path_uses_out_edges() {
        // 0->1->2: from 0 reach {1, 2}, from 1 reach {2}, from 2 reach {} (= 0).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = harmonic_centrality(&g).unwrap();
        // 0: 1/1 + 1/2 = 1.5; / 2 = 0.75.
        close(h[0], 0.75, 1e-12);
        // 1: 1/1 = 1; / 2 = 0.5.
        close(h[1], 0.5, 1e-12);
        // 2: 0; / 2 = 0.
        close(h[2], 0.0, 1e-12);
    }
}
