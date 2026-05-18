//! Weighted `PageRank` centrality (ALGO-PR-011b).
//!
//! Counterpart of `igraph_pagerank(_, IGRAPH_PAGERANK_ALGO_POWER, _, _,
//! vss_all(), directed, 0.85, &weights, NULL_options)` from
//! `references/igraph/src/centrality/pagerank.c`. Same power-iteration
//! framework as PR-011 but every edge `u → v` contributes its weight
//! into the in-flow term:
//!
//! ```text
//! PR_new[v] = (1 - d) / N
//!           + d * Σ_{u → v} w(u, v) * PR[u] / out_strength(u)
//!           + d * dangling_sum / N
//! ```
//!
//! where `out_strength(u) = Σ_{e: u → x} w(e)` and a vertex is
//! "dangling" iff its `out_strength == 0`. For undirected graphs each
//! edge contributes to both endpoints' strengths and propagates rank
//! in both directions.
//!
//! Phase-1 minimal slice: damping `0.85`, `eps = 1e-10`, `max_iter =
//! 1000`. Weights must be non-negative + finite; violations return
//! [`crate::IgraphError::InvalidArgument`].

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

const DEFAULT_DAMPING: f64 = 0.85;
const DEFAULT_EPS: f64 = 1e-10;
const DEFAULT_MAX_ITER: usize = 1000;

/// Weighted `PageRank` scores via power iteration with damping `0.85`.
///
/// Returns a `Vec<f64>` summing approximately to 1. `weights[e]` is
/// the weight of edge `e`; weights must be non-negative and finite,
/// and `weights.len()` must equal `graph.ecount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pagerank_weighted};
///
/// // Triangle with unit weights → matches unweighted PR-011 (uniform 1/3).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let pr = pagerank_weighted(&g, &[1.0, 1.0, 1.0]).unwrap();
/// for v in 0..3 {
///     assert!((pr[v] - 1.0/3.0).abs() < 1e-9);
/// }
/// ```
pub fn pagerank_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![1.0]);
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

    let directed = graph.is_directed();
    let n_f = f64::from(n);

    // Out-strength per vertex + weighted in-adjacency `(u, w/strength)`.
    let mut out_strength = vec![0.0_f64; n_us];
    let m_u = u32::try_from(m).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    if directed {
        for e in 0..m_u {
            let (u, _v) = graph.edge(e as EdgeId)?;
            out_strength[u as usize] += weights[e as usize];
        }
    } else {
        for e in 0..m_u {
            let (u, v) = graph.edge(e as EdgeId)?;
            // Self-loop: contributes 2*w to its vertex's "out-strength"
            // because both directions count.
            if u == v {
                out_strength[u as usize] += 2.0 * weights[e as usize];
            } else {
                out_strength[u as usize] += weights[e as usize];
                out_strength[v as usize] += weights[e as usize];
            }
        }
    }

    // Build weighted in-adjacency: for each in-edge u→v contribute
    // (u, weight(u→v)) so the power-iteration loop can compute
    //     incoming[v] = Σ (w * PR[u] / out_strength[u]).
    let mut in_adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n_us];
    if directed {
        for e in 0..m_u {
            let (u, v) = graph.edge(e as EdgeId)?;
            in_adj[v as usize].push((u, weights[e as usize]));
        }
    } else {
        for e in 0..m_u {
            let (u, v) = graph.edge(e as EdgeId)?;
            let edge_w = weights[e as usize];
            if u == v {
                in_adj[v as usize].push((u, edge_w));
                in_adj[v as usize].push((u, edge_w));
            } else {
                in_adj[u as usize].push((v, edge_w));
                in_adj[v as usize].push((u, edge_w));
            }
        }
    }

    let mut pr = vec![1.0 / n_f; n_us];
    let mut pr_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        let mut dangling_sum: f64 = 0.0;
        for v in 0..n_us {
            if out_strength[v] == 0.0 {
                dangling_sum += pr[v];
            }
        }

        let teleport = (1.0 - DEFAULT_DAMPING) / n_f;
        let dangling_share = DEFAULT_DAMPING * dangling_sum / n_f;

        for v in 0..n_us {
            let mut incoming: f64 = 0.0;
            for &(u, w) in &in_adj[v] {
                let s = out_strength[u as usize];
                if s > 0.0 {
                    incoming += w * pr[u as usize] / s;
                }
            }
            pr_new[v] = teleport + dangling_share + DEFAULT_DAMPING * incoming;
        }

        let mut diff: f64 = 0.0;
        for v in 0..n_us {
            diff += (pr_new[v] - pr[v]).abs();
        }
        std::mem::swap(&mut pr, &mut pr_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    Ok(pr)
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
        assert!(pagerank_weighted(&g, &[]).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(pagerank_weighted(&g, &[]).unwrap(), vec![1.0]);
    }

    #[test]
    fn isolated_vertices_uniform() {
        let g = Graph::with_vertices(4);
        let pr = pagerank_weighted(&g, &[]).unwrap();
        close(&pr, &[0.25, 0.25, 0.25, 0.25], 1e-9);
    }

    #[test]
    fn unit_weights_match_unweighted_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let pw = pagerank_weighted(&g, &[1.0; 3]).unwrap();
        let pu = crate::pagerank(&g).unwrap();
        for (a, b) in pw.iter().zip(pu.iter()) {
            assert!((a - b).abs() < 1e-9, "{a} vs {b}");
        }
    }

    #[test]
    fn unit_weights_match_unweighted_directed_4cycle() {
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let pw = pagerank_weighted(&g, &[1.0; 4]).unwrap();
        let pu = crate::pagerank(&g).unwrap();
        for (a, b) in pw.iter().zip(pu.iter()) {
            assert!((a - b).abs() < 1e-9, "{a} vs {b}");
        }
    }

    #[test]
    fn pagerank_weighted_sums_to_one() {
        // K4 minus an edge with non-uniform weights.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let pr = pagerank_weighted(&g, &[1.0, 2.0, 0.5, 1.5, 1.0]).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum {total} != 1.0");
    }

    #[test]
    fn heavy_edge_concentrates_pagerank() {
        // Directed 0 → 1 with weight 100, plus 0 → 2 with weight 0.01.
        // Almost all of 0's flow goes to 1, so 1 should have higher
        // PageRank than 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let pr = pagerank_weighted(&g, &[100.0, 0.01]).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
        assert!(
            pr[1] > pr[2],
            "heavy-edge target {} should beat light-edge {}",
            pr[1],
            pr[2]
        );
    }

    #[test]
    fn star_centre_has_higher_pagerank_than_leaves() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let pr = pagerank_weighted(&g, &[1.0; 3]).unwrap();
        for &leaf in &pr[1..4] {
            assert!(pr[0] > leaf, "centre {} not > leaf {}", pr[0], leaf);
        }
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(pagerank_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(pagerank_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(pagerank_weighted(&g, &[f64::NAN]).is_err());
    }
}
