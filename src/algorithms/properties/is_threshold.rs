//! Threshold graph predicate (ALGO-PR-073).
//!
//! A threshold graph can be constructed from a single vertex by
//! repeatedly adding either an isolated vertex (connected to no
//! existing vertex) or a dominating vertex (connected to all existing
//! vertices). Equivalently, a graph is threshold iff it has no
//! induced subgraph isomorphic to P4, C4, or 2K2.
//!
//! Recognition is O(V log V) via the degree sequence: sort degrees in
//! non-increasing order, then repeatedly "peel" vertices. A vertex of
//! degree n-1 (dominating) is removed and all remaining degrees are
//! decremented; a vertex of degree 0 (isolated) is simply removed.
//! If at any step the largest degree is neither 0 nor n-1, the graph
//! is not threshold.
//!
//! This characterization applies to simple undirected graphs. For
//! non-simple graphs or directed graphs, the function returns `false`.

use crate::algorithms::properties::degree::{DegreeMode, degree_sequence};
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a threshold graph.
///
/// A threshold graph is built from a single vertex by repeatedly
/// adding isolated or dominating vertices. Uses a degree-sequence
/// peeling algorithm for O(V log V) recognition.
///
/// Returns `false` for directed graphs, multigraphs, or graphs with
/// self-loops (the characterization requires simple undirected graphs).
///
/// An empty graph (0 vertices) is a threshold graph (vacuously).
/// An edgeless graph is a threshold graph (all vertices are isolated).
/// A complete graph is a threshold graph (all vertices are dominating).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_threshold_graph};
///
/// // Star K_{1,3}: built as isolated, then dominating
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert!(is_threshold_graph(&g).unwrap());
///
/// // Path P4 (0-1-2-3) is NOT threshold
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(!is_threshold_graph(&g).unwrap());
/// ```
pub fn is_threshold_graph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(true);
    }

    if !is_simple(graph)? {
        return Ok(false);
    }

    let mut degrees = degree_sequence(graph, DegreeMode::All)?;
    degrees.sort_unstable_by(|a, b| b.cmp(a));

    let mut remaining = n;
    let mut degs = &mut degrees[..];

    while remaining > 0 {
        let top = degs[0];
        let last_idx = remaining.saturating_sub(1);

        if top == 0 {
            return Ok(true);
        }

        let top_usize = top as usize;
        if top_usize == last_idx {
            // Dominating vertex: remove it and decrement top degrees
            degs = &mut degs[1..];
            remaining = remaining.saturating_sub(1);
            for d in &mut degs[..top_usize.min(remaining)] {
                *d = d.saturating_sub(1);
            }
            degs.sort_unstable_by(|a, b| b.cmp(a));
        } else if degs[last_idx] == 0 {
            // Isolated vertex: remove it from the end
            remaining = remaining.saturating_sub(1);
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn edgeless_graph() {
        let g = Graph::with_vertices(5);
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn complete_k3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn star_is_threshold() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn path_3_is_threshold() {
        // P3 = 0-1-2: degrees [1,2,1]. Peel: top=2, n-1=2, yes dominating.
        // Remove, decrement → [0,0]. All zero → threshold.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn path_4_not_threshold() {
        // P4 = 0-1-2-3: degrees [1,2,2,1].
        // Forbidden subgraph P4 itself.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4_not_threshold() {
        // C4 contains P4 and 2K2 as induced subgraphs
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn cycle_5_not_threshold() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn two_k2_not_threshold() {
        // 2K2: two disjoint edges — forbidden induced subgraph
        // degrees [1,1,1,1], top=1, n-1=3 → not dominating, bottom=1≠0
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn threshold_build_sequence() {
        // Build: start {0}, add 1 dominating (→ K2),
        // add 2 isolated, add 3 dominating (→ edges 3-0, 3-1, 3-2)
        // degrees: [2,1,1,3] → sorted [3,2,1,1]
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 1).unwrap();
        g.add_edge(3, 2).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn k4_minus_edge_is_threshold() {
        // K4 minus edge (2,3): build as {2}, add 3 isolated,
        // add 0 dominating, add 1 dominating → threshold
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn self_loop_returns_false() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn parallel_edges_returns_false() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn large_star_threshold() {
        let mut g = Graph::with_vertices(10);
        for i in 1..10u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn bipartite_k23_not_threshold() {
        // K_{2,3}: degrees [3,3,2,2,2] — not threshold
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_threshold_graph(&g).unwrap());
    }

    #[test]
    fn k_plus_isolated() {
        // K3 + 2 isolated vertices → threshold
        // (built: 3 isolated, then add 3,4 as dominating to existing)
        // Actually: K3 is threshold, adding isolated vertices keeps it threshold
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_threshold_graph(&g).unwrap());
    }
}
