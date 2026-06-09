//! Hyper-Wiener index and Harary index (ALGO-TR-044).
//!
//! - **Hyper-Wiener index** `WW(G) = ½ Σ_{u<v} d(u,v) + ½ Σ_{u<v} d(u,v)²`
//!   Introduced by Randić (1993); generalises the Wiener index for
//!   acyclic graphs and is widely used in QSAR/QSPR studies.
//! - **Harary index** `H(G) = Σ_{u<v} 1/d(u,v)`
//!   The reciprocal-distance sum; measures graph compactness.
//!   Named after Frank Harary.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the hyper-Wiener index of a graph.
///
/// `WW(G) = ½ Σ_{u<v} d(u,v) + ½ Σ_{u<v} d(u,v)²`
///
/// Only finite distances contribute. Returns 0.0 for graphs with
/// fewer than 2 vertices or no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hyper_wiener_index};
///
/// // Path 0-1-2: distances 1,2,1
/// // W = 1+2+1 = 4, but u<v only: d(0,1)=1, d(0,2)=2, d(1,2)=1
/// // WW = ½(1+2+1) + ½(1+4+1) = 2 + 3 = 5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((hyper_wiener_index(&g).unwrap() - 5.0).abs() < 1e-10);
/// ```
pub fn hyper_wiener_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut sum_d = 0.0_f64;
    let mut sum_d2 = 0.0_f64;

    for u in 0..n {
        for v in (u + 1)..n {
            let d = dist[u * n + v];
            if d == u32::MAX {
                continue;
            }
            let df = f64::from(d);
            sum_d += df;
            sum_d2 += df * df;
        }
    }

    Ok(0.5 * sum_d + 0.5 * sum_d2)
}

/// Compute the Harary index of a graph.
///
/// `H(G) = Σ_{u<v} 1 / d(u, v)`
///
/// Disconnected pairs (infinite distance) are skipped.
/// Returns 0.0 for graphs with fewer than 2 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, harary_index};
///
/// // Path 0-1-2: d(0,1)=1, d(0,2)=2, d(1,2)=1
/// // H = 1/1 + 1/2 + 1/1 = 2.5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((harary_index(&g).unwrap() - 2.5).abs() < 1e-10);
/// ```
pub fn harary_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut h = 0.0_f64;

    for u in 0..n {
        for v in (u + 1)..n {
            let d = dist[u * n + v];
            if d != u32::MAX && d > 0 {
                h += 1.0 / f64::from(d);
            }
        }
    }

    Ok(h)
}

fn all_pairs_bfs(graph: &Graph, n: usize) -> Vec<u32> {
    let adj = build_adj_list(graph, n);
    let mut dist = vec![u32::MAX; n * n];
    for src in 0..n {
        bfs_distances(&adj, n, src, &mut dist);
    }
    dist
}

fn build_adj_list(graph: &Graph, n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui].push(vi);
        if !graph.is_directed() && ui != vi {
            adj[vi].push(ui);
        }
    }
    adj
}

fn bfs_distances(adj: &[Vec<usize>], n: usize, src: usize, dist: &mut [u32]) {
    dist[src * n + src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);
    while let Some(v) = queue.pop_front() {
        let d = dist[src * n + v];
        for &w in &adj[v] {
            if dist[src * n + w] == u32::MAX {
                dist[src * n + w] = d.saturating_add(1);
                queue.push_back(w);
            }
        }
    }
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

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    // --- hyper_wiener_index ---

    #[test]
    fn hww_empty() {
        let g = Graph::with_vertices(0);
        assert!((hyper_wiener_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hww_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((hyper_wiener_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hww_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((hyper_wiener_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hww_single_edge() {
        // d(0,1)=1, WW = ½·1 + ½·1 = 1
        assert!((hyper_wiener_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hww_path3() {
        // d(0,1)=1, d(0,2)=2, d(1,2)=1
        // WW = ½(1+2+1) + ½(1+4+1) = 2 + 3 = 5
        assert!((hyper_wiener_index(&path3()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn hww_path4() {
        // d(0,1)=1, d(0,2)=2, d(0,3)=3, d(1,2)=1, d(1,3)=2, d(2,3)=1
        // sum_d = 1+2+3+1+2+1 = 10
        // sum_d2 = 1+4+9+1+4+1 = 20
        // WW = ½·10 + ½·20 = 5 + 10 = 15
        assert!((hyper_wiener_index(&path4()).unwrap() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn hww_path5() {
        // distances: 1,2,3,4,1,2,3,1,2,1
        // sum_d = 1+2+3+4+1+2+3+1+2+1 = 20
        // sum_d2 = 1+4+9+16+1+4+9+1+4+1 = 50
        // WW = ½·20 + ½·50 = 10 + 25 = 35
        assert!((hyper_wiener_index(&path5()).unwrap() - 35.0).abs() < 1e-10);
    }

    #[test]
    fn hww_k3() {
        // all distances 1, 3 pairs
        // WW = ½·3 + ½·3 = 3
        assert!((hyper_wiener_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn hww_k4() {
        // all distances 1, 6 pairs
        // WW = ½·6 + ½·6 = 6
        assert!((hyper_wiener_index(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn hww_cycle4() {
        // d: 1,2,1,1,1,2 → wait, let me list u<v pairs:
        // (0,1)=1, (0,2)=2, (0,3)=1, (1,2)=1, (1,3)=2, (2,3)=1
        // sum_d = 1+2+1+1+2+1 = 8
        // sum_d2 = 1+4+1+1+4+1 = 12
        // WW = 4 + 6 = 10
        assert!((hyper_wiener_index(&cycle4()).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn hww_cycle5() {
        // (0,1)=1,(0,2)=2,(0,3)=2,(0,4)=1,(1,2)=1,(1,3)=2,(1,4)=2,(2,3)=1,(2,4)=2,(3,4)=1
        // sum_d = 1+2+2+1+1+2+2+1+2+1 = 15
        // sum_d2 = 1+4+4+1+1+4+4+1+4+1 = 25
        // WW = 7.5 + 12.5 = 20
        assert!((hyper_wiener_index(&cycle5()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn hww_star5() {
        // center 0, leaves 1-4
        // (0,i)=1 for i=1..4 (4 pairs, d=1)
        // (i,j)=2 for i<j in 1..4 (6 pairs, d=2)
        // sum_d = 4·1 + 6·2 = 16
        // sum_d2 = 4·1 + 6·4 = 28
        // WW = 8 + 14 = 22
        assert!((hyper_wiener_index(&star5()).unwrap() - 22.0).abs() < 1e-10);
    }

    #[test]
    fn hww_with_isolated() {
        // 0-1 plus isolated vertex 2: only pair (0,1) contributes
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((hyper_wiener_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hww_complete_formula() {
        // For K_n: all distances=1, C(n,2) pairs
        // WW = ½·C(n,2) + ½·C(n,2) = C(n,2)
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let pairs = f64::from(n) * f64::from(n - 1) / 2.0;
            assert!((hyper_wiener_index(&g).unwrap() - pairs).abs() < 1e-10);
        }
    }

    #[test]
    fn hww_geq_wiener() {
        // WW(G) >= W(G) for connected graphs (since d² >= d for d >= 1)
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            let ww = hyper_wiener_index(g).unwrap();
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut w = 0.0_f64;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d != u32::MAX {
                        w += f64::from(d);
                    }
                }
            }
            assert!(ww >= w, "WW={ww} < W={w}");
        }
    }

    #[test]
    fn hww_tree_formula() {
        // For trees: WW = ½ W + ½ Σ d² (simple identity check)
        for g in &[single_edge(), path3(), path4(), path5(), star5()] {
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut w = 0.0_f64;
            let mut sq = 0.0_f64;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d != u32::MAX {
                        let df = f64::from(d);
                        w += df;
                        sq += df * df;
                    }
                }
            }
            let ww = hyper_wiener_index(g).unwrap();
            assert!((ww - (0.5 * w + 0.5 * sq)).abs() < 1e-10);
        }
    }

    #[test]
    fn hww_two_components() {
        // Two edges 0-1 and 2-3: pairs (0,1) and (2,3) contribute
        // WW = 2 · (½·1 + ½·1) = 2
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!((hyper_wiener_index(&g).unwrap() - 2.0).abs() < 1e-10);
    }

    // --- harary_index ---

    #[test]
    fn hi_empty() {
        let g = Graph::with_vertices(0);
        assert!((harary_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hi_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((harary_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hi_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((harary_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn hi_single_edge() {
        // H = 1/1 = 1
        assert!((harary_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hi_path3() {
        // 1/1 + 1/2 + 1/1 = 2.5
        assert!((harary_index(&path3()).unwrap() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn hi_path4() {
        // 1/1 + 1/2 + 1/3 + 1/1 + 1/2 + 1/1 = 3 + 1 + 1/3 = 13/3
        let h = harary_index(&path4()).unwrap();
        assert!((h - 13.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn hi_k3() {
        // 3 pairs, all d=1 → H=3
        assert!((harary_index(&k3()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn hi_k4() {
        // 6 pairs, all d=1 → H=6
        assert!((harary_index(&k4()).unwrap() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn hi_cycle4() {
        // (0,1)=1,(0,2)=2,(0,3)=1,(1,2)=1,(1,3)=2,(2,3)=1
        // H = 4·(1/1) + 2·(1/2) = 4 + 1 = 5
        assert!((harary_index(&cycle4()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn hi_cycle5() {
        // 5 edges d=1, 5 pairs d=2
        // H = 5·1 + 5·0.5 = 7.5
        assert!((harary_index(&cycle5()).unwrap() - 7.5).abs() < 1e-10);
    }

    #[test]
    fn hi_star5() {
        // (0,i):d=1 → 4 pairs → 4
        // (i,j):d=2 → 6 pairs → 3
        // H = 7
        assert!((harary_index(&star5()).unwrap() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn hi_complete_formula() {
        // K_n: all d=1, C(n,2) pairs → H = C(n,2)
        for n in 2_u32..=6 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let pairs = f64::from(n) * f64::from(n - 1) / 2.0;
            assert!((harary_index(&g).unwrap() - pairs).abs() < 1e-10);
        }
    }

    #[test]
    fn hi_with_isolated() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        assert!((harary_index(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn hi_two_components() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!((harary_index(&g).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn hi_positive_for_connected() {
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            assert!(harary_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn hi_leq_pairs() {
        // H(G) <= C(n,2) since 1/d <= 1 for d >= 1
        for g in &[path3(), path4(), k3(), k4(), cycle4(), star5()] {
            let n = f64::from(g.vcount());
            let max = n * (n - 1.0) / 2.0;
            assert!(harary_index(g).unwrap() <= max + 1e-10);
        }
    }

    #[test]
    fn hi_path5() {
        // d: 1,2,3,4,1,2,3,1,2,1
        // H = 1 + 1/2 + 1/3 + 1/4 + 1 + 1/2 + 1/3 + 1 + 1/2 + 1
        //   = 4 + 3/2 + 2/3 + 1/4
        //   = 4 + 1.5 + 0.6667 + 0.25 = 6.4167
        let h = harary_index(&path5()).unwrap();
        let expected = 4.0 + 1.5 + 2.0 / 3.0 + 0.25;
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn hi_diamond() {
        // K4 minus edge (2,3): vertices 0,1,2,3
        // edges: 0-1,0-2,0-3,1-2,1-3
        // d(0,1)=1,d(0,2)=1,d(0,3)=1,d(1,2)=1,d(1,3)=1,d(2,3)=2
        // H = 5·1 + 1·0.5 = 5.5
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        assert!((harary_index(&g).unwrap() - 5.5).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn hww_equals_wiener_for_d1() {
        // When all distances are 1 (complete graph): WW = W = C(n,2)
        for n in 2_u32..=5 {
            let edges: Vec<(u32, u32)> = (0..n)
                .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
                .collect();
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let h = harary_index(&g).unwrap();
            let ww = hyper_wiener_index(&g).unwrap();
            assert!((h - ww).abs() < 1e-10);
        }
    }

    #[test]
    fn harary_geq_wiener_reciprocal() {
        // H(G) >= n(n-1)/(2W) by AM-HM inequality on distances (connected)
        for g in &[path3(), path4(), k3(), k4(), cycle4(), star5()] {
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut w = 0.0_f64;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d != u32::MAX {
                        w += f64::from(d);
                    }
                }
            }
            let pairs = n as f64 * (n as f64 - 1.0) / 2.0;
            let h = harary_index(g).unwrap();
            // AM-HM: mean(d) >= pairs/H → H >= pairs²/sum_d
            assert!(
                h >= pairs * pairs / w - 1e-10,
                "H={h} < pairs²/W={}, n={n}",
                pairs * pairs / w
            );
        }
    }
}
