//! Edge degree mean-type indices (ALGO-TR-087).
//!
//! Edge-level aggregates using harmonic, geometric, ratio, and RMS
//! functions of endpoint degrees:
//!
//! - **Harmonic sum** `Σ 2·d(u)·d(v) / (d(u)+d(v))` — harmonic mean per edge
//! - **Geometric sum** `Σ √(d(u)·d(v))` — geometric mean per edge
//! - **Ratio sum** `Σ min(d(u),d(v)) / max(d(u),d(v))` — edge regularity
//! - **Endpoint RMS** `√(Σ (d(u)²+d(v)²) / (2m))` — root-mean-square

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the edge degree harmonic sum.
///
/// `Σ_{(u,v)∈E} 2·d(u)·d(v) / (d(u)+d(v))`
///
/// The harmonic mean of endpoint degrees, summed over all edges.
/// Self-loops and edges with a degree-0 endpoint are skipped.
/// Returns 0.0 for edgeless graphs. For regular graphs with
/// degree r: `H = m·r`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_harmonic_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_harmonic_sum(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn edge_degree_harmonic_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let sum = du + dv;
        if sum == 0.0 {
            continue;
        }
        result += 2.0 * du * dv / sum;
    }

    Ok(result)
}

/// Compute the edge degree geometric sum.
///
/// `Σ_{(u,v)∈E} √(d(u)·d(v))`
///
/// The geometric mean of endpoint degrees, summed over all edges.
/// Self-loops are skipped. Related to the Randić index (which sums
/// the *reciprocal* √(d(u)·d(v))). For regular graphs with
/// degree r: `G = m·r`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_geometric_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_geometric_sum(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn edge_degree_geometric_sum(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        result += (du * dv).sqrt();
    }

    Ok(result)
}

/// Compute the edge degree ratio sum.
///
/// `Σ_{(u,v)∈E} min(d(u),d(v)) / max(d(u),d(v))`
///
/// Each edge contributes a value in (0, 1]. Equals m for regular
/// graphs. Self-loops and edges with a degree-0 endpoint are
/// skipped. Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_ratio_sum};
///
/// // K_3: 3 edges, all (2,2) → 3·1.0 = 3.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_ratio_sum(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn edge_degree_ratio_sum(graph: &Graph) -> IgraphResult<f64> {
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
        let min_d = du.min(dv) as f64;
        let max_d = du.max(dv) as f64;
        result += min_d / max_d;
    }

    Ok(result)
}

/// Compute the edge endpoint degree RMS.
///
/// `√( Σ_{(u,v)∈E} (d(u)² + d(v)²) / (2m) )`
///
/// Root-mean-square of endpoint degrees over all edge endpoints.
/// Self-loops are skipped (m = non-loop edge count). Returns 0.0
/// for edgeless graphs. For regular graphs with degree r: `RMS = r`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_degree_rms};
///
/// // K_3: 3 edges, all (2,2) → √((3·(4+4))/(2·3)) = √(4) = 2.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((edge_degree_rms(&g).unwrap() - 2.0).abs() < 1e-10);
/// ```
pub fn edge_degree_rms(graph: &Graph) -> IgraphResult<f64> {
    let mut sum_sq = 0.0_f64;
    let mut edge_count = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        sum_sq += du * du + dv * dv;
        edge_count += 1;
    }

    if edge_count == 0 {
        return Ok(0.0);
    }

    Ok((sum_sq / (2.0 * edge_count as f64)).sqrt())
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

    // --- edge_degree_harmonic_sum ---

    #[test]
    fn harm_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_harmonic_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn harm_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_harmonic_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn harm_regular() {
        // Regular degree r: harmonic mean = r → m·r
        assert!((edge_degree_harmonic_sum(&k3()).unwrap() - 6.0).abs() < 1e-10);
        assert!((edge_degree_harmonic_sum(&k4()).unwrap() - 18.0).abs() < 1e-10);
        assert!((edge_degree_harmonic_sum(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn harm_single_edge() {
        // (1,1): harmonic = 1
        assert!((edge_degree_harmonic_sum(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn harm_star5() {
        // 4 edges (4,1): harmonic = 2·4·1/(4+1) = 8/5 = 1.6 each → 6.4
        assert!((edge_degree_harmonic_sum(&star5()).unwrap() - 6.4).abs() < 1e-10);
    }

    #[test]
    fn harm_path3() {
        // 2 edges (1,2): harmonic = 2·1·2/(1+2) = 4/3 each → 8/3
        let expected = 8.0 / 3.0;
        assert!((edge_degree_harmonic_sum(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn harm_paw() {
        // (0,1)d=(2,2):2  (0,2)d=(2,3):2·6/5=2.4  (1,2)d=(2,3):2.4  (2,3)d=(3,1):2·3/4=1.5
        let expected = 2.0 + 2.4 + 2.4 + 1.5;
        assert!((edge_degree_harmonic_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_geometric_sum ---

    #[test]
    fn geom_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_geometric_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn geom_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_geometric_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn geom_regular() {
        // Regular degree r: geometric mean = r → m·r
        assert!((edge_degree_geometric_sum(&k3()).unwrap() - 6.0).abs() < 1e-10);
        assert!((edge_degree_geometric_sum(&k4()).unwrap() - 18.0).abs() < 1e-10);
        assert!((edge_degree_geometric_sum(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn geom_single_edge() {
        // (1,1): √1 = 1
        assert!((edge_degree_geometric_sum(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn geom_star5() {
        // 4 edges (4,1): √4 = 2 each → 8
        assert!((edge_degree_geometric_sum(&star5()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn geom_path3() {
        // 2 edges (1,2): √2 each → 2√2
        let expected = 2.0 * 2.0_f64.sqrt();
        assert!((edge_degree_geometric_sum(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn geom_paw() {
        // (0,1):√4=2  (0,2):√6  (1,2):√6  (2,3):√3
        let expected = 2.0 + 2.0 * 6.0_f64.sqrt() + 3.0_f64.sqrt();
        assert!((edge_degree_geometric_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_ratio_sum ---

    #[test]
    fn ratio_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_ratio_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ratio_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_ratio_sum(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ratio_regular() {
        // Regular: ratio = 1 → sum = m
        assert!((edge_degree_ratio_sum(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((edge_degree_ratio_sum(&k4()).unwrap() - 6.0).abs() < 1e-10);
        assert!((edge_degree_ratio_sum(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ratio_single_edge() {
        // (1,1): ratio = 1
        assert!((edge_degree_ratio_sum(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ratio_star5() {
        // 4 edges (4,1): ratio = 1/4 = 0.25 each → 1.0
        assert!((edge_degree_ratio_sum(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ratio_path3() {
        // 2 edges (1,2): ratio = 1/2 each → 1.0
        assert!((edge_degree_ratio_sum(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ratio_paw() {
        // (0,1):2/2=1  (0,2):2/3  (1,2):2/3  (2,3):1/3
        let expected = 1.0 + 2.0 / 3.0 + 2.0 / 3.0 + 1.0 / 3.0;
        assert!((edge_degree_ratio_sum(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- edge_degree_rms ---

    #[test]
    fn rms_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_degree_rms(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rms_isolated() {
        let g = Graph::with_vertices(5);
        assert!(edge_degree_rms(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rms_regular() {
        // Regular degree r: √((m·2r²)/(2m)) = r
        assert!((edge_degree_rms(&k3()).unwrap() - 2.0).abs() < 1e-10);
        assert!((edge_degree_rms(&k4()).unwrap() - 3.0).abs() < 1e-10);
        assert!((edge_degree_rms(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rms_single_edge() {
        // (1,1): √((1+1)/2) = 1
        assert!((edge_degree_rms(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rms_star5() {
        // 4 edges (4,1): sum_sq = 4·(16+1) = 68, 2m=8 → √(68/8) = √8.5
        let expected = (68.0 / 8.0_f64).sqrt();
        assert!((edge_degree_rms(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rms_path3() {
        // 2 edges (1,2): sum_sq = 2·(1+4) = 10, 2m=4 → √(10/4) = √2.5
        let expected = (10.0 / 4.0_f64).sqrt();
        assert!((edge_degree_rms(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rms_paw() {
        // (0,1):(4+4)=8  (0,2):(4+9)=13  (1,2):(4+9)=13  (2,3):(9+1)=10
        // sum_sq = 44, 2m = 8 → √(44/8) = √5.5
        let expected = (44.0 / 8.0_f64).sqrt();
        assert!((edge_degree_rms(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn harmonic_le_geometric_le_arithmetic() {
        // AM-GM-HM inequality: harmonic ≤ geometric ≤ arithmetic (per edge)
        // So sums preserve the inequality
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let h = edge_degree_harmonic_sum(g).unwrap();
            let geo = edge_degree_geometric_sum(g).unwrap();
            assert!(h <= geo + 1e-10);
        }
    }

    #[test]
    fn ratio_in_0_m() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let val = edge_degree_ratio_sum(g).unwrap();
            let m = g.edges().filter(|&(u, v)| u != v).count() as f64;
            assert!(val >= -1e-10);
            assert!(val <= m + 1e-10);
        }
    }

    #[test]
    fn ratio_equals_m_for_regular() {
        // Regular graphs: all ratios = 1 → sum = m
        for g in &[k3(), k4(), cycle4()] {
            let val = edge_degree_ratio_sum(g).unwrap();
            let m = g.edges().filter(|&(u, v)| u != v).count() as f64;
            assert!((val - m).abs() < 1e-10);
        }
    }

    #[test]
    fn rms_ge_mean() {
        // RMS ≥ arithmetic mean of endpoint degrees
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let rms = edge_degree_rms(g).unwrap();
            let edges: Vec<_> = g.edges().filter(|&(u, v)| u != v).collect();
            if edges.is_empty() {
                continue;
            }
            let mean: f64 = edges
                .iter()
                .map(|&(u, v)| {
                    let du = g.degree(u).unwrap() as f64;
                    let dv = g.degree(v).unwrap() as f64;
                    f64::midpoint(du, dv)
                })
                .sum::<f64>()
                / edges.len() as f64;
            assert!(rms >= mean - 1e-10);
        }
    }
}
