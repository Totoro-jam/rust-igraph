//! Reduced (decremented) degree-based indices (ALGO-TR-075).
//!
//! Indices using `(d(v)-1)` ("reduced degree") instead of `d(v)`:
//!
//! - **Reduced reciprocal Randić** `RRR(G) = Σ_{(u,v)∈E} 1/√((du-1)(dv-1))`
//! - **Reduced sum-connectivity** `χ_red(G) = Σ_{(u,v)∈E} 1/√((du-1)+(dv-1))`
//! - **Reduced first Zagreb** `M₁_red(G) = Σ_v (d(v)-1)²`
//! - **Reduced second Zagreb** already exists as `reduced_second_zagreb`
//!   in `forgotten_zagreb.rs`; this module adds the remaining family.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the reduced reciprocal Randić index.
///
/// `RRR(G) = Σ_{(u,v)∈E} 1/√((du-1)·(dv-1))`
///
/// Edges where either endpoint has degree ≤ 1 are skipped (the
/// reduced degree would be 0, making the denominator zero).
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_reciprocal_randic};
///
/// // K_3: 3 edges, d=(2,2), each 1/√(1·1)=1 → 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((reduced_reciprocal_randic(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn reduced_reciprocal_randic(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        if du >= 2 && dv >= 2 {
            let product = ((du - 1) * (dv - 1)) as f64;
            result += 1.0 / product.sqrt();
        }
    }

    Ok(result)
}

/// Compute the reduced sum-connectivity index.
///
/// `χ_red(G) = Σ_{(u,v)∈E} 1/√((du-1)+(dv-1))`
///
/// Edges where `du + dv - 2 == 0` (both endpoints degree 1) are
/// skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_sum_connectivity};
///
/// // K_3: 3 edges, d=(2,2), each 1/√(1+1)=1/√2 → 3/√2
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let expected = 3.0 / 2.0_f64.sqrt();
/// assert!((reduced_sum_connectivity(&g).unwrap() - expected).abs() < 1e-10);
/// ```
pub fn reduced_sum_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        let s = du + dv;
        if s > 2 {
            result += 1.0 / ((s - 2) as f64).sqrt();
        }
    }

    Ok(result)
}

/// Compute the reduced first Zagreb index.
///
/// `M₁_red(G) = Σ_v (d(v)-1)²`
///
/// For isolated vertices (degree 0), the term is `(-1)² = 1`, but
/// conventionally these are treated as having reduced degree 0, so
/// we skip vertices with degree 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_first_zagreb};
///
/// // K_3: d=(2,2,2), each (2-1)²=1 → 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(reduced_first_zagreb(&g).unwrap(), 3);
///
/// // Path 0-1-2: d=(1,2,1), (0)²+(1)²+(0)²=1
/// let p = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(reduced_first_zagreb(&p).unwrap(), 1);
/// ```
pub fn reduced_first_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut result = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d >= 1 {
            let rd = (d - 1) as u64;
            result = result.saturating_add(rd.saturating_mul(rd));
        }
    }

    Ok(result)
}

/// Compute the reduced forgotten index.
///
/// `F_red(G) = Σ_v (d(v)-1)³`
///
/// Vertices with degree 0 are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reduced_forgotten_index};
///
/// // K_4: d=(3,3,3,3), each (3-1)³=8 → 32
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert_eq!(reduced_forgotten_index(&g).unwrap(), 32);
/// ```
pub fn reduced_forgotten_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    let mut result = 0_u64;

    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d >= 1 {
            let rd = (d - 1) as u64;
            result = result.saturating_add(rd.saturating_mul(rd).saturating_mul(rd));
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

    // --- reduced_reciprocal_randic ---

    #[test]
    fn rrr_empty() {
        let g = Graph::with_vertices(0);
        assert!(reduced_reciprocal_randic(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rrr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(reduced_reciprocal_randic(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rrr_single_edge() {
        // d=(1,1), both degree 1 → skipped → 0
        assert!(reduced_reciprocal_randic(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rrr_path3() {
        // 2 edges d=(1,2): degree 1 endpoint → skipped → 0
        assert!(reduced_reciprocal_randic(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rrr_k3() {
        // d=(2,2), 1/√(1·1)=1 per edge → 3
        assert!((reduced_reciprocal_randic(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rrr_k4() {
        // d=(3,3), 1/√(2·2)=1/2 per edge, 6 edges → 3
        assert!((reduced_reciprocal_randic(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rrr_cycle4() {
        // d=(2,2), 1/1=1 per edge, 4 edges → 4
        assert!((reduced_reciprocal_randic(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rrr_cycle5() {
        // d=(2,2), 1/1 per edge, 5 edges → 5
        assert!((reduced_reciprocal_randic(&cycle5()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rrr_star5() {
        // d=(4,1): leaf deg=1 → all skipped → 0
        assert!(reduced_reciprocal_randic(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rrr_paw() {
        // (0,1) d=(2,2): 1/1=1
        // (0,2) d=(2,3): 1/√(1·2)=1/√2
        // (1,2) d=(2,3): 1/√2
        // (2,3) d=(3,1): leaf → skipped
        let expected = 1.0 + 2.0 / 2.0_f64.sqrt();
        assert!((reduced_reciprocal_randic(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- reduced_sum_connectivity ---

    #[test]
    fn rsc_empty() {
        let g = Graph::with_vertices(0);
        assert!(reduced_sum_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rsc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(reduced_sum_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rsc_single_edge() {
        // d=(1,1), du+dv-2=0 → skipped → 0
        assert!(reduced_sum_connectivity(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rsc_path3() {
        // d=(1,2): 1+2-2=1, 1/√1=1 per edge → 2
        assert!((reduced_sum_connectivity(&path3()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rsc_k3() {
        // d=(2,2): 2+2-2=2, 1/√2 per edge → 3/√2
        let expected = 3.0 / 2.0_f64.sqrt();
        assert!((reduced_sum_connectivity(&k3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rsc_k4() {
        // d=(3,3): 3+3-2=4, 1/√4=1/2 per edge, 6 edges → 3
        assert!((reduced_sum_connectivity(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rsc_cycle4() {
        // d=(2,2): 2, 1/√2 per edge → 4/√2 = 2√2
        let expected = 4.0 / 2.0_f64.sqrt();
        assert!((reduced_sum_connectivity(&cycle4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rsc_star5() {
        // d=(4,1): 4+1-2=3, 1/√3 per edge → 4/√3
        let expected = 4.0 / 3.0_f64.sqrt();
        assert!((reduced_sum_connectivity(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rsc_paw() {
        // (0,1) d=(2,2): 1/√2
        // (0,2) d=(2,3): 1/√3
        // (1,2) d=(2,3): 1/√3
        // (2,3) d=(3,1): 1/√2
        let expected = 2.0 / 2.0_f64.sqrt() + 2.0 / 3.0_f64.sqrt();
        assert!((reduced_sum_connectivity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- reduced_first_zagreb ---

    #[test]
    fn rfz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(reduced_first_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn rfz_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(reduced_first_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn rfz_single_edge() {
        // d=(1,1), (1-1)²=0 each → 0
        assert_eq!(reduced_first_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn rfz_path3() {
        // d=(1,2,1): 0+1+0=1
        assert_eq!(reduced_first_zagreb(&path3()).unwrap(), 1);
    }

    #[test]
    fn rfz_path4() {
        // d=(1,2,2,1): 0+1+1+0=2
        assert_eq!(reduced_first_zagreb(&path4()).unwrap(), 2);
    }

    #[test]
    fn rfz_k3() {
        // d=(2,2,2): 3×1=3
        assert_eq!(reduced_first_zagreb(&k3()).unwrap(), 3);
    }

    #[test]
    fn rfz_k4() {
        // d=(3,3,3,3): 4×4=16
        assert_eq!(reduced_first_zagreb(&k4()).unwrap(), 16);
    }

    #[test]
    fn rfz_cycle4() {
        // d=(2,2,2,2): 4×1=4
        assert_eq!(reduced_first_zagreb(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn rfz_cycle5() {
        // d=(2,2,2,2,2): 5×1=5
        assert_eq!(reduced_first_zagreb(&cycle5()).unwrap(), 5);
    }

    #[test]
    fn rfz_star5() {
        // d=(4,1,1,1,1): (3)²+0+0+0+0=9
        assert_eq!(reduced_first_zagreb(&star5()).unwrap(), 9);
    }

    #[test]
    fn rfz_paw() {
        // d=(2,2,3,1): 1+1+4+0=6
        assert_eq!(reduced_first_zagreb(&paw()).unwrap(), 6);
    }

    // --- reduced_forgotten_index ---

    #[test]
    fn rfi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(reduced_forgotten_index(&g).unwrap(), 0);
    }

    #[test]
    fn rfi_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(reduced_forgotten_index(&g).unwrap(), 0);
    }

    #[test]
    fn rfi_single_edge() {
        // d=(1,1), (0)³=0 each → 0
        assert_eq!(reduced_forgotten_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn rfi_path3() {
        // d=(1,2,1): 0+1+0=1
        assert_eq!(reduced_forgotten_index(&path3()).unwrap(), 1);
    }

    #[test]
    fn rfi_k3() {
        // d=(2,2,2): 3×1³=3
        assert_eq!(reduced_forgotten_index(&k3()).unwrap(), 3);
    }

    #[test]
    fn rfi_k4() {
        // d=(3,3,3,3): 4×2³=32
        assert_eq!(reduced_forgotten_index(&k4()).unwrap(), 32);
    }

    #[test]
    fn rfi_cycle4() {
        // d=(2,2,2,2): 4×1=4
        assert_eq!(reduced_forgotten_index(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn rfi_star5() {
        // d=(4,1,1,1,1): 3³+0+0+0+0=27
        assert_eq!(reduced_forgotten_index(&star5()).unwrap(), 27);
    }

    #[test]
    fn rfi_paw() {
        // d=(2,2,3,1): 1+1+8+0=10
        assert_eq!(reduced_forgotten_index(&paw()).unwrap(), 10);
    }

    // --- cross-consistency ---

    #[test]
    fn rfz_le_rfi_for_nontrivial() {
        // (d-1)² ≤ (d-1)³ when d-1 ≥ 1, i.e. d ≥ 2
        // Not strictly true for d=2 where equality holds, but sum-wise:
        // rfz ≤ rfi when max degree ≥ 3 (otherwise equal)
        for g in &[k4(), star5()] {
            assert!(reduced_first_zagreb(g).unwrap() <= reduced_forgotten_index(g).unwrap());
        }
    }

    #[test]
    fn regular_rrr_equals_edge_count() {
        // For r-regular (r≥2): each edge 1/√((r-1)²) = 1/(r-1) → m/(r-1)
        // K_3: r=2 → m/1 = 3
        assert!((reduced_reciprocal_randic(&k3()).unwrap() - 3.0).abs() < 1e-10);
        // C_4: r=2 → 4/1 = 4
        assert!((reduced_reciprocal_randic(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
        // K_4: r=3 → 6/2 = 3
        assert!((reduced_reciprocal_randic(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }
}
