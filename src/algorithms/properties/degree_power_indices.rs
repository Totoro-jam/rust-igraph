//! Degree-power indices (ALGO-TR-077).
//!
//! Bond-additive indices that raise degree products/sums to general
//! powers, generalizing many classical indices:
//!
//! - **General zeroth-order Randić** `⁰R_α(G) = Σ_v d(v)^α`
//! - **General first Zagreb** `M₁^α(G) = Σ_v d(v)^α` (same as above
//!   but conventional name uses α=2 for classical M₁)
//! - **Variable sum exdeg** `SEI_a(G) = Σ_v d(v)·a^{d(v)}` for a>0
//! - **Inverse degree power** `ID_k(G) = Σ_v 1/d(v)^k` for d(v)>0

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the general zeroth-order Randić index.
///
/// `⁰R_α(G) = Σ_v d(v)^α`
///
/// For `α = 2` this equals the first Zagreb index.
/// For `α = 3` this equals the forgotten index (F-index).
/// For `α = -1` this equals the inverse degree index.
/// Vertices with degree 0 are skipped when `α < 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, general_zeroth_order_randic};
///
/// // K_3: d=(2,2,2), α=2 → 3·4 = 12 (= first Zagreb index)
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((general_zeroth_order_randic(&g, 2.0).unwrap() - 12.0).abs() < 1e-10);
/// ```
pub fn general_zeroth_order_randic(graph: &Graph, alpha: f64) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d == 0 {
            if alpha >= 0.0 {
                // 0^α = 0 for α > 0, 0^0 = 1 by convention
                if alpha == 0.0 {
                    result += 1.0;
                }
            }
            continue;
        }
        result += (d as f64).powf(alpha);
    }

    Ok(result)
}

/// Compute the variable sum exdeg index.
///
/// `SEI_a(G) = Σ_v d(v) · a^{d(v)}` for `a > 0, a ≠ 1`.
///
/// Returns 0.0 for empty graphs or when `a ≤ 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, variable_sum_exdeg};
///
/// // K_3: d=(2,2,2), a=2 → 3·2·4 = 24
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((variable_sum_exdeg(&g, 2.0).unwrap() - 24.0).abs() < 1e-10);
/// ```
pub fn variable_sum_exdeg(graph: &Graph, a: f64) -> IgraphResult<f64> {
    if a <= 0.0 {
        return Ok(0.0);
    }

    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d == 0 {
            continue;
        }
        result += (d as f64) * a.powf(d as f64);
    }

    Ok(result)
}

/// Compute the inverse degree power index.
///
/// `ID_k(G) = Σ_v 1/d(v)^k` for `d(v) > 0`.
///
/// For `k = 1` this equals the inverse degree index.
/// Vertices with degree 0 are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, inverse_degree_power};
///
/// // K_3: d=(2,2,2), k=2 → 3·(1/4) = 0.75
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((inverse_degree_power(&g, 2.0).unwrap() - 0.75).abs() < 1e-10);
/// ```
pub fn inverse_degree_power(graph: &Graph, k: f64) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let mut result = 0.0_f64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d == 0 {
            continue;
        }
        result += 1.0 / (d as f64).powf(k);
    }

    Ok(result)
}

/// Compute the variable first Zagreb index.
///
/// `M₁^{(p)}(G) = Σ_{(u,v)∈E} (du^p + dv^p)` for real `p`.
///
/// Generalizes: `p=1` → first Zagreb, `p=2` → forgotten index.
/// Edges with degree-0 endpoint are skipped when `p < 0`.
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, variable_first_zagreb};
///
/// // K_3: 3 edges, d=(2,2), p=1 → 3·(2+2) = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((variable_first_zagreb(&g, 1.0).unwrap() - 12.0).abs() < 1e-10);
/// ```
pub fn variable_first_zagreb(graph: &Graph, p: f64) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if p < 0.0 && (du == 0.0 || dv == 0.0) {
            continue;
        }
        result += du.powf(p) + dv.powf(p);
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

    // --- general_zeroth_order_randic ---

    #[test]
    fn gzr_empty() {
        let g = Graph::with_vertices(0);
        assert!(general_zeroth_order_randic(&g, 2.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gzr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(general_zeroth_order_randic(&g, 2.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gzr_alpha2_is_first_zagreb() {
        // α=2: Σ d² = first Zagreb index
        // K_3: 3·4=12
        assert!((general_zeroth_order_randic(&k3(), 2.0).unwrap() - 12.0).abs() < 1e-10);
        // K_4: 4·9=36
        assert!((general_zeroth_order_randic(&k4(), 2.0).unwrap() - 36.0).abs() < 1e-10);
    }

    #[test]
    fn gzr_alpha3_is_forgotten() {
        // α=3: Σ d³ = forgotten index
        // K_3: 3·8=24
        assert!((general_zeroth_order_randic(&k3(), 3.0).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn gzr_alpha1_is_twice_edges() {
        // α=1: Σ d = 2m
        for g in &[k3(), k4(), cycle4(), star5(), paw()] {
            let expected = 2.0 * g.ecount() as f64;
            assert!((general_zeroth_order_randic(g, 1.0).unwrap() - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn gzr_alpha_neg1_is_inverse_degree() {
        // α=-1: Σ 1/d
        // K_3: 3·(1/2) = 1.5
        assert!((general_zeroth_order_randic(&k3(), -1.0).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn gzr_single_edge() {
        // d=(1,1), α=2: 1+1=2
        assert!((general_zeroth_order_randic(&single_edge(), 2.0).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn gzr_star5() {
        // d=(4,1,1,1,1), α=2: 16+1+1+1+1=20
        assert!((general_zeroth_order_randic(&star5(), 2.0).unwrap() - 20.0).abs() < 1e-10);
    }

    // --- variable_sum_exdeg ---

    #[test]
    fn vse_empty() {
        let g = Graph::with_vertices(0);
        assert!(variable_sum_exdeg(&g, 2.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vse_isolated() {
        let g = Graph::with_vertices(5);
        assert!(variable_sum_exdeg(&g, 2.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vse_k3_a2() {
        // d=(2,2,2), a=2: 3·2·2²=24
        assert!((variable_sum_exdeg(&k3(), 2.0).unwrap() - 24.0).abs() < 1e-10);
    }

    #[test]
    fn vse_single_edge_a2() {
        // d=(1,1), a=2: 2·1·2=4
        assert!((variable_sum_exdeg(&single_edge(), 2.0).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn vse_path3_a2() {
        // d=(1,2,1): 1·2 + 2·4 + 1·2 = 2+8+2 = 12
        assert!((variable_sum_exdeg(&path3(), 2.0).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn vse_star5_a2() {
        // d=(4,1,1,1,1): 4·16 + 4·(1·2) = 64+8 = 72
        assert!((variable_sum_exdeg(&star5(), 2.0).unwrap() - 72.0).abs() < 1e-10);
    }

    #[test]
    fn vse_k4_a3() {
        // d=(3,3,3,3), a=3: 4·3·27=324
        assert!((variable_sum_exdeg(&k4(), 3.0).unwrap() - 324.0).abs() < 1e-10);
    }

    #[test]
    fn vse_invalid_a() {
        // a ≤ 0 → 0
        assert!(variable_sum_exdeg(&k3(), 0.0).unwrap().abs() < 1e-10);
        assert!(variable_sum_exdeg(&k3(), -1.0).unwrap().abs() < 1e-10);
    }

    // --- inverse_degree_power ---

    #[test]
    fn idp_empty() {
        let g = Graph::with_vertices(0);
        assert!(inverse_degree_power(&g, 1.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn idp_isolated() {
        let g = Graph::with_vertices(5);
        assert!(inverse_degree_power(&g, 1.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn idp_k1_is_inverse_degree() {
        // k=1: Σ 1/d
        // K_3: 3·(1/2) = 1.5
        assert!((inverse_degree_power(&k3(), 1.0).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn idp_k2_k3() {
        // k=2: Σ 1/d² = 3·(1/4) = 0.75
        assert!((inverse_degree_power(&k3(), 2.0).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn idp_single_edge() {
        // d=(1,1), k=2: 2·(1/1)=2
        assert!((inverse_degree_power(&single_edge(), 2.0).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn idp_star5() {
        // d=(4,1,1,1,1), k=1: 1/4+4·1 = 4.25
        assert!((inverse_degree_power(&star5(), 1.0).unwrap() - 4.25).abs() < 1e-10);
    }

    #[test]
    fn idp_paw() {
        // d=(2,2,3,1), k=1: 1/2+1/2+1/3+1 = 2+1/3
        let expected = 0.5 + 0.5 + 1.0 / 3.0 + 1.0;
        assert!((inverse_degree_power(&paw(), 1.0).unwrap() - expected).abs() < 1e-10);
    }

    // --- variable_first_zagreb ---

    #[test]
    fn vfz_empty() {
        let g = Graph::with_vertices(0);
        assert!(variable_first_zagreb(&g, 1.0).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vfz_p1_is_m1() {
        // p=1: Σ_edges (du+dv) = first Zagreb index
        // K_3: 3·4=12
        assert!((variable_first_zagreb(&k3(), 1.0).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn vfz_p2_is_forgotten() {
        // p=2: Σ_edges (du²+dv²) = forgotten index
        // K_3: 3·(4+4)=24
        assert!((variable_first_zagreb(&k3(), 2.0).unwrap() - 24.0).abs() < 1e-10);
        // K_4: 6·(9+9)=108
        assert!((variable_first_zagreb(&k4(), 2.0).unwrap() - 108.0).abs() < 1e-10);
    }

    #[test]
    fn vfz_single_edge() {
        // d=(1,1), p=1: 1+1=2
        assert!((variable_first_zagreb(&single_edge(), 1.0).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn vfz_path3() {
        // 2 edges d=(1,2): (1+2)+(2+1)=6
        assert!((variable_first_zagreb(&path3(), 1.0).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn vfz_star5() {
        // 4 edges d=(4,1): each (4+1)=5 → 4·5=20
        assert!((variable_first_zagreb(&star5(), 1.0).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn vfz_paw_p2() {
        // (0,1) d=(2,2): 4+4=8
        // (0,2) d=(2,3): 4+9=13
        // (1,2) d=(2,3): 4+9=13
        // (2,3) d=(3,1): 9+1=10
        let expected = 8.0 + 13.0 + 13.0 + 10.0;
        assert!((variable_first_zagreb(&paw(), 2.0).unwrap() - expected).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn gzr_alpha0_is_vertex_count_nonzero() {
        // α=0: Σ d^0 = number of non-isolated vertices + isolated (0^0=1)
        // But we define 0^0=1 and skip degree-0 with count
        // Actually: for d>0, d^0=1; for d=0, we add 1 (α==0)
        // So it's just n (vertex count)
        let g = &k3();
        assert!((general_zeroth_order_randic(g, 0.0).unwrap() - 3.0).abs() < 1e-10);

        // With isolated vertices
        let iso = Graph::with_vertices(5);
        assert!((general_zeroth_order_randic(&iso, 0.0).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn idp_k0_is_nonzero_count() {
        // k=0: Σ 1/d^0 = Σ 1 = number of non-isolated vertices
        assert!((inverse_degree_power(&k3(), 0.0).unwrap() - 3.0).abs() < 1e-10);
        assert!((inverse_degree_power(&star5(), 0.0).unwrap() - 5.0).abs() < 1e-10);
    }
}
