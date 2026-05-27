//! Graph join operator (ALGO-OP-014).
//!
//! Creates the join of two disjoint graphs (disjoint union plus all
//! cross-edges between the two vertex sets).

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Creates the join of two graphs.
///
/// The join first takes the disjoint union of `left` and `right`, then adds
/// all edges between the vertices originating from `left` and those from
/// `right`. This produces a graph with `|V1| + |V2|` vertices and
/// `|E1| + |E2| + |V1|*|V2|` edges (for undirected), or
/// `|E1| + |E2| + 2*|V1|*|V2|` edges (for directed, since both directions
/// are added).
///
/// Both graphs must have the same directedness.
///
/// # Arguments
///
/// * `left` — the first graph.
/// * `right` — the second graph.
///
/// # Errors
///
/// Returns `InvalidArgument` if the graphs differ in directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, join};
///
/// let g1 = Graph::with_vertices(2); // 2 isolated vertices
/// let g2 = Graph::with_vertices(3); // 3 isolated vertices
///
/// let j = join(&g1, &g2).unwrap();
/// assert_eq!(j.vcount(), 5);
/// // 0 original edges + 2*3 = 6 cross-edges
/// assert_eq!(j.ecount(), 6);
/// ```
pub fn join(left: &Graph, right: &Graph) -> IgraphResult<Graph> {
    if left.is_directed() != right.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cannot join directed and undirected graphs".to_string(),
        ));
    }

    let n1 = left.vcount();
    let n2 = right.vcount();
    let directed = left.is_directed();
    let n = n1
        .checked_add(n2)
        .ok_or_else(|| IgraphError::InvalidArgument("vertex count overflow in join".to_string()))?;

    let e1 = left.ecount();
    let e2 = right.ecount();

    // Calculate cross-edges count
    let cross_count = if directed {
        2 * (n1 as usize) * (n2 as usize)
    } else {
        (n1 as usize) * (n2 as usize)
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(e1 + e2 + cross_count);

    // Copy left's edges
    for eid in 0..e1 {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        edges.push(left.edge(eid_u32)?);
    }

    // Copy right's edges, shifted by n1
    for eid in 0..e2 {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = right.edge(eid_u32)?;
        edges.push((src + n1, tgt + n1));
    }

    // Add all cross-edges
    for i in 0..n1 {
        for j in 0..n2 {
            edges.push((i, j + n1));
            if directed {
                edges.push((j + n1, i));
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undirected() {
        let g1 = Graph::with_vertices(2);
        let g2 = Graph::with_vertices(3);

        let j = join(&g1, &g2).unwrap();
        assert_eq!(j.vcount(), 5);
        assert_eq!(j.ecount(), 6); // 2*3
    }

    #[test]
    fn test_with_existing_edges() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let j = join(&g1, &g2).unwrap();
        assert_eq!(j.vcount(), 4);
        // 1 + 1 + 2*2 = 6
        assert_eq!(j.ecount(), 6);
    }

    #[test]
    fn test_directed() {
        let g1 = Graph::new(2, true).unwrap();
        let g2 = Graph::new(3, true).unwrap();

        let j = join(&g1, &g2).unwrap();
        assert!(j.is_directed());
        assert_eq!(j.vcount(), 5);
        // 0 + 0 + 2*2*3 = 12 cross-edges (both directions)
        assert_eq!(j.ecount(), 12);
    }

    #[test]
    fn test_empty_left() {
        let g1 = Graph::with_vertices(0);
        let g2 = Graph::with_vertices(3);

        let j = join(&g1, &g2).unwrap();
        assert_eq!(j.vcount(), 3);
        assert_eq!(j.ecount(), 0); // no cross-edges since |V1|=0
    }

    #[test]
    fn test_empty_right() {
        let g1 = Graph::with_vertices(3);
        let g2 = Graph::with_vertices(0);

        let j = join(&g1, &g2).unwrap();
        assert_eq!(j.vcount(), 3);
        assert_eq!(j.ecount(), 0);
    }

    #[test]
    fn test_single_vertices() {
        let g1 = Graph::with_vertices(1);
        let g2 = Graph::with_vertices(1);

        let j = join(&g1, &g2).unwrap();
        assert_eq!(j.vcount(), 2);
        assert_eq!(j.ecount(), 1); // one edge between vertex 0 and 1
    }

    #[test]
    fn test_mixed_directedness_error() {
        let g1 = Graph::new(2, true).unwrap();
        let g2 = Graph::with_vertices(2);
        assert!(join(&g1, &g2).is_err());
    }

    #[test]
    fn test_vertex_ids_preserved() {
        let mut g1 = Graph::with_vertices(2);
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::with_vertices(2);
        g2.add_edge(0, 1).unwrap();

        let j = join(&g1, &g2).unwrap();
        // First edge is from g1: (0,1)
        assert_eq!(j.edge(0).unwrap(), (0, 1));
        // Second edge is from g2 shifted: (2,3)
        assert_eq!(j.edge(1).unwrap(), (2, 3));
    }
}
