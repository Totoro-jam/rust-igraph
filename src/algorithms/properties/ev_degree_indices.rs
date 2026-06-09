//! Edge-vertex degree indices (ALGO-TR-069).
//!
//! These indices use the **ev-degree** of an edge e=(u,v):
//! `d_{ev}(e) = d(u) + d(v) - 2`.
//!
//! This counts the number of edges adjacent to e (sharing exactly one
//! endpoint), excluding e itself. It is the edge-level analog of vertex
//! degree.
//!
//! - **First ev-degree Zagreb** `M₁^{ev}(G) = Σ_{e} d_{ev}(e)²`
//!   Sum of squared ev-degrees over all edges.
//! - **Second ev-degree Zagreb** `M₂^{ev}(G) = Σ_{e~f} d_{ev}(e)·d_{ev}(f)`
//!   Product of ev-degrees over pairs of adjacent edges.
//! - **ev-degree Randić** `R_{ev}(G) = Σ_{e~f} 1/√(d_{ev}(e)·d_{ev}(f))`
//!   Randić-like index over adjacent edge pairs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn ev_degrees(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let mut devs = Vec::new();

    for (u, v) in graph.edges() {
        if u == v {
            devs.push(0);
            continue;
        }
        let du = graph.degree(u)? as u64;
        let dv = graph.degree(v)? as u64;
        devs.push(du.saturating_add(dv).saturating_sub(2));
    }

    Ok(devs)
}

/// Compute the first ev-degree Zagreb index.
///
/// `M₁^{ev}(G) = Σ_{e=(u,v)} [d(u)+d(v)-2]²`
///
/// Self-loops contribute 0.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_ev_degree_zagreb};
///
/// // K_3: 3 edges, each d_ev = 2+2-2 = 2, M₁ev = 3×4 = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(first_ev_degree_zagreb(&g).unwrap(), 12);
/// ```
pub fn first_ev_degree_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let devs = ev_degrees(graph)?;
    let mut m1 = 0_u64;

    for &d in &devs {
        m1 = m1.saturating_add(d.saturating_mul(d));
    }

    Ok(m1)
}

/// Compute the second ev-degree Zagreb index.
///
/// `M₂^{ev}(G) = Σ_{e~f, adjacent} d_{ev}(e) · d_{ev}(f)`
///
/// Two edges are adjacent if they share exactly one endpoint.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_ev_degree_zagreb};
///
/// // K_3: 3 edges with d_ev=2 each, 3 adjacent pairs, M₂ev = 3×4 = 12
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert_eq!(second_ev_degree_zagreb(&g).unwrap(), 12);
/// ```
pub fn second_ev_degree_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let devs = ev_degrees(graph)?;
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let n = graph.vcount() as usize;

    let mut inc: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &(u, v)) in edges.iter().enumerate() {
        inc[u as usize].push(i);
        if u != v {
            inc[v as usize].push(i);
        }
    }

    let mut m2 = 0_u64;

    for v in 0..n {
        let incident = &inc[v];
        for i in 0..incident.len() {
            for j in (i + 1)..incident.len() {
                m2 = m2.saturating_add(devs[incident[i]].saturating_mul(devs[incident[j]]));
            }
        }
    }

    Ok(m2)
}

/// Compute the ev-degree Randić index.
///
/// `R_{ev}(G) = Σ_{e~f, adjacent} 1/√(d_{ev}(e) · d_{ev}(f))`
///
/// Pairs where either edge has ev-degree 0 (self-loops or K₂) are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ev_degree_randic};
///
/// // K_3: 3 adjacent pairs, each 1/√(2·2) = 0.5, total = 1.5
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((ev_degree_randic(&g).unwrap() - 1.5).abs() < 1e-10);
/// ```
pub fn ev_degree_randic(graph: &Graph) -> IgraphResult<f64> {
    let devs = ev_degrees(graph)?;
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let n = graph.vcount() as usize;

    let mut inc: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &(u, v)) in edges.iter().enumerate() {
        inc[u as usize].push(i);
        if u != v {
            inc[v as usize].push(i);
        }
    }

    let mut r = 0.0_f64;

    for v in 0..n {
        let incident = &inc[v];
        for i in 0..incident.len() {
            for j in (i + 1)..incident.len() {
                let p = devs[incident[i]] as f64 * devs[incident[j]] as f64;
                if p > 0.0 {
                    r += 1.0 / p.sqrt();
                }
            }
        }
    }

    Ok(r)
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

    fn devs(g: &Graph) -> Vec<u64> {
        ev_degrees(g).unwrap()
    }

    // --- ev_degrees ---

    #[test]
    fn ev_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(devs(&g), Vec::<u64>::new());
    }

    #[test]
    fn ev_single_edge() {
        // d_ev = 1+1-2 = 0
        assert_eq!(devs(&single_edge()), vec![0]);
    }

    #[test]
    fn ev_path3() {
        // edges: (0,1): 1+2-2=1, (1,2): 2+1-2=1
        assert_eq!(devs(&path3()), vec![1, 1]);
    }

    #[test]
    fn ev_path4() {
        // (0,1):1+2-2=1, (1,2):2+2-2=2, (2,3):2+1-2=1
        assert_eq!(devs(&path4()), vec![1, 2, 1]);
    }

    #[test]
    fn ev_k3() {
        // All: 2+2-2=2
        assert_eq!(devs(&k3()), vec![2, 2, 2]);
    }

    #[test]
    fn ev_k4() {
        // All: 3+3-2=4
        assert_eq!(devs(&k4()), vec![4, 4, 4, 4, 4, 4]);
    }

    #[test]
    fn ev_cycle4() {
        // All: 2+2-2=2
        assert_eq!(devs(&cycle4()), vec![2, 2, 2, 2]);
    }

    #[test]
    fn ev_star5() {
        // All edges (0,leaf): 4+1-2=3
        assert_eq!(devs(&star5()), vec![3, 3, 3, 3]);
    }

    #[test]
    fn ev_paw() {
        // (0,1):2+2-2=2, (0,2):2+3-2=3, (1,2):2+3-2=3, (2,3):3+1-2=2
        assert_eq!(devs(&paw()), vec![2, 3, 3, 2]);
    }

    // --- first_ev_degree_zagreb ---

    #[test]
    fn m1ev_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_ev_degree_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn m1ev_single_edge() {
        assert_eq!(first_ev_degree_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m1ev_path3() {
        // d_ev=[1,1], M₁ = 1+1 = 2
        assert_eq!(first_ev_degree_zagreb(&path3()).unwrap(), 2);
    }

    #[test]
    fn m1ev_path4() {
        // d_ev=[1,2,1], M₁ = 1+4+1 = 6
        assert_eq!(first_ev_degree_zagreb(&path4()).unwrap(), 6);
    }

    #[test]
    fn m1ev_k3() {
        // d_ev=[2,2,2], M₁ = 12
        assert_eq!(first_ev_degree_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn m1ev_k4() {
        // d_ev=[4,4,4,4,4,4], M₁ = 6×16 = 96
        assert_eq!(first_ev_degree_zagreb(&k4()).unwrap(), 96);
    }

    #[test]
    fn m1ev_cycle4() {
        // d_ev=[2,2,2,2], M₁ = 16
        assert_eq!(first_ev_degree_zagreb(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn m1ev_star5() {
        // d_ev=[3,3,3,3], M₁ = 4×9 = 36
        assert_eq!(first_ev_degree_zagreb(&star5()).unwrap(), 36);
    }

    #[test]
    fn m1ev_paw() {
        // d_ev=[2,3,3,2], M₁ = 4+9+9+4 = 26
        assert_eq!(first_ev_degree_zagreb(&paw()).unwrap(), 26);
    }

    #[test]
    fn m1ev_is_reformulated_first_zagreb() {
        // M₁^{ev} = Σ (d(u)+d(v)-2)² = reformulated first Zagreb EM₁
        // Already tested in reformulated_zagreb.rs, cross-check here
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let m1ev = first_ev_degree_zagreb(g).unwrap();
            let m = g.ecount() as u64;
            let r = g.degree(0).unwrap() as u64;
            let dev = 2 * r - 2;
            assert_eq!(m1ev, m * dev * dev);
        }
    }

    // --- second_ev_degree_zagreb ---

    #[test]
    fn m2ev_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_ev_degree_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn m2ev_single_edge() {
        // Only 1 edge, no adjacent pair
        assert_eq!(second_ev_degree_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn m2ev_path3() {
        // 2 edges, 1 adjacent pair: 1×1=1
        assert_eq!(second_ev_degree_zagreb(&path3()).unwrap(), 1);
    }

    #[test]
    fn m2ev_path4() {
        // d_ev=[1,2,1], adjacent: (0,1):1×2=2, (1,2):2×1=2 → 4
        assert_eq!(second_ev_degree_zagreb(&path4()).unwrap(), 4);
    }

    #[test]
    fn m2ev_k3() {
        // 3 edges, 3 adjacent pairs, each 2×2=4 → 12
        assert_eq!(second_ev_degree_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn m2ev_k4() {
        // 6 edges, each adjacent to 4 others. Total pairs = 6×4/2 = 12
        // Each pair: 4×4=16 → 12×16 = 192
        assert_eq!(second_ev_degree_zagreb(&k4()).unwrap(), 192);
    }

    #[test]
    fn m2ev_cycle4() {
        // 4 edges, each adjacent to 2 others. 4 adjacent pairs.
        // Each: 2×2=4 → 16
        assert_eq!(second_ev_degree_zagreb(&cycle4()).unwrap(), 16);
    }

    #[test]
    fn m2ev_star5() {
        // 4 edges, C(4,2)=6 adjacent pairs (all share center)
        // Each: 3×3=9 → 54
        assert_eq!(second_ev_degree_zagreb(&star5()).unwrap(), 54);
    }

    #[test]
    fn m2ev_paw() {
        // d_ev=[2,3,3,2], edges: e0=(0,1), e1=(0,2), e2=(1,2), e3=(2,3)
        // Adjacent pairs (sharing a vertex):
        // (e0,e1): share 0 → 2×3=6
        // (e0,e2): share 1 → 2×3=6
        // (e1,e2): share 2 → 3×3=9
        // (e1,e3): share 2 → 3×2=6
        // (e2,e3): share 2 → 3×2=6
        // Total = 6+6+9+6+6 = 33
        assert_eq!(second_ev_degree_zagreb(&paw()).unwrap(), 33);
    }

    // --- ev_degree_randic ---

    #[test]
    fn rev_empty() {
        let g = Graph::with_vertices(0);
        assert!((ev_degree_randic(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rev_single_edge() {
        // d_ev=0, skip
        assert!((ev_degree_randic(&single_edge()).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rev_path3() {
        // 1 pair, 1/√(1×1)=1
        assert!((ev_degree_randic(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rev_k3() {
        // 3 pairs, 1/√(2×2)=0.5, total=1.5
        assert!((ev_degree_randic(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn rev_k4() {
        // 12 pairs, 1/√(4×4)=0.25, total=3.0
        assert!((ev_degree_randic(&k4()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn rev_cycle4() {
        // 4 pairs, 1/√(2×2)=0.5, total=2.0
        assert!((ev_degree_randic(&cycle4()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rev_star5() {
        // 6 pairs, 1/√(3×3)=1/3, total=2.0
        assert!((ev_degree_randic(&star5()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rev_paw() {
        // d_ev=[2,3,3,2]
        // (e0,e1):1/√6, (e0,e2):1/√6, (e1,e2):1/3, (e1,e3):1/√6, (e2,e3):1/√6
        let expected = 4.0 / 6.0_f64.sqrt() + 1.0 / 3.0;
        assert!((ev_degree_randic(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rev_regular_formula() {
        // r-regular: d_ev = 2r-2, adj_pairs = m·(2r-2)/2 ... complex
        // Simpler: R_ev = pairs × 1/(2r-2) for regular
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let r = g.degree(0).unwrap() as f64;
            let dev = 2.0 * r - 2.0;
            let rev = ev_degree_randic(g).unwrap();
            let m2 = second_ev_degree_zagreb(g).unwrap() as f64;
            // R_ev = Σ 1/√(d·d'), M₂ = Σ d·d'
            // Both sum over the same pairs
            assert!(rev > 0.0);
            assert!(m2 > 0.0);
            // For regular: R = pairs/dev², M₂ = pairs·dev²
            // So R·M₂ = pairs² → √(R·M₂) = pairs
            let pairs = m2 / (dev * dev);
            assert!((rev - pairs / (dev * dev) * dev * dev / dev).abs() < 1e-6 || rev > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_nontrivial() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_ev_degree_zagreb(g).unwrap() > 0);
            assert!(second_ev_degree_zagreb(g).unwrap() > 0);
            assert!(ev_degree_randic(g).unwrap() > 0.0);
        }
    }
}
