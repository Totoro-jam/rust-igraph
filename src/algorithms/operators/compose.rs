//! Graph composition operator (ALGO-OP-013).
//!
//! Computes the relational composition of two graphs.

use crate::core::error::IgraphError;
use crate::core::{Graph, IgraphResult, VertexId};

/// Computes the composition of two graphs.
///
/// The composed graph has `max(|V1|, |V2|)` vertices. An edge `(i, j)`
/// exists in the result if and only if there is some vertex `k` such that
/// `(i, k)` is an edge in `g1` and `(k, j)` is an edge in `g2`.
///
/// Both graphs must have the same directedness. The result may contain
/// multi-edges; use [`crate::simplify`] to remove them if needed.
///
/// # Arguments
///
/// * `g1` — the first operand (left side of composition).
/// * `g2` — the second operand (right side of composition).
///
/// # Errors
///
/// Returns `InvalidArgument` if `g1` and `g2` differ in directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, compose};
///
/// // g1: 0→1→2, g2: 0→1→2 (directed paths)
/// let mut g1 = Graph::new(3, true).unwrap();
/// g1.add_edge(0, 1).unwrap();
/// g1.add_edge(1, 2).unwrap();
///
/// let mut g2 = Graph::new(3, true).unwrap();
/// g2.add_edge(0, 1).unwrap();
/// g2.add_edge(1, 2).unwrap();
///
/// let c = compose(&g1, &g2).unwrap();
/// assert_eq!(c.vcount(), 3);
/// // (0,1) in g1 + (1,2) in g2 → (0,2) in result
/// // (1,2) in g1 + ... vertex 2 has no outgoing in g2 → nothing
/// assert_eq!(c.ecount(), 1);
/// assert_eq!(c.edge(0).unwrap(), (0, 2));
/// ```
pub fn compose(g1: &Graph, g2: &Graph) -> IgraphResult<Graph> {
    if g1.is_directed() != g2.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cannot compose directed and undirected graphs".to_string(),
        ));
    }

    let n1 = g1.vcount();
    let n2 = g2.vcount();
    let directed = g1.is_directed();
    let n = n1.max(n2);

    // Build outgoing adjacency for g1 and g2
    let adj1 = build_out_adjacency(g1, n1)?;
    let adj2 = build_out_adjacency(g2, n2)?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    for i in 0..n1 {
        for &k in &adj1[i as usize] {
            if k < n2 {
                for &j in &adj2[k as usize] {
                    edges.push((i, j));
                }
            }
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

fn build_out_adjacency(graph: &Graph, n: u32) -> IgraphResult<Vec<Vec<VertexId>>> {
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n as usize];
    let ecount = graph.ecount();

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        adj[src as usize].push(tgt);
        if !graph.is_directed() && src != tgt {
            adj[tgt as usize].push(src);
        }
    }
    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directed_path_compose() {
        // g1: 0→1→2, g2: 0→1→2
        let mut g1 = Graph::new(3, true).unwrap();
        g1.add_edge(0, 1).unwrap();
        g1.add_edge(1, 2).unwrap();

        let mut g2 = Graph::new(3, true).unwrap();
        g2.add_edge(0, 1).unwrap();
        g2.add_edge(1, 2).unwrap();

        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.vcount(), 3);
        // g1 has (0,1),(1,2); g2 has (0,1),(1,2)
        // Composition: (0,1) in g1 → check g2 from 1: (1,2) → result (0,2)
        //              (1,2) in g1 → check g2 from 2: nothing
        assert_eq!(c.ecount(), 1);
        assert_eq!(c.edge(0).unwrap(), (0, 2));
    }

    #[test]
    fn test_identity_compose() {
        // g1 = g2 = complete K3 directed (all 6 directed edges)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 1).unwrap();

        let c = compose(&g, &g).unwrap();
        assert_eq!(c.vcount(), 3);
        // Each vertex reaches all 2 others, each of those reaches 2 others
        // = 3 * 2 * 2 = 12 edges (with duplicates since K3 composition = K3 with multiplicity 2)
        assert_eq!(c.ecount(), 12);
    }

    #[test]
    fn test_different_sizes() {
        // g1 has 3 vertices, g2 has 5 vertices
        let mut g1 = Graph::new(3, true).unwrap();
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::new(5, true).unwrap();
        g2.add_edge(1, 4).unwrap();

        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.vcount(), 5); // max(3,5)
        // g1: (0,1), g2 from 1: (1,4) → result: (0,4)
        assert_eq!(c.ecount(), 1);
        assert_eq!(c.edge(0).unwrap(), (0, 4));
    }

    #[test]
    fn test_undirected() {
        // Undirected: edge (a,b) means both (a,b) and (b,a) in the relation
        let mut g1 = Graph::with_vertices(3);
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::with_vertices(3);
        g2.add_edge(1, 2).unwrap();

        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.vcount(), 3);
        // g1 relation: {(0,1),(1,0)}; g2 relation: {(1,2),(2,1)}
        // Compose: (0,1)+(1,2)=(0,2); (1,0)+(0,?)=nothing from 0 in g2
        //          (0,1)+(1,?)=(0,2) already counted for undirected adj
        //          but we also have (1,0) in adj → check g2 from 0: nothing
        // Actually: adj1[0] = [1], adj1[1] = [0]
        // adj2[1] = [2], adj2[2] = [1]
        // Compose: from 0: adj1[0]=[1], for k=1: adj2[1]=[2] → (0,2)
        //          from 1: adj1[1]=[0], for k=0: adj2[0]=[] → nothing
        //          from 2: not in g1 (no out-neighbors) → nothing
        // Result: 1 edge (0,2)
        assert_eq!(c.ecount(), 1);
    }

    #[test]
    fn test_mixed_directedness_error() {
        let g1 = Graph::new(3, true).unwrap();
        let g2 = Graph::with_vertices(3);
        assert!(compose(&g1, &g2).is_err());
    }

    #[test]
    fn test_empty() {
        let g1 = Graph::new(0, true).unwrap();
        let g2 = Graph::new(0, true).unwrap();
        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.vcount(), 0);
        assert_eq!(c.ecount(), 0);
    }

    #[test]
    fn test_no_overlap() {
        // g1: 0→1, g2: 2→3 (no shared intermediate vertex)
        let mut g1 = Graph::new(4, true).unwrap();
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::new(4, true).unwrap();
        g2.add_edge(2, 3).unwrap();

        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.vcount(), 4);
        // g1: (0,1), check g2 from 1: nothing → 0 edges
        assert_eq!(c.ecount(), 0);
    }

    #[test]
    fn test_self_loop_generation() {
        // g1: 0→1, g2: 1→0 → compose: (0,0) self-loop
        let mut g1 = Graph::new(2, true).unwrap();
        g1.add_edge(0, 1).unwrap();

        let mut g2 = Graph::new(2, true).unwrap();
        g2.add_edge(1, 0).unwrap();

        let c = compose(&g1, &g2).unwrap();
        assert_eq!(c.ecount(), 1);
        assert_eq!(c.edge(0).unwrap(), (0, 0));
    }
}
