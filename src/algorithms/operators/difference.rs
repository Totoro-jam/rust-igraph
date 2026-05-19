//! Difference of two graphs (ALGO-OP-006).
//!
//! Counterpart of `igraph_difference()` from
//! `references/igraph/src/operators/difference.c:54-181`. The result
//! has exactly `orig.vcount()` vertices (the **left** operand only —
//! this is intentionally asymmetric, unlike
//! [`crate::union`] / [`crate::intersection`] which take the max). For
//! each canonicalised endpoint pair the result keeps
//! `max(0, count_orig - count_sub)` copies, i.e. multiset subtraction
//! clamped to zero.
//!
//! For undirected inputs the canonicalised pair is `(min(u,v),
//! max(u,v))`; for directed inputs the pair is taken as-is, so
//! `(u, v)` and `(v, u)` are subtracted independently.
//!
//! Phase-1 minimal slice: two-graph variant only. The upstream
//! `edge_ids` permutation output (used by C for attribute permutation)
//! is not yet exposed.

use std::collections::BTreeMap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns `orig \ sub`: the multiset difference of the edges.
///
/// The result has `orig.vcount()` vertices — vertices in `sub` beyond
/// `orig.vcount()` are simply ignored. For each endpoint pair `(u, v)`,
/// the multiplicity in the result equals
/// `max(0, count_orig - count_sub)`.
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
/// use rust_igraph::{Graph, difference};
///
/// // Triangle \ path: edges {(0,1), (1,2), (2,0)} \ {(0,1), (1,2)}
/// // → {(0,2)} on 3 vertices (canonicalised (0,2) for undirected).
/// let mut a = Graph::with_vertices(3);
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let mut b = Graph::with_vertices(3);
/// b.add_edge(0, 1).unwrap();
/// b.add_edge(1, 2).unwrap();
///
/// let d = difference(&a, &b).unwrap();
/// assert_eq!(d.vcount(), 3);
/// assert_eq!(d.ecount(), 1);
/// ```
pub fn difference(orig: &Graph, sub: &Graph) -> IgraphResult<Graph> {
    if orig.is_directed() != sub.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "difference: cannot mix directed and undirected graphs".to_string(),
        ));
    }
    let directed = orig.is_directed();
    let n = orig.vcount();

    let canon = |u: VertexId, v: VertexId| -> (VertexId, VertexId) {
        if directed || u <= v { (u, v) } else { (v, u) }
    };

    let mut count_orig: BTreeMap<(VertexId, VertexId), u32> = BTreeMap::new();
    let mut count_sub: BTreeMap<(VertexId, VertexId), u32> = BTreeMap::new();

    let m_o = u32::try_from(orig.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m_o {
        let (u, v) = orig.edge(e as EdgeId)?;
        *count_orig.entry(canon(u, v)).or_insert(0) += 1;
    }
    let m_s = u32::try_from(sub.ecount())
        .map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m_s {
        let (u, v) = sub.edge(e as EdgeId)?;
        // Pairs touching vertices not in `orig` cannot match any orig
        // pair anyway, but we still tally — the lookup below simply
        // returns 0 for non-keys.
        *count_sub.entry(canon(u, v)).or_insert(0) += 1;
    }

    // Walk orig's keys (already in lex order) and subtract sub's
    // multiplicity, clamping to 0. Pairs unique to `sub` never appear
    // in the output.
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    for (k, &co) in &count_orig {
        let cs = count_sub.get(k).copied().unwrap_or(0);
        let m = co.saturating_sub(cs);
        for _ in 0..m {
            edges.push(*k);
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
    fn empty_minus_empty() {
        let a = Graph::with_vertices(0);
        let b = Graph::with_vertices(0);
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 0);
        assert_eq!(d.ecount(), 0);
        assert!(!d.is_directed());
    }

    #[test]
    fn vcount_is_orig_only() {
        // sub has more vertices than orig — result still uses orig.vcount().
        let a = Graph::with_vertices(3);
        let b = Graph::with_vertices(7);
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 3);
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn vcount_when_orig_larger() {
        let a = Graph::with_vertices(7);
        let b = Graph::with_vertices(3);
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 7);
    }

    #[test]
    fn triangle_minus_path_doc_example() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 0).unwrap();
        let mut b = Graph::with_vertices(3);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 2).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 3);
        assert_eq!(d.ecount(), 1);
        assert_eq!(sorted_edges(&d), vec![(0, 2)]);
    }

    #[test]
    fn self_difference_is_empty() {
        // diff(a, a) ≡ empty edge set on a.vcount() vertices.
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(0, 2).unwrap();
        a.add_edge(0, 2).unwrap(); // multi-edge
        let d = difference(&a, &a).unwrap();
        assert_eq!(d.vcount(), 4);
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn difference_with_empty_is_identity_edges() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        let b = Graph::with_vertices(3);
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 3);
        assert_eq!(sorted_edges(&d), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn empty_minus_anything_is_empty() {
        let a = Graph::with_vertices(3);
        let mut b = Graph::with_vertices(3);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 2).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 3);
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn pair_unique_to_sub_does_not_appear() {
        // sub has (2,3) which orig lacks — must not appear in result.
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(4);
        b.add_edge(2, 3).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 4);
        assert_eq!(sorted_edges(&d), vec![(0, 1)]);
    }

    #[test]
    fn multiplicity_clamps_to_zero() {
        // orig: 2× (0,1); sub: 5× (0,1). Result: 0 edges.
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(2);
        for _ in 0..5 {
            b.add_edge(0, 1).unwrap();
        }
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn multiplicity_partial_subtraction() {
        // orig: 5× (0,1); sub: 2× (0,1). Result: 3× (0,1).
        let mut a = Graph::with_vertices(2);
        for _ in 0..5 {
            a.add_edge(0, 1).unwrap();
        }
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        b.add_edge(0, 1).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.ecount(), 3);
        assert!(sorted_edges(&d).iter().all(|&p| p == (0, 1)));
    }

    #[test]
    fn directed_orientations_are_independent() {
        // orig: 0→1 + 1→0; sub: 0→1 only. Result: 1→0.
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 0).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(0, 1).unwrap();
        let d = difference(&a, &b).unwrap();
        assert!(d.is_directed());
        assert_eq!(d.ecount(), 1);
        assert_eq!(d.edge(0).unwrap(), (1, 0));
    }

    #[test]
    fn directed_per_orientation_multiplicity() {
        // orig: 4× (0→1), 2× (1→0); sub: 1× (0→1), 3× (1→0).
        // Result: 3× (0→1), 0× (1→0) → 3 edges total.
        let mut a = Graph::new(2, true).unwrap();
        for _ in 0..4 {
            a.add_edge(0, 1).unwrap();
        }
        for _ in 0..2 {
            a.add_edge(1, 0).unwrap();
        }
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(0, 1).unwrap();
        for _ in 0..3 {
            b.add_edge(1, 0).unwrap();
        }
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.ecount(), 3);
        assert!(sorted_edges(&d).iter().all(|&p| p == (0, 1)));
    }

    #[test]
    fn loops_subtract_correctly() {
        // orig: 3× (0,0); sub: 1× (0,0). Result: 2× (0,0).
        let mut a = Graph::with_vertices(1);
        for _ in 0..3 {
            a.add_edge(0, 0).unwrap();
        }
        let mut b = Graph::with_vertices(1);
        b.add_edge(0, 0).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.ecount(), 2);
        assert!(sorted_edges(&d).iter().all(|&p| p == (0, 0)));
    }

    #[test]
    fn mixed_directedness_errors() {
        let a = Graph::with_vertices(2);
        let b = Graph::new(2, true).unwrap();
        assert!(difference(&a, &b).is_err());
    }

    #[test]
    fn undirected_canonicalises_swapped_endpoints() {
        // orig has (1,0); sub has (0,1). Both encode the same
        // undirected pair → result is empty.
        let mut a = Graph::with_vertices(2);
        a.add_edge(1, 0).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.ecount(), 0);
    }

    #[test]
    fn sub_high_index_vertex_ignored() {
        // sub has an edge (3,4) but orig only has 3 vertices, so the
        // pair simply isn't a key in count_orig → ignored.
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        let mut b = Graph::with_vertices(5);
        b.add_edge(0, 1).unwrap();
        b.add_edge(3, 4).unwrap();
        let d = difference(&a, &b).unwrap();
        assert_eq!(d.vcount(), 3);
        assert_eq!(sorted_edges(&d), vec![(1, 2)]);
    }
}
