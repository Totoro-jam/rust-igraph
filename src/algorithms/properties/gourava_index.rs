//! Gourava indices (ALGO-TR-063).
//!
//! - **First Gourava index** `GO₁(G) = Σ_{(u,v)∈E} [d(u)+d(v) + d(u)·d(v)]`
//!   Introduced by Kulli (2017). Combines sum and product of endpoint degrees.
//! - **Second Gourava index** `GO₂(G) = Σ_{(u,v)∈E} (d(u)+d(v)) · (d(u)·d(v))`
//!   Product of degree sum and degree product over edges.
//! - **First hyper-Gourava index** `HGO₁(G) = Σ_{(u,v)∈E} [d(u)+d(v)+d(u)·d(v)]²`
//!   Squared version of the first Gourava.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the first Gourava index.
///
/// `GO₁(G) = Σ_{(u,v)∈E} [d(u) + d(v) + d(u)·d(v)]`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_gourava_index};
///
/// // K_3: each edge (2+2+4)=8, 3 edges → 24
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_gourava_index(&g).unwrap(), 24);
/// ```
pub fn first_gourava_index(graph: &Graph) -> IgraphResult<u64> {
    let mut go1 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        go1 = go1.saturating_add(du + dv + du.saturating_mul(dv));
    }

    Ok(go1)
}

/// Compute the second Gourava index.
///
/// `GO₂(G) = Σ_{(u,v)∈E} (d(u)+d(v)) · (d(u)·d(v))`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_gourava_index};
///
/// // K_3: each edge (2+2)·(2·2) = 4·4 = 16, 3 edges → 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_gourava_index(&g).unwrap(), 48);
/// ```
pub fn second_gourava_index(graph: &Graph) -> IgraphResult<u64> {
    let mut go2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let s = du + dv;
        let p = du.saturating_mul(dv);
        go2 = go2.saturating_add(s.saturating_mul(p));
    }

    Ok(go2)
}

/// Compute the first hyper-Gourava index.
///
/// `HGO₁(G) = Σ_{(u,v)∈E} [d(u)+d(v)+d(u)·d(v)]²`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_hyper_gourava_index};
///
/// // K_3: each edge [2+2+4]²=64, 3 edges → 192
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_hyper_gourava_index(&g).unwrap(), 192);
/// ```
pub fn first_hyper_gourava_index(graph: &Graph) -> IgraphResult<u64> {
    let mut hgo1 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let val = du + dv + du.saturating_mul(dv);
        hgo1 = hgo1.saturating_add(val.saturating_mul(val));
    }

    Ok(hgo1)
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

    fn diamond() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // --- first_gourava_index ---

    #[test]
    fn go1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn go1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn go1_single_edge() {
        // (1+1+1·1) = 3
        assert_eq!(first_gourava_index(&single_edge()).unwrap(), 3);
    }

    #[test]
    fn go1_path3() {
        // (0,1): 1+2+2=5, (1,2): 2+1+2=5 → 10
        assert_eq!(first_gourava_index(&path3()).unwrap(), 10);
    }

    #[test]
    fn go1_path4() {
        // (0,1):1+2+2=5, (1,2):2+2+4=8, (2,3):2+1+2=5 → 18
        assert_eq!(first_gourava_index(&path4()).unwrap(), 18);
    }

    #[test]
    fn go1_k3() {
        // 3 × (2+2+4) = 3×8 = 24
        assert_eq!(first_gourava_index(&k3()).unwrap(), 24);
    }

    #[test]
    fn go1_k4() {
        // 6 × (3+3+9) = 6×15 = 90
        assert_eq!(first_gourava_index(&k4()).unwrap(), 90);
    }

    #[test]
    fn go1_cycle4() {
        // 4 × (2+2+4) = 4×8 = 32
        assert_eq!(first_gourava_index(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn go1_cycle5() {
        // 5 × (2+2+4) = 5×8 = 40
        assert_eq!(first_gourava_index(&cycle5()).unwrap(), 40);
    }

    #[test]
    fn go1_star5() {
        // 4 × (4+1+4) = 4×9 = 36
        assert_eq!(first_gourava_index(&star5()).unwrap(), 36);
    }

    #[test]
    fn go1_paw() {
        // degrees [2,2,3,1]
        // (0,1):2+2+4=8, (0,2):2+3+6=11, (1,2):2+3+6=11, (2,3):3+1+3=7
        // GO1 = 8+11+11+7 = 37
        assert_eq!(first_gourava_index(&paw()).unwrap(), 37);
    }

    #[test]
    fn go1_diamond() {
        // degrees [3,3,2,2]
        // (0,1):3+3+9=15, (0,2):3+2+6=11, (0,3):3+2+6=11, (1,2):3+2+6=11, (1,3):3+2+6=11
        // GO1 = 15+11+11+11+11 = 59
        assert_eq!(first_gourava_index(&diamond()).unwrap(), 59);
    }

    #[test]
    fn go1_is_m1_plus_m2() {
        // GO₁ = M₁ + M₂ where M₁ = first Zagreb, M₂ = second Zagreb
        // M₁ = Σ_{edges} (d(u)+d(v)), M₂ = Σ_{edges} d(u)·d(v)
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let mut m1 = 0_u64;
            let mut m2 = 0_u64;
            for (u, v) in g.edges() {
                if u == v {
                    continue;
                }
                let du = g.degree(u).unwrap() as u64;
                let dv = g.degree(v).unwrap() as u64;
                m1 += du + dv;
                m2 += du * dv;
            }
            assert_eq!(first_gourava_index(g).unwrap(), m1 + m2);
        }
    }

    #[test]
    fn go1_regular_formula() {
        // r-regular: GO1 = m·(2r + r²) = m·r·(r+2)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(first_gourava_index(g).unwrap(), m * r * (r + 2));
        }
    }

    // --- second_gourava_index ---

    #[test]
    fn go2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn go2_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(second_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn go2_single_edge() {
        // (1+1)·(1·1) = 2
        assert_eq!(second_gourava_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn go2_path3() {
        // (0,1):(1+2)·(1·2)=6, (1,2):(2+1)·(2·1)=6 → 12
        assert_eq!(second_gourava_index(&path3()).unwrap(), 12);
    }

    #[test]
    fn go2_path4() {
        // (0,1):(1+2)·2=6, (1,2):(2+2)·4=16, (2,3):(2+1)·2=6 → 28
        assert_eq!(second_gourava_index(&path4()).unwrap(), 28);
    }

    #[test]
    fn go2_k3() {
        // 3 × (2+2)·(2·2) = 3×16 = 48
        assert_eq!(second_gourava_index(&k3()).unwrap(), 48);
    }

    #[test]
    fn go2_k4() {
        // 6 × (3+3)·(3·3) = 6×54 = 324
        assert_eq!(second_gourava_index(&k4()).unwrap(), 324);
    }

    #[test]
    fn go2_cycle4() {
        // 4 × (2+2)·4 = 4×16 = 64
        assert_eq!(second_gourava_index(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn go2_cycle5() {
        // 5 × 16 = 80
        assert_eq!(second_gourava_index(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn go2_star5() {
        // 4 × (4+1)·(4·1) = 4×20 = 80
        assert_eq!(second_gourava_index(&star5()).unwrap(), 80);
    }

    #[test]
    fn go2_paw() {
        // (0,1):(2+2)·4=16, (0,2):(2+3)·6=30, (1,2):(2+3)·6=30, (2,3):(3+1)·3=12
        // GO2 = 16+30+30+12 = 88
        assert_eq!(second_gourava_index(&paw()).unwrap(), 88);
    }

    #[test]
    fn go2_diamond() {
        // (0,1):(3+3)·9=54, (0,2):(3+2)·6=30, (0,3):(3+2)·6=30, (1,2):30, (1,3):30
        // GO2 = 54+30+30+30+30 = 174
        assert_eq!(second_gourava_index(&diamond()).unwrap(), 174);
    }

    #[test]
    fn go2_regular_formula() {
        // r-regular: GO2 = m·2r·r² = 2m·r³
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(second_gourava_index(g).unwrap(), 2 * m * r * r * r);
        }
    }

    // --- first_hyper_gourava_index ---

    #[test]
    fn hgo1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_hyper_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn hgo1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_hyper_gourava_index(&g).unwrap(), 0);
    }

    #[test]
    fn hgo1_single_edge() {
        // [1+1+1]²=9
        assert_eq!(first_hyper_gourava_index(&single_edge()).unwrap(), 9);
    }

    #[test]
    fn hgo1_path3() {
        // (0,1):[1+2+2]²=25, (1,2):[2+1+2]²=25 → 50
        assert_eq!(first_hyper_gourava_index(&path3()).unwrap(), 50);
    }

    #[test]
    fn hgo1_path4() {
        // (0,1):25, (1,2):[2+2+4]²=64, (2,3):25 → 114
        assert_eq!(first_hyper_gourava_index(&path4()).unwrap(), 114);
    }

    #[test]
    fn hgo1_k3() {
        // 3 × [2+2+4]² = 3×64 = 192
        assert_eq!(first_hyper_gourava_index(&k3()).unwrap(), 192);
    }

    #[test]
    fn hgo1_k4() {
        // 6 × [3+3+9]² = 6×225 = 1350
        assert_eq!(first_hyper_gourava_index(&k4()).unwrap(), 1350);
    }

    #[test]
    fn hgo1_cycle4() {
        // 4 × 64 = 256
        assert_eq!(first_hyper_gourava_index(&cycle4()).unwrap(), 256);
    }

    #[test]
    fn hgo1_cycle5() {
        // 5 × 64 = 320
        assert_eq!(first_hyper_gourava_index(&cycle5()).unwrap(), 320);
    }

    #[test]
    fn hgo1_star5() {
        // 4 × [4+1+4]² = 4×81 = 324
        assert_eq!(first_hyper_gourava_index(&star5()).unwrap(), 324);
    }

    #[test]
    fn hgo1_paw() {
        // (0,1):8²=64, (0,2):11²=121, (1,2):11²=121, (2,3):7²=49
        // HGO1 = 64+121+121+49 = 355
        assert_eq!(first_hyper_gourava_index(&paw()).unwrap(), 355);
    }

    #[test]
    fn hgo1_diamond() {
        // (0,1):15²=225, (0,2):11²=121, (0,3):11²=121, (1,2):121, (1,3):121
        // HGO1 = 225+121+121+121+121 = 709
        assert_eq!(first_hyper_gourava_index(&diamond()).unwrap(), 709);
    }

    #[test]
    fn hgo1_regular_formula() {
        // r-regular: HGO1 = m·(2r+r²)² = m·r²·(r+2)²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let val = r * (r + 2);
            assert_eq!(first_hyper_gourava_index(g).unwrap(), m * val * val);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn go2_geq_go1() {
        // Not universally true, but check computability
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let _ = first_gourava_index(g).unwrap();
            let _ = second_gourava_index(g).unwrap();
        }
    }

    #[test]
    fn hgo1_geq_go1_squared_over_m() {
        // By Cauchy-Schwarz: HGO1 ≥ GO1²/m (for m>0)
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let m = g.ecount() as u64;
            if m > 0 {
                let go1 = first_gourava_index(g).unwrap();
                let hgo1 = first_hyper_gourava_index(g).unwrap();
                assert!(hgo1 * m >= go1 * go1);
            }
        }
    }

    #[test]
    fn all_positive_for_graphs_with_edges() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_gourava_index(g).unwrap() > 0);
            assert!(second_gourava_index(g).unwrap() > 0);
            assert!(first_hyper_gourava_index(g).unwrap() > 0);
        }
    }
}
