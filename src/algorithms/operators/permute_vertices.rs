//! Vertex permutation operator (ALGO-OP-009).
//!
//! Creates a new graph by remapping vertex IDs according to a permutation.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Creates a new graph with vertices permuted according to the given mapping.
///
/// `permutation[i]` specifies which old vertex becomes new vertex `i`.
/// In other words, new vertex `i` gets all edges that old vertex
/// `permutation[i]` had, with endpoints remapped accordingly.
///
/// The permutation must be a valid bijection on `[0, n)`.
///
/// # Arguments
///
/// * `graph` — the input graph.
/// * `permutation` — slice of length `n` where `permutation[new] = old`.
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `permutation.len() != graph.vcount()`
/// - Any value is out of range `[0, n)`
/// - Any value appears more than once (not a valid permutation)
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, permute_vertices};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// // Reverse the vertex ordering: new 0 = old 2, new 1 = old 1, new 2 = old 0
/// let perm = [2, 1, 0];
/// let pg = permute_vertices(&g, &perm).unwrap();
/// assert_eq!(pg.vcount(), 3);
/// assert_eq!(pg.ecount(), 2);
/// ```
pub fn permute_vertices(graph: &Graph, permutation: &[VertexId]) -> IgraphResult<Graph> {
    let n = graph.vcount();

    if permutation.len() != n as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "permutation length {} does not match vertex count {n}",
            permutation.len()
        )));
    }

    // Validate permutation and build inverse (old → new)
    let mut inverse = vec![u32::MAX; n as usize];
    for (new_id, &old_id) in permutation.iter().enumerate() {
        if old_id >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "permutation contains invalid vertex id {old_id} (graph has {n} vertices)"
            )));
        }
        if inverse[old_id as usize] != u32::MAX {
            return Err(IgraphError::InvalidArgument(format!(
                "duplicate entry {old_id} in permutation"
            )));
        }
        #[allow(clippy::cast_possible_truncation)]
        let new_id_u32 = new_id as u32;
        inverse[old_id as usize] = new_id_u32;
    }

    // Build new edge list with remapped vertex IDs
    let ecount = graph.ecount();
    let directed = graph.is_directed();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        let new_src = inverse[src as usize];
        let new_tgt = inverse[tgt as usize];
        edges.push((new_src, new_tgt));
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_permutation() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let pg = permute_vertices(&g, &[0, 1, 2, 3]).unwrap();
        assert_eq!(pg.vcount(), 4);
        assert_eq!(pg.ecount(), 3);
        for eid in 0..3u32 {
            assert_eq!(g.edge(eid).unwrap(), pg.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_reverse_permutation() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        // new vertex 0 = old vertex 2, new vertex 1 = old vertex 1, new vertex 2 = old vertex 0
        let pg = permute_vertices(&g, &[2, 1, 0]).unwrap();
        assert_eq!(pg.vcount(), 3);
        assert_eq!(pg.ecount(), 2);
        // Old edge (0,1) → new edge (2,1) → stored as (1,2) (undirected canonical)
        assert_eq!(pg.edge(0).unwrap(), (1, 2));
        // Old edge (1,2) → new edge (1,0) → stored as (0,1)
        assert_eq!(pg.edge(1).unwrap(), (0, 1));
    }

    #[test]
    fn test_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let pg = permute_vertices(&g, &[2, 0, 1]).unwrap();
        assert!(pg.is_directed());
        // old 0 → new 1, old 1 → new 2, old 2 → new 0 (inverse of [2,0,1])
        // Edge (0,1): new = (1,2)
        assert_eq!(pg.edge(0).unwrap(), (1, 2));
        // Edge (1,2): new = (2,0)
        assert_eq!(pg.edge(1).unwrap(), (2, 0));
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let pg = permute_vertices(&g, &[]).unwrap();
        assert_eq!(pg.vcount(), 0);
    }

    #[test]
    fn test_wrong_length() {
        let g = Graph::with_vertices(3);
        let result = permute_vertices(&g, &[0, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_out_of_range() {
        let g = Graph::with_vertices(3);
        let result = permute_vertices(&g, &[0, 1, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate() {
        let g = Graph::with_vertices(3);
        let result = permute_vertices(&g, &[0, 1, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(1, 1).unwrap();

        let pg = permute_vertices(&g, &[2, 0, 1]).unwrap();
        // old 1 → new 2, self-loop (1,1) becomes (2,2)
        // inverse: old 0 → new 1, old 1 → new 2, old 2 → new 0
        assert_eq!(pg.edge(0).unwrap(), (2, 2));
    }

    #[test]
    fn test_cycle_permutation() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();

        // Cyclic shift: perm[new] = old → new 0 = old 1, new 1 = old 2, ...
        let pg = permute_vertices(&g, &[1, 2, 3, 0]).unwrap();
        assert_eq!(pg.vcount(), 4);
        assert_eq!(pg.ecount(), 4);
        // inverse: old 0 → new 3, old 1 → new 0, old 2 → new 1, old 3 → new 2
        // Edge (0,1) → (3, 0) → stored as (0, 3) (undirected canonical)
        assert_eq!(pg.edge(0).unwrap(), (0, 3));
        // Edge (1,2) → (0, 1) → stored as (0, 1)
        assert_eq!(pg.edge(1).unwrap(), (0, 1));
    }
}
