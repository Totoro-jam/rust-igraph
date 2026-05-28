//! Bipartite graph projection (ALGO-OP-024).
//!
//! Counterpart of `igraph_bipartite_projection()` from
//! `references/igraph/src/misc/bipartite.c`.
//!
//! Projects a bipartite (two-mode) network onto one of its vertex
//! sets. Two vertices of the same type are connected in the projection
//! if they share at least one common neighbor of the other type.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of a bipartite projection.
#[derive(Debug, Clone)]
pub struct BipartiteProjection {
    /// The projected graph (undirected).
    pub graph: Graph,
    /// Mapping from projection vertex IDs to original vertex IDs.
    /// `vertex_map[proj_v]` is the original vertex ID.
    pub vertex_map: Vec<VertexId>,
    /// Edge multiplicities: `multiplicity[e]` is the number of common
    /// neighbors that caused projection edge `e` to exist.
    pub multiplicity: Vec<u32>,
}

/// Project a bipartite graph onto one of its vertex types.
///
/// Given a bipartite graph with vertex types specified by `types`
/// (where `types[v]` is `false` for type-0 and `true` for type-1),
/// this function creates the projection onto the specified type.
/// Two vertices of the target type are connected in the projection
/// if they share at least one common neighbor of the other type.
///
/// The graph is treated as undirected regardless of its directedness.
///
/// # Arguments
///
/// * `graph` — the bipartite input graph.
/// * `types` — boolean slice of length `vcount()`, giving vertex types.
/// * `project_type` — which type to project onto (`false` = type-0,
///   `true` = type-1).
///
/// # Errors
///
/// - `InvalidArgument` if `types.len() != vcount()`.
/// - `InvalidArgument` if a non-bipartite edge is found (both
///   endpoints have the same type).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bipartite_projection};
///
/// // Bipartite graph: types [F, F, T, T]
/// // Edges: 0-2, 0-3, 1-2, 1-3
/// // Projection onto type F (false): 0 and 1 share neighbors 2 and 3.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// let types = [false, false, true, true];
/// let proj = bipartite_projection(&g, &types, false).unwrap();
/// assert_eq!(proj.graph.vcount(), 2); // vertices 0, 1
/// assert_eq!(proj.graph.ecount(), 1); // edge 0-1
/// assert_eq!(proj.multiplicity[0], 2); // shared neighbors: 2 and 3
/// assert_eq!(proj.vertex_map, vec![0, 1]);
/// ```
pub fn bipartite_projection(
    graph: &Graph,
    types: &[bool],
    project_type: bool,
) -> IgraphResult<BipartiteProjection> {
    let n = graph.vcount();
    let n_us = n as usize;

    if types.len() != n_us {
        return Err(IgraphError::InvalidArgument(format!(
            "bipartite_projection: types length ({}) != vcount ({n})",
            types.len()
        )));
    }

    // Build undirected adjacency list (ignore directedness).
    let adj = build_undirected_adj(graph, n_us)?;

    // Identify vertices of the target type and build index mapping.
    let mut vertex_map: Vec<VertexId> = Vec::new();
    let mut new_index: Vec<Option<u32>> = vec![None; n_us];
    for (v, &t) in types.iter().enumerate() {
        if t == project_type {
            #[allow(clippy::cast_possible_truncation)]
            let new_id = vertex_map.len() as u32;
            new_index[v] = Some(new_id);
            #[allow(clippy::cast_possible_truncation)]
            let v_id = v as u32;
            vertex_map.push(v_id);
        }
    }

    let proj_n = vertex_map.len();
    let mut edges: Vec<(u32, u32)> = Vec::new();
    let mut multiplicity: Vec<u32> = Vec::new();

    // For each vertex `i` of the target type, find all vertices of
    // the target type reachable through exactly one intermediate step.
    // `added[v]` tracks which vertex `i` last "discovered" a link to `v`.
    let mut added: Vec<u32> = vec![0; n_us];
    let mut mult_count: Vec<u32> = vec![0; n_us];

    for (idx, &orig_i) in vertex_map.iter().enumerate() {
        let i_us = orig_i as usize;
        #[allow(clippy::cast_possible_truncation)]
        let marker = (idx as u32) + 1; // non-zero marker for vertex idx

        for &nei in &adj[i_us] {
            let nei_us = nei as usize;
            if types[nei_us] == project_type {
                return Err(IgraphError::InvalidArgument(format!(
                    "bipartite_projection: non-bipartite edge found ({orig_i}-{nei})"
                )));
            }
            // nei is of the other type. Look at nei's neighbors.
            for &nei2 in &adj[nei_us] {
                let nb2_idx = nei2 as usize;
                if nei2 <= orig_i {
                    continue; // avoid duplicates and self-loops
                }
                if types[nb2_idx] != project_type {
                    continue; // skip vertices of the other type
                }
                if added[nb2_idx] == marker {
                    mult_count[nb2_idx] += 1;
                    continue;
                }
                added[nb2_idx] = marker;
                mult_count[nb2_idx] = 1;
            }
        }

        // Collect edges from this vertex.
        for &nei in &adj[i_us] {
            let nei_us = nei as usize;
            for &nei2 in &adj[nei_us] {
                let nb2_idx = nei2 as usize;
                if nei2 <= orig_i || types[nb2_idx] != project_type {
                    continue;
                }
                if added[nb2_idx] == marker {
                    // First time seeing this in the collection pass
                    #[allow(clippy::cast_possible_truncation)]
                    let new_i = idx as u32;
                    let new_j = new_index[nb2_idx].unwrap_or(0);
                    edges.push((new_i, new_j));
                    multiplicity.push(mult_count[nb2_idx]);
                    added[nb2_idx] = 0; // clear so we don't double-add
                }
            }
        }
    }

    // Build the projection graph.
    #[allow(clippy::cast_possible_truncation)]
    let proj_n_u32 = proj_n as u32;
    let mut proj_graph = Graph::with_vertices(proj_n_u32);
    for &(u, v) in &edges {
        proj_graph.add_edge(u, v)?;
    }

    Ok(BipartiteProjection {
        graph: proj_graph,
        vertex_map,
        multiplicity,
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
    fn simple_complete_bipartite() {
        // K_{2,2}: types [F, F, T, T], edges: 0-2, 0-3, 1-2, 1-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let types = [false, false, true, true];

        let p0 = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p0.graph.vcount(), 2);
        assert_eq!(p0.graph.ecount(), 1);
        assert_eq!(p0.multiplicity[0], 2);
        assert_eq!(p0.vertex_map, vec![0, 1]);

        let p1 = bipartite_projection(&g, &types, true).unwrap();
        assert_eq!(p1.graph.vcount(), 2);
        assert_eq!(p1.graph.ecount(), 1);
        assert_eq!(p1.multiplicity[0], 2);
        assert_eq!(p1.vertex_map, vec![2, 3]);
    }

    #[test]
    fn star_bipartite() {
        // Star: center 0 (type F), leaves 1,2,3 (type T)
        // Projection onto T: all leaves share center → complete graph K3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let types = [false, true, true, true];

        let p = bipartite_projection(&g, &types, true).unwrap();
        assert_eq!(p.graph.vcount(), 3);
        assert_eq!(p.graph.ecount(), 3); // K3
        for &m in &p.multiplicity {
            assert_eq!(m, 1); // only one shared neighbor (vertex 0)
        }
    }

    #[test]
    fn no_edges() {
        // Bipartite with no edges: projection has vertices but no edges.
        let g = Graph::with_vertices(4);
        let types = [false, false, true, true];
        let p = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p.graph.vcount(), 2);
        assert_eq!(p.graph.ecount(), 0);
        assert!(p.multiplicity.is_empty());
    }

    #[test]
    fn single_type() {
        // All vertices are type F with edges between them → error (non-bipartite).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let types = [false, false, false];
        assert!(bipartite_projection(&g, &types, false).is_err());
    }

    #[test]
    fn non_bipartite_edge_error() {
        // Mixed: 0(F)-1(F) is non-bipartite edge
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let types = [false, false, true];
        assert!(bipartite_projection(&g, &types, false).is_err());
    }

    #[test]
    fn types_length_mismatch() {
        let g = Graph::with_vertices(3);
        let types = [false, true];
        assert!(bipartite_projection(&g, &types, false).is_err());
    }

    #[test]
    fn path_bipartite() {
        // Path: 0(F)-1(T)-2(F)-3(T)-4(F)
        // Projection onto F: 0 and 2 share neighbor 1, 2 and 4 share neighbor 3
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let types = [false, true, false, true, false];

        let p = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p.graph.vcount(), 3); // vertices 0, 2, 4
        assert_eq!(p.graph.ecount(), 2); // 0-2 and 2-4
        assert_eq!(p.vertex_map, vec![0, 2, 4]);
        for &m in &p.multiplicity {
            assert_eq!(m, 1);
        }
    }

    #[test]
    fn multiplicity_multiple() {
        // 0(F) connected to 2(T) and 3(T), 1(F) connected to 2(T) and 3(T) and 4(T)
        // Projection onto F: 0 and 1 share TWO common neighbors (2 and 3)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let types = [false, false, true, true, true];

        let p = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p.graph.vcount(), 2);
        assert_eq!(p.graph.ecount(), 1);
        assert_eq!(p.multiplicity[0], 2); // shared: 2 and 3
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let types: [bool; 0] = [];
        let p = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p.graph.vcount(), 0);
        assert_eq!(p.graph.ecount(), 0);
    }

    #[test]
    fn directed_treated_as_undirected() {
        // Directed edges should still work (treated as undirected)
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 1).unwrap();
        let types = [false, false, true, true];

        let p = bipartite_projection(&g, &types, false).unwrap();
        assert_eq!(p.graph.vcount(), 2);
        assert_eq!(p.graph.ecount(), 1);
        assert_eq!(p.multiplicity[0], 2);
    }
}
