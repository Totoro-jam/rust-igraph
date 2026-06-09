//! Degree distribution shape measures (ALGO-TR-085).
//!
//! Structural descriptors of the degree-frequency profile:
//!
//! - **Degree mode** — most frequent degree value (lowest if tied)
//! - **Degree concentration** — fraction of vertices with mode degree
//! - **Degree diversity** — count of distinct degree values
//! - **Hub dominance** `HD(G) = d_max / (2m)` — max-degree share of total

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::HashMap;

/// Compute the mode of the degree sequence.
///
/// Returns the most frequent degree value. If multiple degrees share
/// the highest frequency, the smallest is returned. Returns 0 for
/// empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_mode};
///
/// // Star S_5: degrees [4,1,1,1,1] → mode = 1
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert_eq!(degree_mode(&g).unwrap(), 1);
/// ```
pub fn degree_mode(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let counts = degree_counts(graph)?;

    let mut best_deg = 0_usize;
    let mut best_count = 0_usize;
    for (&deg, &count) in &counts {
        if count > best_count || (count == best_count && deg < best_deg) {
            best_deg = deg;
            best_count = count;
        }
    }

    Ok(best_deg)
}

/// Compute the degree concentration.
///
/// Returns the fraction of vertices that have the mode degree.
/// Values close to 1.0 indicate a nearly regular graph.
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_concentration};
///
/// // K_3: all degree 2 → concentration = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_concentration(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn degree_concentration(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let counts = degree_counts(graph)?;
    let max_count = counts.values().copied().max().unwrap_or(0);

    Ok(max_count as f64 / n as f64)
}

/// Compute the degree diversity (number of distinct degree values).
///
/// Returns the count of distinct degrees in the graph.
/// Regular graphs have diversity 1. Returns 0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_diversity};
///
/// // Paw: degrees {1,2,2,3} → 3 distinct values
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(degree_diversity(&g).unwrap(), 3);
/// ```
pub fn degree_diversity(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let counts = degree_counts(graph)?;
    Ok(counts.len())
}

/// Compute the hub dominance.
///
/// `HD(G) = d_max / (2m)` — the fraction of total degree held by
/// the maximum-degree vertex. Returns 0.0 for edgeless or empty
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hub_dominance};
///
/// // Star S_5: d_max=4, 2m=8 → HD = 0.5
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((hub_dominance(&g).unwrap() - 0.5).abs() < 1e-10);
/// ```
pub fn hub_dominance(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut d_max = 0_usize;
    let mut total = 0_usize;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d > d_max {
            d_max = d;
        }
        total += d;
    }

    if total == 0 {
        return Ok(0.0);
    }

    Ok(d_max as f64 / total as f64)
}

fn degree_counts(graph: &Graph) -> IgraphResult<HashMap<usize, usize>> {
    let n = graph.vcount() as usize;
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        *counts.entry(d).or_insert(0) += 1;
    }
    Ok(counts)
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

    // --- degree_mode ---

    #[test]
    fn mode_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_mode(&g).unwrap(), 0);
    }

    #[test]
    fn mode_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_mode(&g).unwrap(), 0);
    }

    #[test]
    fn mode_k3() {
        assert_eq!(degree_mode(&k3()).unwrap(), 2);
    }

    #[test]
    fn mode_k4() {
        assert_eq!(degree_mode(&k4()).unwrap(), 3);
    }

    #[test]
    fn mode_single_edge() {
        assert_eq!(degree_mode(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn mode_star5() {
        // 4 vertices with degree 1
        assert_eq!(degree_mode(&star5()).unwrap(), 1);
    }

    #[test]
    fn mode_path3() {
        // [1,2,1] → mode=1
        assert_eq!(degree_mode(&path3()).unwrap(), 1);
    }

    #[test]
    fn mode_paw() {
        // [2,2,3,1] → mode=2
        assert_eq!(degree_mode(&paw()).unwrap(), 2);
    }

    // --- degree_concentration ---

    #[test]
    fn conc_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_concentration(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn conc_regular_one() {
        assert!((degree_concentration(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_concentration(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((degree_concentration(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn conc_single_edge() {
        assert!((degree_concentration(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn conc_star5() {
        // 4/5 = 0.8
        assert!((degree_concentration(&star5()).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn conc_path3() {
        // mode=1, count=2, n=3 → 2/3
        let expected = 2.0 / 3.0;
        assert!((degree_concentration(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn conc_paw() {
        // mode=2, count=2, n=4 → 0.5
        assert!((degree_concentration(&paw()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn conc_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val = degree_concentration(g).unwrap();
            assert!(val >= -1e-10);
            assert!(val <= 1.0 + 1e-10);
        }
    }

    // --- degree_diversity ---

    #[test]
    fn div_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_diversity(&g).unwrap(), 0);
    }

    #[test]
    fn div_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(degree_diversity(&g).unwrap(), 1);
    }

    #[test]
    fn div_regular_one() {
        assert_eq!(degree_diversity(&k3()).unwrap(), 1);
        assert_eq!(degree_diversity(&k4()).unwrap(), 1);
        assert_eq!(degree_diversity(&cycle4()).unwrap(), 1);
    }

    #[test]
    fn div_single_edge() {
        assert_eq!(degree_diversity(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn div_star5() {
        // {1,4} → 2
        assert_eq!(degree_diversity(&star5()).unwrap(), 2);
    }

    #[test]
    fn div_path3() {
        // {1,2} → 2
        assert_eq!(degree_diversity(&path3()).unwrap(), 2);
    }

    #[test]
    fn div_paw() {
        // {1,2,3} → 3
        assert_eq!(degree_diversity(&paw()).unwrap(), 3);
    }

    // --- hub_dominance ---

    #[test]
    fn hub_empty() {
        let g = Graph::with_vertices(0);
        assert!(hub_dominance(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hub_isolated() {
        let g = Graph::with_vertices(5);
        assert!(hub_dominance(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn hub_regular() {
        // d_max = d = 2m/n → HD = d/(nd) = 1/n
        let val = hub_dominance(&k3()).unwrap();
        assert!((val - 1.0 / 3.0).abs() < 1e-10);

        let val = hub_dominance(&k4()).unwrap();
        assert!((val - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn hub_single_edge() {
        // d_max=1, 2m=2 → 0.5
        assert!((hub_dominance(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hub_star5() {
        // d_max=4, 2m=8 → 0.5
        assert!((hub_dominance(&star5()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hub_path3() {
        // d_max=2, 2m=4 → 0.5
        assert!((hub_dominance(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn hub_paw() {
        // d_max=3, 2m=8 → 3/8 = 0.375
        assert!((hub_dominance(&paw()).unwrap() - 0.375).abs() < 1e-10);
    }

    #[test]
    fn hub_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val = hub_dominance(g).unwrap();
            assert!(val >= -1e-10);
            assert!(val <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn diversity_le_n() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_diversity(g).unwrap() <= g.vcount() as usize);
        }
    }

    #[test]
    fn conc_ge_1_over_n() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let n = f64::from(g.vcount());
            assert!(degree_concentration(g).unwrap() >= 1.0 / n - 1e-10);
        }
    }
}
