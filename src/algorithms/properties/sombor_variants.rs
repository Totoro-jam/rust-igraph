//! Sombor index variants (ALGO-TR-070).
//!
//! Extensions of the Sombor index family introduced by Gutman (2021):
//!
//! - **Elliptic Sombor** `ESO(G) = Σ_{(u,v)∈E} (d(u)+d(v)) √(d(u)²+d(v)²)`
//! - **Modified Sombor** `mSO(G) = Σ_{(u,v)∈E} 1/√(d(u)²+d(v)²)`
//! - **Sombor coindex** `\overline{SO}(G) = Σ_{u≠v, (u,v)∉E} √(d(u)²+d(v)²)`
//!   Sum over non-adjacent vertex pairs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the elliptic Sombor index.
///
/// `ESO(G) = Σ_{(u,v)∈E} (d(u)+d(v)) √(d(u)²+d(v)²)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, elliptic_sombor_index};
///
/// // K_3: 3 edges, d=2 for all. Each term: (2+2)√(4+4) = 4√8
/// // ESO = 3 × 4√8 = 12√8 ≈ 33.941
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let eso = elliptic_sombor_index(&g).unwrap();
/// assert!((eso - 12.0 * 8.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn elliptic_sombor_index(graph: &Graph) -> IgraphResult<f64> {
    let mut eso = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        eso += (du + dv) * (du * du + dv * dv).sqrt();
    }

    Ok(eso)
}

/// Compute the modified Sombor index.
///
/// `mSO(G) = Σ_{(u,v)∈E} 1/√(d(u)²+d(v)²)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, modified_sombor_index};
///
/// // K_3: 3 edges, each 1/√(4+4) = 1/√8
/// // mSO = 3/√8 ≈ 1.0607
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let mso = modified_sombor_index(&g).unwrap();
/// assert!((mso - 3.0 / 8.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn modified_sombor_index(graph: &Graph) -> IgraphResult<f64> {
    let mut mso = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du * du + dv * dv;
        if s > 0.0 {
            mso += 1.0 / s.sqrt();
        }
    }

    Ok(mso)
}

/// Compute the Sombor coindex.
///
/// `\overline{SO}(G) = Σ_{u<v, (u,v)∉E} √(d(u)²+d(v)²)`
///
/// Sum over non-adjacent distinct vertex pairs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, sombor_coindex};
///
/// // K_3: no non-adjacent pairs → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((sombor_coindex(&g).unwrap()).abs() < 1e-10);
///
/// // Path 0-1-2: non-adjacent pair (0,2), d=(1,1), √(1+1)=√2
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((sombor_coindex(&p).unwrap() - 2.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn sombor_coindex(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount();
    let mut degrees = Vec::with_capacity(n as usize);
    for v in 0..n {
        degrees.push(graph.degree(v)? as f64);
    }

    let mut total_sum = 0.0_f64;
    for u in 0..n {
        for v in (u + 1)..n {
            let du = degrees[u as usize];
            let dv = degrees[v as usize];
            total_sum += (du * du + dv * dv).sqrt();
        }
    }

    let mut edge_sum = 0.0_f64;
    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        edge_sum += (du * du + dv * dv).sqrt();
    }

    Ok(total_sum - edge_sum)
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

    // --- elliptic_sombor_index ---

    #[test]
    fn eso_empty() {
        let g = Graph::with_vertices(0);
        assert!(elliptic_sombor_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eso_isolated() {
        let g = Graph::with_vertices(5);
        assert!(elliptic_sombor_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn eso_single_edge() {
        // (1+1)√(1+1) = 2√2
        let expected = 2.0 * 2.0_f64.sqrt();
        assert!((elliptic_sombor_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_path3() {
        // e(0,1): d=(1,2), (3)√(1+4) = 3√5
        // e(1,2): d=(2,1), same → 3√5
        let expected = 6.0 * 5.0_f64.sqrt();
        assert!((elliptic_sombor_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_k3() {
        // 3 edges, d=(2,2), (4)√(4+4) = 4√8
        let expected = 12.0 * 8.0_f64.sqrt();
        assert!((elliptic_sombor_index(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_k4() {
        // 6 edges, d=(3,3), (6)√(9+9) = 6√18
        let expected = 36.0 * 18.0_f64.sqrt();
        assert!((elliptic_sombor_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_cycle4() {
        // 4 edges, d=(2,2), (4)√8
        let expected = 16.0 * 8.0_f64.sqrt();
        assert!((elliptic_sombor_index(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_cycle5() {
        // 5 edges, d=(2,2), (4)√8
        let expected = 20.0 * 8.0_f64.sqrt();
        assert!((elliptic_sombor_index(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_star5() {
        // 4 edges, d=(4,1), (5)√(16+1) = 5√17
        let expected = 20.0 * 17.0_f64.sqrt();
        assert!((elliptic_sombor_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_paw() {
        // e(0,1): d=(2,2), (4)√8
        // e(0,2): d=(2,3), (5)√(4+9) = 5√13
        // e(1,2): d=(2,3), (5)√13
        // e(2,3): d=(3,1), (4)√(9+1) = 4√10
        let expected = 4.0 * 8.0_f64.sqrt() + 10.0 * 13.0_f64.sqrt() + 4.0 * 10.0_f64.sqrt();
        assert!((elliptic_sombor_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn eso_regular_formula() {
        // r-regular: ESO = m × 2r × √(2r²) = m × 2r × r√2 = 2mr²√2
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = 2.0 * m * r * r * 2.0_f64.sqrt();
            assert!((elliptic_sombor_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- modified_sombor_index ---

    #[test]
    fn mso_empty() {
        let g = Graph::with_vertices(0);
        assert!(modified_sombor_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mso_isolated() {
        let g = Graph::with_vertices(5);
        assert!(modified_sombor_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mso_single_edge() {
        // 1/√(1+1) = 1/√2
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!((modified_sombor_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_path3() {
        // 2 edges, each 1/√(1+4) = 1/√5
        let expected = 2.0 / 5.0_f64.sqrt();
        assert!((modified_sombor_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_k3() {
        // 3 × 1/√8
        let expected = 3.0 / 8.0_f64.sqrt();
        assert!((modified_sombor_index(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_k4() {
        // 6 × 1/√18
        let expected = 6.0 / 18.0_f64.sqrt();
        assert!((modified_sombor_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_cycle4() {
        // 4 × 1/√8
        let expected = 4.0 / 8.0_f64.sqrt();
        assert!((modified_sombor_index(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_star5() {
        // 4 × 1/√(16+1) = 4/√17
        let expected = 4.0 / 17.0_f64.sqrt();
        assert!((modified_sombor_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_paw() {
        // 1/√8 + 2/√13 + 1/√10
        let expected = 1.0 / 8.0_f64.sqrt() + 2.0 / 13.0_f64.sqrt() + 1.0 / 10.0_f64.sqrt();
        assert!((modified_sombor_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mso_regular_formula() {
        // r-regular: mSO = m/(r√2)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m / (r * 2.0_f64.sqrt());
            assert!((modified_sombor_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn mso_reciprocal_of_sombor() {
        // mSO × SO/m² is a useful ratio for regular graphs
        // For r-regular: SO = m·r√2, mSO = m/(r√2)
        // SO × mSO = m²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let so = crate::sombor_index(g).unwrap();
            let mso = modified_sombor_index(g).unwrap();
            assert!((so * mso - m * m).abs() < 1e-6);
        }
    }

    // --- sombor_coindex ---

    #[test]
    fn sco_empty() {
        let g = Graph::with_vertices(0);
        assert!(sombor_coindex(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sco_isolated() {
        // All pairs have d=0 → √0 = 0
        let g = Graph::with_vertices(5);
        assert!(sombor_coindex(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sco_single_edge() {
        // No non-adjacent pairs (only 2 vertices)
        assert!(sombor_coindex(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sco_k3() {
        // Complete graph: no non-adjacent pairs → 0
        assert!(sombor_coindex(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sco_k4() {
        assert!(sombor_coindex(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sco_path3() {
        // Non-adj: (0,2), d=(1,1), √(1+1)=√2
        let expected = 2.0_f64.sqrt();
        assert!((sombor_coindex(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sco_path4() {
        // Non-adj pairs: (0,2) d=(1,2), (0,3) d=(1,1), (1,3) d=(2,1)
        // √(1+4) + √(1+1) + √(4+1) = 2√5 + √2
        let expected = 2.0 * 5.0_f64.sqrt() + 2.0_f64.sqrt();
        assert!((sombor_coindex(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sco_cycle4() {
        // C_4: edges (0,1),(1,2),(2,3),(3,0). Non-adj: (0,2),(1,3)
        // d=(2,2), √(4+4) = √8. Two pairs → 2√8
        let expected = 2.0 * 8.0_f64.sqrt();
        assert!((sombor_coindex(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sco_star5() {
        // S_5: edges (0,1),(0,2),(0,3),(0,4). Non-adj: C(4,2)=6 leaf pairs
        // Each leaf pair d=(1,1), √(1+1)=√2. Total = 6√2
        let expected = 6.0 * 2.0_f64.sqrt();
        assert!((sombor_coindex(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sco_paw() {
        // Paw: edges (0,1),(0,2),(1,2),(2,3). 4 vertices, C(4,2)=6 pairs, 4 edges → 2 non-adj
        // Non-adj: (0,3) d=(2,1), (1,3) d=(2,1)
        // √(4+1) + √(4+1) = 2√5
        let expected = 2.0 * 5.0_f64.sqrt();
        assert!((sombor_coindex(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sco_complement_relation() {
        // SO(G) + \bar{SO}(G) = Σ_{u<v} √(d(u)²+d(v)²) (all pairs)
        for g in &[path3(), k3(), cycle4(), star5(), paw()] {
            let so = crate::sombor_index(g).unwrap();
            let sco = sombor_coindex(g).unwrap();
            let n = g.vcount();
            let mut total = 0.0_f64;
            for u in 0..n {
                for v in (u + 1)..n {
                    let du = g.degree(u).unwrap() as f64;
                    let dv = g.degree(v).unwrap() as f64;
                    total += (du * du + dv * dv).sqrt();
                }
            }
            assert!((so + sco - total).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_nontrivial() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(elliptic_sombor_index(g).unwrap() > 0.0);
            assert!(modified_sombor_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn eso_ge_sombor() {
        // ESO(G) ≥ SO(G) because (d(u)+d(v)) ≥ 1 for edges with d≥1
        // Actually ESO = (du+dv)·√(du²+dv²) and SO = √(du²+dv²)
        // So ESO = (du+dv)·SO_term ≥ 2·SO_term for du,dv≥1
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let eso = elliptic_sombor_index(g).unwrap();
            let so = crate::sombor_index(g).unwrap();
            assert!(eso >= so - 1e-10);
        }
    }
}
