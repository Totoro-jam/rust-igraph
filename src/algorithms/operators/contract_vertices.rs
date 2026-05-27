//! Vertex contraction operator (ALGO-OP-010).
//!
//! Merges groups of vertices into single vertices, remapping edges.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Contracts vertices according to a grouping, merging edges.
///
/// Each vertex `v` is mapped to the group `mapping[v]`. All vertices in the
/// same group are merged into a single vertex in the result graph. Edges are
/// remapped accordingly — this typically produces self-loops (from in-group
/// edges) and multi-edges (from multiple inter-group connections). Use
/// [`crate::simplify`] afterwards to remove these if desired.
///
/// The number of vertices in the result equals `max(mapping) + 1`. To avoid
/// orphan vertices (IDs with no corresponding input vertex), use consecutive
/// integers starting from zero.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `mapping` — slice of length `vcount` where `mapping[old] = new_group`.
///
/// # Errors
///
/// Returns `InvalidArgument` if `mapping.len() != graph.vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, contract_vertices};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// // Merge vertices 0+1 → group 0, vertices 2+3 → group 1
/// let mapping = [0, 0, 1, 1];
/// let cg = contract_vertices(&g, &mapping).unwrap();
/// assert_eq!(cg.vcount(), 2);
/// // Edge (0,1) → self-loop on 0, edge (1,2) → (0,1), edge (2,3) → self-loop on 1
/// assert_eq!(cg.ecount(), 3);
/// ```
pub fn contract_vertices(graph: &Graph, mapping: &[VertexId]) -> IgraphResult<Graph> {
    let n = graph.vcount();

    if mapping.len() != n as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "mapping length {} does not match vertex count {n}",
            mapping.len()
        )));
    }

    // Determine number of vertices in the result
    let new_vcount = if n == 0 {
        0u32
    } else {
        mapping.iter().copied().max().unwrap_or(0) + 1
    };

    // Remap all edges
    let ecount = graph.ecount();
    let directed = graph.is_directed();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        let new_src = mapping[src as usize];
        let new_tgt = mapping[tgt as usize];
        edges.push((new_src, new_tgt));
    }

    let mut result = Graph::new(new_vcount, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_contraction() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        // Merge 0+1 → 0, 2+3 → 1
        let cg = contract_vertices(&g, &[0, 0, 1, 1]).unwrap();
        assert_eq!(cg.vcount(), 2);
        assert_eq!(cg.ecount(), 3);
    }

    #[test]
    fn test_identity_mapping() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let cg = contract_vertices(&g, &[0, 1, 2]).unwrap();
        assert_eq!(cg.vcount(), 3);
        assert_eq!(cg.ecount(), 2);
        for eid in 0..2u32 {
            assert_eq!(g.edge(eid).unwrap(), cg.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_all_into_one() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        // All vertices merge into group 0
        let cg = contract_vertices(&g, &[0, 0, 0, 0]).unwrap();
        assert_eq!(cg.vcount(), 1);
        assert_eq!(cg.ecount(), 4); // all become self-loops
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();

        // Merge 0+2 → 0, 1+3 → 1
        let cg = contract_vertices(&g, &[0, 1, 0, 1]).unwrap();
        assert!(cg.is_directed());
        assert_eq!(cg.vcount(), 2);
        assert_eq!(cg.ecount(), 2);
        assert_eq!(cg.edge(0).unwrap(), (0, 1));
        assert_eq!(cg.edge(1).unwrap(), (0, 1));
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let cg = contract_vertices(&g, &[]).unwrap();
        assert_eq!(cg.vcount(), 0);
        assert_eq!(cg.ecount(), 0);
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(5);
        let cg = contract_vertices(&g, &[0, 0, 1, 1, 2]).unwrap();
        assert_eq!(cg.vcount(), 3);
        assert_eq!(cg.ecount(), 0);
    }

    #[test]
    fn test_wrong_length() {
        let g = Graph::with_vertices(3);
        let result = contract_vertices(&g, &[0, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop_preserved() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(1, 1).unwrap();

        let cg = contract_vertices(&g, &[0, 0, 1]).unwrap();
        assert_eq!(cg.vcount(), 2);
        assert_eq!(cg.ecount(), 1);
        assert_eq!(cg.edge(0).unwrap(), (0, 0));
    }

    #[test]
    fn test_non_consecutive_mapping() {
        // mapping values don't need to cover all IDs in [0, max]
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        // Map to 0, 5, 5 — result has 6 vertices (max+1 = 6)
        let cg = contract_vertices(&g, &[0, 5, 5]).unwrap();
        assert_eq!(cg.vcount(), 6);
        assert_eq!(cg.ecount(), 2);
    }
}
