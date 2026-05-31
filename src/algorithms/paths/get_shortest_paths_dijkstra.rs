//! Weighted shortest paths via Dijkstra (ALGO-SP-038).
//!
//! Counterpart of `igraph_get_shortest_paths_dijkstra()` from
//! `references/igraph/src/paths/dijkstra.c:336`.
//!
//! Returns one shortest path (vertex and edge sequences) from a source
//! vertex to every vertex in the graph, using weighted Dijkstra.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

use super::dijkstra::{DijkstraMode, DijkstraPaths, dijkstra_paths, dijkstra_paths_with_mode};

/// Result of [`get_shortest_paths_dijkstra`]: vertex and edge paths
/// from a single source to all vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortestPathsDijkstra {
    /// `vertex_paths[v]` is the vertex sequence of a shortest path from
    /// `source` to `v` (inclusive at both ends). Empty if `v` is
    /// unreachable.
    pub vertex_paths: Vec<Vec<VertexId>>,
    /// `edge_paths[v]` is the edge-id sequence of a shortest path from
    /// `source` to `v`. Empty if `v` is unreachable.
    pub edge_paths: Vec<Vec<EdgeId>>,
}

/// Returns one shortest path from `source` to every vertex in the
/// graph, using weighted edges (Dijkstra's algorithm).
///
/// For directed graphs, follows edges in the outgoing direction by
/// default. Use [`get_shortest_paths_dijkstra_with_mode`] to control
/// direction.
///
/// # Errors
///
/// - `InvalidArgument` if `source >= vcount()`, or weights are invalid
///   (wrong length, negative, or NaN).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_paths_dijkstra};
///
/// // Path 0-1-2-3, weights [1, 1, 1, 10] — last edge is a heavy shortcut.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 3).unwrap(); // heavy
/// let r = get_shortest_paths_dijkstra(&g, 0, &[1.0, 1.0, 1.0, 10.0]).unwrap();
/// assert_eq!(r.vertex_paths[3], vec![0, 1, 2, 3]);
/// assert_eq!(r.edge_paths[3], vec![0, 1, 2]);
/// ```
pub fn get_shortest_paths_dijkstra(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
) -> IgraphResult<ShortestPathsDijkstra> {
    let dp = dijkstra_paths(graph, source, weights)?;
    Ok(build_all_paths(graph, source, &dp))
}

/// Mode-aware [`get_shortest_paths_dijkstra`].
///
/// For directed graphs, `mode` controls traversal direction. For
/// undirected graphs, `mode` is ignored.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_shortest_paths_dijkstra_with_mode, DijkstraMode};
///
/// // Directed 0→1→2 with unit weights.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let r = get_shortest_paths_dijkstra_with_mode(&g, 0, &[1.0, 1.0], DijkstraMode::Out).unwrap();
/// assert_eq!(r.vertex_paths[2], vec![0, 1, 2]);
/// assert_eq!(r.edge_paths[2], vec![0, 1]);
/// ```
pub fn get_shortest_paths_dijkstra_with_mode(
    graph: &Graph,
    source: VertexId,
    weights: &[f64],
    mode: DijkstraMode,
) -> IgraphResult<ShortestPathsDijkstra> {
    let dp = dijkstra_paths_with_mode(graph, source, weights, mode)?;
    Ok(build_all_paths(graph, source, &dp))
}

fn build_all_paths(graph: &Graph, source: VertexId, dp: &DijkstraPaths) -> ShortestPathsDijkstra {
    let n = graph.vcount() as usize;
    let mut vertex_paths: Vec<Vec<VertexId>> = Vec::with_capacity(n);
    let mut edge_paths: Vec<Vec<EdgeId>> = Vec::with_capacity(n);

    for v in 0..n {
        #[allow(clippy::cast_possible_truncation)]
        let v_id = v as u32;
        if dp.distances[v].is_none() {
            vertex_paths.push(Vec::new());
            edge_paths.push(Vec::new());
            continue;
        }
        if v_id == source {
            vertex_paths.push(vec![source]);
            edge_paths.push(Vec::new());
            continue;
        }
        let mut vs = Vec::new();
        let mut es = Vec::new();
        let mut cur = v_id;
        while cur != source {
            vs.push(cur);
            if let Some(eid) = dp.inbound_edges[cur as usize] {
                es.push(eid);
            }
            match dp.parents[cur as usize] {
                Some(p) => cur = p,
                None => break,
            }
        }
        vs.push(source);
        vs.reverse();
        es.reverse();
        vertex_paths.push(vs);
        edge_paths.push(es);
    }

    ShortestPathsDijkstra {
        vertex_paths,
        edge_paths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton() {
        let g = Graph::with_vertices(1);
        let r = get_shortest_paths_dijkstra(&g, 0, &[]).unwrap();
        assert_eq!(r.vertex_paths, vec![vec![0]]);
        assert_eq!(r.edge_paths, vec![vec![]]);
    }

    #[test]
    fn path_graph_unit_weights() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_shortest_paths_dijkstra(&g, 0, &[1.0, 1.0, 1.0]).unwrap();
        assert_eq!(r.vertex_paths[0], vec![0]);
        assert_eq!(r.vertex_paths[1], vec![0, 1]);
        assert_eq!(r.vertex_paths[2], vec![0, 1, 2]);
        assert_eq!(r.vertex_paths[3], vec![0, 1, 2, 3]);
        assert_eq!(r.edge_paths[0], vec![]);
        assert_eq!(r.edge_paths[1], vec![0]);
        assert_eq!(r.edge_paths[2], vec![0, 1]);
        assert_eq!(r.edge_paths[3], vec![0, 1, 2]);
    }

    #[test]
    fn shortcut_with_high_weight() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // e0: weight 1
        g.add_edge(1, 2).unwrap(); // e1: weight 1
        g.add_edge(2, 3).unwrap(); // e2: weight 1
        g.add_edge(0, 3).unwrap(); // e3: weight 10
        let r = get_shortest_paths_dijkstra(&g, 0, &[1.0, 1.0, 1.0, 10.0]).unwrap();
        assert_eq!(r.vertex_paths[3], vec![0, 1, 2, 3]);
        assert_eq!(r.edge_paths[3], vec![0, 1, 2]);
    }

    #[test]
    fn shortcut_with_low_weight() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // e0: weight 5
        g.add_edge(1, 2).unwrap(); // e1: weight 5
        g.add_edge(2, 3).unwrap(); // e2: weight 5
        g.add_edge(0, 3).unwrap(); // e3: weight 1
        let r = get_shortest_paths_dijkstra(&g, 0, &[5.0, 5.0, 5.0, 1.0]).unwrap();
        assert_eq!(r.vertex_paths[3], vec![0, 3]);
        assert_eq!(r.edge_paths[3], vec![3]);
    }

    #[test]
    fn disconnected_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = get_shortest_paths_dijkstra(&g, 0, &[1.0, 1.0]).unwrap();
        assert_eq!(r.vertex_paths[0], vec![0]);
        assert_eq!(r.vertex_paths[1], vec![0, 1]);
        assert!(r.vertex_paths[2].is_empty());
        assert!(r.vertex_paths[3].is_empty());
        assert!(r.edge_paths[2].is_empty());
        assert!(r.edge_paths[3].is_empty());
    }

    #[test]
    fn source_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(get_shortest_paths_dijkstra(&g, 5, &[]).is_err());
    }

    #[test]
    fn directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r =
            get_shortest_paths_dijkstra_with_mode(&g, 0, &[1.0, 1.0], DijkstraMode::Out).unwrap();
        assert_eq!(r.vertex_paths[2], vec![0, 1, 2]);
        assert_eq!(r.edge_paths[2], vec![0, 1]);
    }

    #[test]
    fn directed_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r =
            get_shortest_paths_dijkstra_with_mode(&g, 2, &[1.0, 1.0], DijkstraMode::In).unwrap();
        assert_eq!(r.vertex_paths[0], vec![2, 1, 0]);
        assert_eq!(r.edge_paths[0], vec![1, 0]);
    }

    #[test]
    fn directed_unreachable() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r =
            get_shortest_paths_dijkstra_with_mode(&g, 2, &[1.0, 1.0], DijkstraMode::Out).unwrap();
        assert_eq!(r.vertex_paths[2], vec![2]);
        assert!(r.vertex_paths[0].is_empty());
        assert!(r.vertex_paths[1].is_empty());
    }

    #[test]
    fn triangle_nonuniform_weights() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // e0: weight 1
        g.add_edge(0, 2).unwrap(); // e1: weight 4
        g.add_edge(1, 2).unwrap(); // e2: weight 2
        let r = get_shortest_paths_dijkstra(&g, 0, &[1.0, 4.0, 2.0]).unwrap();
        // 0→2 directly costs 4, but 0→1→2 costs 3.
        assert_eq!(r.vertex_paths[2], vec![0, 1, 2]);
        assert_eq!(r.edge_paths[2], vec![0, 2]);
    }

    #[test]
    fn edge_path_lengths_consistent() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let r = get_shortest_paths_dijkstra(&g, 0, &[1.0; 4]).unwrap();
        for v in 0..5 {
            if r.vertex_paths[v].is_empty() {
                assert!(r.edge_paths[v].is_empty());
            } else {
                assert_eq!(
                    r.edge_paths[v].len() + 1,
                    r.vertex_paths[v].len(),
                    "edge count should be vertex count - 1 for vertex {v}"
                );
            }
        }
    }
}
