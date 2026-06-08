//! Gromov δ-hyperbolicity (ALGO-TR-039).
//!
//! The **Gromov δ-hyperbolicity** of a graph measures how tree-like its
//! metric structure is. For each 4-tuple of vertices `(u, v, w, x)`,
//! form the three sums of pairwise distances:
//!
//!   `S1 = d(u,v) + d(w,x)`,  `S2 = d(u,w) + d(v,x)`,  `S3 = d(u,x) + d(v,w)`
//!
//! Sort these so that `S_max >= S_mid >= S_min`. The four-point
//! condition gives `δ_4 = (S_max - S_mid) / 2`. The hyperbolicity
//! is `δ = max δ_4` over all 4-tuples.
//!
//! A tree has `δ = 0`. Cycles `C_n` have `δ = ⌊n/4⌋` (for `n >= 4`).
//! The value is always a non-negative half-integer, so we return
//! `2δ` as a `u32` to avoid floating point.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::Graph;
use std::collections::VecDeque;

/// Compute the Gromov δ-hyperbolicity of a graph.
///
/// Returns `2δ` as an integer (since δ is always a half-integer).
/// To get the actual δ, divide by 2. Trees have `2δ = 0`.
///
/// Only considers the largest connected component if the graph is
/// disconnected. Only feasible for small graphs — the brute-force
/// algorithm is O(n^4).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hyperbolicity_twice};
///
/// // Tree: δ = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert_eq!(hyperbolicity_twice(&g).unwrap(), 0);
///
/// // C_4: δ = 1, so 2δ = 2
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// assert_eq!(hyperbolicity_twice(&g).unwrap(), 2);
/// ```
pub fn hyperbolicity_twice(graph: &Graph) -> Result<u32, crate::core::IgraphError> {
    let n = graph.vcount() as usize;
    if n < 4 {
        return Ok(0);
    }

    let dist = all_pairs_bfs(graph, n);

    let mut max_twice_delta: u32 = 0;

    for u in 0..n {
        for v in (u + 1)..n {
            if dist[u * n + v] == u32::MAX {
                continue;
            }
            for w in (v + 1)..n {
                if dist[u * n + w] == u32::MAX || dist[v * n + w] == u32::MAX {
                    continue;
                }
                for x in (w + 1)..n {
                    if dist[u * n + x] == u32::MAX
                        || dist[v * n + x] == u32::MAX
                        || dist[w * n + x] == u32::MAX
                    {
                        continue;
                    }

                    let s1 = dist[u * n + v].saturating_add(dist[w * n + x]);
                    let s2 = dist[u * n + w].saturating_add(dist[v * n + x]);
                    let s3 = dist[u * n + x].saturating_add(dist[v * n + w]);

                    let mut sums = [s1, s2, s3];
                    sums.sort_unstable();

                    let twice_delta = sums[2].saturating_sub(sums[1]);
                    if twice_delta > max_twice_delta {
                        max_twice_delta = twice_delta;
                    }
                }
            }
        }
    }

    Ok(max_twice_delta)
}

/// Compute δ-hyperbolicity as a floating-point value.
///
/// Convenience wrapper around [`hyperbolicity_twice`] that returns δ
/// as `f64`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hyperbolicity};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// assert!((hyperbolicity(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn hyperbolicity(graph: &Graph) -> Result<f64, crate::core::IgraphError> {
    let twice = hyperbolicity_twice(graph)?;
    Ok(f64::from(twice) / 2.0)
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
        if !graph.is_directed() {
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

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn path5() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)], false, Some(5)).unwrap()
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

    fn cycle8() -> Graph {
        Graph::from_edges(
            &[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 0),
            ],
            false,
            Some(8),
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

    // --- hyperbolicity_twice ---

    #[test]
    fn ht_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(hyperbolicity_twice(&g).unwrap(), 0);
    }

    #[test]
    fn ht_small() {
        let g = Graph::with_vertices(3);
        assert_eq!(hyperbolicity_twice(&g).unwrap(), 0);
    }

    #[test]
    fn ht_path4() {
        // Trees: δ = 0
        assert_eq!(hyperbolicity_twice(&path4()).unwrap(), 0);
    }

    #[test]
    fn ht_path5() {
        assert_eq!(hyperbolicity_twice(&path5()).unwrap(), 0);
    }

    #[test]
    fn ht_star5() {
        // Stars are trees: δ = 0
        assert_eq!(hyperbolicity_twice(&star5()).unwrap(), 0);
    }

    #[test]
    fn ht_k4() {
        // Complete graph K_4: all distances = 1
        // S1 = S2 = S3 = 2, so δ = 0
        assert_eq!(hyperbolicity_twice(&k4()).unwrap(), 0);
    }

    #[test]
    fn ht_cycle4() {
        // C_4: δ = 1, 2δ = 2
        assert_eq!(hyperbolicity_twice(&cycle4()).unwrap(), 2);
    }

    #[test]
    fn ht_cycle5() {
        // C_5: δ = 0.5, 2δ = 1
        let ht = hyperbolicity_twice(&cycle5()).unwrap();
        assert_eq!(ht, 1);
    }

    #[test]
    fn ht_cycle6() {
        // C_6: δ = 1, 2δ = 2
        let ht = hyperbolicity_twice(&cycle6()).unwrap();
        assert_eq!(ht, 2);
    }

    #[test]
    fn ht_cycle8() {
        // C_8: δ = 2, 2δ = 4
        assert_eq!(hyperbolicity_twice(&cycle8()).unwrap(), 4);
    }

    #[test]
    fn ht_petersen() {
        // Petersen: diameter = 2, δ = 0.5, 2δ = 1
        let ht = hyperbolicity_twice(&petersen()).unwrap();
        assert_eq!(ht, 1);
    }

    // --- hyperbolicity ---

    #[test]
    fn h_path() {
        let h = hyperbolicity(&path4()).unwrap();
        assert!((h - 0.0).abs() < 1e-10);
    }

    #[test]
    fn h_cycle4() {
        let h = hyperbolicity(&cycle4()).unwrap();
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn h_cycle8() {
        let h = hyperbolicity(&cycle8()).unwrap();
        assert!((h - 2.0).abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn tree_hyperbolicity_zero() {
        let trees = vec![
            path4(),
            path5(),
            star5(),
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (2, 4)], false, Some(5)).unwrap(),
        ];
        for g in &trees {
            assert_eq!(hyperbolicity_twice(g).unwrap(), 0);
        }
    }

    #[test]
    fn hyperbolicity_non_negative() {
        for g in &[path4(), k4(), cycle4(), cycle5(), star5(), petersen()] {
            assert!(hyperbolicity(g).unwrap() >= 0.0);
        }
    }

    #[test]
    fn hyperbolicity_leq_half_diameter() {
        // δ <= diameter / 2
        for g in &[cycle4(), cycle5(), cycle6(), cycle8(), petersen()] {
            let h = hyperbolicity(g).unwrap();
            let diam = g.diameter().unwrap().unwrap_or(0);
            assert!(
                h <= f64::from(diam) / 2.0 + 1e-10,
                "δ={h} > diameter/2={}",
                f64::from(diam) / 2.0
            );
        }
    }
}
