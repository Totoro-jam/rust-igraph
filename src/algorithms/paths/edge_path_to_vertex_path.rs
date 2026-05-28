//! Edge path to vertex path conversion (ALGO-SP-030).
//!
//! Counterpart of `igraph_vertex_path_from_edge_path()` from
//! `references/igraph/src/misc/other.c`.
//!
//! Converts a walk described by edge IDs into the corresponding sequence
//! of vertex IDs.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Direction mode for edge-path traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkMode {
    /// Follow outgoing edges (from → to).
    Out,
    /// Follow incoming edges (to → from).
    In,
    /// Ignore edge direction.
    All,
}

/// Convert an edge path to a vertex path.
///
/// Given a sequence of edge IDs forming a continuous walk starting at
/// `start`, returns the sequence of vertex IDs traversed. The result
/// always has `edge_path.len() + 1` elements.
///
/// If `start` is `None`, the start vertex is inferred from the first
/// edge (requires at least one edge). For directed graphs with `WalkMode::Out`,
/// the source of the first edge is used; for `In`, the target is used.
/// For undirected graphs or `All` mode with multiple edges, the vertex
/// connecting the first two edges is determined.
///
/// The `mode` parameter is ignored for undirected graphs (treated as `All`).
///
/// # Errors
///
/// - `InvalidArgument` if `start` is out of range.
/// - `InvalidArgument` if `start` is `None` and `edge_path` is empty.
/// - `InvalidArgument` if the edge IDs do not form a continuous path.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_path_from_edge_path, WalkMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // eid 0
/// g.add_edge(1, 2).unwrap(); // eid 1
/// g.add_edge(2, 3).unwrap(); // eid 2
///
/// let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 1, 2], WalkMode::All).unwrap();
/// assert_eq!(vpath, vec![0, 1, 2, 3]);
/// ```
pub fn vertex_path_from_edge_path(
    graph: &Graph,
    start: Option<u32>,
    edge_path: &[u32],
    mode: WalkMode,
) -> IgraphResult<Vec<u32>> {
    let effective_mode = if graph.is_directed() {
        mode
    } else {
        WalkMode::All
    };

    let mut current = if let Some(v) = start {
        if v >= graph.vcount() {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex_path_from_edge_path: start vertex {v} out of range"
            )));
        }
        v
    } else {
        if edge_path.is_empty() {
            return Err(IgraphError::InvalidArgument(
                "vertex_path_from_edge_path: cannot infer start vertex from empty edge path".into(),
            ));
        }
        infer_start(graph, edge_path, effective_mode)?
    };

    let mut vertex_path: Vec<u32> = Vec::with_capacity(edge_path.len().saturating_add(1));

    for (i, &eid) in edge_path.iter().enumerate() {
        let (from, to) = graph.edge(eid)?;
        vertex_path.push(current);

        let (ok, next) = match effective_mode {
            WalkMode::Out => (from == current, to),
            WalkMode::In => (to == current, from),
            WalkMode::All => {
                if from == current {
                    (true, to)
                } else if to == current {
                    (true, from)
                } else {
                    (false, 0)
                }
            }
        };

        if !ok {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex_path_from_edge_path: edge {eid} at position {i} does not connect to \
                 current vertex {current}"
            )));
        }

        current = next;
    }

    vertex_path.push(current);
    Ok(vertex_path)
}

fn infer_start(graph: &Graph, edge_path: &[u32], mode: WalkMode) -> IgraphResult<u32> {
    let (from, to) = graph.edge(edge_path[0])?;
    match mode {
        WalkMode::Out => Ok(from),
        WalkMode::In => Ok(to),
        WalkMode::All => {
            if edge_path.len() > 1 {
                let (from2, to2) = graph.edge(edge_path[1])?;
                if to == from2 || to == to2 {
                    Ok(from)
                } else {
                    Ok(to)
                }
            } else {
                Ok(from)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_vertices(n);
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn empty_edge_path_with_start() {
        let g = make_undirected(3, &[(0, 1), (1, 2)]);
        let vpath = vertex_path_from_edge_path(&g, Some(1), &[], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![1]);
    }

    #[test]
    fn empty_edge_path_no_start_error() {
        let g = make_undirected(3, &[(0, 1), (1, 2)]);
        let err = vertex_path_from_edge_path(&g, None, &[], WalkMode::All).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn simple_path_undirected() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 1, 2], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![0, 1, 2, 3]);
    }

    #[test]
    fn reverse_path_undirected() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let vpath = vertex_path_from_edge_path(&g, Some(3), &[2, 1, 0], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![3, 2, 1, 0]);
    }

    #[test]
    fn directed_out_mode() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(1, 2).unwrap(); // eid 1
        g.add_edge(2, 3).unwrap(); // eid 2
        let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 1, 2], WalkMode::Out).unwrap();
        assert_eq!(vpath, vec![0, 1, 2, 3]);
    }

    #[test]
    fn directed_in_mode() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(1, 2).unwrap(); // eid 1
        g.add_edge(2, 3).unwrap(); // eid 2
        let vpath = vertex_path_from_edge_path(&g, Some(3), &[2, 1, 0], WalkMode::In).unwrap();
        assert_eq!(vpath, vec![3, 2, 1, 0]);
    }

    #[test]
    fn directed_discontinuous_error() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(2, 3).unwrap(); // eid 1
        let err = vertex_path_from_edge_path(&g, Some(0), &[0, 1], WalkMode::Out).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn infer_start_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let vpath = vertex_path_from_edge_path(&g, None, &[0, 1], WalkMode::Out).unwrap();
        assert_eq!(vpath, vec![0, 1, 2]);
    }

    #[test]
    fn infer_start_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let vpath = vertex_path_from_edge_path(&g, None, &[1, 0], WalkMode::In).unwrap();
        assert_eq!(vpath, vec![2, 1, 0]);
    }

    #[test]
    fn infer_start_all_multiple_edges() {
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let vpath = vertex_path_from_edge_path(&g, None, &[0, 1, 2], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![0, 1, 2, 3]);
    }

    #[test]
    fn infer_start_all_single_edge() {
        let g = make_undirected(2, &[(0, 1)]);
        let vpath = vertex_path_from_edge_path(&g, None, &[0], WalkMode::All).unwrap();
        // Should pick from (0) as start
        assert_eq!(vpath.len(), 2);
        assert!(vpath == vec![0, 1] || vpath == vec![1, 0]);
    }

    #[test]
    fn cycle_walk() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (2, 0)]);
        let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 1, 2], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![0, 1, 2, 0]);
    }

    #[test]
    fn start_vertex_out_of_range() {
        let g = make_undirected(3, &[(0, 1)]);
        let err = vertex_path_from_edge_path(&g, Some(10), &[0], WalkMode::All).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn walk_with_repeated_vertex() {
        // Walk: 0→1→0→1 via edges 0, 0, 0
        let g = make_undirected(2, &[(0, 1)]);
        let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 0, 0], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![0, 1, 0, 1]);
    }

    #[test]
    fn directed_all_mode_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(2, 1).unwrap(); // eid 1: 2→1, so traversing 1→2 in ALL mode
        let vpath = vertex_path_from_edge_path(&g, Some(0), &[0, 1], WalkMode::All).unwrap();
        assert_eq!(vpath, vec![0, 1, 2]);
    }
}
