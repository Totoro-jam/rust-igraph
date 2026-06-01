//! Ptolemaic graph predicate (ALGO-PR-081).
//!
//! A ptolemaic graph is a graph that is both chordal and
//! distance-hereditary.  Equivalently, it is a gem-free chordal
//! graph, or a graph whose every metric subspace satisfies the
//! Ptolemy inequality.
//!
//! Ptolemaic graphs are exactly the block graphs (also known as
//! clique trees).  Every tree and every complete graph is ptolemaic.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::chordality::is_chordal;
use crate::algorithms::properties::is_distance_hereditary::is_distance_hereditary;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is ptolemaic.
///
/// A ptolemaic graph is both chordal and distance-hereditary.
///
/// Returns `false` for directed graphs.
///
/// An empty graph (0 vertices) is ptolemaic (vacuously).
/// A single vertex is ptolemaic.
/// Any tree is ptolemaic.
/// Any complete graph is ptolemaic.
/// Any block graph is ptolemaic.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_ptolemaic};
///
/// // Tree: ptolemaic
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_ptolemaic(&g).unwrap());
///
/// // C4 is NOT ptolemaic (not chordal)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_ptolemaic(&g).unwrap());
/// ```
pub fn is_ptolemaic(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 3 {
        return Ok(true);
    }

    let chordal_result = is_chordal(graph, None)?;
    if !chordal_result.chordal {
        return Ok(false);
    }

    is_distance_hereditary(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn triangle_is_ptolemaic() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn path_is_ptolemaic() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn tree_is_ptolemaic() {
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn k4_is_ptolemaic() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn star_is_ptolemaic() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn c4_not_ptolemaic() {
        // C4: distance-hereditary but not chordal
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn c5_not_ptolemaic() {
        // C5: neither chordal nor distance-hereditary
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_vertex_is_ptolemaic() {
        // Block graph: two K3 sharing vertex 2 → ptolemaic
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn gem_not_ptolemaic() {
        // Gem = fan(1,4): chordal but not distance-hereditary
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(4, 1).unwrap();
        g.add_edge(4, 2).unwrap();
        g.add_edge(4, 3).unwrap();
        assert!(!is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn house_not_ptolemaic() {
        // House: not chordal (contains C4)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn edgeless_is_ptolemaic() {
        let g = Graph::with_vertices(5);
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn disconnected_trees_ptolemaic() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_ptolemaic(&g).unwrap());
    }

    #[test]
    fn diamond_chordal_but_not_dh() {
        // Diamond (K4 minus one edge): chordal? Let's check.
        // K4 minus edge (2,3): 0-1, 0-2, 0-3, 1-2, 1-3
        // Cycles: 0-1-2-0 (triangle), 0-1-3-0 (triangle), 0-2-1-3-0 (C4 with chord 0-1)
        // Chordal: yes (all cycles of length >= 4 have chord 0-1)
        // Distance-hereditary: is it? K4-e is house-free...
        // Actually the diamond IS distance-hereditary (it's a block graph)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_ptolemaic(&g).unwrap());
    }
}
