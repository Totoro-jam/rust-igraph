//! Multiplicative connectivity indices (ALGO-TR-074).
//!
//! Product-based (multiplicative) versions of classical additive
//! bond-additive indices. Uses logarithmic accumulation to avoid
//! floating-point overflow for large graphs:
//!
//! - **Multiplicative sum-connectivity** `Πsc(G) = Π_{(u,v)∈E} 1/√(du+dv)`
//! - **Multiplicative Randić** `Πχ(G) = Π_{(u,v)∈E} 1/√(du·dv)`
//! - **Multiplicative ABC** `Πabc(G) = Π_{(u,v)∈E} √((du+dv-2)/(du·dv))`
//! - **Multiplicative GA** `Πga(G) = Π_{(u,v)∈E} 2√(du·dv)/(du+dv)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the multiplicative sum-connectivity index.
///
/// `Πsc(G) = Π_{(u,v)∈E} 1/√(du+dv)`
///
/// Edges with `du+dv == 0` are skipped. Self-loops are skipped.
/// Returns 1.0 for edgeless graphs (empty product).
///
/// Uses log-sum to avoid overflow: `Πsc = exp(-½ Σ ln(du+dv))`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multiplicative_sum_connectivity};
///
/// // K_3: 3 edges, d=(2,2), each 1/√4 = 1/2 → (1/2)³ = 1/8
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((multiplicative_sum_connectivity(&g).unwrap() - 0.125).abs() < 1e-10);
/// ```
pub fn multiplicative_sum_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let mut log_sum = 0.0_f64;
    let mut count = 0_usize;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s > 0.0 {
            log_sum -= 0.5 * s.ln();
            count += 1;
        }
    }

    if count == 0 {
        return Ok(1.0);
    }

    Ok(log_sum.exp())
}

/// Compute the multiplicative Randić index.
///
/// `Πχ(G) = Π_{(u,v)∈E} 1/√(du·dv)`
///
/// Edges with a degree-0 endpoint are skipped. Self-loops are skipped.
/// Returns 1.0 for edgeless graphs (empty product).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multiplicative_randic};
///
/// // K_3: 3 edges, d=(2,2), each 1/2 → (1/2)³ = 1/8
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((multiplicative_randic(&g).unwrap() - 0.125).abs() < 1e-10);
/// ```
pub fn multiplicative_randic(graph: &Graph) -> IgraphResult<f64> {
    let mut log_sum = 0.0_f64;
    let mut count = 0_usize;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product > 0.0 {
            log_sum -= 0.5 * product.ln();
            count += 1;
        }
    }

    if count == 0 {
        return Ok(1.0);
    }

    Ok(log_sum.exp())
}

/// Compute the multiplicative atom-bond connectivity index.
///
/// `Πabc(G) = Π_{(u,v)∈E} √((du+dv-2)/(du·dv))`
///
/// Edges where `du+dv-2 ≤ 0` or `du·dv == 0` are skipped.
/// Self-loops are skipped. Returns 1.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multiplicative_abc};
///
/// // K_3: 3 edges, d=(2,2), each √(2/4) = 1/√2 → (1/√2)³
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = (1.0 / 2.0_f64.sqrt()).powi(3);
/// assert!((multiplicative_abc(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn multiplicative_abc(graph: &Graph) -> IgraphResult<f64> {
    let mut log_sum = 0.0_f64;
    let mut count = 0_usize;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        let numer = du + dv - 2.0;
        if product > 0.0 && numer > 0.0 {
            log_sum += 0.5 * (numer / product).ln();
            count += 1;
        }
    }

    if count == 0 {
        return Ok(1.0);
    }

    Ok(log_sum.exp())
}

/// Compute the multiplicative geometric-arithmetic index.
///
/// `Πga(G) = Π_{(u,v)∈E} 2√(du·dv)/(du+dv)`
///
/// Edges with `du+dv == 0` are skipped. Self-loops are skipped.
/// Returns 1.0 for edgeless graphs.
///
/// For regular graphs, each factor is 1, so `Πga = 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, multiplicative_ga};
///
/// // K_3: regular → each factor = 1 → product = 1
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((multiplicative_ga(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn multiplicative_ga(graph: &Graph) -> IgraphResult<f64> {
    let mut log_sum = 0.0_f64;
    let mut count = 0_usize;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s > 0.0 {
            let factor = 2.0 * (du * dv).sqrt() / s;
            if factor > 0.0 {
                log_sum += factor.ln();
                count += 1;
            }
        }
    }

    if count == 0 {
        return Ok(1.0);
    }

    Ok(log_sum.exp())
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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- multiplicative_sum_connectivity ---

    #[test]
    fn msc_empty() {
        let g = Graph::with_vertices(0);
        assert!((multiplicative_sum_connectivity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn msc_isolated() {
        let g = Graph::with_vertices(5);
        assert!((multiplicative_sum_connectivity(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn msc_single_edge() {
        // d=(1,1), 1/√2
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!(
            (multiplicative_sum_connectivity(&single_edge()).unwrap() - expected).abs() < 1e-10
        );
    }

    #[test]
    fn msc_k3() {
        // 3 edges, d=(2,2), each 1/√4=1/2 → (1/2)³=1/8
        assert!((multiplicative_sum_connectivity(&k3()).unwrap() - 0.125).abs() < 1e-10);
    }

    #[test]
    fn msc_k4() {
        // 6 edges, d=(3,3), each 1/√6 → (1/√6)^6 = 1/216
        let expected = (1.0 / 6.0_f64.sqrt()).powi(6);
        assert!((multiplicative_sum_connectivity(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn msc_path3() {
        // 2 edges, d=(1,2): 1/√3 each → (1/√3)² = 1/3
        let expected = 1.0 / 3.0;
        assert!((multiplicative_sum_connectivity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn msc_cycle4() {
        // 4 edges, d=(2,2): 1/2 each → (1/2)⁴ = 1/16
        let expected = 1.0 / 16.0;
        assert!((multiplicative_sum_connectivity(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn msc_star5() {
        // 4 edges, d=(4,1): 1/√5 each → (1/√5)⁴ = 1/25
        let expected = 1.0 / 25.0;
        assert!((multiplicative_sum_connectivity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn msc_paw() {
        // (0,1) d=(2,2): 1/√4=1/2
        // (0,2) d=(2,3): 1/√5
        // (1,2) d=(2,3): 1/√5
        // (2,3) d=(3,1): 1/√4=1/2
        let expected = 0.5 * (1.0 / 5.0_f64.sqrt()).powi(2) * 0.5;
        assert!((multiplicative_sum_connectivity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- multiplicative_randic ---

    #[test]
    fn mr_empty() {
        let g = Graph::with_vertices(0);
        assert!((multiplicative_randic(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mr_isolated() {
        let g = Graph::with_vertices(5);
        assert!((multiplicative_randic(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mr_single_edge() {
        // d=(1,1), 1/1=1
        assert!((multiplicative_randic(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mr_k3() {
        // d=(2,2), 1/2 per edge → (1/2)³ = 1/8
        assert!((multiplicative_randic(&k3()).unwrap() - 0.125).abs() < 1e-10);
    }

    #[test]
    fn mr_k4() {
        // d=(3,3), 1/3 per edge → (1/3)^6
        let expected = (1.0_f64 / 3.0).powi(6);
        assert!((multiplicative_randic(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mr_path3() {
        // d=(1,2): 1/√2 each → (1/√2)² = 1/2
        assert!((multiplicative_randic(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mr_cycle4() {
        // d=(2,2), 1/2 → (1/2)⁴ = 1/16
        let expected = 1.0 / 16.0;
        assert!((multiplicative_randic(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mr_star5() {
        // d=(4,1): 1/2 each → (1/2)⁴ = 1/16
        let expected = 1.0 / 16.0;
        assert!((multiplicative_randic(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mr_paw() {
        // (0,1) d=(2,2): 1/2
        // (0,2) d=(2,3): 1/√6
        // (1,2) d=(2,3): 1/√6
        // (2,3) d=(3,1): 1/√3
        let expected = 0.5 * (1.0 / 6.0_f64.sqrt()).powi(2) * (1.0 / 3.0_f64.sqrt());
        assert!((multiplicative_randic(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- multiplicative_abc ---

    #[test]
    fn mabc_empty() {
        let g = Graph::with_vertices(0);
        assert!((multiplicative_abc(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mabc_isolated() {
        let g = Graph::with_vertices(5);
        assert!((multiplicative_abc(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mabc_single_edge() {
        // d=(1,1), numer=0 → skipped → empty product = 1
        assert!((multiplicative_abc(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mabc_k3() {
        // d=(2,2), √(2/4)=1/√2 → (1/√2)³
        let expected = (1.0 / 2.0_f64.sqrt()).powi(3);
        assert!((multiplicative_abc(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mabc_k4() {
        // d=(3,3), √(4/9)=2/3 → (2/3)^6
        let expected = (2.0_f64 / 3.0).powi(6);
        assert!((multiplicative_abc(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mabc_path3() {
        // d=(1,2): √(1/2)=1/√2 → (1/√2)²=1/2
        assert!((multiplicative_abc(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mabc_cycle4() {
        // d=(2,2): 1/√2 → (1/√2)⁴ = 1/4
        assert!((multiplicative_abc(&cycle4()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn mabc_star5() {
        // d=(4,1): √(3/4)=√3/2 → (√3/2)⁴ = 9/16
        let expected = (3.0_f64.sqrt() / 2.0).powi(4);
        assert!((multiplicative_abc(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mabc_paw() {
        // (0,1) d=(2,2): 1/√2
        // (0,2) d=(2,3): √(3/6)=1/√2
        // (1,2) d=(2,3): 1/√2
        // (2,3) d=(3,1): √(2/3)
        let expected = (1.0 / 2.0_f64.sqrt()).powi(3) * (2.0_f64 / 3.0).sqrt();
        assert!((multiplicative_abc(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- multiplicative_ga ---

    #[test]
    fn mga_empty() {
        let g = Graph::with_vertices(0);
        assert!((multiplicative_ga(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_isolated() {
        let g = Graph::with_vertices(5);
        assert!((multiplicative_ga(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_single_edge() {
        // d=(1,1), 2·1/2=1 → product=1
        assert!((multiplicative_ga(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_k3() {
        // regular → all factors = 1 → product = 1
        assert!((multiplicative_ga(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_k4() {
        assert!((multiplicative_ga(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_cycle4() {
        assert!((multiplicative_ga(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_cycle5() {
        assert!((multiplicative_ga(&cycle5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mga_path3() {
        // d=(1,2): 2√2/3 → (2√2/3)²
        let factor = 2.0 * 2.0_f64.sqrt() / 3.0;
        let expected = factor * factor;
        assert!((multiplicative_ga(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mga_star5() {
        // d=(4,1): 2·2/5=4/5 → (4/5)⁴
        let expected = (4.0_f64 / 5.0).powi(4);
        assert!((multiplicative_ga(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mga_paw() {
        // (0,1) d=(2,2): 1
        // (0,2) d=(2,3): 2√6/5
        // (1,2) d=(2,3): 2√6/5
        // (2,3) d=(3,1): 2√3/4=√3/2
        let expected = (2.0 * 6.0_f64.sqrt() / 5.0).powi(2) * (3.0_f64.sqrt() / 2.0);
        assert!((multiplicative_ga(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn regular_ga_is_one() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            assert!((multiplicative_ga(g).unwrap() - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn ga_le_one_for_simple() {
        // AM-GM: 2√(du·dv)/(du+dv) ≤ 1, so product ≤ 1
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(multiplicative_ga(g).unwrap() <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn all_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(multiplicative_sum_connectivity(g).unwrap() > 0.0);
            assert!(multiplicative_randic(g).unwrap() > 0.0);
            assert!(multiplicative_abc(g).unwrap() > 0.0);
            assert!(multiplicative_ga(g).unwrap() > 0.0);
        }
    }
}
