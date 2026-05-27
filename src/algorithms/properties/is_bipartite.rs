//! Bipartiteness test and vertex partition (ALGO-PR-037).
//!
//! BFS-based 2-coloring to detect if a graph is bipartite.
//! Optionally returns the bipartition (type assignment for each vertex).
//!
//! Counterpart of `igraph_is_bipartite` from
//! `references/igraph/src/misc/bipartite.c`.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of bipartiteness check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BipartiteResult {
    /// Whether the graph is bipartite.
    pub is_bipartite: bool,
    /// If bipartite, the partition assignment for each vertex.
    /// `types[v]` is `false` for one side and `true` for the other.
    /// Empty if `is_bipartite` is false.
    pub types: Vec<bool>,
}

/// Check whether a graph is bipartite and optionally return the partition.
///
/// A graph is bipartite if its vertices can be divided into two disjoint
/// sets such that every edge connects a vertex in one set to a vertex in
/// the other. Equivalently, a graph is bipartite iff it contains no
/// odd-length cycles.
///
/// Edge directions are ignored — the test treats the graph as undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_bipartite};
///
/// // Path 0-1-2-3 is bipartite
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let result = is_bipartite(&g).unwrap();
/// assert!(result.is_bipartite);
/// assert_eq!(result.types.len(), 4);
///
/// // Triangle is not bipartite
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let result = is_bipartite(&g).unwrap();
/// assert!(!result.is_bipartite);
/// ```
pub fn is_bipartite(graph: &Graph) -> IgraphResult<BipartiteResult> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(BipartiteResult {
            is_bipartite: true,
            types: Vec::new(),
        });
    }

    // 0 = unseen, 1 = type A, 2 = type B
    let mut color: Vec<u8> = vec![0; n as usize];
    let mut queue: VecDeque<VertexId> = VecDeque::new();
    let mut bipartite = true;

    'outer: for start in 0..n {
        if color[start as usize] != 0 {
            continue;
        }

        color[start as usize] = 1;
        queue.push_back(start);

        while let Some(v) = queue.pop_front() {
            let v_color = color[v as usize];
            let neighbors = get_all_neighbors(graph, v)?;

            for nei in neighbors {
                if color[nei as usize] == 0 {
                    color[nei as usize] = 3 - v_color;
                    queue.push_back(nei);
                } else if color[nei as usize] == v_color {
                    bipartite = false;
                    break 'outer;
                }
            }
        }
    }

    if bipartite {
        let types: Vec<bool> = color.iter().map(|&c| c == 2).collect();
        Ok(BipartiteResult {
            is_bipartite: true,
            types,
        })
    } else {
        Ok(BipartiteResult {
            is_bipartite: false,
            types: Vec::new(),
        })
    }
}

/// Get all neighbors of vertex v (ignoring direction).
fn get_all_neighbors(graph: &Graph, v: VertexId) -> IgraphResult<Vec<VertexId>> {
    let mut neighbors = Vec::new();
    let out_edges = graph.incident(v)?;

    for eid in &out_edges {
        let (from, to) = graph.edge(*eid)?;
        let nei = if from == v { to } else { from };
        neighbors.push(nei);
    }

    if graph.is_directed() {
        let in_edges = graph.incident_in(v)?;
        for eid in &in_edges {
            if !out_edges.contains(eid) {
                let (from, to) = graph.edge(*eid)?;
                let nei = if from == v { to } else { from };
                neighbors.push(nei);
            }
        }
    }

    Ok(neighbors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        assert!(result.types.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        assert_eq!(result.types.len(), 1);
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(5);
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        assert_eq!(result.types.len(), 5);
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        assert_ne!(result.types[0], result.types[1]);
    }

    #[test]
    fn path_graph() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        // Adjacent vertices must have different types
        for eid in 0..g.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (from, to) = g.edge(eid as u32).unwrap();
            assert_ne!(
                result.types[from as usize], result.types[to as usize],
                "edge ({from}, {to}) violates bipartiteness"
            );
        }
    }

    #[test]
    fn even_cycle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
    }

    #[test]
    fn odd_cycle_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(!result.is_bipartite);
        assert!(result.types.is_empty());
    }

    #[test]
    fn odd_cycle_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(!result.is_bipartite);
    }

    #[test]
    fn complete_bipartite_k23() {
        let mut g = Graph::with_vertices(5);
        // 0,1 on one side; 2,3,4 on the other
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        // 0 and 1 should have the same type; 2,3,4 should have the other
        assert_eq!(result.types[0], result.types[1]);
        assert_eq!(result.types[2], result.types[3]);
        assert_eq!(result.types[3], result.types[4]);
        assert_ne!(result.types[0], result.types[2]);
    }

    #[test]
    fn disconnected_bipartite() {
        let mut g = Graph::with_vertices(6);
        // Component 1: path 0-1-2
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Component 2: edge 3-4
        g.add_edge(3, 4).unwrap();
        // vertex 5 isolated
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
    }

    #[test]
    fn disconnected_not_bipartite() {
        let mut g = Graph::with_vertices(6);
        // Component 1: triangle 0-1-2
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Component 2: edge 3-4 (bipartite)
        g.add_edge(3, 4).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(!result.is_bipartite);
    }

    #[test]
    fn self_loop_not_bipartite() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(!result.is_bipartite);
    }

    #[test]
    fn directed_ignores_direction() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
    }

    #[test]
    fn k4_not_bipartite() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(!result.is_bipartite);
    }

    #[test]
    fn star_bipartite() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
        // Center (0) on one side, leaves on the other
        assert_ne!(result.types[0], result.types[1]);
        assert_eq!(result.types[1], result.types[2]);
        assert_eq!(result.types[2], result.types[3]);
        assert_eq!(result.types[3], result.types[4]);
    }

    #[test]
    fn cube_bipartite() {
        // Cube graph Q3 is bipartite
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 5).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(2, 6).unwrap();
        g.add_edge(3, 7).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(4, 6).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(6, 7).unwrap();
        let result = is_bipartite(&g).unwrap();
        assert!(result.is_bipartite);
    }
}
