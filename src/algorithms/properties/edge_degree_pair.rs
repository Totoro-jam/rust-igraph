//! Edge degree-pair aggregates (ALGO-TR-086).
//!
//! Simple edge-additive indices that aggregate min, max, and product
//! functions of endpoint degrees:
//!
//! - **Edge degree min sum** `EDmin(G) = Σ_{(u,v)∈E} min(d(u),d(v))`
//! - **Edge degree max sum** `EDmax(G) = Σ_{(u,v)∈E} max(d(u),d(v))`
//! - **Edge degree log-product sum** `EDlp(G) = Σ_{(u,v)∈E} ln(d(u)·d(v))`
//!   (skips edges with a degree-0 endpoint)
//! - **Edge degree mean sum** `EDμ(G) = Σ_{(u,v)∈E} (d(u)+d(v))/2`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the edge degree min sum.
///
/// `EDmin(G) = Σ_{(u,v)∈E} min(d(u), d(v))`
///
/// Self-loops are skipped. For regular graphs with degree r:
/// `EDmin = m·r`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_min_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(edge_degree_min_sum(&g).unwrap(), 6);
/// ```
pub fn edge_degree_min_sum(graph: &Graph) -> IgraphResult<u64> {
    let mut result = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        result += du.min(dv) as u64;
    }

    Ok(result)
}

/// Compute the edge degree max sum.
///
/// `EDmax(G) = Σ_{(u,v)∈E} max(d(u), d(v))`
///
/// Self-loops are skipped. For regular graphs with degree r:
/// `EDmax = m·r`. Note that `EDmin + EDmax = M₁` (first Zagreb).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_max_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(edge_degree_max_sum(&g).unwrap(), 6);
/// ```
pub fn edge_degree_max_sum(graph: &Graph) -> IgraphResult<u64> {
    let mut result = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        result += du.max(dv) as u64;
    }

    Ok(result)
}

/// Compute the edge degree log-product sum.
///
/// `EDlp(G) = Σ_{(u,v)∈E} ln(d(u) · d(v))`
///
/// Equivalent to `Σ ln(d(u)) + ln(d(v))`. Self-loops and edges
/// with a degree-0 endpoint are skipped. Returns 0.0 for edgeless
/// graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_log_product};
///
/// // K_3: 3 edges, all (2,2) → 3·ln(4)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_log_product(&g).unwrap() - 3.0 * 4.0_f64.ln()).abs() < 1e-10);
/// ```
pub fn edge_degree_log_product(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        if du == 0 || dv == 0 {
            continue;
        }
        result += ((du * dv) as f64).ln();
    }

    Ok(result)
}

/// Compute the edge degree mean sum.
///
/// `EDμ(G) = Σ_{(u,v)∈E} (d(u) + d(v)) / 2`
///
/// This equals `M₁/2` (half the first Zagreb index). Self-loops are
/// skipped. For regular graphs with degree r: `EDμ = m·r`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_mean_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_mean_sum(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn edge_degree_mean_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        result += f64::midpoint(du, dv);
    }

    Ok(result)
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

    // --- edge_degree_min_sum ---

    #[test]
    fn edmin_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(edge_degree_min_sum(&g).unwrap(), 0);
    }

    #[test]
    fn edmin_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(edge_degree_min_sum(&g).unwrap(), 0);
    }

    #[test]
    fn edmin_regular() {
        // Regular degree r: each edge contributes r → m·r
        assert_eq!(edge_degree_min_sum(&k3()).unwrap(), 6); // 3·2
        assert_eq!(edge_degree_min_sum(&k4()).unwrap(), 18); // 6·3
        assert_eq!(edge_degree_min_sum(&cycle4()).unwrap(), 8); // 4·2
    }

    #[test]
    fn edmin_single_edge() {
        assert_eq!(edge_degree_min_sum(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn edmin_star5() {
        // 4 edges (4,1): min=1 each → 4
        assert_eq!(edge_degree_min_sum(&star5()).unwrap(), 4);
    }

    #[test]
    fn edmin_path3() {
        // 2 edges (1,2): min=1 each → 2
        assert_eq!(edge_degree_min_sum(&path3()).unwrap(), 2);
    }

    #[test]
    fn edmin_paw() {
        // (0,1)d=(2,2):2  (0,2)d=(2,3):2  (1,2)d=(2,3):2  (2,3)d=(3,1):1
        assert_eq!(edge_degree_min_sum(&paw()).unwrap(), 7);
    }

    // --- edge_degree_max_sum ---

    #[test]
    fn edmax_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(edge_degree_max_sum(&g).unwrap(), 0);
    }

    #[test]
    fn edmax_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(edge_degree_max_sum(&g).unwrap(), 0);
    }

    #[test]
    fn edmax_regular() {
        assert_eq!(edge_degree_max_sum(&k3()).unwrap(), 6);
        assert_eq!(edge_degree_max_sum(&k4()).unwrap(), 18);
        assert_eq!(edge_degree_max_sum(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn edmax_single_edge() {
        assert_eq!(edge_degree_max_sum(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn edmax_star5() {
        // 4 edges (4,1): max=4 each → 16
        assert_eq!(edge_degree_max_sum(&star5()).unwrap(), 16);
    }

    #[test]
    fn edmax_path3() {
        // 2 edges (1,2): max=2 each → 4
        assert_eq!(edge_degree_max_sum(&path3()).unwrap(), 4);
    }

    #[test]
    fn edmax_paw() {
        // (0,1):2  (0,2):3  (1,2):3  (2,3):3 → 11
        assert_eq!(edge_degree_max_sum(&paw()).unwrap(), 11);
    }

    // --- min + max = M1 ---

    #[test]
    fn min_plus_max_equals_m1() {
        // min(d(u),d(v)) + max(d(u),d(v)) = d(u)+d(v)
        // So EDmin + EDmax = Σ (d(u)+d(v)) = M₁ (first Zagreb)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let min_val = edge_degree_min_sum(g).unwrap();
            let max_val = edge_degree_max_sum(g).unwrap();
            let m1: u64 = g
                .edges()
                .filter(|&(u, v)| u != v)
                .map(|(u, v)| (g.degree(u).unwrap() + g.degree(v).unwrap()) as u64)
                .sum();
            assert_eq!(min_val + max_val, m1);
        }
    }

    // --- edge_degree_log_product ---

    #[test]
    fn edlp_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_log_product(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn edlp_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_log_product(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn edlp_k3() {
        // 3 edges (2,2): 3·ln(4)
        let expected = 3.0 * 4.0_f64.ln();
        assert!((edge_degree_log_product(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn edlp_single_edge() {
        // 1 edge (1,1): ln(1)=0
        assert!(edge_degree_log_product(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn edlp_star5() {
        // 4 edges (4,1): 4·ln(4)
        let expected = 4.0 * 4.0_f64.ln();
        assert!((edge_degree_log_product(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn edlp_path3() {
        // 2 edges (1,2): 2·ln(2)
        let expected = 2.0 * 2.0_f64.ln();
        assert!((edge_degree_log_product(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn edlp_paw() {
        // (0,1):ln(4)  (0,2):ln(6)  (1,2):ln(6)  (2,3):ln(3)
        let expected = 4.0_f64.ln() + 2.0 * 6.0_f64.ln() + 3.0_f64.ln();
        assert!((edge_degree_log_product(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_mean_sum ---

    #[test]
    fn edmean_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_mean_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn edmean_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_mean_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn edmean_regular() {
        // Regular degree r: each edge mean = r → m·r
        assert!((edge_degree_mean_sum(&k3()).unwrap() - 6.0).abs() < 1e-10);
        assert!((edge_degree_mean_sum(&k4()).unwrap() - 18.0).abs() < 1e-10);
        assert!((edge_degree_mean_sum(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn edmean_single_edge() {
        assert!((edge_degree_mean_sum(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn edmean_star5() {
        // 4 edges (4,1): mean=2.5 each → 10
        assert!((edge_degree_mean_sum(&star5()).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn edmean_paw() {
        // (0,1):2  (0,2):2.5  (1,2):2.5  (2,3):2 → 9.0
        assert!((edge_degree_mean_sum(&paw()).unwrap() - 9.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn edmin_le_edmean_le_edmax() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let min_val = edge_degree_min_sum(g).unwrap() as f64;
            let mean_val = edge_degree_mean_sum(g).unwrap();
            let max_val = edge_degree_max_sum(g).unwrap() as f64;
            assert!(min_val <= mean_val + 1e-10);
            assert!(mean_val <= max_val + 1e-10);
        }
    }

    #[test]
    fn edmean_is_half_m1() {
        // EDmean = M₁/2
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let min_val = edge_degree_min_sum(g).unwrap();
            let max_val = edge_degree_max_sum(g).unwrap();
            let m1 = (min_val + max_val) as f64;
            let mean_val = edge_degree_mean_sum(g).unwrap();
            assert!((mean_val - m1 / 2.0).abs() < 1e-10);
        }
    }
}
