//! Reachability counts (ALGO-CC-020).
//!
//! Counterpart of `igraph_count_reachable()` from
//! `references/igraph/src/connectivity/reachability.c:179`. For each
//! vertex `v`, returns how many vertices (including `v` itself) are
//! reachable from `v`.
//!
//! Phase-1 minimal slice: BFS-from-each-vertex via the existing
//! [`crate::algorithms::paths::distances::distances`] primitive. The
//! O(|V|*(|V|+|E|)) cost is simpler to maintain than upstream's
//! SCC-condensation approach (O(|C||V|/w + |V| + |E|), where |C| is
//! the number of strongly connected components and w is the word size).
//! A perf pass with the SCC trick can land later.
//!
//! Edge directions follow the same rule as `distances`: out-edges only
//! for directed graphs (matching upstream's `IGRAPH_OUT` mode default).

use crate::algorithms::paths::distances::distances;
use crate::core::{Graph, IgraphResult};

/// Per-vertex count of reachable vertices (including the vertex itself).
///
/// Counterpart of `igraph_count_reachable(_, _, IGRAPH_OUT)` for
/// directed graphs and `IGRAPH_ALL` for undirected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_reachable};
///
/// // Path 0-1-2-3 undirected: every vertex reaches every vertex.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..3 { g.add_edge(i, i + 1).unwrap(); }
/// assert_eq!(count_reachable(&g).unwrap(), vec![4, 4, 4, 4]);
///
/// // Directed chain 0 -> 1 -> 2: 0 reaches all 3, 1 reaches 2, 2 reaches just itself.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert_eq!(count_reachable(&g).unwrap(), vec![3, 2, 1]);
/// ```
pub fn count_reachable(graph: &Graph) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();
    let mut out = Vec::with_capacity(n as usize);
    for v in 0..n {
        let d = distances(graph, v)?;
        let count = u32::try_from(d.iter().filter(|x| x.is_some()).count())
            .map_err(|_| crate::core::IgraphError::Internal("reachable count exceeds u32"))?;
        out.push(count);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_yields_empty_counts() {
        let g = Graph::with_vertices(0);
        assert!(count_reachable(&g).unwrap().is_empty());
    }

    #[test]
    fn isolated_vertices_each_reach_only_themselves() {
        let g = Graph::with_vertices(5);
        assert_eq!(count_reachable(&g).unwrap(), vec![1; 5]);
    }

    #[test]
    fn undirected_path_each_reaches_all() {
        let mut g = Graph::with_vertices(4);
        for i in 0..3 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(count_reachable(&g).unwrap(), vec![4, 4, 4, 4]);
    }

    #[test]
    fn undirected_two_components() {
        // {0,1,2} and {3,4}: each vertex reaches its component.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(count_reachable(&g).unwrap(), vec![3, 3, 3, 2, 2]);
    }

    #[test]
    fn directed_chain_each_only_reaches_downstream() {
        // 0 -> 1 -> 2 -> 3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(count_reachable(&g).unwrap(), vec![4, 3, 2, 1]);
    }

    #[test]
    fn directed_cycle_all_reach_all() {
        // 0 -> 1 -> 2 -> 0: each vertex reaches all 3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_reachable(&g).unwrap(), vec![3, 3, 3]);
    }

    #[test]
    fn self_loop_does_not_inflate_count() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(count_reachable(&g).unwrap(), vec![2, 2]);
    }
}
