//! `is_simple` (ALGO-PR-013 + ALGO-PR-013b).
//!
//! Counterpart of `igraph_is_simple()` from
//! `references/igraph/src/properties/multiplicity.c`. A graph is *simple*
//! if it has no self-loops and no parallel edges. Useful as a fast
//! predicate before algorithms that assume simplicity (or to short-circuit
//! [`crate::simplify()`] when nothing needs to change).
//!
//! [`is_simple`] (PR-013) treats directed graphs structurally — `(a,b)`
//! and `(b,a)` are distinct edges so a mutual pair stays simple.
//! [`is_simple_with_mode`] (PR-013b) adds [`SimpleMode::DirectedAsUndirected`]
//! which collapses each directed edge to its endpoint pair, so a mutual
//! `{a → b, b → a}` is reported as a multi-edge.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Direction-handling for [`is_simple_with_mode`]. Counterpart of
/// upstream's `directed` boolean parameter.
///
/// On undirected graphs both modes produce the same answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimpleMode {
    /// `(a, b)` and `(b, a)` are distinct directed edges. Default;
    /// matches what plain [`is_simple`] returns.
    DirectedAsDirected,
    /// `(a, b)` and `(b, a)` collapse to the same undirected edge,
    /// so a mutual pair is reported as a multi-edge.
    DirectedAsUndirected,
}

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
    is_simple_with_mode(graph, SimpleMode::DirectedAsDirected)
}

/// `is_simple` with explicit [`SimpleMode`] (ALGO-PR-013b).
///
/// On undirected graphs both modes are equivalent. On directed graphs
/// [`SimpleMode::DirectedAsUndirected`] returns `false` whenever a
/// mutual pair `{a → b, b → a}` exists (treated as a multi-edge in
/// the undirected view).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_simple_with_mode, SimpleMode};
///
/// // Mutual directed pair: simple in directed mode, NOT simple as
/// // undirected (would be a doubled undirected edge).
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
/// assert!(!is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap());
/// ```
pub fn is_simple_with_mode(graph: &Graph, mode: SimpleMode) -> IgraphResult<bool> {
    let n = graph.vcount();
    let m = graph.ecount();
    if n == 0 || m == 0 {
        return Ok(true);
    }

    // Undirected graphs: both modes are equivalent because the
    // existing per-vertex sorted-neighbour scan already collapses
    // `(a,b)` / `(b,a)` (each undirected edge contributes once to
    // both endpoints' neighbour lists in lexicographic order). The
    // structural-directed scan below also works for undirected
    // graphs since `Graph::neighbors` returns the merged adjacency
    // view.
    if !graph.is_directed() || mode == SimpleMode::DirectedAsDirected {
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
        return Ok(true);
    }

    // Directed-as-undirected: canonicalise each edge to `(min, max)`
    // and look for duplicates after sorting. Self-loops fail
    // immediately. O(|E| log |E|).
    let m_u32 = u32::try_from(m).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let mut pairs: Vec<(u32, u32)> = Vec::with_capacity(m);
    for eid in 0..m_u32 {
        let (src, tgt) = graph.edge(eid as EdgeId)?;
        if src == tgt {
            return Ok(false);
        }
        pairs.push(if src < tgt { (src, tgt) } else { (tgt, src) });
    }
    pairs.sort_unstable();
    for w in pairs.windows(2) {
        if w[0] == w[1] {
            return Ok(false);
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

    // ----- ALGO-PR-013b: SimpleMode::DirectedAsUndirected -----

    #[test]
    fn directed_mutual_pair_not_simple_in_undirected_mode() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
        assert!(!is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap());
    }

    #[test]
    fn directed_mode_default_matches_is_simple() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            is_simple(&g).unwrap(),
            is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap()
        );
    }

    #[test]
    fn directed_3_cycle_simple_in_undirected_mode() {
        // Three directed edges forming a cycle 0→1→2→0. As undirected,
        // they form three distinct edges (0-1, 1-2, 0-2) → simple.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap());
    }

    #[test]
    fn directed_self_loop_not_simple_in_either_mode() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!(!is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
        assert!(!is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap());
    }

    #[test]
    fn undirected_graph_modes_are_equivalent() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let a = is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap();
        let b = is_simple_with_mode(&g, SimpleMode::DirectedAsUndirected).unwrap();
        assert_eq!(a, b);
        assert!(a);
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
