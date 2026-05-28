//! Bipartite projection size estimation (ALGO-OP-025).
//!
//! Counterpart of `igraph_bipartite_projection_size()` from
//! `references/igraph/src/misc/bipartite.c`.
//!
//! Computes the number of vertices and edges in both projections of a
//! bipartite graph without constructing the projected graphs. Useful
//! for memory estimation before calling [`super::bipartite_projection`].

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Sizes of both bipartite projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BipartiteProjectionSize {
    /// Number of vertices in the projection of type-0 (false) vertices.
    pub vcount1: u32,
    /// Number of edges in the projection of type-0 (false) vertices.
    pub ecount1: u32,
    /// Number of vertices in the projection of type-1 (true) vertices.
    pub vcount2: u32,
    /// Number of edges in the projection of type-1 (true) vertices.
    pub ecount2: u32,
}

/// Compute sizes of both bipartite projections without building them.
///
/// Given a bipartite graph with vertex types specified by `types`,
/// this function counts the vertices and edges that each projection
/// would contain if [`super::bipartite_projection`] were called.
///
/// Two vertices of the same type are connected in a projection if
/// they share at least one common neighbor of the other type. This
/// function counts such pairs efficiently using a marking array.
///
/// The graph is treated as undirected regardless of its directedness.
///
/// # Arguments
///
/// * `graph` — the bipartite input graph.
/// * `types` — boolean slice of length `vcount()`, giving vertex types.
///
/// # Errors
///
/// - `InvalidArgument` if `types.len() != vcount()`.
/// - `InvalidArgument` if a non-bipartite edge is found.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bipartite_projection_size};
///
/// // K_{2,3}: types [F, F, T, T, T]
/// // Edges: 0-2, 0-3, 0-4, 1-2, 1-3, 1-4
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// let types = [false, false, true, true, true];
/// let sz = bipartite_projection_size(&g, &types).unwrap();
/// assert_eq!(sz.vcount1, 2); // type-0 vertices: 0, 1
/// assert_eq!(sz.ecount1, 1); // edge 0-1 (share 3 common neighbors)
/// assert_eq!(sz.vcount2, 3); // type-1 vertices: 2, 3, 4
/// assert_eq!(sz.ecount2, 3); // K3 (all share 2 common neighbors)
/// ```
pub fn bipartite_projection_size(
    graph: &Graph,
    types: &[bool],
) -> IgraphResult<BipartiteProjectionSize> {
    let n = graph.vcount();
    let n_us = n as usize;

    if types.len() != n_us {
        return Err(IgraphError::InvalidArgument(format!(
            "bipartite_projection_size: types length ({}) != vcount ({n})",
            types.len()
        )));
    }

    let adj = build_undirected_adj(graph, n_us)?;

    let mut vc1: u32 = 0;
    let mut ec1: u32 = 0;
    let mut vc2: u32 = 0;
    let mut ec2: u32 = 0;

    // `added[v]` stores `i + 1` when vertex `v` was last "discovered"
    // as a 2-hop neighbor of vertex `i`, avoiding double-counting.
    let mut added: Vec<u32> = vec![0; n_us];

    for i in 0..n_us {
        if types[i] {
            vc2 = vc2.saturating_add(1);
        } else {
            vc1 = vc1.saturating_add(1);
        }

        #[allow(clippy::cast_possible_truncation)]
        let marker = (i as u32).wrapping_add(1);

        for &nei in &adj[i] {
            let nei_us = nei as usize;
            if types[nei_us] == types[i] {
                return Err(IgraphError::InvalidArgument(format!(
                    "bipartite_projection_size: non-bipartite edge found ({i}-{nei})"
                )));
            }
            for &nb2 in &adj[nei_us] {
                if nb2 as usize <= i {
                    continue;
                }
                if added[nb2 as usize] == marker {
                    continue;
                }
                added[nb2 as usize] = marker;
                if types[i] {
                    ec2 = ec2.saturating_add(1);
                } else {
                    ec1 = ec1.saturating_add(1);
                }
            }
        }
    }

    Ok(BipartiteProjectionSize {
        vcount1: vc1,
        ecount1: ec1,
        vcount2: vc2,
        ecount2: ec2,
    })
}

fn build_undirected_adj(graph: &Graph, n_us: usize) -> IgraphResult<Vec<Vec<VertexId>>> {
    let m =
        u32::try_from(graph.ecount()).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n_us];
    for eid in 0..m {
        let (from, to) = graph.edge(eid)?;
        adj[from as usize].push(to);
        if from != to {
            adj[to as usize].push(from);
        }
    }
    Ok(adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let types = [false, false, true, true];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 2);
        assert_eq!(sz.ecount1, 1);
        assert_eq!(sz.vcount2, 2);
        assert_eq!(sz.ecount2, 1);
    }

    #[test]
    fn k23() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let types = [false, false, true, true, true];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 2);
        assert_eq!(sz.ecount1, 1);
        assert_eq!(sz.vcount2, 3);
        assert_eq!(sz.ecount2, 3);
    }

    #[test]
    fn star_bipartite() {
        // Center 0 (F), leaves 1,2,3 (T)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let types = [false, true, true, true];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 1);
        assert_eq!(sz.ecount1, 0); // single vertex, no edges
        assert_eq!(sz.vcount2, 3);
        assert_eq!(sz.ecount2, 3); // K3
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(4);
        let types = [false, false, true, true];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 2);
        assert_eq!(sz.ecount1, 0);
        assert_eq!(sz.vcount2, 2);
        assert_eq!(sz.ecount2, 0);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let types: [bool; 0] = [];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 0);
        assert_eq!(sz.ecount1, 0);
        assert_eq!(sz.vcount2, 0);
        assert_eq!(sz.ecount2, 0);
    }

    #[test]
    fn types_mismatch() {
        let g = Graph::with_vertices(3);
        let types = [false, true];
        assert!(bipartite_projection_size(&g, &types).is_err());
    }

    #[test]
    fn non_bipartite_edge() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let types = [false, false, true];
        assert!(bipartite_projection_size(&g, &types).is_err());
    }

    #[test]
    fn path_bipartite() {
        // Path: 0(F)-1(T)-2(F)-3(T)-4(F)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let types = [false, true, false, true, false];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 3); // 0, 2, 4
        assert_eq!(sz.ecount1, 2); // 0-2 (via 1) and 2-4 (via 3)
        assert_eq!(sz.vcount2, 2); // 1, 3
        assert_eq!(sz.ecount2, 1); // 1-3 (via 2)
    }

    #[test]
    fn directed_treated_as_undirected() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 1).unwrap();
        let types = [false, false, true, true];
        let sz = bipartite_projection_size(&g, &types).unwrap();
        assert_eq!(sz.vcount1, 2);
        assert_eq!(sz.ecount1, 1);
        assert_eq!(sz.vcount2, 2);
        assert_eq!(sz.ecount2, 1);
    }
}
