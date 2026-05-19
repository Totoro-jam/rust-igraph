//! Intersection of two graphs (ALGO-OP-005).
//!
//! Counterpart of `igraph_intersection()` from
//! `references/igraph/src/operators/intersection.c:71-77`. Vertex sets
//! are aligned by index — the result has
//! `max(left.vcount(), right.vcount())` vertices (matching upstream's
//! "common edges, larger vertex set" contract). Edges are intersected
//! by *minimum multiplicity*: for each (canonicalised) endpoint pair
//! `(u, v)`, the result contains `min(count_left, count_right)` edges,
//! so a pair only survives when present in BOTH inputs.
//!
//! For undirected inputs the canonicalised pair is `(min(u,v),
//! max(u,v))`; for directed inputs the pair is taken as-is, so
//! `(u, v)` and `(v, u)` are tallied separately.
//!
//! Phase-1 minimal slice: two-graph variant only. Multi-arg
//! `intersection_many` and the edge-mapping outputs (`edge_map1` /
//! `edge_map2`) ship later.

use std::collections::BTreeMap;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Returns the intersection of `left` and `right`.
///
/// Vertex sets are aligned by index — the result has
/// `max(left.vcount(), right.vcount())` vertices. For each endpoint
/// pair `(u, v)`, the multiplicity in the result equals the smaller of
/// the multiplicities in the two inputs (so pairs unique to one side
/// are dropped).
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
/// use rust_igraph::{Graph, intersection};
///
/// // Triangle ∩ path: edges {(0,1), (1,2), (2,0)} ∩ {(0,1), (1,2)}
/// // → {(0,1), (1,2)} on max(3, 3) = 3 vertices.
/// let mut a = Graph::with_vertices(3);
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// a.add_edge(2, 0).unwrap();
/// let mut b = Graph::with_vertices(3);
/// b.add_edge(0, 1).unwrap();
/// b.add_edge(1, 2).unwrap();
///
/// let i = intersection(&a, &b).unwrap();
/// assert_eq!(i.vcount(), 3);
/// assert_eq!(i.ecount(), 2);
/// ```
pub fn intersection(left: &Graph, right: &Graph) -> IgraphResult<Graph> {
    if left.is_directed() != right.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "intersection: cannot mix directed and undirected graphs".to_string(),
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

    // Walk the smaller map and look up matches in the other; only pairs
    // present in BOTH contribute. Iterating the smaller side in BTreeMap
    // order keeps output deterministic without materialising the merged
    // key set.
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let (driver, lookup) = if count_left.len() <= count_right.len() {
        (&count_left, &count_right)
    } else {
        (&count_right, &count_left)
    };
    for (k, &cd) in driver {
        if let Some(&co) = lookup.get(k) {
            let m = std::cmp::min(cd, co);
            for _ in 0..m {
                edges.push(*k);
            }
        }
    }
    // The driver may be `count_right`, in which case we walked the
    // intersection in `right`'s key order — that's still the same set
    // of pairs. But for stable output across left/right swaps, sort.
    edges.sort_unstable();

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
    fn empty_intersect_empty() {
        let a = Graph::with_vertices(0);
        let b = Graph::with_vertices(0);
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.vcount(), 0);
        assert_eq!(i.ecount(), 0);
        assert!(!i.is_directed());
    }

    #[test]
    fn vcount_is_max_of_inputs() {
        let a = Graph::with_vertices(3);
        let b = Graph::with_vertices(7);
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.vcount(), 7);
        assert_eq!(i.ecount(), 0);
    }

    #[test]
    fn triangle_intersect_path_doc_example() {
        let mut a = Graph::with_vertices(3);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 0).unwrap();
        let mut b = Graph::with_vertices(3);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 2).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.vcount(), 3);
        assert_eq!(i.ecount(), 2);
        assert_eq!(sorted_edges(&i), vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn min_multiplicity_when_left_has_more() {
        // left: 3× (0,1); right: 1× (0,1). Result: 1× (0,1).
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.ecount(), 1);
    }

    #[test]
    fn min_multiplicity_when_right_has_more() {
        // left: 2× (0,1); right: 5× (0,1). Result: 2× (0,1).
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(2);
        for _ in 0..5 {
            b.add_edge(0, 1).unwrap();
        }
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.ecount(), 2);
    }

    #[test]
    fn disjoint_edge_sets_yields_empty() {
        // No shared pair → empty graph (max-vcount preserved).
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(4);
        b.add_edge(2, 3).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.vcount(), 4);
        assert_eq!(i.ecount(), 0);
    }

    #[test]
    fn idempotent_with_self() {
        // intersection(a, a) ≡ a (min(k, k) = k).
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(0, 2).unwrap();
        a.add_edge(0, 2).unwrap(); // multi-edge
        let i = intersection(&a, &a).unwrap();
        assert_eq!(i.vcount(), a.vcount());
        assert_eq!(i.ecount(), a.ecount());
        assert_eq!(sorted_edges(&i), sorted_edges(&a));
    }

    #[test]
    fn directed_keeps_orientation_separate() {
        // left: 0→1 + 1→0; right: 0→1 only. Intersection: 0→1.
        let mut a = Graph::new(2, true).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 0).unwrap();
        let mut b = Graph::new(2, true).unwrap();
        b.add_edge(0, 1).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert!(i.is_directed());
        assert_eq!(i.ecount(), 1);
        assert_eq!(i.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn directed_min_multiplicity_per_orientation() {
        // left: 3× (0→1), 2× (1→0); right: 2× (0→1), 4× (1→0).
        // Result: 2× (0→1), 2× (1→0) → 4 edges total.
        let mut a = Graph::new(2, true).unwrap();
        for _ in 0..3 {
            a.add_edge(0, 1).unwrap();
        }
        for _ in 0..2 {
            a.add_edge(1, 0).unwrap();
        }
        let mut b = Graph::new(2, true).unwrap();
        for _ in 0..2 {
            b.add_edge(0, 1).unwrap();
        }
        for _ in 0..4 {
            b.add_edge(1, 0).unwrap();
        }
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.ecount(), 4);
        let s = sorted_edges(&i);
        assert_eq!(s.iter().filter(|&&p| p == (0, 1)).count(), 2);
        assert_eq!(s.iter().filter(|&&p| p == (1, 0)).count(), 2);
    }

    #[test]
    fn loops_are_preserved_with_min_multiplicity() {
        // left: 3× (0,0); right: 1× (0,0). Result: 1× (0,0).
        let mut a = Graph::with_vertices(1);
        for _ in 0..3 {
            a.add_edge(0, 0).unwrap();
        }
        let mut b = Graph::with_vertices(1);
        b.add_edge(0, 0).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.ecount(), 1);
        assert_eq!(i.edge(0).unwrap(), (0, 0));
    }

    #[test]
    fn unaligned_vertex_sizes_use_max() {
        // a has 2 vertices, b has 5 vertices, no shared edges.
        let mut a = Graph::with_vertices(2);
        a.add_edge(0, 1).unwrap();
        let mut b = Graph::with_vertices(5);
        b.add_edge(3, 4).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.vcount(), 5);
        assert_eq!(i.ecount(), 0);
    }

    #[test]
    fn mixed_directedness_errors() {
        let a = Graph::with_vertices(2);
        let b = Graph::new(2, true).unwrap();
        assert!(intersection(&a, &b).is_err());
    }

    #[test]
    fn undirected_canonicalises_swapped_endpoints() {
        // left has (1,0); right has (0,1). Both encode the same
        // undirected pair → intersection has 1× that pair.
        let mut a = Graph::with_vertices(2);
        a.add_edge(1, 0).unwrap();
        let mut b = Graph::with_vertices(2);
        b.add_edge(0, 1).unwrap();
        let i = intersection(&a, &b).unwrap();
        assert_eq!(i.ecount(), 1);
    }

    #[test]
    fn order_independent() {
        // intersection(a, b) and intersection(b, a) produce the same
        // edge multiset (commutative).
        let mut a = Graph::with_vertices(4);
        a.add_edge(0, 1).unwrap();
        a.add_edge(0, 1).unwrap();
        a.add_edge(1, 2).unwrap();
        a.add_edge(2, 3).unwrap();
        let mut b = Graph::with_vertices(4);
        b.add_edge(0, 1).unwrap();
        b.add_edge(1, 2).unwrap();
        b.add_edge(1, 2).unwrap();
        let ab = intersection(&a, &b).unwrap();
        let ba = intersection(&b, &a).unwrap();
        assert_eq!(sorted_edges(&ab), sorted_edges(&ba));
    }
}
