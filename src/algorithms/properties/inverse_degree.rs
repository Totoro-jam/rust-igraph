//! Inverse degree and Zagreb coindex (ALGO-TR-056).
//!
//! - **Inverse degree index** `ID(G) = Σ_{v∈V, d(v)≥1} 1/d(v)`
//!   (Also called the zeroth-order Randić index.) Sum of reciprocals of
//!   non-zero vertex degrees. Introduced by Fajtlowicz (1987).
//! - **First Zagreb coindex** `\bar{M}_1(G) = Σ_{(u,v)∉E, u≠v} (d(u)+d(v))`
//!   Sum of degree sums over non-edges. Introduced by Došlić (2008).
//!   Computed via the identity `\bar{M}_1 = 2m(n-1) - M_1`.
//! - **Second Zagreb coindex** `\bar{M}_2(G) = Σ_{(u,v)∉E, u≠v} d(u)·d(v)`
//!   Product of degrees over non-edges. Computed via
//!   `\bar{M}_2 = 2m² - M_2` where `M_2` is the second Zagreb index.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the inverse degree index (zeroth-order Randić index).
///
/// `ID(G) = Σ_{v∈V, d(v)≥1} 1/d(v)`
///
/// Isolated vertices are excluded.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, inverse_degree_index};
///
/// // Star S_4: degrees [4,1,1,1,1] → ID = 1/4 + 4·1 = 4.25
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((inverse_degree_index(&g).unwrap() - 4.25).abs() < 1e-10);
/// ```
pub fn inverse_degree_index(graph: &Graph) -> IgraphResult<f64> {
    let mut id = 0.0_f64;

    for v in 0..graph.vcount() {
        let d = graph.degree(v)?;
        if d > 0 {
            id += 1.0 / d as f64;
        }
    }

    Ok(id)
}

/// Compute the first Zagreb coindex.
///
/// `\bar{M}_1(G) = Σ_{(u,v)∉E, u≠v} (d(u) + d(v))`
///
/// Uses the identity: `\bar{M}_1 = 2m(n-1) - M_1` where `M_1 = Σ_v d(v)²`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_zagreb_coindex};
///
/// // Path 0-1-2: M₁ = 1+4+1 = 6, m=2, n=3
/// // bar_M₁ = 2·2·2 - 6 = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(first_zagreb_coindex(&g).unwrap(), 2);
/// ```
pub fn first_zagreb_coindex(graph: &Graph) -> IgraphResult<i64> {
    let n = i64::from(graph.vcount());
    let m = graph.ecount() as i64;

    let mut m1: i64 = 0;
    for v in 0..graph.vcount() {
        let d = graph.degree(v)? as i64;
        m1 = m1.saturating_add(d.saturating_mul(d));
    }

    Ok(2_i64
        .saturating_mul(m)
        .saturating_mul(n - 1)
        .saturating_sub(m1))
}

/// Compute the second Zagreb coindex.
///
/// `\bar{M}_2(G) = Σ_{(u,v)∉E, u≠v} d(u) · d(v)`
///
/// Uses the identity: `\bar{M}_2 = 2m² - M_2` where
/// `M_2 = Σ_{(u,v)∈E} d(u)·d(v)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_zagreb_coindex};
///
/// // K_3: M₂ = 3·(2·2)=12, m=3 → bar_M₂ = 2·9-12 = 6
/// // But K_3 has no non-edges, so bar_M₂ = 0. Let's check:
/// // 2m² = 18, M₂ = 12 → 18-12 = 6? No, for K_3 there are no non-edges.
/// // The identity is: Σ_{u<v} d(u)d(v) = M₂ + bar_M₂
/// // Σ_{u<v} d(u)d(v) = (Σ d(v))²/2 - Σ d(v)²/2 = (2m)²/2 - M₁/2
/// // = 2m² - M₁/2. So bar_M₂ = 2m² - M₁/2 - M₂.
/// // For path 0-1-2: m=2, M₁=6, M₂=2+2=4
/// // bar_M₂ = 2·4 - 6/2 - 4 = 8-3-4 = 1
/// // Non-edge (0,2): d(0)·d(2) = 1·1 = 1. ✓
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_zagreb_coindex(&g).unwrap(), 1);
/// ```
pub fn second_zagreb_coindex(graph: &Graph) -> IgraphResult<i64> {
    let mut sum_d: i64 = 0;
    let mut sum_d2: i64 = 0;
    for v in 0..graph.vcount() {
        let d = graph.degree(v)? as i64;
        sum_d = sum_d.saturating_add(d);
        sum_d2 = sum_d2.saturating_add(d.saturating_mul(d));
    }

    let mut m2: i64 = 0;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as i64;
        let dv = graph.degree(v)? as i64;
        m2 = m2.saturating_add(du.saturating_mul(dv));
    }

    // Σ_{u<v} d(u)d(v) = ((Σd)² - Σd²) / 2
    let all_pairs_prod = (sum_d.saturating_mul(sum_d) - sum_d2) / 2;
    Ok(all_pairs_prod - m2)
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

    // --- inverse_degree_index ---

    #[test]
    fn id_empty() {
        let g = Graph::with_vertices(0);
        assert!((inverse_degree_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn id_isolated() {
        let g = Graph::with_vertices(5);
        assert!((inverse_degree_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn id_single_edge() {
        assert!((inverse_degree_index(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn id_path3() {
        // 1/1 + 1/2 + 1/1 = 2.5
        assert!((inverse_degree_index(&path3()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn id_path4() {
        // 1/1 + 1/2 + 1/2 + 1/1 = 3
        assert!((inverse_degree_index(&path4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn id_k3() {
        assert!((inverse_degree_index(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn id_k4() {
        assert!((inverse_degree_index(&k4()).unwrap() - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn id_star5() {
        assert!((inverse_degree_index(&star5()).unwrap() - 4.25).abs() < 1e-10);
    }

    #[test]
    fn id_cycle4() {
        // 4 · (1/2) = 2
        assert!((inverse_degree_index(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn id_cycle5() {
        // 5 · (1/2) = 2.5
        assert!((inverse_degree_index(&cycle5()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn id_paw() {
        // 1/2 + 1/2 + 1/3 + 1/1 = 7/3
        assert!((inverse_degree_index(&paw()).unwrap() - 7.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn id_regular_is_n_over_r() {
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = f64::from(g.vcount());
            let r = g.degree(0).unwrap() as f64;
            assert!((inverse_degree_index(g).unwrap() - n / r).abs() < 1e-8);
        }
    }

    #[test]
    fn id_with_isolated() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((inverse_degree_index(&g).unwrap() - 2.0).abs() < 1e-10);
    }

    // --- first_zagreb_coindex ---

    #[test]
    fn zco1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn zco1_single_edge() {
        assert_eq!(first_zagreb_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn zco1_path3() {
        assert_eq!(first_zagreb_coindex(&path3()).unwrap(), 2);
    }

    #[test]
    fn zco1_path4() {
        // m=3, n=4, M₁=1+4+4+1=10: 2·3·3-10=8
        assert_eq!(first_zagreb_coindex(&path4()).unwrap(), 8);
    }

    #[test]
    fn zco1_k3() {
        assert_eq!(first_zagreb_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn zco1_k4() {
        assert_eq!(first_zagreb_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn zco1_cycle4() {
        // m=4, n=4, M₁=16: 2·4·3-16=8
        assert_eq!(first_zagreb_coindex(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn zco1_cycle5() {
        // m=5, n=5, M₁=20: 2·5·4-20=20
        assert_eq!(first_zagreb_coindex(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn zco1_star5() {
        // m=4, n=5, M₁=16+4=20: 2·4·4-20=12
        assert_eq!(first_zagreb_coindex(&star5()).unwrap(), 12);
    }

    #[test]
    fn zco1_paw() {
        // m=4, n=4, M₁=4+4+9+1=18: 2·4·3-18=6
        assert_eq!(first_zagreb_coindex(&paw()).unwrap(), 6);
    }

    #[test]
    fn zco1_zero_for_complete() {
        assert_eq!(first_zagreb_coindex(&k3()).unwrap(), 0);
        assert_eq!(first_zagreb_coindex(&k4()).unwrap(), 0);
    }

    // --- second_zagreb_coindex ---

    #[test]
    fn zco2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_zagreb_coindex(&g).unwrap(), 0);
    }

    #[test]
    fn zco2_single_edge() {
        assert_eq!(second_zagreb_coindex(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn zco2_path3() {
        // Non-edge (0,2): 1·1 = 1
        assert_eq!(second_zagreb_coindex(&path3()).unwrap(), 1);
    }

    #[test]
    fn zco2_path4() {
        // Non-edges: (0,2):1·2=2, (0,3):1·1=1, (1,3):2·1=2 → 5
        assert_eq!(second_zagreb_coindex(&path4()).unwrap(), 5);
    }

    #[test]
    fn zco2_k3() {
        assert_eq!(second_zagreb_coindex(&k3()).unwrap(), 0);
    }

    #[test]
    fn zco2_k4() {
        assert_eq!(second_zagreb_coindex(&k4()).unwrap(), 0);
    }

    #[test]
    fn zco2_cycle4() {
        // Non-edges: (0,2):2·2=4, (1,3):2·2=4 → 8
        assert_eq!(second_zagreb_coindex(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn zco2_cycle5() {
        // Non-edges: 5 pairs at dist 2, each 2·2=4 → 20
        assert_eq!(second_zagreb_coindex(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn zco2_star5() {
        // Non-edges: C(4,2)=6 leaf-leaf pairs, each 1·1=1 → 6
        assert_eq!(second_zagreb_coindex(&star5()).unwrap(), 6);
    }

    #[test]
    fn zco2_paw() {
        // degrees [2,2,3,1], edges: (0,1),(0,2),(1,2),(2,3)
        // Non-edges: (0,3):2·1=2, (1,3):2·1=2 → 4
        assert_eq!(second_zagreb_coindex(&paw()).unwrap(), 4);
    }

    #[test]
    fn zco2_zero_for_complete() {
        assert_eq!(second_zagreb_coindex(&k3()).unwrap(), 0);
        assert_eq!(second_zagreb_coindex(&k4()).unwrap(), 0);
    }

    // --- cross-consistency ---

    #[test]
    fn coindices_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_zagreb_coindex(g).unwrap() >= 0);
            assert!(second_zagreb_coindex(g).unwrap() >= 0);
        }
    }

    #[test]
    fn id_geq_1_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!(inverse_degree_index(g).unwrap() >= 1.0 - 1e-10);
        }
    }
}
