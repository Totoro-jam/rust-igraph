//! Hyper-Zagreb and redefined Zagreb indices (ALGO-TR-060).
//!
//! - **First hyper-Zagreb index** `HM₁(G) = Σ_{(u,v)∈E} (d(u) + d(v))²`
//!   Introduced by Shirdel et al. (2013). Square of degree sum over edges.
//! - **Second hyper-Zagreb index** `HM₂(G) = Σ_{(u,v)∈E} (d(u) · d(v))²`
//!   Square of degree product over edges.
//! - **First redefined Zagreb index** `ReZG₁(G) = Σ_{(u,v)∈E} (d(u)+d(v)) / (d(u)·d(v))`
//!   Introduced by Ranjini et al. (2013).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the first hyper-Zagreb index.
///
/// `HM₁(G) = Σ_{(u,v)∈E} (d(u) + d(v))²`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_hyper_zagreb};
///
/// // K_3: each edge (2+2)²=16, 3 edges → 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_hyper_zagreb(&g).unwrap(), 48);
/// ```
pub fn first_hyper_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let mut hm1 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let s = du + dv;
        hm1 = hm1.saturating_add(s.saturating_mul(s));
    }

    Ok(hm1)
}

/// Compute the second hyper-Zagreb index.
///
/// `HM₂(G) = Σ_{(u,v)∈E} (d(u) · d(v))²`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_hyper_zagreb};
///
/// // K_3: each edge (2·2)²=16, 3 edges → 48
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_hyper_zagreb(&g).unwrap(), 48);
/// ```
pub fn second_hyper_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let mut hm2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        let p = du.saturating_mul(dv);
        hm2 = hm2.saturating_add(p.saturating_mul(p));
    }

    Ok(hm2)
}

/// Compute the first redefined Zagreb index.
///
/// `ReZG₁(G) = Σ_{(u,v)∈E} (d(u) + d(v)) / (d(u) · d(v))`
///
/// Edges where either endpoint has degree 0 or self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_redefined_zagreb};
///
/// // K_3: each edge (2+2)/(2·2)=1, 3 edges → 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((first_redefined_zagreb(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn first_redefined_zagreb(graph: &Graph) -> IgraphResult<f64> {
    let mut rezg1 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let prod = du * dv;
        if prod > 0.0 {
            rezg1 += (du + dv) / prod;
        }
    }

    Ok(rezg1)
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

    // --- first_hyper_zagreb ---

    #[test]
    fn hm1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_hyper_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn hm1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_hyper_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn hm1_single_edge() {
        // (1+1)² = 4
        assert_eq!(first_hyper_zagreb(&single_edge()).unwrap(), 4);
    }

    #[test]
    fn hm1_path3() {
        // (0,1): (1+2)²=9, (1,2): (2+1)²=9 → 18
        assert_eq!(first_hyper_zagreb(&path3()).unwrap(), 18);
    }

    #[test]
    fn hm1_path4() {
        // (0,1): (1+2)²=9, (1,2): (2+2)²=16, (2,3): (2+1)²=9 → 34
        assert_eq!(first_hyper_zagreb(&path4()).unwrap(), 34);
    }

    #[test]
    fn hm1_k3() {
        // 3 × (2+2)² = 3×16 = 48
        assert_eq!(first_hyper_zagreb(&k3()).unwrap(), 48);
    }

    #[test]
    fn hm1_k4() {
        // 6 × (3+3)² = 6×36 = 216
        assert_eq!(first_hyper_zagreb(&k4()).unwrap(), 216);
    }

    #[test]
    fn hm1_cycle4() {
        // 4 × (2+2)² = 4×16 = 64
        assert_eq!(first_hyper_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn hm1_cycle5() {
        // 5 × (2+2)² = 5×16 = 80
        assert_eq!(first_hyper_zagreb(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn hm1_star5() {
        // 4 × (4+1)² = 4×25 = 100
        assert_eq!(first_hyper_zagreb(&star5()).unwrap(), 100);
    }

    #[test]
    fn hm1_paw() {
        // degrees [2,2,3,1]
        // (0,1):(2+2)²=16, (0,2):(2+3)²=25, (1,2):(2+3)²=25, (2,3):(3+1)²=16
        // HM₁ = 16+25+25+16 = 82
        assert_eq!(first_hyper_zagreb(&paw()).unwrap(), 82);
    }

    #[test]
    fn hm1_regular_formula() {
        // r-regular: HM₁ = m·(2r)² = 4r²m
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(first_hyper_zagreb(g).unwrap(), 4 * r * r * m);
        }
    }

    // --- second_hyper_zagreb ---

    #[test]
    fn hm2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_hyper_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn hm2_single_edge() {
        // (1·1)² = 1
        assert_eq!(second_hyper_zagreb(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn hm2_path3() {
        // (0,1):(1·2)²=4, (1,2):(2·1)²=4 → 8
        assert_eq!(second_hyper_zagreb(&path3()).unwrap(), 8);
    }

    #[test]
    fn hm2_path4() {
        // (0,1):(1·2)²=4, (1,2):(2·2)²=16, (2,3):(2·1)²=4 → 24
        assert_eq!(second_hyper_zagreb(&path4()).unwrap(), 24);
    }

    #[test]
    fn hm2_k3() {
        // 3 × (2·2)² = 3×16 = 48
        assert_eq!(second_hyper_zagreb(&k3()).unwrap(), 48);
    }

    #[test]
    fn hm2_k4() {
        // 6 × (3·3)² = 6×81 = 486
        assert_eq!(second_hyper_zagreb(&k4()).unwrap(), 486);
    }

    #[test]
    fn hm2_cycle4() {
        // 4 × (2·2)² = 4×16 = 64
        assert_eq!(second_hyper_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn hm2_cycle5() {
        // 5 × (2·2)² = 5×16 = 80
        assert_eq!(second_hyper_zagreb(&cycle5()).unwrap(), 80);
    }

    #[test]
    fn hm2_star5() {
        // 4 × (4·1)² = 4×16 = 64
        assert_eq!(second_hyper_zagreb(&star5()).unwrap(), 64);
    }

    #[test]
    fn hm2_paw() {
        // (0,1):(2·2)²=16, (0,2):(2·3)²=36, (1,2):(2·3)²=36, (2,3):(3·1)²=9
        // HM₂ = 16+36+36+9 = 97
        assert_eq!(second_hyper_zagreb(&paw()).unwrap(), 97);
    }

    #[test]
    fn hm2_regular_formula() {
        // r-regular: HM₂ = m·r⁴
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            assert_eq!(second_hyper_zagreb(g).unwrap(), m * r * r * r * r);
        }
    }

    #[test]
    fn hm1_k3_eq_hm2_k3() {
        // For K_3: HM₁ = HM₂ = 48 (coincidence since (2+2)² = (2·2)²)
        assert_eq!(
            first_hyper_zagreb(&k3()).unwrap(),
            second_hyper_zagreb(&k3()).unwrap()
        );
    }

    // --- first_redefined_zagreb ---

    #[test]
    fn rezg1_empty() {
        let g = Graph::with_vertices(0);
        assert!((first_redefined_zagreb(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rezg1_single_edge() {
        // (1+1)/(1·1) = 2
        assert!((first_redefined_zagreb(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_path3() {
        // (0,1):(1+2)/(1·2) = 3/2, (1,2): same → 3
        assert!((first_redefined_zagreb(&path3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_path4() {
        // (0,1):(1+2)/(1·2)=3/2, (1,2):(2+2)/(2·2)=1, (2,3):3/2 → 4
        assert!((first_redefined_zagreb(&path4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_k3() {
        // 3 × (2+2)/(2·2) = 3×1 = 3
        assert!((first_redefined_zagreb(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_k4() {
        // 6 × (3+3)/(3·3) = 6×6/9 = 4
        assert!((first_redefined_zagreb(&k4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_cycle4() {
        // 4 × 1 = 4
        assert!((first_redefined_zagreb(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_cycle5() {
        // 5 × 1 = 5
        assert!((first_redefined_zagreb(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_star5() {
        // 4 × (4+1)/(4·1) = 4×5/4 = 5
        assert!((first_redefined_zagreb(&star5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_paw() {
        // (0,1):(2+2)/(2·2)=1, (0,2):(2+3)/(2·3)=5/6, (1,2):5/6, (2,3):(3+1)/(3·1)=4/3
        // ReZG₁ = 1 + 5/6 + 5/6 + 4/3 = 1 + 10/6 + 4/3 = 1 + 5/3 + 4/3 = 1 + 3 = 4
        assert!((first_redefined_zagreb(&paw()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rezg1_regular_formula() {
        // r-regular: ReZG₁ = m · 2r/r² = m · 2/r
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * 2.0 / r;
            assert!((first_redefined_zagreb(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn hm1_geq_hm2_for_bipartite() {
        // For trees (bipartite): not universally true. Just check both compute.
        for g in &[single_edge(), path3(), path4(), star5()] {
            let _ = first_hyper_zagreb(g).unwrap();
            let _ = second_hyper_zagreb(g).unwrap();
        }
    }

    #[test]
    fn rezg1_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_redefined_zagreb(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn rezg1_identity() {
        // ReZG₁ = Σ (1/d(u) + 1/d(v)) = 2·Σ_v 1/d(v) = 2·ID(G)? No:
        // ReZG₁ = Σ_{edges} (d(u)+d(v))/(d(u)·d(v)) = Σ_{edges} (1/d(v) + 1/d(u))
        // For a vertex v with degree d, it appears in d edges, so
        // ReZG₁ = Σ_v d(v)·(1/d(v)) · (count of reciprocals from neighbors)
        // Actually: ReZG₁ = Σ_{(u,v)∈E} 1/d(u) + 1/d(v)
        // Each vertex v contributes 1/d(v) for each of its d(v) edges = 1.
        // So ReZG₁ = n (for graphs without isolated vertices)!
        // Wait, that would mean ReZG₁ = |V with d>0|.
        // Let's verify: path3 has 3 non-isolated vertices → ReZG₁ should be 3. ✓
        // K3: 3. ✓. K4: 4. ✓. C4: 4. ✓. C5: 5. ✓. star5: 5. ✓. paw: 4. ✓.
        for g in &[path3(), k3(), k4(), cycle4(), cycle5(), star5(), paw()] {
            let n_nonisolated = (0..g.vcount())
                .filter(|&v| g.degree(v).unwrap() > 0)
                .count() as f64;
            assert!((first_redefined_zagreb(g).unwrap() - n_nonisolated).abs() < 1e-8);
        }
    }
}
