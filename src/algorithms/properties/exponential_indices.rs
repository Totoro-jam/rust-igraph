//! Exponential degree-based indices (ALGO-TR-073).
//!
//! Exponential versions of classical bond-additive indices, summing
//! `exp(f(du,dv))` over edges. Well-studied in mathematical chemistry
//! for their improved discriminating power:
//!
//! - **Exponential augmented Zagreb** `EAZ(G) = Σ_{(u,v)∈E} exp(du·dv/(du+dv-2))`
//! - **Exponential Randić** `ER(G) = Σ_{(u,v)∈E} exp(1/√(du·dv))`
//! - **Exponential atom-bond connectivity** `EABC(G) = Σ exp(√((du+dv-2)/(du·dv)))`
//! - **Exponential geometric-arithmetic** `EGA(G) = Σ exp(2√(du·dv)/(du+dv))`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the exponential augmented Zagreb index.
///
/// `EAZ(G) = Σ_{(u,v)∈E} exp(du·dv / (du+dv-2))`
///
/// Edges where `du + dv - 2 == 0` (i.e. both endpoints have degree 1,
/// which requires a multi-edge or a single-edge graph with n=2) contribute
/// `exp(+∞)` → skipped with a contribution of 0. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_augmented_zagreb};
///
/// // K_3: 3 edges, d=(2,2), each exp(4/2) = exp(2) → 3·exp(2)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 * 2.0_f64.exp();
/// assert!((exponential_augmented_zagreb(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn exponential_augmented_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let denom = du + dv - 2.0;
        if denom > 0.0 {
            result += (du * dv / denom).exp();
        }
    }

    Ok(result)
}

/// Compute the exponential Randić index.
///
/// `ER(G) = Σ_{(u,v)∈E} exp(1 / √(du·dv))`
///
/// Edges with a degree-0 endpoint are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_randic};
///
/// // K_3: 3 edges, d=(2,2), each exp(1/2) → 3·exp(0.5)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 * 0.5_f64.exp();
/// assert!((exponential_randic(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn exponential_randic(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        if product > 0.0 {
            result += (1.0 / product.sqrt()).exp();
        }
    }

    Ok(result)
}

/// Compute the exponential atom-bond connectivity index.
///
/// `EABC(G) = Σ_{(u,v)∈E} exp(√((du+dv-2) / (du·dv)))`
///
/// Edges with a degree-0 endpoint or `du+dv-2 < 0` are skipped.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_abc};
///
/// // K_3: 3 edges, d=(2,2), each exp(√(2/4)) = exp(1/√2) → 3·exp(1/√2)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 * (1.0_f64 / 2.0_f64.sqrt()).exp();
/// assert!((exponential_abc(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn exponential_abc(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let product = du * dv;
        let numer = du + dv - 2.0;
        if product > 0.0 && numer >= 0.0 {
            result += (numer / product).sqrt().exp();
        }
    }

    Ok(result)
}

/// Compute the exponential geometric-arithmetic index.
///
/// `EGA(G) = Σ_{(u,v)∈E} exp(2√(du·dv) / (du+dv))`
///
/// Edges with `du+dv == 0` are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, exponential_ga};
///
/// // K_3: 3 edges, d=(2,2), each exp(2·2/4) = exp(1) = e → 3e
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 * 1.0_f64.exp();
/// assert!((exponential_ga(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn exponential_ga(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let sum = du + dv;
        if sum > 0.0 {
            result += (2.0 * (du * dv).sqrt() / sum).exp();
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

    fn cycle5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- exponential_augmented_zagreb ---

    #[test]
    fn eaz_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_augmented_zagreb(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eaz_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_augmented_zagreb(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eaz_single_edge() {
        // d=(1,1), du+dv-2=0 → skipped → 0
        assert!(exponential_augmented_zagreb(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eaz_k3() {
        // d=(2,2), du*dv/(du+dv-2) = 4/2 = 2, exp(2)
        let expected = 3.0 * 2.0_f64.exp();
        assert!((exponential_augmented_zagreb(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_k4() {
        // d=(3,3), 9/4 = 2.25, exp(2.25) per edge, 6 edges
        let expected = 6.0 * 2.25_f64.exp();
        assert!((exponential_augmented_zagreb(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_path3() {
        // 2 edges d=(1,2): 1*2/(1+2-2) = 2/1 = 2, exp(2) each
        let expected = 2.0 * 2.0_f64.exp();
        assert!((exponential_augmented_zagreb(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_cycle4() {
        // d=(2,2), 4/2=2, exp(2) per edge, 4 edges
        let expected = 4.0 * 2.0_f64.exp();
        assert!((exponential_augmented_zagreb(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_cycle5() {
        let expected = 5.0 * 2.0_f64.exp();
        assert!((exponential_augmented_zagreb(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_star5() {
        // 4 edges d=(4,1): 4*1/(4+1-2) = 4/3, exp(4/3) each
        let expected = 4.0 * (4.0_f64 / 3.0).exp();
        assert!((exponential_augmented_zagreb(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eaz_paw() {
        // (0,1) d=(2,2): exp(2)
        // (0,2) d=(2,3): exp(6/3)=exp(2)
        // (1,2) d=(2,3): exp(2)
        // (2,3) d=(3,1): exp(3/2)
        let expected = 3.0 * 2.0_f64.exp() + 1.5_f64.exp();
        assert!((exponential_augmented_zagreb(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- exponential_randic ---

    #[test]
    fn er_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_randic(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_randic(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn er_single_edge() {
        // d=(1,1), exp(1/1)=e
        let expected = 1.0_f64.exp();
        assert!((exponential_randic(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_k3() {
        // d=(2,2), exp(1/2) per edge
        let expected = 3.0 * 0.5_f64.exp();
        assert!((exponential_randic(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_k4() {
        // d=(3,3), exp(1/3) per edge
        let expected = 6.0 * (1.0_f64 / 3.0).exp();
        assert!((exponential_randic(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_path3() {
        // d=(1,2): exp(1/√2)
        let expected = 2.0 * (1.0 / 2.0_f64.sqrt()).exp();
        assert!((exponential_randic(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_cycle4() {
        // d=(2,2): exp(1/2) per edge
        let expected = 4.0 * 0.5_f64.exp();
        assert!((exponential_randic(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_cycle5() {
        let expected = 5.0 * 0.5_f64.exp();
        assert!((exponential_randic(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_star5() {
        // d=(4,1): exp(1/2)
        let expected = 4.0 * 0.5_f64.exp();
        assert!((exponential_randic(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn er_paw() {
        // (0,1) d=(2,2): exp(1/2)
        // (0,2) d=(2,3): exp(1/√6)
        // (1,2) d=(2,3): exp(1/√6)
        // (2,3) d=(3,1): exp(1/√3)
        let expected =
            0.5_f64.exp() + 2.0 * (1.0 / 6.0_f64.sqrt()).exp() + (1.0 / 3.0_f64.sqrt()).exp();
        assert!((exponential_randic(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- exponential_abc ---

    #[test]
    fn eabc_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_abc(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eabc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_abc(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eabc_single_edge() {
        // d=(1,1), (1+1-2)/(1*1)=0, exp(0)=1
        let expected = 1.0;
        assert!((exponential_abc(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_k3() {
        // d=(2,2), √(2/4)=1/√2, exp(1/√2)
        let expected = 3.0 * (1.0 / 2.0_f64.sqrt()).exp();
        assert!((exponential_abc(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_k4() {
        // d=(3,3), √(4/9)=2/3, exp(2/3)
        let expected = 6.0 * (2.0 / 3.0_f64).exp();
        assert!((exponential_abc(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_path3() {
        // d=(1,2): √((1+2-2)/(1*2))=√(1/2)=1/√2, exp(1/√2)
        let expected = 2.0 * (1.0 / 2.0_f64.sqrt()).exp();
        assert!((exponential_abc(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_cycle4() {
        // d=(2,2): √(2/4)=1/√2, exp(1/√2)
        let expected = 4.0 * (1.0 / 2.0_f64.sqrt()).exp();
        assert!((exponential_abc(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_star5() {
        // d=(4,1): √((4+1-2)/(4*1))=√(3/4)=√3/2, exp(√3/2)
        let expected = 4.0 * (3.0_f64.sqrt() / 2.0).exp();
        assert!((exponential_abc(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eabc_paw() {
        // (0,1) d=(2,2): exp(1/√2)
        // (0,2) d=(2,3): √(3/6)=√(1/2)=1/√2, exp(1/√2)
        // (1,2) d=(2,3): exp(1/√2)
        // (2,3) d=(3,1): √(2/3), exp(√(2/3))
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let expected = 3.0 * inv_sqrt2.exp() + (2.0_f64 / 3.0).sqrt().exp();
        assert!((exponential_abc(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- exponential_ga ---

    #[test]
    fn ega_empty() {
        let g = Graph::with_vertices(0);
        assert!(exponential_ga(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ega_isolated() {
        let g = Graph::with_vertices(5);
        assert!(exponential_ga(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ega_single_edge() {
        // d=(1,1), 2√1/2=1, exp(1)=e
        let expected = 1.0_f64.exp();
        assert!((exponential_ga(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_k3() {
        // d=(2,2), 2·2/4=1, exp(1)=e
        let expected = 3.0 * 1.0_f64.exp();
        assert!((exponential_ga(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_k4() {
        // d=(3,3), 2·3/6=1, exp(1)=e
        let expected = 6.0 * 1.0_f64.exp();
        assert!((exponential_ga(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_path3() {
        // d=(1,2): 2√2/3, exp(2√2/3)
        let expected = 2.0 * (2.0 * 2.0_f64.sqrt() / 3.0).exp();
        assert!((exponential_ga(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_cycle4() {
        // d=(2,2): exp(1) per edge
        let expected = 4.0 * 1.0_f64.exp();
        assert!((exponential_ga(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_cycle5() {
        let expected = 5.0 * 1.0_f64.exp();
        assert!((exponential_ga(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_star5() {
        // d=(4,1): 2·2/5=4/5, exp(4/5)
        let expected = 4.0 * 0.8_f64.exp();
        assert!((exponential_ga(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ega_paw() {
        // (0,1) d=(2,2): exp(1)
        // (0,2) d=(2,3): 2√6/5, exp(2√6/5)
        // (1,2) d=(2,3): exp(2√6/5)
        // (2,3) d=(3,1): 2√3/4=√3/2, exp(√3/2)
        let expected =
            1.0_f64.exp() + 2.0 * (2.0 * 6.0_f64.sqrt() / 5.0).exp() + (3.0_f64.sqrt() / 2.0).exp();
        assert!((exponential_ga(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn regular_ega_equals_m_times_e() {
        // For r-regular, GA ratio = 1, so EGA = m·e
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let expected = m * 1.0_f64.exp();
            assert!((exponential_ga(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn regular_er() {
        // For r-regular: ER = m·exp(1/r)
        let g = &k4();
        let m = g.ecount() as f64;
        let expected = m * (1.0_f64 / 3.0).exp();
        assert!((exponential_randic(g).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn all_positive_for_nonempty() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(exponential_randic(g).unwrap() > 0.0);
            assert!(exponential_abc(g).unwrap() > 0.0);
            assert!(exponential_ga(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn eaz_positive_for_connected_nonedge() {
        // EAZ skips single-edge (denom=0) but positive for others
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(exponential_augmented_zagreb(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn ega_ge_m_for_nontrivial() {
        // GA ratio ≤ 1, so EGA ≥ m·exp(0) = m... wait, GA ≤ 1 means
        // 2√(du·dv)/(du+dv) ≤ 1, so exp(≤1) ≥ 1, so EGA ≥ m.
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount() as f64;
            assert!(exponential_ga(g).unwrap() >= m - 1e-10);
        }
    }
}
