//! Cactus graph predicate (ALGO-PR-071).
//!
//! A cactus graph (also called a Husimi tree) is a connected graph in
//! which every biconnected component is either a single edge or a
//! simple cycle. Equivalently, every edge belongs to at most one
//! simple cycle.

use crate::algorithms::connectivity::biconnected::biconnected_components;
use crate::algorithms::connectivity::is_connected::{ConnectednessMode, is_connected};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a cactus graph.
///
/// A cactus graph is a connected graph where every biconnected
/// component is either a single edge (bridge) or a simple cycle.
///
/// An empty graph (0 vertices) is considered a cactus (vacuously).
/// A single vertex with no edges is a cactus.
/// A graph that is not connected is NOT a cactus.
///
/// For directed graphs, connectivity is checked as weak connectivity
/// and edges are treated as undirected for the biconnected component
/// decomposition.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_cactus_graph};
///
/// // Triangle is a cactus (one cycle component)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(is_cactus_graph(&g).unwrap());
///
/// // K4 is NOT a cactus (biconnected component has too many edges)
/// let mut g = Graph::with_vertices(4);
/// for u in 0..4u32 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// assert!(!is_cactus_graph(&g).unwrap());
/// ```
pub fn is_cactus_graph(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();

    if n <= 1 {
        return Ok(true);
    }

    if !is_connected(graph, ConnectednessMode::Weak)? {
        return Ok(false);
    }

    let bc = biconnected_components(graph)?;

    for i in 0..bc.count as usize {
        let n_verts = bc.components[i].len();
        let n_edges = bc.component_edges[i].len();

        if n_edges == 1 {
            continue;
        }

        if n_edges != n_verts {
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
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn path_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn two_cycles_sharing_vertex() {
        // Two triangles sharing vertex 2: 0-1-2-0 and 2-3-4-2
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn cycle_with_bridge() {
        // Triangle 0-1-2-0 with bridge 2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn k4_not_cactus() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn disconnected_not_cactus() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4_with_diagonal_not_cactus() {
        // C4 + diagonal: forms K4 minus one edge, which has a
        // biconnected component with 4 vertices and 5 edges → not cactus
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(!is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn star_is_cactus() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn tree_is_cactus() {
        // Any tree is a cactus
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn self_loop_cactus() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn directed_cycle_cactus() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn parallel_edges_not_cactus() {
        // Two parallel edges between same vertices: biconnected
        // component has 2 vertices, 2 edges → that's a 2-cycle, ok
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        // Actually: 2 vertices, 2 edges = a cycle of length 2 → cactus
        assert!(is_cactus_graph(&g).unwrap());
    }

    #[test]
    fn triple_parallel_not_cactus() {
        // Three parallel edges: 2 verts, 3 edges → not a cycle
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_cactus_graph(&g).unwrap());
    }
}
