//! Directed → undirected conversion (ALGO-OP-020).
//!
//! Converts a directed graph to an undirected graph using various edge
//! preservation modes. Counterpart of `igraph_to_undirected`.

use crate::core::{Graph, IgraphResult};

/// Mode for converting a directed graph to undirected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToUndirectedMode {
    /// Each directed edge becomes an undirected edge.
    /// Parallel directed edges (same direction) become parallel undirected edges.
    Each,
    /// Collapse mutual edges: if both (u,v) and (v,u) exist, produce one
    /// undirected edge. Non-mutual directed edges are also kept as one
    /// undirected edge.
    Collapse,
    /// Keep only mutual edges: an undirected edge (u,v) is produced only if
    /// both (u,v) and (v,u) exist in the directed graph.
    Mutual,
}

/// Converts a directed graph to an undirected graph.
///
/// If the graph is already undirected, returns a copy.
///
/// # Modes
///
/// - `Each`: every directed edge becomes an undirected edge. A pair of
///   mutual edges (u→v and v→u) becomes two parallel undirected edges.
/// - `Collapse`: each pair of vertices gets at most one undirected edge
///   regardless of how many directed edges connect them.
/// - `Mutual`: only vertex pairs connected by mutual edges (both u→v and
///   v→u) get an undirected edge.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, to_undirected, ToUndirectedMode};
///
/// // Mutual pair collapses to one edge
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let u = to_undirected(&g, ToUndirectedMode::Collapse).unwrap();
/// assert!(!u.is_directed());
/// assert_eq!(u.ecount(), 2); // (0,1) collapsed + (1,2)
///
/// let u = to_undirected(&g, ToUndirectedMode::Mutual).unwrap();
/// assert_eq!(u.ecount(), 1); // only (0,1) mutual pair survives
///
/// let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
/// assert_eq!(u.ecount(), 3); // every directed edge → undirected
/// ```
pub fn to_undirected(graph: &Graph, mode: ToUndirectedMode) -> IgraphResult<Graph> {
    let n = graph.vcount();

    if !graph.is_directed() {
        // Already undirected — clone the structure
        let mut result = Graph::with_vertices(n);
        let ecount = graph.ecount();
        for eid in 0..ecount {
            #[allow(clippy::cast_possible_truncation)]
            let (src, tgt) = graph.edge(eid as u32)?;
            result.add_edge(src, tgt)?;
        }
        return Ok(result);
    }

    let mut result = Graph::with_vertices(n);
    let ecount = graph.ecount();

    match mode {
        ToUndirectedMode::Each => {
            for eid in 0..ecount {
                #[allow(clippy::cast_possible_truncation)]
                let (src, tgt) = graph.edge(eid as u32)?;
                result.add_edge(src, tgt)?;
            }
        }
        ToUndirectedMode::Collapse => {
            let mut seen = std::collections::HashSet::with_capacity(ecount);
            for eid in 0..ecount {
                #[allow(clippy::cast_possible_truncation)]
                let (src, tgt) = graph.edge(eid as u32)?;
                let key = if src <= tgt { (src, tgt) } else { (tgt, src) };
                if seen.insert(key) {
                    result.add_edge(key.0, key.1)?;
                }
            }
        }
        ToUndirectedMode::Mutual => {
            // Build set of all directed edges, then add undirected for mutual pairs
            let mut forward = std::collections::HashSet::with_capacity(ecount);
            for eid in 0..ecount {
                #[allow(clippy::cast_possible_truncation)]
                let (src, tgt) = graph.edge(eid as u32)?;
                forward.insert((src, tgt));
            }
            // Track which canonical pairs we've already added
            let mut added = std::collections::HashSet::new();
            for &(src, tgt) in &forward {
                if src != tgt && forward.contains(&(tgt, src)) {
                    let key = if src <= tgt { (src, tgt) } else { (tgt, src) };
                    if added.insert(key) {
                        result.add_edge(key.0, key.1)?;
                    }
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
        assert!(!u.is_directed());
        assert_eq!(u.vcount(), 3);
        assert_eq!(u.ecount(), 2);
    }

    #[test]
    fn test_each_mode_mutual_pair() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
        assert!(!u.is_directed());
        assert_eq!(u.ecount(), 2); // two parallel undirected edges
    }

    #[test]
    fn test_each_mode_single_edge() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
        assert_eq!(u.ecount(), 2);
    }

    #[test]
    fn test_collapse_mode_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Collapse).unwrap();
        assert_eq!(u.ecount(), 2); // (0,1) collapsed + (1,2)
    }

    #[test]
    fn test_collapse_mode_parallel() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Collapse).unwrap();
        assert_eq!(u.ecount(), 1); // all collapse to one
    }

    #[test]
    fn test_mutual_mode_basic() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Mutual).unwrap();
        assert_eq!(u.ecount(), 1); // only (0,1) is mutual
    }

    #[test]
    fn test_mutual_mode_no_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Mutual).unwrap();
        assert_eq!(u.ecount(), 0); // no mutual pairs
    }

    #[test]
    fn test_mutual_mode_all_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        for i in 0..3u32 {
            for j in 0..3u32 {
                if i != j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        let u = to_undirected(&g, ToUndirectedMode::Mutual).unwrap();
        assert_eq!(u.ecount(), 3); // 3 mutual pairs → 3 undirected
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
        assert!(!u.is_directed());
        assert_eq!(u.vcount(), 0);
        assert_eq!(u.ecount(), 0);
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::new(5, true).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Collapse).unwrap();
        assert_eq!(u.vcount(), 5);
        assert_eq!(u.ecount(), 0);
    }

    #[test]
    fn test_self_loops_each() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Each).unwrap();
        assert_eq!(u.ecount(), 2);
    }

    #[test]
    fn test_self_loops_mutual() {
        // Self-loop: (0,0) is its own reverse, so it IS mutual
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Mutual).unwrap();
        // Self-loop: forward.contains((0,0)) and forward.contains((0,0)) → yes
        // But src != tgt check filters self-loops in mutual mode
        assert_eq!(u.ecount(), 0);
    }

    #[test]
    fn test_vcount_preserved() {
        let mut g = Graph::new(10, true).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        let u = to_undirected(&g, ToUndirectedMode::Collapse).unwrap();
        assert_eq!(u.vcount(), 10);
    }
}
