//! Degree-ratio indices (ALGO-TR-081).
//!
//! Bond-additive indices based on degree ratios and harmonic-type
//! combinations:
//!
//! - **Symmetric ratio index** `SR(G) = Σ_{(u,v)∈E} (d(u)/d(v) + d(v)/d(u))`
//! - **Min-max degree ratio** `mm(G) = Σ_{(u,v)∈E} min(d(u),d(v))/max(d(u),d(v))`
//! - **Degree harmonic mean index** `DHM(G) = Σ_{(u,v)∈E} 2·d(u)·d(v)/(d(u)+d(v))`
//! - **Degree difference connectivity** `DDC(G) = Σ_{(u,v)∈E} 1/√(|d(u)-d(v)|+1)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the symmetric degree ratio index.
///
/// `SR(G) = Σ_{(u,v)∈E} (d(u)/d(v) + d(v)/d(u))`
///
/// Each term ≥ 2 by AM-GM, with equality when d(u)=d(v).
/// For regular graphs: `SR(G) = 2m`. Edges with a degree-0 endpoint
/// are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, symmetric_degree_ratio};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((symmetric_degree_ratio(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn symmetric_degree_ratio(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if du <= 0.0 || dv <= 0.0 {
            continue;
        }
        result += du / dv + dv / du;
    }

    Ok(result)
}

/// Compute the min-max degree ratio index.
///
/// `mm(G) = Σ_{(u,v)∈E} min(d(u),d(v))/max(d(u),d(v))`
///
/// Each term is in (0,1], equalling 1 when d(u)=d(v).
/// For regular graphs: `mm(G) = m`. Edges with a degree-0 endpoint
/// are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, minmax_degree_ratio};
///
/// // K_3: 3 edges → 3·1 = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((minmax_degree_ratio(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn minmax_degree_ratio(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        if du <= 0.0 || dv <= 0.0 {
            continue;
        }
        result += du.min(dv) / du.max(dv);
    }

    Ok(result)
}

/// Compute the degree harmonic mean index.
///
/// `DHM(G) = Σ_{(u,v)∈E} 2·d(u)·d(v)/(d(u)+d(v))`
///
/// This is the sum of the harmonic means of endpoint degrees.
/// For regular graphs with degree r: `DHM(G) = m·r`.
/// Edges with a degree-0 endpoint are skipped. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_harmonic_mean_index};
///
/// // K_3: 3 edges, all (2,2) → 3·2 = 6
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_harmonic_mean_index(&g).unwrap() - 6.0).abs() < 1e-10);
/// ```
pub fn degree_harmonic_mean_index(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let s = du + dv;
        if s <= 0.0 {
            continue;
        }
        result += 2.0 * du * dv / s;
    }

    Ok(result)
}

/// Compute the degree difference connectivity index.
///
/// `DDC(G) = Σ_{(u,v)∈E} 1/√(|d(u)-d(v)|+1)`
///
/// Each term is in (0,1], equalling 1 when d(u)=d(v).
/// For regular graphs: `DDC(G) = m`. Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_diff_connectivity};
///
/// // K_3: 3 edges, all (2,2) → 3·1 = 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((degree_diff_connectivity(&g).unwrap() - 3.0).abs() < 1e-10);
/// ```
pub fn degree_diff_connectivity(graph: &Graph) -> IgraphResult<f64> {
    let mut result = 0.0_f64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = graph.degree(u)? as f64;
        let dv = graph.degree(v)? as f64;
        let diff = (du - dv).abs() + 1.0;
        result += 1.0 / diff.sqrt();
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

    // --- symmetric_degree_ratio ---

    #[test]
    fn sdr_empty() {
        let g = Graph::with_vertices(0);
        assert!(symmetric_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sdr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(symmetric_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn sdr_regular_is_2m() {
        // Regular: each term = 2 → SR = 2m
        assert!((symmetric_degree_ratio(&k3()).unwrap() - 6.0).abs() < 1e-10);
        assert!((symmetric_degree_ratio(&k4()).unwrap() - 12.0).abs() < 1e-10);
        assert!((symmetric_degree_ratio(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn sdr_single_edge() {
        // (1,1): 1+1=2
        assert!((symmetric_degree_ratio(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn sdr_star5() {
        // 4 edges (4,1): 4·(4/1+1/4) = 4·4.25 = 17
        assert!((symmetric_degree_ratio(&star5()).unwrap() - 17.0).abs() < 1e-10);
    }

    #[test]
    fn sdr_path3() {
        // 2 edges (1,2): 2·(1/2+2/1) = 2·2.5 = 5
        assert!((symmetric_degree_ratio(&path3()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn sdr_paw() {
        // (0,1)d=(2,2): 2
        // (0,2)d=(2,3): 2/3+3/2 = 13/6
        // (1,2)d=(2,3): 13/6
        // (2,3)d=(3,1): 3+1/3 = 10/3
        let expected = 2.0 + 2.0 * 13.0 / 6.0 + 10.0 / 3.0;
        assert!((symmetric_degree_ratio(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn sdr_ge_2m() {
        // Each term ≥ 2 by AM-GM
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(symmetric_degree_ratio(g).unwrap() >= 2.0 * g.ecount() as f64 - 1e-10);
        }
    }

    // --- minmax_degree_ratio ---

    #[test]
    fn mmr_empty() {
        let g = Graph::with_vertices(0);
        assert!(minmax_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mmr_isolated() {
        let g = Graph::with_vertices(5);
        assert!(minmax_degree_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mmr_regular_is_m() {
        // Regular: each term = 1 → mm = m
        assert!((minmax_degree_ratio(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((minmax_degree_ratio(&k4()).unwrap() - 6.0).abs() < 1e-10);
        assert!((minmax_degree_ratio(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn mmr_single_edge() {
        assert!((minmax_degree_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mmr_star5() {
        // 4 edges (4,1): 4·(1/4) = 1
        assert!((minmax_degree_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mmr_path3() {
        // 2 edges (1,2): 2·(1/2) = 1
        assert!((minmax_degree_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mmr_paw() {
        // (0,1)d=(2,2): 1
        // (0,2)d=(2,3): 2/3
        // (1,2)d=(2,3): 2/3
        // (2,3)d=(3,1): 1/3
        let expected = 1.0 + 2.0 * 2.0 / 3.0 + 1.0 / 3.0;
        assert!((minmax_degree_ratio(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn mmr_le_m() {
        // Each term ≤ 1 → mm ≤ m
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(minmax_degree_ratio(g).unwrap() <= g.ecount() as f64 + 1e-10);
        }
    }

    // --- degree_harmonic_mean_index ---

    #[test]
    fn dhm_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_harmonic_mean_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dhm_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_harmonic_mean_index(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dhm_regular_is_mr() {
        // Regular degree r: each term = 2r²/(2r)=r → DHM = m·r
        // K_3: 3·2=6
        assert!((degree_harmonic_mean_index(&k3()).unwrap() - 6.0).abs() < 1e-10);
        // K_4: 6·3=18
        assert!((degree_harmonic_mean_index(&k4()).unwrap() - 18.0).abs() < 1e-10);
        // C_4: 4·2=8
        assert!((degree_harmonic_mean_index(&cycle4()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn dhm_single_edge() {
        // (1,1): 2·1·1/2 = 1
        assert!((degree_harmonic_mean_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dhm_star5() {
        // 4 edges (4,1): 4·(2·4·1/5) = 4·1.6 = 6.4
        assert!((degree_harmonic_mean_index(&star5()).unwrap() - 6.4).abs() < 1e-10);
    }

    #[test]
    fn dhm_paw() {
        // (0,1)d=(2,2): 2·4/4=2
        // (0,2)d=(2,3): 2·6/5=2.4
        // (1,2)d=(2,3): 2.4
        // (2,3)d=(3,1): 2·3/4=1.5
        let expected = 2.0 + 2.0 * 2.4 + 1.5;
        assert!((degree_harmonic_mean_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    // --- degree_diff_connectivity ---

    #[test]
    fn ddc_empty() {
        let g = Graph::with_vertices(0);
        assert!(degree_diff_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_isolated() {
        let g = Graph::with_vertices(5);
        assert!(degree_diff_connectivity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ddc_regular_is_m() {
        // Regular: |du-dv|=0 → each 1/√1=1 → DDC = m
        assert!((degree_diff_connectivity(&k3()).unwrap() - 3.0).abs() < 1e-10);
        assert!((degree_diff_connectivity(&k4()).unwrap() - 6.0).abs() < 1e-10);
        assert!((degree_diff_connectivity(&cycle4()).unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn ddc_single_edge() {
        assert!((degree_diff_connectivity(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ddc_star5() {
        // 4 edges (4,1): |4-1|=3 → 4·1/√4 = 4·0.5 = 2
        assert!((degree_diff_connectivity(&star5()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn ddc_path3() {
        // 2 edges (1,2): |1-2|=1 → 2·1/√2
        let expected = 2.0 / 2.0_f64.sqrt();
        assert!((degree_diff_connectivity(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ddc_paw() {
        // (0,1)d=(2,2): 1/√1=1
        // (0,2)d=(2,3): 1/√2
        // (1,2)d=(2,3): 1/√2
        // (2,3)d=(3,1): 1/√3
        let expected = 1.0 + 2.0 / 2.0_f64.sqrt() + 1.0 / 3.0_f64.sqrt();
        assert!((degree_diff_connectivity(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn ddc_le_m() {
        // Each term ≤ 1 → DDC ≤ m
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_diff_connectivity(g).unwrap() <= g.ecount() as f64 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn dhm_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(degree_harmonic_mean_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn sdr_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            assert!(symmetric_degree_ratio(g).unwrap() > 0.0);
        }
    }
}
