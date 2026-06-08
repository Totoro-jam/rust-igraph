//! Wiener polarity index (ALGO-TR-035).
//!
//! The **Wiener polarity index** `W_p(G)` counts the number of
//! unordered vertex pairs at distance exactly 3 in an undirected graph.
//! Introduced by Wiener (1947) alongside the ordinary Wiener index.
//!
//! For directed graphs the shortest-path distance from `u` to `v`
//! (not necessarily equal to `v` to `u`) is used; a pair `(u,v)` is
//! counted if `d(u,v) = 3`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};
use std::collections::VecDeque;

/// Compute the Wiener polarity index of a graph.
///
/// Counts unordered vertex pairs `{u, v}` with shortest-path
/// distance exactly 3. For directed graphs, counts ordered pairs
/// `(u, v)` with `d(u,v) = 3`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, wiener_polarity_index};
///
/// // Path 0-1-2-3: only pair {0,3} is at distance 3
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(wiener_polarity_index(&g).unwrap(), 1);
/// ```
pub fn wiener_polarity_index(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n < 4 {
        return Ok(0);
    }

    let directed = graph.is_directed();
    let adj = build_adj_list(graph, n);

    let mut count: u64 = 0;

    for src in 0..n {
        let at3 = bfs_count_at_distance(&adj, n, src, 3);
        count = count.saturating_add(at3 as u64);
    }

    if !directed {
        count /= 2;
    }

    Ok(count)
}

/// Compute the number of vertex pairs at a given distance.
///
/// More general version: counts pairs at distance exactly `k`.
/// For undirected graphs, counts unordered pairs; for directed,
/// counts ordered pairs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_pairs_at_distance};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,4)], false, Some(5)).unwrap();
/// // Distance 2: {0,2}, {1,3}, {2,4} = 3 pairs
/// assert_eq!(count_pairs_at_distance(&g, 2).unwrap(), 3);
/// ```
pub fn count_pairs_at_distance(graph: &Graph, k: u32) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if k == 0 {
        return Ok(n as u64);
    }

    let directed = graph.is_directed();
    let adj = build_adj_list(graph, n);

    let mut count: u64 = 0;

    for src in 0..n {
        let at_k = bfs_count_at_distance(&adj, n, src, k);
        count = count.saturating_add(at_k as u64);
    }

    if !directed {
        count /= 2;
    }

    Ok(count)
}

fn build_adj_list(graph: &Graph, n: usize) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); n];
    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;
        adj[ui].push(vi);
        if !graph.is_directed() {
            adj[vi].push(ui);
        }
    }
    adj
}

fn bfs_count_at_distance(adj: &[Vec<usize>], n: usize, src: usize, target_dist: u32) -> usize {
    let mut dist = vec![u32::MAX; n];
    dist[src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);

    let mut count: usize = 0;

    while let Some(v) = queue.pop_front() {
        let d = dist[v];
        if d > target_dist {
            break;
        }
        if d == target_dist {
            count = count.saturating_add(1);
            continue;
        }
        for &w in &adj[v] {
            if dist[w] == u32::MAX {
                dist[w] = d.saturating_add(1);
                queue.push_back(w);
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

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
                (7, 9),
                (9, 6),
                (6, 8),
                (8, 5),
            ],
            false,
            Some(10),
        )
        .unwrap()
    }

    // --- wiener_polarity_index ---

    #[test]
    fn wpi_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(wiener_polarity_index(&g).unwrap(), 0);
    }

    #[test]
    fn wpi_single() {
        let g = Graph::with_vertices(1);
        assert_eq!(wiener_polarity_index(&g).unwrap(), 0);
    }

    #[test]
    fn wpi_no_edges() {
        let g = Graph::with_vertices(5);
        assert_eq!(wiener_polarity_index(&g).unwrap(), 0);
    }

    #[test]
    fn wpi_path4() {
        // 0-1-2-3: only {0,3} at distance 3
        assert_eq!(wiener_polarity_index(&path4()).unwrap(), 1);
    }

    #[test]
    fn wpi_path5() {
        // 0-1-2-3-4: pairs at distance 3: {0,3}, {1,4}
        assert_eq!(wiener_polarity_index(&path5()).unwrap(), 2);
    }

    #[test]
    fn wpi_k3() {
        // All pairs at distance 1
        assert_eq!(wiener_polarity_index(&k3()).unwrap(), 0);
    }

    #[test]
    fn wpi_k4() {
        // Complete graph: max distance = 1
        assert_eq!(wiener_polarity_index(&k4()).unwrap(), 0);
    }

    #[test]
    fn wpi_star5() {
        // Star: max distance between leaves = 2, no distance-3 pairs
        assert_eq!(wiener_polarity_index(&star5()).unwrap(), 0);
    }

    #[test]
    fn wpi_cycle5() {
        // C_5: distances are 1 or 2 only (diameter = 2)
        assert_eq!(wiener_polarity_index(&cycle5()).unwrap(), 0);
    }

    #[test]
    fn wpi_cycle6() {
        // C_6: diameter = 3; exactly 3 pairs at distance 3: {0,3},{1,4},{2,5}
        assert_eq!(wiener_polarity_index(&cycle6()).unwrap(), 3);
    }

    #[test]
    fn wpi_petersen() {
        // Petersen: diameter = 2, so no pairs at distance 3
        assert_eq!(wiener_polarity_index(&petersen()).unwrap(), 0);
    }

    #[test]
    fn wpi_two_components() {
        // Two isolated edges: 0-1, 2-3 — no pairs at distance 3
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert_eq!(wiener_polarity_index(&g).unwrap(), 0);
    }

    // --- count_pairs_at_distance ---

    #[test]
    fn cpad_zero() {
        let g = path4();
        // k=0: each vertex is at distance 0 from itself → 4
        assert_eq!(count_pairs_at_distance(&g, 0).unwrap(), 4);
    }

    #[test]
    fn cpad_one_path4() {
        // k=1: edges = 3
        assert_eq!(count_pairs_at_distance(&path4(), 1).unwrap(), 3);
    }

    #[test]
    fn cpad_two_path5() {
        // 0-1-2-3-4: pairs at distance 2: {0,2},{1,3},{2,4} = 3
        assert_eq!(count_pairs_at_distance(&path5(), 2).unwrap(), 3);
    }

    #[test]
    fn cpad_three_path5() {
        // {0,3},{1,4} = 2
        assert_eq!(count_pairs_at_distance(&path5(), 3).unwrap(), 2);
    }

    #[test]
    fn cpad_four_path5() {
        // {0,4} = 1
        assert_eq!(count_pairs_at_distance(&path5(), 4).unwrap(), 1);
    }

    #[test]
    fn cpad_five_path5() {
        // No pairs at distance 5 in a 5-vertex path
        assert_eq!(count_pairs_at_distance(&path5(), 5).unwrap(), 0);
    }

    #[test]
    fn cpad_one_k4() {
        // K_4: 6 edges, all at distance 1
        assert_eq!(count_pairs_at_distance(&k4(), 1).unwrap(), 6);
    }

    #[test]
    fn cpad_two_cycle6() {
        // C_6: distance-2 pairs: 6
        assert_eq!(count_pairs_at_distance(&cycle6(), 2).unwrap(), 6);
    }

    #[test]
    fn cpad_matches_wpi() {
        // count_pairs_at_distance(g, 3) should equal wiener_polarity_index(g)
        for g in &[path4(), path5(), k3(), k4(), cycle5(), cycle6(), star5()] {
            let wpi = wiener_polarity_index(g).unwrap();
            let cpad = count_pairs_at_distance(g, 3).unwrap();
            assert_eq!(wpi, cpad);
        }
    }

    #[test]
    fn cpad_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(count_pairs_at_distance(&g, 1).unwrap(), 0);
    }

    // --- sum of all distances ---

    #[test]
    fn sum_distances_path4() {
        // Wiener index of P_4: 1+2+3 + 1+2 + 1 = 10
        let g = path4();
        let mut total: u64 = 0;
        for k in 1..4 {
            total = total.saturating_add(count_pairs_at_distance(&g, k).unwrap() * u64::from(k));
        }
        assert_eq!(total, 10);
    }
}
