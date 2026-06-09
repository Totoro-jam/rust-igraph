//! Distance matrix spectrum and derived metrics (ALGO-TR-025).
//!
//! The **distance matrix** `D` has entries `D(u,v) = d(u,v)` (shortest
//! path length). Its spectral properties reveal global structural
//! information complementary to the adjacency and Laplacian spectra.
//!
//! - **Distance spectrum**: eigenvalues of `D` sorted in decreasing
//!   order of absolute value.
//! - **Distance spectral radius**: largest eigenvalue `ρ_D = λ_1(D)`.
//! - **Distance energy**: `E_D = Σ |λ_i(D)|` — the distance analogue
//!   of graph energy.
//! - **Distance Estrada index**: `DEE = Σ exp(λ_i(D))`.
//! - **Wiener index**: `W(G) = Σ_{u<v} d(u,v)` — the sum of all
//!   pairwise distances. Also equals half the sum of all distance
//!   matrix entries.
//!
//! All functions require connected undirected graphs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};
use std::collections::VecDeque;

/// Build the dense distance matrix via multi-source BFS.
/// Returns `None` if the graph is disconnected (any pair has infinite distance).
fn dense_distance_matrix(graph: &Graph) -> Option<Vec<f64>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Some(Vec::new());
    }

    let mut dist = vec![f64::INFINITY; n * n];
    for s in 0..n {
        dist[s * n + s] = 0.0;
        let mut queue = VecDeque::new();
        queue.push_back(s as u32);
        let mut visited = vec![false; n];
        visited[s] = true;

        while let Some(u) = queue.pop_front() {
            let ui = u as usize;
            if let Ok(nbrs) = graph.neighbors(u) {
                for &v in &nbrs {
                    let vi = v as usize;
                    if !visited[vi] {
                        visited[vi] = true;
                        dist[s * n + vi] = dist[s * n + ui] + 1.0;
                        queue.push_back(v);
                    }
                }
            }
        }

        for j in 0..n {
            if dist[s * n + j].is_infinite() {
                return None;
            }
        }
    }

    Some(dist)
}

/// Jacobi eigenvalue algorithm for real symmetric matrices.
/// Returns eigenvalues sorted in **decreasing** order.
fn jacobi_eigen_descending(mat: &mut [f64], n: usize) -> Vec<f64> {
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
    eigenvalues.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    eigenvalues
}

/// Compute the Wiener index of a connected graph.
///
/// `W(G) = Σ_{u<v} d(u,v)` — the sum of all pairwise shortest-path
/// distances. Returns an error for disconnected or directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, wiener_index};
///
/// // Path 0-1-2: W = d(0,1) + d(0,2) + d(1,2) = 1 + 2 + 1 = 4
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let w = wiener_index(&g).unwrap();
/// assert!((w - 4.0).abs() < 0.01);
/// ```
pub fn wiener_index(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "wiener_index is defined for connected undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0.0);
    }

    let dist = dense_distance_matrix(graph).ok_or_else(|| {
        IgraphError::InvalidArgument("wiener_index requires a connected graph".into())
    })?;

    let mut w = 0.0_f64;
    for u in 0..n {
        for v in (u + 1)..n {
            w += dist[u * n + v];
        }
    }
    Ok(w)
}

/// Compute the distance spectrum, sorted in decreasing order.
///
/// Returns the eigenvalues of the distance matrix. Requires a
/// connected undirected graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_spectrum};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let spec = distance_spectrum(&g).unwrap();
/// assert_eq!(spec.len(), 3);
/// // Largest eigenvalue is positive
/// assert!(spec[0] > 0.0);
/// ```
pub fn distance_spectrum(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "distance_spectrum is defined for connected undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![0.0]);
    }

    let mut dist = dense_distance_matrix(graph).ok_or_else(|| {
        IgraphError::InvalidArgument("distance_spectrum requires a connected graph".into())
    })?;

    Ok(jacobi_eigen_descending(&mut dist, n))
}

/// Compute the distance spectral radius `ρ_D = λ_1(D)`.
///
/// The largest eigenvalue of the distance matrix. For connected
/// undirected graphs only.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_spectral_radius};
///
/// // K_3: distance matrix has all off-diag = 1, so ρ_D = 2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let rho = distance_spectral_radius(&g).unwrap();
/// assert!((rho - 2.0).abs() < 0.1);
/// ```
pub fn distance_spectral_radius(graph: &Graph) -> IgraphResult<f64> {
    let spec = distance_spectrum(graph)?;
    if spec.is_empty() {
        return Ok(0.0);
    }
    Ok(spec[0])
}

/// Compute the distance energy `E_D = Σ |λ_i(D)|`.
///
/// The sum of absolute values of all distance matrix eigenvalues.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_energy};
///
/// // K_3: eigenvalues {2, -1, -1}, E_D = 4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let e = distance_energy(&g).unwrap();
/// assert!((e - 4.0).abs() < 0.1);
/// ```
pub fn distance_energy(graph: &Graph) -> IgraphResult<f64> {
    let spec = distance_spectrum(graph)?;
    Ok(spec.iter().map(|v| v.abs()).sum())
}

/// Compute the distance Estrada index `DEE = Σ exp(λ_i(D))`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, distance_estrada_index};
///
/// // K_3: eigenvalues {2, -1, -1}, DEE = e^2 + 2e^{-1}
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let dee = distance_estrada_index(&g).unwrap();
/// let expected = 2.0_f64.exp() + 2.0 * (-1.0_f64).exp();
/// assert!((dee - expected).abs() < 0.1);
/// ```
pub fn distance_estrada_index(graph: &Graph) -> IgraphResult<f64> {
    let spec = distance_spectrum(graph)?;
    Ok(spec.iter().map(|v| v.exp()).sum())
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

    // --- wiener_index ---

    #[test]
    fn wi_path3() {
        let g = path3();
        let w = wiener_index(&g).unwrap();
        // d(0,1)=1, d(0,2)=2, d(1,2)=1 → W = 4
        assert!((w - 4.0).abs() < 0.01);
    }

    #[test]
    fn wi_path4() {
        let g = path4();
        let w = wiener_index(&g).unwrap();
        // d(0,1)=1, d(0,2)=2, d(0,3)=3, d(1,2)=1, d(1,3)=2, d(2,3)=1 → W = 10
        assert!((w - 10.0).abs() < 0.01);
    }

    #[test]
    fn wi_k3() {
        let g = k3();
        let w = wiener_index(&g).unwrap();
        // All pairs distance 1: C(3,2) = 3
        assert!((w - 3.0).abs() < 0.01);
    }

    #[test]
    fn wi_k4() {
        let g = k4();
        let w = wiener_index(&g).unwrap();
        // C(4,2) = 6 pairs, each distance 1 → W = 6
        assert!((w - 6.0).abs() < 0.01);
    }

    #[test]
    fn wi_cycle4() {
        let g = cycle4();
        let w = wiener_index(&g).unwrap();
        // 4 adj pairs dist 1, 2 opposite pairs dist 2 → W = 4+4 = 8
        assert!((w - 8.0).abs() < 0.01);
    }

    #[test]
    fn wi_star4() {
        let g = star4();
        let w = wiener_index(&g).unwrap();
        // 3 pairs (center,leaf) dist 1, 3 pairs (leaf,leaf) dist 2 → W = 3+6 = 9
        assert!((w - 9.0).abs() < 0.01);
    }

    #[test]
    fn wi_single() {
        let g = Graph::with_vertices(1);
        let w = wiener_index(&g).unwrap();
        assert!(w.abs() < 1e-10);
    }

    #[test]
    fn wi_disconnected_error() {
        let g = Graph::with_vertices(3);
        assert!(wiener_index(&g).is_err());
    }

    #[test]
    fn wi_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(wiener_index(&g).is_err());
    }

    // --- distance_spectrum ---

    #[test]
    fn ds_k3() {
        let g = k3();
        let spec = distance_spectrum(&g).unwrap();
        // Distance matrix of K_3: J - I with eigenvalues {2, -1, -1}
        assert_eq!(spec.len(), 3);
        assert!((spec[0] - 2.0).abs() < 0.1);
        assert!((spec[1] - (-1.0)).abs() < 0.1);
        assert!((spec[2] - (-1.0)).abs() < 0.1);
    }

    #[test]
    fn ds_decreasing() {
        let g = path4();
        let spec = distance_spectrum(&g).unwrap();
        for i in 1..spec.len() {
            assert!(spec[i] <= spec[i - 1] + 1e-10);
        }
    }

    #[test]
    fn ds_empty() {
        let g = Graph::with_vertices(0);
        let spec = distance_spectrum(&g).unwrap();
        assert!(spec.is_empty());
    }

    #[test]
    fn ds_single() {
        let g = Graph::with_vertices(1);
        let spec = distance_spectrum(&g).unwrap();
        assert_eq!(spec.len(), 1);
        assert!(spec[0].abs() < 1e-10);
    }

    #[test]
    fn ds_disconnected_error() {
        let g = Graph::with_vertices(3);
        assert!(distance_spectrum(&g).is_err());
    }

    // --- distance_spectral_radius ---

    #[test]
    fn dsr_k3() {
        let g = k3();
        let rho = distance_spectral_radius(&g).unwrap();
        assert!((rho - 2.0).abs() < 0.1);
    }

    #[test]
    fn dsr_k4() {
        let g = k4();
        let rho = distance_spectral_radius(&g).unwrap();
        // K_4: distance matrix = J - I, eigenvalues {3, -1, -1, -1}
        assert!((rho - 3.0).abs() < 0.1);
    }

    #[test]
    fn dsr_positive() {
        let g = star4();
        let rho = distance_spectral_radius(&g).unwrap();
        assert!(rho > 0.0);
    }

    // --- distance_energy ---

    #[test]
    fn de_k3() {
        let g = k3();
        let e = distance_energy(&g).unwrap();
        // eigenvalues {2, -1, -1}: E_D = 2 + 1 + 1 = 4
        assert!((e - 4.0).abs() < 0.1);
    }

    #[test]
    fn de_k4() {
        let g = k4();
        let e = distance_energy(&g).unwrap();
        // eigenvalues {3, -1, -1, -1}: E_D = 3 + 1 + 1 + 1 = 6
        assert!((e - 6.0).abs() < 0.1);
    }

    #[test]
    fn de_nonneg() {
        let g = cycle4();
        let e = distance_energy(&g).unwrap();
        assert!(e >= -1e-10);
    }

    // --- distance_estrada_index ---

    #[test]
    fn dee_k3() {
        let g = k3();
        let dee = distance_estrada_index(&g).unwrap();
        let expected = 2.0_f64.exp() + 2.0 * (-1.0_f64).exp();
        assert!((dee - expected).abs() < 0.1);
    }

    #[test]
    fn dee_positive() {
        let g = path4();
        let dee = distance_estrada_index(&g).unwrap();
        assert!(dee > 0.0);
    }

    // --- cross-consistency ---

    #[test]
    fn wiener_equals_half_trace_sum() {
        let g = star4();
        let w = wiener_index(&g).unwrap();
        let dist = dense_distance_matrix(&g).unwrap();
        let n = 4;
        let total: f64 = dist.iter().sum();
        assert!((w - total / 2.0).abs() < 0.01);

        let _ = n;
    }

    #[test]
    fn kn_distance_spectral_radius_is_n_minus_1() {
        for n in 2_u32..=5 {
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    edges.push((u, v));
                }
            }
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let rho = distance_spectral_radius(&g).unwrap();
            assert!(
                (rho - f64::from(n - 1)).abs() < 0.5,
                "K_{n}: ρ_D = {rho}, expected {}",
                n - 1
            );
        }
    }

    #[test]
    fn trace_of_distance_matrix_is_zero() {
        let g = path4();
        let spec = distance_spectrum(&g).unwrap();
        let trace: f64 = spec.iter().sum();
        assert!(trace.abs() < 0.1);
    }
}
