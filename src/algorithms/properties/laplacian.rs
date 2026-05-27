//! Graph Laplacian matrix (ALGO-PR-041).
//!
//! Returns the Laplacian matrix L = D - A in dense form, with optional
//! normalization (unnormalized, symmetric, left-stochastic, right-stochastic).
//!
//! Counterpart of `igraph_get_laplacian` from
//! `references/igraph/src/properties/spectral.c`.

use crate::algorithms::properties::degree::DegreeMode;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Laplacian normalization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaplacianNormalization {
    /// L = D - A (unnormalized).
    Unnormalized,
    /// Symmetric normalization: D^{-1/2} L D^{-1/2}.
    Symmetric,
    /// Left stochastic: D^{-1} L (row-normalized).
    Left,
    /// Right stochastic: L D^{-1} (column-normalized).
    Right,
}

/// Compute the Laplacian matrix of a graph.
///
/// Returns an n×n matrix stored in row-major order as `Vec<Vec<f64>>`.
///
/// - `mode`: degree mode (Out/In/All). For undirected graphs, forced to All.
/// - `normalization`: normalization method.
/// - `weights`: optional edge weights. If `None`, unweighted.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, DegreeMode, get_laplacian, LaplacianNormalization};
///
/// // Path 0-1-2: unnormalized Laplacian
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let lap = get_laplacian(&g, DegreeMode::All, LaplacianNormalization::Unnormalized, None).unwrap();
/// assert!((lap[0][0] - 1.0).abs() < 1e-10);  // deg(0) = 1
/// assert!((lap[1][1] - 2.0).abs() < 1e-10);  // deg(1) = 2
/// assert!((lap[0][1] + 1.0).abs() < 1e-10);  // -A[0][1] = -1
/// ```
pub fn get_laplacian(
    graph: &Graph,
    mode: DegreeMode,
    normalization: LaplacianNormalization,
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "get_laplacian: weight vector length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    let directed = graph.is_directed();
    let mode_eff = if directed { mode } else { DegreeMode::All };
    let treat_as_undirected = !directed || mode == DegreeMode::All;

    // Compute degree/strength
    let mut degree = vec![0.0_f64; n];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let w = weights.map_or(1.0, |ws| ws[eid]);

        match mode_eff {
            DegreeMode::Out => {
                degree[from as usize] += w;
                if !directed && from != to {
                    degree[to as usize] += w;
                }
            }
            DegreeMode::In => {
                degree[to as usize] += w;
                if !directed && from != to {
                    degree[from as usize] += w;
                }
            }
            DegreeMode::All => {
                degree[from as usize] += w;
                if from == to {
                    degree[from as usize] += w;
                } else {
                    degree[to as usize] += w;
                }
            }
        }
    }

    let mut res = vec![vec![0.0_f64; n]; n];

    // Set diagonal based on normalization
    let mut norm_factor = vec![0.0_f64; n];
    for i in 0..n {
        match normalization {
            LaplacianNormalization::Unnormalized => {
                res[i][i] = degree[i];
            }
            LaplacianNormalization::Symmetric => {
                if degree[i] > 0.0 {
                    res[i][i] = 1.0;
                    norm_factor[i] = 1.0 / degree[i].sqrt();
                }
            }
            LaplacianNormalization::Left | LaplacianNormalization::Right => {
                if degree[i] > 0.0 {
                    res[i][i] = 1.0;
                    norm_factor[i] = 1.0 / degree[i];
                }
            }
        }
    }

    // Set off-diagonal
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let f = from as usize;
        let t = to as usize;

        match normalization {
            LaplacianNormalization::Unnormalized => {
                res[f][t] -= w;
                if treat_as_undirected {
                    res[t][f] -= w;
                }
            }
            LaplacianNormalization::Symmetric => {
                let nf = norm_factor[f] * norm_factor[t];
                let wn = w * nf;
                res[f][t] -= wn;
                if treat_as_undirected {
                    res[t][f] -= wn;
                }
            }
            LaplacianNormalization::Left => {
                res[f][t] -= w * norm_factor[f];
                if treat_as_undirected {
                    res[t][f] -= w * norm_factor[t];
                }
            }
            LaplacianNormalization::Right => {
                res[f][t] -= w * norm_factor[t];
                if treat_as_undirected {
                    res[t][f] -= w * norm_factor[f];
                }
            }
        }
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        assert!(lap.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        assert_eq!(lap.len(), 1);
        assert!(approx_eq(lap[0][0], 0.0));
    }

    #[test]
    fn path_3_unnormalized() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        // L = [[ 1, -1,  0],
        //      [-1,  2, -1],
        //      [ 0, -1,  1]]
        assert!(approx_eq(lap[0][0], 1.0));
        assert!(approx_eq(lap[0][1], -1.0));
        assert!(approx_eq(lap[0][2], 0.0));
        assert!(approx_eq(lap[1][0], -1.0));
        assert!(approx_eq(lap[1][1], 2.0));
        assert!(approx_eq(lap[1][2], -1.0));
        assert!(approx_eq(lap[2][0], 0.0));
        assert!(approx_eq(lap[2][1], -1.0));
        assert!(approx_eq(lap[2][2], 1.0));
    }

    #[test]
    fn row_sum_zero_unnormalized() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        for row in &lap {
            let sum: f64 = row.iter().sum();
            assert!(approx_eq(sum, 0.0));
        }
    }

    #[test]
    fn symmetric_normalization() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let lap =
            get_laplacian(&g, DegreeMode::All, LaplacianNormalization::Symmetric, None).unwrap();
        // Diagonal should be 1 for non-isolated vertices
        assert!(approx_eq(lap[0][0], 1.0));
        assert!(approx_eq(lap[1][1], 1.0));
        assert!(approx_eq(lap[2][2], 1.0));
        // L_sym[0][1] = -1/(sqrt(1)*sqrt(2)) = -1/sqrt(2)
        assert!(approx_eq(lap[0][1], -1.0 / 2.0_f64.sqrt()));
    }

    #[test]
    fn left_normalization() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let lap = get_laplacian(&g, DegreeMode::All, LaplacianNormalization::Left, None).unwrap();
        // Rows should sum to 0
        for row in &lap {
            let sum: f64 = row.iter().sum();
            assert!(approx_eq(sum, 0.0));
        }
        // Diagonal is 1
        assert!(approx_eq(lap[0][0], 1.0));
        assert!(approx_eq(lap[1][1], 1.0));
    }

    #[test]
    fn weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let weights = vec![2.0, 3.0];
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            Some(&weights),
        )
        .unwrap();
        // deg(0) = 2, deg(1) = 5, deg(2) = 3
        assert!(approx_eq(lap[0][0], 2.0));
        assert!(approx_eq(lap[1][1], 5.0));
        assert!(approx_eq(lap[2][2], 3.0));
        assert!(approx_eq(lap[0][1], -2.0));
        assert!(approx_eq(lap[1][2], -3.0));
    }

    #[test]
    fn directed_out_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let lap = get_laplacian(
            &g,
            DegreeMode::Out,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        // out-deg: 0→2, 1→1, 2→0
        assert!(approx_eq(lap[0][0], 2.0));
        assert!(approx_eq(lap[1][1], 1.0));
        assert!(approx_eq(lap[2][2], 0.0));
        // Off-diagonal: only from→to
        assert!(approx_eq(lap[0][1], -1.0));
        assert!(approx_eq(lap[0][2], -1.0));
        assert!(approx_eq(lap[1][0], 0.0)); // no edge 1→0
    }

    #[test]
    fn directed_in_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let lap = get_laplacian(
            &g,
            DegreeMode::In,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        // in-deg: 0→0, 1→1, 2→2
        assert!(approx_eq(lap[0][0], 0.0));
        assert!(approx_eq(lap[1][1], 1.0));
        assert!(approx_eq(lap[2][2], 2.0));
    }

    #[test]
    fn self_loop_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let lap = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            None,
        )
        .unwrap();
        // L_ii = d_i - A_ii. deg(0)=3, A[0][0]=2 → L[0][0]=1
        assert!(approx_eq(lap[0][0], 1.0));
        assert!(approx_eq(lap[1][1], 1.0));
        assert!(approx_eq(lap[0][1], -1.0));
    }

    #[test]
    fn weight_mismatch() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = get_laplacian(
            &g,
            DegreeMode::All,
            LaplacianNormalization::Unnormalized,
            Some(&[1.0, 2.0]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn isolated_vertex_symmetric() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        // vertex 2 is isolated
        let lap =
            get_laplacian(&g, DegreeMode::All, LaplacianNormalization::Symmetric, None).unwrap();
        // Isolated vertex: diagonal = 0, norm_factor = 0
        assert!(approx_eq(lap[2][2], 0.0));
    }
}
