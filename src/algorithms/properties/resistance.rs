//! Effective resistance and Kirchhoff index (ALGO-TR-022).
//!
//! Computes resistance-distance metrics derived from the Laplacian
//! pseudoinverse. Applicable to connected undirected graphs.
//!
//! - **Effective resistance**: `R(u,v) = L†(u,u) + L†(v,v) - 2·L†(u,v)`
//!   where `L†` is the Moore-Penrose pseudoinverse of the Laplacian.
//! - **Kirchhoff index**: `Kf(G) = Σ_{u<v} R(u,v) = n · Σ_{i≥2} 1/λ_i`
//!   — the sum of all pairwise effective resistances.
//! - **Resistance centrality**: `C_R(v) = n / Σ_u R(v,u)` — vertices
//!   with lower total resistance to all others are more central.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the dense Laplacian matrix L = D - A (row-major, n×n).
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
/// Returns eigenvalues (sorted decreasing) and eigenvector columns.
fn jacobi_eigen_full(mat: &mut [f64], n: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
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
        eigenvalues[b]
            .partial_cmp(&eigenvalues[a])
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

/// Compute the Laplacian pseudoinverse L† from the eigendecomposition.
///
/// For a connected graph: `L† = Σ_{λ_i > ε} (1/λ_i) · φ_i · φ_i^T`
/// (skip the zero eigenvalue corresponding to the constant eigenvector).
fn laplacian_pseudoinverse(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Vec::new();
    }

    let mut lap = dense_laplacian(graph);
    let (eigenvalues, eigenvectors) = jacobi_eigen_full(&mut lap, n);

    let mut lpinv = vec![0.0_f64; n * n];
    let eps = 1e-10;

    for (j, &lam) in eigenvalues.iter().enumerate() {
        if lam.abs() < eps {
            continue;
        }
        let inv_lam = 1.0 / lam;
        let phi = &eigenvectors[j];
        for u in 0..n {
            for v in u..n {
                let contrib = inv_lam * phi[u] * phi[v];
                lpinv[u * n + v] += contrib;
                if u != v {
                    lpinv[v * n + u] += contrib;
                }
            }
        }
    }

    lpinv
}

/// Compute the effective resistance between two vertices.
///
/// `R(u,v) = L†(u,u) + L†(v,v) - 2·L†(u,v)`
///
/// For connected undirected graphs only. The effective resistance is
/// a metric (satisfies triangle inequality) and equals the commute
/// time divided by `2·|E|`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, effective_resistance};
///
/// // Path 0-1-2: R(0,2) = 2 (two unit resistors in series)
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let r = effective_resistance(&g, 0, 2).unwrap();
/// assert!((r - 2.0).abs() < 0.01);
/// ```
pub fn effective_resistance(graph: &Graph, u: u32, v: u32) -> IgraphResult<f64> {
    let n = graph.vcount();
    if u >= n || v >= n {
        return Err(IgraphError::InvalidArgument(format!(
            "vertex index out of range: u={u}, v={v}, vcount={n}"
        )));
    }
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "effective_resistance is defined for undirected graphs only".into(),
        ));
    }
    if u == v {
        return Ok(0.0);
    }

    let n = n as usize;
    let lpinv = laplacian_pseudoinverse(graph);
    let ui = u as usize;
    let vi = v as usize;

    Ok(lpinv[ui * n + ui] + lpinv[vi * n + vi] - 2.0 * lpinv[ui * n + vi])
}

/// Compute the effective resistance matrix for all pairs.
///
/// Returns a flattened `n × n` matrix in row-major order where entry
/// `(u,v)` is the effective resistance `R(u,v)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, effective_resistance_matrix};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let r = effective_resistance_matrix(&g).unwrap();
/// // R(0,1) = 1, R(1,2) = 1, R(0,2) = 2
/// assert!((r[0 * 3 + 1] - 1.0).abs() < 0.01);
/// assert!((r[0 * 3 + 2] - 2.0).abs() < 0.01);
/// ```
pub fn effective_resistance_matrix(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "effective_resistance_matrix is defined for undirected graphs only".into(),
        ));
    }

    let lpinv = laplacian_pseudoinverse(graph);
    let mut result = vec![0.0_f64; n * n];

    for u in 0..n {
        for v in (u + 1)..n {
            let r = lpinv[u * n + u] + lpinv[v * n + v] - 2.0 * lpinv[u * n + v];
            result[u * n + v] = r;
            result[v * n + u] = r;
        }
    }

    Ok(result)
}

/// Compute the Kirchhoff index of a graph.
///
/// `Kf(G) = Σ_{u<v} R(u,v) = n · Σ_{i≥2} 1/λ_i`
///
/// where `λ_i` are the non-zero Laplacian eigenvalues.
/// The Kirchhoff index measures the overall resistance of the network.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, kirchhoff_index};
///
/// // Path 0-1-2: R(0,1)=1, R(0,2)=2, R(1,2)=1 → Kf = 4
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let kf = kirchhoff_index(&g).unwrap();
/// assert!((kf - 4.0).abs() < 0.01);
/// ```
pub fn kirchhoff_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "kirchhoff_index is defined for undirected graphs only".into(),
        ));
    }
    if n <= 1 {
        return Ok(0.0);
    }

    let mut lap = dense_laplacian(graph);
    let (eigenvalues, _) = jacobi_eigen_full(&mut lap, n);

    let eps = 1e-10;
    let mut kf = 0.0_f64;
    for &lam in &eigenvalues {
        if lam.abs() > eps {
            kf += 1.0 / lam;
        }
    }
    kf *= n as f64;

    Ok(kf)
}

/// Compute resistance centrality for all vertices.
///
/// `C_R(v) = (n-1) / Σ_{u≠v} R(v,u)`
///
/// Vertices with lower total resistance to all others receive higher
/// centrality scores. Returns `0.0` for isolated vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, resistance_centrality};
///
/// // K_3: all vertices equivalent, so equal centrality
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let c = resistance_centrality(&g).unwrap();
/// assert!((c[0] - c[1]).abs() < 0.01);
/// assert!((c[1] - c[2]).abs() < 0.01);
/// ```
pub fn resistance_centrality(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "resistance_centrality is defined for undirected graphs only".into(),
        ));
    }
    if n <= 1 {
        return Ok(vec![0.0; n]);
    }

    let rmat = effective_resistance_matrix(graph)?;
    let mut centrality = vec![0.0_f64; n];

    for v in 0..n {
        let total_r: f64 = (0..n).filter(|&u| u != v).map(|u| rmat[v * n + u]).sum();
        if total_r > 1e-300 {
            centrality[v] = (n - 1) as f64 / total_r;
        }
    }

    Ok(centrality)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
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

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    // --- effective_resistance ---

    #[test]
    fn er_self() {
        let g = k3();
        let r = effective_resistance(&g, 0, 0).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn er_path_adjacent() {
        let g = path3();
        let r = effective_resistance(&g, 0, 1).unwrap();
        assert!((r - 1.0).abs() < 0.01);
    }

    #[test]
    fn er_path_ends() {
        let g = path3();
        let r = effective_resistance(&g, 0, 2).unwrap();
        assert!((r - 2.0).abs() < 0.01);
    }

    #[test]
    fn er_complete_graph() {
        // K_n: R(u,v) = 2/n for all u≠v
        let g = k4();
        let r = effective_resistance(&g, 0, 1).unwrap();
        assert!((r - 0.5).abs() < 0.01); // 2/4 = 0.5
    }

    #[test]
    fn er_symmetric() {
        let g = star4();
        let r01 = effective_resistance(&g, 0, 1).unwrap();
        let r10 = effective_resistance(&g, 1, 0).unwrap();
        assert!((r01 - r10).abs() < 1e-10);
    }

    #[test]
    fn er_triangle_inequality() {
        let g = path3();
        let r01 = effective_resistance(&g, 0, 1).unwrap();
        let r12 = effective_resistance(&g, 1, 2).unwrap();
        let r02 = effective_resistance(&g, 0, 2).unwrap();
        assert!(r02 <= r01 + r12 + 1e-10);
    }

    #[test]
    fn er_nonneg() {
        let g = cycle4();
        for u in 0..4_u32 {
            for v in 0..4_u32 {
                let r = effective_resistance(&g, u, v).unwrap();
                assert!(r >= -1e-10, "R({u},{v}) = {r} < 0");
            }
        }
    }

    #[test]
    fn er_out_of_range() {
        let g = k3();
        assert!(effective_resistance(&g, 0, 5).is_err());
    }

    #[test]
    fn er_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(effective_resistance(&g, 0, 1).is_err());
    }

    // --- effective_resistance_matrix ---

    #[test]
    fn erm_symmetric() {
        let g = star4();
        let r = effective_resistance_matrix(&g).unwrap();
        let n = 4;
        for u in 0..n {
            for v in 0..n {
                assert!(
                    (r[u * n + v] - r[v * n + u]).abs() < 1e-10,
                    "R({u},{v}) != R({v},{u})"
                );
            }
        }
    }

    #[test]
    fn erm_diagonal_zero() {
        let g = k3();
        let r = effective_resistance_matrix(&g).unwrap();
        let n = 3;
        for v in 0..n {
            assert!(r[v * n + v].abs() < 1e-10);
        }
    }

    #[test]
    fn erm_path3() {
        let g = path3();
        let r = effective_resistance_matrix(&g).unwrap();
        let n = 3;
        assert!((r[1] - 1.0).abs() < 0.01); // R(0,1)
        assert!((r[2] - 2.0).abs() < 0.01); // R(0,2)
        assert!((r[n + 2] - 1.0).abs() < 0.01); // R(1,2)
    }

    // --- kirchhoff_index ---

    #[test]
    fn kf_empty() {
        let g = Graph::with_vertices(1);
        let kf = kirchhoff_index(&g).unwrap();
        assert!(kf.abs() < 1e-10);
    }

    #[test]
    fn kf_path3() {
        let g = path3();
        let kf = kirchhoff_index(&g).unwrap();
        // R(0,1)=1, R(0,2)=2, R(1,2)=1 → Kf = 4
        assert!((kf - 4.0).abs() < 0.1);
    }

    #[test]
    fn kf_complete() {
        // K_n: Kf = C(n,2) · 2/n = n(n-1)/2 · 2/n = n-1
        let g = k4();
        let kf = kirchhoff_index(&g).unwrap();
        assert!((kf - 3.0).abs() < 0.1);
    }

    #[test]
    fn kf_cycle4() {
        // C_4: R(adj)=3/4, R(opp)=1. Kf = 4*(3/4) + 2*(1) = 5
        let g = cycle4();
        let kf = kirchhoff_index(&g).unwrap();
        assert!((kf - 5.0).abs() < 0.1);
    }

    #[test]
    fn kf_equals_sum_of_resistances() {
        let g = star4();
        let rmat = effective_resistance_matrix(&g).unwrap();
        let n = 4;
        let mut sum = 0.0_f64;
        for u in 0..n {
            for v in (u + 1)..n {
                sum += rmat[u * n + v];
            }
        }
        let kf = kirchhoff_index(&g).unwrap();
        assert!((kf - sum).abs() < 0.1);
    }

    #[test]
    fn kf_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(kirchhoff_index(&g).is_err());
    }

    // --- resistance_centrality ---

    #[test]
    fn rc_k3_symmetric() {
        let g = k3();
        let c = resistance_centrality(&g).unwrap();
        assert!((c[0] - c[1]).abs() < 0.01);
        assert!((c[1] - c[2]).abs() < 0.01);
    }

    #[test]
    fn rc_star_center_highest() {
        let g = star4();
        let c = resistance_centrality(&g).unwrap();
        assert!(c[0] > c[1]);
        assert!(c[0] > c[2]);
        assert!(c[0] > c[3]);
    }

    #[test]
    fn rc_path_center_highest() {
        let g = path3();
        let c = resistance_centrality(&g).unwrap();
        // Center vertex (1) should have highest centrality
        assert!(c[1] > c[0]);
        assert!(c[1] > c[2]);
    }

    #[test]
    fn rc_all_positive() {
        let g = k4();
        let c = resistance_centrality(&g).unwrap();
        for &v in &c {
            assert!(v > 0.0);
        }
    }

    #[test]
    fn rc_single_vertex() {
        let g = Graph::with_vertices(1);
        let c = resistance_centrality(&g).unwrap();
        assert!(c[0].abs() < 1e-10);
    }
}
