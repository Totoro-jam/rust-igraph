//! Induced subgraph edges query (ALGO-OP-026).
//!
//! Counterpart of `igraph_induced_subgraph_edges()` from
//! `references/igraph/src/operators/subgraph.c`.
//!
//! Returns the edge IDs connecting vertices in a specified set, without
//! building the full subgraph.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Returns the IDs of edges whose both endpoints lie in `vids`.
///
/// This is a lightweight alternative to [`crate::induced_subgraph`] when
/// you only need the edge list, not the full subgraph structure. It's
/// useful for counting or iterating over edges within a vertex subset.
///
/// The returned edge IDs are sorted in ascending order. Duplicate vertex
/// IDs in `vids` are silently ignored.
///
/// # Errors
///
/// Returns `InvalidArgument` if any vertex ID in `vids` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, induced_subgraph_edges};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap(); // eid 0
/// g.add_edge(1, 2).unwrap(); // eid 1
/// g.add_edge(2, 3).unwrap(); // eid 2
/// g.add_edge(3, 4).unwrap(); // eid 3
/// g.add_edge(0, 4).unwrap(); // eid 4
///
/// // Edges within {0, 1, 2}: only 0-1 and 1-2
/// let edges = induced_subgraph_edges(&g, &[0, 1, 2]).unwrap();
/// assert_eq!(edges, vec![0, 1]);
/// ```
pub fn induced_subgraph_edges(graph: &Graph, vids: &[u32]) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    let m = graph.ecount();

    // Validate and build a membership set.
    let mut in_set: Vec<bool> = vec![false; n as usize];
    for &vid in vids {
        if vid >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "induced_subgraph_edges: vertex ID {vid} out of range (graph has {n} vertices)"
            )));
        }
        in_set[vid as usize] = true;
    }

    // Scan all edges and collect those with both endpoints in the set.
    let mut result: Vec<u32> = Vec::new();
    for eid_usize in 0..m {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (from, to) = graph.edge(eid)?;
        if in_set[from as usize] && in_set[to as usize] {
            result.push(eid);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_undirected() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2
        g.add_edge(3, 4).unwrap(); // 3
        g.add_edge(0, 4).unwrap(); // 4

        let edges = induced_subgraph_edges(&g, &[0, 1, 2]).unwrap();
        assert_eq!(edges, vec![0, 1]);
    }

    #[test]
    fn all_vertices() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let edges = induced_subgraph_edges(&g, &[0, 1, 2, 3]).unwrap();
        assert_eq!(edges, vec![0, 1, 2]);
    }

    #[test]
    fn single_vertex() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let edges = induced_subgraph_edges(&g, &[1]).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn self_loop_included() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap(); // 0: self-loop
        g.add_edge(0, 1).unwrap(); // 1
        g.add_edge(1, 2).unwrap(); // 2

        let edges = induced_subgraph_edges(&g, &[0]).unwrap();
        assert_eq!(edges, vec![0]); // self-loop counts
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 0).unwrap(); // 2
        g.add_edge(2, 3).unwrap(); // 3

        let edges = induced_subgraph_edges(&g, &[0, 1, 2]).unwrap();
        assert_eq!(edges, vec![0, 1, 2]);
    }

    #[test]
    fn empty_vertex_set() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();

        let edges = induced_subgraph_edges(&g, &[]).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn no_matching_edges() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();

        // Vertices 0 and 2 have no edge between them
        let edges = induced_subgraph_edges(&g, &[0, 2]).unwrap();
        assert!(edges.is_empty());
    }

    #[test]
    fn duplicate_vertex_ids() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1

        let edges = induced_subgraph_edges(&g, &[0, 1, 0, 1]).unwrap();
        assert_eq!(edges, vec![0]);
    }

    #[test]
    fn vertex_out_of_range() {
        let g = Graph::with_vertices(3);
        let err = induced_subgraph_edges(&g, &[5]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn graph_with_no_edges() {
        let g = Graph::with_vertices(5);
        let edges = induced_subgraph_edges(&g, &[0, 1, 2]).unwrap();
        assert!(edges.is_empty());
    }
}
