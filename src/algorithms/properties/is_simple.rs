//! `is_simple` (ALGO-PR-013).
//!
//! Counterpart of `igraph_is_simple()` from
//! `references/igraph/src/properties/multiplicity.c`. A graph is *simple*
//! if it has no self-loops and no parallel edges. Useful as a fast
//! predicate before algorithms that assume simplicity (or to short-circuit
//! [`crate::simplify`] when nothing needs to change).
//!
//! Phase-1 minimal slice: undirected and directed graphs (the latter
//! treated structurally — `(a, b)` and `(b, a)` are considered distinct
//! parallel edges, matching upstream's `directed = IGRAPH_DIRECTED`
//! variant). The "treat directed graph as undirected" mode (`directed
//! = IGRAPH_UNDIRECTED`, where mutual edge pairs count as parallel)
//! ships in PR-013b.

use crate::core::{Graph, IgraphResult};

/// Returns `true` if `graph` has neither self-loops nor parallel edges.
///
/// Empty / no-edge graphs return `true` (vacuously simple).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_simple};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_simple(&g).unwrap());
///
/// // Add a parallel edge → no longer simple.
/// g.add_edge(0, 1).unwrap();
/// assert!(!is_simple(&g).unwrap());
/// ```
pub fn is_simple(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    if n == 0 || graph.ecount() == 0 {
        return Ok(true);
    }

    // For each vertex, scan its sorted out-neighbours for self-loops or
    // adjacent duplicates. `Graph::neighbors` already returns ascending
    // order (see core::graph module docs), so this is O(|V| + |E|).
    for v in 0..n {
        let neis = graph.neighbors(v)?;
        let mut prev: Option<u32> = None;
        for &u in &neis {
            if u == v {
                return Ok(false);
            }
            if Some(u) == prev {
                return Ok(false);
            }
            prev = Some(u);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_simple() {
        let g = Graph::with_vertices(0);
        assert!(is_simple(&g).unwrap());
    }

    #[test]
    fn no_edge_graph_is_simple() {
        let g = Graph::with_vertices(5);
        assert!(is_simple(&g).unwrap());
    }

    #[test]
    fn path_is_simple() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_simple(&g).unwrap());
    }

    #[test]
    fn self_loop_breaks_simplicity() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_simple(&g).unwrap());
    }

    #[test]
    fn parallel_edge_breaks_simplicity_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_simple(&g).unwrap());
    }

    #[test]
    fn reversed_parallel_undirected_breaks_simplicity() {
        // Undirected (1,0) and (0,1) are the same edge.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(!is_simple(&g).unwrap());
    }

    #[test]
    fn directed_mutual_pair_is_simple() {
        // Directed (a,b) and (b,a) are distinct edges — phase-1 minimal
        // slice treats this as simple (matches upstream's directed=true).
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(is_simple(&g).unwrap());
    }

    #[test]
    fn directed_parallel_breaks_simplicity() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_simple(&g).unwrap());
    }

    #[test]
    fn directed_self_loop_breaks_simplicity() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!(!is_simple(&g).unwrap());
    }

    #[test]
    fn simplify_makes_graph_simple() {
        // Round-trip: simplify(g) is always simple.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_simple(&g).unwrap());
        let s = crate::simplify(&g, true, true).unwrap();
        assert!(is_simple(&s).unwrap());
    }
}
