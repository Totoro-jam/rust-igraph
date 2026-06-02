//! Eigenvalues of a graph's adjacency matrix.
//!
//! Convenience wrapper around [`super::eigen_matrix_symmetric`] that
//! constructs the matrix-vector product `y = A·x` from the graph's
//! adjacency structure.
//!
//! ```
//! use rust_igraph::{Graph, eigen_adjacency, EigenWhich};
//!
//! // K_3 (complete graph on 3 vertices): adjacency eigenvalues are 2, -1, -1
//! let mut g = Graph::with_vertices(3);
//! g.add_edge(0, 1).unwrap();
//! g.add_edge(0, 2).unwrap();
//! g.add_edge(1, 2).unwrap();
//!
//! let result = eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).unwrap();
//! assert!((result.eigenvalues[0] - 2.0).abs() < 1e-6);
//! ```

use super::symmetric::{EigenDecomposition, EigenWhich, eigen_matrix_symmetric};
use crate::core::error::{IgraphError, IgraphResult};
use crate::core::graph::Graph;

/// Compute selected eigenvalues of a graph's adjacency matrix.
///
/// For undirected graphs, the adjacency matrix is real symmetric and the
/// eigenvalues are real. For directed graphs, the matrix-vector product
/// uses the **symmetrized** adjacency `(A + A^T) / 2` so that the Lanczos
/// solver applies; the eigenvalues correspond to this symmetrized form.
///
/// # Arguments
///
/// * `graph` — the graph whose adjacency spectrum to compute
/// * `nev` — number of eigenvalues to compute (clamped to `vcount`)
/// * `which` — which part of the spectrum to target
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if the graph has zero vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigen_adjacency, EigenWhich};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
///
/// let result = eigen_adjacency(&g, 2, EigenWhich::LargestAlgebraic).unwrap();
/// assert_eq!(result.eigenvalues.len(), 2);
/// ```
pub fn eigen_adjacency(
    graph: &Graph,
    nev: usize,
    which: EigenWhich,
) -> IgraphResult<EigenDecomposition> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "eigen_adjacency: graph has zero vertices".into(),
        ));
    }

    let directed = graph.is_directed();
    let ecount = graph.ecount();

    // Pre-build the edge list so the closure is allocation-free.
    let ecount_u32 = u32::try_from(ecount).unwrap_or(u32::MAX);
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(ecount);
    for eid in 0..ecount_u32 {
        if let Ok((u, v)) = graph.edge(eid) {
            edges.push((u as usize, v as usize));
        }
    }

    let matvec = |x: &[f64], y: &mut [f64]| {
        for val in y.iter_mut() {
            *val = 0.0;
        }
        for &(u, v) in &edges {
            if directed {
                // Symmetrized: (A + A^T) / 2
                y[u] += x[v];
                y[v] += x[u];
            } else {
                // Undirected: each edge (u,v) contributes A[u][v] and A[v][u]
                y[u] += x[v];
                y[v] += x[u];
            }
        }
        if directed {
            for val in y.iter_mut() {
                *val *= 0.5;
            }
        }
    };

    eigen_matrix_symmetric(n, matvec, nev, which)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::graph::Graph;

    #[test]
    fn k3_largest_eigenvalue() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();

        let result = eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).unwrap();
        assert_eq!(result.eigenvalues.len(), 1);
        assert!(
            (result.eigenvalues[0] - 2.0).abs() < 1e-4,
            "K3 largest eigenvalue should be 2.0, got {}",
            result.eigenvalues[0]
        );
    }

    #[test]
    fn path_graph_eigenvalues() {
        // Path P_3: vertices 0-1-2, eigenvalues are sqrt(2), 0, -sqrt(2)
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let result = eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).unwrap();
        let expected = std::f64::consts::SQRT_2;
        assert!(
            (result.eigenvalues[0] - expected).abs() < 1e-4,
            "P3 largest eigenvalue should be sqrt(2)={expected}, got {}",
            result.eigenvalues[0]
        );
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(5);
        let result = eigen_adjacency(&g, 3, EigenWhich::LargestAlgebraic).unwrap();
        for &ev in &result.eigenvalues {
            assert!(
                ev.abs() < 1e-6,
                "isolated graph eigenvalue should be 0.0, got {ev}"
            );
        }
    }

    #[test]
    fn empty_graph_error() {
        let g = Graph::with_vertices(0);
        assert!(eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).is_err());
    }

    #[test]
    fn star_graph_eigenvalues() {
        // Star S_4 (center 0, leaves 1,2,3): eigenvalues are sqrt(3), 0, 0, -sqrt(3)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();

        let result = eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).unwrap();
        let expected = 3.0_f64.sqrt();
        assert!(
            (result.eigenvalues[0] - expected).abs() < 1e-4,
            "S4 largest eigenvalue should be sqrt(3)={expected}, got {}",
            result.eigenvalues[0]
        );
    }

    #[test]
    fn cycle_graph_eigenvalues() {
        // C_4: eigenvalues are 2, 0, 0, -2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        let result = eigen_adjacency(&g, 1, EigenWhich::LargestAlgebraic).unwrap();
        assert!(
            (result.eigenvalues[0] - 2.0).abs() < 1e-4,
            "C4 largest eigenvalue should be 2.0, got {}",
            result.eigenvalues[0]
        );
    }
}
