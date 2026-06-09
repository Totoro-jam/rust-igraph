//! Sombor index variants (ALGO-TR-057).
//!
//! - **Sombor index** `SO(G) = Σ_{(u,v)∈E} √(d(u)² + d(v)²)`
//!   Introduced by Gutman (2021). Degree-based geometric topological index.
//! - **Reduced Sombor index** `SO_red(G) = Σ_{(u,v)∈E} √((d(u)-1)² + (d(v)-1)²)`
//!   Uses reduced degrees (d-1). Zero contribution from pendant edges.
//! - **Average Sombor index** `SO_avg(G) = SO(G) / m`
//!   Sombor index normalized by edge count.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Sombor index.
///
/// `SO(G) = Σ_{(u,v)∈E} √(d(u)² + d(v)²)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sombor_index};
///
/// // K_3: each edge √(4+4) = 2√2, 3 edges → 6√2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((sombor_index(&g).unwrap() - 6.0 * std::f64::consts::SQRT_2).abs() < 1e-10);
/// ```
pub fn sombor_index(graph: &Graph) -> IgraphResult<f64> {
    let mut so = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        so += (du * du + dv * dv).sqrt();
    }

    Ok(so)
}

/// Compute the reduced Sombor index.
///
/// `SO_red(G) = Σ_{(u,v)∈E} √((d(u)-1)² + (d(v)-1)²)`
///
/// Uses reduced degrees `d(v)-1`. Self-loops are skipped.
/// Pendant edges (where one endpoint has degree 1) contribute
/// `√(0² + (d-1)²) = d-1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_sombor_index};
///
/// // K_3: each edge √((2-1)²+(2-1)²) = √2, 3 edges → 3√2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((reduced_sombor_index(&g).unwrap() - 3.0 * std::f64::consts::SQRT_2).abs() < 1e-10);
/// ```
pub fn reduced_sombor_index(graph: &Graph) -> IgraphResult<f64> {
    let mut so_red = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        let a = if du > 0 { (du - 1) as f64 } else { 0.0 };
        let b = if dv > 0 { (dv - 1) as f64 } else { 0.0 };
        so_red += (a * a + b * b).sqrt();
    }

    Ok(so_red)
}

/// Compute the average Sombor index.
///
/// `SO_avg(G) = SO(G) / m`
///
/// Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, average_sombor_index};
///
/// // K_3: SO = 6√2, m = 3 → SO_avg = 2√2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((average_sombor_index(&g).unwrap() - 2.0 * std::f64::consts::SQRT_2).abs() < 1e-10);
/// ```
pub fn average_sombor_index(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }
    let so = sombor_index(graph)?;
    Ok(so / m as f64)
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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- sombor_index ---

    #[test]
    fn so_empty() {
        let g = Graph::with_vertices(0);
        assert!((sombor_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_isolated() {
        let g = Graph::with_vertices(5);
        assert!((sombor_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_single_edge() {
        // √(1+1) = √2
        let expected = std::f64::consts::SQRT_2;
        assert!((sombor_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_path3() {
        // (0,1): √(1+4)=√5, (1,2): √(4+1)=√5
        // SO = 2√5
        let expected = 2.0 * 5.0_f64.sqrt();
        assert!((sombor_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_path4() {
        // (0,1): √(1+4)=√5, (1,2): √(4+4)=2√2, (2,3): √(4+1)=√5
        // SO = 2√5 + 2√2
        let expected = 2.0 * 5.0_f64.sqrt() + 2.0 * std::f64::consts::SQRT_2;
        assert!((sombor_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_k3() {
        // each edge √(4+4) = 2√2, 3 edges → 6√2
        let expected = 6.0 * std::f64::consts::SQRT_2;
        assert!((sombor_index(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_k4() {
        // each edge √(9+9) = 3√2, 6 edges → 18√2
        let expected = 18.0 * std::f64::consts::SQRT_2;
        assert!((sombor_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_cycle4() {
        // each edge √(4+4) = 2√2, 4 edges → 8√2
        let expected = 8.0 * std::f64::consts::SQRT_2;
        assert!((sombor_index(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_cycle5() {
        // each edge √(4+4) = 2√2, 5 edges → 10√2
        let expected = 10.0 * std::f64::consts::SQRT_2;
        assert!((sombor_index(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_star5() {
        // each edge √(16+1) = √17, 4 edges → 4√17
        let expected = 4.0 * 17.0_f64.sqrt();
        assert!((sombor_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_paw() {
        // degrees [2,2,3,1]
        // (0,1): √(4+4)=2√2, (0,2): √(4+9)=√13, (1,2): √(4+9)=√13, (2,3): √(9+1)=√10
        let expected = 2.0 * std::f64::consts::SQRT_2 + 2.0 * 13.0_f64.sqrt() + 10.0_f64.sqrt();
        assert!((sombor_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_regular_formula() {
        // r-regular: SO = m · √(2r²) = m · r√2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * r * std::f64::consts::SQRT_2;
            assert!((sombor_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- reduced_sombor_index ---

    #[test]
    fn so_red_empty() {
        let g = Graph::with_vertices(0);
        assert!((reduced_sombor_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_red_single_edge() {
        // √((1-1)²+(1-1)²) = 0
        assert!((reduced_sombor_index(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_red_path3() {
        // (0,1): √(0²+1²)=1, (1,2): √(1²+0²)=1
        // SO_red = 2
        assert!((reduced_sombor_index(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn so_red_path4() {
        // (0,1): √(0+1)=1, (1,2): √(1+1)=√2, (2,3): √(1+0)=1
        // SO_red = 2 + √2
        let expected = 2.0 + std::f64::consts::SQRT_2;
        assert!((reduced_sombor_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_k3() {
        // each edge √(1+1)=√2, 3 edges → 3√2
        let expected = 3.0 * std::f64::consts::SQRT_2;
        assert!((reduced_sombor_index(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_k4() {
        // each edge √(4+4)=2√2, 6 edges → 12√2
        let expected = 12.0 * std::f64::consts::SQRT_2;
        assert!((reduced_sombor_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_cycle4() {
        // each edge √(1+1)=√2, 4 edges → 4√2
        let expected = 4.0 * std::f64::consts::SQRT_2;
        assert!((reduced_sombor_index(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_cycle5() {
        // each edge √(1+1)=√2, 5 edges → 5√2
        let expected = 5.0 * std::f64::consts::SQRT_2;
        assert!((reduced_sombor_index(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_star5() {
        // (0,leaf): √((4-1)²+(1-1)²) = √(9+0) = 3, 4 edges → 12
        assert!((reduced_sombor_index(&star5()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn so_red_paw() {
        // degrees [2,2,3,1]
        // (0,1): √(1+1)=√2, (0,2): √(1+4)=√5, (1,2): √(1+4)=√5, (2,3): √(4+0)=2
        let expected = std::f64::consts::SQRT_2 + 2.0 * 5.0_f64.sqrt() + 2.0;
        assert!((reduced_sombor_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_red_regular_formula() {
        // r-regular: SO_red = m · √(2(r-1)²) = m · (r-1)·√2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * (r - 1.0) * std::f64::consts::SQRT_2;
            assert!((reduced_sombor_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn so_red_leq_so() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(reduced_sombor_index(g).unwrap() <= sombor_index(g).unwrap() + 1e-10);
        }
    }

    // --- average_sombor_index ---

    #[test]
    fn so_avg_empty() {
        let g = Graph::with_vertices(0);
        assert!((average_sombor_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_avg_isolated() {
        let g = Graph::with_vertices(5);
        assert!((average_sombor_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn so_avg_single_edge() {
        // SO/m = √2/1 = √2
        let expected = std::f64::consts::SQRT_2;
        assert!((average_sombor_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_avg_k3() {
        // SO = 6√2, m = 3 → 2√2
        let expected = 2.0 * std::f64::consts::SQRT_2;
        assert!((average_sombor_index(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_avg_k4() {
        // SO = 18√2, m = 6 → 3√2
        let expected = 3.0 * std::f64::consts::SQRT_2;
        assert!((average_sombor_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn so_avg_regular_is_r_sqrt2() {
        // r-regular: SO_avg = SO/m = r√2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let r = g.degree(0).unwrap() as f64;
            let expected = r * std::f64::consts::SQRT_2;
            assert!((average_sombor_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn so_avg_consistency() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount();
            if m > 0 {
                let so = sombor_index(g).unwrap();
                let so_avg = average_sombor_index(g).unwrap();
                assert!((so_avg - so / m as f64).abs() < 1e-10);
            }
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(sombor_index(g).unwrap() >= -1e-10);
            assert!(reduced_sombor_index(g).unwrap() >= -1e-10);
            assert!(average_sombor_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn so_geq_degree_sum() {
        // SO(G) ≥ DS(G) since √(a²+b²) ≥ √(a+b) for a,b ≥ 1 is not always true,
        // but SO(G) ≥ m·√2 for non-empty graphs (minimum when d_u=d_v=1)
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let m = g.ecount() as f64;
            assert!(sombor_index(g).unwrap() >= m * std::f64::consts::SQRT_2 - 1e-10);
        }
    }
}
