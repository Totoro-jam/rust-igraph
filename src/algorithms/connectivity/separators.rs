//! Vertex separators (ALGO-CN-015).
//!
//! Counterpart of `igraph_is_separator()` and
//! `igraph_is_minimal_separator()` from
//! `references/igraph/src/connectivity/separators.c`.
//!
//! A *vertex separator* of a connected graph is a set of vertices
//! whose removal disconnects the graph (or isolates a vertex from
//! the rest). A separator is *minimal* if no proper subset of it is
//! also a separator.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Check whether a set of vertices is a separator of the graph.
///
/// A vertex set S is a separator if removing S (and all incident edges)
/// makes the remaining graph disconnected, OR if removing S leaves
/// fewer vertices than the original graph minus |S| (i.e., some vertex
/// becomes isolated). For a graph that is already disconnected, any
/// set is technically a separator — this function returns `true` for
/// the empty set in that case.
///
/// For undirected graphs only.
///
/// # Errors
///
/// - `InvalidArgument` if the graph is directed.
/// - `InvalidArgument` if any vertex ID in `candidates` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_separator};
///
/// // Path 0-1-2: removing vertex 1 disconnects the graph.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_separator(&g, &[1]).unwrap());
/// assert!(!is_separator(&g, &[0]).unwrap()); // leaf removal doesn't disconnect
/// ```
pub fn is_separator(graph: &Graph, candidates: &[VertexId]) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "is_separator: only defined for undirected graphs".into(),
        ));
    }

    let n = graph.vcount();
    for &v in candidates {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "is_separator: vertex {v} out of range (vcount={n})"
            )));
        }
    }

    if n == 0 {
        return Ok(false);
    }

    // Mark candidates in a set for O(1) lookup.
    let n_us = n as usize;
    let mut removed = vec![false; n_us];
    for &v in candidates {
        removed[v as usize] = true;
    }

    // Count remaining vertices (those not in the removed set).
    let remaining = (0..n_us).filter(|&v| !removed[v]).count();

    if remaining == 0 {
        return Ok(false);
    }

    // BFS from the first non-removed vertex. If it can't reach all
    // remaining vertices, the graph is disconnected → separator.
    let start = (0..n_us).find(|&v| !removed[v]).unwrap();
    #[allow(clippy::cast_possible_truncation)] // start < n which is u32
    let reached = bfs_count(graph, start as u32, &removed)?;

    Ok(reached < remaining)
}

/// Check whether a set of vertices is a *minimal* separator.
///
/// A separator is minimal if no proper subset is also a separator.
/// Equivalently, S is a minimal separator if it is a separator and
/// for every vertex v in S, removing S \ {v} does NOT disconnect the
/// graph.
///
/// # Errors
///
/// - `InvalidArgument` if the graph is directed.
/// - `InvalidArgument` if any vertex ID is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_minimal_separator};
///
/// // 4-cycle: {1,3} is a minimal separator (removing both disconnects 0 from 2).
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(is_minimal_separator(&g, &[1, 3]).unwrap());
/// // {0,1,3} leaves only vertex 2 — not a separator, hence not minimal.
/// assert!(!is_minimal_separator(&g, &[0, 1, 3]).unwrap());
/// ```
pub fn is_minimal_separator(graph: &Graph, candidates: &[VertexId]) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "is_minimal_separator: only defined for undirected graphs".into(),
        ));
    }

    let n = graph.vcount();
    for &v in candidates {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "is_minimal_separator: vertex {v} out of range (vcount={n})"
            )));
        }
    }

    // First check: is it a separator at all?
    if !is_separator(graph, candidates)? {
        return Ok(false);
    }

    // For each vertex in the candidate set, check if removing it
    // still leaves a separator. If yes for any vertex, it's not minimal.
    let n_us = n as usize;
    for (idx, _) in candidates.iter().enumerate() {
        // Build the "removed" set without vertex v.
        let mut removed = vec![false; n_us];
        for (j, &u) in candidates.iter().enumerate() {
            if j != idx {
                removed[u as usize] = true;
            }
        }

        let remaining = (0..n_us).filter(|&x| !removed[x]).count();
        if remaining == 0 {
            continue;
        }

        let start = (0..n_us).find(|&x| !removed[x]).unwrap();
        #[allow(clippy::cast_possible_truncation)] // start < n which is u32
        let reached = bfs_count(graph, start as u32, &removed)?;

        if reached < remaining {
            // S \ {v} is still a separator → S is not minimal.
            return Ok(false);
        }
    }

    Ok(true)
}

/// BFS from `start`, skipping removed vertices. Returns count of reachable vertices.
fn bfs_count(graph: &Graph, start: u32, removed: &[bool]) -> IgraphResult<usize> {
    let n_us = graph.vcount() as usize;
    let mut visited = vec![false; n_us];
    let mut queue = VecDeque::new();
    let mut count = 0usize;

    visited[start as usize] = true;
    queue.push_back(start);
    count += 1;

    while let Some(cur) = queue.pop_front() {
        let neighbors = graph.neighbors(cur)?;
        for &nb in &neighbors {
            let nidx = nb as usize;
            if !visited[nidx] && !removed[nidx] {
                visited[nidx] = true;
                queue.push_back(nb);
                count += 1;
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_separator(&g, &[]).unwrap());
    }

    #[test]
    fn singleton_not_separator() {
        let g = Graph::with_vertices(1);
        assert!(!is_separator(&g, &[0]).unwrap());
    }

    #[test]
    fn path_middle_is_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_separator(&g, &[1]).unwrap());
    }

    #[test]
    fn path_leaf_not_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn triangle_no_single_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // No single vertex disconnects a triangle.
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[1]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn triangle_pair_not_separator() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Removing two vertices leaves a single vertex — trivially connected.
        assert!(!is_separator(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn cycle_4_opposite_vertices() {
        // 0-1-2-3-0. {1,3} separates 0 from 2.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn cycle_4_adjacent_not_separator() {
        // 0-1-2-3-0. {0,1} does NOT disconnect (2-3 is still connected).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_separator(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn already_disconnected_empty_set_is_separator() {
        // Two components: {0,1}, {2,3}. Empty set "separates".
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_separator(&g, &[]).unwrap());
    }

    #[test]
    fn k4_articulation() {
        // K4 minus one edge: 0-1, 0-2, 0-3, 1-2, 2-3 (missing 1-3).
        // Vertex 2 is not an articulation point because 0 connects to all others.
        // Actually: adjacencies: 0→{1,2,3}, 1→{0,2}, 2→{0,1,3}, 3→{0,2}
        // Remove 0 → remaining {1,2,3}: 1-2, 2-3 → connected. Not separator.
        // Remove 2 → remaining {0,1,3}: 0-1, 0-3 → connected. Not separator.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_separator(&g, &[0]).unwrap());
        assert!(!is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn bowtie_articulation() {
        // Two triangles sharing vertex 2: {0,1,2} and {2,3,4}.
        // Vertex 2 is an articulation point → separator.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_separator(&g, &[2]).unwrap());
    }

    #[test]
    fn directed_rejected() {
        let g = Graph::new(3, true).unwrap();
        assert!(is_separator(&g, &[0]).is_err());
        assert!(is_minimal_separator(&g, &[0]).is_err());
    }

    #[test]
    fn out_of_range_rejected() {
        let g = Graph::with_vertices(3);
        assert!(is_separator(&g, &[5]).is_err());
    }

    // --- Minimal separator tests ---

    #[test]
    fn minimal_path_middle() {
        // Path 0-1-2: {1} is a minimal separator.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_minimal_separator(&g, &[1]).unwrap());
    }

    #[test]
    fn minimal_cycle_4_opposite() {
        // 0-1-2-3-0: {1,3} is a minimal separator.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_minimal_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn not_minimal_superset() {
        // Path 0-1-2-3-4: {1,3} is a separator but NOT minimal ({1} alone suffices).
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_separator(&g, &[1, 3]).unwrap());
        assert!(!is_minimal_separator(&g, &[1, 3]).unwrap());
    }

    #[test]
    fn not_separator_not_minimal() {
        // Triangle: {0} is not a separator → not a minimal separator.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_minimal_separator(&g, &[0]).unwrap());
    }

    #[test]
    fn minimal_bowtie_articulation() {
        // Bowtie: {2} is a minimal separator.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_minimal_separator(&g, &[2]).unwrap());
    }
}
