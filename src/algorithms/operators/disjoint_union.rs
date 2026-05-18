//! Disjoint union (ALGO-OP-002).
//!
//! Counterpart of `igraph_disjoint_union()` from
//! `references/igraph/src/operators/disjoint_union.c`. Vertices of
//! `right` are relabelled to follow `left`'s vertices: vertex `v` in
//! `right` becomes `v + left.vcount()` in the result. Vertex and edge
//! ordering is preserved (left's edges first, then right's edges in
//! original order with shifted endpoints).
//!
//! Phase-1 minimal slice: two-graph variant only. Multi-argument
//! `disjoint_union_many` ships later (ALGO-OP-002b). Edge / vertex
//! attributes are dropped (Phase-1 minimal — see ALGO-AT-* milestone).

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns the disjoint union of `left` and `right`.
///
/// The result has `left.vcount() + right.vcount()` vertices and
/// `left.ecount() + right.ecount()` edges. Vertices from `right` are
/// shifted by `left.vcount()`.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if the two graphs differ in
///   directedness — disjoint union is only defined for graphs of the
///   same directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, disjoint_union};
///
/// // Two triangles → 6-vertex graph with two disjoint triangles.
/// let mut a = Graph::with_vertices(3);
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let b = a.clone();
///
/// let u = disjoint_union(&a, &b).unwrap();
/// assert_eq!(u.vcount(), 6);
/// assert_eq!(u.ecount(), 6);
/// // The right triangle's edges are shifted by 3.
/// assert_eq!(u.edge(3).unwrap(), (3, 4));
/// ```
pub fn disjoint_union(left: &Graph, right: &Graph) -> IgraphResult<Graph> {
    disjoint_union_many(&[left, right])
}

/// Disjoint union of any number of graphs (ALGO-OP-002b).
///
/// Counterpart of `igraph_disjoint_union_many()`. Vertices of the
/// `i`-th graph are relabelled to start at `Σ_{j<i} graphs[j].vcount()`,
/// preserving each input's internal vertex / edge ordering.
///
/// All inputs must have the same directedness. The empty slice yields
/// the null graph (`vcount = 0`, undirected). A single-graph input
/// returns a clone of that graph.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if directedness diverges across
///   inputs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, disjoint_union_many};
///
/// // Three triangles → 9-vertex graph with three disjoint triangles.
/// let mut t = Graph::with_vertices(3);
/// t.add_edge(0, 1).unwrap();
/// t.add_edge(1, 2).unwrap();
/// t.add_edge(2, 0).unwrap();
/// let u = disjoint_union_many(&[&t, &t, &t]).unwrap();
/// assert_eq!(u.vcount(), 9);
/// assert_eq!(u.ecount(), 9);
/// // Third triangle's last edge is `(2, 0)` shifted by 6. The
/// // undirected adjacency canonicalises to (min, max).
/// assert_eq!(u.edge(8).unwrap(), (6, 8));
/// ```
pub fn disjoint_union_many(graphs: &[&Graph]) -> IgraphResult<Graph> {
    if graphs.is_empty() {
        return Graph::new(0, false);
    }
    let directed = graphs[0].is_directed();
    for g in &graphs[1..] {
        if g.is_directed() != directed {
            return Err(IgraphError::InvalidArgument(
                "disjoint_union_many: cannot mix directed and undirected graphs".to_string(),
            ));
        }
    }

    // Total vertex / edge count with overflow guards. Cumulative
    // shifts let us relabel each graph's edges in one pass.
    let mut shifts: Vec<u32> = Vec::with_capacity(graphs.len());
    let mut total_v: u32 = 0;
    let mut total_e: usize = 0;
    for g in graphs {
        shifts.push(total_v);
        total_v = total_v
            .checked_add(g.vcount())
            .ok_or(IgraphError::Internal("vertex count overflow"))?;
        total_e = total_e
            .checked_add(g.ecount())
            .ok_or(IgraphError::Internal("edge count overflow"))?;
    }

    let mut out = Graph::new(total_v, directed)?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_e);
    for (i, g) in graphs.iter().enumerate() {
        let shift = shifts[i];
        let m = u32::try_from(g.ecount())
            .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
        for e in 0..m {
            let (u, v) = g.edge(e as EdgeId)?;
            edges.push((u + shift, v + shift));
        }
    }
    out.add_edges(edges)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).unwrap();
        let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn two_triangles_undirected() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 0).unwrap();
        let b = a.clone();
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 6);
        assert_eq!(u.ecount(), 6);
        assert_eq!(
            sorted_edges(&u),
            vec![(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)]
        );
    }

    #[test]
    fn empty_left() {
        let a = Graph::with_vertices(0);
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 2);
        assert_eq!(u.ecount(), 1);
        assert_eq!(u.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn empty_right() {
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let b = Graph::with_vertices(0);
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 2);
        assert_eq!(u.ecount(), 1);
    }

    #[test]
    fn both_empty() {
        let a = Graph::with_vertices(0);
        let b = Graph::with_vertices(0);
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 0);
        assert_eq!(u.ecount(), 0);
    }

    #[test]
    fn isolated_plus_edge() {
        // Right graph has only isolated vertices; output preserves them.
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let b = Graph::with_vertices(3);
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 5);
        assert_eq!(u.ecount(), 1);
    }

    #[test]
    fn directed_directed_succeeds() {
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(1, 0).unwrap();
        let u = disjoint_union(&a, &b).unwrap();
        assert!(u.is_directed());
        assert_eq!(u.vcount(), 4);
        assert_eq!(u.ecount(), 2);
        assert_eq!(u.edge(0).unwrap(), (0, 1));
        assert_eq!(u.edge(1).unwrap(), (3, 2));
    }

    #[test]
    fn mixed_directedness_errors() {
        let a = Graph::with_vertices(2);
        let b = Graph::new(2, true).unwrap();
        assert!(disjoint_union(&a, &b).is_err());
    }

    #[test]
    fn vertex_count_preserved_for_isolated_left() {
        let a = Graph::with_vertices(5);
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let u = disjoint_union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 7);
        assert_eq!(u.edge(0).unwrap(), (5, 6));
    }

    #[test]
    fn idempotent_with_self_when_relabelled() {
        // disjoint_union(a, a) doubles vertex count and edge count.
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 3).unwrap();
        let u = disjoint_union(&a, &a).unwrap();
        assert_eq!(u.vcount(), 8);
        assert_eq!(u.ecount(), 6);
    }

    // ----- ALGO-OP-002b: disjoint_union_many -----

    #[test]
    fn many_empty_slice_yields_null_graph() {
        let u = disjoint_union_many(&[]).unwrap();
        assert_eq!(u.vcount(), 0);
        assert_eq!(u.ecount(), 0);
        assert!(!u.is_directed());
    }

    #[test]
    fn many_single_graph_yields_clone() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        let u = disjoint_union_many(&[&a]).unwrap();
        assert_eq!(u.vcount(), a.vcount());
        assert_eq!(u.ecount(), a.ecount());
        assert_eq!(sorted_edges(&u), sorted_edges(&a));
    }

    #[test]
    fn many_three_triangles_shifts_correctly() {
        let mut t = Graph::with_vertices(3);
        t.add_edge(0, 1).unwrap();
        t.add_edge(1, 2).unwrap();
        t.add_edge(2, 0).unwrap();
        let u = disjoint_union_many(&[&t, &t, &t]).unwrap();
        assert_eq!(u.vcount(), 9);
        assert_eq!(u.ecount(), 9);
        // Each triangle contributes its three edges in order.
        assert_eq!(u.edge(0).unwrap(), (0, 1));
        assert_eq!(u.edge(3).unwrap(), (3, 4));
        assert_eq!(u.edge(6).unwrap(), (6, 7));
    }

    #[test]
    fn many_matches_pairwise_left_to_right() {
        // disjoint_union_many(&[a, b, c]) ≡ disjoint_union(disjoint_union(a, b), c)
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(3);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 2).unwrap();
        let mut c = Graph::with_vertices(2);
        c.add_edge(0, 1).unwrap();
        let u = disjoint_union_many(&[&a, &b, &c]).unwrap();
        let pairwise = disjoint_union(&disjoint_union(&a, &b).unwrap(), &c).unwrap();
        assert_eq!(u.vcount(), pairwise.vcount());
        assert_eq!(u.ecount(), pairwise.ecount());
        assert_eq!(sorted_edges(&u), sorted_edges(&pairwise));
    }

    #[test]
    fn many_mixed_directedness_errors() {
        let a = Graph::with_vertices(2);
        let b = Graph::with_vertices(2);
        let c = Graph::new(2, true).unwrap();
        assert!(disjoint_union_many(&[&a, &b, &c]).is_err());
    }

    #[test]
    fn many_directed_preserves_orientation() {
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(1, 0).unwrap();
        let u = disjoint_union_many(&[&a, &b]).unwrap();
        assert!(u.is_directed());
        assert_eq!(u.vcount(), 4);
        assert_eq!(u.ecount(), 2);
        assert_eq!(u.edge(0).unwrap(), (0, 1));
        assert_eq!(u.edge(1).unwrap(), (3, 2));
    }
}
