//! Mostar index and degree-distance index (ALGO-TR-042).
//!
//! - **Mostar index** `Mo(G) = Σ_{(u,v)∈E} |n_u(e) - n_v(e)|`
//!   where `n_u(e)` = vertices strictly closer to `u` than `v`.
//!   Measures peripherality — trees maximise it among graphs with
//!   the same order and size; `Mo = 0` iff the graph is
//!   distance-balanced.
//! - **Degree-distance index** `DD(G) = Σ_{u≠v} (deg(u)+deg(v))·d(u,v)`
//!   (Schultz-type invariant; equals the Schultz index for trees).
//! - **Gutman index** `Gut(G) = Σ_{u<v} deg(u)·deg(v)·d(u,v)`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::comparison_chain,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the Mostar index of a graph.
///
/// `Mo(G) = Σ_{(u,v)∈E} |n_u(e) - n_v(e)|`
///
/// A graph is distance-balanced iff `Mo = 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mostar_index};
///
/// // Cycle C_4 is distance-balanced → Mo = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// assert_eq!(mostar_index(&g).unwrap(), 0);
///
/// // Path 0-1-2: edge(0,1) |1-2|=1, edge(1,2) |2-1|=1 → Mo = 2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(mostar_index(&g).unwrap(), 2);
/// ```
pub fn mostar_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut mo: u64 = 0;

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        if ui == vi {
            continue;
        }
        let (nu, nv) = count_closer(&dist, n, ui, vi);
        mo = mo.saturating_add(nu.abs_diff(nv) as u64);
    }

    Ok(mo)
}

/// Compute the degree-distance index (Schultz-type).
///
/// `DD(G) = Σ_{u≠v} (deg(u) + deg(v)) · d(u, v)`
///
/// For connected graphs only; disconnected pairs are skipped.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_distance};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // pairs: (0,1) d=1 (1+2)=3, (0,2) d=2 (1+1)=2·2=4,
/// //        (1,0) d=1 3, (1,2) d=1 3, (2,0) d=2 4, (2,1) d=1 3
/// // DD = 3+4+3+3+4+3 = 20
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(degree_distance(&g).unwrap(), 20);
/// ```
pub fn degree_distance(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut deg = vec![0_usize; n];
    for v in 0..n as u32 {
        deg[v as usize] = graph.degree(v)?;
    }

    let mut dd: u64 = 0;
    for u in 0..n {
        for v in 0..n {
            if u == v {
                continue;
            }
            let d = dist[u * n + v];
            if d == u32::MAX {
                continue;
            }
            let sum_deg = (deg[u] as u64).saturating_add(deg[v] as u64);
            dd = dd.saturating_add(sum_deg.saturating_mul(u64::from(d)));
        }
    }

    Ok(dd)
}

/// Compute the Gutman index.
///
/// `Gut(G) = Σ_{u<v} deg(u) · deg(v) · d(u, v)`
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, gutman_index};
///
/// // Path 0-1-2: degrees [1,2,1]
/// // (0,1): 1·2·1=2, (0,2): 1·1·2=2, (1,2): 2·1·1=2
/// // Gut = 6
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert_eq!(gutman_index(&g).unwrap(), 6);
/// ```
pub fn gutman_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);
    let mut deg = vec![0_usize; n];
    for v in 0..n as u32 {
        deg[v as usize] = graph.degree(v)?;
    }

    let mut gut: u64 = 0;
    for u in 0..n {
        for v in (u + 1)..n {
            let d = dist[u * n + v];
            if d == u32::MAX {
                continue;
            }
            let prod = (deg[u] as u64).saturating_mul(deg[v] as u64);
            gut = gut.saturating_add(prod.saturating_mul(u64::from(d)));
        }
    }

    Ok(gut)
}

fn count_closer(dist: &[u32], n: usize, u: usize, v: usize) -> (usize, usize) {
    let mut nu = 0_usize;
    let mut nv = 0_usize;
    for w in 0..n {
        let du = dist[u * n + w];
        let dv = dist[v * n + w];
        if du == u32::MAX || dv == u32::MAX {
            continue;
        }
        if du < dv {
            nu += 1;
        } else if dv < du {
            nv += 1;
        }
    }
    (nu, nv)
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

    // --- mostar_index ---

    #[test]
    fn mo_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(mostar_index(&g).unwrap(), 0);
    }

    #[test]
    fn mo_no_edges() {
        let g = Graph::with_vertices(3);
        assert_eq!(mostar_index(&g).unwrap(), 0);
    }

    #[test]
    fn mo_single_edge() {
        // n_u=1, n_v=1 → |1-1| = 0
        assert_eq!(mostar_index(&single_edge()).unwrap(), 0);
    }

    #[test]
    fn mo_path3() {
        // edge(0,1): nu=1, nv=2 → |1-2|=1
        // edge(1,2): nu=2, nv=1 → |2-1|=1
        // Mo = 2
        assert_eq!(mostar_index(&path3()).unwrap(), 2);
    }

    #[test]
    fn mo_path4() {
        // edge(0,1): nu=1, nv=3 → 2
        // edge(1,2): nu=2, nv=2 → 0
        // edge(2,3): nu=3, nv=1 → 2
        // Mo = 4
        assert_eq!(mostar_index(&path4()).unwrap(), 4);
    }

    #[test]
    fn mo_k3() {
        // Each edge: nu=1, nv=1 → 0
        // Mo = 0 (K_n is distance-balanced)
        assert_eq!(mostar_index(&k3()).unwrap(), 0);
    }

    #[test]
    fn mo_cycle4() {
        // C4: each edge nu=2, nv=2 → 0 (cycles are distance-balanced)
        assert_eq!(mostar_index(&cycle4()).unwrap(), 0);
    }

    #[test]
    fn mo_star5() {
        // edge(0,i): nu=4, nv=1 → 3; 4 edges → Mo = 12
        assert_eq!(mostar_index(&star5()).unwrap(), 12);
    }

    #[test]
    fn mo_regular_distance_balanced() {
        // Complete graphs and cycles are distance-balanced
        assert_eq!(mostar_index(&k4()).unwrap(), 0);
        assert_eq!(mostar_index(&cycle4()).unwrap(), 0);
    }

    // --- degree_distance ---

    #[test]
    fn dd_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(degree_distance(&g).unwrap(), 0);
    }

    #[test]
    fn dd_single_edge() {
        // (0,1) and (1,0): (1+1)·1 = 2, twice → 4
        assert_eq!(degree_distance(&single_edge()).unwrap(), 4);
    }

    #[test]
    fn dd_path3() {
        // degrees [1,2,1]
        // (0,1)+(1,0): 2·(1+2)·1 = 6
        // (0,2)+(2,0): 2·(1+1)·2 = 8
        // (1,2)+(2,1): 2·(2+1)·1 = 6
        // DD = 20
        assert_eq!(degree_distance(&path3()).unwrap(), 20);
    }

    #[test]
    fn dd_k3() {
        // degrees all 2, all distances 1
        // 6 ordered pairs, each (2+2)·1 = 4 → 24
        assert_eq!(degree_distance(&k3()).unwrap(), 24);
    }

    #[test]
    fn dd_k4() {
        // degrees all 3, all distances 1
        // 12 ordered pairs, each (3+3)·1 = 6 → 72
        assert_eq!(degree_distance(&k4()).unwrap(), 72);
    }

    #[test]
    fn dd_star5() {
        // degrees [4,1,1,1,1]
        // center-leaf pairs: 4·(4+1)·1 = 20 (×2 for both directions = 40)
        // leaf-leaf pairs: d=2, (1+1)·2=4; 12 ordered pairs → 48
        // DD = 40 + 48 = 88
        assert_eq!(degree_distance(&star5()).unwrap(), 88);
    }

    // --- gutman_index ---

    #[test]
    fn gut_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(gutman_index(&g).unwrap(), 0);
    }

    #[test]
    fn gut_single_edge() {
        // (0,1): 1·1·1 = 1
        assert_eq!(gutman_index(&single_edge()).unwrap(), 1);
    }

    #[test]
    fn gut_path3() {
        // (0,1): 1·2·1=2, (0,2): 1·1·2=2, (1,2): 2·1·1=2 → 6
        assert_eq!(gutman_index(&path3()).unwrap(), 6);
    }

    #[test]
    fn gut_k3() {
        // 3 pairs, each 2·2·1=4 → 12
        assert_eq!(gutman_index(&k3()).unwrap(), 12);
    }

    #[test]
    fn gut_k4() {
        // 6 pairs, each 3·3·1=9 → 54
        assert_eq!(gutman_index(&k4()).unwrap(), 54);
    }

    #[test]
    fn gut_star5() {
        // center-leaf: 4 pairs, 4·1·1=4 each → 16
        // leaf-leaf: 6 pairs, 1·1·2=2 each → 12
        // Gut = 28
        assert_eq!(gutman_index(&star5()).unwrap(), 28);
    }

    // --- cross-consistency ---

    #[test]
    fn dd_is_twice_sum_unordered() {
        // DD sums over ordered pairs = 2 × sum over unordered pairs
        for g in &[path3(), k3(), k4(), star5()] {
            let dd = degree_distance(g).unwrap();
            let n = g.vcount() as usize;
            let dist = all_pairs_bfs(g, n);
            let mut deg = vec![0_usize; n];
            for v in 0..n as u32 {
                deg[v as usize] = g.degree(v).unwrap();
            }
            let mut half: u64 = 0;
            for u in 0..n {
                for v in (u + 1)..n {
                    let d = dist[u * n + v];
                    if d == u32::MAX {
                        continue;
                    }
                    half += (deg[u] as u64 + deg[v] as u64) * u64::from(d);
                }
            }
            assert_eq!(dd, 2 * half);
        }
    }

    #[test]
    fn gutman_leq_dd_div2_times_max_deg() {
        // Gut(G) <= DD(G)/2 · Δ(G) for simple graphs (loose bound check)
        for g in &[path3(), k3(), k4(), star5()] {
            let gut = gutman_index(g).unwrap();
            let dd = degree_distance(g).unwrap();
            let max_d = u64::from(
                crate::algorithms::properties::degree::max_degree(
                    g,
                    crate::algorithms::properties::degree::DegreeMode::All,
                )
                .unwrap(),
            );
            assert!(
                gut <= dd / 2 * max_d + max_d,
                "Gutman {gut} too large relative to DD {dd}"
            );
        }
    }
}
