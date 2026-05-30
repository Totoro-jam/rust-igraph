//! Perfect-graph test (ALGO-PR-018).
//!
//! Counterpart of `igraph_is_perfect()` from
//! `references/igraph/src/properties/perfect.c:55-190`.
//!
//! A graph is *perfect* when, for every induced subgraph, the chromatic
//! number equals the size of the largest clique. By the Strong Perfect
//! Graph Theorem (Chudnovsky–Robertson–Seymour–Thomas, conjectured by
//! Berge) a graph is perfect iff neither it nor its complement contains an
//! induced odd cycle of length ≥ 5 (an "odd hole").
//!
//! The implementation is a faithful port of upstream's decision cascade,
//! composed entirely from primitives already ported in this crate:
//!
//! 1. Trivial perfect cases: fewer than 5 vertices; very sparse / very
//!    dense small graphs (≤ 4 edges, or within 5 edges of complete).
//! 2. Bipartite or chordal graphs are perfect (so are their complements —
//!    the Weak Perfect Graph Theorem lets us also test the complement).
//! 3. An odd girth > 3 in the graph or its complement proves *not* perfect.
//! 4. Otherwise search for an induced odd hole of length ≥ 5 in both the
//!    graph and its complement via induced LAD subgraph isomorphism.
//!
//! Scope mirrors upstream: undirected, simple graphs only (directed or
//! non-simple input is an error).

use crate::algorithms::chordality::is_chordal;
use crate::algorithms::constructors::ring::ring_graph;
use crate::algorithms::isomorphism::lad::subisomorphic_lad;
use crate::algorithms::operators::complementer::complementer;
use crate::algorithms::properties::girth::girth;
use crate::algorithms::properties::is_bipartite::is_bipartite;
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Returns `true` when `graph` is a perfect graph.
///
/// The graph must be **undirected** and **simple**; otherwise an
/// [`IgraphError::InvalidArgument`] is returned, matching upstream's
/// `IGRAPH_EINVAL`.
///
/// Time complexity: worst case exponential (the odd-hole search runs
/// induced subgraph isomorphism), but the bipartite / chordal / girth
/// fast paths resolve most inputs cheaply.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_perfect};
///
/// // The 5-cycle C5 is the smallest imperfect graph (it is its own odd hole).
/// let mut c5 = Graph::new(5, false)?;
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)] {
///     c5.add_edge(u, v)?;
/// }
/// assert!(!is_perfect(&c5)?);
///
/// // A tree (here a path) is bipartite, hence perfect.
/// let mut path = Graph::new(4, false)?;
/// for (u, v) in [(0, 1), (1, 2), (2, 3)] {
///     path.add_edge(u, v)?;
/// }
/// assert!(is_perfect(&path)?);
/// # Ok::<(), rust_igraph::IgraphError>(())
/// ```
pub fn is_perfect(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "The concept of perfect graphs is only defined for undirected graphs.".to_string(),
        ));
    }

    if !is_simple(graph)? {
        return Err(IgraphError::InvalidArgument(
            "Perfect graph testing is implemented for simple graphs only. Simplify the graph."
                .to_string(),
        ));
    }

    let no_of_nodes = graph.vcount();
    let no_of_edges = graph.ecount();

    // All graphs with fewer than 5 vertices are perfect.
    if no_of_nodes < 5 {
        return Ok(true);
    }

    // Graphs with fewer than 5 edges, or whose complement has fewer than 5
    // edges, are perfect. Bounded to small graphs to avoid overflow and
    // because the check's usefulness vanishes as the graph grows.
    let n = u64::from(no_of_nodes);
    let max_edges = (n - 1) * n / 2;
    if no_of_nodes < 10_000
        && (no_of_edges < 5 || (max_edges >= 5 && no_of_edges as u64 > max_edges - 5))
    {
        return Ok(true);
    }

    // Bipartite and chordal graphs are perfect.
    if is_bipartite(graph)?.is_bipartite {
        return Ok(true);
    }
    if is_chordal(graph, None)?.chordal {
        return Ok(true);
    }

    // Weak Perfect Graph Theorem: G is perfect iff its complement is.
    let comp = complementer(graph, false)?;

    if is_bipartite(&comp)?.is_bipartite {
        return Ok(true);
    }
    if is_chordal(&comp, None)?.chordal {
        return Ok(true);
    }

    // Past the bipartite check both girths are finite (bipartite also
    // catches forests). A finite odd girth > 3 is an odd hole on its own.
    let Some(girth_g) = girth(graph)? else {
        return Ok(true);
    };
    if girth_g > 3 && girth_g % 2 == 1 {
        return Ok(false);
    }

    let Some(girth_c) = girth(&comp)? else {
        return Ok(true);
    };
    if girth_c > 3 && girth_c % 2 == 1 {
        return Ok(false);
    }

    // Strong Perfect Graph Theorem: search for an induced odd hole of
    // length ≥ 5 in either the graph or its complement.
    let min_girth = girth_g.min(girth_c);
    let mut cycle_len = if min_girth % 2 == 0 {
        min_girth + 1
    } else {
        min_girth + 2
    };

    while cycle_len <= no_of_nodes {
        let cycle = ring_graph(cycle_len, false, false, true)?;

        if cycle_len > girth_g && subisomorphic_lad(&cycle, graph, None, true)?.iso {
            return Ok(false);
        }
        if cycle_len > girth_c && subisomorphic_lad(&cycle, &comp, None, true)?.iso {
            return Ok(false);
        }

        cycle_len += 2;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::new(n, false).expect("graph init");
        for &(a, b) in edges {
            g.add_edge(a, b).expect("add edge");
        }
        g
    }

    fn ring(n: u32) -> Graph {
        ring_graph(n, false, false, true).expect("ring")
    }

    #[test]
    fn null_graph_is_perfect() {
        let g = Graph::new(0, false).expect("graph");
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn singleton_is_perfect() {
        let g = Graph::new(1, false).expect("graph");
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn empty_two_vertices_is_perfect() {
        let g = Graph::new(2, false).expect("graph");
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn small_graphs_under_five_vertices_are_perfect() {
        // K4 — under the 5-vertex trivial bound.
        let g = undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn c5_is_not_perfect() {
        assert!(!is_perfect(&ring(5)).expect("is_perfect"));
    }

    #[test]
    fn c7_is_not_perfect() {
        assert!(!is_perfect(&ring(7)).expect("is_perfect"));
    }

    #[test]
    fn c6_is_perfect() {
        // Even cycles are bipartite, hence perfect.
        assert!(is_perfect(&ring(6)).expect("is_perfect"));
    }

    #[test]
    fn complement_of_c7_is_not_perfect() {
        let c7 = ring(7);
        let comp = complementer(&c7, false).expect("complement");
        assert!(!is_perfect(&comp).expect("is_perfect"));
    }

    #[test]
    fn paley_order_9_is_perfect() {
        // Paley graph of order 9 — self-complementary, perfect.
        let g = undirected(
            9,
            &[
                (0, 1),
                (0, 3),
                (0, 6),
                (0, 2),
                (1, 2),
                (1, 4),
                (1, 7),
                (2, 5),
                (2, 8),
                (3, 4),
                (3, 5),
                (3, 6),
                (4, 5),
                (4, 7),
                (5, 8),
                (6, 7),
                (7, 8),
                (6, 8),
            ],
        );
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn house_graph_is_perfect() {
        // "House": a C5 would be imperfect, but the House (square + roof
        // triangle) is chordal-free yet perfect.
        let g = undirected(5, &[(0, 1), (0, 2), (1, 3), (2, 3), (2, 4), (3, 4)]);
        assert!(is_perfect(&g).expect("is_perfect"));
    }

    #[test]
    fn directed_graph_errors() {
        let mut g = Graph::new(5, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert!(is_perfect(&g).is_err());
    }

    #[test]
    fn non_simple_graph_errors() {
        let mut g = Graph::new(5, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("parallel edge");
        assert!(is_perfect(&g).is_err());
    }
}
