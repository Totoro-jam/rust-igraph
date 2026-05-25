//! Circulant graph constructor (ALGO-CN-011).
//!
//! Counterpart of `igraph_circulant()` in
//! `references/igraph/src/constructors/circulant.c:51-112`.
//!
//! A circulant graph `G(n, shifts)` has `n` vertices `v_0, …, v_{n-1}`
//! and, for every shift `s` in `shifts`, the edges
//! `(v_j, v_{(j + s) mod n})` for every `j ∈ [0, n)`.
//!
//! The constructor handles a few canonical-form rules so the same graph
//! is built regardless of how shifts are spelled:
//!
//! * Each shift is reduced modulo `n` into `[0, n)`.
//! * In the **undirected** case, a shift `s` and its complement `n − s`
//!   generate the same set of edges, so a shift `≥ (n + 1) / 2` is
//!   replaced with `n − s` before dedup.
//! * `shift == 0` is always dropped (it would emit `n` self-loops).
//! * Repeated shifts (after the above normalisation) are deduplicated
//!   so the result has no parallel edges.
//! * Special case: when `n` is even, the graph is undirected, and a
//!   shift equals `n / 2`, only the first `n / 2` edges are emitted —
//!   the wrap-around would otherwise produce a duplicate of every
//!   antipodal edge.
//!
//! Specializations:
//!
//! * `circulant(n, &[1], false)` is `ring_graph(n)` — the cycle `C_n`.
//! * `circulant(n, &[k], false)` (with `0 < k < n/2`) is precisely the
//!   inner layer of [`generalized_petersen(n, k)`].
//! * `circulant(n, &[1, 2], false)` is the squared cycle / Möbius
//!   ladder family.
//! * `circulant(n, &(1..n/2).collect::<Vec<_>>(), false)` is the
//!   complete graph `K_n` (every distinct undirected shift contributes).
//!
//! Time complexity: `O(|V| · |shifts|)`.
//!
//! [`generalized_petersen(n, k)`]: super::generalized_petersen::generalized_petersen

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the circulant graph `G(n, shifts)`.
///
/// The result is directed iff `directed` is `true`. The graph never
/// contains self-loops or parallel edges; redundant or zero shifts are
/// silently dropped per the normalisation rules described in the
/// module-level docs.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — any individual shift cannot be
///   represented as `i64` (this only fires on negative inputs that
///   would not round-trip, since the public type is unsigned).
/// * [`IgraphError::InvalidArgument`] — `n * |shifts|` overflows `usize`
///   so the edge buffer cannot be sized safely.
///
/// # Examples
///
/// ```
/// use rust_igraph::circulant;
///
/// // C_6 — cycle on 6 vertices.
/// let g = circulant(6, &[1], false).unwrap();
/// assert_eq!(g.vcount(), 6);
/// assert_eq!(g.ecount(), 6);
///
/// // K_4 — complete graph via every distinct undirected shift.
/// let k4 = circulant(4, &[1, 2], false).unwrap();
/// assert_eq!(k4.vcount(), 4);
/// assert_eq!(k4.ecount(), 6); // 4*3/2
/// ```
pub fn circulant(n: u32, shifts: &[i64], directed: bool) -> IgraphResult<Graph> {
    if n == 0 {
        return Graph::new(0, directed);
    }

    // Normalise shifts: bring into [0, n); fold s and n-s in undirected.
    // We track normalised shifts in a small Vec (deduplicated) rather
    // than a bool[n] table — the canonical igraph pass uses bool[n],
    // but |shifts| is generally tiny.
    let n_i64 = i64::from(n);
    let mut seen: Vec<u32> = Vec::with_capacity(shifts.len());
    seen.push(0); // zero shift always suppressed (self-loops)

    let mut canonical_shifts: Vec<u32> = Vec::with_capacity(shifts.len());
    for &raw in shifts {
        // Modulo into [0, n), Rust style: rem then add n if negative.
        let mut s = raw % n_i64;
        if s < 0 {
            s += n_i64;
        }
        let mut s = u32::try_from(s).map_err(|_| {
            IgraphError::InvalidArgument(format!(
                "circulant: shift {raw} (normalised {s}) cannot fit u32"
            ))
        })?;

        if !directed && s >= n.div_ceil(2) {
            s = n - s;
        }

        if !seen.contains(&s) {
            seen.push(s);
            canonical_shifts.push(s);
        }
    }

    // Pre-size the edge buffer using the worst-case `n * |canonical|`.
    // Even-n undirected at shift n/2 shaves half off, but the saving is
    // small and not worth a second pass.
    let cap = usize::try_from(n)
        .ok()
        .and_then(|nu| nu.checked_mul(canonical_shifts.len()))
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "circulant: n * |shifts| overflows usize (n = {n}, |shifts| = {})",
                canonical_shifts.len()
            ))
        })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(cap);
    for &shift in &canonical_shifts {
        // Even-n undirected antipodal shift halves the loop range.
        let limit = if !directed && n % 2 == 0 && shift == n / 2 {
            n / 2
        } else {
            n
        };
        for j in 0..limit {
            edges.push((j, (j + shift) % n));
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    fn canon_set(edges: &[(u32, u32)]) -> std::collections::BTreeSet<(u32, u32)> {
        edges.iter().map(|&(u, v)| canon(u, v)).collect()
    }

    #[test]
    fn empty_graph_for_n_zero() {
        let g = circulant(0, &[1], false).expect("n=0 ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_with_any_shift_is_singleton_no_edges() {
        // Every shift normalises to 0 mod 1 and is dropped.
        let g = circulant(1, &[0, 1, 2, 5], false).expect("n=1 ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn shift_one_equals_ring_graph() {
        // circulant(n, &[1], false) is exactly C_n.
        for n in 3..=12u32 {
            let g = circulant(n, &[1], false).expect("valid");
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount(), n as usize);
            for v in 0..n {
                assert_eq!(
                    g.degree(v).expect("in range"),
                    2,
                    "C_{n} should be 2-regular"
                );
            }
        }
    }

    #[test]
    fn directed_shift_one_is_directed_cycle() {
        let g = circulant(5, &[1], true).expect("valid");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 5);
        // Each vertex has out-degree 1 (to (j+1) mod 5).
        let edges = dump_edges(&g);
        for j in 0..5 {
            assert!(edges.iter().any(|&e| e == (j, (j + 1) % 5)));
        }
    }

    #[test]
    fn shift_zero_yields_no_edges() {
        let g = circulant(5, &[0], false).expect("ok");
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn duplicate_shifts_are_deduplicated() {
        let g = circulant(7, &[2, 2, 9, -5], false).expect("ok");
        // After mod 7: 2, 2, 2, 2 → 2 only. n - 2 = 5 also collapses to 2.
        // Edge count = n = 7.
        assert_eq!(g.ecount(), 7);
    }

    #[test]
    fn undirected_complementary_shift_collapses() {
        // For undirected n=7: shift 5 ≡ shift 2.
        let a = circulant(7, &[2], false).expect("ok");
        let b = circulant(7, &[5], false).expect("ok");
        assert_eq!(canon_set(&dump_edges(&a)), canon_set(&dump_edges(&b)));
    }

    #[test]
    fn directed_keeps_complementary_distinct() {
        let a = circulant(7, &[2], true).expect("ok");
        let b = circulant(7, &[5], true).expect("ok");
        // In directed mode, shifts 2 and 5 give different edge sets.
        let ea: std::collections::BTreeSet<(u32, u32)> = dump_edges(&a).into_iter().collect();
        let eb: std::collections::BTreeSet<(u32, u32)> = dump_edges(&b).into_iter().collect();
        assert_ne!(ea, eb);
    }

    #[test]
    fn even_n_antipodal_shift_halves_emission() {
        // n=6 undirected, shift=3: only 3 edges (perfect matching).
        let g = circulant(6, &[3], false).expect("ok");
        assert_eq!(g.ecount(), 3);
        let edges = canon_set(&dump_edges(&g));
        for j in 0..3u32 {
            assert!(edges.contains(&(j, j + 3)));
        }
    }

    #[test]
    fn odd_n_no_antipodal_shortcut() {
        // n=7 undirected, shift=3: 7 edges (no halving — there is no n/2).
        let g = circulant(7, &[3], false).expect("ok");
        assert_eq!(g.ecount(), 7);
    }

    #[test]
    fn complete_graph_via_all_distinct_shifts() {
        // K_n = circulant(n, &[1, 2, …, n/2], false) — every undirected
        // shift class is present.
        for n in 3..=10u32 {
            let shifts: Vec<i64> = (1..=(n / 2)).map(i64::from).collect();
            let g = circulant(n, &shifts, false).expect("valid");
            let expected_edges = (n as usize) * ((n as usize) - 1) / 2;
            assert_eq!(g.ecount(), expected_edges, "K_{n}");
            for v in 0..n {
                assert_eq!(
                    g.degree(v).expect("in range"),
                    (n - 1) as usize,
                    "K_{n} regularity"
                );
            }
        }
    }

    #[test]
    fn negative_shifts_normalised_into_range() {
        // shift -1 ≡ n-1 in mod, then folded to 1 in undirected mode.
        let g = circulant(5, &[-1], false).expect("ok");
        let r = circulant(5, &[1], false).expect("ok");
        assert_eq!(canon_set(&dump_edges(&g)), canon_set(&dump_edges(&r)));
    }

    #[test]
    fn no_self_loops_in_any_simple_case() {
        for n in 1..=12u32 {
            let max_shift = i64::from(n / 2);
            if max_shift == 0 {
                continue;
            }
            let shifts: Vec<i64> = (1..=max_shift).collect();
            let g = circulant(n, &shifts, false).expect("valid");
            for (u, v) in dump_edges(&g) {
                assert_ne!(u, v, "self-loop in circulant({n}, [{shifts:?}])");
            }
        }
    }

    #[test]
    fn no_parallel_edges() {
        // The shift dedup must prevent any duplicate canonical edge.
        for n in 3..=12u32 {
            let shifts: Vec<i64> = (1..=i64::from(n / 2)).collect();
            let g = circulant(n, &shifts, false).expect("valid");
            let edges = dump_edges(&g);
            let canon: std::collections::HashSet<(u32, u32)> = edges
                .iter()
                .map(|&(u, v)| super::tests::canon(u, v))
                .collect();
            assert_eq!(
                canon.len(),
                edges.len(),
                "parallel edges in circulant({n}, [{shifts:?}])"
            );
        }
    }

    #[test]
    fn matches_inner_layer_of_generalized_petersen() {
        // The inner layer of generalized_petersen(n, k) is the circulant
        // with shift k, just on vertex ids [n, 2n).
        use crate::generalized_petersen;
        for n in 3..=12u32 {
            let max_k = (n - 1) / 2;
            for k in 1..=max_k {
                let gpg = generalized_petersen(n, k).expect("valid");
                let inner_edges: std::collections::BTreeSet<(u32, u32)> = (0..gpg.ecount())
                    .map(|e| {
                        gpg.edge(u32::try_from(e).expect("ecount fits u32"))
                            .expect("in range")
                    })
                    .filter(|&(u, v)| u >= n && v >= n)
                    .map(|(u, v)| canon(u - n, v - n))
                    .collect();
                let c = circulant(n, &[i64::from(k)], false).expect("valid");
                let c_edges = canon_set(&dump_edges(&c));
                assert_eq!(inner_edges, c_edges, "inner layer of G({n},{k})");
            }
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    fn dump_edges(g: &Graph) -> Vec<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .collect()
    }

    fn canon(u: u32, v: u32) -> (u32, u32) {
        if u <= v { (u, v) } else { (v, u) }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            max_shrink_iters: 1000,
            .. ProptestConfig::default()
        })]

        /// No self-loops or parallel edges in any undirected circulant
        /// whose shifts come from `1..=n/2`.
        #[test]
        fn simple_undirected(n in 1u32..=40, mask in 0u64..(1u64 << 20)) {
            let max = (n / 2).min(20);
            if max == 0 { return Ok(()); }
            let shifts: Vec<i64> = (1..=max)
                .filter(|s| mask & (1u64 << (s - 1)) != 0)
                .map(i64::from)
                .collect();
            if shifts.is_empty() { return Ok(()); }
            let g = circulant(n, &shifts, false).expect("valid");
            let mut set: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
            for (u, v) in dump_edges(&g) {
                prop_assert_ne!(u, v);
                prop_assert!(set.insert(canon(u, v)));
            }
        }

        /// Vertex regularity: every vertex has the same degree, which is
        /// `2 * |canonical shifts|` minus 1 per antipodal shift on even n
        /// (because the antipodal edge counts once per endpoint, not
        /// twice).
        #[test]
        fn vertex_regularity(n in 2u32..=30, mask in 0u64..(1u64 << 15)) {
            let max = (n / 2).min(15);
            if max == 0 { return Ok(()); }
            let shifts: Vec<i64> = (1..=max)
                .filter(|s| mask & (1u64 << (s - 1)) != 0)
                .map(i64::from)
                .collect();
            if shifts.is_empty() { return Ok(()); }
            let g = circulant(n, &shifts, false).expect("valid");
            let d0 = g.degree(0).expect("in range");
            for v in 1..n {
                prop_assert_eq!(g.degree(v).expect("in range"), d0, "vertex {} differs", v);
            }
        }

        /// Negative shifts always produce the same graph as their
        /// positive normalisations in undirected mode.
        #[test]
        fn negative_shifts_equiv(n in 3u32..=30, raw_shift in 1i64..=200) {
            let g_pos = circulant(n, &[raw_shift], false).expect("valid");
            let g_neg = circulant(n, &[-raw_shift], false).expect("valid");
            let ep: std::collections::BTreeSet<(u32, u32)> =
                dump_edges(&g_pos).into_iter().map(|(u, v)| canon(u, v)).collect();
            let en: std::collections::BTreeSet<(u32, u32)> =
                dump_edges(&g_neg).into_iter().map(|(u, v)| canon(u, v)).collect();
            prop_assert_eq!(ep, en);
        }

        /// Complete-graph specialization: `circulant(n, &[1..=n/2], false)`
        /// has exactly `n*(n-1)/2` edges and every vertex has degree
        /// `n-1`.
        #[test]
        fn complete_specialization(n in 2u32..=30) {
            let shifts: Vec<i64> = (1..=(n / 2)).map(i64::from).collect();
            let g = circulant(n, &shifts, false).expect("valid");
            let exp = (n as usize) * ((n as usize) - 1) / 2;
            prop_assert_eq!(g.ecount(), exp);
            for v in 0..n {
                prop_assert_eq!(g.degree(v).expect("in range"), (n - 1) as usize);
            }
        }
    }
}
