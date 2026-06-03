//! Line graph construction (ALGO-OP-028).
//!
//! Counterpart of `igraph_linegraph()` from
//! `references/igraph/src/operators/misc_internal.c`.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Construct the line graph L(G) of the input graph.
///
/// The line graph has one vertex for each edge of the original graph.
/// Two vertices of L(G) are adjacent if and only if the corresponding
/// edges in G share an endpoint.
///
/// For directed graphs, edge (u→v) is adjacent to edge (w→x) in the
/// line graph iff v == w (the head of the first matches the tail of
/// the second), matching igraph's upstream convention.
///
/// Counterpart of `igraph_linegraph()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, line_graph};
///
/// // Triangle 0-1-2-0 has 3 edges; L(G) is also a triangle.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
///
/// let lg = line_graph(&g).unwrap();
/// assert_eq!(lg.vcount(), 3);
/// assert_eq!(lg.ecount(), 3);
/// ```
///
/// ```
/// use rust_igraph::{Graph, line_graph};
///
/// // Path 0-1-2 has 2 edges; L(G) is a single edge.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let lg = line_graph(&g).unwrap();
/// assert_eq!(lg.vcount(), 2);
/// assert_eq!(lg.ecount(), 1);
/// ```
pub fn line_graph(graph: &Graph) -> IgraphResult<Graph> {
    let m = graph.ecount();
    let directed = graph.is_directed();
    let m_u32 = u32::try_from(m).map_err(|_| {
        crate::core::IgraphError::Internal("edge count exceeds u32::MAX for line graph")
    })?;
    let mut result = Graph::new(m_u32, directed)?;

    if m == 0 {
        return Ok(result);
    }

    let n = graph.vcount();

    if directed {
        // For directed: edge i (u→v) connects to edge j (w→x) in L(G) iff v == w.
        // For each vertex v, every incoming edge connects to every outgoing edge.
        for v in 0..n {
            let in_eids = graph.incident_in(v)?;
            let out_eids = graph.incident(v)?;

            for &in_eid in &in_eids {
                for &out_eid in &out_eids {
                    result.add_edge(in_eid as VertexId, out_eid as VertexId)?;
                }
            }
        }
    } else {
        // For undirected: two edges are adjacent in L(G) iff they share
        // a vertex. For each vertex v, all edges incident to v form a
        // clique in L(G).
        for v in 0..n {
            let incident = incident_unique(graph, v)?;
            let len = incident.len();
            for i in 0..len {
                for j in (i + 1)..len {
                    result.add_edge(incident[i] as VertexId, incident[j] as VertexId)?;
                }
            }
        }
    }

    Ok(result)
}

/// All edge IDs incident to vertex v, deduplicated (self-loops only once).
fn incident_unique(graph: &Graph, v: VertexId) -> IgraphResult<Vec<EdgeId>> {
    let mut ids = graph.incident(v)?;
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_line_graph_is_empty() {
        let g = Graph::with_vertices(5);
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 0);
        assert_eq!(lg.ecount(), 0);
    }

    #[test]
    fn single_edge_line_graph_is_single_vertex() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 1);
        assert_eq!(lg.ecount(), 0);
    }

    #[test]
    fn triangle_line_graph_is_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 3);
        assert_eq!(lg.ecount(), 3);
    }

    #[test]
    fn path_line_graph() {
        // Path 0-1-2-3 has 3 edges. L(G) is a path of length 2.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 3);
        assert_eq!(lg.ecount(), 2);
    }

    #[test]
    fn star_line_graph_is_complete() {
        // Star K_{1,4}: center 0 connected to 1,2,3,4.
        // 4 edges, all share vertex 0 → L(G) = K_4 with 6 edges.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 4);
        assert_eq!(lg.ecount(), 6);
    }

    #[test]
    fn directed_path_line_graph() {
        // 0 -> 1 -> 2: two edges. Edge 0 (0→1) connects to edge 1 (1→2)
        // because target of edge 0 == source of edge 1.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 2);
        assert_eq!(lg.ecount(), 1);
        assert!(lg.is_directed());
    }

    #[test]
    fn directed_divergent_edges_not_connected() {
        // 0 -> 1, 0 -> 2: both share source 0, but in the directed line
        // graph, edge i→j connects to edge k→l only if j == k. Here
        // targets are 1 and 2, sources are both 0. No connection.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 2);
        assert_eq!(lg.ecount(), 0);
    }

    #[test]
    fn directed_convergent_edges_not_connected() {
        // 1 -> 0, 2 -> 0: edges share target 0. For connection we need
        // target of one == source of other. Target = 0, but sources are 1,2.
        // No connection in directed line graph.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 0).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 2);
        assert_eq!(lg.ecount(), 0);
    }

    #[test]
    fn self_loop_undirected() {
        // Self-loop on vertex 0 plus edge 0-1.
        // Edge 0: self-loop on 0, Edge 1: 0-1.
        // Both incident to vertex 0 → connected in L(G).
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let lg = line_graph(&g).unwrap();
        assert_eq!(lg.vcount(), 2);
        assert_eq!(lg.ecount(), 1);
    }
}
