//! Local scan statistics (ALGO-PR-051).
//!
//! Counterpart of `igraph_local_scan_1_ecount()` from
//! `references/igraph/src/misc/scan.c:224`.
//!
//! For each vertex, counts edges (or sums edge weights) within its
//! closed 1-neighborhood (the vertex and all its neighbors).

use crate::core::{Graph, IgraphError, IgraphResult};

/// For each vertex, count edges within its closed 1-neighborhood.
///
/// The closed 1-neighborhood of vertex `v` is `{v} ∪ neighbors(v)`.
/// This function counts all edges that have both endpoints in that set.
///
/// For undirected graphs, each such edge is counted once per vertex whose
/// neighborhood contains it.
///
/// `weights`: optional edge weights (length must equal `ecount()`).
/// When provided, sums edge weights instead of counting edges.
///
/// Returns a vector of length `vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, local_scan_1};
///
/// // Triangle: each vertex's 1-neighborhood is the whole graph.
/// // 3 edges in the neighborhood of each vertex.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let s = local_scan_1(&g, None).unwrap();
/// assert!((s[0] - 3.0).abs() < 1e-10);
/// assert!((s[1] - 3.0).abs() < 1e-10);
/// assert!((s[2] - 3.0).abs() < 1e-10);
/// ```
pub fn local_scan_1(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "local_scan_1: weights length ({}) does not match edge count ({ecount})",
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

    // For each vertex, mark its closed neighborhood, then count edges
    // with both endpoints in the set.
    let mut marked = vec![0u32; n_usize];

    for v in 0..n {
        let v_us = v as usize;
        let tag = v + 1;

        // Mark v and all its neighbors.
        marked[v_us] = tag;
        for &nei in &adj[v_us] {
            marked[nei as usize] = tag;
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

        result[v_us] = count;
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
        let s = local_scan_1(&g, None).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let s = local_scan_1(&g, None).unwrap();
        assert!(s.iter().all(|&v| close(v, 0.0)));
    }

    #[test]
    fn triangle() {
        // Each vertex sees all 3 edges.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 3.0));
    }

    #[test]
    fn path_5() {
        // 0-1-2-3-4.
        // N[0] = {0,1}: edges {0-1} → 1
        // N[1] = {0,1,2}: edges {0-1, 1-2} → 2
        // N[2] = {1,2,3}: edges {1-2, 2-3} → 2
        // N[3] = {2,3,4}: edges {2-3, 3-4} → 2
        // N[4] = {3,4}: edges {3-4} → 1
        let mut g = Graph::with_vertices(5);
        for i in 0..4u32 {
            g.add_edge(i, i + 1).unwrap();
        }
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 1.0));
        assert!(close(s[1], 2.0));
        assert!(close(s[2], 2.0));
        assert!(close(s[3], 2.0));
        assert!(close(s[4], 1.0));
    }

    #[test]
    fn star() {
        // 0 connected to 1,2,3. N[0] = all. N[1] = {0,1}.
        // Edges: 0-1, 0-2, 0-3.
        // N[0] = {0,1,2,3}: all 3 edges → 3
        // N[1] = {0,1}: edges {0-1} → 1
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 1.0));
        assert!(close(s[2], 1.0));
        assert!(close(s[3], 1.0));
    }

    #[test]
    fn k4() {
        // Complete graph K4: every vertex's 1-neighborhood is the whole graph.
        // 6 edges total. Each vertex sees all 6.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let s = local_scan_1(&g, None).unwrap();
        for &val in &s {
            assert!(close(val, 6.0));
        }
    }

    #[test]
    fn two_triangles_with_bridge() {
        // {0,1,2} form triangle, {3,4,5} form triangle, bridge 2-3.
        // N[0] = {0,1,2}: edges {0-1,0-2,1-2} → 3
        // N[2] = {0,1,2,3}: edges {0-1,0-2,1-2,2-3} → 4
        // N[3] = {2,3,4,5}: edges {2-3,3-4,3-5,4-5} → 4
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(2, 3).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 3.0));
        assert!(close(s[1], 3.0));
        assert!(close(s[2], 4.0));
        assert!(close(s[3], 4.0));
        assert!(close(s[4], 3.0));
        assert!(close(s[5], 3.0));
    }

    #[test]
    fn weighted() {
        // Triangle with weights [2, 3, 5].
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let w = vec![2.0, 3.0, 5.0];
        let s = local_scan_1(&g, Some(&w)).unwrap();
        // Each vertex sees all 3 edges → sum = 10.
        assert!(close(s[0], 10.0));
        assert!(close(s[1], 10.0));
        assert!(close(s[2], 10.0));
    }

    #[test]
    fn self_loop() {
        // 0-0, 0-1. N[0] = {0,1}: edges {0-0, 0-1} → 2. N[1] = {0,1}: same → 2.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = local_scan_1(&g, None).unwrap();
        assert!(close(s[0], 2.0));
        assert!(close(s[1], 2.0));
    }

    #[test]
    fn weights_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(local_scan_1(&g, Some(&[1.0, 2.0])).is_err());
    }
}
