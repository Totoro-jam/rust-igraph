//! Personalized `PageRank` (ALGO-PR-055).
//!
//! Counterpart of `igraph_personalized_pagerank()` from
//! `references/igraph/src/centrality/pagerank.c`.
//!
//! Unlike standard `PageRank` which teleports uniformly to 1/N,
//! personalized `PageRank` uses a custom reset distribution vector.
//! This enables topic-sensitive ranking, local community detection,
//! and link prediction.

use crate::core::{Graph, IgraphError, IgraphResult};

const DEFAULT_DAMPING: f64 = 0.85;
const DEFAULT_EPS: f64 = 1e-10;
const DEFAULT_MAX_ITER: usize = 1000;

/// Personalized `PageRank` scores via power iteration.
///
/// `reset`: the personalization vector. Must have length `vcount()`
/// and contain non-negative values that sum to a positive number
/// (they are internally normalized to sum to 1). Vertices with
/// higher reset weight attract more rank during teleportation.
///
/// `damping`: the damping factor (probability of following edges
/// vs. teleporting). Must be in (0, 1). Use `0.85` for standard
/// behavior.
///
/// Returns a `Vec<f64>` summing approximately to 1.
///
/// Counterpart of
/// `igraph_personalized_pagerank(_, IGRAPH_PAGERANK_ALGO_POWER, _,
///  _, vss_all(), directed, damping, reset, weights=NULL, _)`
///
/// # Errors
///
/// - `InvalidArgument` if `reset` length does not match `vcount()`.
/// - `InvalidArgument` if `reset` contains negative values.
/// - `InvalidArgument` if `reset` sums to zero.
/// - `InvalidArgument` if `damping` is not in (0, 1).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, personalized_pagerank};
///
/// // 4-cycle: bias teleportation toward vertex 1.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// // Reset only on vertex 1 — it gets the highest PR.
/// let reset = vec![0.0, 1.0, 0.0, 0.0];
/// let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
/// assert!(pr[1] > pr[0]);
/// assert!(pr[1] > pr[2]);
/// assert!(pr[1] > pr[3]);
///
/// // Uniform reset: all vertices equal on a symmetric graph.
/// let uniform = vec![0.25, 0.25, 0.25, 0.25];
/// let pr_uniform = personalized_pagerank(&g, &uniform, 0.85).unwrap();
/// let sum: f64 = pr_uniform.iter().sum();
/// assert!((sum - 1.0).abs() < 1e-9);
/// ```
pub fn personalized_pagerank(graph: &Graph, reset: &[f64], damping: f64) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if n == 0 {
        if reset.is_empty() {
            return Ok(Vec::new());
        }
        return Err(IgraphError::InvalidArgument(
            "personalized_pagerank: reset vector non-empty but graph has no vertices".into(),
        ));
    }

    if reset.len() != n_us {
        return Err(IgraphError::InvalidArgument(format!(
            "personalized_pagerank: reset length ({}) does not match vcount ({n})",
            reset.len()
        )));
    }

    if damping <= 0.0 || damping >= 1.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "personalized_pagerank: damping ({damping}) must be in (0, 1)"
        )));
    }

    let reset_sum: f64 = reset.iter().sum();
    if reset_sum <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "personalized_pagerank: reset vector must sum to a positive value".into(),
        ));
    }
    for (i, &val) in reset.iter().enumerate() {
        if val < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "personalized_pagerank: negative reset value ({val}) at index {i}"
            )));
        }
    }

    if n == 1 {
        return Ok(vec![1.0]);
    }

    // Normalize the reset vector.
    let inv_sum = 1.0 / reset_sum;
    let norm_reset: Vec<f64> = reset.iter().map(|&v| v * inv_sum).collect();

    let directed = graph.is_directed();

    // Out-degree per vertex.
    let mut out_deg = vec![0u64; n_us];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        out_deg[v as usize] = nbrs.len() as u64;
    }

    // Build in-adjacency lists.
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut in_adj: Vec<Vec<u32>> = vec![Vec::new(); n_us];

    if directed {
        for e in 0..m {
            let (u, v) = graph.edge(e)?;
            in_adj[v as usize].push(u);
        }
    } else {
        for e in 0..m {
            let (u, v) = graph.edge(e)?;
            if u == v {
                in_adj[v as usize].push(u);
                in_adj[v as usize].push(u);
            } else {
                in_adj[u as usize].push(v);
                in_adj[v as usize].push(u);
            }
        }
    }

    // Initial distribution from the reset vector.
    let mut pr = norm_reset.clone();
    let mut pr_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        // Dangling vertex rank sum.
        let mut dangling_sum: f64 = 0.0;
        for v in 0..n_us {
            if out_deg[v] == 0 {
                dangling_sum += pr[v];
            }
        }

        for v in 0..n_us {
            let mut incoming: f64 = 0.0;
            for &u in &in_adj[v] {
                #[allow(clippy::cast_precision_loss)]
                let denom = out_deg[u as usize] as f64;
                if denom > 0.0 {
                    incoming += pr[u as usize] / denom;
                }
            }
            // Personalized teleport: use norm_reset[v] instead of 1/N.
            // Dangling nodes also distribute according to norm_reset.
            pr_new[v] = (1.0 - damping) * norm_reset[v]
                + damping * (incoming + dangling_sum * norm_reset[v]);
        }

        // Convergence check: L1 norm.
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

/// Personalized `PageRank` with default damping factor (0.85).
///
/// Convenience wrapper around [`personalized_pagerank`] with
/// `damping = 0.85`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, personalized_pagerank_default};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let reset = vec![1.0, 0.0, 0.0]; // bias toward vertex 0
/// let pr = personalized_pagerank_default(&g, &reset).unwrap();
/// assert!(pr[0] > pr[1]);
/// assert!(pr[0] > pr[2]);
/// ```
pub fn personalized_pagerank_default(graph: &Graph, reset: &[f64]) -> IgraphResult<Vec<f64>> {
    personalized_pagerank(graph, reset, DEFAULT_DAMPING)
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
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let pr = personalized_pagerank(&g, &[], 0.85).unwrap();
        assert!(pr.is_empty());
    }

    #[test]
    fn singleton() {
        let g = Graph::with_vertices(1);
        let pr = personalized_pagerank(&g, &[1.0], 0.85).unwrap();
        assert_eq!(pr, vec![1.0]);
    }

    #[test]
    fn uniform_reset_matches_standard_pagerank() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let standard = crate::pagerank(&g).unwrap();
        let uniform = vec![0.25; 4];
        let personalized = personalized_pagerank(&g, &uniform, 0.85).unwrap();
        close(&personalized, &standard, 1e-9);
    }

    #[test]
    fn biased_reset_changes_ranking() {
        // Star: center 0 connected to leaves 1, 2, 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();

        // Uniform reset: center has highest PR (absorbs from all leaves).
        let uniform = vec![0.25; 4];
        let pr_u = personalized_pagerank(&g, &uniform, 0.85).unwrap();
        assert!(pr_u[0] > pr_u[1]);

        // Reset only on leaf 1: leaf 1 gets more teleport, but center
        // still collects from all leaves. Leaf 1 should still be above
        // leaves 2 and 3 (which get zero teleport).
        let biased = vec![0.0, 1.0, 0.0, 0.0];
        let pr_b = personalized_pagerank(&g, &biased, 0.85).unwrap();
        assert!(pr_b[1] > pr_b[2], "leaf 1 > leaf 2");
        assert!(pr_b[1] > pr_b[3], "leaf 1 > leaf 3");
        // Center still dominates in star topology.
        assert!(pr_b[0] > pr_b[1], "center > biased leaf in star");
    }

    #[test]
    fn sums_to_one() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();

        let reset = vec![0.5, 0.2, 0.1, 0.1, 0.1];
        let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum={total}");
    }

    #[test]
    fn directed_biased() {
        // 0→1→2, reset on 0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let reset = vec![1.0, 0.0, 0.0];
        let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
        // Flow: 0 sends to 1, 1 sends to 2. 2 is dangling → distributes back to 0.
        // 0 also gets all teleport. So pr[0] > pr[1] > pr[2] is NOT necessarily true.
        // Actually: 2 is dangling, distributes reset-weighted (all to 0).
        // Teleport goes to 0 only.
        // So 0 gets lots of rank; 1 gets from 0; 2 gets from 1.
        // pr[0] > pr[1] > pr[2].
        assert!(pr[0] > pr[1], "pr[0]={} > pr[1]={}", pr[0], pr[1]);
        assert!(pr[1] > pr[2], "pr[1]={} > pr[2]={}", pr[1], pr[2]);
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn dangling_vertices_redistribute_to_reset() {
        // 0→1, both vertices. Vertex 1 is dangling.
        // Reset = [0.3, 0.7]. Dangling redistribution goes to reset weights.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let reset = vec![0.3, 0.7];
        let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn different_damping_factors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let reset = vec![1.0, 0.0, 0.0];

        // High damping → more link-following → more uniform.
        let pr_high = personalized_pagerank(&g, &reset, 0.99).unwrap();
        // Low damping → more teleporting → more biased toward reset.
        let pr_low = personalized_pagerank(&g, &reset, 0.5).unwrap();
        // With low damping, vertex 0 should have more weight.
        assert!(pr_low[0] > pr_high[0]);
    }

    #[test]
    fn error_on_wrong_reset_length() {
        let g = Graph::with_vertices(3);
        assert!(personalized_pagerank(&g, &[1.0, 1.0], 0.85).is_err());
    }

    #[test]
    fn error_on_negative_reset() {
        let g = Graph::with_vertices(3);
        assert!(personalized_pagerank(&g, &[1.0, -0.5, 0.5], 0.85).is_err());
    }

    #[test]
    fn error_on_zero_sum_reset() {
        let g = Graph::with_vertices(3);
        assert!(personalized_pagerank(&g, &[0.0, 0.0, 0.0], 0.85).is_err());
    }

    #[test]
    fn error_on_invalid_damping() {
        let g = Graph::with_vertices(2);
        assert!(personalized_pagerank(&g, &[0.5, 0.5], 0.0).is_err());
        assert!(personalized_pagerank(&g, &[0.5, 0.5], 1.0).is_err());
        assert!(personalized_pagerank(&g, &[0.5, 0.5], -0.1).is_err());
        assert!(personalized_pagerank(&g, &[0.5, 0.5], 1.5).is_err());
    }

    #[test]
    fn unnormalized_reset_works() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Reset = [10, 0, 0] should give same result as [1, 0, 0].
        let pr1 = personalized_pagerank(&g, &[10.0, 0.0, 0.0], 0.85).unwrap();
        let pr2 = personalized_pagerank(&g, &[1.0, 0.0, 0.0], 0.85).unwrap();
        close(&pr1, &pr2, 1e-9);
    }

    #[test]
    fn default_wrapper() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let reset = vec![1.0, 0.0, 0.0];
        let pr1 = personalized_pagerank(&g, &reset, 0.85).unwrap();
        let pr2 = personalized_pagerank_default(&g, &reset).unwrap();
        close(&pr1, &pr2, 1e-15);
    }

    #[test]
    fn isolated_vertices_with_biased_reset() {
        // 4 isolated vertices, reset only on vertex 2.
        // All dangling → all rank flows through reset.
        let g = Graph::with_vertices(4);
        let reset = vec![0.0, 0.0, 1.0, 0.0];
        let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
        // All rank ends up on vertex 2.
        assert!((pr[2] - 1.0).abs() < 1e-9);
        assert!(pr[0].abs() < 1e-9);
        assert!(pr[1].abs() < 1e-9);
        assert!(pr[3].abs() < 1e-9);
    }

    #[test]
    fn oracle_5_cycle_biased() {
        // Verified against python-igraph personalized_pagerank.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let reset = vec![0.5, 0.2, 0.1, 0.1, 0.1];
        let pr = personalized_pagerank(&g, &reset, 0.85).unwrap();
        let expected = [
            0.246_408_839_8,
            0.210_246_107_5,
            0.177_699_648_4,
            0.172_576_594_7,
            0.193_068_809_6,
        ];
        close(&pr, &expected, 1e-6);
    }
}
