//! Simplify-and-colorize (`ALGO-ISO-030`).
//!
//! Port of `igraph_simplify_and_colorize()` from
//! `references/igraph/src/isomorphism/isomorphism_misc.c:54-114`. It turns
//! a graph that may have self-loops and multi-edges into a *simple* graph
//! plus two colour vectors:
//!
//! - `vertex_color[v]` — the number of self-loops incident to `v`,
//! - `edge_color[e]` — the multiplicity of the merged parallel edge `e`
//!   in the result.
//!
//! The colored simple graph is what isomorphism backends such as VF2
//! consume: they only accept simple graphs but can take vertex/edge
//! colours into account, so the multiplicities are preserved as colours.
//!
//! Upstream iterates edges in `IGRAPH_EDGEORDER_FROM` order — the `oi`
//! out-index order, which sorts edges by `(from, to)`. That groups every
//! parallel copy of an edge consecutively, so a single linear scan can
//! both detect parallels (equal `(from, to)` as the previous kept edge)
//! and self-loops (`from == to`). We reproduce that order by sorting edge
//! ids by their stored endpoints; for undirected graphs the endpoints are
//! already canonicalised (`from <= to`) by [`Graph`], so edges given in
//! either vertex order collapse together exactly as upstream intends.

use crate::algorithms::constructors::create::create;
use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// The colored simple graph produced by [`simplify_and_colorize`].
///
/// `vertex_color` has one entry per vertex of the input (self-loop
/// multiplicities); `edge_color` has one entry per edge of `graph` (the
/// parallel-edge multiplicities, each `>= 1`).
#[derive(Debug, Clone)]
pub struct SimplifyAndColorize {
    /// The simplified graph: no self-loops, no multi-edges. Same vertex
    /// count and directedness as the input.
    pub graph: Graph,
    /// Self-loop multiplicity of each input vertex (`vcount` entries).
    pub vertex_color: Vec<u32>,
    /// Multiplicity of each merged edge in [`SimplifyAndColorize::graph`]
    /// (`ecount` entries, each `>= 1`).
    pub edge_color: Vec<u32>,
}

/// Build a vertex- and edge-colored simple graph from `graph`.
///
/// Self-loops are removed and counted into `vertex_color`; parallel edges
/// are merged and their multiplicity recorded in `edge_color`. The result
/// graph keeps the input's vertex count (isolated vertices survive) and
/// directedness.
///
/// On undirected graphs an edge given as `(u, v)` and one given as
/// `(v, u)` denote the same edge and are merged; on directed graphs they
/// are distinct arcs and are kept separate.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] only if a colour count or an edge id
/// would overflow (not reachable for graphs that fit in memory), and
/// propagates any error from rebuilding the result graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, simplify_and_colorize};
///
/// // Undirected: two parallel 0-1 edges and one self-loop on vertex 1.
/// let mut g = Graph::new(2, false).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 1).unwrap();
/// let r = simplify_and_colorize(&g).unwrap();
/// assert_eq!(r.graph.ecount(), 1);
/// assert_eq!(r.vertex_color, vec![0, 1]); // one self-loop on vertex 1
/// assert_eq!(r.edge_color, vec![2]);      // edge 0-1 merged from 2 copies
/// ```
pub fn simplify_and_colorize(graph: &Graph) -> IgraphResult<SimplifyAndColorize> {
    let n = graph.vcount();
    let m = graph.ecount();
    let directed = graph.is_directed();

    let mut vertex_color = vec![0u32; n as usize];

    // Cache each edge's stored endpoints, then sort edge ids by
    // `(from, to)` to reproduce IGRAPH_EDGEORDER_FROM (the `oi` order).
    let mut endpoints: Vec<(u32, u32)> = Vec::with_capacity(m);
    for e in 0..m {
        let eid = EdgeId::try_from(e)
            .map_err(|_| IgraphError::Internal("simplify_and_colorize: edge id overflow"))?;
        endpoints.push(graph.edge(eid)?);
    }
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by_key(|&e| endpoints[e]);

    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(m);
    let mut edge_color: Vec<u32> = Vec::with_capacity(m);
    // The (from, to) of the previously kept (non-loop) edge, or `None`.
    let mut prev: Option<(u32, u32)> = None;

    for &e in &order {
        let (from, to) = endpoints[e];

        if from == to {
            let slot = &mut vertex_color[to as usize];
            *slot = slot.checked_add(1).ok_or(IgraphError::Internal(
                "simplify_and_colorize: vertex color overflow",
            ))?;
            continue;
        }

        if prev == Some((from, to)) {
            // Parallel copy of the last kept edge: bump its multiplicity.
            let slot = edge_color.last_mut().ok_or(IgraphError::Internal(
                "simplify_and_colorize: missing edge color",
            ))?;
            *slot = slot.checked_add(1).ok_or(IgraphError::Internal(
                "simplify_and_colorize: edge color overflow",
            ))?;
        } else {
            edges.push((from, to));
            edge_color.push(1);
        }

        prev = Some((from, to));
    }

    let res = create(&edges, n, directed)?;

    Ok(SimplifyAndColorize {
        graph: res,
        vertex_color,
        edge_color,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_graph() {
        let g = Graph::new(0, false).unwrap();
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.vertex_color.is_empty());
        assert!(r.edge_color.is_empty());
    }

    #[test]
    fn singleton_graph() {
        let g = Graph::new(1, false).unwrap();
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.graph.vcount(), 1);
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.vertex_color, vec![0]);
        assert!(r.edge_color.is_empty());
    }

    #[test]
    fn cycle_four_undirected() {
        // ring(4): 0-1, 1-2, 2-3, 3-0. No loops, no parallels.
        let mut g = Graph::new(4, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 4);
        assert_eq!(r.vertex_color, vec![0, 0, 0, 0]);
        assert_eq!(r.edge_color, vec![1, 1, 1, 1]);
        // Edges emitted in (from, to) order.
        let edges: Vec<_> = (0..r.graph.ecount())
            .map(|e| r.graph.edge(u32::try_from(e).unwrap()).unwrap())
            .collect();
        assert_eq!(edges, vec![(0, 1), (0, 3), (1, 2), (2, 3)]);
    }

    #[test]
    fn undirected_multi_and_loops() {
        // small(2): 0-1, 0-1, 1-1.
        let mut g = Graph::new(2, false).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.vertex_color, vec![0, 1]);
        assert_eq!(r.edge_color, vec![2]);
        assert_eq!(r.graph.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn undirected_parallel_different_orderings() {
        // small(3): 0-1, 1-2, 2-0, 2-2, 2-2, 2-1.
        // 1-2 and 2-1 are the same undirected edge -> merge.
        let mut g = Graph::new(3, false).unwrap();
        for &(u, v) in &[(0, 1), (1, 2), (2, 0), (2, 2), (2, 2), (2, 1)] {
            g.add_edge(u, v).unwrap();
        }
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.vertex_color, vec![0, 0, 2]);
        assert_eq!(r.edge_color, vec![1, 1, 2]);
        let edges: Vec<_> = (0..r.graph.ecount())
            .map(|e| r.graph.edge(u32::try_from(e).unwrap()).unwrap())
            .collect();
        assert_eq!(edges, vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn directed_keeps_arc_orientation() {
        // small(3, directed): 0-1, 1-2, 2-0, 2-2, 2-2, 2-1.
        // 1->2 and 2->1 are distinct arcs -> not merged.
        let mut g = Graph::new(3, true).unwrap();
        for &(u, v) in &[(0, 1), (1, 2), (2, 0), (2, 2), (2, 2), (2, 1)] {
            g.add_edge(u, v).unwrap();
        }
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.vertex_color, vec![0, 0, 2]);
        assert_eq!(r.edge_color, vec![1, 1, 1, 1]);
        let edges: Vec<_> = (0..r.graph.ecount())
            .map(|e| r.graph.edge(u32::try_from(e).unwrap()).unwrap())
            .collect();
        assert_eq!(edges, vec![(0, 1), (1, 2), (2, 0), (2, 1)]);
        assert!(r.graph.is_directed());
    }

    #[test]
    fn directed_isolated_vertices_preserved() {
        // small(4, directed): 0-1, 0-1, 1-0, 1-0, 1-0, 1-1.
        // Vertices 2, 3 are isolated and must survive.
        let mut g = Graph::new(4, true).unwrap();
        for &(u, v) in &[(0, 1), (0, 1), (1, 0), (1, 0), (1, 0), (1, 1)] {
            g.add_edge(u, v).unwrap();
        }
        let r = simplify_and_colorize(&g).unwrap();
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.vertex_color, vec![0, 1, 0, 0]);
        assert_eq!(r.edge_color, vec![2, 3]);
        let edges: Vec<_> = (0..r.graph.ecount())
            .map(|e| r.graph.edge(u32::try_from(e).unwrap()).unwrap())
            .collect();
        assert_eq!(edges, vec![(0, 1), (1, 0)]);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::create;
    use proptest::prelude::*;

    fn arb_edges(max_v: u32) -> impl Strategy<Value = (u32, Vec<(u32, u32)>, bool)> {
        (2..=max_v, any::<bool>()).prop_flat_map(|(n, directed)| {
            proptest::collection::vec((0..n, 0..n), 0..=15)
                .prop_map(move |edges| (n, edges, directed))
        })
    }

    proptest! {
        /// The result is always a simple graph (no self-loops, no
        /// multi-edges) with the same vcount and directedness, and the
        /// colour vectors have the documented lengths and bounds.
        #[test]
        fn result_is_simple_and_well_formed(
            (n, edges, directed) in arb_edges(6),
        ) {
            let g = create(&edges, n, directed).expect("build graph");
            let r = simplify_and_colorize(&g).expect("ok");

            prop_assert_eq!(r.graph.vcount(), n);
            prop_assert_eq!(r.graph.is_directed(), directed);
            prop_assert_eq!(r.vertex_color.len(), n as usize);
            prop_assert_eq!(r.edge_color.len(), r.graph.ecount());

            // No self-loops, every edge multiplicity >= 1.
            let mut seen = std::collections::HashSet::new();
            for e in 0..r.graph.ecount() {
                let (a, b) = r.graph.edge(u32::try_from(e).expect("fits u32")).expect("edge");
                prop_assert_ne!(a, b, "result has a self-loop");
                prop_assert!(r.edge_color[e] >= 1);
                let key = if directed { (a, b) } else { (a.min(b), a.max(b)) };
                prop_assert!(seen.insert(key), "result has a parallel edge");
            }
        }

        /// Conservation: self-loop colours sum to the number of input
        /// self-loops, and edge colours sum to the number of input
        /// non-loop edges.
        #[test]
        fn colours_conserve_edge_count(
            (n, edges, directed) in arb_edges(6),
        ) {
            let g = create(&edges, n, directed).expect("build graph");
            let r = simplify_and_colorize(&g).expect("ok");

            let loops_in = edges.iter().filter(|(a, b)| a == b).count();
            let nonloops_in = edges.len() - loops_in;

            let vsum: u64 = r.vertex_color.iter().map(|&c| u64::from(c)).sum();
            let esum: u64 = r.edge_color.iter().map(|&c| u64::from(c)).sum();

            prop_assert_eq!(vsum, loops_in as u64);
            prop_assert_eq!(esum, nonloops_in as u64);
        }
    }
}
