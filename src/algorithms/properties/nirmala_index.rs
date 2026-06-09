//! Nirmala indices (ALGO-TR-064).
//!
//! - **Nirmala index** `N(G) = Σ_{(u,v)∈E} √(d(u) + d(v))`
//!   Introduced by Kulli (2021). Square root of degree sum over edges.
//! - **First inverse Nirmala index** `IN₁(G) = Σ_{(u,v)∈E} 1/√(d(u)+d(v))`
//!   Reciprocal square root of degree sum.
//! - **Second inverse Nirmala index** `IN₂(G) = Σ_{(u,v)∈E} 1/√(d(u)·d(v))`
//!   Reciprocal square root of degree product (equals the Randić index).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Nirmala index.
///
/// `N(G) = Σ_{(u,v)∈E} √(d(u) + d(v))`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, nirmala_index};
///
/// // K_3: 3 edges × √(2+2) = 3×2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((nirmala_index(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn nirmala_index(graph: &Graph) -> IgraphResult<f64> {
    let mut n_idx = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        n_idx += (du + dv).sqrt();
    }

    Ok(n_idx)
}

/// Compute the first inverse Nirmala index.
///
/// `IN₁(G) = Σ_{(u,v)∈E} 1/√(d(u) + d(v))`
///
/// Self-loops and edges where both endpoints have degree 0 are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_inverse_nirmala};
///
/// // K_3: 3 edges × 1/√4 = 3×0.5 = 1.5
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((first_inverse_nirmala(&g).unwrap() - 1.5).abs() < 1e-10);
/// ```
pub fn first_inverse_nirmala(graph: &Graph) -> IgraphResult<f64> {
    let mut in1 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s > 0.0 {
            in1 += 1.0 / s.sqrt();
        }
    }

    Ok(in1)
}

/// Compute the second inverse Nirmala index.
///
/// `IN₂(G) = Σ_{(u,v)∈E} 1/√(d(u) · d(v))`
///
/// This equals the Randić connectivity index `R(G)`.
/// Self-loops and edges where either endpoint has degree 0 are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_inverse_nirmala};
///
/// // K_3: 3 edges × 1/√(2·2) = 3×0.5 = 1.5
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((second_inverse_nirmala(&g).unwrap() - 1.5).abs() < 1e-10);
/// ```
pub fn second_inverse_nirmala(graph: &Graph) -> IgraphResult<f64> {
    let mut in2 = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let p = du * dv;
        if p > 0.0 {
            in2 += 1.0 / p.sqrt();
        }
    }

    Ok(in2)
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

    // --- nirmala_index ---

    #[test]
    fn nirmala_empty() {
        let g = Graph::with_vertices(0);
        assert!((nirmala_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn nirmala_isolated() {
        let g = Graph::with_vertices(5);
        assert!((nirmala_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn nirmala_single_edge() {
        // √(1+1) = √2
        let expected = 2.0_f64.sqrt();
        assert!((nirmala_index(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_path3() {
        // (0,1):√(1+2)=√3, (1,2):√3 → 2√3
        let expected = 2.0 * 3.0_f64.sqrt();
        assert!((nirmala_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_path4() {
        // (0,1):√3, (1,2):√4=2, (2,3):√3 → 2√3 + 2
        let expected = 2.0 * 3.0_f64.sqrt() + 2.0;
        assert!((nirmala_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_k3() {
        // 3 × √(2+2) = 3×2 = 6
        assert!((nirmala_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn nirmala_k4() {
        // 6 × √(3+3) = 6√6
        let expected = 6.0 * 6.0_f64.sqrt();
        assert!((nirmala_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_cycle4() {
        // 4 × √4 = 4×2 = 8
        assert!((nirmala_index(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn nirmala_cycle5() {
        // 5 × 2 = 10
        assert!((nirmala_index(&cycle5()).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn nirmala_star5() {
        // 4 × √(4+1) = 4√5
        let expected = 4.0 * 5.0_f64.sqrt();
        assert!((nirmala_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_paw() {
        // (0,1):√4=2, (0,2):√5, (1,2):√5, (2,3):√4=2
        let expected = 4.0 + 2.0 * 5.0_f64.sqrt();
        assert!((nirmala_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_diamond() {
        // (0,1):√6, (0,2):√5, (0,3):√5, (1,2):√5, (1,3):√5
        let expected = 6.0_f64.sqrt() + 4.0 * 5.0_f64.sqrt();
        assert!((nirmala_index(&diamond()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn nirmala_regular_formula() {
        // r-regular: N = m·√(2r)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m * (2.0 * r).sqrt();
            assert!((nirmala_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- first_inverse_nirmala ---

    #[test]
    fn in1_empty() {
        let g = Graph::with_vertices(0);
        assert!((first_inverse_nirmala(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn in1_single_edge() {
        // 1/√2
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!((first_inverse_nirmala(&single_edge()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in1_path3() {
        // 2/√3
        let expected = 2.0 / 3.0_f64.sqrt();
        assert!((first_inverse_nirmala(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in1_k3() {
        // 3 × 1/√4 = 3/2 = 1.5
        assert!((first_inverse_nirmala(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn in1_k4() {
        // 6/√6 = √6
        let expected = 6.0_f64.sqrt();
        assert!((first_inverse_nirmala(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in1_cycle4() {
        // 4 × 1/√4 = 4/2 = 2
        assert!((first_inverse_nirmala(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn in1_cycle5() {
        // 5/2 = 2.5
        assert!((first_inverse_nirmala(&cycle5()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn in1_star5() {
        // 4/√5
        let expected = 4.0 / 5.0_f64.sqrt();
        assert!((first_inverse_nirmala(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in1_paw() {
        // (0,1):1/2, (0,2):1/√5, (1,2):1/√5, (2,3):1/2
        let expected = 1.0 + 2.0 / 5.0_f64.sqrt();
        assert!((first_inverse_nirmala(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in1_regular_formula() {
        // r-regular: IN₁ = m/√(2r)
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m / (2.0 * r).sqrt();
            assert!((first_inverse_nirmala(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- second_inverse_nirmala ---

    #[test]
    fn in2_empty() {
        let g = Graph::with_vertices(0);
        assert!((second_inverse_nirmala(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn in2_single_edge() {
        // 1/√(1·1) = 1
        assert!((second_inverse_nirmala(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn in2_path3() {
        // (0,1):1/√2, (1,2):1/√2 → 2/√2 = √2
        let expected = 2.0_f64.sqrt();
        assert!((second_inverse_nirmala(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in2_k3() {
        // 3 × 1/√(2·2) = 3/2 = 1.5
        assert!((second_inverse_nirmala(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn in2_k4() {
        // 6 × 1/√9 = 6/3 = 2
        assert!((second_inverse_nirmala(&k4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn in2_cycle4() {
        // 4/2 = 2
        assert!((second_inverse_nirmala(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn in2_cycle5() {
        // 5/2 = 2.5
        assert!((second_inverse_nirmala(&cycle5()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn in2_star5() {
        // 4 × 1/√4 = 4/2 = 2
        assert!((second_inverse_nirmala(&star5()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn in2_paw() {
        // (0,1):1/2, (0,2):1/√6, (1,2):1/√6, (2,3):1/√3
        let expected = 0.5 + 2.0 / 6.0_f64.sqrt() + 1.0 / 3.0_f64.sqrt();
        assert!((second_inverse_nirmala(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn in2_equals_randic() {
        // IN₂ = Randić index = Σ 1/√(d(u)·d(v))
        // Verify against known Randić values for regular graphs
        // r-regular: R = m/r
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            assert!((second_inverse_nirmala(g).unwrap() - m / r).abs() < 1e-8);
        }
    }

    #[test]
    fn in2_regular_formula() {
        // r-regular: IN₂ = m/r
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m = g.ecount() as f64;
            let r = g.degree(0).unwrap() as f64;
            let expected = m / r;
            assert!((second_inverse_nirmala(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn nirmala_times_in1_geq_m() {
        // By Cauchy-Schwarz: N·IN₁ ≥ m²/m = m
        // Actually: (Σ √s)·(Σ 1/√s) ≥ m² by Cauchy-Schwarz (with f=√s, g=1/√s)
        // Wait: Σ(√s · 1/√s) = m, so by C-S: (Σ √s²)(Σ 1/s) ≥ m²? No.
        // Correct: Cauchy-Schwarz: (Σ √s)(Σ 1/√s) ≥ (Σ 1)² = m²
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let n_val = nirmala_index(g).unwrap();
            let in1_val = first_inverse_nirmala(g).unwrap();
            let m = g.ecount() as f64;
            assert!(n_val * in1_val >= m * m - 1e-8);
        }
    }

    #[test]
    fn all_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(nirmala_index(g).unwrap() > 0.0);
            assert!(first_inverse_nirmala(g).unwrap() > 0.0);
            assert!(second_inverse_nirmala(g).unwrap() > 0.0);
        }
    }
}
