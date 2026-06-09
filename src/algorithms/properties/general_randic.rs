//! General Randić index and harmonic variants (ALGO-TR-050).
//!
//! - **General Randić index** `R_α(G) = Σ_{(u,v)∈E} (d_u · d_v)^α`
//!   The classical Randić index is the special case α = −½.
//!   The second Zagreb index is α = 1. The general form was introduced
//!   by Bollobás & Erdős (1998).
//! - **General sum-connectivity index** `χ_α(G) = Σ_{(u,v)∈E} (d_u + d_v)^α`
//!   Generalisation of the sum-connectivity index (α = −½) and the
//!   first Zagreb index (α = 1). Introduced by Zhou & Trinajstić (2010).
//! - **Reciprocal Randić index** `RR(G) = Σ_{(u,v)∈E} √(d_u · d_v)`
//!   The Randić index with α = +½ instead of −½. Measures branching
//!   in the opposite direction.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the general Randić index `R_α(G)`.
///
/// `R_α(G) = Σ_{(u,v)∈E} (d_u · d_v)^α`
///
/// Special cases: α = −0.5 gives the classical Randić index;
/// α = 1 gives the second Zagreb index `M₂`.
///
/// Self-loops and edges with `d_u · d_v = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, general_randic_index};
///
/// // K_3: all degrees 2, α = 1 → 3 · (2·2)^1 = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((general_randic_index(&g, 1.0).unwrap() - 12.0).abs() < 1e-10);
/// ```
pub fn general_randic_index(graph: &Graph, alpha: f64) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut r = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let prod = du * dv;
        if prod <= 0.0 {
            continue;
        }
        r += prod.powf(alpha);
    }

    Ok(r)
}

/// Compute the general sum-connectivity index `χ_α(G)`.
///
/// `χ_α(G) = Σ_{(u,v)∈E} (d_u + d_v)^α`
///
/// Special cases: α = −0.5 gives the sum-connectivity index;
/// α = 1 gives the first Zagreb index `M₁`.
///
/// Self-loops and edges with `d_u + d_v = 0` are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, general_sum_connectivity_index};
///
/// // K_3: all degrees 2, α = 1 → 3 · (2+2)^1 = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((general_sum_connectivity_index(&g, 1.0).unwrap() - 12.0).abs() < 1e-10);
/// ```
pub fn general_sum_connectivity_index(graph: &Graph, alpha: f64) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut chi = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let sum_d = du + dv;
        if sum_d <= 0.0 {
            continue;
        }
        chi += sum_d.powf(alpha);
    }

    Ok(chi)
}

/// Compute the reciprocal Randić index.
///
/// `RR(G) = Σ_{(u,v)∈E} √(d_u · d_v)`
///
/// This is `R_{+½}(G)`, the general Randić index with α = +½.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocal_randic_index};
///
/// // K_3: all degrees 2 → 3·√(2·2) = 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((reciprocal_randic_index(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn reciprocal_randic_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(0.0);
    }

    let mut rr = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let prod = du * dv;
        if prod <= 0.0 {
            continue;
        }
        rr += prod.sqrt();
    }

    Ok(rr)
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

    // --- general_randic_index ---

    #[test]
    fn gr_empty() {
        let g = Graph::with_vertices(0);
        assert!((general_randic_index(&g, -0.5).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gr_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((general_randic_index(&g, 1.0).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gr_single_edge() {
        // α=1: (1·1)^1 = 1
        assert!((general_randic_index(&single_edge(), 1.0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gr_alpha_neg_half_is_randic() {
        // classical Randić: Σ 1/√(d_u·d_v)
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let gr = general_randic_index(g, -0.5).unwrap();
            let mut expected = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                expected += 1.0 / (du * dv).sqrt();
            }
            assert!((gr - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn gr_alpha_1_is_second_zagreb() {
        // α=1: Σ d_u·d_v = M₂
        for g in &[single_edge(), path3(), k3(), k4(), star5()] {
            let gr = general_randic_index(g, 1.0).unwrap();
            let mut m2 = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                m2 += du * dv;
            }
            assert!((gr - m2).abs() < 1e-8);
        }
    }

    #[test]
    fn gr_alpha_0_equals_m() {
        // α=0: (d_u·d_v)^0 = 1 per edge → sum = m
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let gr = general_randic_index(g, 0.0).unwrap();
            assert!((gr - g.ecount() as f64).abs() < 1e-10);
        }
    }

    #[test]
    fn gr_k3() {
        // α=1: 3·(2·2) = 12
        assert!((general_randic_index(&k3(), 1.0).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn gr_k4() {
        // α=1: 6·(3·3) = 54
        assert!((general_randic_index(&k4(), 1.0).unwrap() - 54.0).abs() < 1e-10);
    }

    #[test]
    fn gr_star5_alpha1() {
        // center=4, leaf=1: 4·(4·1)=16
        assert!((general_randic_index(&star5(), 1.0).unwrap() - 16.0).abs() < 1e-10);
    }

    #[test]
    fn gr_path3_alpha2() {
        // (0,1): (1·2)²=4, (1,2): (2·1)²=4 → 8
        assert!((general_randic_index(&path3(), 2.0).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn gr_regular_formula() {
        // r-regular: R_α = m · (r²)^α = m · r^(2α)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            for &alpha in &[-0.5_f64, 0.0, 0.5, 1.0, 2.0] {
                let expected = m * r.powf(2.0 * alpha);
                let actual = general_randic_index(g, alpha).unwrap();
                assert!(
                    (actual - expected).abs() < 1e-6,
                    "alpha={alpha}, expected={expected}, got={actual}"
                );
            }
        }
    }

    // --- general_sum_connectivity_index ---

    #[test]
    fn gs_empty() {
        let g = Graph::with_vertices(0);
        assert!((general_sum_connectivity_index(&g, -0.5).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn gs_alpha_neg_half_is_sci() {
        // χ_{-½} = Σ 1/√(d_u+d_v) = SCI
        for g in &[single_edge(), path3(), k3(), star5()] {
            let gs = general_sum_connectivity_index(g, -0.5).unwrap();
            let mut expected = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                expected += 1.0 / (du + dv).sqrt();
            }
            assert!((gs - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn gs_alpha_1_is_first_zagreb() {
        // χ_1 = Σ (d_u+d_v) = M₁
        for g in &[single_edge(), path3(), k3(), k4(), star5()] {
            let gs = general_sum_connectivity_index(g, 1.0).unwrap();
            let mut m1 = 0.0_f64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as f64;
                let dv = g.degree(v).unwrap() as f64;
                m1 += du + dv;
            }
            assert!((gs - m1).abs() < 1e-8);
        }
    }

    #[test]
    fn gs_alpha_0_equals_m() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let gs = general_sum_connectivity_index(g, 0.0).unwrap();
            assert!((gs - g.ecount() as f64).abs() < 1e-10);
        }
    }

    #[test]
    fn gs_k3_alpha1() {
        // 3·(2+2) = 12
        assert!((general_sum_connectivity_index(&k3(), 1.0).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn gs_k4_alpha1() {
        // 6·(3+3) = 36
        assert!((general_sum_connectivity_index(&k4(), 1.0).unwrap() - 36.0).abs() < 1e-10);
    }

    #[test]
    fn gs_star5_alpha1() {
        // 4·(4+1) = 20
        assert!((general_sum_connectivity_index(&star5(), 1.0).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn gs_regular_formula() {
        // r-regular: χ_α = m · (2r)^α
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            for &alpha in &[-0.5_f64, 0.0, 0.5, 1.0, 2.0] {
                let expected = m * (2.0 * r).powf(alpha);
                let actual = general_sum_connectivity_index(g, alpha).unwrap();
                assert!((actual - expected).abs() < 1e-6);
            }
        }
    }

    // --- reciprocal_randic_index ---

    #[test]
    fn rr_empty() {
        let g = Graph::with_vertices(0);
        assert!((reciprocal_randic_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rr_single_edge() {
        // √(1·1) = 1
        assert!((reciprocal_randic_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_path3() {
        // (0,1):√2, (1,2):√2 → 2√2
        let expected = 2.0 * 2.0_f64.sqrt();
        assert!((reciprocal_randic_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rr_k3() {
        // 3·√4 = 6
        assert!((reciprocal_randic_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn rr_k4() {
        // 6·√9 = 18
        assert!((reciprocal_randic_index(&k4()).unwrap() - 18.0).abs() < 1e-10);
    }

    #[test]
    fn rr_star5() {
        // 4·√4 = 8
        assert!((reciprocal_randic_index(&star5()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn rr_cycle4() {
        // 4·√4 = 8
        assert!((reciprocal_randic_index(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn rr_equals_gr_half() {
        // RR = R_{+½}
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let rr = reciprocal_randic_index(g).unwrap();
            let gr = general_randic_index(g, 0.5).unwrap();
            assert!((rr - gr).abs() < 1e-8);
        }
    }

    #[test]
    fn rr_regular_formula() {
        // r-regular: RR = m·r
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            assert!((reciprocal_randic_index(g).unwrap() - m * r).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(general_randic_index(g, 1.0).unwrap() > 0.0);
            assert!(general_sum_connectivity_index(g, 1.0).unwrap() > 0.0);
            assert!(reciprocal_randic_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn gr_geq_rr_for_alpha_geq_half() {
        // R_1 ≥ R_{½} since (d_u·d_v)^1 ≥ (d_u·d_v)^{½} when d_u·d_v ≥ 1
        for g in &[single_edge(), path3(), k3(), k4(), star5()] {
            let r1 = general_randic_index(g, 1.0).unwrap();
            let rr = reciprocal_randic_index(g).unwrap();
            assert!(r1 >= rr - 1e-8);
        }
    }

    #[test]
    fn paw_alpha1() {
        // degrees [2,2,3,1]
        // R_1: (0,1):4, (0,2):6, (1,2):6, (2,3):3 → 19
        assert!((general_randic_index(&paw(), 1.0).unwrap() - 19.0).abs() < 1e-10);
    }

    #[test]
    fn path4_alpha1() {
        // degrees [1,2,2,1]
        // R_1: (0,1):2, (1,2):4, (2,3):2 → 8
        assert!((general_randic_index(&path4(), 1.0).unwrap() - 8.0).abs() < 1e-10);
    }
}
