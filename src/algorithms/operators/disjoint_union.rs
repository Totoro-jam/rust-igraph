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
    if left.is_directed() != right.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "disjoint_union: cannot mix directed and undirected graphs".to_string(),
        ));
    }
    let n_left = left.vcount();
    let n_right = right.vcount();
    let n_total = n_left
        .checked_add(n_right)
        .ok_or(IgraphError::Internal("vertex count overflow"))?;

    let mut g = Graph::new(n_total, left.is_directed())?;

    let m_left = u32::try_from(left.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let m_right = u32::try_from(right.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity((m_left + m_right) as usize);
    for e in 0..m_left {
        edges.push(left.edge(e as EdgeId)?);
    }
    for e in 0..m_right {
        let (u, v) = right.edge(e as EdgeId)?;
        edges.push((u + n_left, v + n_left));
    }
    g.add_edges(edges)?;
    Ok(g)
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
}
