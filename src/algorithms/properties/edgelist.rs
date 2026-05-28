//! Edge list extraction (ALGO-PR-042).
//!
//! Counterpart of `igraph_get_edgelist()` in
//! `references/igraph/src/misc/conversion.c:327-329`.
//!
//! Returns all edges of a graph in edge-ID order.

use crate::core::{Graph, IgraphResult, VertexId};

/// Return all edges of the graph as a list of `(source, target)` pairs.
///
/// Edges are returned in edge-ID order (0, 1, 2, …).
///
/// # Examples
///
/// ```
/// use rust_igraph::{create, get_edgelist};
///
/// let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).unwrap();
/// let el = get_edgelist(&g).unwrap();
/// assert_eq!(el.len(), 3);
/// ```
pub fn get_edgelist(graph: &Graph) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let ecount = graph.ecount();
    let mut result = Vec::with_capacity(ecount);
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let e = eid as u32;
        result.push(graph.edge(e)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert!(el.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(5, false).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert!(el.is_empty());
    }

    #[test]
    fn triangle_undirected() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert_eq!(el.len(), 3);
    }

    #[test]
    fn directed_edges_preserved() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert_eq!(el.len(), 2);
        assert_eq!(el[0], (0, 1));
        assert_eq!(el[1], (1, 2));
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = Graph::new(1, false).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert!(el.is_empty());
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 0), (0, 1)]).unwrap();
        let el = get_edgelist(&g).unwrap();
        assert_eq!(el.len(), 2);
    }
}
