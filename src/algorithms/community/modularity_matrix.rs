//! Modularity matrix (ALGO-CO-007).
//!
//! Counterpart of `igraph_modularity_matrix()` from
//! `references/igraph/src/community/modularity.c:323`.
//!
//! Produces the dense modularity matrix **B** where:
//! - Undirected: `B_ij = A_ij − γ k_i k_j / (2m)`
//! - Directed:   `B_ij = A_ij − γ k^out_i k^in_j / m`
//!
//! Self-loops in undirected graphs contribute 2 to `A_ii` (matches
//! upstream). When weights are given, `A` becomes the weighted
//! adjacency and `k` becomes the strength.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the modularity matrix **B** of a graph.
///
/// Returns a dense `n×n` matrix (row-major `Vec<Vec<f64>>`).
///
/// - `weights`: optional edge weights (length must equal `ecount()`).
/// - `resolution`: the γ parameter (≥ 0; default 1.0).
/// - `directed`: for directed graphs, whether to use directed formula.
///   Ignored for undirected graphs.
///
/// Returns an empty matrix for graphs with no vertices. Returns an error
/// when the total edge weight is zero (the matrix is undefined in that
/// case, matching upstream's documentation).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modularity_matrix};
///
/// // Triangle (K3): A is all-1 off-diagonal (undirected stores both
/// // directions). Each vertex has degree 2, m = 3, so
/// // B_ij = A_ij - 2*2/(2*3) = 1 - 2/3 = 1/3 for i≠j,
/// // B_ii = 0 - 4/6 = -2/3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let b = modularity_matrix(&g, None, 1.0, true).unwrap();
/// assert!((b[0][0] - (-2.0/3.0)).abs() < 1e-10);
/// assert!((b[0][1] - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn modularity_matrix(
    graph: &Graph,
    weights: Option<&[f64]>,
    resolution: f64,
    directed: bool,
) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if !resolution.is_finite() || resolution < 0.0 {
        return Err(IgraphError::InvalidArgument(
            "resolution parameter must be non-negative and finite".to_string(),
        ));
    }
    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "modularity_matrix: weights length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    if n == 0 {
        return Ok(Vec::new());
    }

    let n_usize = n as usize;
    let directed = directed && graph.is_directed();

    // Build adjacency matrix.
    let mut matrix = vec![vec![0.0_f64; n_usize]; n_usize];

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;

    for eid in 0..m_u32 {
        let (from, to) = graph.edge(eid)?;
        let f = from as usize;
        let t = to as usize;
        let w = weights.map_or(1.0, |ws| ws[eid as usize]);
        matrix[f][t] += w;
        if !directed {
            matrix[t][f] += w;
        }
    }

    // Compute degree/strength vectors.
    #[allow(clippy::cast_precision_loss)]
    let sw: f64 = weights.map_or(ecount as f64, |ws| ws.iter().sum());
    if sw == 0.0 {
        return Err(IgraphError::InvalidArgument(
            "modularity_matrix: total edge weight is zero; matrix is undefined".to_string(),
        ));
    }

    if directed {
        // Directed: B_ij = A_ij - γ * k_out_i * k_in_j / m
        let mut k_out = vec![0.0_f64; n_usize];
        let mut k_in = vec![0.0_f64; n_usize];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            let w = weights.map_or(1.0, |ws| ws[eid as usize]);
            k_out[from as usize] += w;
            k_in[to as usize] += w;
        }

        let scaling = resolution / sw;
        for (j, &k_in_j) in k_in.iter().enumerate().take(n_usize) {
            for i in 0..n_usize {
                matrix[i][j] -= k_out[i] * k_in_j * scaling;
            }
        }
    } else {
        // Undirected: B_ij = A_ij - γ * k_i * k_j / (2m)
        let mut deg = vec![0.0_f64; n_usize];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            let w = weights.map_or(1.0, |ws| ws[eid as usize]);
            deg[from as usize] += w;
            deg[to as usize] += w;
        }

        let scaling = resolution / (2.0 * sw);
        for j in 0..n_usize {
            for i in 0..n_usize {
                matrix[i][j] -= deg[i] * deg[j] * scaling;
            }
        }
    }

    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let m = modularity_matrix(&g, None, 1.0, false).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn single_vertex_no_edges_error() {
        let g = Graph::with_vertices(1);
        // No edges → sw = 0, undefined.
        assert!(modularity_matrix(&g, None, 1.0, false).is_err());
    }

    #[test]
    fn triangle_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let b = modularity_matrix(&g, None, 1.0, false).unwrap();
        // m = 3, k_i = 2 for all. B_ii = 0 - 4/6 = -2/3. B_ij = 1 - 4/6 = 1/3.
        for (i, row) in b.iter().enumerate() {
            assert!(close(row[i], -2.0 / 3.0));
            for (j, &val) in row.iter().enumerate() {
                if i != j {
                    assert!(close(val, 1.0 / 3.0));
                }
            }
        }
    }

    #[test]
    fn path_3_undirected() {
        // 0-1-2: deg = [1, 2, 1], m = 2
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let b = modularity_matrix(&g, None, 1.0, false).unwrap();
        // B[0][0] = 0 - 1*1/4 = -0.25
        // B[0][1] = 1 - 1*2/4 = 0.5
        // B[0][2] = 0 - 1*1/4 = -0.25
        // B[1][1] = 0 - 2*2/4 = -1.0
        // B[1][2] = 1 - 2*1/4 = 0.5
        // B[2][2] = 0 - 1*1/4 = -0.25
        assert!(close(b[0][0], -0.25));
        assert!(close(b[0][1], 0.5));
        assert!(close(b[0][2], -0.25));
        assert!(close(b[1][0], 0.5));
        assert!(close(b[1][1], -1.0));
        assert!(close(b[1][2], 0.5));
        assert!(close(b[2][0], -0.25));
        assert!(close(b[2][1], 0.5));
        assert!(close(b[2][2], -0.25));
    }

    #[test]
    fn directed_simple() {
        // 0→1→2, directed. m = 2.
        // A = [[0,1,0],[0,0,1],[0,0,0]]
        // k_out = [1, 1, 0], k_in = [0, 1, 1]
        // B_ij = A_ij - k_out_i * k_in_j / 2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let b = modularity_matrix(&g, None, 1.0, true).unwrap();
        // B[0][0] = 0 - 1*0/2 = 0
        // B[0][1] = 1 - 1*1/2 = 0.5
        // B[0][2] = 0 - 1*1/2 = -0.5
        // B[1][0] = 0 - 1*0/2 = 0
        // B[1][1] = 0 - 1*1/2 = -0.5
        // B[1][2] = 1 - 1*1/2 = 0.5
        // B[2][0] = 0 - 0*0/2 = 0
        // B[2][1] = 0 - 0*1/2 = 0
        // B[2][2] = 0 - 0*1/2 = 0
        assert!(close(b[0][0], 0.0));
        assert!(close(b[0][1], 0.5));
        assert!(close(b[0][2], -0.5));
        assert!(close(b[1][0], 0.0));
        assert!(close(b[1][1], -0.5));
        assert!(close(b[1][2], 0.5));
        assert!(close(b[2][0], 0.0));
        assert!(close(b[2][1], 0.0));
        assert!(close(b[2][2], 0.0));
    }

    #[test]
    fn directed_flag_ignored_for_undirected_graph() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let b1 = modularity_matrix(&g, None, 1.0, true).unwrap();
        let b2 = modularity_matrix(&g, None, 1.0, false).unwrap();
        for (row1, row2) in b1.iter().zip(b2.iter()) {
            for (&v1, &v2) in row1.iter().zip(row2.iter()) {
                assert!(close(v1, v2));
            }
        }
    }

    #[test]
    fn weighted_undirected() {
        // 0-1 (w=3), 1-2 (w=1). sw = 4.
        // deg = [3, 4, 1]
        // B[0][0] = 0 - 3*3/8 = -9/8
        // B[0][1] = 3 - 3*4/8 = 3 - 12/8 = 12/8
        // B[0][2] = 0 - 3*1/8 = -3/8
        // B[1][1] = 0 - 4*4/8 = -16/8 = -2
        // B[1][2] = 1 - 4*1/8 = 1 - 4/8 = 4/8
        // B[2][2] = 0 - 1*1/8 = -1/8
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![3.0, 1.0];
        let b = modularity_matrix(&g, Some(&weights), 1.0, false).unwrap();
        assert!(close(b[0][0], -9.0 / 8.0));
        assert!(close(b[0][1], 3.0 - 12.0 / 8.0));
        assert!(close(b[0][2], -3.0 / 8.0));
        assert!(close(b[1][0], 3.0 - 12.0 / 8.0));
        assert!(close(b[1][1], -16.0 / 8.0));
        assert!(close(b[1][2], 1.0 - 4.0 / 8.0));
        assert!(close(b[2][0], -3.0 / 8.0));
        assert!(close(b[2][1], 1.0 - 4.0 / 8.0));
        assert!(close(b[2][2], -1.0 / 8.0));
    }

    #[test]
    fn self_loop_undirected() {
        // 0-0 (self-loop), 0-1. m = 2, sw = 2.
        // Adjacency: A[0][0] = 2 (self-loop counts twice), A[0][1] = A[1][0] = 1.
        // deg = [0] endpoint contributes: edge(0,0) → deg[0] += 1+1 = 2 from self-loop,
        //        edge(0,1) → deg[0] += 1, deg[1] += 1. So deg = [3, 1].
        // B[0][0] = 2 - 3*3/4 = 2 - 9/4 = -1/4
        // B[0][1] = 1 - 3*1/4 = 1/4
        // B[1][0] = 1 - 1*3/4 = 1/4
        // B[1][1] = 0 - 1*1/4 = -1/4
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let b = modularity_matrix(&g, None, 1.0, false).unwrap();
        assert!(close(b[0][0], -1.0 / 4.0));
        assert!(close(b[0][1], 1.0 / 4.0));
        assert!(close(b[1][0], 1.0 / 4.0));
        assert!(close(b[1][1], -1.0 / 4.0));
    }

    #[test]
    fn resolution_parameter() {
        // With γ = 0, B = A (the adjacency matrix).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let b = modularity_matrix(&g, None, 0.0, false).unwrap();
        // Undirected triangle adjacency: 0 off-diag, 1 on connections.
        assert!(close(b[0][0], 0.0));
        assert!(close(b[0][1], 1.0));
        assert!(close(b[0][2], 1.0));
        assert!(close(b[1][2], 1.0));
    }

    #[test]
    fn negative_resolution_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(modularity_matrix(&g, None, -1.0, false).is_err());
    }

    #[test]
    fn weights_length_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(modularity_matrix(&g, Some(&[1.0, 2.0]), 1.0, false).is_err());
    }

    #[test]
    fn row_sums_zero_for_undirected_gamma_1() {
        // For γ=1, the modularity matrix of a connected undirected graph
        // has row sums = 0 (since Σ_j A_ij = k_i and Σ_j k_j = 2m).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let b = modularity_matrix(&g, None, 1.0, false).unwrap();
        for row in &b {
            let row_sum: f64 = row.iter().sum();
            assert!(close(row_sum, 0.0));
        }
    }
}
