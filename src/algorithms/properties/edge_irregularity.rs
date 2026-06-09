//! Edge-based irregularity indices (ALGO-TR-079).
//!
//! Bond-additive irregularity measures that quantify how much edge
//! endpoints differ in degree:
//!
//! - **IRD** (square-difference irregularity)
//!   `IRD(G) = Σ_{(u,v)∈E} |d(u)² - d(v)²|`
//! - **IRA** (power-difference irregularity)
//!   `IRA_α(G) = Σ_{(u,v)∈E} |d(u)^α - d(v)^α|` for real α
//! - **IRB** (root-difference irregularity)
//!   `IRB(G) = Σ_{(u,v)∈E} (√d(u) - √d(v))²`
//! - **IRGA** (geometric-arithmetic irregularity)
//!   `IRGA(G) = Σ_{(u,v)∈E} ln((d(u)+d(v))/(2√(d(u)·d(v))))`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the square-difference irregularity index.
///
/// `IRD(G) = Σ_{(u,v)∈E} |d(u)² - d(v)²|`
///
/// Equals 0 for regular graphs. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ird_index};
///
/// // K_3: all degrees 2, IRD = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(ird_index(&g).unwrap().abs() < 1e-10);
/// ```
pub fn ird_index(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        result += (du * du - dv * dv).abs();
    }

    Ok(result)
}

/// Compute the power-difference irregularity index.
///
/// `IRA_α(G) = Σ_{(u,v)∈E} |d(u)^α - d(v)^α|` for real α.
///
/// Special cases:
/// - `α = 1`: equals the Albertson index `Σ |d(u)-d(v)|`
/// - `α = 2`: equals `IRD(G)`
/// - `α = 0.5`: equals `Σ |√d(u) - √d(v)|`
///
/// Edges with degree-0 endpoint are skipped when `α < 0`.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ira_index};
///
/// // Star S_5: 4 edges (4,1), α=2 → 4·|16-1| = 60
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((ira_index(&g, 2.0).unwrap() - 60.0).abs() < 1e-10);
/// ```
pub fn ira_index(graph: &Graph, alpha: f64) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if alpha < 0.0 && (du == 0.0 || dv == 0.0) {
            continue;
        }
        result += (du.powf(alpha) - dv.powf(alpha)).abs();
    }

    Ok(result)
}

/// Compute the root-difference irregularity index.
///
/// `IRB(G) = Σ_{(u,v)∈E} (√d(u) - √d(v))²`
///
/// This is always non-negative and equals 0 for regular graphs.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, irb_index};
///
/// // K_3: all degrees 2, IRB = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(irb_index(&g).unwrap().abs() < 1e-10);
/// ```
pub fn irb_index(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = (graph.degree(u)? as f64).sqrt();
        let dv = (graph.degree(v)? as f64).sqrt();
        let diff = du - dv;
        result += diff * diff;
    }

    Ok(result)
}

/// Compute the geometric-arithmetic irregularity index.
///
/// `IRGA(G) = Σ_{(u,v)∈E} ln((d(u)+d(v)) / (2√(d(u)·d(v))))`
///
/// By AM-GM inequality, each term is ≥ 0. Equals 0 for regular graphs.
/// Edges with degree-0 endpoint are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, irga_index};
///
/// // K_3: all degrees 2, IRGA = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(irga_index(&g).unwrap().abs() < 1e-10);
/// ```
pub fn irga_index(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product <= 0.0 {
            continue;
        }
        let am = f64::midpoint(du, dv);
        let gm = product.sqrt();
        result += (am / gm).ln();
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

    // --- ird_index ---

    #[test]
    fn ird_empty() {
        let g = Graph::with_vertices(0);
        assert!(ird_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ird_isolated() {
        let g = Graph::with_vertices(5);
        assert!(ird_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ird_regular_zero() {
        assert!(ird_index(&k3()).unwrap().abs() < 1e-10);
        assert!(ird_index(&k4()).unwrap().abs() < 1e-10);
        assert!(ird_index(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ird_single_edge() {
        // d=(1,1): |1-1|=0
        assert!(ird_index(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ird_star5() {
        // 4 edges (4,1): 4·|16-1| = 60
        assert!((ird_index(&star5()).unwrap() - 60.0).abs() < 1e-10);
    }

    #[test]
    fn ird_path3() {
        // 2 edges (1,2): 2·|1-4| = 6
        assert!((ird_index(&path3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn ird_paw() {
        // (0,1) d=(2,2): |4-4|=0
        // (0,2) d=(2,3): |4-9|=5
        // (1,2) d=(2,3): |4-9|=5
        // (2,3) d=(3,1): |9-1|=8
        assert!((ird_index(&paw()).unwrap() - 18.0).abs() < 1e-10);
    }

    // --- ira_index ---

    #[test]
    fn ira_empty() {
        let g = Graph::with_vertices(0);
        assert!(ira_index(&g, 2.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ira_regular_zero() {
        assert!(ira_index(&k3(), 2.0).unwrap().abs() < 1e-10);
        assert!(ira_index(&k4(), 3.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ira_alpha1_is_albertson() {
        // α=1: Σ |du-dv| = Albertson index
        // Star S_5: 4·|4-1| = 12
        assert!((ira_index(&star5(), 1.0).unwrap() - 12.0).abs() < 1e-10);
        // Paw: |2-2|+|2-3|+|2-3|+|3-1| = 0+1+1+2 = 4
        assert!((ira_index(&paw(), 1.0).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ira_alpha2_equals_ird() {
        // α=2: equals IRD
        for g in &[single_edge(), path3(), k3(), star5(), paw()] {
            let ird_val = ird_index(g).unwrap();
            let ira_val = ira_index(g, 2.0).unwrap();
            assert!(
                (ird_val - ira_val).abs() < 1e-10,
                "IRD={ird_val} IRA_2={ira_val}"
            );
        }
    }

    #[test]
    fn ira_star5_alpha2() {
        assert!((ira_index(&star5(), 2.0).unwrap() - 60.0).abs() < 1e-10);
    }

    #[test]
    fn ira_star5_alpha3() {
        // 4·|64-1| = 252
        assert!((ira_index(&star5(), 3.0).unwrap() - 252.0).abs() < 1e-10);
    }

    // --- irb_index ---

    #[test]
    fn irb_empty() {
        let g = Graph::with_vertices(0);
        assert!(irb_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irb_isolated() {
        let g = Graph::with_vertices(5);
        assert!(irb_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irb_regular_zero() {
        assert!(irb_index(&k3()).unwrap().abs() < 1e-10);
        assert!(irb_index(&k4()).unwrap().abs() < 1e-10);
        assert!(irb_index(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irb_single_edge() {
        assert!(irb_index(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irb_star5() {
        // 4 edges (4,1): (√4-√1)² = (2-1)² = 1 → 4·1 = 4
        assert!((irb_index(&star5()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn irb_path3() {
        // 2 edges (1,2): (1-√2)² each → 2·(1-√2)² = 2·(3-2√2)
        let expected = 2.0 * (3.0 - 2.0 * 2.0_f64.sqrt());
        assert!((irb_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irb_paw() {
        // (0,1) d=(2,2): 0
        // (0,2) d=(2,3): (√2-√3)²
        // (1,2) d=(2,3): (√2-√3)²
        // (2,3) d=(3,1): (√3-1)²
        let d23 = (2.0_f64.sqrt() - 3.0_f64.sqrt()).powi(2);
        let d31 = (3.0_f64.sqrt() - 1.0).powi(2);
        let expected = 2.0 * d23 + d31;
        assert!((irb_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irb_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(irb_index(g).unwrap() >= -1e-10);
        }
    }

    // --- irga_index ---

    #[test]
    fn irga_empty() {
        let g = Graph::with_vertices(0);
        assert!(irga_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irga_isolated() {
        let g = Graph::with_vertices(5);
        assert!(irga_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irga_regular_zero() {
        assert!(irga_index(&k3()).unwrap().abs() < 1e-10);
        assert!(irga_index(&k4()).unwrap().abs() < 1e-10);
        assert!(irga_index(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irga_single_edge() {
        // d=(1,1): ln(1/1)=0
        assert!(irga_index(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn irga_star5() {
        // 4 edges (4,1): ln((4+1)/(2·√4)) = ln(5/4)
        let expected = 4.0 * (5.0_f64 / 4.0).ln();
        assert!((irga_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irga_path3() {
        // 2 edges (1,2): ln((1+2)/(2·√2)) = ln(3/(2√2))
        let expected = 2.0 * (3.0 / (2.0 * 2.0_f64.sqrt())).ln();
        assert!((irga_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn irga_nonnegative() {
        // By AM-GM, each term ≥ 0
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(irga_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn irga_paw() {
        // (0,1) d=(2,2): ln(4/(2·2))=ln(1)=0
        // (0,2) d=(2,3): ln(5/(2·√6))
        // (1,2) d=(2,3): ln(5/(2·√6))
        // (2,3) d=(3,1): ln(4/(2·√3))=ln(2/√3)
        let t1 = (5.0 / (2.0 * 6.0_f64.sqrt())).ln();
        let t2 = (2.0 / 3.0_f64.sqrt()).ln();
        let expected = 2.0 * t1 + t2;
        assert!((irga_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn ird_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(ird_index(g).unwrap() >= -1e-10);
        }
    }

    #[test]
    fn ira_nonnegative() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(ira_index(g, 1.0).unwrap() >= -1e-10);
            assert!(ira_index(g, 2.0).unwrap() >= -1e-10);
            assert!(ira_index(g, 0.5).unwrap() >= -1e-10);
        }
    }
}
