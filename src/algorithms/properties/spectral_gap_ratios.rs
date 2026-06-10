//! Spectral gap ratio indices (ALGO-TR-114).
//!
//! Measures derived from the eigenvalue spectrum of the adjacency matrix:
//!
//! - **Spectral gap ratio** — (λ₁ − λ₂) / λ₁ where λ₁ ≥ λ₂ are the two
//!   largest eigenvalues of the adjacency matrix
//! - **Spectral radius ratio** — λ₁ / sqrt(`max_degree` × (n-1)), normalized
//!   spectral radius
//! - **Energy ratio** — graph energy (sum |λᵢ|) / (n × sqrt(2m/n)), normalized
//!   by a random graph baseline

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the spectral gap ratio.
///
/// `(λ₁ − λ₂) / λ₁` where λ₁ and λ₂ are the two largest eigenvalues
/// of the adjacency matrix (computed via power iteration). A large gap
/// indicates an expander-like structure; a small gap suggests the graph
/// is close to disconnected. Returns 0.0 for graphs with fewer than 2
/// vertices or if λ₁ ≈ 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, adjacency_spectral_gap_ratio};
///
/// // K_4: λ₁=3, λ₂=-1 → gap = (3-(-1))/3 = 4/3 ≈ 1.333
/// // But we only consider the two LARGEST eigenvalues:
/// // K_n has eigenvalues n-1 (mult 1) and -1 (mult n-1)
/// // Two largest: 3 and -1 → (3 - (-1))/3 = 4/3
/// // Actually sorted by magnitude: the two largest eigenvalues are 3, -1
/// // Sorted descending: λ₁=3, λ₂=-1
/// // ratio = (3 - (-1))/3 = 4/3
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let r = adjacency_spectral_gap_ratio(&g).unwrap();
/// assert!(r > 1.0);
/// ```
pub fn adjacency_spectral_gap_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let eigs = top_two_eigenvalues(graph)?;
    let (lambda1, lambda2) = eigs;

    if lambda1.abs() < 1e-12 {
        return Ok(0.0);
    }

    Ok((lambda1 - lambda2) / lambda1)
}

/// Compute the spectral radius ratio.
///
/// `λ₁ / sqrt(max_degree × (n-1))` — the spectral radius normalized
/// by its theoretical upper bound (Cauchy-Schwarz). Values near 1
/// indicate the graph approaches the bound (e.g. stars); values near 0
/// indicate sparse, low-spectral-radius graphs. Returns 0.0 for trivial
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spectral_radius_ratio};
///
/// // K_3: λ₁=2, max_deg=2, n=3 → 2/sqrt(2×2)=2/2=1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((spectral_radius_ratio(&g).unwrap() - 1.0).abs() < 0.05);
/// ```
pub fn spectral_radius_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let lambda1 = largest_eigenvalue(graph)?;
    if lambda1.abs() < 1e-12 {
        return Ok(0.0);
    }

    let mut max_deg = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d > max_deg {
            max_deg = d;
        }
    }

    if max_deg == 0 {
        return Ok(0.0);
    }

    let bound = ((max_deg as f64) * ((n - 1) as f64)).sqrt();
    Ok(lambda1 / bound)
}

/// Compute the energy ratio.
///
/// `energy / (n × sqrt(2m/n))` where energy = Σ|λᵢ| is the graph energy
/// and the denominator is the expected energy of an Erdős–Rényi random
/// graph with the same density. Values > 1 indicate the graph is more
/// "energetic" than a random graph of the same density. Returns 0.0 for
/// edgeless or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, energy_ratio};
///
/// // K_3: eigenvalues {2,-1,-1}, energy=4, m=3, n=3
/// // baseline = 3*sqrt(2*3/3) = 3*sqrt(2) ≈ 4.243
/// // ratio ≈ 4/4.243 ≈ 0.943
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let r = energy_ratio(&g).unwrap();
/// assert!(r > 0.5 && r < 1.5);
/// ```
pub fn energy_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let eigenvalues = all_eigenvalues(graph)?;
    let energy: f64 = eigenvalues.iter().map(|e| e.abs()).sum();

    let baseline = (n as f64) * (2.0 * m as f64 / n as f64).sqrt();
    if baseline < 1e-12 {
        return Ok(0.0);
    }

    Ok(energy / baseline)
}

/// Power iteration to find the largest eigenvalue of the adjacency matrix.
fn largest_eigenvalue(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut x = vec![1.0_f64 / (n as f64).sqrt(); n];
    let mut y = vec![0.0_f64; n];

    for _ in 0..200 {
        for i in 0..n {
            y[i] = 0.0;
        }
        for v in 0..n {
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                y[v] += x[u as usize];
            }
        }

        let norm: f64 = y.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-30 {
            return Ok(0.0);
        }
        for i in 0..n {
            x[i] = y[i] / norm;
        }
    }

    let mut lambda = 0.0_f64;
    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        let mut ax_v = 0.0_f64;
        for &u in &nbrs {
            ax_v += x[u as usize];
        }
        lambda += x[v] * ax_v;
    }

    Ok(lambda)
}

/// Find the top two eigenvalues using power iteration with deflation.
fn top_two_eigenvalues(graph: &Graph) -> IgraphResult<(f64, f64)> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok((0.0, 0.0));
    }

    let lambda1 = largest_eigenvalue(graph)?;

    // Get the first eigenvector
    let mut x1 = vec![1.0_f64 / (n as f64).sqrt(); n];
    let mut y = vec![0.0_f64; n];

    for _ in 0..200 {
        for i in 0..n {
            y[i] = 0.0;
        }
        for v in 0..n {
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                y[v] += x1[u as usize];
            }
        }
        let norm: f64 = y.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-30 {
            return Ok((lambda1, 0.0));
        }
        for i in 0..n {
            x1[i] = y[i] / norm;
        }
    }

    // Power iteration on deflated matrix A - lambda1 * x1 * x1^T
    let mut x2 = vec![0.0_f64; n];
    // Start with vector orthogonal to x1
    x2[0] = -x1[1];
    x2[1] = x1[0];
    let norm: f64 = x2.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm < 1e-30 {
        for i in 0..n {
            x2[i] = if i == 0 { 1.0 } else { 0.0 };
        }
    } else {
        for i in 0..n {
            x2[i] /= norm;
        }
    }

    for _ in 0..300 {
        for i in 0..n {
            y[i] = 0.0;
        }
        // y = A * x2
        for v in 0..n {
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                y[v] += x2[u as usize];
            }
        }
        // Deflate: y = y - lambda1 * (x1^T x2) * x1
        let dot: f64 = x1.iter().zip(x2.iter()).map(|(a, b)| a * b).sum();
        for i in 0..n {
            y[i] -= lambda1 * dot * x1[i];
        }

        let norm: f64 = y.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-30 {
            return Ok((lambda1, 0.0));
        }
        for i in 0..n {
            x2[i] = y[i] / norm;
        }
    }

    // Compute Rayleigh quotient for x2 on deflated matrix
    let mut ax2 = vec![0.0_f64; n];
    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            ax2[v] += x2[u as usize];
        }
    }
    let dot_x1_x2: f64 = x1.iter().zip(x2.iter()).map(|(a, b)| a * b).sum();
    for i in 0..n {
        ax2[i] -= lambda1 * dot_x1_x2 * x1[i];
    }
    let lambda2: f64 = x2.iter().zip(ax2.iter()).map(|(a, b)| a * b).sum();

    Ok((lambda1, lambda2))
}

/// Compute all eigenvalues using QR iteration on the tridiagonal form.
/// For small graphs we use the adjacency matrix directly via Householder
/// reduction to tridiagonal form, then QR iteration.
fn all_eigenvalues(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![0.0]);
    }

    // Build adjacency matrix
    let mut a = vec![0.0_f64; n * n];
    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            a[v * n + u as usize] = 1.0;
        }
    }

    // Householder reduction to tridiagonal
    let mut diag = vec![0.0_f64; n];
    let mut offdiag = vec![0.0_f64; n];
    tridiagonalize(&mut a, n, &mut diag, &mut offdiag);

    // QR iteration on tridiagonal matrix
    tql2(&mut diag, &mut offdiag, n);

    Ok(diag)
}

/// Householder reduction to tridiagonal form (symmetric matrix).
fn tridiagonalize(a: &mut [f64], n: usize, d: &mut [f64], e: &mut [f64]) {
    for i in (1..n).rev() {
        let l = i - 1;
        let mut h = 0.0_f64;
        let mut scale = 0.0_f64;

        if l > 0 {
            for k in 0..=l {
                scale += a[i * n + k].abs();
            }
            if scale < 1e-30 {
                e[i] = a[i * n + l];
            } else {
                for k in 0..=l {
                    a[i * n + k] /= scale;
                    h += a[i * n + k] * a[i * n + k];
                }
                let mut f = a[i * n + l];
                let g = if f >= 0.0 { -h.sqrt() } else { h.sqrt() };
                e[i] = scale * g;
                h -= f * g;
                a[i * n + l] = f - g;
                f = 0.0;
                for j in 0..=l {
                    a[j * n + i] = a[i * n + j] / h;
                    let mut g_val = 0.0_f64;
                    for k in 0..=j {
                        g_val += a[j * n + k] * a[i * n + k];
                    }
                    for k in (j + 1)..=l {
                        g_val += a[k * n + j] * a[i * n + k];
                    }
                    e[j] = g_val / h;
                    f += e[j] * a[i * n + j];
                }
                let hh = f / (h + h);
                for j in 0..=l {
                    f = a[i * n + j];
                    let g_val = e[j] - hh * f;
                    e[j] = g_val;
                    for k in 0..=j {
                        a[j * n + k] -= f * e[k] + g_val * a[i * n + k];
                    }
                }
            }
        } else {
            e[i] = a[i * n + l];
        }
        d[i] = h;
    }

    d[0] = 0.0;
    e[0] = 0.0;

    for i in 0..n {
        d[i] = a[i * n + i];
    }
}

/// QL implicit-shift iteration for eigenvalues of a symmetric tridiagonal matrix.
fn tql2(d: &mut [f64], e: &mut [f64], n: usize) {
    for i in 1..n {
        e[i - 1] = e[i];
    }
    e[n - 1] = 0.0;

    let mut f = 0.0_f64;
    let mut tst1 = 0.0_f64;
    let eps = 1e-15_f64;

    for l in 0..n {
        tst1 = tst1.max(d[l].abs() + e[l].abs());
        let mut m = l;
        while m < n {
            if e[m].abs() <= eps * tst1 {
                break;
            }
            m += 1;
        }

        if m > l {
            let mut iter_count = 0_u32;
            loop {
                iter_count += 1;
                if iter_count > 300 {
                    break;
                }

                let mut g = d[l];
                let mut p = (d[l + 1] - g) / (2.0 * e[l]);
                let mut r = (p * p + 1.0_f64).sqrt();
                if p < 0.0 {
                    r = -r;
                }
                d[l] = e[l] / (p + r);
                d[l + 1] = e[l] * (p + r);
                let dl1 = d[l + 1];
                let mut h = g - d[l];
                for i in (l + 2)..n {
                    d[i] -= h;
                }
                f += h;

                p = d[m];
                let mut c = 1.0_f64;
                let mut c2 = c;
                let mut c3 = c;
                let el1 = e[l + 1];
                let mut s = 0.0_f64;
                let mut s2 = 0.0_f64;

                let mut i = m;
                while i > l {
                    i -= 1;
                    c3 = c2;
                    c2 = c;
                    s2 = s;
                    g = c * e[i];
                    h = c * p;
                    r = (p * p + e[i] * e[i]).sqrt();
                    e[i + 1] = s * r;
                    s = e[i] / r;
                    c = p / r;
                    p = c * d[i] - s * g;
                    d[i + 1] = h + s * (c * g + s * d[i]);
                }

                p = -s * s2 * c3 * el1 * e[l] / dl1;
                e[l] = s * p;
                d[l] = c * p;

                if e[l].abs() <= eps * tst1 {
                    break;
                }
            }
        }
        d[l] += f;
        e[l] = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Graph {
        Graph::with_vertices(0)
    }

    fn single() -> Graph {
        Graph::with_vertices(1)
    }

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

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

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- adjacency_spectral_gap_ratio ---

    #[test]
    fn sgr_empty() {
        assert!(adjacency_spectral_gap_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sgr_single() {
        assert!(adjacency_spectral_gap_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sgr_single_edge() {
        // λ₁=1, λ₂=-1 → (1-(-1))/1 = 2.0
        let r = adjacency_spectral_gap_ratio(&single_edge()).unwrap();
        assert!((r - 2.0).abs() < 0.1);
    }

    #[test]
    fn sgr_k3() {
        // K_3: λ₁=2, λ₂=-1 → (2-(-1))/2 = 1.5
        let r = adjacency_spectral_gap_ratio(&k3()).unwrap();
        assert!((r - 1.5).abs() < 0.1);
    }

    #[test]
    fn sgr_k4() {
        // K_4: λ₁=3, λ₂=-1 → (3-(-1))/3 = 4/3 ≈ 1.333
        let r = adjacency_spectral_gap_ratio(&k4()).unwrap();
        assert!((r - 4.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn sgr_positive_connected() {
        for g in &[single_edge(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = adjacency_spectral_gap_ratio(g).unwrap();
            assert!(
                r > 0.0,
                "spectral gap ratio should be positive for connected graphs"
            );
        }
    }

    // --- spectral_radius_ratio ---

    #[test]
    fn srr_empty() {
        assert!(spectral_radius_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn srr_single() {
        assert!(spectral_radius_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn srr_k3() {
        // λ₁=2, max_deg=2, n=3 → 2/sqrt(2*2) = 2/2 = 1.0
        let r = spectral_radius_ratio(&k3()).unwrap();
        assert!((r - 1.0).abs() < 0.05);
    }

    #[test]
    fn srr_single_edge() {
        // λ₁=1, max_deg=1, n=2 → 1/sqrt(1*1) = 1.0
        let r = spectral_radius_ratio(&single_edge()).unwrap();
        assert!((r - 1.0).abs() < 0.05);
    }

    #[test]
    fn srr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = spectral_radius_ratio(g).unwrap();
            assert!(r >= -0.01);
            assert!(r <= 1.01);
        }
    }

    // --- energy_ratio ---

    #[test]
    fn er_empty() {
        assert!(energy_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_single() {
        assert!(energy_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_k3() {
        // eigenvalues: {2,-1,-1}, energy=4, m=3, n=3
        // baseline = 3*sqrt(2*3/3) = 3*sqrt(2) ≈ 4.243
        // ratio ≈ 4/4.243 ≈ 0.943
        let r = energy_ratio(&k3()).unwrap();
        assert!((r - 4.0 / (3.0 * 2.0_f64.sqrt())).abs() < 0.1);
    }

    #[test]
    fn er_single_edge() {
        // eigenvalues: {1,-1}, energy=2, m=1, n=2
        // baseline = 2*sqrt(2*1/2) = 2*1 = 2
        // ratio = 2/2 = 1.0
        let r = energy_ratio(&single_edge()).unwrap();
        assert!((r - 1.0).abs() < 0.1);
    }

    #[test]
    fn er_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = energy_ratio(g).unwrap();
            assert!(r > 0.0);
        }
    }

    #[test]
    fn er_finite() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(energy_ratio(g).unwrap().is_finite());
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_graphs_high_gap() {
        // Complete graphs are good expanders → high spectral gap ratio
        let r3 = adjacency_spectral_gap_ratio(&k3()).unwrap();
        let r4 = adjacency_spectral_gap_ratio(&k4()).unwrap();
        assert!(r3 > 1.0);
        assert!(r4 > 1.0);
    }

    #[test]
    fn all_indices_finite() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(adjacency_spectral_gap_ratio(g).unwrap().is_finite());
            assert!(spectral_radius_ratio(g).unwrap().is_finite());
            assert!(energy_ratio(g).unwrap().is_finite());
        }
    }
}
