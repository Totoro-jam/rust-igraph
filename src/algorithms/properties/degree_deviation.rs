//! Degree deviation measures (ALGO-TR-084).
//!
//! Robust deviation and dispersion measures for the degree sequence,
//! complementing the variance-based `bell_index` / `degree_cv`:
//!
//! - **Mean absolute deviation** `MAD(G) = (1/n) Σ_v |d(v) − d̄|`
//! - **Median absolute deviation** `MedAD(G) = median_v |d(v) − median(d)|`
//! - **Degree entropy (nat)** `H(G) = −Σ_k p(k) ln p(k)` (natural-log)
//! - **Normalized degree entropy** `H_norm(G) = H(G) / ln(n)` → [0, 1]

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::HashMap;

/// Compute the mean absolute deviation of the degree sequence.
///
/// `MAD(G) = (1/n) Σ_v |d(v) - d̄|`
///
/// A robust dispersion measure less sensitive to outliers than
/// variance. Returns 0.0 for empty or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_mad};
///
/// // K_3: all degrees equal → MAD = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_mad(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_mad(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let mean = degrees.iter().sum::<f64>() / n as f64;

    let sum_abs: f64 = degrees.iter().map(|&d| (d - mean).abs()).sum();
    Ok(sum_abs / n as f64)
}

/// Compute the median absolute deviation of the degree sequence.
///
/// `MedAD(G) = median_v |d(v) - median(d)|`
///
/// The most robust dispersion measure — the median of absolute
/// deviations from the median. Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_median_ad};
///
/// // K_3: all degrees equal → MedAD = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_median_ad(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_median_ad(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let degrees = collect_degrees(graph)?;
    let med = median_of(&degrees);

    let mut abs_devs: Vec<f64> = degrees.iter().map(|&d| (d - med).abs()).collect();
    abs_devs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    Ok(median_sorted(&abs_devs))
}

/// Compute the Shannon entropy of the degree distribution (natural log).
///
/// `H(G) = -Σ_k p(k) ln(p(k))`
///
/// where `p(k) = n_k / n` is the fraction of vertices with degree k.
/// Returns 0.0 for empty graphs or when all vertices have the same
/// degree. Uses the natural logarithm (ln), complementing the
/// base-2 `degree_entropy` in `graph_entropy.rs`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_entropy_ln};
///
/// // K_3: all degree 2 → single class → H = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(degree_entropy_ln(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_entropy_ln(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut counts: HashMap<usize, usize> = HashMap::new();
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        *counts.entry(d).or_insert(0) += 1;
    }

    let nf = n as f64;
    let mut entropy = 0.0_f64;
    for &count in counts.values() {
        if count > 0 {
            let p = count as f64 / nf;
            entropy -= p * p.ln();
        }
    }

    Ok(entropy)
}

/// Compute the normalized degree entropy.
///
/// `H_norm(G) = H(G) / ln(n)`
///
/// Normalizes the natural-log degree entropy to [0, 1]. Returns 0.0
/// for graphs with fewer than 2 vertices. A value of 1.0 indicates
/// maximum diversity (all vertices have distinct degrees).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_entropy_normalized};
///
/// // K_4: all degrees equal → H_norm = 0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4),
/// ).unwrap();
/// assert!(degree_entropy_normalized(&g).unwrap().abs() < 1e-10);
/// ```
pub fn degree_entropy_normalized(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let h = degree_entropy_ln(graph)?;
    let h_max = (n as f64).ln();

    if h_max <= 0.0 {
        return Ok(0.0);
    }

    Ok(h / h_max)
}

fn collect_degrees(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)? as f64);
    }
    Ok(degrees)
}

fn median_of(vals: &[f64]) -> f64 {
    let mut sorted = vals.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    median_sorted(&sorted)
}

fn median_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        f64::midpoint(sorted[n / 2 - 1], sorted[n / 2])
    }
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

    // --- degree_mad ---

    #[test]
    fn mad_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_mad(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mad_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_mad(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mad_regular_zero() {
        assert!(degree_mad(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_mad(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_mad(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mad_single_edge() {
        assert!(degree_mad(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mad_star5() {
        // degrees: [4,1,1,1,1], mean=1.6
        // |d-mean|: [2.4,0.6,0.6,0.6,0.6]
        // MAD = (2.4+4·0.6)/5 = 4.8/5 = 0.96
        assert!((degree_mad(&star5()).unwrap() - 0.96).abs() < 1e-10);
    }

    #[test]
    fn mad_path3() {
        // degrees: [1,2,1], mean=4/3
        // |d-mean|: [1/3,2/3,1/3]
        // MAD = (4/3)/3 = 4/9
        let expected = 4.0 / 9.0;
        assert!((degree_mad(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mad_paw() {
        // degrees: [2,2,3,1], mean=2
        // |d-mean|: [0,0,1,1]
        // MAD = 2/4 = 0.5
        assert!((degree_mad(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mad_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_mad(g).unwrap() >= -1e-10);
        }
    }

    // --- degree_median_ad ---

    #[test]
    fn medad_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_median_ad(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_median_ad(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_regular_zero() {
        assert!(degree_median_ad(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_median_ad(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_median_ad(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_single_edge() {
        assert!(degree_median_ad(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_star5() {
        // degrees: [4,1,1,1,1], median=1
        // |d-median|: [3,0,0,0,0], sorted: [0,0,0,0,3]
        // median of abs devs = 0
        assert!(degree_median_ad(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_path3() {
        // degrees: [1,2,1], median=1
        // |d-median|: [0,1,0], sorted: [0,0,1]
        // MedAD = 0
        assert!(degree_median_ad(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn medad_paw() {
        // degrees: [2,2,3,1], median=2
        // |d-median|: [0,0,1,1], sorted: [0,0,1,1]
        // MedAD = (0+1)/2 = 0.5
        assert!((degree_median_ad(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn medad_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_median_ad(g).unwrap() >= -1e-10);
        }
    }

    // --- degree_entropy_ln ---

    #[test]
    fn ent_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_entropy_ln(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ent_isolated() {
        // all degree 0 → single class → H=0
        let g = Graph::with_vertices(5);
        assert!(degree_entropy_ln(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ent_regular_zero() {
        assert!(degree_entropy_ln(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_ln(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_ln(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ent_single_edge() {
        // both degree 1 → H=0
        assert!(degree_entropy_ln(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ent_star5() {
        // degrees: {4:1, 1:4}, p(4)=1/5, p(1)=4/5
        // H = -(1/5)ln(1/5) - (4/5)ln(4/5)
        let expected = -(0.2_f64 * 0.2_f64.ln() + 0.8 * 0.8_f64.ln());
        assert!((degree_entropy_ln(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ent_path3() {
        // degrees: {1:2, 2:1}, p(1)=2/3, p(2)=1/3
        // H = -(2/3)ln(2/3) - (1/3)ln(1/3)
        let p1: f64 = 2.0 / 3.0;
        let p2: f64 = 1.0 / 3.0;
        let expected = -(p1 * p1.ln() + p2 * p2.ln());
        assert!((degree_entropy_ln(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ent_paw() {
        // degrees: {2:2, 3:1, 1:1}, p(2)=1/2, p(3)=1/4, p(1)=1/4
        // H = -(1/2)ln(1/2) - (1/4)ln(1/4) - (1/4)ln(1/4)
        let expected = -(0.5_f64 * 0.5_f64.ln() + 2.0 * 0.25 * 0.25_f64.ln());
        assert!((degree_entropy_ln(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ent_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_entropy_ln(g).unwrap() >= -1e-10);
        }
    }

    // --- degree_entropy_normalized ---

    #[test]
    fn entnorm_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_entropy_normalized(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn entnorm_single() {
        let g = Graph::with_vertices(1);
        assert!(degree_entropy_normalized(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn entnorm_regular_zero() {
        assert!(degree_entropy_normalized(&k3()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_normalized(&k4()).unwrap().abs() < 1e-10);
        assert!(degree_entropy_normalized(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn entnorm_star5() {
        let h = degree_entropy_ln(&star5()).unwrap();
        let h_max = 5.0_f64.ln();
        let expected = h / h_max;
        assert!((degree_entropy_normalized(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn entnorm_paw() {
        let h = degree_entropy_ln(&paw()).unwrap();
        let h_max = 4.0_f64.ln();
        let expected = h / h_max;
        assert!((degree_entropy_normalized(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn entnorm_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val = degree_entropy_normalized(g).unwrap();
            assert!(val >= -1e-10);
            assert!(val <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn mad_le_maxdev() {
        // MAD ≤ max deviation always
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let mad_val = degree_mad(g).unwrap();
            let n = g.vcount() as usize;
            let mut degrees = Vec::with_capacity(n);
            for v in 0..n {
                degrees.push(g.degree(v as u32).unwrap() as f64);
            }
            let mean = degrees.iter().sum::<f64>() / n as f64;
            let max_dev: f64 = degrees
                .iter()
                .map(|&d| (d - mean).abs())
                .fold(0.0, f64::max);
            assert!(mad_val <= max_dev + 1e-10);
        }
    }

    #[test]
    fn medad_le_mad() {
        // MedAD ≤ MAD for well-behaved distributions (not always, but for our test cases)
        for g in &[k3(), k4(), cycle4()] {
            assert!(degree_median_ad(g).unwrap() <= degree_mad(g).unwrap() + 1e-10);
        }
    }
}
