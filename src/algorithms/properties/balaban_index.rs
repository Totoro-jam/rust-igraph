//! Balaban J index (ALGO-TR-043).
//!
//! The **Balaban J index** is a distance-based topological index:
//!
//! `J(G) = m / (μ + 1) · Σ_{(u,v)∈E} 1/√(σ_u · σ_v)`
//!
//! where `m` = number of edges, `μ` = cyclomatic number = `m - n + c`
//! (with `c` = number of connected components), and
//! `σ_v = Σ_w d(v, w)` is the distance sum (transmission) of vertex `v`.
//!
//! Only vertices reachable from each other contribute to the distance sum.
//! Isolated vertices have `σ = 0` and edges incident to them are skipped.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the Balaban J index of a graph.
///
/// `J(G) = m / (μ + 1) · Σ_{(u,v)∈E} 1/√(σ_u · σ_v)`
///
/// Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, balaban_j_index};
///
/// // Path 0-1-2: m=2, n=3, c=1, μ=0
/// // σ_0=3, σ_1=2, σ_2=3
/// // J = 2/1 · (1/√(3·2) + 1/√(2·3)) = 2 · 2/√6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let j = balaban_j_index(&g).unwrap();
/// assert!((j - 4.0 / 6.0_f64.sqrt()).abs() < 1e-10);
/// ```
pub fn balaban_j_index(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();
    if m == 0 || n == 0 {
        return Ok(0.0);
    }

    let (dist, sigma) = compute_distances_and_sums(graph, n);
    let components = count_components(&dist, n);

    let mu = m + components - n;
    let prefix = m as f64 / (mu as f64 + 1.0);

    let mut sum = 0.0;
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let su = sigma[ui];
        let sv = sigma[vi];
        if su > 0.0 && sv > 0.0 {
            sum += 1.0 / (su * sv).sqrt();
        }
    }

    Ok(prefix * sum)
}

fn compute_distances_and_sums(graph: &Graph, n: usize) -> (Vec<u32>, Vec<f64>) {
    let adj = build_adj_list(graph, n);
    let mut dist = vec![u32::MAX; n * n];

    for src in 0..n {
        bfs_distances(&adj, n, src, &mut dist);
    }

    let mut sigma = vec![0.0_f64; n];
    for v in 0..n {
        let mut s = 0.0_f64;
        for w in 0..n {
            let d = dist[v * n + w];
            if d != u32::MAX {
                s += f64::from(d);
            }
        }
        sigma[v] = s;
    }

    (dist, sigma)
}

fn count_components(dist: &[u32], n: usize) -> usize {
    let mut visited = vec![false; n];
    let mut components = 0_usize;
    for v in 0..n {
        if !visited[v] {
            components += 1;
            for w in 0..n {
                if dist[v * n + w] != u32::MAX {
                    visited[w] = true;
                }
            }
        }
    }
    components
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

    #[test]
    fn bj_empty() {
        let g = Graph::with_vertices(0);
        assert!((balaban_j_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn bj_no_edges() {
        let g = Graph::with_vertices(3);
        assert!((balaban_j_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn bj_single_edge() {
        // m=1, n=2, c=1, μ=0
        // σ_0=1, σ_1=1
        // J = 1/1 · 1/√(1·1) = 1
        assert!((balaban_j_index(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bj_path3() {
        // m=2, n=3, c=1, μ=0
        // σ_0=0+1+2=3, σ_1=1+0+1=2, σ_2=2+1+0=3
        // J = 2/1 · (1/√(3·2) + 1/√(2·3)) = 4/√6
        let j = balaban_j_index(&path3()).unwrap();
        assert!((j - 4.0 / 6.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn bj_k3() {
        // m=3, n=3, c=1, μ=1
        // all σ=2
        // J = 3/2 · 3·(1/√(2·2)) = 3/2 · 3/2 = 9/4 = 2.25
        let j = balaban_j_index(&k3()).unwrap();
        assert!((j - 2.25).abs() < 1e-10);
    }

    #[test]
    fn bj_cycle4() {
        // m=4, n=4, c=1, μ=1
        // σ_i = 1+2+1 = 4 for all i
        // J = 4/2 · 4·(1/√(4·4)) = 2 · 4 · 1/4 = 2
        let j = balaban_j_index(&cycle4()).unwrap();
        assert!((j - 2.0).abs() < 1e-10);
    }

    #[test]
    fn bj_star5() {
        // m=4, n=5, c=1, μ=0
        // σ_0(center)=1+1+1+1=4, σ_i(leaf)=1+2+2+2=7
        // J = 4/1 · 4·(1/√(4·7)) = 16/√28 = 16/(2√7)
        let j = balaban_j_index(&star5()).unwrap();
        let expected = 16.0 / (2.0 * 7.0_f64.sqrt());
        assert!((j - expected).abs() < 1e-10);
    }

    #[test]
    fn bj_k4() {
        // m=6, n=4, c=1, μ=3
        // all σ=3
        // J = 6/4 · 6·(1/√(3·3)) = 6/4 · 6/3 = 6/4 · 2 = 3
        let j = balaban_j_index(&k4()).unwrap();
        assert!((j - 3.0).abs() < 1e-10);
    }

    #[test]
    fn bj_path4() {
        // m=3, n=4, c=1, μ=0
        // σ_0=0+1+2+3=6, σ_1=1+0+1+2=4, σ_2=2+1+0+1=4, σ_3=3+2+1+0=6
        // J = 3/1 · (1/√(6·4) + 1/√(4·4) + 1/√(4·6))
        //   = 3 · (1/√24 + 1/4 + 1/√24)
        //   = 3 · (2/√24 + 0.25)
        let j = balaban_j_index(&path4()).unwrap();
        let expected = 3.0 * (2.0 / 24.0_f64.sqrt() + 0.25);
        assert!((j - expected).abs() < 1e-10);
    }

    #[test]
    fn bj_positive_for_connected() {
        for g in &[
            single_edge(),
            path3(),
            path4(),
            k3(),
            k4(),
            cycle4(),
            star5(),
        ] {
            assert!(balaban_j_index(g).unwrap() > 0.0);
        }
    }

    #[test]
    fn bj_with_isolated() {
        // Graph with an isolated vertex — should still work
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        // m=1, n=3, c=2, μ=0
        // σ_0=1, σ_1=1, σ_2=0
        // J = 1/1 · 1/√(1·1) = 1
        assert!((j - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bj_regular_graphs() {
        // For r-regular graph: all σ equal, so J = m/(μ+1) · m/σ
        // K3: σ=2 for all, m=3, μ=1 → J = 3/2 · 3/2 = 9/4
        let j = balaban_j_index(&k3()).unwrap();
        assert!((j - 2.25).abs() < 1e-10);
    }

    #[test]
    fn bj_single_vertex() {
        let g = Graph::with_vertices(1);
        assert!((balaban_j_index(&g).unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn bj_cycle5() {
        // m=5, n=5, c=1, μ=1
        // σ_i = 1+2+2+1 = 6 for all i
        // J = 5/2 · 5·(1/√(6·6)) = 5/2 · 5/6 = 25/12
        let g =
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)], false, Some(5)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        assert!((j - 25.0 / 12.0).abs() < 1e-10);
    }

    #[test]
    fn bj_cycle6() {
        // m=6, n=6, c=1, μ=1
        // σ_i = 1+2+3+3+2+1-3 = wait: 0+1+2+3+2+1 = 9 for all i
        // J = 6/2 · 6·(1/√(9·9)) = 3 · 6/9 = 2
        let g = Graph::from_edges(
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)],
            false,
            Some(6),
        )
        .unwrap();
        let j = balaban_j_index(&g).unwrap();
        assert!((j - 2.0).abs() < 1e-10);
    }

    #[test]
    fn bj_two_components() {
        // Two disconnected edges: 0-1 and 2-3
        // m=2, n=4, c=2, μ=0
        // σ_0=1, σ_1=1, σ_2=1, σ_3=1
        // J = 2/1 · (1/√(1·1) + 1/√(1·1)) = 2 · 2 = 4
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        assert!((j - 4.0).abs() < 1e-10);
    }

    #[test]
    fn bj_two_triangles() {
        // Two disjoint K_3: 0-1-2 and 3-4-5
        // m=6, n=6, c=2, μ=2
        // σ_i = 2 for each (within component), σ across components = MAX → ignored
        // J = 6/3 · 6·(1/√(2·2)) = 2 · 3 = 6
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)],
            false,
            Some(6),
        )
        .unwrap();
        let j = balaban_j_index(&g).unwrap();
        assert!((j - 6.0).abs() < 1e-10);
    }

    #[test]
    fn bj_path5() {
        // m=4, n=5, c=1, μ=0
        // σ_0=0+1+2+3+4=10, σ_1=1+0+1+2+3=7, σ_2=2+1+0+1+2=6
        // σ_3=3+2+1+0+1=7, σ_4=4+3+2+1+0=10
        // J = 4/1 · (1/√(10·7) + 1/√(7·6) + 1/√(6·7) + 1/√(7·10))
        //   = 4 · (2/√70 + 2/√42)
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        let expected = 4.0 * (2.0 / 70.0_f64.sqrt() + 2.0 / 42.0_f64.sqrt());
        assert!((j - expected).abs() < 1e-10);
    }

    #[test]
    fn bj_k5() {
        // m=10, n=5, c=1, μ=6
        // all σ=4 (each vertex at distance 1 from 4 others)
        // J = 10/7 · 10·(1/√(4·4)) = 10/7 · 10/4 = 100/28 = 25/7
        let edges: Vec<(u32, u32)> = (0..5_u32)
            .flat_map(|i| ((i + 1)..5).map(move |j| (i, j)))
            .collect();
        let g = Graph::from_edges(&edges, false, Some(5)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        assert!((j - 25.0 / 7.0).abs() < 1e-10);
    }

    #[test]
    fn bj_complete_bipartite_k23() {
        // K_{2,3}: parts {0,1} and {2,3,4}, every vertex in one part
        // connected to every vertex in the other.
        // m=6, n=5, c=1, μ=2
        // σ_0 = 0+1+1+1+1 = wait, distances:
        // 0→1: 2 (through 2,3,or4), 0→2: 1, 0→3: 1, 0→4: 1
        // so σ_0 = 2+1+1+1 = 5, σ_1 = 2+1+1+1 = 5
        // σ_2 = 1+1+0+2+2 = 6, σ_3 = same = 6, σ_4 = same = 6
        // wait let me recalculate: 2→3 dist = 2 (through 0 or 1)
        // σ_2 = 1+1+0+2+2 = 6
        // J = 6/3 · 6·(1/√(5·6)) = 2 · 6/√30 = 12/√30
        let g = Graph::from_edges(
            &[(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)],
            false,
            Some(5),
        )
        .unwrap();
        let j = balaban_j_index(&g).unwrap();
        let expected = 12.0 / 30.0_f64.sqrt();
        assert!((j - expected).abs() < 1e-10);
    }

    #[test]
    fn bj_monotone_with_edges() {
        // Adding edges to a path should change J, but J stays positive
        let p = path4();
        let jp = balaban_j_index(&p).unwrap();
        assert!(jp > 0.0);

        // Add edge to make it a cycle
        let c = cycle4();
        let jc = balaban_j_index(&c).unwrap();
        assert!(jc > 0.0);
        assert!((jp - jc).abs() > 1e-12);
    }

    #[test]
    fn bj_isomorphic_same_value() {
        // Two isomorphic copies of K3 with different vertex labels should give same J
        let g1 = Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap();
        let g2 = Graph::from_edges(&[(0, 2), (2, 1), (0, 1)], false, Some(3)).unwrap();
        let j1 = balaban_j_index(&g1).unwrap();
        let j2 = balaban_j_index(&g2).unwrap();
        assert!((j1 - j2).abs() < 1e-10);
    }

    #[test]
    fn bj_regular_formula() {
        // For r-regular connected graph: all σ equal, J = m/(μ+1) · m/σ
        // Verify this formula holds for several regular graphs
        for g in &[k3(), k4(), cycle4()] {
            let n = g.vcount() as usize;
            let m = g.ecount();
            let j = balaban_j_index(g).unwrap();

            let deg0 = g.degree(0).unwrap();
            let sigma0: f64 = {
                let (_, sigma) = compute_distances_and_sums(g, n);
                sigma[0]
            };
            let components = {
                let (dist, _) = compute_distances_and_sums(g, n);
                count_components(&dist, n)
            };
            let mu = m + components - n;
            let prefix = m as f64 / (mu as f64 + 1.0);
            let expected = prefix * m as f64 / sigma0;

            assert!(
                (j - expected).abs() < 1e-10,
                "Regular formula failed for {deg0}-regular graph: J={j}, expected={expected}"
            );
        }
    }

    #[test]
    fn bj_diamond() {
        // Diamond graph: K4 minus one edge, e.g., missing (2,3)
        // 0-1, 0-2, 0-3, 1-2, 1-3 (5 edges)
        // n=4, m=5, c=1, μ=2
        // σ_0 = 1+1+1 = 3, σ_1 = 1+1+1 = 3
        // σ_2 = 1+1+2 = 4, σ_3 = 1+1+2 = 4
        // prefix = 5/3
        // edges: (0,1):1/√(3·3)=1/3, (0,2):1/√(3·4)=1/√12,
        //        (0,3):1/√(3·4)=1/√12, (1,2):1/√(3·4)=1/√12,
        //        (1,3):1/√(3·4)=1/√12
        // sum = 1/3 + 4/√12 = 1/3 + 4/(2√3) = 1/3 + 2/√3
        let g =
            Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap();
        let j = balaban_j_index(&g).unwrap();
        let expected = 5.0 / 3.0 * (1.0 / 3.0 + 2.0 / 3.0_f64.sqrt());
        assert!((j - expected).abs() < 1e-10);
    }
}
