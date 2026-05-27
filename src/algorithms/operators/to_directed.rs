//! Undirected → directed conversion (ALGO-OP-021).
//!
//! Converts an undirected graph to a directed graph using various edge
//! conversion modes. Counterpart of `igraph_to_directed`.

use crate::core::{Graph, IgraphResult};

/// Mode for converting an undirected graph to directed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToDirectedMode {
    /// Each undirected edge becomes two directed edges: u→v and v→u.
    Mutual,
    /// Each undirected edge is assigned an arbitrary direction (smaller
    /// vertex → larger vertex, matching igraph C's canonical order).
    Arbitrary,
}

/// Converts an undirected graph to a directed graph.
///
/// If the graph is already directed, returns a copy.
///
/// # Modes
///
/// - `Mutual`: each undirected edge {u,v} becomes two directed edges
///   u→v and v→u.
/// - `Arbitrary`: each undirected edge {u,v} becomes a single directed
///   edge from min(u,v) → max(u,v).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, to_directed, ToDirectedMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// // Mutual: each edge becomes two directed edges
/// let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
/// assert!(d.is_directed());
/// assert_eq!(d.ecount(), 4); // 2 edges × 2 directions
///
/// // Arbitrary: each edge gets one direction (canonical)
/// let d = to_directed(&g, ToDirectedMode::Arbitrary).unwrap();
/// assert!(d.is_directed());
/// assert_eq!(d.ecount(), 2);
/// ```
pub fn to_directed(graph: &Graph, mode: ToDirectedMode) -> IgraphResult<Graph> {
    let n = graph.vcount();

    if graph.is_directed() {
        // Already directed — clone the structure
        let mut result = Graph::new(n, true)?;
        let ecount = graph.ecount();
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (src, tgt) = graph.edge(eid as u32)?;
            result.add_edge(src, tgt)?;
        }
        return Ok(result);
    }

    let mut result = Graph::new(n, true)?;
    let ecount = graph.ecount();

    match mode {
        ToDirectedMode::Mutual => {
            for eid in 0..ecount {
                #[allow(clippy::cast_possible_truncation)]
                let (src, tgt) = graph.edge(eid as u32)?;
                result.add_edge(src, tgt)?;
                if src != tgt {
                    result.add_edge(tgt, src)?;
                }
            }
        }
        ToDirectedMode::Arbitrary => {
            for eid in 0..ecount {
                #[allow(clippy::cast_possible_truncation)]
                let (src, tgt) = graph.edge(eid as u32)?;
                // Undirected graphs store (min, max) canonically
                result.add_edge(src, tgt)?;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_directed() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert!(d.is_directed());
        assert_eq!(d.vcount(), 3);
        assert_eq!(d.ecount(), 2);
    }

    #[test]
    fn test_mutual_mode_single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert!(d.is_directed());
        assert_eq!(d.ecount(), 2);
    }

    #[test]
    fn test_mutual_mode_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert_eq!(d.ecount(), 6); // 3 edges × 2 directions
    }

    #[test]
    fn test_arbitrary_mode_single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let d = to_directed(&g, ToDirectedMode::Arbitrary).unwrap();
        assert!(d.is_directed());
        assert_eq!(d.ecount(), 1);
    }

    #[test]
    fn test_arbitrary_mode_path() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = to_directed(&g, ToDirectedMode::Arbitrary).unwrap();
        assert_eq!(d.ecount(), 3);
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert!(d.is_directed());
        assert_eq!(d.vcount(), 0);
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(5);
        let d = to_directed(&g, ToDirectedMode::Arbitrary).unwrap();
        assert_eq!(d.vcount(), 5);
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn test_self_loop_mutual() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        // Self-loop: only one direction (src==tgt), regular edge: two
        assert_eq!(d.ecount(), 3);
    }

    #[test]
    fn test_self_loop_arbitrary() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let d = to_directed(&g, ToDirectedMode::Arbitrary).unwrap();
        assert_eq!(d.ecount(), 2);
    }

    #[test]
    fn test_vcount_preserved() {
        let mut g = Graph::with_vertices(10);
        g.add_edge(3, 7).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert_eq!(d.vcount(), 10);
    }

    #[test]
    fn test_mutual_then_to_undirected_roundtrip() {
        // to_directed(Mutual) then to_undirected(Collapse) should recover
        // the same edge set
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let d = to_directed(&g, ToDirectedMode::Mutual).unwrap();
        assert_eq!(d.ecount(), 6);

        // Collapse should give back 3 edges
        let u = super::super::to_undirected::to_undirected(
            &d,
            super::super::to_undirected::ToUndirectedMode::Collapse,
        )
        .unwrap();
        assert_eq!(u.ecount(), 3);
    }
}
