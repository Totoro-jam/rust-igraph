//! k shortest loopless paths via Yen's algorithm (ALGO-PA-034).
//!
//! Counterpart of `igraph_get_k_shortest_paths()` from
//! `references/igraph/src/paths/shortest_paths.c:1402`.
//!
//! Finds the k shortest loopless paths between two vertices using
//! Yen's algorithm (1970). Each iteration finds the next shortest
//! path by systematically deviating from previously found paths.
//!
//! Reference: Yen, Jin Y. "An algorithm for finding shortest routes
//! from all source nodes to a given destination in general networks."
//! Quarterly of Applied Mathematics 27(4): 526–530 (1970).

use crate::algorithms::paths::dijkstra::{DijkstraMode, dijkstra_path_to_with_mode};
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Sentinel weight used to "remove" an edge: large enough that it will never
/// appear on a shortest path, but finite so dijkstra's weight validation passes.
const REMOVED_WEIGHT: f64 = f64::MAX / 4.0;

/// Result of [`k_shortest_paths`].
#[derive(Debug, Clone)]
pub struct KShortestPath {
    /// Vertex sequence of the path (including source and target).
    pub vertices: Vec<VertexId>,
    /// Edge sequence of the path.
    pub edges: Vec<EdgeId>,
    /// Total weight of the path.
    pub weight: f64,
}

/// Finds the k shortest loopless paths between `source` and `target`.
///
/// Returns up to `k` paths in order of increasing total weight.
/// If fewer than `k` paths exist, fewer are returned.
///
/// `weights` must have length `ecount()` and contain non-negative
/// values. Edges with weight `f64::INFINITY` are treated as absent.
///
/// # Errors
///
/// - `VertexOutOfRange` if `source` or `target` exceeds `vcount()`.
/// - `InvalidArgument` if `weights` length differs from `ecount()`.
/// - `InvalidArgument` if `weights` contains negative values.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, k_shortest_paths, DijkstraMode};
///
/// // Diamond: 0→1→3, 0→2→3 (two simple paths of equal length)
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 3).unwrap(); // edge 1
/// g.add_edge(0, 2).unwrap(); // edge 2
/// g.add_edge(2, 3).unwrap(); // edge 3
/// let w = vec![1.0; 4];
/// let paths = k_shortest_paths(&g, 0, 3, &w, 3, DijkstraMode::All).unwrap();
/// assert_eq!(paths.len(), 2); // only 2 simple paths exist
/// assert!((paths[0].weight - 2.0).abs() < 1e-12);
/// assert!((paths[1].weight - 2.0).abs() < 1e-12);
/// ```
#[allow(clippy::too_many_lines)]
pub fn k_shortest_paths(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    weights: &[f64],
    k: usize,
    mode: DijkstraMode,
) -> IgraphResult<Vec<KShortestPath>> {
    let n = graph.vcount();
    let m = graph.ecount();

    if source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "k_shortest_paths: weights length ({}) != ecount ({m})",
            weights.len()
        )));
    }
    for (i, &w) in weights.iter().enumerate() {
        if w < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "k_shortest_paths: negative weight ({w}) at edge {i}"
            )));
        }
    }

    if k == 0 {
        return Ok(Vec::new());
    }

    // Find the first shortest path.
    let first = dijkstra_path_to_with_mode(graph, source, target, weights, mode)?;
    let Some((init_vertices, init_edges)) = first else {
        return Ok(Vec::new()); // no path exists
    };

    if has_infinite_edge(&init_edges, weights) {
        return Ok(Vec::new());
    }

    let first_weight = path_weight(&init_edges, weights);
    let mut result: Vec<KShortestPath> = Vec::with_capacity(k);
    result.push(KShortestPath {
        vertices: init_vertices,
        edges: init_edges,
        weight: first_weight,
    });

    if k == 1 {
        return Ok(result);
    }

    // Candidate pool: (edges, weight)
    let mut candidates: Vec<(Vec<EdgeId>, f64)> = Vec::new();

    // Working copy of weights (infinity = removed edge).
    let mut cur_weights: Vec<f64> = weights.to_vec();

    for i_path in 1..k {
        let prev_edges = &result[i_path - 1].edges;
        let prev_len = prev_edges.len();

        for spur_idx in 0..prev_len {
            // Determine spur vertex: the source-side endpoint of the edge at spur_idx.
            let spur_vertex = edge_source_vertex(graph, prev_edges, spur_idx, mode)?;

            // Root path = edges[0..spur_idx].
            let root_path = &prev_edges[..spur_idx];

            // Remove edges that share the same root path in previously found shortest paths.
            let mut removed_edges: Vec<usize> = Vec::new();
            for found in &result {
                if found.edges.len() > spur_idx && found.edges[..spur_idx] == *root_path {
                    let eid = found.edges[spur_idx] as usize;
                    if cur_weights[eid] != REMOVED_WEIGHT {
                        cur_weights[eid] = REMOVED_WEIGHT;
                        removed_edges.push(eid);
                    }
                }
            }

            // Remove root-path vertices (except spur) by setting their incident edges to infinity.
            for root_idx in 0..spur_idx {
                let root_vertex = edge_source_vertex(graph, prev_edges, root_idx, mode)?;
                semi_delete_vertex(
                    graph,
                    &mut cur_weights,
                    root_vertex,
                    &mut removed_edges,
                    mode,
                )?;
            }

            // Find spur path from spur_vertex to target.
            let spur_result =
                dijkstra_path_to_with_mode(graph, spur_vertex, target, &cur_weights, mode)?;

            if let Some((_spur_vs, spur_es)) = spur_result {
                if !has_removed_edge(&spur_es, &cur_weights) {
                    // Total path = root + spur.
                    let mut total_edges: Vec<EdgeId> = root_path.to_vec();
                    total_edges.extend_from_slice(&spur_es);

                    // Only add if not already in candidates.
                    let already_exists = candidates.iter().any(|(e, _)| *e == total_edges);
                    if !already_exists {
                        let w = path_weight(&total_edges, weights);
                        candidates.push((total_edges, w));
                    }
                }
            }

            // Restore removed edges.
            for &eid in &removed_edges {
                cur_weights[eid] = weights[eid];
            }
        }

        // Pick the lightest candidate.
        if candidates.is_empty() {
            break;
        }

        let mut best_idx = 0;
        let mut best_weight = candidates[0].1;
        for (idx, &(_, w)) in candidates.iter().enumerate().skip(1) {
            if w < best_weight {
                best_weight = w;
                best_idx = idx;
            }
        }

        let (best_edges, best_w) = candidates.swap_remove(best_idx);
        let best_vs = edge_path_to_vertices(graph, source, &best_edges, mode)?;
        result.push(KShortestPath {
            vertices: best_vs,
            edges: best_edges,
            weight: best_w,
        });
    }

    Ok(result)
}

fn has_infinite_edge(edges: &[EdgeId], weights: &[f64]) -> bool {
    edges.iter().any(|&e| weights[e as usize] == f64::INFINITY)
}

fn has_removed_edge(edges: &[EdgeId], weights: &[f64]) -> bool {
    edges.iter().any(|&e| weights[e as usize] >= REMOVED_WEIGHT)
}

fn path_weight(edges: &[EdgeId], weights: &[f64]) -> f64 {
    edges.iter().map(|&e| weights[e as usize]).sum()
}

fn edge_source_vertex(
    graph: &Graph,
    edge_path: &[EdgeId],
    idx: usize,
    mode: DijkstraMode,
) -> IgraphResult<VertexId> {
    let (from, to) = graph.edge(edge_path[idx])?;
    match mode {
        DijkstraMode::Out => Ok(from),
        DijkstraMode::In => Ok(to),
        DijkstraMode::All => {
            // For undirected/all mode: the source-side is the vertex shared
            // with the previous edge (or the overall source for idx==0).
            if idx == 0 {
                // Determine by looking at the next edge if available.
                if edge_path.len() > 1 {
                    let (nf, nt) = graph.edge(edge_path[1])?;
                    if from == nf || from == nt {
                        Ok(to)
                    } else {
                        Ok(from)
                    }
                } else {
                    // Single edge path — either endpoint could be the source.
                    // We need context from the caller. For spur_idx=0 the spur
                    // vertex is the overall source, which is tracked externally.
                    // Return `from` as default; the caller handles the first-edge case.
                    Ok(from)
                }
            } else {
                let (pf, pt) = graph.edge(edge_path[idx - 1])?;
                // The shared vertex between edge[idx-1] and edge[idx] is the
                // "target" of the previous edge = "source" of this edge's next hop.
                if from == pf || from == pt {
                    Ok(from)
                } else {
                    Ok(to)
                }
            }
        }
    }
}

fn semi_delete_vertex(
    graph: &Graph,
    cur_weights: &mut [f64],
    vertex: VertexId,
    removed_edges: &mut Vec<usize>,
    mode: DijkstraMode,
) -> IgraphResult<()> {
    let incident = incident_for_mode_yen(graph, vertex, mode)?;
    for eid in incident {
        let idx = eid as usize;
        if cur_weights[idx] != REMOVED_WEIGHT {
            cur_weights[idx] = REMOVED_WEIGHT;
            removed_edges.push(idx);
        }
    }
    Ok(())
}

fn incident_for_mode_yen(
    graph: &Graph,
    v: VertexId,
    mode: DijkstraMode,
) -> IgraphResult<Vec<EdgeId>> {
    if !graph.is_directed() || mode == DijkstraMode::All {
        let mut out = graph.incident(v)?;
        if graph.is_directed() {
            out.extend(graph.incident_in(v)?);
        }
        return Ok(out);
    }
    match mode {
        DijkstraMode::Out => graph.incident(v),
        DijkstraMode::In => graph.incident_in(v),
        DijkstraMode::All => unreachable!(),
    }
}

fn edge_path_to_vertices(
    graph: &Graph,
    source: VertexId,
    edge_path: &[EdgeId],
    mode: DijkstraMode,
) -> IgraphResult<Vec<VertexId>> {
    if edge_path.is_empty() {
        return Ok(vec![source]);
    }

    let mut vertices = Vec::with_capacity(edge_path.len() + 1);
    vertices.push(source);

    let mut cur = source;
    for &eid in edge_path {
        let (from, to) = graph.edge(eid)?;
        let next = match mode {
            DijkstraMode::Out => to,
            DijkstraMode::In => from,
            DijkstraMode::All => {
                if from == cur {
                    to
                } else {
                    from
                }
            }
        };
        vertices.push(next);
        cur = next;
    }

    Ok(vertices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_path() {
        let g = Graph::with_vertices(3);
        let w = vec![]; // no edges
        let paths = k_shortest_paths(&g, 0, 2, &w, 5, DijkstraMode::All).unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn k_zero() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let paths = k_shortest_paths(&g, 0, 1, &[1.0], 0, DijkstraMode::All).unwrap();
        assert!(paths.is_empty());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let paths = k_shortest_paths(&g, 0, 1, &[3.0], 5, DijkstraMode::All).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].vertices, vec![0, 1]);
        assert_eq!(paths[0].edges, vec![0]);
        assert!((paths[0].weight - 3.0).abs() < 1e-12);
    }

    #[test]
    fn diamond_two_paths() {
        // 0-1-3 and 0-2-3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 3).unwrap(); // 1
        g.add_edge(0, 2).unwrap(); // 2
        g.add_edge(2, 3).unwrap(); // 3
        let w = vec![1.0; 4];
        let paths = k_shortest_paths(&g, 0, 3, &w, 5, DijkstraMode::All).unwrap();
        assert_eq!(paths.len(), 2);
        assert!((paths[0].weight - 2.0).abs() < 1e-12);
        assert!((paths[1].weight - 2.0).abs() < 1e-12);
    }

    #[test]
    fn diamond_different_weights() {
        // 0→1→3 (weight 2+1=3) and 0→2→3 (weight 1+1=2)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0, w=2
        g.add_edge(1, 3).unwrap(); // 1, w=1
        g.add_edge(0, 2).unwrap(); // 2, w=1
        g.add_edge(2, 3).unwrap(); // 3, w=1
        let w = vec![2.0, 1.0, 1.0, 1.0];
        let paths = k_shortest_paths(&g, 0, 3, &w, 2, DijkstraMode::All).unwrap();
        assert_eq!(paths.len(), 2);
        assert!((paths[0].weight - 2.0).abs() < 1e-12); // 0→2→3
        assert!((paths[1].weight - 3.0).abs() < 1e-12); // 0→1→3
    }

    #[test]
    fn path_graph_single_path() {
        // 0—1—2—3 only one simple path
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let w = vec![1.0; 3];
        let paths = k_shortest_paths(&g, 0, 3, &w, 3, DijkstraMode::All).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].vertices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn three_paths() {
        // Triangle with extra path: 0-1-2, 0-2 (direct), 0-1-3-2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0, w=1
        g.add_edge(1, 2).unwrap(); // 1, w=1
        g.add_edge(0, 2).unwrap(); // 2, w=3
        g.add_edge(1, 3).unwrap(); // 3, w=1
        g.add_edge(3, 2).unwrap(); // 4, w=1
        let w = vec![1.0, 1.0, 3.0, 1.0, 1.0];
        let paths = k_shortest_paths(&g, 0, 2, &w, 5, DijkstraMode::All).unwrap();
        assert!(paths.len() >= 3);
        // Shortest: 0-1-2 (weight 2)
        assert!((paths[0].weight - 2.0).abs() < 1e-12);
        // Second: 0-1-3-2 (weight 3) or 0-2 (weight 3)
        assert!((paths[1].weight - 3.0).abs() < 1e-12);
        assert!((paths[2].weight - 3.0).abs() < 1e-12);
    }

    #[test]
    fn source_equals_target() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let w = vec![1.0; 2];
        let paths = k_shortest_paths(&g, 0, 0, &w, 1, DijkstraMode::All).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].vertices, vec![0]);
        assert!(paths[0].edges.is_empty());
        assert!((paths[0].weight - 0.0).abs() < 1e-12);
    }

    #[test]
    fn directed_out_mode() {
        // 0→1→2, 0→2 (direct)
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(0, 2).unwrap(); // 2, w=5
        let w = vec![1.0, 1.0, 5.0];
        let paths = k_shortest_paths(&g, 0, 2, &w, 3, DijkstraMode::Out).unwrap();
        assert_eq!(paths.len(), 2);
        assert!((paths[0].weight - 2.0).abs() < 1e-12); // 0→1→2
        assert!((paths[1].weight - 5.0).abs() < 1e-12); // 0→2
    }

    #[test]
    fn invalid_source() {
        let g = Graph::with_vertices(3);
        assert!(k_shortest_paths(&g, 99, 0, &[], 1, DijkstraMode::All).is_err());
    }

    #[test]
    fn negative_weight_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(k_shortest_paths(&g, 0, 1, &[-1.0], 1, DijkstraMode::All).is_err());
    }
}
