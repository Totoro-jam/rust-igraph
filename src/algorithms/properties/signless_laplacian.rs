//! Signless Laplacian spectrum and derived metrics (ALGO-TR-026).
//!
//! The **signless Laplacian** `Q = D + A` is the positive-definite
//! analogue of the combinatorial Laplacian `L = D - A`. Its spectrum
//! is particularly useful for detecting bipartiteness and odd cycles.
//!
//! - **Signless Laplacian spectrum**: eigenvalues `q_1 ≤ … ≤ q_n`
//!   of `Q = D + A`, sorted ascending.
//! - **Smallest `Q`-eigenvalue**: `q_1(Q)` — equals 0 iff the graph
//!   has a bipartite component.
//! - **Largest `Q`-eigenvalue**: `q_n(Q)` — the signless Laplacian
//!   spectral radius.
//! - **Signless Laplacian energy**: `QE = Σ |q_i - 2m/n|` where
//!   `m = |E|`, analogous to graph energy but centered on the mean
//!   eigenvalue.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the dense signless Laplacian Q = D + A (row-major, n×n).
fn dense_signless_laplacian(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    let mut q = vec![0.0_f64; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        q[ui * n + vi] += 1.0;
        q[vi * n + ui] += 1.0;
        q[ui * n + ui] += 1.0;
        q[vi * n + vi] += 1.0;
    }
    q
}

/// Jacobi eigenvalue algorithm for real symmetric matrices.
/// Returns eigenvalues sorted in **increasing** order.
fn jacobi_eigen_ascending(mat: &mut [f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![mat[0]];
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
    }

    let mut eigenvalues: Vec<f64> = (0..n).map(|i| mat[i * n + i]).collect();
    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    eigenvalues
}

/// Compute the signless Laplacian spectrum, sorted ascending.
///
/// The signless Laplacian `Q = D + A` has all non-negative eigenvalues.
/// The smallest eigenvalue `q_1 = 0` if and only if the graph has a
/// bipartite connected component.
///
/// For undirected graphs only.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, signless_laplacian_spectrum};
///
/// // K_3: Q eigenvalues {1, 1, 4} (sorted: {1, 1, 4})
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let spec = signless_laplacian_spectrum(&g).unwrap();
/// assert!((spec[0] - 1.0).abs() < 0.1);
/// assert!((spec[2] - 4.0).abs() < 0.1);
/// ```
pub fn signless_laplacian_spectrum(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "signless_laplacian_spectrum is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    let mut q = dense_signless_laplacian(graph);
    Ok(jacobi_eigen_ascending(&mut q, n))
}

/// Compute the smallest signless Laplacian eigenvalue `q_1(Q)`.
///
/// Equals 0 for graphs with a bipartite component, positive otherwise.
/// This is a useful bipartiteness test: a connected graph is bipartite
/// if and only if `q_1 = 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, signless_laplacian_smallest};
///
/// // K_{2,2} (bipartite): q_1 = 0
/// let g = Graph::from_edges(&[(0,2),(0,3),(1,2),(1,3)], false, Some(4)).unwrap();
/// let q1 = signless_laplacian_smallest(&g).unwrap();
/// assert!(q1.abs() < 0.01);
///
/// // K_3 (non-bipartite): q_1 = 1
/// let k3 = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let q1k = signless_laplacian_smallest(&k3).unwrap();
/// assert!(q1k > 0.5);
/// ```
pub fn signless_laplacian_smallest(graph: &Graph) -> IgraphResult<f64> {
    let spec = signless_laplacian_spectrum(graph)?;
    if spec.is_empty() {
        return Ok(0.0);
    }
    Ok(spec[0].max(0.0))
}

/// Compute the signless Laplacian spectral radius `q_n(Q)`.
///
/// The largest eigenvalue of `Q = D + A`. Bounded above by
/// `2 · max_degree`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, signless_laplacian_spectral_radius};
///
/// // K_3: Q eigenvalues {1, 1, 4} → q_max = 4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let qn = signless_laplacian_spectral_radius(&g).unwrap();
/// assert!((qn - 4.0).abs() < 0.1);
/// ```
pub fn signless_laplacian_spectral_radius(graph: &Graph) -> IgraphResult<f64> {
    let spec = signless_laplacian_spectrum(graph)?;
    if spec.is_empty() {
        return Ok(0.0);
    }
    Ok(spec[spec.len() - 1])
}

/// Compute the signless Laplacian energy.
///
/// `QE = Σ |q_i - 2m/n|` where `m = |E|` and `n = |V|`.
/// This is the deviation of the signless Laplacian eigenvalues from
/// their mean value `2m/n`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, signless_laplacian_energy};
///
/// // K_3: eigenvalues {1, 1, 4}, mean = 6/3 = 2
/// // QE = |1-2| + |1-2| + |4-2| = 1 + 1 + 2 = 4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let qe = signless_laplacian_energy(&g).unwrap();
/// assert!((qe - 4.0).abs() < 0.1);
/// ```
pub fn signless_laplacian_energy(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "signless_laplacian_energy is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let spec = signless_laplacian_spectrum(graph)?;
    let m = graph.ecount() as f64;
    let mean = 2.0 * m / n as f64;

    Ok(spec.iter().map(|&q| (q - mean).abs()).sum())
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

    // --- signless_laplacian_spectrum ---

    #[test]
    fn sls_empty() {
        let g = Graph::with_vertices(0);
        let spec = signless_laplacian_spectrum(&g).unwrap();
        assert!(spec.is_empty());
    }

    #[test]
    fn sls_single() {
        let g = Graph::with_vertices(1);
        let spec = signless_laplacian_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 1);
        assert!(spec[0].abs() < 1e-10);
    }

    #[test]
    fn sls_k3() {
        let g = k3();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 3);
        // Q(K_3) = D + A = 2I + A; A eigenvalues {2,-1,-1}
        // so Q eigenvalues: {4, 1, 1}
        assert!((spec[0] - 1.0).abs() < 0.1);
        assert!((spec[1] - 1.0).abs() < 0.1);
        assert!((spec[2] - 4.0).abs() < 0.1);
    }

    #[test]
    fn sls_k4() {
        let g = k4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        // Q(K_n) = (n-1)I + A; A eigenvalues {n-1, -1, ..., -1}
        // K_4: Q eigenvalues = {6, 2, 2, 2}
        assert!((spec[0] - 2.0).abs() < 0.1);
        assert!((spec[3] - 6.0).abs() < 0.1);
    }

    #[test]
    fn sls_nonneg() {
        let g = star4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        for &v in &spec {
            assert!(v >= -0.01, "eigenvalue {v} < 0");
        }
    }

    #[test]
    fn sls_ascending() {
        let g = cycle4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        for i in 1..spec.len() {
            assert!(spec[i] >= spec[i - 1] - 1e-10);
        }
    }

    #[test]
    fn sls_bipartite_has_zero() {
        let g = k22();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        assert!(spec[0].abs() < 0.01);
    }

    #[test]
    fn sls_path_bipartite_has_zero() {
        let g = path4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        assert!(spec[0] < 0.01);
    }

    #[test]
    fn sls_isolated_vertices() {
        let g = Graph::with_vertices(3);
        let spec = signless_laplacian_spectrum(&g).unwrap();
        for &v in &spec {
            assert!(v.abs() < 0.01);
        }
    }

    #[test]
    fn sls_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(signless_laplacian_spectrum(&g).is_err());
    }

    // --- signless_laplacian_smallest ---

    #[test]
    fn slsmall_bipartite_zero() {
        let g = k22();
        let q1 = signless_laplacian_smallest(&g).unwrap();
        assert!(q1.abs() < 0.01);
    }

    #[test]
    fn slsmall_nonbipartite_positive() {
        let g = k3();
        let q1 = signless_laplacian_smallest(&g).unwrap();
        assert!(q1 > 0.5);
    }

    #[test]
    fn slsmall_single() {
        let g = Graph::with_vertices(1);
        let q1 = signless_laplacian_smallest(&g).unwrap();
        assert!(q1.abs() < 1e-10);
    }

    // --- signless_laplacian_spectral_radius ---

    #[test]
    fn slsr_k3() {
        let g = k3();
        let qn = signless_laplacian_spectral_radius(&g).unwrap();
        // Q eigenvalues {1,1,4} → q_max = 4
        assert!((qn - 4.0).abs() < 0.1);
    }

    #[test]
    fn slsr_k4() {
        let g = k4();
        let qn = signless_laplacian_spectral_radius(&g).unwrap();
        // Q eigenvalues {2,2,2,6} → q_max = 6
        assert!((qn - 6.0).abs() < 0.1);
    }

    #[test]
    fn slsr_at_most_2maxdeg() {
        let g = star4();
        let qn = signless_laplacian_spectral_radius(&g).unwrap();
        // max_degree = 3, so q_n ≤ 6
        assert!(qn <= 6.01);
    }

    // --- signless_laplacian_energy ---

    #[test]
    fn sle_k3() {
        let g = k3();
        let qe = signless_laplacian_energy(&g).unwrap();
        // eigenvalues {1,1,4}, mean=2, QE = |1-2|+|1-2|+|4-2| = 4
        assert!((qe - 4.0).abs() < 0.1);
    }

    #[test]
    fn sle_nonneg() {
        let g = cycle4();
        let qe = signless_laplacian_energy(&g).unwrap();
        assert!(qe >= -1e-10);
    }

    #[test]
    fn sle_empty() {
        let g = Graph::with_vertices(0);
        let qe = signless_laplacian_energy(&g).unwrap();
        assert!(qe.abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn trace_equals_twice_edges() {
        let g = star4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        let trace: f64 = spec.iter().sum();
        // tr(Q) = tr(D) + tr(A) = 2m + 0 = 2m (for simple graphs tr(A)=0)
        let m = g.ecount() as f64;
        assert!((trace - 2.0 * m).abs() < 0.1);
    }

    #[test]
    fn q_and_l_share_same_trace() {
        // tr(Q) = tr(L) = 2m since tr(D+A) = tr(D-A) + 2·tr(A)
        // and tr(A) = 0 for simple graphs
        let g = cycle4();
        let q_spec = signless_laplacian_spectrum(&g).unwrap();
        let q_trace: f64 = q_spec.iter().sum();
        let m = g.ecount() as f64;
        assert!((q_trace - 2.0 * m).abs() < 0.1);
    }

    #[test]
    fn regular_q_eigenvalue_relation() {
        // For a k-regular graph: q_i = k + λ_i where λ_i are adjacency eigenvalues
        // C_4 is 2-regular, adjacency eigenvalues {2, 0, 0, -2}
        // So Q eigenvalues should be {4, 2, 2, 0}
        let g = cycle4();
        let spec = signless_laplacian_spectrum(&g).unwrap();
        // sorted ascending: {0, 2, 2, 4}
        assert!(spec[0].abs() < 0.1);
        assert!((spec[1] - 2.0).abs() < 0.1);
        assert!((spec[2] - 2.0).abs() < 0.1);
        assert!((spec[3] - 4.0).abs() < 0.1);
    }
}
