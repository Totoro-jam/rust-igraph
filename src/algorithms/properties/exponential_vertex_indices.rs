//! Exponential vertex-degree indices (ALGO-TR-080).
//!
//! Vertex-additive indices using the exponential of degree-based
//! functions, complementing the edge-based `exponential_indices.rs`:
//!
//! - **Exponential first Zagreb** `eM₁(G) = Σ_v e^{d(v)²}`
//! - **Exponential forgotten** `eF(G) = Σ_v e^{d(v)³}`
//! - **Exponential inverse degree** `eID(G) = Σ_v e^{1/d(v)}`
//!   (d(v)>0 only)
//! - **Exponential sum-connectivity** `eSC(G) = Σ_{(u,v)∈E}
//!   e^{1/√(d(u)+d(v))}` — edge-based variant

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the exponential first Zagreb index.
///
/// `eM₁(G) = Σ_v e^{d(v)²}`
///
/// For each vertex, the contribution is `e` raised to the square of
/// its degree. Isolated vertices contribute `e^0 = 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_first_zagreb};
///
/// // K_3: 3 vertices with d=2 → 3·e^4
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((exponential_first_zagreb(&g).unwrap() - 3.0 * 4.0_f64.exp()).abs() < 1e-10);
/// ```
pub fn exponential_first_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as f64;
        result += (d * d).exp();
    }

    Ok(result)
}

/// Compute the exponential forgotten index.
///
/// `eF(G) = Σ_v e^{d(v)³}`
///
/// For each vertex, the contribution is `e` raised to the cube of
/// its degree. Isolated vertices contribute `e^0 = 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_forgotten};
///
/// // K_3: 3 vertices with d=2 → 3·e^8
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((exponential_forgotten(&g).unwrap() - 3.0 * 8.0_f64.exp()).abs() < 1e-6);
/// ```
pub fn exponential_forgotten(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)? as f64;
        result += (d * d * d).exp();
    }

    Ok(result)
}

/// Compute the exponential inverse degree index.
///
/// `eID(G) = Σ_v e^{1/d(v)}` for `d(v) > 0`.
///
/// Vertices with degree 0 are skipped. Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_inverse_degree};
///
/// // K_3: 3 vertices with d=2 → 3·e^{0.5}
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((exponential_inverse_degree(&g).unwrap() - 3.0 * 0.5_f64.exp()).abs() < 1e-10);
/// ```
pub fn exponential_inverse_degree(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d == 0 {
            continue;
        }
        result += (1.0 / d as f64).exp();
    }

    Ok(result)
}

/// Compute the exponential sum-connectivity index.
///
/// `eSC(G) = Σ_{(u,v)∈E} e^{1/√(d(u)+d(v))}`
///
/// Self-loops are skipped. Returns 0.0 for edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_sum_connectivity};
///
/// // K_3: 3 edges, each d=(2,2) → 3·e^{1/2}
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((exponential_sum_connectivity(&g).unwrap() - 3.0 * 0.5_f64.exp()).abs() < 1e-10);
/// ```
pub fn exponential_sum_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s > 0.0 {
            result += (1.0 / s.sqrt()).exp();
        }
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

    // --- exponential_first_zagreb ---

    #[test]
    fn efz_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_first_zagreb(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn efz_isolated() {
        // d=0: e^0=1 per vertex → 5
        let g = Graph::with_vertices(5);
        assert!((exponential_first_zagreb(&g).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn efz_k3() {
        // 3·e^4
        assert!((exponential_first_zagreb(&k3()).unwrap() - 3.0 * 4.0_f64.exp()).abs() < 1e-10);
    }

    #[test]
    fn efz_k4() {
        // 4·e^9
        assert!((exponential_first_zagreb(&k4()).unwrap() - 4.0 * 9.0_f64.exp()).abs() < 1e-6);
    }

    #[test]
    fn efz_single_edge() {
        // 2·e^1
        assert!(
            (exponential_first_zagreb(&single_edge()).unwrap() - 2.0 * 1.0_f64.exp()).abs() < 1e-10
        );
    }

    #[test]
    fn efz_star5() {
        // e^16 + 4·e^1
        let expected = 16.0_f64.exp() + 4.0 * 1.0_f64.exp();
        assert!((exponential_first_zagreb(&star5()).unwrap() - expected).abs() < 1e-6);
    }

    #[test]
    fn efz_cycle4() {
        // 4·e^4
        assert!((exponential_first_zagreb(&cycle4()).unwrap() - 4.0 * 4.0_f64.exp()).abs() < 1e-10);
    }

    // --- exponential_forgotten ---

    #[test]
    fn ef_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_forgotten(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ef_isolated() {
        let g = Graph::with_vertices(5);
        assert!((exponential_forgotten(&g).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ef_k3() {
        // 3·e^8
        assert!((exponential_forgotten(&k3()).unwrap() - 3.0 * 8.0_f64.exp()).abs() < 1e-6);
    }

    #[test]
    fn ef_single_edge() {
        // 2·e^1
        assert!(
            (exponential_forgotten(&single_edge()).unwrap() - 2.0 * 1.0_f64.exp()).abs() < 1e-10
        );
    }

    #[test]
    fn ef_star5() {
        // e^64 + 4·e^1
        let expected = 64.0_f64.exp() + 4.0 * 1.0_f64.exp();
        assert!((exponential_forgotten(&star5()).unwrap() - expected).abs() / expected < 1e-10);
    }

    // --- exponential_inverse_degree ---

    #[test]
    fn eid_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_inverse_degree(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eid_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_inverse_degree(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eid_k3() {
        // 3·e^{1/2}
        assert!((exponential_inverse_degree(&k3()).unwrap() - 3.0 * 0.5_f64.exp()).abs() < 1e-10);
    }

    #[test]
    fn eid_single_edge() {
        // 2·e^1
        assert!(
            (exponential_inverse_degree(&single_edge()).unwrap() - 2.0 * 1.0_f64.exp()).abs()
                < 1e-10
        );
    }

    #[test]
    fn eid_star5() {
        // e^{1/4} + 4·e^1
        let expected = 0.25_f64.exp() + 4.0 * 1.0_f64.exp();
        assert!((exponential_inverse_degree(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eid_paw() {
        // d=(2,2,3,1): e^{1/2}+e^{1/2}+e^{1/3}+e^1
        let expected = 2.0 * 0.5_f64.exp() + (1.0 / 3.0_f64).exp() + 1.0_f64.exp();
        assert!((exponential_inverse_degree(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- exponential_sum_connectivity ---

    #[test]
    fn esc_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_sum_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_sum_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn esc_k3() {
        // 3 edges, each (2,2): 3·e^{1/2}
        assert!((exponential_sum_connectivity(&k3()).unwrap() - 3.0 * 0.5_f64.exp()).abs() < 1e-10);
    }

    #[test]
    fn esc_single_edge() {
        // 1 edge (1,1): e^{1/√2}
        let expected = (1.0 / 2.0_f64.sqrt()).exp();
        assert!((exponential_sum_connectivity(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn esc_k4() {
        // 6 edges, each (3,3): 6·e^{1/√6}
        let expected = 6.0 * (1.0 / 6.0_f64.sqrt()).exp();
        assert!((exponential_sum_connectivity(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn esc_star5() {
        // 4 edges (4,1): 4·e^{1/√5}
        let expected = 4.0 * (1.0 / 5.0_f64.sqrt()).exp();
        assert!((exponential_sum_connectivity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn esc_cycle4() {
        // 4 edges (2,2): 4·e^{1/2}
        assert!(
            (exponential_sum_connectivity(&cycle4()).unwrap() - 4.0 * 0.5_f64.exp()).abs() < 1e-10
        );
    }

    #[test]
    fn esc_paw() {
        // (0,1)d=(2,2): e^{1/2}
        // (0,2)d=(2,3): e^{1/√5}
        // (1,2)d=(2,3): e^{1/√5}
        // (2,3)d=(3,1): e^{1/2}
        let expected = 2.0 * 0.5_f64.exp() + 2.0 * (1.0 / 5.0_f64.sqrt()).exp();
        assert!((exponential_sum_connectivity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn efz_ge_n() {
        // e^{d²} ≥ 1, so eM₁ ≥ n
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(exponential_first_zagreb(g).unwrap() >= f64::from(g.vcount()) - 1e-10);
        }
    }

    #[test]
    fn esc_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(exponential_sum_connectivity(g).unwrap() > 0.0);
        }
    }
}
