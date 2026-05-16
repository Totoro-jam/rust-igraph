//! `is_biconnected` (ALGO-CC-013).
//!
//! Counterpart of `igraph_is_biconnected()` from
//! `references/igraph/src/connectivity/components.c:1254-1379`.
//!
//! A graph is **biconnected** if removing any single vertex (and its
//! incident edges) does not disconnect it. Per upstream's
//! documentation:
//!
//! - The null graph (n = 0) and singleton graph (n = 1) are NOT
//!   biconnected.
//! - The two-vertex graph with at least one connecting edge IS
//!   biconnected (igraph's convention).
//! - For n ≥ 3, biconnectedness ≡ weakly connected ∧ no articulation
//!   points.
//!
//! Phase-1 minimal slice delegates to the existing
//! [`connected_components`] and [`articulation_points`] primitives
//! for n ≥ 3. Upstream's bespoke single-DFS-with-early-exit (faster
//! on large graphs that aren't biconnected) ships in a future perf pass.

use crate::core::{Graph, IgraphResult};

/// Check whether `graph` is biconnected.
///
/// On directed graphs the input is treated as undirected (matches
/// upstream `IGRAPH_ALL` mode default at `components.c:1292`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_biconnected};
///
/// // Triangle: biconnected.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_biconnected(&g).unwrap());
///
/// // Path 0-1-2: vertex 1 is an articulation point → not biconnected.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!is_biconnected(&g).unwrap());
/// ```
pub fn is_biconnected(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    // Null graph + singleton are NOT biconnected (upstream
    // components.c:1263-1268).
    if n < 2 {
        return Ok(false);
    }
    // Two-vertex special case: biconnected iff there's at least one
    // edge between them. Self-loops alone don't count (upstream uses
    // IGRAPH_NO_LOOPS in its lazy adjlist).
    if n == 2 {
        let nbrs = graph.neighbors(0)?;
        return Ok(nbrs.contains(&1));
    }
    // n ≥ 3: must be weakly connected AND have no articulation points.
    let cc = super::components::connected_components(graph)?;
    if cc.count != 1 {
        return Ok(false);
    }
    let aps = super::articulation::articulation_points(graph)?;
    Ok(aps.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_not_biconnected() {
        let g = Graph::with_vertices(0);
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn singleton_is_not_biconnected() {
        let g = Graph::with_vertices(1);
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn two_vertices_no_edge_is_not_biconnected() {
        let g = Graph::with_vertices(2);
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn two_vertices_one_edge_is_biconnected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_biconnected(&g).unwrap());
    }

    #[test]
    fn triangle_is_biconnected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_biconnected(&g).unwrap());
    }

    #[test]
    fn path_3_is_not_biconnected() {
        // Vertex 1 is an articulation point.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn cycle_4_is_biconnected() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        assert!(is_biconnected(&g).unwrap());
    }

    #[test]
    fn k4_complete_is_biconnected() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_biconnected(&g).unwrap());
    }

    #[test]
    fn disconnected_graph_is_not_biconnected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn star_is_not_biconnected() {
        // Centre is articulation.
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn cycle_with_pendant_is_not_biconnected() {
        // Triangle 0-1-2-0 + pendant 2-3: vertex 2 is articulation.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_biconnected(&g).unwrap());
    }

    #[test]
    fn isolated_extra_vertex_breaks_biconnected() {
        // Triangle plus an isolated vertex 3: not connected → not biconnected.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_biconnected(&g).unwrap());
    }
}
