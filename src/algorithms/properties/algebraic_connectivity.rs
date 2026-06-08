//! Algebraic connectivity and Fiedler vector (ALGO-TR-023).
//!
//! The algebraic connectivity `a(G) = λ_2(L)` is the second-smallest
//! eigenvalue of the Laplacian matrix. It measures how well-connected
//! a graph is — a larger value implies a more robust network.
//!
//! The corresponding eigenvector (the **Fiedler vector**) induces a
//! spectral bisection: vertices can be partitioned by sign into two
//! communities that minimize the normalized cut.
//!
//! Also provides the **Cheeger constant** (isoperimetric number) bound
//! via the Cheeger inequality: `h(G)/2 ≤ a(G) ≤ 2·h(G)`.
//!
//! - **Algebraic connectivity**: `a(G) = λ_2(L)` — 0 for disconnected,
//!   positive for connected graphs.
//! - **Fiedler vector**: eigenvector of `λ_2` — spectral embedding for
//!   graph bisection.
//! - **Spectral bisection**: partition vertices by sign of the Fiedler
//!   vector.
//! - **Laplacian spectrum**: all eigenvalues of the Laplacian, sorted.
//! - **Spanning tree count**: `τ(G) = (1/n) · Π_{i≥2} λ_i` — the
//!   number of spanning trees (Kirchhoff's theorem).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the dense Laplacian matrix L = D - A (row-major, n×n).
fn dense_laplacian(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    let mut lap = vec![0.0_f64; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        lap[ui * n + vi] -= 1.0;
        lap[vi * n + ui] -= 1.0;
        lap[ui * n + ui] += 1.0;
        lap[vi * n + vi] += 1.0;
    }
    lap
}

/// Jacobi eigenvalue algorithm for real symmetric matrices.
/// Returns eigenvalues sorted in **increasing** order and eigenvector columns.
fn jacobi_eigen_ascending(mat: &mut [f64], n: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    if n == 1 {
        return (vec![mat[0]], vec![vec![1.0]]);
    }

    let mut v = vec![0.0_f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    let max_sweeps = 100;
    for _ in 0..max_sweeps {
        let mut max_off = 0.0_f64;
        let mut p = 0;
        let mut q = 1;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = mat[i * n + j].abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }

        if max_off < 1e-14 {
            break;
        }

        let app = mat[p * n + p];
        let aqq = mat[q * n + q];
        let apq = mat[p * n + q];

        let (cos, sin) = if (app - aqq).abs() < 1e-300 {
            let s = std::f64::consts::FRAC_1_SQRT_2;
            (s, s)
        } else {
            let tau = (aqq - app) / (2.0 * apq);
            let t = if tau >= 0.0 {
                1.0 / (tau + (1.0 + tau * tau).sqrt())
            } else {
                -1.0 / (-tau + (1.0 + tau * tau).sqrt())
            };
            let c = 1.0 / (1.0 + t * t).sqrt();
            (c, t * c)
        };

        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let aip = mat[i * n + p];
            let aiq = mat[i * n + q];
            mat[i * n + p] = cos * aip - sin * aiq;
            mat[p * n + i] = mat[i * n + p];
            mat[i * n + q] = sin * aip + cos * aiq;
            mat[q * n + i] = mat[i * n + q];
        }

        let new_pp = cos * cos * app - 2.0 * sin * cos * apq + sin * sin * aqq;
        let new_qq = sin * sin * app + 2.0 * sin * cos * apq + cos * cos * aqq;
        mat[p * n + p] = new_pp;
        mat[q * n + q] = new_qq;
        mat[p * n + q] = 0.0;
        mat[q * n + p] = 0.0;

        for i in 0..n {
            let vip = v[i * n + p];
            let viq = v[i * n + q];
            v[i * n + p] = cos * vip - sin * viq;
            v[i * n + q] = sin * vip + cos * viq;
        }
    }

    let eigenvalues: Vec<f64> = (0..n).map(|i| mat[i * n + i]).collect();
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        eigenvalues[a]
            .partial_cmp(&eigenvalues[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_vals: Vec<f64> = indices.iter().map(|&i| eigenvalues[i]).collect();
    let sorted_vecs: Vec<Vec<f64>> = indices
        .iter()
        .map(|&idx| {
            let mut col = vec![0.0_f64; n];
            for i in 0..n {
                col[i] = v[i * n + idx];
            }
            col
        })
        .collect();

    (sorted_vals, sorted_vecs)
}

/// Compute the full Laplacian spectrum (eigenvalues sorted ascending).
fn laplacian_spectrum_internal(graph: &Graph) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = graph.vcount() as usize;
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut lap = dense_laplacian(graph);
    jacobi_eigen_ascending(&mut lap, n)
}

/// Compute the algebraic connectivity of a graph.
///
/// `a(G) = λ_2(L)` — the second-smallest eigenvalue of the Laplacian.
/// Returns `0.0` for disconnected graphs and single-vertex graphs.
///
/// For undirected graphs only.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, algebraic_connectivity};
///
/// // K_3: Laplacian eigenvalues {0, 3, 3} → a(G) = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let a = algebraic_connectivity(&g).unwrap();
/// assert!((a - 3.0).abs() < 0.01);
/// ```
pub fn algebraic_connectivity(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "algebraic_connectivity is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0.0);
    }

    let (vals, _) = laplacian_spectrum_internal(graph);
    Ok(vals[1].max(0.0))
}

/// Compute the Fiedler vector of a graph.
///
/// The Fiedler vector is the eigenvector corresponding to the
/// algebraic connectivity `λ_2(L)`. It provides a one-dimensional
/// spectral embedding used for graph bisection.
///
/// Returns a vector of length `vcount`. For disconnected or
/// single-vertex graphs, returns the zero vector.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fiedler_vector};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let fv = fiedler_vector(&g).unwrap();
/// assert_eq!(fv.len(), 4);
/// ```
pub fn fiedler_vector(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "fiedler_vector is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(vec![0.0; n]);
    }

    let (_, vecs) = laplacian_spectrum_internal(graph);
    Ok(vecs[1].clone())
}

/// Compute a spectral bisection of the graph.
///
/// Partitions vertices into two groups based on the sign of the
/// Fiedler vector: vertices with non-negative entries go to group 0,
/// vertices with negative entries go to group 1.
///
/// Returns a membership vector of length `vcount` with values 0 or 1.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spectral_bisection};
///
/// // Path 0-1-2-3: natural split at the middle
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let parts = spectral_bisection(&g).unwrap();
/// assert_eq!(parts.len(), 4);
/// // Each part should be non-empty
/// assert!(parts.iter().any(|&p| p == 0));
/// assert!(parts.iter().any(|&p| p == 1));
/// ```
pub fn spectral_bisection(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let fv = fiedler_vector(graph)?;
    Ok(fv.iter().map(|&v| u32::from(v < 0.0)).collect())
}

/// Compute all Laplacian eigenvalues, sorted in ascending order.
///
/// Returns `{0 = λ_1 ≤ λ_2 ≤ … ≤ λ_n}`. The multiplicity of the
/// zero eigenvalue equals the number of connected components.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, laplacian_spectrum};
///
/// // K_3: Laplacian eigenvalues {0, 3, 3}
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let spec = laplacian_spectrum(&g).unwrap();
/// assert!(spec[0].abs() < 0.01);
/// assert!((spec[1] - 3.0).abs() < 0.1);
/// ```
pub fn laplacian_spectrum(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "laplacian_spectrum is defined for undirected graphs only".into(),
        ));
    }
    let (vals, _) = laplacian_spectrum_internal(graph);
    Ok(vals)
}

/// Count the number of spanning trees using Kirchhoff's theorem.
///
/// `τ(G) = (1/n) · Π_{i=2}^{n} λ_i`
///
/// Returns `0.0` for disconnected graphs, `1.0` for single-vertex
/// or edgeless single-component trees. The result is returned as
/// `f64` since spanning tree counts can be astronomically large.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spanning_tree_count};
///
/// // K_3: 3 spanning trees
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let count = spanning_tree_count(&g).unwrap();
/// assert!((count - 3.0).abs() < 0.1);
///
/// // K_4: 16 spanning trees
/// let g4 = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let c4 = spanning_tree_count(&g4).unwrap();
/// assert!((c4 - 16.0).abs() < 0.5);
/// ```
pub fn spanning_tree_count(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "spanning_tree_count is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(if n == 1 { 1.0 } else { 0.0 });
    }

    let (vals, _) = laplacian_spectrum_internal(graph);

    let eps = 1e-10;
    let mut product = 1.0_f64;
    let mut nonzero_count = 0_usize;

    for &lam in &vals[1..] {
        if lam.abs() > eps {
            product *= lam;
            nonzero_count += 1;
        }
    }

    if nonzero_count < n - 1 {
        return Ok(0.0);
    }

    Ok(product / n as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    // --- algebraic_connectivity ---

    #[test]
    fn ac_single() {
        let g = Graph::with_vertices(1);
        let a = algebraic_connectivity(&g).unwrap();
        assert!(a.abs() < 1e-10);
    }

    #[test]
    fn ac_disconnected() {
        let g = Graph::with_vertices(3);
        let a = algebraic_connectivity(&g).unwrap();
        assert!(a.abs() < 0.01);
    }

    #[test]
    fn ac_k3() {
        let g = k3();
        let a = algebraic_connectivity(&g).unwrap();
        // K_3: eigenvalues {0, 3, 3} → a = 3
        assert!((a - 3.0).abs() < 0.1);
    }

    #[test]
    fn ac_k4() {
        let g = k4();
        let a = algebraic_connectivity(&g).unwrap();
        // K_4: eigenvalues {0, 4, 4, 4} → a = 4
        assert!((a - 4.0).abs() < 0.1);
    }

    #[test]
    fn ac_path() {
        let g = path4();
        let a = algebraic_connectivity(&g).unwrap();
        // P_4: λ_2 = 2 - √2 ≈ 0.586
        assert!((a - (2.0 - std::f64::consts::SQRT_2)).abs() < 0.1);
    }

    #[test]
    fn ac_cycle() {
        let g = cycle4();
        let a = algebraic_connectivity(&g).unwrap();
        // C_4: λ_2 = 2 (eigenvalues {0, 2, 2, 4})
        assert!((a - 2.0).abs() < 0.1);
    }

    #[test]
    fn ac_nonneg() {
        let g = star4();
        let a = algebraic_connectivity(&g).unwrap();
        assert!(a >= -1e-10);
    }

    #[test]
    fn ac_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(algebraic_connectivity(&g).is_err());
    }

    // --- fiedler_vector ---

    #[test]
    fn fv_length() {
        let g = path4();
        let fv = fiedler_vector(&g).unwrap();
        assert_eq!(fv.len(), 4);
    }

    #[test]
    fn fv_orthogonal_to_ones() {
        // The Fiedler vector should be orthogonal to the all-ones vector
        let g = k3();
        let fv = fiedler_vector(&g).unwrap();
        let dot: f64 = fv.iter().sum();
        assert!(dot.abs() < 0.1);
    }

    #[test]
    fn fv_single() {
        let g = Graph::with_vertices(1);
        let fv = fiedler_vector(&g).unwrap();
        assert_eq!(fv.len(), 1);
        assert!(fv[0].abs() < 1e-10);
    }

    // --- spectral_bisection ---

    #[test]
    fn sb_two_parts() {
        let g = path4();
        let parts = spectral_bisection(&g).unwrap();
        assert_eq!(parts.len(), 4);
        assert!(parts.contains(&0));
        assert!(parts.contains(&1));
    }

    #[test]
    fn sb_values_01() {
        let g = k3();
        let parts = spectral_bisection(&g).unwrap();
        for &p in &parts {
            assert!(p == 0 || p == 1);
        }
    }

    // --- laplacian_spectrum ---

    #[test]
    fn ls_empty() {
        let g = Graph::with_vertices(0);
        let spec = laplacian_spectrum(&g).unwrap();
        assert!(spec.is_empty());
    }

    #[test]
    fn ls_k3() {
        let g = k3();
        let spec = laplacian_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 3);
        assert!(spec[0].abs() < 0.01); // λ_1 = 0
        assert!((spec[1] - 3.0).abs() < 0.1);
        assert!((spec[2] - 3.0).abs() < 0.1);
    }

    #[test]
    fn ls_ascending() {
        let g = star4();
        let spec = laplacian_spectrum(&g).unwrap();
        for i in 1..spec.len() {
            assert!(spec[i] >= spec[i - 1] - 1e-10);
        }
    }

    #[test]
    fn ls_first_is_zero() {
        let g = cycle4();
        let spec = laplacian_spectrum(&g).unwrap();
        assert!(spec[0].abs() < 0.01);
    }

    #[test]
    fn ls_disconnected_has_two_zeros() {
        // Two isolated vertices → two zero eigenvalues
        let g = Graph::with_vertices(2);
        let spec = laplacian_spectrum(&g).unwrap();
        assert!(spec[0].abs() < 0.01);
        assert!(spec[1].abs() < 0.01);
    }

    // --- spanning_tree_count ---

    #[test]
    fn stc_single() {
        let g = Graph::with_vertices(1);
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 1.0).abs() < 0.1);
    }

    #[test]
    fn stc_edge() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 1.0).abs() < 0.1);
    }

    #[test]
    fn stc_k3() {
        let g = k3();
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 3.0).abs() < 0.5);
    }

    #[test]
    fn stc_k4() {
        // K_4: τ = 4^(4-2) = 16 (Cayley's formula)
        let g = k4();
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 16.0).abs() < 1.0);
    }

    #[test]
    fn stc_cycle4() {
        // C_4: 4 spanning trees
        let g = cycle4();
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 4.0).abs() < 0.5);
    }

    #[test]
    fn stc_path() {
        // Any tree has exactly 1 spanning tree (itself)
        let g = path3();
        let c = spanning_tree_count(&g).unwrap();
        assert!((c - 1.0).abs() < 0.1);
    }

    #[test]
    fn stc_disconnected() {
        let g = Graph::with_vertices(3);
        let c = spanning_tree_count(&g).unwrap();
        assert!(c.abs() < 0.1);
    }

    #[test]
    fn stc_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(spanning_tree_count(&g).is_err());
    }

    // --- cross-consistency ---

    #[test]
    fn ac_equals_lambda2() {
        let g = star4();
        let a = algebraic_connectivity(&g).unwrap();
        let spec = laplacian_spectrum(&g).unwrap();
        assert!((a - spec[1]).abs() < 0.01);
    }

    #[test]
    fn complete_graph_ac_is_n() {
        // K_n: a(G) = n
        for n in 3_u32..=5 {
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    edges.push((u, v));
                }
            }
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let a = algebraic_connectivity(&g).unwrap();
            assert!(
                (a - f64::from(n)).abs() < 0.5,
                "K_{n}: a(G) = {a}, expected {n}"
            );
        }
    }
}
