//! Degree sequence and extremal degree queries (ALGO-PR-036).
//!
//! Batch computation of vertex degrees and max/min degree queries.
//! Counterpart of `igraph_degree`, `igraph_maxdegree`, `igraph_strength`
//! (the unweighted part).

use crate::core::{Graph, IgraphResult, VertexId};

/// Direction mode for degree computation in directed graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegreeMode {
    /// Out-degree (number of outgoing edges).
    Out,
    /// In-degree (number of incoming edges).
    In,
    /// Total degree (out + in; for undirected graphs, same as either).
    All,
}

/// Returns the degree sequence of the graph.
///
/// For undirected graphs, `mode` is ignored. For directed graphs:
/// - `Out`: count outgoing edges per vertex.
/// - `In`: count incoming edges per vertex.
/// - `All`: count all incident edges per vertex.
///
/// Self-loops contribute 2 to a vertex's degree in `All` mode for
/// undirected graphs (matching igraph C `IGRAPH_LOOPS_TWICE` convention),
/// and 1 each to in-degree and out-degree for directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_sequence, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let deg = degree_sequence(&g, DegreeMode::All).unwrap();
/// assert_eq!(deg, vec![3, 1, 1, 1]);
///
/// // Directed graph
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let out_deg = degree_sequence(&g, DegreeMode::Out).unwrap();
/// assert_eq!(out_deg, vec![2, 1, 0]);
/// let in_deg = degree_sequence(&g, DegreeMode::In).unwrap();
/// assert_eq!(in_deg, vec![0, 1, 2]);
/// ```
pub fn degree_sequence(graph: &Graph, mode: DegreeMode) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut degrees = vec![0u32; n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;

        if graph.is_directed() {
            match mode {
                DegreeMode::Out => {
                    degrees[src as usize] += 1;
                }
                DegreeMode::In => {
                    degrees[tgt as usize] += 1;
                }
                DegreeMode::All => {
                    degrees[src as usize] += 1;
                    if src != tgt {
                        degrees[tgt as usize] += 1;
                    }
                }
            }
        } else {
            degrees[src as usize] += 1;
            degrees[tgt as usize] += 1;
        }
    }

    Ok(degrees)
}

/// Returns the maximum degree in the graph.
///
/// Returns 0 for empty graphs (no vertices). For directed graphs,
/// uses the specified `mode`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, max_degree, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert_eq!(max_degree(&g, DegreeMode::All).unwrap(), 3);
/// ```
pub fn max_degree(graph: &Graph, mode: DegreeMode) -> IgraphResult<u32> {
    let degrees = degree_sequence(graph, mode)?;
    Ok(degrees.into_iter().max().unwrap_or(0))
}

/// Returns the minimum degree in the graph.
///
/// Returns 0 for empty graphs (no vertices). For directed graphs,
/// uses the specified `mode`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, min_degree, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert_eq!(min_degree(&g, DegreeMode::All).unwrap(), 1);
/// ```
pub fn min_degree(graph: &Graph, mode: DegreeMode) -> IgraphResult<u32> {
    let degrees = degree_sequence(graph, mode)?;
    Ok(degrees.into_iter().min().unwrap_or(0))
}

/// Returns the vertex (or one of the vertices) with the maximum degree.
///
/// Returns `None` for an empty graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, max_degree_vertex, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// assert_eq!(max_degree_vertex(&g, DegreeMode::All).unwrap(), Some(0));
/// ```
pub fn max_degree_vertex(graph: &Graph, mode: DegreeMode) -> IgraphResult<Option<VertexId>> {
    let degrees = degree_sequence(graph, mode)?;
    Ok(degrees
        .iter()
        .enumerate()
        .max_by_key(|(_, d)| *d)
        .map(|(i, _)| {
            #[allow(clippy::cast_possible_truncation)]
            let v = i as u32;
            v
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert!(deg.is_empty());
        assert_eq!(max_degree(&g, DegreeMode::All).unwrap(), 0);
        assert_eq!(min_degree(&g, DegreeMode::All).unwrap(), 0);
    }

    #[test]
    fn test_isolated_vertices() {
        let g = Graph::with_vertices(5);
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert_eq!(deg, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_undirected_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert_eq!(deg, vec![1, 2, 2, 1]);
    }

    #[test]
    fn test_undirected_star() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert_eq!(deg, vec![4, 1, 1, 1, 1]);
        assert_eq!(max_degree(&g, DegreeMode::All).unwrap(), 4);
        assert_eq!(min_degree(&g, DegreeMode::All).unwrap(), 1);
    }

    #[test]
    fn test_directed_out() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let deg = degree_sequence(&g, DegreeMode::Out).unwrap();
        assert_eq!(deg, vec![2, 1, 0]);
    }

    #[test]
    fn test_directed_in() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let deg = degree_sequence(&g, DegreeMode::In).unwrap();
        assert_eq!(deg, vec![0, 1, 2]);
    }

    #[test]
    fn test_directed_all() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert_eq!(deg, vec![2, 2, 2]);
    }

    #[test]
    fn test_self_loop_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        // Self-loop contributes 2 to vertex 0
        assert_eq!(deg, vec![3, 1]);
    }

    #[test]
    fn test_self_loop_directed_out() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let deg = degree_sequence(&g, DegreeMode::Out).unwrap();
        assert_eq!(deg, vec![2, 0]);
    }

    #[test]
    fn test_self_loop_directed_in() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let deg = degree_sequence(&g, DegreeMode::In).unwrap();
        assert_eq!(deg, vec![1, 1]);
    }

    #[test]
    fn test_self_loop_directed_all() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        // Self-loop: src == tgt, only counts once in All mode
        assert_eq!(deg, vec![2, 1]);
    }

    #[test]
    fn test_max_degree_vertex_basic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(max_degree_vertex(&g, DegreeMode::All).unwrap(), Some(0));
    }

    #[test]
    fn test_max_degree_vertex_empty() {
        let g = Graph::with_vertices(0);
        assert_eq!(max_degree_vertex(&g, DegreeMode::All).unwrap(), None);
    }

    #[test]
    fn test_complete_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert!(deg.iter().all(|&d| d == 4));
        assert_eq!(max_degree(&g, DegreeMode::All).unwrap(), 4);
        assert_eq!(min_degree(&g, DegreeMode::All).unwrap(), 4);
    }

    #[test]
    fn test_multi_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let deg = degree_sequence(&g, DegreeMode::All).unwrap();
        assert_eq!(deg, vec![3, 3]);
    }
}
