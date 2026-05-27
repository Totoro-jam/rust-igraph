//! Generalized local scan-k edge count (ALGO-PR-053).
//!
//! Counterpart of `igraph_local_scan_k_ecount()` from
//! `references/igraph/src/misc/scan.c:450`.
//!
//! For each vertex, counts edges (or sums edge weights) within its
//! closed k-neighborhood (all vertices reachable within k steps).

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult};

/// For each vertex, count edges within its closed k-neighborhood.
///
/// The closed k-neighborhood of vertex `v` is the set of all vertices
/// reachable from `v` in at most `k` hops. This function counts all
/// edges that have both endpoints in that set.
///
/// For undirected graphs, each edge is counted once.
/// For directed graphs with `IGRAPH_ALL` mode equivalent, edges are
/// counted once per direction observed.
///
/// `k`: radius of the neighborhood (must be ≥ 0). k=0 counts only
/// self-loops at each vertex. k=1 is equivalent to `local_scan_1`.
///
/// `weights`: optional edge weights (length must equal `ecount()`).
/// When provided, sums edge weights instead of counting edges.
///
/// Returns a vector of length `vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_k};
///
/// // Triangle: k=1 neighborhood of each vertex is the whole graph.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let s = local_scan_k(&g, 1, None).unwrap();
/// assert!((s[0] - 3.0).abs() < 1e-10);
///
/// // Path 0-1-2-3-4 with k=2:
/// // N_2[0] = {0,1,2}, edges {0-1,1-2} → 2
/// // N_2[2] = {0,1,2,3,4}, all 4 edges → 4
/// let mut g = Graph::with_vertices(5);
/// for i in 0..4u32 { g.add_edge(i, i+1).unwrap(); }
/// let s = local_scan_k(&g, 2, None).unwrap();
/// assert!((s[0] - 2.0).abs() < 1e-10);
/// assert!((s[2] - 4.0).abs() < 1e-10);
/// ```
pub fn local_scan_k(graph: &Graph, k: u32, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "local_scan_k: weights length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    let n_usize = n as usize;
    let mut result = vec![0.0_f64; n_usize];

    if n == 0 || ecount == 0 {
        return Ok(result);
    }

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;

    // Build adjacency lists.
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n_usize];
    for eid in 0..m_u32 {
        let (from, to) = graph.edge(eid)?;
        adj[from as usize].push(to);
        if !graph.is_directed() && from != to {
            adj[to as usize].push(from);
        }
    }

    // For each vertex, BFS to distance k, mark all reachable vertices,
    // then count edges with both endpoints marked.
    let mut marked = vec![0u32; n_usize];
    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();

    for v in 0..n {
        let tag = v + 1;
        marked[v as usize] = tag;
        queue.clear();
        queue.push_back((v, 0));

        while let Some((cur, dist)) = queue.pop_front() {
            if dist < k {
                for &nei in &adj[cur as usize] {
                    if marked[nei as usize] != tag {
                        marked[nei as usize] = tag;
                        queue.push_back((nei, dist + 1));
                    }
                }
            }
        }

        // Count edges with both endpoints marked.
        let mut count = 0.0_f64;
        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            if marked[from as usize] == tag && marked[to as usize] == tag {
                let w = weights.map_or(1.0, |ws| ws[eid as usize]);
                count += w;
            }
        }

        result[v as usize] = count;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let s = local_scan_k(&g, 1, None).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let s = local_scan_k(&g, 2, None).unwrap();
        assert!(s.iter().all(|&v| close(v, 0.0)));
    }

    #[test]
    fn k_zero_no_self_loops() {
        // k=0: only the vertex itself is in the neighborhood.
        // No self-loops → no edges in neighborhood.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = local_scan_k(&g, 0, None).unwrap();
        assert!(s.iter().all(|&v| close(v, 0.0)));
    }

    #[test]
    fn k_zero_with_self_loop() {
        // k=0: only the vertex itself. Self-loop on 0.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = local_scan_k(&g, 0, None).unwrap();
        assert!(close(s[0], 1.0)); // self-loop
        assert!(close(s[1], 0.0)); // no self-loop on 1
    }

    #[test]
    fn k_one_triangle() {
        // Same as local_scan_1: each vertex sees all 3 edges.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let s = local_scan_k(&g, 1, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 3.0));
    }

    #[test]
    fn k_one_path_5() {
        // Same as local_scan_1.
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = local_scan_k(&g, 1, None).unwrap();
        assert!(close(s[0], 1.0));
        assert!(close(s[1], 2.0));
        assert!(close(s[2], 2.0));
        assert!(close(s[3], 2.0));
        assert!(close(s[4], 1.0));
    }

    #[test]
    fn k_two_path_5() {
        // N_2[0] = {0,1,2}: edges {0-1, 1-2} → 2
        // N_2[1] = {0,1,2,3}: edges {0-1, 1-2, 2-3} → 3
        // N_2[2] = {0,1,2,3,4}: all 4 edges → 4
        // N_2[3] = {1,2,3,4}: edges {1-2, 2-3, 3-4} → 3
        // N_2[4] = {2,3,4}: edges {2-3, 3-4} → 2
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = local_scan_k(&g, 2, None).unwrap();
        assert!(close(s[0], 2.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 4.0));
        assert!(close(s[3], 3.0));
        assert!(close(s[4], 2.0));
    }

    #[test]
    fn k_large_returns_total_edges() {
        // k large enough to cover entire graph → every vertex sees all edges.
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = local_scan_k(&g, 100, None).unwrap();
        for &val in &s {
            assert!(close(val, 4.0));
        }
    }

    #[test]
    fn two_components() {
        // {0,1,2} triangle + {3,4} edge.
        // N_1[0] = {0,1,2}: 3 edges.
        // N_1[3] = {3,4}: 1 edge.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        let s = local_scan_k(&g, 1, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[3], 1.0));
        assert!(close(s[4], 1.0));
    }

    #[test]
    fn weighted() {
        // Triangle with weights [2, 3, 5], k=1.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let w = vec![2.0, 3.0, 5.0];
        let s = local_scan_k(&g, 1, Some(&w)).unwrap();
        // Each vertex sees all 3 edges → sum = 10.
        assert!(close(s[0], 10.0));
        assert!(close(s[1], 10.0));
        assert!(close(s[2], 10.0));
    }

    #[test]
    fn star_k2() {
        // Star: 0 connected to 1,2,3. k=2: N_2[1] = {0,1,2,3} (all).
        // All 3 edges in that set → 3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let s = local_scan_k(&g, 2, None).unwrap();
        // N_2[0] = all → 3, N_2[1] = all → 3
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 3.0));
        assert!(close(s[3], 3.0));
    }

    #[test]
    fn weights_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(local_scan_k(&g, 1, Some(&[1.0, 2.0])).is_err());
    }
}
