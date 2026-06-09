//! Schultz molecular topological index (ALGO-TR-053).
//!
//! **Schultz index** `S(G) = Σ_{u<v} (d(u) + d(v)) · dist(u,v)`
//!
//! Introduced by Schultz (1989). Measures molecular topology by
//! weighting shortest-path distances with endpoint degree sums.
//! Related to the Gutman index (which uses degree products instead
//! of sums); both are defined in the `mostar_index` module.
//!
//! For disconnected graphs, infinite distances are excluded from
//! the summation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

fn all_pairs_bfs(graph: &Graph, n: usize) -> Vec<u32> {
    let mut dist = vec![u32::MAX; n * n];
    for s in 0..n {
        dist[s * n + s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s as u32);
        while let Some(u) = queue.pop_front() {
            let d_u = dist[s * n + u as usize];
            if let Ok(nbs) = graph.neighbors(u) {
                for nb in nbs {
                    let idx = s * n + nb as usize;
                    if dist[idx] == u32::MAX {
                        dist[idx] = d_u + 1;
                        queue.push_back(nb);
                    }
                }
            }
        }
    }
    dist
}

/// Compute the Schultz index (degree-distance index).
///
/// `S(G) = Σ_{u<v} (d(u) + d(v)) · dist(u,v)`
///
/// Only finite distances contribute. Returns 0 for graphs with
/// fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, schultz_index};
///
/// // Path 0-1-2: degrees [1,2,1], distances d(0,1)=1, d(0,2)=2, d(1,2)=1
/// // S = (1+2)·1 + (1+1)·2 + (2+1)·1 = 3+4+3 = 10
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(schultz_index(&g).unwrap(), 10);
/// ```
pub fn schultz_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut deg = vec![0_usize; n];
    for v in 0..n as u32 {
        deg[v as usize] = graph.degree(v)?;
    }

    let mut s: u64 = 0;
    for u in 0..n {
        for v in (u + 1)..n {
            let d = dist[u * n + v];
            if d == u32::MAX {
                continue;
            }
            let sum_deg = (deg[u] as u64).saturating_add(deg[v] as u64);
            s = s.saturating_add(sum_deg.saturating_mul(u64::from(d)));
        }
    }

    Ok(s)
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

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
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

    fn cycle6() -> Graph {
        Graph::from_edges(
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)],
            false,
            Some(6),
        )
        .unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    #[test]
    fn schultz_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(schultz_index(&g).unwrap(), 0);
    }

    #[test]
    fn schultz_single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(schultz_index(&g).unwrap(), 0);
    }

    #[test]
    fn schultz_isolated() {
        let g = Graph::with_vertices(5);
        assert_eq!(schultz_index(&g).unwrap(), 0);
    }

    #[test]
    fn schultz_single_edge() {
        // degrees [1,1], dist=1: S = (1+1)·1 = 2
        assert_eq!(schultz_index(&single_edge()).unwrap(), 2);
    }

    #[test]
    fn schultz_path3() {
        // degrees [1,2,1], dists: (0,1)=1,(0,2)=2,(1,2)=1
        // S = (1+2)·1 + (1+1)·2 + (2+1)·1 = 3+4+3 = 10
        assert_eq!(schultz_index(&path3()).unwrap(), 10);
    }

    #[test]
    fn schultz_path4() {
        // degrees [1,2,2,1]
        // (0,1):(1+2)·1=3, (0,2):(1+2)·2=6, (0,3):(1+1)·3=6
        // (1,2):(2+2)·1=4, (1,3):(2+1)·2=6, (2,3):(2+1)·1=3
        // S = 3+6+6+4+6+3 = 28
        assert_eq!(schultz_index(&path4()).unwrap(), 28);
    }

    #[test]
    fn schultz_path5() {
        // degrees [1,2,2,2,1]
        // (0,1):(1+2)·1=3, (0,2):(1+2)·2=6, (0,3):(1+2)·3=9, (0,4):(1+1)·4=8
        // (1,2):(2+2)·1=4, (1,3):(2+2)·2=8, (1,4):(2+1)·3=9
        // (2,3):(2+2)·1=4, (2,4):(2+1)·2=6
        // (3,4):(2+1)·1=3
        // S = 3+6+9+8+4+8+9+4+6+3 = 60
        assert_eq!(schultz_index(&path5()).unwrap(), 60);
    }

    #[test]
    fn schultz_k3() {
        // degrees [2,2,2], all dists=1, 3 pairs
        // S = 3·(2+2)·1 = 12
        assert_eq!(schultz_index(&k3()).unwrap(), 12);
    }

    #[test]
    fn schultz_k4() {
        // degrees [3,3,3,3], all dists=1, 6 pairs
        // S = 6·(3+3)·1 = 36
        assert_eq!(schultz_index(&k4()).unwrap(), 36);
    }

    #[test]
    fn schultz_cycle4() {
        // degrees [2,2,2,2]
        // (0,1)=1,(0,2)=2,(0,3)=1,(1,2)=1,(1,3)=2,(2,3)=1
        // 4 pairs at dist 1: 4·(2+2)·1=16
        // 2 pairs at dist 2: 2·(2+2)·2=16
        // S = 32
        assert_eq!(schultz_index(&cycle4()).unwrap(), 32);
    }

    #[test]
    fn schultz_cycle5() {
        // degrees [2,2,2,2,2]
        // 5 pairs at dist 1, 5 pairs at dist 2
        // S = 5·4·1 + 5·4·2 = 20+40 = 60
        assert_eq!(schultz_index(&cycle5()).unwrap(), 60);
    }

    #[test]
    fn schultz_cycle6() {
        // degrees [2,2,2,2,2,2], 15 pairs
        // 6 at dist 1: 6·4·1=24
        // 6 at dist 2: 6·4·2=48
        // 3 at dist 3: 3·4·3=36
        // S = 24+48+36 = 108
        assert_eq!(schultz_index(&cycle6()).unwrap(), 108);
    }

    #[test]
    fn schultz_star5() {
        // degrees [4,1,1,1,1]
        // (0,leaf): dist=1, 4 pairs, each (4+1)·1=5 → 20
        // (leaf,leaf): dist=2, 6 pairs, each (1+1)·2=4 → 24
        // S = 20+24 = 44
        assert_eq!(schultz_index(&star5()).unwrap(), 44);
    }

    #[test]
    fn schultz_paw() {
        // degrees [2,2,3,1]
        // (0,1):(2+2)·1=4, (0,2):(2+3)·1=5, (0,3):(2+1)·2=6
        // (1,2):(2+3)·1=5, (1,3):(2+1)·2=6, (2,3):(3+1)·1=4
        // S = 4+5+6+5+6+4 = 30
        assert_eq!(schultz_index(&paw()).unwrap(), 30);
    }

    #[test]
    fn schultz_disconnected() {
        // Two isolated edges: 0-1 and 2-3
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        // Only finite pairs: (0,1) and (2,3)
        // S = (1+1)·1 + (1+1)·1 = 4
        assert_eq!(schultz_index(&g).unwrap(), 4);
    }

    #[test]
    fn schultz_regular_formula() {
        // r-regular: S = 2r · W(G) where W = Wiener index
        for g in &[k3(), k4(), cycle4(), cycle5(), cycle6()] {
            let r = g.degree(0).unwrap() as u64;
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut w: u64 = 0;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d != u32::MAX {
                        w += u64::from(d);
                    }
                }
            }
            let expected = 2 * r * w;
            assert_eq!(schultz_index(g).unwrap(), expected, "r={r}, W={w}");
        }
    }

    #[test]
    fn schultz_nonneg() {
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            paw(),
        ] {
            assert!(schultz_index(g).unwrap() > 0);
        }
    }

    #[test]
    fn schultz_vs_gutman_regular() {
        // For r-regular: S/Gut = 2r/(r²) = 2/r
        // S = 2r·W, Gut = r²·W → S·r = 2·Gut
        use crate::algorithms::properties::mostar_index::gutman_index;
        for g in &[k3(), k4(), cycle4(), cycle5()] {
            let r = g.degree(0).unwrap() as u64;
            let s = schultz_index(g).unwrap();
            let gut = gutman_index(g).unwrap();
            assert_eq!(s * r, 2 * gut, "r={r}");
        }
    }

    #[test]
    fn schultz_geq_wiener_times_2() {
        // S ≥ 2·W for any connected graph (since d(u)+d(v) ≥ 2 for all edges)
        for g in &[single_edge(), path3(), k3(), cycle4(), star5()] {
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut w: u64 = 0;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d != u32::MAX {
                        w += u64::from(d);
                    }
                }
            }
            assert!(schultz_index(g).unwrap() >= 2 * w);
        }
    }
}
