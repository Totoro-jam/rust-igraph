//! Simplify (ALGO-OP-001).
//!
//! Counterpart of `igraph_simplify()` from
//! `references/igraph/src/operators/simplify.c`. Returns a new graph with
//! self-loops and/or parallel edges removed, according to the two boolean
//! flags. The input is borrowed and left untouched (upstream igraph mutates
//! the graph in place; we prefer immutability — call sites do
//! `g = simplify(&g, …)?` if they want C-style replacement).
//!
//! Phase-1 minimal slice: no edge-attribute combination (the upstream
//! `edge_comb` argument is ignored — attributes are an out-of-scope ALGO-AT
//! milestone). Both directed and undirected graphs are supported.
//!
//! Time complexity: O(|V| + |E| log |E|) when `remove_multiple = true`
//! (sort dominated); O(|V| + |E|) when only loops are removed.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult, VertexId};

/// Return a copy of `graph` with self-loops and/or parallel edges removed.
///
/// - `remove_multiple` — collapse parallel edges to a single edge between
///   each ordered pair (directed) / unordered pair (undirected).
/// - `remove_loops`    — drop every self-loop edge `(v, v)`.
///
/// If both flags are `false` this returns a structural clone of `graph`.
///
/// Edge attributes are dropped (Phase-1 minimal slice — see module docs).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, simplify};
///
/// // Triangle plus a self-loop and a parallel edge.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 0).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
///
/// let s = simplify(&g, true, true).unwrap();
/// assert_eq!(s.vcount(), 3);
/// assert_eq!(s.ecount(), 3);
/// ```
pub fn simplify(graph: &Graph, remove_multiple: bool, remove_loops: bool) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if !remove_multiple && !remove_loops {
        return Ok(graph.clone());
    }

    let m = graph.ecount();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(m);
    for eid in 0..m {
        let eid_u32 = u32::try_from(eid)
            .map_err(|_| crate::IgraphError::Internal("edge id exceeds u32::MAX"))?;
        let (f, t) = graph.edge(eid_u32 as EdgeId)?;
        if remove_loops && f == t {
            continue;
        }
        edges.push((f, t));
    }

    if remove_multiple {
        // For undirected graphs, `Graph::add_edges` canonicalises endpoints
        // so that stored from <= to (see `core::graph` module docs). The
        // edges we just collected already obey that invariant for
        // undirected inputs, and for directed inputs we treat (a,b) and
        // (b,a) as distinct — sort+dedup on the tuple does the right
        // thing in both cases.
        edges.sort_unstable();
        edges.dedup();
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn no_op_when_both_flags_false() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = simplify(&g, false, false).unwrap();
        assert_eq!(s.vcount(), g.vcount());
        assert_eq!(s.ecount(), g.ecount());
        assert_eq!(sorted_edges(&s), sorted_edges(&g));
    }

    #[test]
    fn removes_self_loops_only() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();

        let s = simplify(&g, false, true).unwrap();
        assert_eq!(s.ecount(), 2);
        assert_eq!(sorted_edges(&s), vec![(0, 1), (0, 1)]);
    }

    #[test]
    fn removes_multi_edges_only_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();

        let s = simplify(&g, true, false).unwrap();
        assert_eq!(s.ecount(), 2);
        assert_eq!(sorted_edges(&s), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn removes_multi_edges_directed() {
        // Directed: (a,b) and (b,a) are distinct.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();

        let s = simplify(&g, true, false).unwrap();
        assert!(s.is_directed());
        assert_eq!(s.ecount(), 3);
        assert_eq!(sorted_edges(&s), vec![(0, 1), (1, 0), (1, 2)]);
    }

    #[test]
    fn removes_both_loops_and_multi() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.ecount(), 3);
        assert_eq!(sorted_edges(&s), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn idempotent_on_simple_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.ecount(), g.ecount());
        assert_eq!(sorted_edges(&s), sorted_edges(&g));
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.vcount(), 0);
        assert_eq!(s.ecount(), 0);
    }

    #[test]
    fn vertices_preserved_when_no_edges_remain() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.vcount(), 3);
        assert_eq!(s.ecount(), 0);
    }

    #[test]
    fn igraph_simplify_example_directed_5_parallel() {
        // From references/igraph/examples/simple/igraph_simplify.c case 1:
        // directed 0->1 ×5, simplify(true,true) → single edge 0→1.
        let mut g = Graph::new(2, true).unwrap();
        for _ in 0..5 {
            g.add_edge(0, 1).unwrap();
        }
        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.ecount(), 1);
        assert_eq!(s.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn igraph_simplify_example_directed_loops_and_multi() {
        // From igraph_simplify.c case 3:
        // directed 0-0, 1-1, 2-2, 1-2; simplify(true,true) → only 1→2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(2, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = simplify(&g, true, true).unwrap();
        assert_eq!(s.ecount(), 1);
        assert_eq!(s.edge(0).unwrap(), (1, 2));
    }
}
