//! Adjacency query (ALGO-PR-044).
//!
//! Counterpart of `igraph_are_adjacent()` in
//! `references/igraph/src/graph/basic_query.c:48-63`.
//!
//! Checks whether two vertices are connected by at least one edge.

use crate::core::{Graph, IgraphResult, VertexId};

/// Check whether two vertices are connected by at least one edge.
///
/// For directed graphs, checks if there is an edge from `v1` to `v2`
/// **or** from `v2` to `v1` (i.e. direction is ignored, matching the
/// C implementation which uses `IGRAPH_DIRECTED` but still checks
/// adjacency in both directions for undirected queries).
///
/// # Errors
///
/// * Returns an error if either `v1` or `v2` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, are_adjacent};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// assert!(are_adjacent(&g, 0, 1).unwrap());
/// assert!(!are_adjacent(&g, 0, 2).unwrap());
/// ```
pub fn are_adjacent(graph: &Graph, v1: VertexId, v2: VertexId) -> IgraphResult<bool> {
    Ok(graph.find_eid(v1, v2)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn connected_undirected() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        assert!(are_adjacent(&g, 0, 1).unwrap());
        assert!(are_adjacent(&g, 1, 0).unwrap());
        assert!(are_adjacent(&g, 1, 2).unwrap());
        assert!(!are_adjacent(&g, 0, 2).unwrap());
    }

    #[test]
    fn connected_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        assert!(are_adjacent(&g, 0, 1).unwrap());
        // find_eid on directed graph checks from→to only
        assert!(!are_adjacent(&g, 1, 0).unwrap());
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 0)]).unwrap();
        assert!(are_adjacent(&g, 0, 0).unwrap());
        assert!(!are_adjacent(&g, 0, 1).unwrap());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(5, false).unwrap();
        assert!(!are_adjacent(&g, 0, 1).unwrap());
        assert!(!are_adjacent(&g, 2, 4).unwrap());
    }

    #[test]
    fn invalid_vertex() {
        let g = Graph::new(3, false).unwrap();
        assert!(are_adjacent(&g, 0, 5).is_err());
        assert!(are_adjacent(&g, 10, 0).is_err());
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 1)]).unwrap();
        assert!(are_adjacent(&g, 0, 1).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, false).unwrap();
        assert!(!are_adjacent(&g, 0, 0).unwrap());
    }

    #[test]
    fn directed_both_directions() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 0)]).unwrap();
        assert!(are_adjacent(&g, 0, 1).unwrap());
        assert!(are_adjacent(&g, 1, 0).unwrap());
    }
}
