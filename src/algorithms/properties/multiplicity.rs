//! Companion predicates to [`super::is_simple::is_simple`] (ALGO-PR-014).
//!
//! - [`has_loop`]    — does the graph contain at least one self-loop?
//! - [`has_multiple`] — does the graph contain at least one parallel
//!   (multi-) edge?
//!
//! Counterparts of `igraph_has_loop()` from
//! `references/igraph/src/properties/loops.c` and `igraph_has_multiple()`
//! from `references/igraph/src/properties/multiplicity.c`.
//!
//! Both run in O(|V| + |E|) (the upstream cache subsystem we don't
//! have yet would shortcut to O(1) on subsequent calls — that's an
//! ALGO-CORE-001e responsibility).

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult};

/// Returns `true` iff `graph` has at least one self-loop edge.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, has_loop};
///
/// let mut g = Graph::with_vertices(2);
/// g.add_edge(0, 1).unwrap();
/// assert!(!has_loop(&g).unwrap());
/// g.add_edge(1, 1).unwrap();
/// assert!(has_loop(&g).unwrap());
/// ```
pub fn has_loop(graph: &Graph) -> IgraphResult<bool> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m {
        let (u, v) = graph.edge(e as EdgeId)?;
        if u == v {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns `true` iff `graph` has at least one parallel edge.
///
/// For undirected graphs, two self-loops at the same vertex *do* count
/// as parallel (matching upstream `igraph_has_multiple()`), but a single
/// self-loop does not. For directed graphs, `(a, b)` and `(b, a)` are
/// distinct so only same-direction repeats count.
///
/// O(|E| log |E|) via sort-and-scan over stored edges. Storage already
/// canonicalises undirected endpoints to `from <= to`, so `(a,b)` and
/// `(b,a)` collapse to the same canonical pair, which is exactly the
/// behaviour we want.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, has_multiple};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!has_multiple(&g).unwrap());
/// g.add_edge(0, 1).unwrap();
/// assert!(has_multiple(&g).unwrap());
/// ```
pub fn has_multiple(graph: &Graph) -> IgraphResult<bool> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    if m < 2 {
        return Ok(false);
    }
    let mut pairs: Vec<(u32, u32)> = Vec::with_capacity(m as usize);
    for e in 0..m {
        pairs.push(graph.edge(e as EdgeId)?);
    }
    pairs.sort_unstable();
    for i in 1..pairs.len() {
        if pairs[i] == pairs[i - 1] {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_has_no_loop() {
        let g = Graph::with_vertices(0);
        assert!(!has_loop(&g).unwrap());
    }

    #[test]
    fn empty_graph_has_no_multi() {
        let g = Graph::with_vertices(0);
        assert!(!has_multiple(&g).unwrap());
    }

    #[test]
    fn no_edge_graph_has_neither() {
        let g = Graph::with_vertices(5);
        assert!(!has_loop(&g).unwrap());
        assert!(!has_multiple(&g).unwrap());
    }

    #[test]
    fn path_has_neither() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!has_loop(&g).unwrap());
        assert!(!has_multiple(&g).unwrap());
    }

    #[test]
    fn detects_self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        assert!(has_loop(&g).unwrap());
        // A lone self-loop should NOT count as a multi-edge.
        assert!(!has_multiple(&g).unwrap());
    }

    #[test]
    fn detects_parallel_undirected() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(!has_loop(&g).unwrap());
        assert!(has_multiple(&g).unwrap());
    }

    #[test]
    fn detects_parallel_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(has_multiple(&g).unwrap());
    }

    #[test]
    fn directed_mutual_pair_not_parallel() {
        // Directed (a,b) and (b,a) are distinct → not parallel.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(!has_multiple(&g).unwrap());
    }

    #[test]
    fn duplicate_self_loops_count_as_parallel() {
        // Two self-loops on the same vertex: both has_loop and has_multiple
        // return true (matches upstream igraph_has_multiple).
        let mut g = Graph::with_vertices(1);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!(has_loop(&g).unwrap());
        assert!(has_multiple(&g).unwrap());
    }

    #[test]
    fn matches_is_simple_negation_for_simple_graphs() {
        // Simple graphs have neither.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!has_loop(&g).unwrap());
        assert!(!has_multiple(&g).unwrap());
        assert!(crate::is_simple(&g).unwrap());
    }
}
