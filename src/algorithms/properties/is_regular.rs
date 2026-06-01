//! Regularity check (ALGO-PR-070).
//!
//! A graph is *k-regular* if every vertex has the same degree *k*.
//! For directed graphs, a graph is regular if every vertex has the
//! same out-degree AND the same in-degree (not necessarily equal to
//! each other).

use crate::algorithms::properties::degree::{DegreeMode, degree_sequence};
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is regular.
///
/// A graph is regular if every vertex has the same degree. For
/// directed graphs, this checks that all out-degrees are equal AND
/// all in-degrees are equal.
///
/// An empty graph (0 vertices) is considered regular.
/// A graph with one vertex is regular (degree may be nonzero due to
/// self-loops).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_regular};
///
/// // Cycle C4 is 2-regular
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(is_regular(&g).unwrap());
///
/// // Path P3 is not regular (endpoints have degree 1, middle has 2)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!is_regular(&g).unwrap());
/// ```
pub fn is_regular(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n <= 1 {
        return Ok(true);
    }

    let out_deg = degree_sequence(graph, DegreeMode::Out)?;
    if out_deg.windows(2).any(|w| w[0] != w[1]) {
        return Ok(false);
    }

    if graph.is_directed() {
        let in_deg = degree_sequence(graph, DegreeMode::In)?;
        if in_deg.windows(2).any(|w| w[0] != w[1]) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Return the regularity degree of the graph, or `None` if the graph
/// is not regular.
///
/// For undirected graphs, returns the common degree. For directed
/// graphs, returns the common out-degree (which equals the common
/// in-degree in a regular directed graph where out-degree == in-degree
/// for all vertices; if out-degrees are all equal and in-degrees are
/// all equal but differ from each other, returns `None` since such a
/// graph is not considered regular in the standard sense for simple
/// regularity queries).
///
/// An empty graph returns `None` (no vertices, no degree).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, regularity};
///
/// // K3 is 2-regular
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(regularity(&g).unwrap(), Some(2));
///
/// // Star graph is not regular
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert_eq!(regularity(&g).unwrap(), None);
/// ```
pub fn regularity(graph: &Graph) -> IgraphResult<Option<u32>> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(None);
    }

    let out_deg = degree_sequence(graph, DegreeMode::Out)?;
    if out_deg.windows(2).any(|w| w[0] != w[1]) {
        return Ok(None);
    }

    if graph.is_directed() {
        let in_deg = degree_sequence(graph, DegreeMode::In)?;
        if in_deg.windows(2).any(|w| w[0] != w[1]) {
            return Ok(None);
        }
        if out_deg[0] != in_deg[0] {
            return Ok(None);
        }
    }

    Ok(Some(out_deg[0]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_regular() {
        let g = Graph::with_vertices(0);
        assert!(is_regular(&g).unwrap());
    }

    #[test]
    fn empty_graph_regularity_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(regularity(&g).unwrap(), None);
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::with_vertices(1);
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(0));
    }

    #[test]
    fn single_vertex_self_loop() {
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        assert!(is_regular(&g).unwrap());
    }

    #[test]
    fn two_vertices_no_edges() {
        let g = Graph::with_vertices(2);
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(0));
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(1));
    }

    #[test]
    fn path_3_not_regular() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), None);
    }

    #[test]
    fn triangle_is_2_regular() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(2));
    }

    #[test]
    fn cycle_4_is_2_regular() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(2));
    }

    #[test]
    fn complete_k4_is_3_regular() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(3));
    }

    #[test]
    fn star_not_regular() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(!is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), None);
    }

    #[test]
    fn petersen_is_3_regular() {
        let mut g = Graph::with_vertices(10);
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (5, 7),
            (7, 9),
            (9, 6),
            (6, 8),
            (8, 5),
            (0, 5),
            (1, 6),
            (2, 7),
            (3, 8),
            (4, 9),
        ];
        for (u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(3));
    }

    #[test]
    fn directed_cycle_is_regular() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(1));
    }

    #[test]
    fn directed_unequal_in_out_not_regular() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), None);
    }

    #[test]
    fn parallel_edges_regular() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(2));
    }

    #[test]
    fn bipartite_k22_is_2_regular() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(is_regular(&g).unwrap());
        assert_eq!(regularity(&g).unwrap(), Some(2));
    }
}
