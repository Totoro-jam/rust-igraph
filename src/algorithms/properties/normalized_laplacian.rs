//! Normalized Laplacian spectrum and derived metrics (ALGO-TR-024).
//!
//! The **normalized Laplacian** `L_norm = D^{-1/2} L D^{-1/2}` has
//! eigenvalues in `[0, 2]` and is better suited than the combinatorial
//! Laplacian for comparing graphs of different sizes/densities.
//!
//! - **Normalized Laplacian spectrum**: eigenvalues `μ_1 ≤ … ≤ μ_n` of
//!   `L_norm = I - D^{-1/2} A D^{-1/2}`.
//! - **Cheeger constant bound**: `h(G)/2 ≤ μ_2 ≤ 2·h(G)` (Cheeger
//!   inequality). We report a lower bound `μ_2/2` and upper bound
//!   `√(2·μ_2)` for the isoperimetric number.
//! - **Spectral gap ratio**: `μ_2 / μ_n` — a normalized measure of
//!   expansion; closer to 1 implies more uniform connectivity.
//! - **Normalized algebraic connectivity**: `μ_2(L_norm)` — degree-normalized
//!   version of the algebraic connectivity.
//! - **Bipartiteness ratio**: `(2 - μ_n) / 2` — measures deviation from
//!   bipartiteness; 0 for bipartite graphs, positive otherwise.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the dense normalized Laplacian `L_norm = I - D^{-1/2} A D^{-1/2}` (row-major, n×n).
///
/// Isolated vertices get `L_norm[v,v] = 0` (convention: `0/0 = 0`).
fn dense_normalized_laplacian(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    let mut lnorm = vec![0.0_f64; n * n];

    let mut deg = vec![0_u32; n];
    for (u, v) in graph.edges() {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
    }

    let inv_sqrt_deg: Vec<f64> = deg
        .iter()
        .map(|&d| {
            if d == 0 {
                0.0
            } else {
                1.0 / (f64::from(d)).sqrt()
            }
        })
        .collect();

    for i in 0..n {
        if deg[i] > 0 {
            lnorm[i * n + i] = 1.0;
        }
    }

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        let val = inv_sqrt_deg[ui] * inv_sqrt_deg[vi];
        lnorm[ui * n + vi] -= val;
        lnorm[vi * n + ui] -= val;
    }

    lnorm
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

/// Compute the internal decomposition.
fn normalized_laplacian_internal(graph: &Graph) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = graph.vcount() as usize;
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut lnorm = dense_normalized_laplacian(graph);
    jacobi_eigen_ascending(&mut lnorm, n)
}

/// Compute the normalized Laplacian spectrum, sorted ascending.
///
/// The normalized Laplacian `L_norm = I - D^{-1/2} A D^{-1/2}` has all
/// eigenvalues in `[0, 2]`. The multiplicity of the zero eigenvalue
/// equals the number of connected components (among non-isolated
/// vertices).
///
/// For undirected graphs only.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, normalized_laplacian_spectrum};
///
/// // K_3: normalized Laplacian eigenvalues {0, 3/2, 3/2}
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let spec = normalized_laplacian_spectrum(&g).unwrap();
/// assert!(spec[0].abs() < 0.01);
/// assert!((spec[1] - 1.5).abs() < 0.1);
/// ```
pub fn normalized_laplacian_spectrum(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "normalized_laplacian_spectrum is defined for undirected graphs only".into(),
        ));
    }
    let (vals, _) = normalized_laplacian_internal(graph);
    Ok(vals)
}

/// Compute the normalized algebraic connectivity `μ_2(L_norm)`.
///
/// This is the second-smallest eigenvalue of the normalized Laplacian.
/// It is in `[0, 1]` for connected graphs and equals 0 for disconnected
/// ones. Unlike the combinatorial `a(G) = λ_2(L)`, it is bounded
/// independently of the graph size.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, normalized_algebraic_connectivity};
///
/// // K_3: μ_2 = 3/2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let mu2 = normalized_algebraic_connectivity(&g).unwrap();
/// assert!((mu2 - 1.5).abs() < 0.1);
/// ```
pub fn normalized_algebraic_connectivity(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "normalized_algebraic_connectivity is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0.0);
    }

    let (vals, _) = normalized_laplacian_internal(graph);
    Ok(vals[1].max(0.0))
}

/// Compute Cheeger constant bounds from the normalized Laplacian.
///
/// The Cheeger inequality relates the isoperimetric number `h(G)` to the
/// normalized algebraic connectivity `μ_2`:
///
/// `μ_2/2 ≤ h(G) ≤ √(2·μ_2)`
///
/// Returns `(lower_bound, upper_bound)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cheeger_bounds};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let (lo, hi) = cheeger_bounds(&g).unwrap();
/// assert!(lo <= hi + 1e-10);
/// assert!(lo >= 0.0);
/// ```
pub fn cheeger_bounds(graph: &Graph) -> IgraphResult<(f64, f64)> {
    let mu2 = normalized_algebraic_connectivity(graph)?;
    let lower = mu2 / 2.0;
    let upper = (2.0 * mu2).sqrt();
    Ok((lower, upper))
}

/// Compute the spectral gap ratio `μ_2 / μ_n`.
///
/// A ratio close to 1 indicates uniform expansion (the graph is close
/// to Ramanujan). Returns `0.0` if `μ_n` is zero or the graph has
/// fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spectral_gap_ratio};
///
/// // K_3: μ_2 = μ_3 = 3/2, ratio = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let r = spectral_gap_ratio(&g).unwrap();
/// assert!((r - 1.0).abs() < 0.1);
/// ```
pub fn spectral_gap_ratio(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "spectral_gap_ratio is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0.0);
    }

    let (vals, _) = normalized_laplacian_internal(graph);
    let mu_n = vals[n - 1];
    if mu_n.abs() < 1e-14 {
        return Ok(0.0);
    }
    Ok(vals[1].max(0.0) / mu_n)
}

/// Compute the bipartiteness ratio `(2 - μ_n) / 2`.
///
/// For bipartite graphs `μ_n = 2`, so the ratio is `0`. For
/// non-bipartite graphs `μ_n < 2`, giving a positive value that
/// measures how far the graph is from being bipartite.
///
/// Returns `1.0` for graphs with `μ_n = 0` (isolated vertices only).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bipartiteness_ratio};
///
/// // K_{2,2} (bipartite): ratio = 0
/// let g = Graph::from_edges(&[(0,2),(0,3),(1,2),(1,3)], false, Some(4)).unwrap();
/// let br = bipartiteness_ratio(&g).unwrap();
/// assert!(br.abs() < 0.01);
///
/// // K_3 (non-bipartite): μ_n = 3/2, ratio = (2 - 3/2)/2 = 1/4
/// let k3 = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let br3 = bipartiteness_ratio(&k3).unwrap();
/// assert!((br3 - 0.25).abs() < 0.05);
/// ```
pub fn bipartiteness_ratio(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "bipartiteness_ratio is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(1.0);
    }

    let (vals, _) = normalized_laplacian_internal(graph);
    let mu_n = vals[n - 1];
    Ok((2.0 - mu_n) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    fn k22() -> Graph {
        Graph::from_edges(&[(0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // --- normalized_laplacian_spectrum ---

    #[test]
    fn nls_empty() {
        let g = Graph::with_vertices(0);
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert!(spec.is_empty());
    }

    #[test]
    fn nls_single() {
        let g = Graph::with_vertices(1);
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 1);
        assert!(spec[0].abs() < 1e-10);
    }

    #[test]
    fn nls_k3() {
        let g = k3();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 3);
        assert!(spec[0].abs() < 0.01);
        assert!((spec[1] - 1.5).abs() < 0.1);
        assert!((spec[2] - 1.5).abs() < 0.1);
    }

    #[test]
    fn nls_k4() {
        let g = k4();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        // K_n: normalized Laplacian eigenvalues {0, n/(n-1), ..., n/(n-1)}
        // K_4: {0, 4/3, 4/3, 4/3}
        assert!(spec[0].abs() < 0.01);
        for i in 1..4 {
            assert!((spec[i] - 4.0 / 3.0).abs() < 0.1);
        }
    }

    #[test]
    fn nls_in_range() {
        let g = star4();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        for &v in &spec {
            assert!((-0.01..=2.01).contains(&v), "eigenvalue {v} out of [0,2]");
        }
    }

    #[test]
    fn nls_ascending() {
        let g = cycle4();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        for i in 1..spec.len() {
            assert!(spec[i] >= spec[i - 1] - 1e-10);
        }
    }

    #[test]
    fn nls_first_is_zero() {
        let g = path4();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert!(spec[0].abs() < 0.01);
    }

    #[test]
    fn nls_bipartite_last_is_two() {
        // Bipartite graphs have μ_n = 2
        let g = k22();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert!((spec[spec.len() - 1] - 2.0).abs() < 0.01);
    }

    #[test]
    fn nls_disconnected() {
        let g = Graph::with_vertices(3);
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        for &v in &spec {
            assert!(v.abs() < 0.01);
        }
    }

    #[test]
    fn nls_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(normalized_laplacian_spectrum(&g).is_err());
    }

    // --- normalized_algebraic_connectivity ---

    #[test]
    fn nac_k3() {
        let g = k3();
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        assert!((mu2 - 1.5).abs() < 0.1);
    }

    #[test]
    fn nac_k4() {
        let g = k4();
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        assert!((mu2 - 4.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn nac_disconnected() {
        let g = Graph::with_vertices(3);
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        assert!(mu2.abs() < 0.01);
    }

    #[test]
    fn nac_single() {
        let g = Graph::with_vertices(1);
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        assert!(mu2.abs() < 1e-10);
    }

    #[test]
    fn nac_cycle4() {
        let g = cycle4();
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        // C_4: regular with degree 2, so normalized = combinatorial / degree
        // Combinatorial eigenvalues {0, 2, 2, 4}, normalized {0, 1, 1, 2}
        assert!((mu2 - 1.0).abs() < 0.1);
    }

    // --- cheeger_bounds ---

    #[test]
    fn cb_connected() {
        let g = k3();
        let (lo, hi) = cheeger_bounds(&g).unwrap();
        assert!(lo >= 0.0);
        assert!(lo <= hi + 1e-10);
    }

    #[test]
    fn cb_disconnected() {
        let g = Graph::with_vertices(3);
        let (lo, hi) = cheeger_bounds(&g).unwrap();
        assert!(lo.abs() < 0.01);
        assert!(hi.abs() < 0.01);
    }

    #[test]
    fn cb_single() {
        let g = Graph::with_vertices(1);
        let (lo, hi) = cheeger_bounds(&g).unwrap();
        assert!(lo.abs() < 1e-10);
        assert!(hi.abs() < 1e-10);
    }

    // --- spectral_gap_ratio ---

    #[test]
    fn sgr_k3() {
        let g = k3();
        let r = spectral_gap_ratio(&g).unwrap();
        // K_3: μ_2 = μ_3 = 3/2, ratio = 1.0
        assert!((r - 1.0).abs() < 0.1);
    }

    #[test]
    fn sgr_k4() {
        let g = k4();
        let r = spectral_gap_ratio(&g).unwrap();
        // All non-zero eigenvalues equal → ratio = 1.0
        assert!((r - 1.0).abs() < 0.1);
    }

    #[test]
    fn sgr_path() {
        let g = path4();
        let r = spectral_gap_ratio(&g).unwrap();
        assert!(r > 0.0 && r <= 1.0 + 1e-10);
    }

    #[test]
    fn sgr_single() {
        let g = Graph::with_vertices(1);
        let r = spectral_gap_ratio(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn sgr_disconnected() {
        let g = Graph::with_vertices(3);
        let r = spectral_gap_ratio(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    // --- bipartiteness_ratio ---

    #[test]
    fn br_bipartite() {
        let g = k22();
        let br = bipartiteness_ratio(&g).unwrap();
        assert!(br.abs() < 0.01);
    }

    #[test]
    fn br_path_bipartite() {
        let g = path4();
        let br = bipartiteness_ratio(&g).unwrap();
        assert!(br.abs() < 0.01);
    }

    #[test]
    fn br_k3_nonbipartite() {
        let g = k3();
        let br = bipartiteness_ratio(&g).unwrap();
        // μ_n = 3/2, ratio = (2 - 3/2) / 2 = 1/4
        assert!((br - 0.25).abs() < 0.05);
    }

    #[test]
    fn br_k4_nonbipartite() {
        let g = k4();
        let br = bipartiteness_ratio(&g).unwrap();
        // μ_n = 4/3, ratio = (2 - 4/3)/2 = 1/3
        assert!((br - 1.0 / 3.0).abs() < 0.05);
    }

    #[test]
    fn br_nonneg() {
        let g = cycle4();
        let br = bipartiteness_ratio(&g).unwrap();
        assert!(br >= -0.01);
    }

    // --- cross-consistency ---

    #[test]
    fn nac_equals_second_eigenvalue() {
        let g = star4();
        let mu2 = normalized_algebraic_connectivity(&g).unwrap();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        assert!((mu2 - spec[1]).abs() < 0.01);
    }

    #[test]
    fn regular_graph_normalized_equals_scaled_combinatorial() {
        // For k-regular graphs: normalized eigenvalues = combinatorial / k
        let g = cycle4(); // 2-regular
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        // Combinatorial eigenvalues of C_4: {0, 2, 2, 4}
        // Normalized: {0, 1, 1, 2}
        assert!(spec[0].abs() < 0.01);
        assert!((spec[1] - 1.0).abs() < 0.1);
        assert!((spec[2] - 1.0).abs() < 0.1);
        assert!((spec[3] - 2.0).abs() < 0.1);
    }

    #[test]
    fn trace_equals_nontrivial_vertex_count() {
        // tr(L_norm) = number of non-isolated vertices
        let g = star4();
        let spec = normalized_laplacian_spectrum(&g).unwrap();
        let trace: f64 = spec.iter().sum();
        assert!((trace - 4.0).abs() < 0.1);
    }
}
