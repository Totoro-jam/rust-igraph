//! Bipartite graph constructor with validation (ALGO-CN-036).
//!
//! Counterpart of `igraph_create_bipartite()` in
//! `references/igraph/src/misc/bipartite.c:556-590`.
//!
//! Creates a graph from a type vector and edge list, verifying that
//! every edge connects vertices of different types (the bipartite
//! invariant).

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Create a bipartite graph from a type vector and edge list.
///
/// Validates that every edge connects vertices of different types
/// (one endpoint in each partition). The type vector length defines
/// the vertex count.
///
/// # Parameters
///
/// * `types` — per-vertex partition: `false` = partition 0, `true` = partition 1.
/// * `edges` — pairs `(from, to)` of vertex IDs.
/// * `directed` — whether the resulting graph is directed.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — a vertex ID in `edges` is out of
///   range, or an edge connects two vertices of the same type.
///
/// # Examples
///
/// ```
/// use rust_igraph::create_bipartite;
///
/// // Simple K_{2,2} bipartite graph.
/// let types = vec![false, false, true, true];
/// let edges = vec![(0, 2), (0, 3), (1, 2), (1, 3)];
/// let g = create_bipartite(&types, &edges, false).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
///
/// // Rejects same-type edges.
/// let bad_edges = vec![(0, 1)]; // both false
/// assert!(create_bipartite(&types, &bad_edges, false).is_err());
/// ```
pub fn create_bipartite(
    types: &[bool],
    edges: &[(VertexId, VertexId)],
    directed: bool,
) -> IgraphResult<Graph> {
    let n = u32::try_from(types.len()).map_err(|_| {
        IgraphError::InvalidArgument("create_bipartite: vertex count exceeds u32::MAX".to_string())
    })?;

    for (i, &(from, to)) in edges.iter().enumerate() {
        if from >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "create_bipartite: edge {i} source vertex {from} out of range (n={n})"
            )));
        }
        if to >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "create_bipartite: edge {i} target vertex {to} out of range (n={n})"
            )));
        }
        let t_from = types[from as usize];
        let t_to = types[to as usize];
        if t_from == t_to {
            return Err(IgraphError::InvalidArgument(format!(
                "create_bipartite: edge {i} ({from}, {to}) connects vertices of the same type"
            )));
        }
    }

    let mut graph = Graph::new(n, directed)?;
    if !edges.is_empty() {
        graph.add_edges(edges.to_vec())?;
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = create_bipartite(&[], &[], false).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn no_edges() {
        let types = vec![false, true, false, true];
        let g = create_bipartite(&types, &[], false).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn k22_undirected() {
        let types = vec![false, false, true, true];
        let edges = vec![(0, 2), (0, 3), (1, 2), (1, 3)];
        let g = create_bipartite(&types, &edges, false).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert!(!g.is_directed());
    }

    #[test]
    fn k22_directed() {
        let types = vec![false, false, true, true];
        let edges = vec![(0, 2), (0, 3), (1, 2), (1, 3)];
        let g = create_bipartite(&types, &edges, true).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert!(g.is_directed());
    }

    #[test]
    fn rejects_same_type_edge_both_false() {
        let types = vec![false, false, true];
        let edges = vec![(0, 1)];
        let result = create_bipartite(&types, &edges, false);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_same_type_edge_both_true() {
        let types = vec![false, true, true];
        let edges = vec![(1, 2)];
        let result = create_bipartite(&types, &edges, false);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_out_of_range_source() {
        let types = vec![false, true];
        let edges = vec![(5, 1)];
        let result = create_bipartite(&types, &edges, false);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_out_of_range_target() {
        let types = vec![false, true];
        let edges = vec![(0, 5)];
        let result = create_bipartite(&types, &edges, false);
        assert!(result.is_err());
    }

    #[test]
    fn single_edge() {
        let types = vec![false, true];
        let edges = vec![(0, 1)];
        let g = create_bipartite(&types, &edges, false).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn mixed_type_ordering() {
        // Types don't need to be contiguous
        let types = vec![true, false, true, false];
        let edges = vec![(0, 1), (0, 3), (2, 1), (2, 3)];
        let g = create_bipartite(&types, &edges, false).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
    }

    #[test]
    fn valid_directed_reverse_edges() {
        let types = vec![false, true];
        let edges = vec![(1, 0)]; // true -> false is valid
        let g = create_bipartite(&types, &edges, true).unwrap();
        assert_eq!(g.ecount(), 1);
    }
}
