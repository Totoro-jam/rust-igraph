//! Generic isomorphism dispatchers (`ALGO-ISO-007`).
//!
//! [`isomorphic`] and [`subisomorphic`] are the "first set" of igraph's
//! isomorphism API: they pick a backend automatically rather than exposing
//! the engine knobs. Only the yes/no verdict is observable, so the choice of
//! backend is an implementation detail.

use super::canonical::isomorphic_bliss::isomorphic_bliss;
use super::simplify_and_colorize::simplify_and_colorize;
use super::subiso::subisomorphic_vf2;
use super::vf2::isomorphic_vf2;
use crate::algorithms::properties::multiplicity::has_multiple;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Decide whether two graphs are isomorphic, choosing a backend automatically.
///
/// Mirrors `igraph_isomorphic`: the dispatch picks whichever engine fits the
/// input. Only the Boolean verdict is returned — call [`isomorphic_vf2`] or
/// [`isomorphic_bliss`] directly when you also need the vertex mapping.
///
/// The dispatch is:
/// 1. directed vs. undirected → [`IgraphError::InvalidArgument`];
/// 2. if either graph has multi-edges, both are reduced with
///    [`simplify_and_colorize`] and compared by colour-aware VF2 (self-loops
///    become vertex colours, parallel-edge multiplicities become edge
///    colours);
/// 3. otherwise, differing vertex *or* edge counts → `false`;
/// 4. otherwise the canonical-form (BLISS) test decides.
///
/// Upstream takes an O(1) precomputed-`isoclass` shortcut for tiny graphs
/// (directed 3–4, undirected 3–6 vertices) before falling back to BLISS; the
/// canonical-form test returns the identical verdict, so this port uses it
/// uniformly. Self-loops are supported (folded into the BLISS vertex
/// invariant).
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if the two graphs differ in
/// directedness. Propagates backend errors otherwise.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isomorphic};
///
/// // A 4-cycle and its relabelling are isomorphic.
/// let mut a = Graph::new(4, false).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
///     a.add_edge(u, v).unwrap();
/// }
/// let mut b = Graph::new(4, false).unwrap();
/// for (u, v) in [(0, 2), (2, 1), (1, 3), (3, 0)] {
///     b.add_edge(u, v).unwrap();
/// }
/// assert!(isomorphic(&a, &b).unwrap());
///
/// // A 4-cycle is not isomorphic to a path on 4 vertices.
/// let mut p = Graph::new(4, false).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 3)] {
///     p.add_edge(u, v).unwrap();
/// }
/// assert!(!isomorphic(&a, &p).unwrap());
/// ```
pub fn isomorphic(graph1: &Graph, graph2: &Graph) -> IgraphResult<bool> {
    if graph1.is_directed() != graph2.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cannot compare directed and undirected graphs for isomorphism".to_owned(),
        ));
    }

    // Multi-edge path: reduce both graphs to a vertex/edge coloured simple
    // graph and let colour-aware VF2 decide (BLISS rejects multi-edges).
    if has_multiple(graph1)? || has_multiple(graph2)? {
        let r1 = simplify_and_colorize(graph1)?;
        let r2 = simplify_and_colorize(graph2)?;
        let res = isomorphic_vf2(
            &r1.graph,
            &r2.graph,
            Some(&r1.vertex_color),
            Some(&r2.vertex_color),
            Some(&r1.edge_color),
            Some(&r2.edge_color),
        )?;
        return Ok(res.iso);
    }

    if graph1.vcount() != graph2.vcount() || graph1.ecount() != graph2.ecount() {
        return Ok(false);
    }

    Ok(isomorphic_bliss(graph1, graph2, None, None)?.iso)
}

/// Decide whether `graph2` is isomorphic to a subgraph of `graph1`.
///
/// Mirrors `igraph_subisomorphic`, which simply delegates to the VF2 subgraph
/// engine for all inputs. `graph1` is the (larger) target and `graph2` the
/// (smaller) pattern; both must share directedness. Like upstream, non-simple
/// graphs are not supported by the VF2 backend.
///
/// Only the Boolean verdict is returned — call [`subisomorphic_vf2`] directly
/// when you also need a concrete embedding.
///
/// # Errors
///
/// Propagates the VF2 backend's errors (e.g. directedness mismatch, or a
/// graph the engine rejects).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, subisomorphic};
///
/// // A single edge embeds into a triangle.
/// let mut triangle = Graph::new(3, false).unwrap();
/// for (u, v) in [(0, 1), (1, 2), (2, 0)] {
///     triangle.add_edge(u, v).unwrap();
/// }
/// let mut edge = Graph::new(2, false).unwrap();
/// edge.add_edge(0, 1).unwrap();
/// assert!(subisomorphic(&triangle, &edge).unwrap());
/// ```
pub fn subisomorphic(graph1: &Graph, graph2: &Graph) -> IgraphResult<bool> {
    Ok(subisomorphic_vf2(graph1, graph2, None, None, None, None)?.iso)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).expect("edge");
        }
        g
    }

    fn path(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n.saturating_sub(1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    fn complete(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for a in 0..n {
            for b in (a + 1)..n {
                g.add_edge(a, b).expect("edge");
            }
        }
        g
    }

    #[test]
    fn relabelled_cycle_is_isomorphic() {
        let a = ring(6, false);
        let mut b = Graph::new(6, false).expect("graph");
        for (u, v) in [(0, 2), (2, 4), (4, 1), (1, 3), (3, 5), (5, 0)] {
            b.add_edge(u, v).expect("edge");
        }
        assert!(isomorphic(&a, &b).expect("iso"));
    }

    #[test]
    fn cycle_is_not_a_path() {
        assert!(!isomorphic(&ring(6, false), &path(6)).expect("iso"));
    }

    #[test]
    fn different_edge_count_short_circuits_to_false() {
        // Same vertex count, different edge count: dispatch returns false
        // without invoking a backend.
        let c4 = ring(4, false);
        let mut three_edges = Graph::new(4, false).expect("graph");
        for (u, v) in [(0, 1), (1, 2), (2, 3)] {
            three_edges.add_edge(u, v).expect("edge");
        }
        assert!(!isomorphic(&c4, &three_edges).expect("iso"));
    }

    #[test]
    fn directedness_mismatch_is_an_error() {
        assert!(isomorphic(&ring(3, false), &ring(3, true)).is_err());
    }

    #[test]
    fn directed_cycles_respect_orientation() {
        assert!(isomorphic(&ring(5, true), &ring(5, true)).expect("iso"));
    }

    #[test]
    fn self_loops_are_supported() {
        // Two single-vertex graphs each with one self-loop are isomorphic;
        // BLISS folds the loop into the vertex invariant.
        let mut a = Graph::new(2, false).expect("graph");
        a.add_edge(0, 0).expect("loop");
        a.add_edge(0, 1).expect("edge");
        let mut b = Graph::new(2, false).expect("graph");
        b.add_edge(1, 1).expect("loop");
        b.add_edge(0, 1).expect("edge");
        assert!(isomorphic(&a, &b).expect("iso"));
    }

    #[test]
    fn multigraph_path_uses_colorized_vf2() {
        // Two parallel edges between 0-1 plus a pendant; relabelled copy must
        // still be detected as isomorphic via the simplify_and_colorize path.
        let mut a = Graph::new(3, false).expect("graph");
        a.add_edge(0, 1).expect("edge");
        a.add_edge(0, 1).expect("edge");
        a.add_edge(1, 2).expect("edge");
        let mut b = Graph::new(3, false).expect("graph");
        b.add_edge(2, 1).expect("edge");
        b.add_edge(2, 1).expect("edge");
        b.add_edge(1, 0).expect("edge");
        assert!(isomorphic(&a, &b).expect("iso"));
    }

    #[test]
    fn multigraph_distinguishes_multiplicity() {
        // Both have 3 edges on 3 vertices, but a doubles edge 0-1 while b is a
        // simple path: edge-colour (multiplicity) breaks the match.
        let mut a = Graph::new(3, false).expect("graph");
        a.add_edge(0, 1).expect("edge");
        a.add_edge(0, 1).expect("edge");
        a.add_edge(1, 2).expect("edge");
        let mut b = Graph::new(3, false).expect("graph");
        b.add_edge(0, 1).expect("edge");
        b.add_edge(1, 2).expect("edge");
        b.add_edge(2, 0).expect("edge");
        assert!(!isomorphic(&a, &b).expect("iso"));
    }

    #[test]
    fn edge_embeds_into_triangle() {
        let mut edge = Graph::new(2, false).expect("graph");
        edge.add_edge(0, 1).expect("edge");
        assert!(subisomorphic(&complete(3), &edge).expect("subiso"));
    }

    #[test]
    fn triangle_does_not_embed_into_path() {
        assert!(!subisomorphic(&path(3), &ring(3, false)).expect("subiso"));
    }

    #[test]
    fn whole_graph_embeds_into_itself() {
        let g = complete(4);
        assert!(subisomorphic(&g, &g).expect("subiso"));
    }
}
