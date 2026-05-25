//! LCF (Lederberg-Coxeter-Frucht) cubic-graph constructor (ALGO-CN-018).
//!
//! Counterpart of `igraph_lcf()` in
//! `references/igraph/src/constructors/lcf.c:52-99`.
//!
//! LCF notation is a compact encoding for 3-regular Hamiltonian graphs.
//! Given `n` vertices, a shift vector `shifts` and a `repeats` count, the
//! constructor builds a Hamilton cycle on `0..n` and then adds chord
//! edges per shift, looping `repeats * shifts.len()` times.
//!
//! Many small cubic graphs admit LCF descriptions — Franklin `[5, -5]^6`
//! on `n=12`, Truncated-tetrahedron `[2, 6, -2, -6]^3` on `n=12`,
//! Truncated-octahedron `[3, -7, 7, -3]^6` on `n=24`, Frucht
//! `[-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2]^1` on `n=12`, Heawood
//! `[5, -5]^7` on `n=14`, …
//!
//! Per the C source, after the candidate edge list is emitted the graph
//! is **simplified** (self-loops + parallel edges removed) before being
//! returned. The Rust port handles this inline via a canonical `(min,
//! max)` `BTreeSet` insert with self-loop skip so we never allocate a
//! duplicate-bearing intermediate graph.
//!
//! `igraph_lcf_small` (varargs convenience) is intentionally **not**
//! ported — Rust has no varargs; callers pass the shift slice directly.
//!
//! Time complexity: `O(|V| + |E|)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};
use std::collections::BTreeSet;

/// Construct an undirected graph from LCF notation.
///
/// `n` is the vertex count, `shifts` is the chord shift sequence
/// (entries may be negative — wraparound is modular), and `repeats`
/// controls how many full passes over `shifts` to walk.
///
/// The result is always undirected and always simple (self-loops and
/// parallel edges introduced by the chord pattern are removed in place).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `n + repeats * shifts.len()`
///   overflows the internal edge-buffer index.
///
/// # Examples
///
/// ```
/// use rust_igraph::lcf;
///
/// // Franklin graph: [5, -5]^6 on n=12 — 12 vertices, 18 edges, bipartite cubic.
/// let franklin = lcf(12, &[5, -5], 6).unwrap();
/// assert_eq!(franklin.vcount(), 12);
/// assert_eq!(franklin.ecount(), 18);
/// assert!(!franklin.is_directed());
/// ```
pub fn lcf(n: u32, shifts: &[i64], repeats: u32) -> IgraphResult<Graph> {
    // Overflow guard for the candidate-edge count: n + repeats * len.
    let len = u32::try_from(shifts.len()).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "lcf: shifts.len() = {} exceeds u32::MAX",
            shifts.len()
        ))
    })?;
    let chord_count = repeats.checked_mul(len).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "lcf: repeats * shifts.len() = {repeats} * {len} overflows u32",
        ))
    })?;
    let _total_candidates = n.checked_add(chord_count).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "lcf: n + repeats * shifts.len() = {n} + {chord_count} overflows u32",
        ))
    })?;

    // Degenerate: n == 0 short-circuits to the empty graph regardless of
    // shifts/repeats. This matches upstream regression test for bug #996
    // (igraph_lcf_small(&g, 0, 0) → vcount==0 && ecount==0).
    if n == 0 {
        return Graph::new(0, false);
    }

    // Canonical-edge accumulator — every chord/backbone insert is
    // canonicalised to (min, max) and deduplicated, with self-loops
    // skipped. This collapses what the C source delegates to
    // `igraph_simplify(loops=true, multi=true)` into a single pass.
    let mut edge_set: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();

    // 1) Hamilton cycle on 0..n. Edge (n-1, 0) closes the ring; for n==1
    //    the loop emits a single self-loop (i=0 → (0, 1 mod 1) = (0, 0))
    //    which the self-loop skip drops. For n==2 backbone (0,1) is the
    //    only edge that survives — chord wraparound from both endpoints
    //    is symmetric and collapses to (0,1) again.
    for i in 0..n {
        let from = i;
        let to = (i + 1) % n;
        if from != to {
            let (lo, hi) = if from < to { (from, to) } else { (to, from) };
            edge_set.insert((lo, hi));
        }
    }

    // 2) Chord pass — repeats * len candidate edges. The C source uses
    //    `(no_of_nodes + sptr + sh) % no_of_nodes` where `sh` is signed
    //    and `no_of_nodes + sptr` is wide enough to keep the intermediate
    //    non-negative for sane inputs. We use `i64::rem_euclid` so even
    //    pathologically large negative shifts wrap correctly.
    if chord_count > 0 && !shifts.is_empty() {
        let n_i64 = i64::from(n);
        for sptr in 0..chord_count {
            let shift = shifts[(sptr % len) as usize];
            let from = sptr % n;
            // (n + sptr + shift) mod n, computed with Euclidean modulus
            // so a negative offset wraps positively.
            let to_i64 = (i64::from(sptr) + shift).rem_euclid(n_i64);
            let to = u32::try_from(to_i64).map_err(|_| {
                IgraphError::InvalidArgument(format!("lcf: chord target {to_i64} out of u32 range"))
            })?;
            if from != to {
                let (lo, hi) = if from < to { (from, to) } else { (to, from) };
                edge_set.insert((lo, hi));
            }
        }
    }

    let mut graph = Graph::new(n, false)?;
    graph.add_edges(edge_set)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut out: Vec<(VertexId, VertexId)> = (0..m)
            .map(|e| g.edge(e).expect("edge in range"))
            .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
            .collect();
        out.sort_unstable();
        out
    }

    fn degree(g: &Graph, v: VertexId) -> usize {
        g.neighbors(v).expect("neighbors").len()
    }

    #[test]
    fn null_graph_regression_bug_996() {
        let g = lcf(0, &[], 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn franklin_5_minus5_repeats_6() {
        // Franklin graph: [5, -5]^6 on 12 vertices → 18 edges, cubic.
        let g = lcf(12, &[5, -5], 6).unwrap();
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 18);
        for v in 0..12 {
            assert_eq!(degree(&g, v), 3, "Franklin must be 3-regular at v={v}");
        }
    }

    #[test]
    fn three_minus2_repeats_4_n8_yields_16_edges() {
        // Upstream: [3, -2]^4, n=8 → ecount 16.
        let g = lcf(8, &[3, -2], 4).unwrap();
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 16);
    }

    #[test]
    fn n2_collapses_to_single_edge_two_chord_pattern() {
        // Upstream: [2, -2]^2, n=2 → ecount 1 (all chords are self-loops).
        let g = lcf(2, &[2, -2], 2).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
        assert_eq!(canonical_edges(&g), vec![(0, 1)]);
    }

    #[test]
    fn n2_collapses_to_single_edge_one_chord() {
        // Upstream: [2]^2, n=2 → ecount 1.
        let g = lcf(2, &[2], 2).unwrap();
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 1);
    }

    #[test]
    fn empty_shifts_yields_pure_cycle() {
        let g = lcf(6, &[], 0).unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 6);
        for v in 0..6 {
            assert_eq!(degree(&g, v), 2);
        }
    }

    #[test]
    fn repeats_zero_yields_pure_cycle() {
        let g = lcf(5, &[1, 2, 3], 0).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 5);
    }

    #[test]
    fn heawood_graph_5_minus5_repeats_7() {
        // Heawood: [5, -5]^7 on n=14 → 21 edges, cubic, bipartite, girth 6.
        let g = lcf(14, &[5, -5], 7).unwrap();
        assert_eq!(g.vcount(), 14);
        assert_eq!(g.ecount(), 21);
        for v in 0..14 {
            assert_eq!(degree(&g, v), 3, "Heawood must be 3-regular at v={v}");
        }
    }

    #[test]
    fn truncated_tetrahedron_2_6_minus2_minus6_repeats_3() {
        // Truncated tetrahedron: [2, 6, -2, -6]^3, n=12 → 18 edges, cubic.
        let g = lcf(12, &[2, 6, -2, -6], 3).unwrap();
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 18);
        for v in 0..12 {
            assert_eq!(degree(&g, v), 3);
        }
    }

    #[test]
    fn truncated_octahedron_3_minus7_7_minus3_repeats_6() {
        // Truncated octahedron: [3, -7, 7, -3]^6 on n=24 → 36 edges, cubic.
        let g = lcf(24, &[3, -7, 7, -3], 6).unwrap();
        assert_eq!(g.vcount(), 24);
        assert_eq!(g.ecount(), 36);
        for v in 0..24 {
            assert_eq!(degree(&g, v), 3);
        }
    }

    #[test]
    fn n1_yields_singleton_no_edges() {
        let g = lcf(1, &[3, -5], 4).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn always_undirected_no_self_loops() {
        let g = lcf(10, &[3, -3], 5).unwrap();
        assert!(!g.is_directed());
        let m = u32::try_from(g.ecount()).expect("ecount fits u32");
        for e in 0..m {
            let (a, b) = g.edge(e).expect("edge in range");
            assert_ne!(a, b, "no self-loops");
        }
    }

    #[test]
    fn always_simple_no_parallel_edges() {
        let g = lcf(8, &[3, -2], 4).unwrap();
        let edges = canonical_edges(&g);
        let mut unique = edges.clone();
        unique.dedup();
        assert_eq!(edges.len(), unique.len(), "no parallel edges");
    }

    #[test]
    fn frucht_graph_full_12_shift_pattern() {
        // Frucht: [-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2]^1, n=12 →
        // 18 edges, cubic. Asymmetric — the only 3-regular planar graph
        // with no non-trivial automorphisms.
        let g = lcf(12, &[-5, -2, -4, 2, 5, -2, 2, 5, -2, -5, 4, 2], 1).unwrap();
        assert_eq!(g.vcount(), 12);
        assert_eq!(g.ecount(), 18);
        for v in 0..12 {
            assert_eq!(degree(&g, v), 3, "Frucht must be 3-regular at v={v}");
        }
    }

    #[test]
    fn negative_shift_larger_than_n_wraps_correctly() {
        // shift = -100 on n=10 vertices: from sptr=0 → to = (-100) mod 10 = 0.
        // Self-loop at vertex 0 dropped. Subsequent sptrs each produce one
        // chord per pass, and the construction should remain simple.
        let g = lcf(10, &[-100, 100], 5).unwrap();
        assert_eq!(g.vcount(), 10);
        let edges = canonical_edges(&g);
        let mut unique = edges.clone();
        unique.dedup();
        assert_eq!(edges.len(), unique.len(), "no parallels");
        for (a, b) in &edges {
            assert_ne!(a, b);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn arb_lcf_params() -> impl Strategy<Value = (u32, Vec<i64>, u32)> {
        // n ∈ [0, 30], shifts of length ≤ 6 with values in [-30, 30],
        // repeats ∈ [0, 4]. Wide enough to exercise wraparound, n=0/1/2
        // edge cases, empty-shift and zero-repeat short-circuits, while
        // keeping fixture sizes small.
        (
            0u32..=30,
            prop::collection::vec(-30i64..=30, 0..=6),
            0u32..=4,
        )
    }

    proptest! {
        #[test]
        fn always_undirected_no_self_loops_no_parallels(
            (n, shifts, repeats) in arb_lcf_params()
        ) {
            let g = lcf(n, &shifts, repeats).unwrap();
            prop_assert!(!g.is_directed());
            prop_assert_eq!(g.vcount(), n);

            let m = u32::try_from(g.ecount()).unwrap();
            let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
            for e in 0..m {
                let (a, b) = g.edge(e).unwrap();
                prop_assert_ne!(a, b, "self-loop survived simplify");
                let canon = if a <= b { (a, b) } else { (b, a) };
                prop_assert!(seen.insert(canon), "duplicate edge survived simplify");
            }
        }

        #[test]
        fn ecount_bounded_by_candidate_count(
            (n, shifts, repeats) in arb_lcf_params()
        ) {
            // After simplify, ecount ≤ Hamilton-cycle edges + chord pass.
            let g = lcf(n, &shifts, repeats).unwrap();
            let backbone_edges = if n >= 2 { n as usize } else { 0 };
            let chord_edges = (repeats as usize)
                .checked_mul(shifts.len())
                .unwrap_or(0);
            prop_assert!(g.ecount() <= backbone_edges + chord_edges);
        }

        #[test]
        fn pure_cycle_when_no_chords(n in 3u32..=30, shifts_empty in any::<bool>()) {
            // shifts.is_empty() || repeats == 0 ⇒ result is C_n: n edges
            // each forming the Hamilton ring, every vertex degree 2.
            let shifts: Vec<i64> = if shifts_empty { vec![] } else { vec![5, -5] };
            let repeats: u32 = if shifts_empty { 7 } else { 0 };
            let g = lcf(n, &shifts, repeats).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), n as usize);
            for v in 0..n {
                prop_assert_eq!(g.neighbors(v).unwrap().len(), 2);
            }
        }

        #[test]
        fn n_zero_yields_empty_regardless_of_pattern(
            shifts in prop::collection::vec(-30i64..=30, 0..=6),
            repeats in 0u32..=4,
        ) {
            let g = lcf(0, &shifts, repeats).unwrap();
            prop_assert_eq!(g.vcount(), 0);
            prop_assert_eq!(g.ecount(), 0);
            prop_assert!(!g.is_directed());
        }

        #[test]
        fn n_one_yields_singleton_regardless_of_pattern(
            shifts in prop::collection::vec(-30i64..=30, 0..=6),
            repeats in 0u32..=4,
        ) {
            // n=1: Hamilton step (0, 0) and every chord is a self-loop, all
            // dropped by the simplify pass.
            let g = lcf(1, &shifts, repeats).unwrap();
            prop_assert_eq!(g.vcount(), 1);
            prop_assert_eq!(g.ecount(), 0);
        }

        #[test]
        fn matches_unrolled_shifts(
            n in 3u32..=20,
            shifts in prop::collection::vec(-15i64..=15, 1..=4),
            repeats in 1u32..=3,
        ) {
            // lcf(n, [s0, s1, ..., sk-1], repeats) must be edge-equivalent to
            // lcf(n, [s0, s1, ..., sk-1] repeated `repeats` times, 1).
            let g_repeated = lcf(n, &shifts, repeats).unwrap();
            let unrolled: Vec<i64> = (0..repeats)
                .flat_map(|_| shifts.iter().copied())
                .collect();
            let g_unrolled = lcf(n, &unrolled, 1).unwrap();
            prop_assert_eq!(g_repeated.ecount(), g_unrolled.ecount());

            let collect_canon = |g: &Graph| -> BTreeSet<(u32, u32)> {
                let m = u32::try_from(g.ecount()).unwrap();
                (0..m)
                    .map(|e| g.edge(e).unwrap())
                    .map(|(a, b)| if a <= b { (a, b) } else { (b, a) })
                    .collect()
            };
            prop_assert_eq!(collect_canon(&g_repeated), collect_canon(&g_unrolled));
        }

        #[test]
        fn maximum_degree_bounded(
            (n, shifts, repeats) in arb_lcf_params()
        ) {
            // Every vertex appears on the Hamilton cycle (+2 degree once
            // n ≥ 3) and as `from` exactly `chord_count.div_ceil(n)` times
            // plus the `to`-side mirror. Hard upper bound: 2 (backbone)
            // + 2 * chord_count / n (rounded up). Looser but cheap check.
            let g = lcf(n, &shifts, repeats).unwrap();
            if n == 0 {
                return Ok(());
            }
            let chord_count = (repeats as usize) * shifts.len();
            let max_deg = 2usize.saturating_add(2 * chord_count);
            for v in 0..n {
                prop_assert!(g.neighbors(v).unwrap().len() <= max_deg);
            }
        }
    }
}
