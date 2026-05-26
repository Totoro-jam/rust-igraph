//! Full multipartite graph constructor (ALGO-CN-026).
//!
//! Counterpart of `igraph_full_multipartite()` in
//! `references/igraph/src/constructors/full.c:154-247`.
//!
//! A *complete (full) multipartite graph* on partitions of sizes
//! `n_0, n_1, …, n_{k-1}` is the graph whose vertex set is the disjoint
//! union of the `k` partitions, with an edge between every pair of
//! vertices that belong to **different** partitions and no edge between
//! vertices that share a partition.
//!
//! Vertices are laid out in partition-major order:
//!
//! ```text
//! partition 0: vertices [0,                 n_0)
//! partition 1: vertices [n_0,         n_0 + n_1)
//! …
//! partition k: vertices [Σ n_i (i < k), Σ n_i (i ≤ k))
//! ```
//!
//! Edge orientation is governed by [`MultipartiteMode`]:
//!
//! * [`MultipartiteMode::Out`] (directed only) — emit `(from, to)` where
//!   `from` lives in the earlier partition (lower partition index).
//! * [`MultipartiteMode::In`]  (directed only) — emit `(to, from)`,
//!   reversing every arc.
//! * [`MultipartiteMode::All`] — for directed graphs, emit **both**
//!   `(from, to)` and `(to, from)` per pair (mutual). For undirected
//!   graphs this is the only meaningful setting; `Out` and `In` collapse
//!   to the same edge multiset, so we accept `All` as the canonical
//!   choice and treat `Out`/`In` as equivalent.
//!
//! Edge counts:
//!
//! * Empty partition list (`k = 0`) — empty graph, no vertices.
//! * `E_undirected = Σ_{i < j} n_i · n_j`, equivalently
//!   `½ · Σ_i n_i · (N − n_i)` where `N = Σ_i n_i`.
//! * Directed `Out`/`In` arc count equals `E_undirected`.
//! * Directed `All` arc count equals `2 · E_undirected`.
//!
//! The returned [`FullMultipartite`] bundles the constructed [`Graph`]
//! with a `types` vector that records the partition index of every
//! vertex (`types[v]` is the partition `v` lives in), matching the
//! optional `types` out-parameter of the upstream C entry point.
//!
//! Degenerate cases:
//!
//! * `partitions == &[]` → empty graph (`vcount = 0`, `types == []`).
//! * Any individual `n_i == 0` → that partition contributes no vertices
//!   and no edges; later partitions are still numbered consecutively in
//!   the natural way.
//! * `partitions == &[n]` (single partition) → graph with `n` isolated
//!   vertices, no edges, `types == [0; n]`.
//!
//! Time complexity: `O(|V| + |E|)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Direction policy for [`full_multipartite`].
///
/// Mirrors `igraph_neimode_t` in the C reference, restricted to the
/// three settings the upstream `igraph_full_multipartite` handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultipartiteMode {
    /// Emit both arcs per pair (mutual). The only legal choice for
    /// undirected graphs; for directed graphs the result has exactly
    /// `2 · E_undirected` arcs.
    All,
    /// Directed: arcs flow **from** the earlier-indexed partition to
    /// the later-indexed partition. Ignored for undirected graphs.
    Out,
    /// Directed: arcs flow **from** the later-indexed partition back
    /// to the earlier-indexed partition. Ignored for undirected graphs.
    In,
}

/// Bundled return type of [`full_multipartite`].
///
/// `graph` is the constructed complete multipartite graph; `types[v]`
/// is the partition index (0-based) of vertex `v`. `types` always has
/// length equal to `graph.vcount()`.
#[derive(Debug, Clone)]
pub struct FullMultipartite {
    /// The constructed graph.
    pub graph: Graph,
    /// Per-vertex partition index.
    pub types: Vec<u32>,
}

/// Build the complete (full) multipartite graph on the given partitions.
///
/// `partitions[i]` gives the number of vertices in partition `i`.
/// The empty slice yields an empty graph and an empty `types` vector.
///
/// See the module documentation for the precise semantics of
/// [`MultipartiteMode`] and the vertex / edge layout.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the total vertex count
///   `Σ partitions[i]` overflows `u32`, or the (intermediate) edge
///   count overflows `usize`. The overflow checks mirror the upstream
///   `IGRAPH_SAFE_ADD` / `IGRAPH_SAFE_MULT` guards.
///
/// # Examples
///
/// ```
/// use rust_igraph::{full_multipartite, MultipartiteMode};
///
/// // Complete bipartite K_{2,3} on 5 vertices, undirected → 6 edges.
/// let result = full_multipartite(&[2, 3], false, MultipartiteMode::All).unwrap();
/// assert_eq!(result.graph.vcount(), 5);
/// assert_eq!(result.graph.ecount(), 6);
/// assert_eq!(result.types, vec![0, 0, 1, 1, 1]);
///
/// // Same partitions, directed mutual → 12 arcs.
/// let dir = full_multipartite(&[2, 3], true, MultipartiteMode::All).unwrap();
/// assert_eq!(dir.graph.ecount(), 12);
/// ```
pub fn full_multipartite(
    partitions: &[u32],
    directed: bool,
    mode: MultipartiteMode,
) -> IgraphResult<FullMultipartite> {
    let no_of_types = partitions.len();

    if no_of_types == 0 {
        let graph = Graph::new(0, directed)?;
        return Ok(FullMultipartite {
            graph,
            types: Vec::new(),
        });
    }

    // n_acc[i] = sum_{k < i} partitions[k]. Length = no_of_types + 1.
    // We keep it in u64 during accumulation so we can detect overflow
    // against u32::MAX, mirroring the upstream IGRAPH_SAFE_ADD chain.
    let mut n_acc: Vec<u64> = Vec::with_capacity(no_of_types + 1);
    n_acc.push(0);
    for &n_i in partitions {
        let next = n_acc[n_acc.len() - 1]
            .checked_add(u64::from(n_i))
            .ok_or_else(|| {
                IgraphError::InvalidArgument(
                    "full_multipartite: cumulative partition size overflows u64".to_string(),
                )
            })?;
        n_acc.push(next);
    }
    let total_v_u64 = n_acc[no_of_types];
    let total_v = u32::try_from(total_v_u64).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "full_multipartite: total vertex count {total_v_u64} cannot fit u32"
        ))
    })?;

    // Undirected edge count: sum_{i < j} n_i * n_j.
    // We compute it via the upstream formula:
    //   no_of_edges = (1/2) * sum_i n_i * (N - n_i)
    // For "All on directed", we keep the unhalved value (= 2·undirected).
    // We track in usize with checked arithmetic so allocation is safe.
    let total_v_usize = usize::try_from(total_v).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "full_multipartite: total vertex count {total_v} cannot fit usize"
        ))
    })?;
    let mut sum_2e: usize = 0;
    for &n_i in partitions {
        let n_i_us = n_i as usize;
        // partial = (total_v - n_i) * n_i
        let partial = total_v_usize
            .checked_sub(n_i_us)
            .and_then(|d| d.checked_mul(n_i_us))
            .ok_or_else(|| {
                IgraphError::InvalidArgument(
                    "full_multipartite: partition product overflows usize".to_string(),
                )
            })?;
        sum_2e = sum_2e.checked_add(partial).ok_or_else(|| {
            IgraphError::InvalidArgument("full_multipartite: edge sum overflows usize".to_string())
        })?;
    }
    // sum_2e is exactly 2 · E_undirected because every unordered pair
    // {i,j} contributes n_i·n_j (from i's iteration) plus n_j·n_i (from
    // j's iteration). Undirected edge count is half of that.
    let e_undirected = sum_2e / 2;
    let edge_count = if directed && mode == MultipartiteMode::All {
        sum_2e
    } else {
        e_undirected
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_count);

    // Emission walk mirrors full.c:201-225 verbatim.
    //
    //   for from_type in 0..no_of_types-1:
    //     edge_from = n_acc[from_type]
    //     for i in 0..n[from_type]:
    //       for to_type in from_type+1..no_of_types:
    //         edge_to = n_acc[to_type]
    //         for j in 0..n[to_type]:
    //           emit per (directed, mode)
    //           edge_to++
    //       edge_from++
    if no_of_types >= 2 {
        for from_type in 0..(no_of_types - 1) {
            // edge_from starts at n_acc[from_type] and increments per i.
            // The cast to VertexId is safe: n_acc[from_type] < total_v ≤ u32::MAX.
            #[allow(clippy::cast_possible_truncation)]
            let from_base = n_acc[from_type] as VertexId;
            for i in 0..partitions[from_type] {
                #[allow(clippy::cast_possible_truncation)]
                let edge_from = from_base + i as VertexId;
                for to_type in (from_type + 1)..no_of_types {
                    #[allow(clippy::cast_possible_truncation)]
                    let to_base = n_acc[to_type] as VertexId;
                    for j in 0..partitions[to_type] {
                        #[allow(clippy::cast_possible_truncation)]
                        let edge_to = to_base + j as VertexId;
                        if !directed || mode == MultipartiteMode::Out {
                            edges.push((edge_from, edge_to));
                        } else if mode == MultipartiteMode::In {
                            edges.push((edge_to, edge_from));
                        } else {
                            // directed && mode == All: emit both arcs.
                            edges.push((edge_from, edge_to));
                            edges.push((edge_to, edge_from));
                        }
                    }
                }
            }
        }
    }
    debug_assert_eq!(edges.len(), edge_count);

    // types[v] = partition index of v.
    // Mirrors the C loop: v=1; for i in 0..N: if i == n_acc[v] { v++; } types[i] = v-1.
    let mut types: Vec<u32> = Vec::with_capacity(total_v_usize);
    if total_v_usize > 0 {
        let mut v: usize = 1;
        for i in 0..total_v_usize {
            // Skip any empty partitions (n_acc[v-1] == n_acc[v] means partition v-1
            // is empty, and now i has caught up with the next partition boundary).
            while v < no_of_types && (i as u64) == n_acc[v] {
                v += 1;
            }
            // v - 1 is the partition index for vertex i; bounded by no_of_types - 1.
            let part_idx = u32::try_from(v - 1).map_err(|_| {
                IgraphError::InvalidArgument(
                    "full_multipartite: partition index overflows u32".to_string(),
                )
            })?;
            types.push(part_idx);
        }
    }
    debug_assert_eq!(types.len(), total_v_usize);

    let mut graph = Graph::new(total_v, directed)?;
    graph.add_edges(edges)?;
    Ok(FullMultipartite { graph, types })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dump_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..ec)
            .map(|e| g.edge(e).expect("edge id in range"))
            .collect()
    }

    fn canonical_undirected(g: &Graph) -> BTreeSet<(VertexId, VertexId)> {
        dump_edges(g)
            .into_iter()
            .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
            .collect()
    }

    fn partition_of(types: &[u32], v: VertexId) -> u32 {
        types[v as usize]
    }

    // -------- degenerate cases (matching .out fixtures 1, 2, 6) --------

    #[test]
    fn empty_partitions_directed_all() {
        let r = full_multipartite(&[], true, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.graph.is_directed());
        assert!(r.types.is_empty());
    }

    #[test]
    fn empty_partitions_undirected_all() {
        let r = full_multipartite(&[], false, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(!r.graph.is_directed());
        assert!(r.types.is_empty());
    }

    #[test]
    fn single_partition_n4_directed_all() {
        // .out fixture 2: directed n=[4] ALL → vcount=4, 0 edges, types=[0,0,0,0]
        let r = full_multipartite(&[4], true, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.graph.is_directed());
        assert_eq!(r.types, vec![0, 0, 0, 0]);
    }

    #[test]
    fn all_zero_partitions_directed_all() {
        // .out fixture 6: directed n=[0,0,0] ALL → vcount=0, 0 edges, types=[]
        let r = full_multipartite(&[0, 0, 0], true, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.types.is_empty());
    }

    // -------- canonical .out fixtures (3, 4, 5, 7) --------

    #[test]
    fn directed_three_partitions_all_2_3_3() {
        // .out fixture 3: directed n=[2,3,3] ALL → vcount=8, 42 arcs
        let r = full_multipartite(&[2, 3, 3], true, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 8);
        assert_eq!(r.graph.ecount(), 42);
        assert_eq!(r.types, vec![0, 0, 1, 1, 1, 2, 2, 2]);
        // Byte-stable emission order must match upstream.
        let expected: Vec<(u32, u32)> = vec![
            (0, 2),
            (2, 0),
            (0, 3),
            (3, 0),
            (0, 4),
            (4, 0),
            (0, 5),
            (5, 0),
            (0, 6),
            (6, 0),
            (0, 7),
            (7, 0),
            (1, 2),
            (2, 1),
            (1, 3),
            (3, 1),
            (1, 4),
            (4, 1),
            (1, 5),
            (5, 1),
            (1, 6),
            (6, 1),
            (1, 7),
            (7, 1),
            (2, 5),
            (5, 2),
            (2, 6),
            (6, 2),
            (2, 7),
            (7, 2),
            (3, 5),
            (5, 3),
            (3, 6),
            (6, 3),
            (3, 7),
            (7, 3),
            (4, 5),
            (5, 4),
            (4, 6),
            (6, 4),
            (4, 7),
            (7, 4),
        ];
        assert_eq!(dump_edges(&r.graph), expected);
    }

    #[test]
    fn directed_four_partitions_in_2_3_4_2() {
        // .out fixture 4: directed n=[2,3,4,2] IN → vcount=11, 44 arcs
        let r = full_multipartite(&[2, 3, 4, 2], true, MultipartiteMode::In).expect("ok");
        assert_eq!(r.graph.vcount(), 11);
        assert_eq!(r.graph.ecount(), 44);
        assert_eq!(r.types, vec![0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3]);
        let expected: Vec<(u32, u32)> = vec![
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
            (10, 0),
            (2, 1),
            (3, 1),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
            (8, 1),
            (9, 1),
            (10, 1),
            (5, 2),
            (6, 2),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (5, 3),
            (6, 3),
            (7, 3),
            (8, 3),
            (9, 3),
            (10, 3),
            (5, 4),
            (6, 4),
            (7, 4),
            (8, 4),
            (9, 4),
            (10, 4),
            (9, 5),
            (10, 5),
            (9, 6),
            (10, 6),
            (9, 7),
            (10, 7),
            (9, 8),
            (10, 8),
        ];
        assert_eq!(dump_edges(&r.graph), expected);
    }

    #[test]
    fn undirected_four_partitions_all_2_3_4_2() {
        // .out fixture 5: undirected n=[2,3,4,2] ALL → vcount=11, 44 edges
        let r = full_multipartite(&[2, 3, 4, 2], false, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 11);
        assert_eq!(r.graph.ecount(), 44);
        assert!(!r.graph.is_directed());
        assert_eq!(r.types, vec![0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3]);
    }

    #[test]
    fn directed_one_empty_partition_2_0_3() {
        // .out fixture 7: directed n=[2,0,3] ALL → vcount=5, 12 arcs
        let r = full_multipartite(&[2, 0, 3], true, MultipartiteMode::All).expect("ok");
        assert_eq!(r.graph.vcount(), 5);
        assert_eq!(r.graph.ecount(), 12);
        // Vertices 0,1 belong to partition 0; vertices 2,3,4 to partition 2.
        // Partition 1 is empty.
        assert_eq!(r.types, vec![0, 0, 2, 2, 2]);
    }

    // -------- additional cross-mode consistency tests --------

    #[test]
    fn directed_out_count_matches_undirected_for_same_partitions() {
        let parts = [3u32, 4, 2];
        let undir = full_multipartite(&parts, false, MultipartiteMode::All).expect("ok");
        let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
        let in_ = full_multipartite(&parts, true, MultipartiteMode::In).expect("ok");
        let all = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
        assert_eq!(undir.graph.ecount(), out.graph.ecount());
        assert_eq!(undir.graph.ecount(), in_.graph.ecount());
        assert_eq!(all.graph.ecount(), 2 * undir.graph.ecount());
    }

    #[test]
    fn out_and_in_have_reversed_arcs() {
        let parts = [2u32, 3, 1];
        let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
        let in_ = full_multipartite(&parts, true, MultipartiteMode::In).expect("ok");
        let out_edges: BTreeSet<(VertexId, VertexId)> =
            dump_edges(&out.graph).into_iter().collect();
        let in_reversed: BTreeSet<(VertexId, VertexId)> = dump_edges(&in_.graph)
            .into_iter()
            .map(|(a, b)| (b, a))
            .collect();
        assert_eq!(out_edges, in_reversed);
    }

    #[test]
    fn no_intra_partition_edges() {
        let parts = [2u32, 3, 4, 2];
        let r = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
        for (u, v) in dump_edges(&r.graph) {
            assert_ne!(
                partition_of(&r.types, u),
                partition_of(&r.types, v),
                "edge ({u}, {v}) connects same-partition vertices"
            );
        }
    }

    #[test]
    fn types_are_monotone_non_decreasing() {
        // Layout is partition-major so types must be non-decreasing.
        let r = full_multipartite(&[3, 0, 2, 1, 4], false, MultipartiteMode::All).expect("ok");
        for w in r.types.windows(2) {
            assert!(w[0] <= w[1], "types {:?} must be monotone", r.types);
        }
    }

    #[test]
    fn vcount_equals_partition_sum() {
        let parts = [5u32, 7, 0, 3];
        let r = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
        let want: u64 = parts.iter().map(|&x| u64::from(x)).sum();
        assert_eq!(u64::from(r.graph.vcount()), want);
    }

    #[test]
    fn directed_out_undirected_share_canonical_multiset() {
        let parts = [3u32, 4, 2];
        let undir = full_multipartite(&parts, false, MultipartiteMode::All).expect("ok");
        let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
        assert_eq!(
            canonical_undirected(&undir.graph),
            canonical_undirected(&out.graph),
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_partitions() -> impl Strategy<Value = Vec<u32>> {
        // Up to 6 partitions, each up to 8 vertices.
        prop::collection::vec(0u32..=8, 0..=6)
    }

    proptest! {
        #[test]
        fn pp_vcount_equals_partition_sum(parts in arb_partitions()) {
            let r = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
            let want: u64 = parts.iter().map(|&x| u64::from(x)).sum();
            prop_assert_eq!(r.graph.vcount() as u64, want);
            prop_assert_eq!(r.types.len() as u64, want);
        }

        #[test]
        fn pp_no_intra_partition_edges(parts in arb_partitions()) {
            let r = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
            let ec = u32::try_from(r.graph.ecount()).unwrap();
            for e in 0..ec {
                let (u, v) = r.graph.edge(e).expect("edge in range");
                prop_assert_ne!(r.types[u as usize], r.types[v as usize]);
            }
        }

        #[test]
        fn pp_all_doubles_out_ecount(parts in arb_partitions()) {
            let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
            let all = full_multipartite(&parts, true, MultipartiteMode::All).expect("ok");
            prop_assert_eq!(all.graph.ecount(), 2 * out.graph.ecount());
        }

        #[test]
        fn pp_out_undirected_same_canonical_multiset(parts in arb_partitions()) {
            let und = full_multipartite(&parts, false, MultipartiteMode::All).expect("ok");
            let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
            let canon = |g: &Graph| -> std::collections::BTreeSet<(VertexId, VertexId)> {
                let ec = u32::try_from(g.ecount()).unwrap();
                (0..ec)
                    .map(|e| g.edge(e).expect("ok"))
                    .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
                    .collect()
            };
            prop_assert_eq!(canon(&und.graph), canon(&out.graph));
        }

        #[test]
        fn pp_types_monotone_non_decreasing(parts in arb_partitions()) {
            let r = full_multipartite(&parts, false, MultipartiteMode::All).expect("ok");
            for w in r.types.windows(2) {
                prop_assert!(w[0] <= w[1]);
            }
        }

        #[test]
        fn pp_in_reverses_out(parts in arb_partitions()) {
            let out = full_multipartite(&parts, true, MultipartiteMode::Out).expect("ok");
            let in_ = full_multipartite(&parts, true, MultipartiteMode::In).expect("ok");
            prop_assert_eq!(out.graph.ecount(), in_.graph.ecount());
            let out_set: std::collections::BTreeSet<(VertexId, VertexId)> = {
                let ec = u32::try_from(out.graph.ecount()).unwrap();
                (0..ec).map(|e| out.graph.edge(e).expect("ok")).collect()
            };
            let in_reversed: std::collections::BTreeSet<(VertexId, VertexId)> = {
                let ec = u32::try_from(in_.graph.ecount()).unwrap();
                (0..ec)
                    .map(|e| in_.graph.edge(e).expect("ok"))
                    .map(|(a, b)| (b, a))
                    .collect()
            };
            prop_assert_eq!(out_set, in_reversed);
        }
    }
}
