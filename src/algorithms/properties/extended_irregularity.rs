//! Extended irregularity indices (ALGO-TR-078).
//!
//! Irregularity measures beyond the basic Albertson/sigma set:
//!
//! - **Bell index** `B(G) = Σ_v (d(v) - d̄)²/n` (population degree
//!   variance, i.e. `degree_variance` divided by n — but here as a
//!   standalone function from the Bell (1992) reference)
//! - **Collatz–Sinogowitz irregularity** `CS(G) = ρ(A) - 2m/n` where
//!   `ρ(A)` is the spectral radius and `2m/n` the average degree
//! - **IRL(G)** (irregularity by logarithm) `= Σ_{(u,v)∈E} |ln d(u) - ln d(v)|`
//! - **IRLU(G)** (irregularity by logarithm of ratio)
//!   `= Σ_{(u,v)∈E} |ln(d(u)/d(v))|` — same as IRL for positive degrees
//! - **Degree coefficient of variation** `CV(G) = σ/d̄` where `σ` is
//!   the degree standard deviation and `d̄` the average degree

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Bell index (degree population variance).
///
/// `B(G) = (1/n) Σ_v (d(v) - d̄)²`
///
/// where `d̄ = 2m/n` is the average degree. Equals 0 for regular graphs.
/// Returns 0.0 for graphs with fewer than 1 vertex.
///
/// This is the population variance of the degree sequence, as defined
/// by Bell (1992). It differs from `degree_variance` only if the latter
/// uses sample variance (n-1 denominator); our `degree_variance` already
/// uses population variance, so they are equivalent. This function is
/// provided for nomenclature completeness in chemical graph theory.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bell_index};
///
/// // K_3: d=(2,2,2), d̄=2, B=0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(bell_index(&g).unwrap().abs() < 1e-10);
/// ```
pub fn bell_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    let d_bar = 2.0 * m as f64 / n as f64;
    let mut sum = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as f64;
        let diff = d - d_bar;
        sum += diff * diff;
    }

    Ok(sum / n as f64)
}

/// Compute the Collatz–Sinogowitz irregularity.
///
/// `CS(G) = ρ(A) - 2m/n`
///
/// where `ρ(A)` is the spectral radius (largest eigenvalue of the
/// adjacency matrix) and `2m/n` is the average degree. Always ≥ 0 by
/// the Perron-Frobenius theorem for connected graphs.
///
/// Returns 0.0 for empty graphs (0 vertices or 0 edges).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, collatz_sinogowitz};
///
/// // K_3: spectral radius = 2, avg degree = 2, CS = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(collatz_sinogowitz(&g).unwrap().abs() < 1e-10);
/// ```
pub fn collatz_sinogowitz(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let d_bar = 2.0 * m as f64 / n as f64;

    let rho = crate::algorithms::properties::spectral_metrics::spectral_radius(graph)?;

    Ok((rho - d_bar).max(0.0))
}

/// Compute the IRL irregularity (irregularity by logarithm).
///
/// `IRL(G) = Σ_{(u,v)∈E} |ln d(u) - ln d(v)|`
///
/// Edges with a degree-0 endpoint are skipped. Self-loops are skipped.
/// Returns 0.0 for regular graphs and edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, irl_irregularity};
///
/// // K_3: all degrees 2, IRL = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(irl_irregularity(&g).unwrap().abs() < 1e-10);
/// ```
pub fn irl_irregularity(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if du <= 0.0 || dv <= 0.0 {
            continue;
        }
        result += (du.ln() - dv.ln()).abs();
    }

    Ok(result)
}

/// Compute the IRLU irregularity (irregularity by log-ratio).
///
/// `IRLU(G) = Σ_{(u,v)∈E} |ln(d(u)/d(v))|`
///
/// Numerically equivalent to `IRL(G)` for all positive degrees (by
/// logarithm properties), but provided for nomenclature completeness
/// in the literature. Edges with a degree-0 endpoint are skipped.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, irlu_irregularity};
///
/// // Star S_5: center d=4, leaves d=1
/// // 4 edges each |ln(4/1)| = ln(4) → 4·ln(4)
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((irlu_irregularity(&g).unwrap() - 4.0 * 4.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn irlu_irregularity(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if du <= 0.0 || dv <= 0.0 {
            continue;
        }
        result += (du / dv).ln().abs();
    }

    Ok(result)
}

/// Compute the degree coefficient of variation.
///
/// `CV(G) = σ/d̄` where `σ = √(B(G))` is the degree standard
/// deviation and `d̄ = 2m/n` is the average degree.
///
/// Returns 0.0 for regular graphs, edgeless graphs, or empty graphs.
/// The CV is undefined when `d̄ = 0` (edgeless); we return 0.0 in
/// that case.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_cv};
///
/// // K_3: regular, CV = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_cv(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_cv(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let d_bar = 2.0 * m as f64 / n as f64;
    let bell = bell_index(graph)?;
    let sigma = bell.sqrt();

    Ok(sigma / d_bar)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- bell_index ---

    #[test]
    fn bell_empty() {
        let g = Graph::with_vertices(0);
        assert!(bell_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bell_isolated() {
        let g = Graph::with_vertices(5);
        assert!(bell_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bell_regular_zero() {
        assert!(bell_index(&k3()).unwrap().abs() < 1e-10);
        assert!(bell_index(&k4()).unwrap().abs() < 1e-10);
        assert!(bell_index(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bell_single_edge() {
        // d=(1,1), d̄=1, B=0
        assert!(bell_index(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bell_path3() {
        // d=(1,2,1), d̄=4/3
        // (1-4/3)²+(2-4/3)²+(1-4/3)² = 2·(1/3)²+(2/3)² = 2/9+4/9 = 6/9 = 2/3
        // B = (2/3)/3 = 2/9
        assert!((bell_index(&path3()).unwrap() - 2.0 / 9.0).abs() < 1e-10);
    }

    #[test]
    fn bell_star5() {
        // d=(4,1,1,1,1), d̄=8/5=1.6
        // (4-1.6)²+4·(1-1.6)² = 5.76+4·0.36 = 5.76+1.44 = 7.2
        // B = 7.2/5 = 1.44
        assert!((bell_index(&star5()).unwrap() - 1.44).abs() < 1e-10);
    }

    #[test]
    fn bell_paw() {
        // d=(2,2,3,1), d̄=8/4=2
        // (2-2)²+(2-2)²+(3-2)²+(1-2)² = 0+0+1+1 = 2
        // B = 2/4 = 0.5
        assert!((bell_index(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    // --- collatz_sinogowitz ---

    #[test]
    fn cs_empty() {
        let g = Graph::with_vertices(0);
        assert!(collatz_sinogowitz(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cs_isolated() {
        let g = Graph::with_vertices(5);
        assert!(collatz_sinogowitz(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cs_regular_zero() {
        // Regular: spectral radius = degree, avg degree = degree → CS = 0
        assert!(collatz_sinogowitz(&k3()).unwrap().abs() < 1e-10);
        assert!(collatz_sinogowitz(&k4()).unwrap().abs() < 1e-10);
        assert!(collatz_sinogowitz(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cs_star5() {
        // S_5: spectral radius = √4 = 2, avg degree = 8/5 = 1.6
        // CS = 2 - 1.6 = 0.4
        assert!((collatz_sinogowitz(&star5()).unwrap() - 0.4).abs() < 1e-10);
    }

    #[test]
    fn cs_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(collatz_sinogowitz(g).unwrap() >= -1e-10);
        }
    }

    // --- irl_irregularity ---

    #[test]
    fn irl_empty() {
        let g = Graph::with_vertices(0);
        assert!(irl_irregularity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irl_isolated() {
        let g = Graph::with_vertices(5);
        assert!(irl_irregularity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irl_regular_zero() {
        assert!(irl_irregularity(&k3()).unwrap().abs() < 1e-10);
        assert!(irl_irregularity(&k4()).unwrap().abs() < 1e-10);
        assert!(irl_irregularity(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irl_single_edge() {
        // d=(1,1): |ln1-ln1| = 0
        assert!(irl_irregularity(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irl_star5() {
        // 4 edges (4,1): 4·|ln4-ln1| = 4·ln4
        let expected = 4.0 * 4.0_f64.ln();
        assert!((irl_irregularity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irl_path3() {
        // 2 edges: (1,2) and (2,1) → 2·|ln1-ln2| = 2·ln2
        let expected = 2.0 * 2.0_f64.ln();
        assert!((irl_irregularity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irl_paw() {
        // (0,1) d=(2,2): 0
        // (0,2) d=(2,3): |ln2-ln3|
        // (1,2) d=(2,3): |ln2-ln3|
        // (2,3) d=(3,1): |ln3-ln1| = ln3
        let expected = 2.0 * (3.0_f64.ln() - 2.0_f64.ln()) + 3.0_f64.ln();
        assert!((irl_irregularity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- irlu_irregularity ---

    #[test]
    fn irlu_empty() {
        let g = Graph::with_vertices(0);
        assert!(irlu_irregularity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irlu_regular_zero() {
        assert!(irlu_irregularity(&k3()).unwrap().abs() < 1e-10);
        assert!(irlu_irregularity(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irlu_equals_irl() {
        // For positive degrees, IRL = IRLU (by ln properties)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val_irl = irl_irregularity(g).unwrap();
            let val_irlu = irlu_irregularity(g).unwrap();
            assert!(
                (val_irl - val_irlu).abs() < 1e-10,
                "IRL={val_irl} IRLU={val_irlu}"
            );
        }
    }

    #[test]
    fn irlu_star5() {
        let expected = 4.0 * 4.0_f64.ln();
        assert!((irlu_irregularity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    // --- degree_cv ---

    #[test]
    fn cv_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_cv(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cv_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_cv(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cv_regular_zero() {
        assert!(degree_cv(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_cv(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_cv(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cv_single_edge() {
        // d=(1,1), d̄=1, σ=0, CV=0
        assert!(degree_cv(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cv_star5() {
        // B=1.44, σ=√1.44=1.2, d̄=1.6, CV=1.2/1.6=0.75
        assert!((degree_cv(&star5()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn cv_paw() {
        // B=0.5, σ=√0.5, d̄=2, CV=√0.5/2
        let expected = 0.5_f64.sqrt() / 2.0;
        assert!((degree_cv(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn cv_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_cv(g).unwrap() >= -1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn irl_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(irl_irregularity(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn bell_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(bell_index(g).unwrap() >= -1e-10);
        }
    }
}
