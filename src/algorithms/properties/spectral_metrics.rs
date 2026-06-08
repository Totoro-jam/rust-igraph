//! Spectral graph metrics (ALGO-TR-021).
//!
//! Eigenvalue-based invariants derived from the adjacency spectrum.
//! All metrics operate on the full adjacency eigenspectrum `{λ_1, …, λ_n}`
//! (computed via the Lanczos solver), making them suitable for moderate-size
//! graphs (say ≤ 5 000 vertices).
//!
//! - **Estrada index**: `EE(G) = Σ_i exp(λ_i)` — measures the "folded-ness"
//!   of the network; related to subgraph counts.
//! - **Subgraph centrality**: `SC(v) = Σ_k (e^A)_{vv}` — the diagonal of
//!   the matrix exponential, a centrality measure counting closed walks.
//! - **Natural connectivity**: `λ̄ = ln(EE(G) / n)` — a robust measure of
//!   network structural redundancy / fault tolerance.
//! - **Spectral radius**: `ρ(G) = max_i |λ_i|` — the largest eigenvalue
//!   magnitude, upper-bounds epidemic thresholds and walk counts.
//! - **Energy**: `E(G) = Σ_i |λ_i|` — the sum of absolute eigenvalues,
//!   originating from mathematical chemistry (Hückel theory).
//! - **Spectral gap**: `Δ = λ_1 - λ_2` — the difference between the two
//!   largest eigenvalues; large gap implies good expansion / rapid mixing.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build the dense adjacency matrix (row-major, n×n).
fn dense_adjacency(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    let mut a = vec![0.0_f64; n * n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        a[ui * n + vi] += 1.0;
        if !graph.is_directed() && ui != vi {
            a[vi * n + ui] += 1.0;
        }
    }
    a
}

/// Jacobi eigenvalue algorithm for real symmetric matrices.
///
/// Returns all eigenvalues sorted in decreasing order, and (optionally)
/// the eigenvector matrix in column-major order.
///
/// `mat` is row-major n×n. Overwrites it.
fn jacobi_eigen(mat: &mut [f64], n: usize, need_vectors: bool) -> (Vec<f64>, Vec<Vec<f64>>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    if n == 1 {
        let val = mat[0];
        let vecs = if need_vectors {
            vec![vec![1.0]]
        } else {
            Vec::new()
        };
        return (vec![val], vecs);
    }

    // Initialize eigenvector matrix to identity
    let mut v = vec![0.0_f64; n * n];
    if need_vectors {
        for i in 0..n {
            v[i * n + i] = 1.0;
        }
    }

    let max_sweeps = 100;
    for _ in 0..max_sweeps {
        // Find the off-diagonal element with largest absolute value
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

        // Compute rotation angle
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

        // Apply Jacobi rotation
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

        // Update eigenvectors
        if need_vectors {
            for i in 0..n {
                let vip = v[i * n + p];
                let viq = v[i * n + q];
                v[i * n + p] = cos * vip - sin * viq;
                v[i * n + q] = sin * vip + cos * viq;
            }
        }
    }

    // Extract eigenvalues (diagonal)
    let mut eigenvalues: Vec<f64> = (0..n).map(|i| mat[i * n + i]).collect();

    // Sort by decreasing eigenvalue
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        eigenvalues[b]
            .partial_cmp(&eigenvalues[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_vals: Vec<f64> = indices.iter().map(|&i| eigenvalues[i]).collect();

    let sorted_vecs = if need_vectors {
        indices
            .iter()
            .map(|&idx| {
                let mut col = vec![0.0_f64; n];
                for i in 0..n {
                    col[i] = v[i * n + idx];
                }
                col
            })
            .collect()
    } else {
        Vec::new()
    };

    eigenvalues = sorted_vals;
    (eigenvalues, sorted_vecs)
}

/// Compute the full adjacency eigenspectrum, sorted in decreasing order.
fn full_spectrum(graph: &Graph) -> Vec<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Vec::new();
    }
    let mut a = dense_adjacency(graph);
    let (vals, _) = jacobi_eigen(&mut a, n, false);
    vals
}

/// Full eigen decomposition (eigenvalues + eigenvectors).
fn full_decomposition(graph: &Graph) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = graph.vcount() as usize;
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut a = dense_adjacency(graph);
    jacobi_eigen(&mut a, n, true)
}

/// Compute the Estrada index of a graph.
///
/// `EE(G) = Σ_i exp(λ_i)` where `{λ_i}` are the adjacency eigenvalues.
///
/// For an empty graph (no vertices) returns `0.0`. For an edgeless graph
/// on `n` vertices returns `n` (since all eigenvalues are zero).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, estrada_index};
///
/// // K_3: eigenvalues {2, -1, -1} → EE = e² + 2·e⁻¹
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let ee = estrada_index(&g).unwrap();
/// let expected = (2.0_f64).exp() + 2.0 * (-1.0_f64).exp();
/// assert!((ee - expected).abs() < 0.01);
/// ```
pub fn estrada_index(graph: &Graph) -> IgraphResult<f64> {
    let spectrum = full_spectrum(graph);
    Ok(spectrum.iter().map(|&lam| lam.exp()).sum())
}

/// Compute the graph energy.
///
/// `E(G) = Σ_i |λ_i|` — the sum of absolute adjacency eigenvalues.
/// Originates from Hückel molecular orbital theory.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_energy};
///
/// // K_3: eigenvalues {2, -1, -1} → energy = 2 + 1 + 1 = 4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let e = graph_energy(&g).unwrap();
/// assert!((e - 4.0).abs() < 0.01);
/// ```
pub fn graph_energy(graph: &Graph) -> IgraphResult<f64> {
    let spectrum = full_spectrum(graph);
    Ok(spectrum.iter().map(|&lam| lam.abs()).sum())
}

/// Compute the spectral radius of a graph.
///
/// `ρ(G) = max_i |λ_i|` — the largest eigenvalue in absolute value.
/// For undirected graphs, this equals `λ_1` (the largest eigenvalue).
///
/// Returns `0.0` for an empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spectral_radius};
///
/// // K_3: eigenvalues {2, -1, -1} → spectral radius = 2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let rho = spectral_radius(&g).unwrap();
/// assert!((rho - 2.0).abs() < 0.01);
/// ```
pub fn spectral_radius(graph: &Graph) -> IgraphResult<f64> {
    let spectrum = full_spectrum(graph);
    Ok(spectrum
        .iter()
        .map(|&lam| lam.abs())
        .fold(0.0_f64, f64::max))
}

/// Compute the spectral gap of a graph.
///
/// `Δ = λ_1 - λ_2` — the difference between the largest and
/// second-largest eigenvalue. A large spectral gap implies good
/// expansion properties and rapid mixing of random walks.
///
/// Returns `0.0` if the graph has fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spectral_gap};
///
/// // K_3: eigenvalues {2, -1, -1} → gap = 2 - (-1) = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let gap = spectral_gap(&g).unwrap();
/// assert!((gap - 3.0).abs() < 0.01);
/// ```
pub fn spectral_gap(graph: &Graph) -> IgraphResult<f64> {
    let spectrum = full_spectrum(graph);
    if spectrum.len() < 2 {
        return Ok(0.0);
    }
    Ok(spectrum[0] - spectrum[1])
}

/// Compute the natural connectivity of a graph.
///
/// `λ̄ = ln(EE(G) / n) = ln((1/n) Σ_i exp(λ_i))`
///
/// A robust measure of network structural redundancy / fault tolerance.
/// Higher values indicate more alternative paths.
///
/// Returns `0.0` if the graph has no vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, natural_connectivity};
///
/// // K_3: EE = e² + 2e⁻¹ → λ̄ = ln(EE/3)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let nc = natural_connectivity(&g).unwrap();
/// let expected = ((2.0_f64).exp() + 2.0 * (-1.0_f64).exp()).ln() - (3.0_f64).ln();
/// assert!((nc - expected).abs() < 0.01);
/// ```
pub fn natural_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }
    let ee = estrada_index(graph)?;
    Ok((ee / n as f64).ln())
}

/// Compute subgraph centrality for all vertices.
///
/// `SC(v) = (e^A)_{vv} = Σ_j (φ_j(v))² · exp(λ_j)`
///
/// where `φ_j` is the `j`-th eigenvector and `λ_j` the corresponding
/// eigenvalue. Measures the participation of vertex `v` in all subgraphs
/// (closed walks), weighted exponentially by length.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, subgraph_centrality};
///
/// // K_3: all vertices are symmetric, so all SC values are equal
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let sc = subgraph_centrality(&g).unwrap();
/// assert!((sc[0] - sc[1]).abs() < 0.01);
/// assert!((sc[1] - sc[2]).abs() < 0.01);
/// ```
pub fn subgraph_centrality(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    let (eigenvalues, eigenvectors) = full_decomposition(graph);

    let mut sc = vec![0.0_f64; n];

    for (j, &lam) in eigenvalues.iter().enumerate() {
        let exp_lam = lam.exp();
        let phi = &eigenvectors[j];
        for v in 0..n {
            sc[v] += phi[v] * phi[v] * exp_lam;
        }
    }

    Ok(sc)
}

/// Compute the communicability matrix between all pairs of vertices.
///
/// `C(u,v) = (e^A)_{uv} = Σ_j φ_j(u) · φ_j(v) · exp(λ_j)`
///
/// Returns a flattened `n × n` matrix in row-major order. The diagonal
/// entries equal the subgraph centrality.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if the graph has more than
/// 10 000 vertices (the `n²` output would be impractical).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, communicability_matrix};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let c = communicability_matrix(&g).unwrap();
/// // Symmetric: C(0,1) == C(1,0)
/// assert!((c[0 * 3 + 1] - c[1 * 3 + 0]).abs() < 1e-10);
/// // Diagonal = subgraph centrality
/// assert!(c[0] > 0.0);
/// ```
pub fn communicability_matrix(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }

    if n > 10_000 {
        return Err(IgraphError::InvalidArgument(format!(
            "communicability_matrix: graph has {n} vertices; n² = {} entries would be impractical",
            (n as u64).saturating_mul(n as u64)
        )));
    }

    let (eigenvalues, eigenvectors) = full_decomposition(graph);

    let mut mat = vec![0.0_f64; n * n];

    for (j, &lam) in eigenvalues.iter().enumerate() {
        let exp_lam = lam.exp();
        let phi = &eigenvectors[j];
        for u in 0..n {
            for v in u..n {
                let contrib = phi[u] * phi[v] * exp_lam;
                mat[u * n + v] += contrib;
                if u != v {
                    mat[v * n + u] += contrib;
                }
            }
        }
    }

    Ok(mat)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
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

    // --- Estrada index ---

    #[test]
    fn estrada_empty() {
        let g = Graph::with_vertices(0);
        let ee = estrada_index(&g).unwrap();
        assert!(ee.abs() < 1e-10);
    }

    #[test]
    fn estrada_no_edges() {
        let g = Graph::with_vertices(5);
        let ee = estrada_index(&g).unwrap();
        // All eigenvalues zero → EE = 5 * e^0 = 5
        assert!((ee - 5.0).abs() < 0.1);
    }

    #[test]
    fn estrada_k3() {
        let g = k3();
        let ee = estrada_index(&g).unwrap();
        let expected = (2.0_f64).exp() + 2.0 * (-1.0_f64).exp();
        assert!((ee - expected).abs() < 0.05);
    }

    #[test]
    fn estrada_k4() {
        let g = k4();
        let ee = estrada_index(&g).unwrap();
        // K_4: eigenvalues {3, -1, -1, -1}
        let expected = (3.0_f64).exp() + 3.0 * (-1.0_f64).exp();
        assert!((ee - expected).abs() < 0.1);
    }

    #[test]
    fn estrada_positive() {
        let g = cycle4();
        let ee = estrada_index(&g).unwrap();
        assert!(ee > 0.0);
    }

    // --- Graph energy ---

    #[test]
    fn energy_empty() {
        let g = Graph::with_vertices(0);
        let e = graph_energy(&g).unwrap();
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn energy_no_edges() {
        let g = Graph::with_vertices(3);
        let e = graph_energy(&g).unwrap();
        assert!(e.abs() < 0.1);
    }

    #[test]
    fn energy_k3() {
        let g = k3();
        let e = graph_energy(&g).unwrap();
        // eigenvalues {2, -1, -1} → energy = 2+1+1 = 4
        assert!((e - 4.0).abs() < 0.1);
    }

    #[test]
    fn energy_k4() {
        let g = k4();
        let e = graph_energy(&g).unwrap();
        // eigenvalues {3, -1, -1, -1} → energy = 3+1+1+1 = 6
        assert!((e - 6.0).abs() < 0.1);
    }

    #[test]
    fn energy_nonneg() {
        let g = star4();
        let e = graph_energy(&g).unwrap();
        assert!(e >= -1e-10);
    }

    // --- Spectral radius ---

    #[test]
    fn radius_empty() {
        let g = Graph::with_vertices(0);
        let r = spectral_radius(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn radius_k3() {
        let g = k3();
        let r = spectral_radius(&g).unwrap();
        assert!((r - 2.0).abs() < 0.01);
    }

    #[test]
    fn radius_complete_n() {
        // K_n has spectral radius n-1
        let g = k4();
        let r = spectral_radius(&g).unwrap();
        assert!((r - 3.0).abs() < 0.01);
    }

    #[test]
    fn radius_path() {
        let g = path3();
        let r = spectral_radius(&g).unwrap();
        // P_3: eigenvalues are {√2, 0, -√2} → ρ = √2
        assert!((r - std::f64::consts::SQRT_2).abs() < 0.01);
    }

    // --- Spectral gap ---

    #[test]
    fn gap_empty() {
        let g = Graph::with_vertices(0);
        let gap = spectral_gap(&g).unwrap();
        assert!(gap.abs() < 1e-10);
    }

    #[test]
    fn gap_single() {
        let g = Graph::with_vertices(1);
        let gap = spectral_gap(&g).unwrap();
        assert!(gap.abs() < 1e-10);
    }

    #[test]
    fn gap_k3() {
        let g = k3();
        let gap = spectral_gap(&g).unwrap();
        // eigenvalues {2, -1, -1} → gap = 2 - (-1) = 3
        assert!((gap - 3.0).abs() < 0.1);
    }

    #[test]
    fn gap_complete_is_n() {
        // K_n: eigenvalues {n-1, -1, ..., -1} → gap = n
        let g = k4();
        let gap = spectral_gap(&g).unwrap();
        assert!((gap - 4.0).abs() < 0.1);
    }

    // --- Natural connectivity ---

    #[test]
    fn nc_empty() {
        let g = Graph::with_vertices(0);
        let nc = natural_connectivity(&g).unwrap();
        assert!(nc.abs() < 1e-10);
    }

    #[test]
    fn nc_no_edges() {
        let g = Graph::with_vertices(4);
        let nc = natural_connectivity(&g).unwrap();
        // EE = 4, nc = ln(4/4) = ln(1) = 0
        assert!(nc.abs() < 0.1);
    }

    #[test]
    fn nc_k3() {
        let g = k3();
        let nc = natural_connectivity(&g).unwrap();
        let expected = ((2.0_f64).exp() + 2.0 * (-1.0_f64).exp()).ln() - (3.0_f64).ln();
        assert!((nc - expected).abs() < 0.05);
    }

    #[test]
    fn nc_more_edges_higher() {
        // Adding edges should not decrease natural connectivity
        let p = path3();
        let k = k3();
        let nc_p = natural_connectivity(&p).unwrap();
        let nc_k = natural_connectivity(&k).unwrap();
        assert!(nc_k > nc_p - 0.01);
    }

    // --- Subgraph centrality ---

    #[test]
    fn sc_empty() {
        let g = Graph::with_vertices(0);
        let sc = subgraph_centrality(&g).unwrap();
        assert!(sc.is_empty());
    }

    #[test]
    fn sc_no_edges() {
        let g = Graph::with_vertices(3);
        let sc = subgraph_centrality(&g).unwrap();
        // (e^0)_{vv} = 1 for all v
        for &v in &sc {
            assert!((v - 1.0).abs() < 0.1);
        }
    }

    #[test]
    fn sc_k3_symmetric() {
        let g = k3();
        let sc = subgraph_centrality(&g).unwrap();
        // All vertices equivalent
        assert!((sc[0] - sc[1]).abs() < 0.01);
        assert!((sc[1] - sc[2]).abs() < 0.01);
    }

    #[test]
    fn sc_star_center_highest() {
        let g = star4();
        let sc = subgraph_centrality(&g).unwrap();
        // Center vertex (0) should have highest SC
        assert!(sc[0] > sc[1]);
        assert!(sc[0] > sc[2]);
        assert!(sc[0] > sc[3]);
    }

    #[test]
    fn sc_all_positive() {
        let g = cycle4();
        let sc = subgraph_centrality(&g).unwrap();
        for &v in &sc {
            assert!(v > 0.0);
        }
    }

    #[test]
    fn sc_sum_equals_estrada() {
        let g = k3();
        let sc = subgraph_centrality(&g).unwrap();
        let ee = estrada_index(&g).unwrap();
        let sc_sum: f64 = sc.iter().sum();
        assert!((sc_sum - ee).abs() < 0.1);
    }

    // --- Communicability matrix ---

    #[test]
    fn comm_empty() {
        let g = Graph::with_vertices(0);
        let c = communicability_matrix(&g).unwrap();
        assert!(c.is_empty());
    }

    #[test]
    fn comm_symmetric() {
        let g = path3();
        let c = communicability_matrix(&g).unwrap();
        let n = 3;
        for u in 0..n {
            for v in 0..n {
                assert!(
                    (c[u * n + v] - c[v * n + u]).abs() < 0.01,
                    "C({u},{v}) != C({v},{u})"
                );
            }
        }
    }

    #[test]
    fn comm_diagonal_is_sc() {
        let g = k3();
        let c = communicability_matrix(&g).unwrap();
        let sc = subgraph_centrality(&g).unwrap();
        let n = 3;
        for v in 0..n {
            assert!(
                (c[v * n + v] - sc[v]).abs() < 0.1,
                "Diagonal mismatch at vertex {v}"
            );
        }
    }

    #[test]
    fn comm_k3_all_positive() {
        let g = k3();
        let c = communicability_matrix(&g).unwrap();
        for &val in &c {
            assert!(val > 0.0);
        }
    }

    #[test]
    fn comm_neighbors_higher_than_distant() {
        let g = path3();
        let c = communicability_matrix(&g).unwrap();
        // C(0,1) > C(0,2) since 0 is adjacent to 1 but not 2
        assert!(c[1] > c[2]);
    }

    #[test]
    fn comm_too_large() {
        // Test the size guard
        assert!(communicability_matrix(&Graph::with_vertices(10_001)).is_err());
    }
}
