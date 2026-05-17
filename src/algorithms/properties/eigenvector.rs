//! Eigenvector centrality (ALGO-PR-012).
//!
//! Counterpart of `igraph_eigenvector_centrality()` from
//! `references/igraph/src/centrality/eigenvector.c`. The eigenvector
//! centrality of `v` is its component in the dominant eigenvector of
//! the adjacency matrix (largest eigenvalue, real and positive by
//! Perron-Frobenius for connected non-bipartite graphs).
//!
//! Implemented via shifted power iteration on `(A + I)`:
//! `x_new[v] = x_old[v] + Σ_{u ~ v} x_old[u]`, then normalize so
//! `max(x) == 1`. The shift doesn't change eigenvectors but breaks the
//! `±λ` symmetry of bipartite adjacency matrices, ensuring convergence
//! to the *positive* dominant eigenvector. Iterate until L1 change <
//! `1e-10` or 1000 iterations. Matches `python-igraph 0.11`'s
//! normalization convention (max = 1).
//!
//! Phase-1 minimal slice: undirected, unweighted. Directed mode (with
//! either `IGRAPH_OUT` or `IGRAPH_IN` semantics) and weighted variants
//! ship later.

use crate::core::{Graph, IgraphError, IgraphResult};

// Tighter than the L1-convergence default we use elsewhere because
// rustdoc/conformance compare bit-precise vs python-igraph's ARPACK.
const DEFAULT_EPS: f64 = 1e-14;
const DEFAULT_MAX_ITER: usize = 5000;

/// Eigenvector centrality scores normalized so `max == 1`.
///
/// Returns `Vec<f64>` of length `vcount`. Empty graph returns empty.
/// Directed graphs return [`crate::IgraphError::Unsupported`] for now —
/// directed eigenvector centrality requires careful eigenvector choice
/// (in-edges vs out-edges) and ships later.
///
/// Counterpart of
/// `igraph_eigenvector_centrality(_, _, NULL_eval, /*directed=*/false,
/// /*scale=*/true, NULL_weights, NULL_options)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigenvector_centrality};
///
/// // Triangle: every vertex has identical centrality 1.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let ec = eigenvector_centrality(&g).unwrap();
/// assert!((ec[0] - 1.0).abs() < 1e-9);
/// assert!((ec[1] - 1.0).abs() < 1e-9);
/// assert!((ec[2] - 1.0).abs() < 1e-9);
/// ```
pub fn eigenvector_centrality(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed eigenvector_centrality is PR-012b; not yet ported",
        ));
    }
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        // Single vertex: trivial dominant-eigenvector convention is 1.
        return Ok(vec![1.0]);
    }

    // Pre-cache neighbour lists.
    let mut adj: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        adj.push(graph.neighbors(v)?);
    }

    // Initial uniform distribution. Some entry must be nonzero or the
    // iteration is stuck at zero — uniform 1.0 works.
    let mut x = vec![1.0_f64; n_us];
    let mut x_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        // x_new = (A + I) * x   to break ±λ bipartite symmetry.
        for v in 0..n_us {
            let mut sum = x[v];
            for &u in &adj[v] {
                sum += x[u as usize];
            }
            x_new[v] = sum;
        }

        // Normalize so max(|x_new|) = 1. If max is zero (graph has
        // isolated vertices and we started with all-zero somehow),
        // skip normalization to avoid /0.
        let max = x_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if max > 0.0 {
            for slot in &mut x_new {
                *slot /= max;
            }
        }

        // L1 convergence check.
        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (x_new[v] - x[v]).abs();
        }
        std::mem::swap(&mut x, &mut x_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    Ok(x)
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
        assert!(eigenvector_centrality(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(eigenvector_centrality(&g).unwrap(), vec![1.0]);
    }

    #[test]
    fn isolated_vertices_yield_uniform_one() {
        // No edges → (A + I) = I → eigenvector is uniform; after max-1
        // normalization, [1, 1, ..., 1]. Matches python-igraph 0.11.
        let g = Graph::with_vertices(3);
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn triangle_uniform_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn star_4_centre_one_leaves_inverse_sqrt_three() {
        // 4-star: dominant eigenvalue is sqrt(3); ratio centre:leaf = sqrt(3):1.
        // After max-1 normalization: centre = 1, leaves = 1/sqrt(3).
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let ec = eigenvector_centrality(&g).unwrap();
        let inv_sqrt3 = 1.0 / 3f64.sqrt();
        close(&ec, &[1.0, inv_sqrt3, inv_sqrt3, inv_sqrt3], 1e-9);
    }

    #[test]
    fn k4_uniform_one() {
        // K_n: eigenvector is uniform (every vertex has the same role).
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn directed_graph_returns_unsupported() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(eigenvector_centrality(&g).is_err());
    }
}
