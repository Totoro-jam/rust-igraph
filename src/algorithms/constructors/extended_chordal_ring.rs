//! Extended chordal ring constructor (ALGO-CN-028).
//!
//! Counterpart of `igraph_extended_chordal_ring()` in
//! `references/igraph/src/constructors/regular.c:868-963`.
//!
//! An *extended chordal ring* is the cycle `C_n` together with one
//! additional chord per row of a chord matrix `W`. The matrix is
//! interpreted in upstream igraph as follows:
//!
//! * `period = ncol(W)`. The period must divide `nodes`.
//! * For every vertex `i ∈ [0, nodes)` and every row `j ∈ [0, nrow(W))`
//!   the constructor emits the directed edge `(i, (i + W[j][i mod
//!   period]) mod nodes)`. Matrix entries are allowed to be negative —
//!   the result is reduced into `[0, nodes)` with Euclidean modulus.
//!
//! The total edge count is therefore `nodes + nodes * nrow(W)` (plus an
//! extra factor of `2` only inside the candidate buffer — `igraph_create`
//! still treats each `(i, v)` as a single directed/undirected edge). The
//! result is **not** simplified: matrix patterns that produce the same
//! chord twice (e.g. the article example with `W = [[4, 2], [8, 10]]` on
//! 12 nodes) intentionally retain the parallel edges so the caller can
//! preserve the published multigraph or simplify it themselves.
//!
//! Rust matrix encoding: `w` is passed as `&[&[i64]]` — a slice of rows
//! where every row has the same length (the period). An empty `w` (no
//! rows) skips the chord pass entirely and returns the bare cycle
//! `C_n`; the upstream divide-by-zero in that case is replaced with a
//! defensive no-op.
//!
//! Degenerate / error cases (matching upstream wherever it is defined):
//!
//! * `nodes < 3` → [`IgraphError::InvalidArgument`].
//! * `nrow(W) > 0` and any row has a different length than the first →
//!   [`IgraphError::InvalidArgument`] (the C version stores the matrix
//!   as a true rectangle and cannot represent ragged input — we reject
//!   it explicitly).
//! * `nrow(W) > 0` and `period == 0` → [`IgraphError::InvalidArgument`]
//!   (a zero-column matrix is not a valid chord spec).
//! * `nrow(W) > 0` and `nodes % period != 0` →
//!   [`IgraphError::InvalidArgument`].
//!
//! Time complexity: `O(|V| + |E|) = O(nodes · (1 + nrow))`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the extended chordal ring `R(nodes, W)`.
///
/// `nodes` is the cycle length (must be ≥ 3); `w` is a matrix of chord
/// offsets stored row-major as a slice of equal-length rows; `directed`
/// controls whether the resulting graph is a digraph. The cycle edges
/// always run forward `(i, i+1)` and close with `(n-1, 0)`; chord edges
/// run `(i, (i + W[j][i mod period]) mod nodes)`.
///
/// The returned graph is **not** simplified; if `W` is designed in a way
/// that produces parallel chords or self-loops, they are preserved.
///
/// # Errors
///
/// See the module-level docs for the full list. Briefly:
///
/// * [`IgraphError::InvalidArgument`] — `nodes < 3`, or `w` is non-empty
///   and ragged, or its row width is zero, or it does not divide
///   `nodes`.
///
/// # Examples
///
/// ```
/// use rust_igraph::extended_chordal_ring;
///
/// // Pentagram on 5 vertices: 5-cycle plus chords two steps ahead.
/// // W is a 1×1 matrix containing the offset 2.
/// let pent = extended_chordal_ring(5, &[&[2]], true).unwrap();
/// assert_eq!(pent.vcount(), 5);
/// assert_eq!(pent.ecount(), 10); // 5 cycle + 5 chord
/// assert!(pent.is_directed());
///
/// // Negative offsets are allowed — W = [[-3]] gives the same edge set
/// // as W = [[2]] in the directed pentagram, since (i - 3) ≡ (i + 2) mod 5.
/// let alt = extended_chordal_ring(5, &[&[-3]], true).unwrap();
/// assert_eq!(alt.ecount(), 10);
/// ```
pub fn extended_chordal_ring(nodes: u32, w: &[&[i64]], directed: bool) -> IgraphResult<Graph> {
    if nodes < 3 {
        return Err(IgraphError::InvalidArgument(
            "extended_chordal_ring: an extended chordal ring has at least 3 nodes".into(),
        ));
    }

    let nrow = w.len();
    let period = if nrow > 0 { w[0].len() } else { 0 };

    // Compute and validate the period eagerly so the chord pass can use
    // the already-narrowed u32 form without an `as`-cast.
    let period_u32 = if nrow > 0 {
        if period == 0 {
            return Err(IgraphError::InvalidArgument(
                "extended_chordal_ring: matrix W has zero columns".into(),
            ));
        }
        // Reject ragged input — upstream's `igraph_matrix_int_t` is a
        // rectangle and we want the same contract.
        if w.iter().any(|row| row.len() != period) {
            return Err(IgraphError::InvalidArgument(
                "extended_chordal_ring: all rows of W must have the same length (period)".into(),
            ));
        }
        let p = u32::try_from(period).map_err(|_| {
            IgraphError::InvalidArgument(
                "extended_chordal_ring: matrix period exceeds u32::MAX".into(),
            )
        })?;
        if nodes % p != 0 {
            return Err(IgraphError::InvalidArgument(format!(
                "extended_chordal_ring: period (W ncol = {p}) must divide nodes ({nodes})"
            )));
        }
        p
    } else {
        0
    };

    // Edge-count budget: cycle (`nodes`) + chord pass (`nodes * nrow`).
    // Use checked arithmetic so a pathological `nrow` cannot wrap.
    let nrow_u64 = nrow as u64;
    let nodes_u64 = u64::from(nodes);
    let chord_total = nodes_u64.checked_mul(nrow_u64).ok_or_else(|| {
        IgraphError::InvalidArgument("extended_chordal_ring: nodes * nrow overflows u64".into())
    })?;
    let edge_total_u64 = chord_total.checked_add(nodes_u64).ok_or_else(|| {
        IgraphError::InvalidArgument("extended_chordal_ring: total edge count overflows u64".into())
    })?;
    let edge_total = usize::try_from(edge_total_u64).map_err(|_| {
        IgraphError::InvalidArgument(
            "extended_chordal_ring: total edge count exceeds usize::MAX".into(),
        )
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_total);

    // 1) Backbone cycle C_n — emission order matches upstream:
    //    (0,1) (1,2) ... (n-2, n-1) (n-1, 0).
    for i in 0..(nodes - 1) {
        edges.push((i, i + 1));
    }
    edges.push((nodes - 1, 0));

    // 2) Chord pass. Per the C source, mpos advances every outer-i
    //    iteration and wraps at `period`; since `period | nodes`, the
    //    natural alternative is `(i % period)`, which we use directly.
    if nrow > 0 {
        let n_i64 = i64::from(nodes);
        for i in 0..nodes {
            let mpos = (i % period_u32) as usize;
            for row in w {
                let offset = row[mpos];
                let v_i64 = (i64::from(i) + offset).rem_euclid(n_i64);
                let v = u32::try_from(v_i64).map_err(|_| {
                    IgraphError::Internal("extended_chordal_ring: chord target out of u32 range")
                })?;
                edges.push((i, v));
            }
        }
    }

    let mut graph = Graph::new(nodes, directed)?;
    graph.add_edges(edges)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn edges_in_order(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn canonical_undirected_multiset(g: &Graph) -> BTreeMap<(VertexId, VertexId), usize> {
        let mut out = BTreeMap::new();
        for (u, v) in edges_in_order(g) {
            let key = if u <= v { (u, v) } else { (v, u) };
            *out.entry(key).or_insert(0) += 1;
        }
        out
    }

    #[test]
    fn rejects_nodes_less_than_three() {
        for n in [0u32, 1, 2] {
            let err = extended_chordal_ring(n, &[&[1]], false).expect_err("must reject n < 3");
            match err {
                IgraphError::InvalidArgument(_) => {}
                other => panic!("expected InvalidArgument, got {other:?}"),
            }
        }
    }

    #[test]
    fn rejects_zero_width_matrix() {
        let empty_row: &[i64] = &[];
        let err = extended_chordal_ring(6, &[empty_row], false)
            .expect_err("zero-column W must be rejected");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_ragged_matrix() {
        let err = extended_chordal_ring(6, &[&[1, 2], &[3]], false)
            .expect_err("ragged W must be rejected");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_period_not_dividing_nodes() {
        // 7 vertices with period 3 — 7 % 3 = 1.
        let err = extended_chordal_ring(7, &[&[1, 1, 1]], false)
            .expect_err("non-dividing period must be rejected");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn empty_w_returns_pure_cycle() {
        // No chord rows → bare C_5. Use the directed variant so the
        // (n-1, 0) closing arc is preserved verbatim (undirected edges
        // canonicalize endpoint order via `(lo, hi)`).
        let g = extended_chordal_ring(5, &[], true).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 5);
        assert!(g.is_directed());
        let expected: Vec<(u32, u32)> = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
        assert_eq!(edges_in_order(&g), expected);
    }

    #[test]
    fn empty_w_undirected_yields_pure_cycle_canonical() {
        let g = extended_chordal_ring(5, &[], false).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 5);
        assert!(!g.is_directed());
        let mset = canonical_undirected_multiset(&g);
        let expected: std::collections::BTreeMap<(u32, u32), usize> = [
            ((0, 1), 1),
            ((1, 2), 1),
            ((2, 3), 1),
            ((3, 4), 1),
            ((0, 4), 1),
        ]
        .into_iter()
        .collect();
        assert_eq!(mset, expected);
    }

    #[test]
    fn pentagram_directed_w_positive_matches_upstream() {
        // Upstream test 1: igraph_extended_chordal_ring(5, [[2]], directed).
        // Cycle: (0,1)(1,2)(2,3)(3,4)(4,0); chords: (0,2)(1,3)(2,4)(3,0)(4,1).
        let g = extended_chordal_ring(5, &[&[2]], true).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 10);
        assert!(g.is_directed());
        let expected: Vec<(u32, u32)> = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 0),
            (0, 2),
            (1, 3),
            (2, 4),
            (3, 0),
            (4, 1),
        ];
        assert_eq!(edges_in_order(&g), expected);
    }

    #[test]
    fn pentagram_directed_w_negative_equivalent() {
        // Upstream test 1b: W = [[-3]] should be edge-identical to [[2]]
        // because (i - 3) ≡ (i + 2) (mod 5).
        let g_pos = extended_chordal_ring(5, &[&[2]], true).expect("ok");
        let g_neg = extended_chordal_ring(5, &[&[-3]], true).expect("ok");
        assert_eq!(edges_in_order(&g_pos), edges_in_order(&g_neg));
    }

    #[test]
    fn article_example_12_undirected_produces_parallel_chords() {
        // Upstream test 2: W = [[4, 2], [8, 10]] on 12 nodes, undirected.
        // The article matrix intentionally produces double-edges in
        // igraph's interpretation: chord row 0 column 0 emits (i, i+4)
        // for even i, while chord row 1 column 0 emits (i, i+8) ≡ (i, i-4)
        // for even i — same canonical edge. Same pattern on odd i with
        // offsets 2 and 10 ≡ -2.
        let g = extended_chordal_ring(12, &[&[4, 2], &[8, 10]], false).expect("ok");
        assert_eq!(g.vcount(), 12);
        assert!(!g.is_directed());
        // Edge count: 12 cycle + 12 nodes × 2 rows = 36.
        assert_eq!(g.ecount(), 36);

        let mset = canonical_undirected_multiset(&g);
        // Backbone cycle: 12 distinct undirected edges, each appearing
        // exactly once.
        for i in 0..12u32 {
            let e = if i < 11 { (i, i + 1) } else { (0, 11) };
            assert!(
                mset.contains_key(&e),
                "expected backbone edge {e:?} to be present"
            );
        }
        // Even-side chord set: 6 unique edges, each with multiplicity 2.
        let even_chords: [(u32, u32); 6] = [(0, 4), (2, 6), (4, 8), (6, 10), (0, 8), (2, 10)];
        for &e in &even_chords {
            assert_eq!(
                mset.get(&e).copied().unwrap_or(0),
                2,
                "expected even chord {e:?} ×2"
            );
        }
        // Odd-side chord set: 6 unique edges, each with multiplicity 2.
        let odd_chords: [(u32, u32); 6] = [(1, 3), (3, 5), (5, 7), (7, 9), (9, 11), (1, 11)];
        for &e in &odd_chords {
            assert_eq!(
                mset.get(&e).copied().unwrap_or(0),
                2,
                "expected odd chord {e:?} ×2"
            );
        }
    }

    #[test]
    fn undirected_simple_w_yields_no_self_loops() {
        // For W = [[2]] on n=5, undirected, every chord (i, i+2) has
        // distinct endpoints.
        let g = extended_chordal_ring(5, &[&[2]], false).expect("ok");
        for (u, v) in edges_in_order(&g) {
            assert_ne!(u, v, "extended_chordal_ring should not self-loop here");
        }
    }

    #[test]
    fn chord_zero_offset_emits_self_loops() {
        // W = [[0]] explicitly asks for chord (i, i) — should keep them
        // (no auto-simplification).
        let g = extended_chordal_ring(4, &[&[0]], false).expect("ok");
        // 4 cycle + 4 self-loop chords.
        assert_eq!(g.ecount(), 8);
        let self_loops: usize = edges_in_order(&g).iter().filter(|(u, v)| u == v).count();
        assert_eq!(self_loops, 4);
    }

    #[test]
    fn chord_offset_equal_to_nodes_emits_self_loops() {
        // W = [[5]] on n=5 wraps back to i: every chord is a self-loop.
        let g = extended_chordal_ring(5, &[&[5]], false).expect("ok");
        assert_eq!(g.ecount(), 10);
        let self_loops: usize = edges_in_order(&g).iter().filter(|(u, v)| u == v).count();
        assert_eq!(self_loops, 5);
    }

    #[test]
    fn large_period_two_rows_directed_edge_count() {
        // n=12, W = [[1, 3, 5], [2, 4, 6]] → 12 + 12*2 = 36 edges.
        let g = extended_chordal_ring(12, &[&[1, 3, 5], &[2, 4, 6]], true).expect("ok");
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 36);
        assert!(g.is_directed());
    }

    #[test]
    fn chord_targets_use_euclidean_modulus_for_negatives() {
        // n=7, W = [[-1]]: chords (i, i-1 mod 7).
        let g = extended_chordal_ring(7, &[&[-1]], true).expect("ok");
        // Cycle: (0,1) (1,2) ... (5,6) (6,0).
        // Chords: (0,6) (1,0) (2,1) (3,2) (4,3) (5,4) (6,5).
        let expected: Vec<(u32, u32)> = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 0),
            (0, 6),
            (1, 0),
            (2, 1),
            (3, 2),
            (4, 3),
            (5, 4),
            (6, 5),
        ];
        assert_eq!(edges_in_order(&g), expected);
    }

    #[test]
    fn directed_and_undirected_share_canonical_edge_multiset() {
        // The same W must produce the same canonical (lo, hi) endpoint
        // multiset regardless of `directed`; only the per-edge ordering
        // and the digraph flag differ.
        let g_d = extended_chordal_ring(8, &[&[3, 5]], true).expect("ok");
        let g_u = extended_chordal_ring(8, &[&[3, 5]], false).expect("ok");
        assert_eq!(g_d.ecount(), g_u.ecount());
        assert_eq!(
            canonical_undirected_multiset(&g_d),
            canonical_undirected_multiset(&g_u),
        );
    }

    #[test]
    fn cycle_is_always_present_in_emission_order() {
        let g = extended_chordal_ring(6, &[&[2, 4]], true).expect("ok");
        let first_six: Vec<_> = edges_in_order(&g).into_iter().take(6).collect();
        let expected: Vec<(u32, u32)> = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)];
        assert_eq!(first_six, expected);
    }

    #[test]
    fn isolated_period_one_chord_offset_one_is_doubled_cycle() {
        // W = [[1]] on n=5, directed: chord (i, i+1) duplicates each
        // backbone arc — total 10 edges with every (i, i+1) arc appearing
        // twice.
        let g = extended_chordal_ring(5, &[&[1]], true).expect("ok");
        assert_eq!(g.ecount(), 10);
        // Count multiplicity per (i, i+1) directed arc.
        let mut counts = BTreeMap::<(u32, u32), usize>::new();
        for (u, v) in edges_in_order(&g) {
            *counts.entry((u, v)).or_insert(0) += 1;
        }
        for i in 0..5u32 {
            let arc = (i, (i + 1) % 5);
            assert_eq!(
                counts.get(&arc).copied().unwrap_or(0),
                2,
                "arc {arc:?} should appear twice"
            );
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    fn rect_matrix() -> impl Strategy<Value = Vec<Vec<i64>>> {
        (1usize..4, 1usize..4).prop_flat_map(|(rows, cols)| {
            let cell = -50i64..50;
            proptest::collection::vec(proptest::collection::vec(cell, cols), rows)
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn vcount_always_nodes(n in 3u32..30, w in rect_matrix(), directed in any::<bool>()) {
            // Truncate `n` so it is a multiple of the matrix period.
            let period = w[0].len() as u32;
            let n_adj = ((n / period).max(1)) * period;
            if n_adj < 3 { return Ok(()); }
            let w_refs: Vec<&[i64]> = w.iter().map(|r| r.as_slice()).collect();
            let g = extended_chordal_ring(n_adj, &w_refs, directed).expect("ok");
            prop_assert_eq!(g.vcount(), n_adj);
        }

        #[test]
        fn ecount_equals_nodes_times_one_plus_nrow(
            n in 3u32..30,
            w in rect_matrix(),
            directed in any::<bool>(),
        ) {
            let period = w[0].len() as u32;
            let n_adj = ((n / period).max(1)) * period;
            if n_adj < 3 { return Ok(()); }
            let nrow = w.len();
            let w_refs: Vec<&[i64]> = w.iter().map(|r| r.as_slice()).collect();
            let g = extended_chordal_ring(n_adj, &w_refs, directed).expect("ok");
            let expected = (n_adj as usize) * (1 + nrow);
            prop_assert_eq!(g.ecount(), expected);
        }

        #[test]
        fn directed_flag_propagates(
            n in 3u32..20,
            w in rect_matrix(),
            directed in any::<bool>(),
        ) {
            let period = w[0].len() as u32;
            let n_adj = ((n / period).max(1)) * period;
            if n_adj < 3 { return Ok(()); }
            let w_refs: Vec<&[i64]> = w.iter().map(|r| r.as_slice()).collect();
            let g = extended_chordal_ring(n_adj, &w_refs, directed).expect("ok");
            prop_assert_eq!(g.is_directed(), directed);
        }

        #[test]
        fn empty_w_is_pure_cycle(n in 3u32..30, directed in any::<bool>()) {
            let g = extended_chordal_ring(n, &[], directed).expect("ok");
            prop_assert_eq!(g.ecount(), n as usize);
        }

        #[test]
        fn chord_targets_are_in_range(
            n in 3u32..30,
            w in rect_matrix(),
        ) {
            let period = w[0].len() as u32;
            let n_adj = ((n / period).max(1)) * period;
            if n_adj < 3 { return Ok(()); }
            let w_refs: Vec<&[i64]> = w.iter().map(|r| r.as_slice()).collect();
            let g = extended_chordal_ring(n_adj, &w_refs, true).expect("ok");
            let m = u32::try_from(g.ecount()).unwrap();
            for e in 0..m {
                let (u, v) = g.edge(e).unwrap();
                prop_assert!(u < n_adj);
                prop_assert!(v < n_adj);
            }
        }

        #[test]
        fn negative_and_positive_offsets_agree_mod_nodes(
            n in 3u32..15,
        ) {
            // For any single-row W with offset k, the result is
            // edge-identical to W with offset (k - n).
            for k in 1..(n as i64) {
                let w_pos = vec![vec![k]];
                let w_neg = vec![vec![k - n as i64]];
                let pos_refs: Vec<&[i64]> = w_pos.iter().map(|r| r.as_slice()).collect();
                let neg_refs: Vec<&[i64]> = w_neg.iter().map(|r| r.as_slice()).collect();
                let g_pos = extended_chordal_ring(n, &pos_refs, true).expect("ok");
                let g_neg = extended_chordal_ring(n, &neg_refs, true).expect("ok");
                let m = u32::try_from(g_pos.ecount()).unwrap();
                for e in 0..m {
                    prop_assert_eq!(g_pos.edge(e).unwrap(), g_neg.edge(e).unwrap());
                }
            }
        }
    }
}
