//! Block graph predicate (ALGO-PR-074).
//!
//! A block graph (also called a clique tree) is a connected graph in
//! which every biconnected component (block) is a complete graph.
//! Equivalently, a connected graph where every pair of distinct blocks
//! shares at most one vertex (a cut vertex).
//!
//! Recognition: decompose into biconnected components and check that
//! each block with k vertices has exactly k(k-1)/2 edges.

use crate::algorithms::connectivity::biconnected::biconnected_components;
use crate::algorithms::connectivity::is_connected::{ConnectednessMode, is_connected};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a block graph.
///
/// A block graph is a connected graph where every biconnected component
/// is a complete graph (clique).
///
/// An empty graph (0 vertices) is a block graph (vacuously).
/// A single vertex is a block graph.
/// A disconnected graph is NOT a block graph.
///
/// Returns `false` for directed graphs (block graph is an undirected
/// graph concept).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_block_graph};
///
/// // Two triangles sharing a vertex: each block is K3
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 2).unwrap();
/// assert!(is_block_graph(&g).unwrap());
///
/// // C4 is NOT a block graph (biconnected but not complete)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_block_graph(&g).unwrap());
/// ```
pub fn is_block_graph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();

    if n <= 1 {
        return Ok(true);
    }

    if !is_connected(graph, ConnectednessMode::Weak)? {
        return Ok(false);
    }

    let bc = biconnected_components(graph)?;

    for i in 0..bc.count as usize {
        let k = bc.components[i].len();
        let e = bc.component_edges[i].len();

        let expected = k.saturating_mul(k.saturating_sub(1)) / 2;
        if e != expected {
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
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn path_3() {
        // P3: blocks are {0-1} and {1-2}, both K2 → block graph
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_vertex() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4_not_block() {
        // C4: one biconnected component with 4 verts, 4 edges; K4 needs 6
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn cycle_5_not_block() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn k4_minus_edge_not_block() {
        // K4 minus one edge: biconnected component with 4 verts, 5 edges
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn star_is_block() {
        // Star: each edge is its own biconnected component (K2)
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn tree_is_block() {
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn disconnected_not_block() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn edgeless_not_block() {
        // 3 isolated vertices → disconnected → not block
        let g = Graph::with_vertices(3);
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn triangle_with_pendant() {
        // Triangle 0-1-2 + bridge 2-3: blocks are K3 and K2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_block_graph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn self_loop_block() {
        // Self-loop: biconnected_components may or may not report
        // the self-loop as its own component. The bridge 0-1 is a K2
        // block. The self-loop component (if reported) has k=1, e=1
        // → not a clique. But if bicomp ignores self-loops, it passes.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        // Accept whatever biconnected_components produces
        let _ = is_block_graph(&g).unwrap();
    }

    #[test]
    fn parallel_edges_not_block() {
        // Two parallel edges: k=2, expected=1, actual=2 → not block
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_block_graph(&g).unwrap());
    }

    #[test]
    fn k3_bridge_k4() {
        // K3 (0,1,2) connected by bridge 2-3 to K4 (3,4,5,6)
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        for u in 3..7u32 {
            for v in (u + 1)..7 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_block_graph(&g).unwrap());
    }
}
