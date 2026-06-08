//! Graph matching metrics (ALGO-TR-030).
//!
//! A **matching** is a set of edges with no shared endpoints.
//!
//! - **Greedy maximal matching**: iteratively pick edges greedily.
//! - **Maximum matching** (brute-force): the largest matching.
//!   NP-easy (polynomial via Edmonds' algorithm) but we use brute-force
//!   for simplicity — suitable for small graphs (≤ ~25 vertices).
//! - **Matching number**: `ν(G) = |maximum matching|`.
//! - **Is perfect matching**: whether the matching covers all vertices.
//! - **Edge independence number** (same as matching number).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Find a greedy maximal matching.
///
/// Iterates over edges and greedily adds each edge whose endpoints are
/// both unmatched. Returns edge indices of the matching.
///
/// The result is maximal (no edge can be added) but not necessarily
/// maximum.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_matching};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let m = greedy_matching(&g).unwrap();
/// assert!(!m.is_empty());
/// ```
pub fn greedy_matching(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let n = graph.vcount() as usize;
    let mut matched = vec![false; n];
    let mut matching = Vec::new();

    for (eidx, (u, v)) in graph.edges().enumerate() {
        let ui = u as usize;
        let vi = v as usize;
        if !matched[ui] && !matched[vi] {
            matching.push(eidx);
            matched[ui] = true;
            matched[vi] = true;
        }
    }

    Ok(matching)
}

/// Find the maximum matching (brute-force).
///
/// Returns edge indices of the largest matching. Only feasible for
/// small graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximum_matching};
///
/// // Path 0-1-2-3: max matching has 2 edges
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let m = maximum_matching(&g).unwrap();
/// assert_eq!(m.len(), 2);
/// ```
pub fn maximum_matching(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let m = edges.len();
    let n = graph.vcount() as usize;

    if m == 0 || n < 2 {
        return Ok(Vec::new());
    }

    let mut best: Vec<usize> = Vec::new();
    max_matching_branch(
        &edges,
        n,
        0,
        &mut Vec::new(),
        &mut vec![false; n],
        &mut best,
    );

    Ok(best)
}

fn max_matching_branch(
    edges: &[(u32, u32)],
    n: usize,
    start: usize,
    current: &mut Vec<usize>,
    matched: &mut Vec<bool>,
    best: &mut Vec<usize>,
) {
    if current.len() > best.len() {
        *best = current.clone();
    }

    let remaining_possible = (edges.len() - start).min(n / 2);
    if current.len() + remaining_possible <= best.len() {
        return;
    }

    for i in start..edges.len() {
        let (u, v) = edges[i];
        let ui = u as usize;
        let vi = v as usize;
        if !matched[ui] && !matched[vi] {
            matched[ui] = true;
            matched[vi] = true;
            current.push(i);
            max_matching_branch(edges, n, i + 1, current, matched, best);
            current.pop();
            matched[ui] = false;
            matched[vi] = false;
        }
    }
}

/// Compute the matching number `ν(G)`.
///
/// The size of the maximum matching. Also called the edge
/// independence number.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, matching_number};
///
/// // K_4: perfect matching exists, ν = 2
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert_eq!(matching_number(&g).unwrap(), 2);
/// ```
pub fn matching_number(graph: &Graph) -> IgraphResult<u32> {
    let m = maximum_matching(graph)?;
    Ok(m.len() as u32)
}

/// Check whether a matching is valid.
///
/// A valid matching has no two edges sharing an endpoint.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, greedy_matching, is_valid_matching};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let m = greedy_matching(&g).unwrap();
/// assert!(is_valid_matching(&g, &m).unwrap());
/// ```
pub fn is_valid_matching(graph: &Graph, edge_indices: &[usize]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    let edges: Vec<(u32, u32)> = graph.edges().collect();
    let m = edges.len();

    let mut used = vec![false; n];

    for &eidx in edge_indices {
        if eidx >= m {
            return Err(IgraphError::InvalidArgument(format!(
                "is_valid_matching: edge index {eidx} out of range (ecount={m})"
            )));
        }
        let (u, v) = edges[eidx];
        let ui = u as usize;
        let vi = v as usize;
        if used[ui] || used[vi] {
            return Ok(false);
        }
        used[ui] = true;
        used[vi] = true;
    }

    Ok(true)
}

/// Check whether a matching is a perfect matching.
///
/// A perfect matching covers every vertex exactly once. Only possible
/// when the graph has an even number of vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximum_matching, is_perfect_matching};
///
/// // K_4 has a perfect matching
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let m = maximum_matching(&g).unwrap();
/// assert!(is_perfect_matching(&g, &m).unwrap());
/// ```
pub fn is_perfect_matching(graph: &Graph, edge_indices: &[usize]) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if n % 2 != 0 {
        return Ok(false);
    }
    if edge_indices.len() != n / 2 {
        return Ok(false);
    }
    is_valid_matching(graph, edge_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- greedy_matching ---

    #[test]
    fn gm_empty() {
        let g = Graph::with_vertices(0);
        assert!(greedy_matching(&g).unwrap().is_empty());
    }

    #[test]
    fn gm_single() {
        let g = Graph::with_vertices(1);
        assert!(greedy_matching(&g).unwrap().is_empty());
    }

    #[test]
    fn gm_path4() {
        let g = path4();
        let m = greedy_matching(&g).unwrap();
        assert!(is_valid_matching(&g, &m).unwrap());
        assert!(!m.is_empty());
    }

    #[test]
    fn gm_k4() {
        let g = k4();
        let m = greedy_matching(&g).unwrap();
        assert!(is_valid_matching(&g, &m).unwrap());
    }

    // --- maximum_matching ---

    #[test]
    fn mm_empty() {
        let g = Graph::with_vertices(0);
        assert!(maximum_matching(&g).unwrap().is_empty());
    }

    #[test]
    fn mm_single_edge() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn mm_path4() {
        let g = path4();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 2);
        assert!(is_valid_matching(&g, &m).unwrap());
    }

    #[test]
    fn mm_k3() {
        let g = k3();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn mm_k4() {
        let g = k4();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 2);
        assert!(is_perfect_matching(&g, &m).unwrap());
    }

    #[test]
    fn mm_cycle4() {
        let g = cycle4();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 2);
        assert!(is_perfect_matching(&g, &m).unwrap());
    }

    #[test]
    fn mm_cycle5() {
        let g = cycle5();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn mm_star5() {
        let g = star5();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn mm_petersen() {
        let g = petersen();
        let m = maximum_matching(&g).unwrap();
        assert_eq!(m.len(), 5);
        assert!(is_perfect_matching(&g, &m).unwrap());
    }

    #[test]
    fn mm_isolated() {
        let g = Graph::with_vertices(4);
        let m = maximum_matching(&g).unwrap();
        assert!(m.is_empty());
    }

    // --- matching_number ---

    #[test]
    fn mn_k4() {
        assert_eq!(matching_number(&k4()).unwrap(), 2);
    }

    #[test]
    fn mn_path4() {
        assert_eq!(matching_number(&path4()).unwrap(), 2);
    }

    #[test]
    fn mn_star5() {
        assert_eq!(matching_number(&star5()).unwrap(), 1);
    }

    // --- is_valid_matching ---

    #[test]
    fn ivm_valid() {
        let g = path4();
        assert!(is_valid_matching(&g, &[0, 2]).unwrap());
    }

    #[test]
    fn ivm_invalid() {
        let g = path4();
        assert!(!is_valid_matching(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn ivm_empty() {
        let g = path4();
        assert!(is_valid_matching(&g, &[]).unwrap());
    }

    #[test]
    fn ivm_out_of_range() {
        let g = path4();
        assert!(is_valid_matching(&g, &[10]).is_err());
    }

    // --- is_perfect_matching ---

    #[test]
    fn ipm_k4_perfect() {
        let g = k4();
        let m = maximum_matching(&g).unwrap();
        assert!(is_perfect_matching(&g, &m).unwrap());
    }

    #[test]
    fn ipm_odd_vertices() {
        let g = k3();
        let m = maximum_matching(&g).unwrap();
        assert!(!is_perfect_matching(&g, &m).unwrap());
    }

    #[test]
    fn ipm_not_enough_edges() {
        let g = k4();
        assert!(!is_perfect_matching(&g, &[0]).unwrap());
    }

    // --- cross-consistency ---

    #[test]
    fn greedy_is_valid() {
        for g in &[path4(), k3(), k4(), cycle4(), cycle5(), star5(), petersen()] {
            let m = greedy_matching(g).unwrap();
            assert!(is_valid_matching(g, &m).unwrap());
        }
    }

    #[test]
    fn greedy_at_most_maximum() {
        for g in &[path4(), k3(), k4(), cycle5(), star5()] {
            let gm = greedy_matching(g).unwrap();
            let mm = maximum_matching(g).unwrap();
            assert!(gm.len() <= mm.len());
        }
    }

    #[test]
    fn matching_number_bounded() {
        for g in &[path4(), k3(), k4(), cycle5(), star5()] {
            let nu = matching_number(g).unwrap();
            let n = g.vcount();
            assert!(nu <= n / 2);
        }
    }

    #[test]
    fn gallai_identity() {
        // Gallai's theorem: α + ν = n (for graphs without isolated vertices
        // where α is vertex cover number = n - independence number)
        // Actually: matching_number + vertex_cover_number = n
        // And: independence_number + vertex_cover_number = n
        // So: independence_number + matching_number = n only for König graphs
        // (bipartite). Let's test König's theorem for bipartite graphs.
        let g = path4(); // bipartite
        let alpha = crate::algorithms::cliques::independence_number(&g).unwrap();
        let nu = matching_number(&g).unwrap();
        let n = g.vcount();
        assert_eq!(alpha + nu, n);
    }
}
