//! Batch edge-ID lookup from vertex pairs.
//!
//! Counterpart of `igraph_get_eids()` in
//! `references/igraph/src/graph/type_indexededgelist.c`.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Look up edge IDs for a batch of vertex pairs.
///
/// Each element `(from, to)` in `pairs` is resolved to the
/// corresponding edge ID. If no edge exists for a given pair,
/// the result depends on `error`: when `true`, the function returns
/// an error; when `false`, the missing entry is `None`.
///
/// Counterpart of `igraph_get_eids` from
/// `references/igraph/src/graph/type_indexededgelist.c`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_eids};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // eid 0
/// g.add_edge(1, 2).unwrap(); // eid 1
/// g.add_edge(2, 3).unwrap(); // eid 2
///
/// let eids = get_eids(&g, &[(0, 1), (2, 3)], true).unwrap();
/// assert_eq!(eids, vec![Some(0), Some(2)]);
/// ```
pub fn get_eids(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
    error: bool,
) -> IgraphResult<Vec<Option<EdgeId>>> {
    let mut result = Vec::with_capacity(pairs.len());
    for &(from, to) in pairs {
        let eid = graph.find_eid(from, to)?;
        if error && eid.is_none() {
            return Err(IgraphError::InvalidArgument(format!(
                "no edge between {from} and {to}"
            )));
        }
        result.push(eid);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_lookup_undirected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let eids = get_eids(&g, &[(0, 1), (1, 2), (2, 3)], true).unwrap();
        assert_eq!(eids, vec![Some(0), Some(1), Some(2)]);
    }

    #[test]
    fn reverse_pair_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let eids = get_eids(&g, &[(1, 0)], true).unwrap();
        assert_eq!(eids, vec![Some(0)]);
    }

    #[test]
    fn missing_edge_error_mode() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let err = get_eids(&g, &[(0, 2)], true).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn missing_edge_silent_mode() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let eids = get_eids(&g, &[(0, 1), (0, 2)], false).unwrap();
        assert_eq!(eids, vec![Some(0), None]);
    }

    #[test]
    fn directed_one_way() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let eids = get_eids(&g, &[(0, 1)], true).unwrap();
        assert_eq!(eids, vec![Some(0)]);
        let eids = get_eids(&g, &[(1, 0)], false).unwrap();
        assert_eq!(eids, vec![None]);
    }

    #[test]
    fn empty_pairs() {
        let g = Graph::with_vertices(3);
        let eids = get_eids(&g, &[], true).unwrap();
        assert!(eids.is_empty());
    }

    #[test]
    fn invalid_vertex_errors() {
        let g = Graph::with_vertices(2);
        assert!(get_eids(&g, &[(0, 5)], true).is_err());
    }
}
