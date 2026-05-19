//! Union of two graphs (ALGO-OP-004).
//!
//! Counterpart of `igraph_union()` from
//! `references/igraph/src/operators/union.c:69-75`. Vertex sets are
//! aligned by index — the result has
//! `max(left.vcount(), right.vcount())` vertices. Edges are unioned by
//! *maximum multiplicity*: for each (canonicalised) endpoint pair
//! `(u, v)`, the result contains `max(count_left, count_right)` edges.
//! For undirected inputs the canonicalised pair is `(min(u,v),
//! max(u,v))`; for directed inputs the pair is taken as-is, so
//! `(u, v)` and `(v, u)` are tallied separately.
//!
//! Phase-1 minimal slice: two-graph variant only. Multi-arg
//! `union_many` and the edge-mapping outputs (`edge_map1` /
//! `edge_map2`) ship later.

use std::collections::BTreeMap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns the union of `left` and `right`.
///
/// Vertex sets are aligned by index — the result has
/// `max(left.vcount(), right.vcount())` vertices. For each endpoint
/// pair `(u, v)`, the multiplicity in the result equals the larger of
/// the multiplicities in the two inputs.
///
/// Both inputs must agree on directedness; an undirected edge
/// `(u, v)` is canonicalised to `(min(u, v), max(u, v))` before
/// counting, while directed edges are tallied as-is.
///
/// Output edges are emitted in lexicographic `(src, tgt)` order; two
/// edges sharing the same canonicalised pair appear consecutively.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if directedness diverges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, union};
///
/// // Triangle ∪ path: edges {(0,1), (1,2), (2,0)} ∪ {(0,1), (1,3)}
/// // → {(0,1), (1,2), (2,0), (1,3)} on 4 vertices.
/// let mut a = Graph::with_vertices(3);
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let mut b = Graph::with_vertices(4);
/// b.add_edge(0, 1).unwrap();
/// b.add_edge(1, 3).unwrap();
///
/// let u = union(&a, &b).unwrap();
/// assert_eq!(u.vcount(), 4);
/// assert_eq!(u.ecount(), 4);
/// ```
pub fn union(left: &Graph, right: &Graph) -> IgraphResult<Graph> {
    if left.is_directed() != right.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "union: cannot mix directed and undirected graphs".to_string(),
        ));
    }
    let directed = left.is_directed();
    let n = std::cmp::max(left.vcount(), right.vcount());

    let canon = |u: VertexId, v: VertexId| -> (VertexId, VertexId) {
        if directed || u <= v { (u, v) } else { (v, u) }
    };

    let mut count_left: BTreeMap<(VertexId, VertexId), u32> = BTreeMap::new();
    let mut count_right: BTreeMap<(VertexId, VertexId), u32> = BTreeMap::new();

    let m_l = u32::try_from(left.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m_l {
        let (u, v) = left.edge(e as EdgeId)?;
        *count_left.entry(canon(u, v)).or_insert(0) += 1;
    }
    let m_r = u32::try_from(right.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m_r {
        let (u, v) = right.edge(e as EdgeId)?;
        *count_right.entry(canon(u, v)).or_insert(0) += 1;
    }

    // Walk the merged key set in lex order and emit the per-pair
    // multiplicity-max edges. Using BTreeMap keeps output edges
    // deterministic (sorted by `(src, tgt)`).
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut iter_l = count_left.iter().peekable();
    let mut iter_r = count_right.iter().peekable();
    loop {
        let next = match (iter_l.peek(), iter_r.peek()) {
            (None, None) => break,
            (Some((kl, _)), None) => Some((**kl, true, false)),
            (None, Some((kr, _))) => Some((**kr, false, true)),
            (Some((kl, _)), Some((kr, _))) => match kl.cmp(kr) {
                std::cmp::Ordering::Less => Some((**kl, true, false)),
                std::cmp::Ordering::Greater => Some((**kr, false, true)),
                std::cmp::Ordering::Equal => Some((**kl, true, true)),
            },
        };
        let Some((k, take_l, take_r)) = next else {
            break;
        };
        let cl = if take_l {
            *iter_l.next().expect("peek matched").1
        } else {
            0
        };
        let cr = if take_r {
            *iter_r.next().expect("peek matched").1
        } else {
            0
        };
        let m = std::cmp::max(cl, cr);
        for _ in 0..m {
            edges.push(k);
        }
    }

    let mut out = Graph::new(n, directed)?;
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
    fn empty_union_empty() {
        let a = Graph::with_vertices(0);
        let b = Graph::with_vertices(0);
        let u = union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 0);
        assert_eq!(u.ecount(), 0);
        assert!(!u.is_directed());
    }

    #[test]
    fn vcount_is_max_of_inputs() {
        // a: 3 isolated vertices, b: 6 isolated vertices.
        let a = Graph::with_vertices(3);
        let b = Graph::with_vertices(6);
        let u = union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 6);
        assert_eq!(u.ecount(), 0);
    }

    #[test]
    fn triangle_union_path_doc_example() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 0).unwrap();
        let mut b = Graph::with_vertices(4);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 3).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 4);
        assert_eq!(u.ecount(), 4);
        assert_eq!(sorted_edges(&u), vec![(0, 1), (0, 2), (1, 2), (1, 3)]);
    }

    #[test]
    fn max_multiplicity_when_left_has_more() {
        // left: 3× (0,1); right: 1× (0,1). Result: 3× (0,1).
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(u.ecount(), 3);
    }

    #[test]
    fn max_multiplicity_when_right_has_more() {
        // left: 1× (0,1); right: 5× (0,1). Result: 5× (0,1).
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(2);
        for _ in 0..5 {
            b.add_edge(0, 1).unwrap();
        }
        let u = union(&a, &b).unwrap();
        assert_eq!(u.ecount(), 5);
    }

    #[test]
    fn disjoint_edge_sets_become_simple_union() {
        // a: edges (0,1); b: edges (2,3) on 4 vertices.
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(4);
        b.add_edge(2, 3).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(sorted_edges(&u), vec![(0, 1), (2, 3)]);
    }

    #[test]
    fn idempotent_with_self() {
        // union(a, a) ≡ a (max-multiplicity: max(k, k) = k).
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(0, 2).unwrap();
        a.add_edge(0, 2).unwrap(); // multi-edge
        let u = union(&a, &a).unwrap();
        assert_eq!(u.vcount(), a.vcount());
        assert_eq!(u.ecount(), a.ecount());
        assert_eq!(sorted_edges(&u), sorted_edges(&a));
    }

    #[test]
    fn directed_keeps_orientation_separate() {
        // left: (0→1); right: (1→0). Both edges land in the result.
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(1, 0).unwrap();
        let u = union(&a, &b).unwrap();
        assert!(u.is_directed());
        assert_eq!(u.ecount(), 2);
        assert_eq!(sorted_edges(&u), vec![(0, 1), (1, 0)]);
    }

    #[test]
    fn directed_max_multiplicity_per_orientation() {
        // left: 2× (0→1); right: 3× (1→0); both kept independently.
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        for _ in 0..3 {
            b.add_edge(1, 0).unwrap();
        }
        let u = union(&a, &b).unwrap();
        assert_eq!(u.ecount(), 5);
        let s = sorted_edges(&u);
        assert_eq!(s.iter().filter(|&&p| p == (0, 1)).count(), 2);
        assert_eq!(s.iter().filter(|&&p| p == (1, 0)).count(), 3);
    }

    #[test]
    fn loops_are_preserved() {
        // left: 2× (0,0); right: 1× (0,0). Result: 2× (0,0).
        let mut a = Graph::with_vertices(1);
        a.add_edge(0, 0).unwrap();
        a.add_edge(0, 0).unwrap();
        let mut b = Graph::with_vertices(1);
        b.add_edge(0, 0).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(u.ecount(), 2);
        assert!(u.edge(0).unwrap() == (0, 0));
    }

    #[test]
    fn unaligned_vertex_sizes_use_max() {
        // a has 2 vertices, b has 5 vertices. Result must have 5 vertices.
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(5);
        b.add_edge(3, 4).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 5);
        assert_eq!(sorted_edges(&u), vec![(0, 1), (3, 4)]);
    }

    #[test]
    fn mixed_directedness_errors() {
        let a = Graph::with_vertices(2);
        let b = Graph::new(2, true).unwrap();
        assert!(union(&a, &b).is_err());
    }

    #[test]
    fn undirected_canonicalises_swapped_endpoints() {
        // Both edges encode the same undirected pair (1,0) and (0,1).
        let mut a = Graph::with_vertices(2);
        a.add_edge(1, 0).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let u = union(&a, &b).unwrap();
        // max(1,1) = 1.
        assert_eq!(u.ecount(), 1);
        let endpoints = u.edge(0).unwrap();
        assert!(endpoints == (0, 1) || endpoints == (1, 0));
    }

    #[test]
    fn larger_left_vertex_count_preserved() {
        // Mirror the previous test with sides swapped.
        let mut a = Graph::with_vertices(5);
        a.add_edge(3, 4).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let u = union(&a, &b).unwrap();
        assert_eq!(u.vcount(), 5);
        assert_eq!(sorted_edges(&u), vec![(0, 1), (3, 4)]);
    }
}
