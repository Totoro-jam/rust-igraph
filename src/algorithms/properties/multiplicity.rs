//! Companion predicates to [`super::is_simple::is_simple`] (ALGO-PR-014).
//!
//! - [`has_loop`]      — does the graph contain at least one self-loop?
//! - [`has_multiple`]  — does the graph contain at least one parallel
//!   (multi-) edge?
//! - [`is_loop`]       — per-edge: which edges are self-loops?
//! - [`is_multiple`]   — per-edge: which edges are parallel duplicates?
//! - [`count_loops`]   — number of self-loop edges (PR-014c).
//! - [`count_multiple`] — per-edge multiplicity, the number of edges
//!   sharing this edge's endpoint pair (PR-014c).
//!
//! Counterparts of `igraph_has_loop()` / `igraph_count_loops()` from
//! `references/igraph/src/properties/loops.c` and `igraph_has_multiple()` /
//! `igraph_count_multiple()` from `references/igraph/src/properties/multiplicity.c`.
//!
//! Both predicate variants run in O(|V| + |E|) (the upstream cache
//! subsystem we don't have yet would shortcut to O(1) on subsequent
//! calls — that's an ALGO-CORE-001f responsibility).

use crate::core::cache::CachedProperty;
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
    if let Some(v) = graph.cache_get(CachedProperty::HasLoop) {
        return Ok(v);
    }
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    for e in 0..m {
        let (u, v) = graph.edge(e as EdgeId)?;
        if u == v {
            graph.cache_set(CachedProperty::HasLoop, true);
            return Ok(true);
        }
    }
    graph.cache_set(CachedProperty::HasLoop, false);
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
    if let Some(v) = graph.cache_get(CachedProperty::HasMulti) {
        return Ok(v);
    }
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    if m < 2 {
        graph.cache_set(CachedProperty::HasMulti, false);
        return Ok(false);
    }
    let mut pairs: Vec<(u32, u32)> = Vec::with_capacity(m as usize);
    for e in 0..m {
        pairs.push(graph.edge(e as EdgeId)?);
    }
    pairs.sort_unstable();
    for i in 1..pairs.len() {
        if pairs[i] == pairs[i - 1] {
            graph.cache_set(CachedProperty::HasMulti, true);
            return Ok(true);
        }
    }
    graph.cache_set(CachedProperty::HasMulti, false);
    Ok(false)
}

/// Returns a per-edge boolean vector marking self-loops.
///
/// `result[e] == true` iff `graph.edge(e) == (v, v)` for some `v`.
/// Counterpart of `igraph_is_loop()` from
/// `references/igraph/src/properties/loops.c` (with `es =
/// igraph_ess_all()`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_loop};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(2, 2).unwrap();
/// assert_eq!(is_loop(&g).unwrap(), vec![false, true]);
/// ```
pub fn is_loop(graph: &Graph) -> IgraphResult<Vec<bool>> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let mut out = Vec::with_capacity(m as usize);
    for e in 0..m {
        let (u, v) = graph.edge(e as EdgeId)?;
        out.push(u == v);
    }
    Ok(out)
}

/// Returns a per-edge boolean vector marking multiple (parallel) edges.
///
/// `result[e] == true` iff there is another edge with the same canonical
/// endpoint pair *and a smaller edge id*. Per upstream
/// `igraph_is_multiple()`'s contract (loops.c:230): the result is true
/// "only for the second or more appearances" — the canonical/first
/// occurrence stays `false`, parallel copies after it are `true`.
///
/// O(|E| log |E|) via sort by canonical pair (within each pair group
/// we keep edges in their natural id order, so the first id stays
/// `false`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_multiple};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// // Edge 0 is the canonical (0,1); edge 1 is the duplicate.
/// assert_eq!(is_multiple(&g).unwrap(), vec![false, true, false]);
/// ```
pub fn is_multiple(graph: &Graph) -> IgraphResult<Vec<bool>> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let m_us = m as usize;
    if m_us == 0 {
        return Ok(Vec::new());
    }
    // Pull the original edges, then sort by canonical (from, to) with
    // edge id as tiebreaker so the first-occurring id stays first.
    let mut pairs: Vec<((u32, u32), u32)> = Vec::with_capacity(m_us);
    for e in 0..m {
        pairs.push((graph.edge(e as EdgeId)?, e));
    }
    pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut out = vec![false; m_us];
    let mut i = 0usize;
    while i < m_us {
        let mut j = i + 1;
        while j < m_us && pairs[j].0 == pairs[i].0 {
            j += 1;
        }
        // Skip the canonical (first) edge in this group — leave it false.
        for entry in &pairs[i + 1..j] {
            out[entry.1 as usize] = true;
        }
        i = j;
    }
    Ok(out)
}

/// Counts self-loop edges in `graph`.
///
/// A self-loop is an edge `(v, v)`. Returns the total number of such
/// edges, counting parallel self-loops separately.
///
/// Counterpart of `igraph_count_loops()` from
/// `references/igraph/src/properties/loops.c`.
///
/// O(|E|) linear scan.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_loops};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 1).unwrap();
/// g.add_edge(2, 2).unwrap();
/// g.add_edge(2, 2).unwrap();
/// assert_eq!(count_loops(&g).unwrap(), 3);
/// ```
pub fn count_loops(graph: &Graph) -> IgraphResult<usize> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let mut count = 0usize;
    for e in 0..m {
        let (u, v) = graph.edge(e as EdgeId)?;
        if u == v {
            count += 1;
        }
    }
    Ok(count)
}

/// Per-edge multiplicity: how many edges share each edge's endpoint pair.
///
/// `result[e]` is the number of edges `e'` (including `e` itself) whose
/// endpoint pair matches `e`'s. Storage already canonicalises undirected
/// pairs to `(min, max)`, so undirected `(a,b)` and `(b,a)` collapse to
/// the same group; directed pairs are kept ordered.
///
/// Self-loops at the same vertex group together — each such loop has
/// multiplicity equal to the number of loops at that vertex (matching
/// upstream's `IGRAPH_LOOPS_ONCE` lazy-adjlist semantics).
///
/// Counterpart of `igraph_count_multiple()` from
/// `references/igraph/src/properties/multiplicity.c`.
///
/// O(|E| log |E|) via sort-and-group over stored edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_multiple};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// // Edge 0 and edge 1 share (0,1) → multiplicity 2 each.
/// // Edge 2 is alone → multiplicity 1.
/// assert_eq!(count_multiple(&g).unwrap(), vec![2, 2, 1]);
/// ```
pub fn count_multiple(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let m = u32::try_from(graph.ecount())
        .map_err(|_| crate::IgraphError::Internal("ecount exceeds u32::MAX"))?;
    let m_us = m as usize;
    if m_us == 0 {
        return Ok(Vec::new());
    }
    let mut pairs: Vec<((u32, u32), u32)> = Vec::with_capacity(m_us);
    for e in 0..m {
        pairs.push((graph.edge(e as EdgeId)?, e));
    }
    pairs.sort_unstable_by_key(|p| p.0);
    let mut out = vec![0usize; m_us];
    let mut i = 0usize;
    while i < m_us {
        let mut j = i + 1;
        while j < m_us && pairs[j].0 == pairs[i].0 {
            j += 1;
        }
        let group_size = j - i;
        for entry in &pairs[i..j] {
            out[entry.1 as usize] = group_size;
        }
        i = j;
    }
    Ok(out)
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
    fn is_loop_per_edge_marks_self_loops_only() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(is_loop(&g).unwrap(), vec![false, true, false]);
    }

    #[test]
    fn is_loop_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_loop(&g).unwrap().is_empty());
    }

    #[test]
    fn is_multiple_per_edge_marks_only_duplicates_after_first() {
        // Per upstream's "second-or-more" contract, edge 0 (canonical
        // (0,1)) stays false; edge 1 (the duplicate) is true; edge 2
        // (lone (1,2)) is false.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(is_multiple(&g).unwrap(), vec![false, true, false]);
    }

    #[test]
    fn is_multiple_directed_mutual_pair_not_multiple() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(is_multiple(&g).unwrap(), vec![false, false]);
    }

    #[test]
    fn is_multiple_three_copies_first_canonical_only() {
        // Three parallel edges → first one stays canonical, the next
        // two flip to true.
        let mut g = Graph::with_vertices(2);
        for _ in 0..3 {
            g.add_edge(0, 1).unwrap();
        }
        assert_eq!(is_multiple(&g).unwrap(), vec![false, true, true]);
    }

    #[test]
    fn is_multiple_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_multiple(&g).unwrap().is_empty());
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

    #[test]
    fn count_loops_empty_graph() {
        let g = Graph::with_vertices(0);
        assert_eq!(count_loops(&g).unwrap(), 0);
    }

    #[test]
    fn count_loops_no_loops() {
        let mut g = Graph::with_vertices(4);
        for (u, v) in [(0, 1), (1, 2), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(count_loops(&g).unwrap(), 0);
    }

    #[test]
    fn count_loops_counts_each_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        g.add_edge(2, 2).unwrap();
        g.add_edge(2, 2).unwrap();
        assert_eq!(count_loops(&g).unwrap(), 3);
    }

    #[test]
    fn count_loops_directed_self_loops() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 2).unwrap();
        assert_eq!(count_loops(&g).unwrap(), 2);
    }

    #[test]
    fn count_multiple_empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(count_multiple(&g).unwrap().is_empty());
    }

    #[test]
    fn count_multiple_simple_graph_all_ones() {
        let mut g = Graph::with_vertices(4);
        for (u, v) in [(0, 1), (1, 2), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(count_multiple(&g).unwrap(), vec![1, 1, 1]);
    }

    #[test]
    fn count_multiple_groups_parallel_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        // Undirected (1,0) canonicalises to (0,1); both share group → multiplicity 2.
        assert_eq!(count_multiple(&g).unwrap(), vec![2, 2, 1]);
    }

    #[test]
    fn count_multiple_directed_mutual_pair_distinct() {
        // Directed (0,1) and (1,0) are distinct pairs → multiplicity 1 each.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(count_multiple(&g).unwrap(), vec![1, 1]);
    }

    #[test]
    fn count_multiple_three_parallel_copies() {
        let mut g = Graph::with_vertices(2);
        for _ in 0..3 {
            g.add_edge(0, 1).unwrap();
        }
        assert_eq!(count_multiple(&g).unwrap(), vec![3, 3, 3]);
    }

    #[test]
    fn count_multiple_self_loops_grouped_per_vertex() {
        // Two self-loops at v=0 → each has multiplicity 2.
        // One self-loop at v=2 → multiplicity 1.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(2, 2).unwrap();
        assert_eq!(count_multiple(&g).unwrap(), vec![2, 2, 1]);
    }

    #[test]
    fn count_multiple_mixed_graph() {
        // Mix of simple, parallel, and loop edges.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(0, 1).unwrap(); // 2 — parallel of 0
        g.add_edge(3, 3).unwrap(); // 3 — lone loop
        g.add_edge(2, 3).unwrap(); // 4
        assert_eq!(count_multiple(&g).unwrap(), vec![2, 1, 2, 1, 1]);
    }

    #[test]
    fn count_multiple_length_matches_ecount() {
        let mut g = Graph::with_vertices(5);
        for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 4), (0, 4)] {
            g.add_edge(u, v).unwrap();
        }
        assert_eq!(count_multiple(&g).unwrap().len(), g.ecount());
    }

    #[test]
    fn count_multiple_consistent_with_is_multiple_and_has_multiple() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mults = count_multiple(&g).unwrap();
        let is_mult = is_multiple(&g).unwrap();
        // is_multiple is "second-or-more occurrence"; multiplicity > 1
        // exactly when at least one of the group's edges is is_multiple.
        let has_mult_via_count = mults.iter().any(|&m| m > 1);
        assert_eq!(has_mult_via_count, has_multiple(&g).unwrap());
        // For each edge e: is_multiple[e] implies count_multiple[e] > 1.
        for (e, &flag) in is_mult.iter().enumerate() {
            if flag {
                assert!(mults[e] > 1, "is_multiple but mult = {}", mults[e]);
            }
        }
    }

    #[test]
    fn count_loops_consistent_with_is_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let n_loops = count_loops(&g).unwrap();
        let is_l = is_loop(&g).unwrap();
        assert_eq!(n_loops, is_l.iter().filter(|&&b| b).count());
    }
}
