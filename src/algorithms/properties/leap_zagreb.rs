//! Leap Zagreb indices (ALGO-TR-066).
//!
//! These indices use the **second degree** (or **2-distance degree**)
//! of each vertex: `d₂(v) = |{w : dist(v,w) = 2}|`, the number of
//! vertices at distance exactly 2 from v.
//!
//! - **First leap Zagreb** `LM₁(G) = Σ_v d₂(v)²`
//! - **Second leap Zagreb** `LM₂(G) = Σ_{(u,v)∈E} d₂(u)·d₂(v)`
//! - **Third leap Zagreb** `LM₃(G) = Σ_v d(v)·d₂(v)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

use crate::core::{Graph, IgraphResult};

fn second_degrees(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let mut d2 = vec![0_u32; n];

    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[u];
            if d_u >= 2 {
                continue;
            }
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
        let mut count = 0_u32;
        for &d in &dist {
            if d == 2 {
                count += 1;
            }
        }
        d2[s] = count;
    }

    Ok(d2)
}

/// Compute the first leap Zagreb index.
///
/// `LM₁(G) = Σ_v d₂(v)²` where `d₂(v)` is the number of vertices
/// at distance exactly 2 from v.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, first_leap_zagreb};
///
/// // Path 0-1-2: d₂(0)=1, d₂(1)=0, d₂(2)=1
/// // LM₁ = 1 + 0 + 1 = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(first_leap_zagreb(&g).unwrap(), 2);
/// ```
pub fn first_leap_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let d2 = second_degrees(graph)?;
    let mut lm1 = 0_u64;

    for &dv in &d2 {
        let dv64 = u64::from(dv);
        lm1 = lm1.saturating_add(dv64.saturating_mul(dv64));
    }

    Ok(lm1)
}

/// Compute the second leap Zagreb index.
///
/// `LM₂(G) = Σ_{(u,v)∈E} d₂(u)·d₂(v)`
///
/// Self-loops are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, second_leap_zagreb};
///
/// // Path 0-1-2: d₂=[1,0,1]
/// // edges: (0,1):1×0=0, (1,2):0×1=0 → LM₂=0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(second_leap_zagreb(&g).unwrap(), 0);
/// ```
pub fn second_leap_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let d2 = second_degrees(graph)?;
    let mut lm2 = 0_u64;

    for (u, v) in graph.edges() {
        if u == v {
            continue;
        }
        let du = u64::from(d2[u as usize]);
        let dv = u64::from(d2[v as usize]);
        lm2 = lm2.saturating_add(du.saturating_mul(dv));
    }

    Ok(lm2)
}

/// Compute the third leap Zagreb index.
///
/// `LM₃(G) = Σ_v d(v)·d₂(v)` where `d(v)` is the ordinary degree
/// and `d₂(v)` is the second degree (count of vertices at distance 2).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, third_leap_zagreb};
///
/// // Path 0-1-2: d=[1,2,1], d₂=[1,0,1]
/// // LM₃ = 1×1 + 2×0 + 1×1 = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(third_leap_zagreb(&g).unwrap(), 2);
/// ```
pub fn third_leap_zagreb(graph: &Graph) -> IgraphResult<u64> {
    let d2 = second_degrees(graph)?;
    let n = graph.vcount() as usize;
    let mut lm3 = 0_u64;

    for i in 0..n {
        let deg = graph.degree(i as u32)? as u64;
        let d2v = u64::from(d2[i]);
        lm3 = lm3.saturating_add(deg.saturating_mul(d2v));
    }

    Ok(lm3)
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

    fn petersen() -> Graph {
        Graph::from_edges(
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 0),
                (0, 5),
                (1, 6),
                (2, 7),
                (3, 8),
                (4, 9),
                (5, 7),
                (5, 8),
                (6, 8),
                (6, 9),
                (7, 9),
            ],
            false,
            Some(10),
        )
        .unwrap()
    }

    fn d2(g: &Graph) -> Vec<u32> {
        second_degrees(g).unwrap()
    }

    // --- second_degrees ---

    #[test]
    fn d2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(d2(&g), Vec::<u32>::new());
    }

    #[test]
    fn d2_isolated() {
        let g = Graph::with_vertices(3);
        assert_eq!(d2(&g), vec![0, 0, 0]);
    }

    #[test]
    fn d2_single_edge() {
        // 0-1: no vertex at distance 2
        assert_eq!(d2(&single_edge()), vec![0, 0]);
    }

    #[test]
    fn d2_path3() {
        // 0-1-2: d₂(0)=1 (vertex 2), d₂(1)=0, d₂(2)=1 (vertex 0)
        assert_eq!(d2(&path3()), vec![1, 0, 1]);
    }

    #[test]
    fn d2_path4() {
        // 0-1-2-3: d₂(0)=1(2), d₂(1)=1(3), d₂(2)=1(0), d₂(3)=1(1)
        assert_eq!(d2(&path4()), vec![1, 1, 1, 1]);
    }

    #[test]
    fn d2_k3() {
        // All pairs connected directly — no vertex at distance 2
        assert_eq!(d2(&k3()), vec![0, 0, 0]);
    }

    #[test]
    fn d2_k4() {
        assert_eq!(d2(&k4()), vec![0, 0, 0, 0]);
    }

    #[test]
    fn d2_cycle4() {
        // C4: each vertex has 2 neighbors and 1 vertex at distance 2
        assert_eq!(d2(&cycle4()), vec![1, 1, 1, 1]);
    }

    #[test]
    fn d2_cycle5() {
        // C5: each vertex has 2 neighbors, 2 vertices at distance 2
        assert_eq!(d2(&cycle5()), vec![2, 2, 2, 2, 2]);
    }

    #[test]
    fn d2_star5() {
        // Star: center has 0 at dist 2 (all at dist 1),
        // each leaf has 3 other leaves at dist 2
        assert_eq!(d2(&star5()), vec![0, 3, 3, 3, 3]);
    }

    #[test]
    fn d2_paw() {
        // Paw: 0-1, 0-2, 1-2, 2-3
        // d₂(0)=1 (vertex 3), d₂(1)=1 (vertex 3), d₂(2)=0, d₂(3)=2 (0,1)
        assert_eq!(d2(&paw()), vec![1, 1, 0, 2]);
    }

    #[test]
    fn d2_petersen() {
        // Petersen graph is 3-regular, diameter 2 — each vertex has
        // 3 neighbors and 6 vertices at distance 2
        let d2v = d2(&petersen());
        for &v in &d2v {
            assert_eq!(v, 6);
        }
    }

    // --- first_leap_zagreb ---

    #[test]
    fn lm1_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(first_leap_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn lm1_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(first_leap_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn lm1_single_edge() {
        assert_eq!(first_leap_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn lm1_path3() {
        // d₂=[1,0,1], LM₁ = 1+0+1 = 2
        assert_eq!(first_leap_zagreb(&path3()).unwrap(), 2);
    }

    #[test]
    fn lm1_path4() {
        // d₂=[1,1,1,1], LM₁ = 4
        assert_eq!(first_leap_zagreb(&path4()).unwrap(), 4);
    }

    #[test]
    fn lm1_k3() {
        assert_eq!(first_leap_zagreb(&k3()).unwrap(), 0);
    }

    #[test]
    fn lm1_k4() {
        assert_eq!(first_leap_zagreb(&k4()).unwrap(), 0);
    }

    #[test]
    fn lm1_cycle4() {
        // d₂=[1,1,1,1], LM₁ = 4
        assert_eq!(first_leap_zagreb(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn lm1_cycle5() {
        // d₂=[2,2,2,2,2], LM₁ = 5×4 = 20
        assert_eq!(first_leap_zagreb(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn lm1_star5() {
        // d₂=[0,3,3,3,3], LM₁ = 0+9+9+9+9 = 36
        assert_eq!(first_leap_zagreb(&star5()).unwrap(), 36);
    }

    #[test]
    fn lm1_paw() {
        // d₂=[1,1,0,2], LM₁ = 1+1+0+4 = 6
        assert_eq!(first_leap_zagreb(&paw()).unwrap(), 6);
    }

    #[test]
    fn lm1_petersen() {
        // d₂=6 for all, LM₁ = 10×36 = 360
        assert_eq!(first_leap_zagreb(&petersen()).unwrap(), 360);
    }

    #[test]
    fn lm1_complete_is_zero() {
        // Complete graphs: all pairs at distance 1, d₂=0 for all
        for g in &[k3(), k4()] {
            assert_eq!(first_leap_zagreb(g).unwrap(), 0);
        }
    }

    // --- second_leap_zagreb ---

    #[test]
    fn lm2_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(second_leap_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn lm2_single_edge() {
        assert_eq!(second_leap_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn lm2_path3() {
        // d₂=[1,0,1], edges: (0,1):0, (1,2):0 → 0
        assert_eq!(second_leap_zagreb(&path3()).unwrap(), 0);
    }

    #[test]
    fn lm2_path4() {
        // d₂=[1,1,1,1], edges: (0,1):1, (1,2):1, (2,3):1 → 3
        assert_eq!(second_leap_zagreb(&path4()).unwrap(), 3);
    }

    #[test]
    fn lm2_k3() {
        assert_eq!(second_leap_zagreb(&k3()).unwrap(), 0);
    }

    #[test]
    fn lm2_k4() {
        assert_eq!(second_leap_zagreb(&k4()).unwrap(), 0);
    }

    #[test]
    fn lm2_cycle4() {
        // d₂=[1,1,1,1], 4 edges × 1 = 4
        assert_eq!(second_leap_zagreb(&cycle4()).unwrap(), 4);
    }

    #[test]
    fn lm2_cycle5() {
        // d₂=[2,2,2,2,2], 5 edges × 4 = 20
        assert_eq!(second_leap_zagreb(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn lm2_star5() {
        // d₂=[0,3,3,3,3], edges are (0,leaf): 0×3=0 → 0
        assert_eq!(second_leap_zagreb(&star5()).unwrap(), 0);
    }

    #[test]
    fn lm2_paw() {
        // d₂=[1,1,0,2], edges: (0,1):1, (0,2):0, (1,2):0, (2,3):0 → 1
        assert_eq!(second_leap_zagreb(&paw()).unwrap(), 1);
    }

    #[test]
    fn lm2_petersen() {
        // d₂=6 for all, 15 edges × 36 = 540
        assert_eq!(second_leap_zagreb(&petersen()).unwrap(), 540);
    }

    // --- third_leap_zagreb ---

    #[test]
    fn lm3_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(third_leap_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn lm3_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(third_leap_zagreb(&g).unwrap(), 0);
    }

    #[test]
    fn lm3_single_edge() {
        // d=[1,1], d₂=[0,0] → 0
        assert_eq!(third_leap_zagreb(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn lm3_path3() {
        // d=[1,2,1], d₂=[1,0,1], LM₃ = 1×1+2×0+1×1 = 2
        assert_eq!(third_leap_zagreb(&path3()).unwrap(), 2);
    }

    #[test]
    fn lm3_path4() {
        // d=[1,2,2,1], d₂=[1,1,1,1], LM₃ = 1+2+2+1 = 6
        assert_eq!(third_leap_zagreb(&path4()).unwrap(), 6);
    }

    #[test]
    fn lm3_k3() {
        assert_eq!(third_leap_zagreb(&k3()).unwrap(), 0);
    }

    #[test]
    fn lm3_k4() {
        assert_eq!(third_leap_zagreb(&k4()).unwrap(), 0);
    }

    #[test]
    fn lm3_cycle4() {
        // d=[2,2,2,2], d₂=[1,1,1,1], LM₃ = 4×2 = 8
        assert_eq!(third_leap_zagreb(&cycle4()).unwrap(), 8);
    }

    #[test]
    fn lm3_cycle5() {
        // d=[2,2,2,2,2], d₂=[2,2,2,2,2], LM₃ = 5×4 = 20
        assert_eq!(third_leap_zagreb(&cycle5()).unwrap(), 20);
    }

    #[test]
    fn lm3_star5() {
        // d=[4,1,1,1,1], d₂=[0,3,3,3,3], LM₃ = 0+3+3+3+3 = 12
        assert_eq!(third_leap_zagreb(&star5()).unwrap(), 12);
    }

    #[test]
    fn lm3_paw() {
        // d=[2,2,3,1], d₂=[1,1,0,2], LM₃ = 2+2+0+2 = 6
        assert_eq!(third_leap_zagreb(&paw()).unwrap(), 6);
    }

    #[test]
    fn lm3_petersen() {
        // d=3, d₂=6 for all, LM₃ = 10×18 = 180
        assert_eq!(third_leap_zagreb(&petersen()).unwrap(), 180);
    }

    // --- cross-consistency ---

    #[test]
    fn lm3_equals_sum_of_d_times_d2() {
        // LM₃ = Σ d(v)·d₂(v), verify manually
        for g in &[path3(), path4(), cycle4(), cycle5(), star5(), paw()] {
            let d2v = d2(g);
            let n = g.vcount() as usize;
            let mut expected = 0_u64;
            for i in 0..n {
                let deg = g.degree(i as u32).unwrap() as u64;
                expected += deg * u64::from(d2v[i]);
            }
            assert_eq!(third_leap_zagreb(g).unwrap(), expected);
        }
    }

    #[test]
    fn all_complete_graphs_have_zero_leap_zagreb() {
        for g in &[k3(), k4()] {
            assert_eq!(first_leap_zagreb(g).unwrap(), 0);
            assert_eq!(second_leap_zagreb(g).unwrap(), 0);
            assert_eq!(third_leap_zagreb(g).unwrap(), 0);
        }
    }

    #[test]
    fn all_positive_for_nonclique_connected() {
        // For connected non-complete graphs with at least 3 vertices,
        // LM₁ and LM₃ should be positive (some vertex has d₂ > 0)
        for g in &[path3(), path4(), cycle4(), cycle5(), star5(), paw()] {
            assert!(first_leap_zagreb(g).unwrap() > 0);
            assert!(third_leap_zagreb(g).unwrap() > 0);
        }
    }
}
