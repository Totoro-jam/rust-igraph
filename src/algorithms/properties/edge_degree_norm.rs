//! Edge degree normalized indices (ALGO-TR-090).
//!
//! Edge-level indices that normalize endpoint degrees:
//!
//! - **Inverse sum index** `Σ 1/(d(u)+d(v))` — reciprocal of degree sum
//! - **Difference ratio** `Σ |d(u)-d(v)|/(d(u)+d(v))` — normalized asymmetry
//! - **Sørensen index** `Σ 2·min(d(u),d(v))/(d(u)+d(v))` — per-edge similarity
//! - **Product-sum ratio** `Σ d(u)·d(v)/(d(u)+d(v))²` — product/sum² ratio

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the inverse degree-sum index over edges.
///
/// `Σ_{(u,v)∈E} 1 / (d(u) + d(v))`
///
/// Self-loops and edges with a zero-degree endpoint are skipped.
/// Related to the inverse sum indeg index (ISD) but sums over
/// edges rather than vertex neighborhoods.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_inverse_degree_sum};
///
/// // K_3: 3 edges, each (2,2) → 3·(1/4) = 0.75
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_inverse_degree_sum(&g).unwrap() - 0.75).abs() < 1e-10);
/// ```
pub fn edge_inverse_degree_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s == 0.0 {
            continue;
        }
        result += 1.0 / s;
    }

    Ok(result)
}

/// Compute the degree difference ratio over edges.
///
/// `Σ_{(u,v)∈E} |d(u) - d(v)| / (d(u) + d(v))`
///
/// Each edge contributes a value in [0, 1). Returns 0.0 for regular
/// or edgeless graphs. Self-loops and zero-degree endpoints are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_diff_ratio};
///
/// // K_3: all (2,2) → |0|/4 = 0 per edge → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(edge_degree_diff_ratio(&g).unwrap().abs() < 1e-10);
/// ```
pub fn edge_degree_diff_ratio(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        let s = du + dv;
        if s == 0 {
            continue;
        }
        result += du.abs_diff(dv) as f64 / s as f64;
    }

    Ok(result)
}

/// Compute the Sørensen edge degree index.
///
/// `Σ_{(u,v)∈E} 2·min(d(u),d(v)) / (d(u) + d(v))`
///
/// Each edge contributes a value in (0, 1]. Equals m for regular
/// graphs. Self-loops and zero-degree endpoints are skipped.
/// Note: `sorensen + diff_ratio = m` (for non-loop edges with
/// non-zero degree sum).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_sorensen};
///
/// // K_3: all (2,2) → 2·2/4 = 1 per edge → 3.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_sorensen(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn edge_degree_sorensen(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        let s = du + dv;
        if s == 0 {
            continue;
        }
        let min_d = du.min(dv);
        result += 2.0 * min_d as f64 / s as f64;
    }

    Ok(result)
}

/// Compute the product-sum ratio index over edges.
///
/// `Σ_{(u,v)∈E} d(u)·d(v) / (d(u) + d(v))²`
///
/// Each edge contributes a value in (0, 0.25]. Equals `m/4` for
/// regular graphs. Self-loops and zero-degree endpoints are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_product_ratio};
///
/// // K_3: all (2,2) → 4/16 = 0.25 per edge → 0.75
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_product_ratio(&g).unwrap() - 0.75).abs() < 1e-10);
/// ```
pub fn edge_degree_product_ratio(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s == 0.0 {
            continue;
        }
        result += du * dv / (s * s);
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

    // --- edge_inverse_degree_sum ---

    #[test]
    fn inv_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_inverse_degree_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn inv_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_inverse_degree_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn inv_k3() {
        // 3·(1/4) = 0.75
        assert!((edge_inverse_degree_sum(&k3()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn inv_k4() {
        // 6·(1/6) = 1.0
        assert!((edge_inverse_degree_sum(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn inv_single_edge() {
        // 1/(1+1) = 0.5
        assert!((edge_inverse_degree_sum(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn inv_star5() {
        // 4·(1/(4+1)) = 4/5 = 0.8
        assert!((edge_inverse_degree_sum(&star5()).unwrap() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn inv_path3() {
        // 2·(1/(1+2)) = 2/3
        let expected = 2.0 / 3.0;
        assert!((edge_inverse_degree_sum(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn inv_paw() {
        // (0,1):1/4  (0,2):1/5  (1,2):1/5  (2,3):1/4
        let expected = 0.25 + 0.2 + 0.2 + 0.25;
        assert!((edge_inverse_degree_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_diff_ratio ---

    #[test]
    fn diff_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_diff_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn diff_regular() {
        // Regular: all diffs = 0
        assert!(edge_degree_diff_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(edge_degree_diff_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(edge_degree_diff_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn diff_single_edge() {
        // (1,1) → |0|/2 = 0
        assert!(edge_degree_diff_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn diff_star5() {
        // 4 edges (4,1): |3|/5 = 3/5 each → 12/5 = 2.4
        assert!((edge_degree_diff_ratio(&star5()).unwrap() - 2.4).abs() < 1e-10);
    }

    #[test]
    fn diff_path3() {
        // 2 edges (1,2): |1|/3 each → 2/3
        let expected = 2.0 / 3.0;
        assert!((edge_degree_diff_ratio(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn diff_paw() {
        // (0,1):|0|/4=0  (0,2):|1|/5=0.2  (1,2):|1|/5=0.2  (2,3):|2|/4=0.5
        let expected = 0.0 + 0.2 + 0.2 + 0.5;
        assert!((edge_degree_diff_ratio(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_sorensen ---

    #[test]
    fn sor_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_sorensen(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sor_regular() {
        // Regular: 2·r/(2r) = 1 per edge → m
        assert!((edge_degree_sorensen(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((edge_degree_sorensen(&k4()).unwrap() - 6.0).abs() < 1e-10);
        assert!((edge_degree_sorensen(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn sor_single_edge() {
        // 2·1/2 = 1
        assert!((edge_degree_sorensen(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sor_star5() {
        // 4·(2·1/5) = 8/5 = 1.6
        assert!((edge_degree_sorensen(&star5()).unwrap() - 1.6).abs() < 1e-10);
    }

    #[test]
    fn sor_path3() {
        // 2·(2·1/3) = 4/3
        let expected = 4.0 / 3.0;
        assert!((edge_degree_sorensen(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sor_paw() {
        // (0,1):4/4=1  (0,2):4/5=0.8  (1,2):4/5=0.8  (2,3):2/4=0.5
        let expected = 1.0 + 0.8 + 0.8 + 0.5;
        assert!((edge_degree_sorensen(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_product_ratio ---

    #[test]
    fn pr_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_product_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn pr_regular() {
        // r²/(2r)² = 1/4 per edge → m/4
        assert!((edge_degree_product_ratio(&k3()).unwrap() - 0.75).abs() < 1e-10);
        assert!((edge_degree_product_ratio(&k4()).unwrap() - 1.5).abs() < 1e-10);
        assert!((edge_degree_product_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pr_single_edge() {
        // 1·1/(2)² = 1/4 = 0.25
        assert!((edge_degree_product_ratio(&single_edge()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn pr_star5() {
        // 4·(4·1/25) = 16/25 = 0.64
        assert!((edge_degree_product_ratio(&star5()).unwrap() - 0.64).abs() < 1e-10);
    }

    #[test]
    fn pr_path3() {
        // 2·(1·2/9) = 4/9
        let expected = 4.0 / 9.0;
        assert!((edge_degree_product_ratio(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn pr_paw() {
        // (0,1):4/16=0.25  (0,2):6/25=0.24  (1,2):6/25=0.24  (2,3):3/16=0.1875
        let expected = 0.25 + 6.0 / 25.0 + 6.0 / 25.0 + 3.0 / 16.0;
        assert!((edge_degree_product_ratio(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn sorensen_plus_diff_equals_m() {
        // 2min/(d(u)+d(v)) + |d(u)-d(v)|/(d(u)+d(v)) = (2min + |diff|)/(sum)
        // = (2min + max - min)/(sum) = (min + max)/(sum) = 1
        // So sorensen + diff_ratio = m (non-loop edges with nonzero sum)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let sor = edge_degree_sorensen(g).unwrap();
            let diff = edge_degree_diff_ratio(g).unwrap();
            let m = g.edges().filter(|&(u, v)| u != v).count() as f64;
            assert!(
                (sor + diff - m).abs() < 1e-10,
                "sorensen({sor}) + diff({diff}) != m({m})"
            );
        }
    }

    #[test]
    fn product_ratio_le_quarter_m() {
        // d(u)·d(v)/(d(u)+d(v))² ≤ 1/4 by AM-GM
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let pr = edge_degree_product_ratio(g).unwrap();
            let m = g.edges().filter(|&(u, v)| u != v).count() as f64;
            assert!(pr <= m / 4.0 + 1e-10);
        }
    }

    #[test]
    fn diff_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(edge_degree_diff_ratio(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn inv_positive_for_nonempty() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(edge_inverse_degree_sum(g).unwrap() > 0.0);
        }
    }
}
