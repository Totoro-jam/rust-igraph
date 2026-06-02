//! Katz centrality — attenuated walk-count centrality.
//!
//! Katz centrality for node *i* is defined as:
//!   `c_i = alpha * sum_j A_ji * c_j + beta`
//!
//! where `alpha` is an attenuation factor (must be less than `1/lambda_max`
//! for convergence) and `beta` is a baseline score for each node.
//!
//! Reference: Katz, L. "A New Status Index Derived from Sociometric Analysis",
//! Psychometrika 18(1), 39–43, 1953.

use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::Graph;

/// Compute Katz centrality for all vertices using power iteration.
///
/// `alpha` is the attenuation factor — must satisfy `0 < alpha < 1/lambda_max`
/// where `lambda_max` is the largest eigenvalue of the adjacency matrix.
/// A safe default for most graphs is `0.1`.
///
/// `beta` is the baseline centrality for each node (typically `1.0`).
///
/// `max_iter` caps the number of iterations (default: 1000 if `None`).
///
/// `tol` is the convergence tolerance on the L2-norm change (default: 1e-6
/// if `None`).
///
/// For directed graphs, centrality counts walks *arriving* at each node
/// (i.e. uses in-neighbors).
///
/// # Errors
///
/// Returns an error if `alpha <= 0`, `max_iter` is 0, or if the iteration
/// does not converge within `max_iter` steps.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, katz_centrality};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, None).unwrap();
/// let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
/// // Symmetric graph → all centralities equal
/// let diff = c[0] - c[2];
/// assert!(diff.abs() < 1e-6);
/// ```
pub fn katz_centrality(
    graph: &Graph,
    alpha: f64,
    beta: f64,
    max_iter: Option<u32>,
    tol: Option<f64>,
) -> IgraphResult<Vec<f64>> {
    if alpha <= 0.0 || alpha.is_nan() {
        return Err(IgraphError::InvalidArgument(
            "alpha must be positive and finite".into(),
        ));
    }
    let max_iter = max_iter.unwrap_or(1000);
    if max_iter == 0 {
        return Err(IgraphError::InvalidArgument(
            "max_iter must be positive".into(),
        ));
    }
    let tol = tol.unwrap_or(1e-6);
    let n = graph.vcount() as usize;

    if n == 0 {
        return Ok(Vec::new());
    }

    // Precompute adjacency for efficiency: for each vertex, store its
    // in-neighbors (for directed) or all neighbors (for undirected).
    let mut in_adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (u, v) in graph.edges() {
        // Edge u → v: v receives from u
        in_adj[v as usize].push(u);
        if !graph.is_directed() {
            // Undirected: u also receives from v
            in_adj[u as usize].push(v);
        }
    }

    let mut c = vec![0.0_f64; n];
    let mut c_new = vec![0.0_f64; n];

    for _ in 0..max_iter {
        for i in 0..n {
            let mut sum = 0.0_f64;
            for &u in &in_adj[i] {
                sum += c[u as usize];
            }
            c_new[i] = alpha * sum + beta;
        }

        // Check convergence (L2 norm of difference)
        let mut norm_sq = 0.0_f64;
        for i in 0..n {
            let diff = c_new[i] - c[i];
            norm_sq += diff * diff;
        }
        std::mem::swap(&mut c, &mut c_new);

        if norm_sq.sqrt() < tol {
            return Ok(c);
        }
    }

    Err(IgraphError::DidNotConverge {
        iters: max_iter as usize,
        residual: {
            let mut norm_sq = 0.0_f64;
            for i in 0..n {
                let diff = c_new[i] - c[i];
                norm_sq += diff * diff;
            }
            norm_sq.sqrt()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
        assert!(c.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
        assert!((c[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn symmetric_graph_equal_centrality() {
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, None).unwrap();
        let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
        for i in 1..4 {
            assert!((c[i] - c[0]).abs() < 1e-6);
        }
    }

    #[test]
    fn star_center_higher() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, None).unwrap();
        let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
        // Center (vertex 0) should have higher centrality
        assert!(c[0] > c[1]);
    }

    #[test]
    fn directed_asymmetric() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (1, 2)], true, None).unwrap();
        let c = katz_centrality(&g, 0.1, 1.0, None, None).unwrap();
        // Vertex 2 has most in-edges
        assert!(c[2] > c[0]);
    }

    #[test]
    fn invalid_alpha() {
        let g = Graph::with_vertices(2);
        assert!(katz_centrality(&g, 0.0, 1.0, None, None).is_err());
        assert!(katz_centrality(&g, -0.1, 1.0, None, None).is_err());
    }

    #[test]
    fn isolated_vertices_get_beta() {
        let g = Graph::with_vertices(3);
        let c = katz_centrality(&g, 0.1, 2.0, None, None).unwrap();
        for &v in &c {
            assert!((v - 2.0).abs() < 1e-10);
        }
    }
}
