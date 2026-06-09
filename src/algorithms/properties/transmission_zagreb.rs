//! Transmission-based indices (ALGO-TR-065).
//!
//! These indices use the **transmission** (or **status**) of each vertex:
//! `σ(v) = Σ_w d(v,w)`, the sum of distances from v to all other vertices.
//!
//! - **First transmission Zagreb index** `TZ₁(G) = Σ_v σ(v)²`
//!   Squared vertex transmissions.
//! - **Second transmission Zagreb index** `TZ₂(G) = Σ_{(u,v)∈E} σ(u)·σ(v)`
//!   Product of endpoint transmissions over edges.
//! - **Reciprocal transmission index** `RT(G) = Σ_v 1/σ(v)`
//!   Sum of reciprocal transmissions (skipping isolated vertices
//!   or vertices in disconnected components with σ=0).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use crate::core::{Graph, IgraphResult};

fn vertex_transmissions(graph: &Graph) -> IgraphResult<Vec<u64>> {
    let n = graph.vcount() as usize;
    let mut sigma = vec![0_u64; n];

    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u];
            if let Ok(nbs) = graph.neighbors(u as u32) {
                for nb in nbs {
                    let idx = nb as usize;
                    if dist[idx] == u32::MAX {
                        dist[idx] = d_u + 1;
                        queue.push_back(idx);
                    }
                }
            }
        }
        let mut total = 0_u64;
        for &d in &dist {
            if d != u32::MAX {
                total = total.saturating_add(u64::from(d));
            }
        }
        sigma[s] = total;
    }

    Ok(sigma)
}

/// Compute the first transmission Zagreb index.
///
/// `TZ₁(G) = Σ_v σ(v)²` where `σ(v) = Σ_w d(v,w)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_transmission_zagreb};
///
/// // Path 0-1-2: σ(0)=3, σ(1)=2, σ(2)=3
/// // TZ₁ = 9 + 4 + 9 = 22
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(first_transmission_zagreb(&g).unwrap(), 22);
/// ```
pub fn first_transmission_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let sigma = vertex_transmissions(graph)?;
    let mut tz1 = 0_u64;

    for &sv in &sigma {
        tz1 = tz1.saturating_add(sv.saturating_mul(sv));
    }

    Ok(tz1)
}

/// Compute the second transmission Zagreb index.
///
/// `TZ₂(G) = Σ_{(u,v)∈E} σ(u) · σ(v)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_transmission_zagreb};
///
/// // Path 0-1-2: σ=[3,2,3]
/// // (0,1): 3×2=6, (1,2): 2×3=6 → TZ₂=12
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_transmission_zagreb(&g).unwrap(), 12);
/// ```
pub fn second_transmission_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let sigma = vertex_transmissions(graph)?;
    let mut tz2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let su = sigma[u as usize];
        let sv = sigma[v as usize];
        tz2 = tz2.saturating_add(su.saturating_mul(sv));
    }

    Ok(tz2)
}

/// Compute the reciprocal transmission index.
///
/// `RT(G) = Σ_v 1/σ(v)` where `σ(v) = Σ_w d(v,w)`.
///
/// Vertices with σ(v) = 0 (isolated or single-vertex components) are
/// skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocal_transmission_index};
///
/// // Path 0-1-2: σ=[3,2,3], RT = 1/3 + 1/2 + 1/3 = 7/6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((reciprocal_transmission_index(&g).unwrap() - 7.0/6.0).abs() < 1e-10);
/// ```
pub fn reciprocal_transmission_index(graph: &Graph) -> IgraphResult<f64> {
    let sigma = vertex_transmissions(graph)?;
    let mut rt = 0.0_f64;

    for &sv in &sigma {
        if sv > 0 {
            rt += 1.0 / sv as f64;
        }
    }

    Ok(rt)
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

    // Helper: verify transmissions
    fn transmissions(g: &Graph) -> Vec<u64> {
        vertex_transmissions(g).unwrap()
    }

    // --- vertex_transmissions ---

    #[test]
    fn sigma_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(transmissions(&g), Vec::<u64>::new());
    }

    #[test]
    fn sigma_isolated() {
        let g = Graph::with_vertices(3);
        assert_eq!(transmissions(&g), vec![0, 0, 0]);
    }

    #[test]
    fn sigma_single_edge() {
        // σ(0)=1, σ(1)=1
        assert_eq!(transmissions(&single_edge()), vec![1, 1]);
    }

    #[test]
    fn sigma_path3() {
        // σ(0)=1+2=3, σ(1)=1+1=2, σ(2)=2+1=3
        assert_eq!(transmissions(&path3()), vec![3, 2, 3]);
    }

    #[test]
    fn sigma_path4() {
        // σ(0)=1+2+3=6, σ(1)=1+1+2=4, σ(2)=2+1+1=4, σ(3)=3+2+1=6
        assert_eq!(transmissions(&path4()), vec![6, 4, 4, 6]);
    }

    #[test]
    fn sigma_k3() {
        // All distances 1, σ(v)=2 for all
        assert_eq!(transmissions(&k3()), vec![2, 2, 2]);
    }

    #[test]
    fn sigma_k4() {
        // σ(v) = 3 for all
        assert_eq!(transmissions(&k4()), vec![3, 3, 3, 3]);
    }

    #[test]
    fn sigma_cycle4() {
        // C4: distances [0,1,2,1] so σ=4 for all
        assert_eq!(transmissions(&cycle4()), vec![4, 4, 4, 4]);
    }

    #[test]
    fn sigma_cycle5() {
        // C5: distances [0,1,2,2,1] so σ=6 for all
        assert_eq!(transmissions(&cycle5()), vec![6, 6, 6, 6, 6]);
    }

    #[test]
    fn sigma_star5() {
        // center: 4×1=4, leaves: 1+2+2+2=7
        assert_eq!(transmissions(&star5()), vec![4, 7, 7, 7, 7]);
    }

    #[test]
    fn sigma_paw() {
        // σ(0)=1+1+2=4, σ(1)=1+1+2=4, σ(2)=1+1+1=3, σ(3)=2+2+1=5
        assert_eq!(transmissions(&paw()), vec![4, 4, 3, 5]);
    }

    #[test]
    fn sigma_sum_equals_wiener() {
        // Σ σ(v) = 2·W(G) where W is the Wiener index
        // For path3: W = 1+2+1 = 4, Σσ = 3+2+3 = 8 = 2×4 ✓
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let sigma = transmissions(g);
            let sum: u64 = sigma.iter().sum();
            assert_eq!(sum % 2, 0);
        }
    }

    // --- first_transmission_zagreb ---

    #[test]
    fn tz1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_transmission_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn tz1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_transmission_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn tz1_single_edge() {
        // σ=[1,1], TZ1 = 1+1 = 2
        assert_eq!(first_transmission_zagreb(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn tz1_path3() {
        // σ=[3,2,3], TZ1 = 9+4+9 = 22
        assert_eq!(first_transmission_zagreb(&path3()).unwrap(), 22);
    }

    #[test]
    fn tz1_path4() {
        // σ=[6,4,4,6], TZ1 = 36+16+16+36 = 104
        assert_eq!(first_transmission_zagreb(&path4()).unwrap(), 104);
    }

    #[test]
    fn tz1_k3() {
        // σ=[2,2,2], TZ1 = 3×4 = 12
        assert_eq!(first_transmission_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn tz1_k4() {
        // σ=[3,3,3,3], TZ1 = 4×9 = 36
        assert_eq!(first_transmission_zagreb(&k4()).unwrap(), 36);
    }

    #[test]
    fn tz1_cycle4() {
        // σ=[4,4,4,4], TZ1 = 4×16 = 64
        assert_eq!(first_transmission_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn tz1_cycle5() {
        // σ=[6,6,6,6,6], TZ1 = 5×36 = 180
        assert_eq!(first_transmission_zagreb(&cycle5()).unwrap(), 180);
    }

    #[test]
    fn tz1_star5() {
        // σ=[4,7,7,7,7], TZ1 = 16+49+49+49+49 = 212
        assert_eq!(first_transmission_zagreb(&star5()).unwrap(), 212);
    }

    #[test]
    fn tz1_paw() {
        // σ=[4,4,3,5], TZ1 = 16+16+9+25 = 66
        assert_eq!(first_transmission_zagreb(&paw()).unwrap(), 66);
    }

    #[test]
    fn tz1_transmission_regular() {
        // For transmission-regular graphs: TZ1 = n·σ²
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = u64::from(g.vcount());
            let s = transmissions(g)[0];
            assert_eq!(first_transmission_zagreb(g).unwrap(), n * s * s);
        }
    }

    // --- second_transmission_zagreb ---

    #[test]
    fn tz2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_transmission_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn tz2_single_edge() {
        // σ=[1,1], 1 edge: 1×1=1
        assert_eq!(second_transmission_zagreb(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn tz2_path3() {
        // σ=[3,2,3], edges: (0,1):6, (1,2):6 → TZ2=12
        assert_eq!(second_transmission_zagreb(&path3()).unwrap(), 12);
    }

    #[test]
    fn tz2_path4() {
        // σ=[6,4,4,6], edges: (0,1):24, (1,2):16, (2,3):24 → TZ2=64
        assert_eq!(second_transmission_zagreb(&path4()).unwrap(), 64);
    }

    #[test]
    fn tz2_k3() {
        // σ=[2,2,2], 3 edges × 4 = 12
        assert_eq!(second_transmission_zagreb(&k3()).unwrap(), 12);
    }

    #[test]
    fn tz2_k4() {
        // σ=[3,3,3,3], 6 × 9 = 54
        assert_eq!(second_transmission_zagreb(&k4()).unwrap(), 54);
    }

    #[test]
    fn tz2_cycle4() {
        // σ=[4,4,4,4], 4 × 16 = 64
        assert_eq!(second_transmission_zagreb(&cycle4()).unwrap(), 64);
    }

    #[test]
    fn tz2_star5() {
        // σ=[4,7,7,7,7], edges: (0,1):28, (0,2):28, (0,3):28, (0,4):28
        // TZ2 = 4×28 = 112
        assert_eq!(second_transmission_zagreb(&star5()).unwrap(), 112);
    }

    #[test]
    fn tz2_paw() {
        // σ=[4,4,3,5], edges: (0,1):16, (0,2):12, (1,2):12, (2,3):15
        // TZ2 = 16+12+12+15 = 55
        assert_eq!(second_transmission_zagreb(&paw()).unwrap(), 55);
    }

    // --- reciprocal_transmission_index ---

    #[test]
    fn rt_empty() {
        let g = Graph::with_vertices(0);
        assert!((reciprocal_transmission_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rt_isolated() {
        let g = Graph::with_vertices(5);
        assert!((reciprocal_transmission_index(&g).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn rt_single_edge() {
        // σ=[1,1], RT = 1+1 = 2
        assert!((reciprocal_transmission_index(&single_edge()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rt_path3() {
        // σ=[3,2,3], RT = 1/3+1/2+1/3 = 7/6
        let expected = 7.0 / 6.0;
        assert!((reciprocal_transmission_index(&path3()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_path4() {
        // σ=[6,4,4,6], RT = 1/6+1/4+1/4+1/6 = 2/6+2/4 = 1/3+1/2 = 5/6
        let expected = 5.0 / 6.0;
        assert!((reciprocal_transmission_index(&path4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_k3() {
        // σ=[2,2,2], RT = 3/2 = 1.5
        assert!((reciprocal_transmission_index(&k3()).unwrap() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn rt_k4() {
        // σ=[3,3,3,3], RT = 4/3
        let expected = 4.0 / 3.0;
        assert!((reciprocal_transmission_index(&k4()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_cycle4() {
        // σ=[4,4,4,4], RT = 4/4 = 1
        assert!((reciprocal_transmission_index(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rt_cycle5() {
        // σ=[6,6,6,6,6], RT = 5/6
        let expected = 5.0 / 6.0;
        assert!((reciprocal_transmission_index(&cycle5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_star5() {
        // σ=[4,7,7,7,7], RT = 1/4 + 4/7
        let expected = 0.25 + 4.0 / 7.0;
        assert!((reciprocal_transmission_index(&star5()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_paw() {
        // σ=[4,4,3,5], RT = 1/4+1/4+1/3+1/5 = 15/60+15/60+20/60+12/60 = 62/60 = 31/30
        let expected = 31.0 / 30.0;
        assert!((reciprocal_transmission_index(&paw()).unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn rt_transmission_regular() {
        // Transmission-regular: RT = n/σ
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let n = f64::from(g.vcount());
            let s = transmissions(g)[0] as f64;
            let expected = n / s;
            assert!((reciprocal_transmission_index(g).unwrap() - expected).abs() < 1e-8);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn all_positive_for_connected() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(first_transmission_zagreb(g).unwrap() > 0);
            assert!(second_transmission_zagreb(g).unwrap() > 0);
            assert!(reciprocal_transmission_index(g).unwrap() > 0.0);
        }
    }
}
