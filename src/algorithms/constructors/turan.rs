//! Turán graph constructor (ALGO-CN-027).
//!
//! Counterpart of `igraph_turan()` in
//! `references/igraph/src/constructors/full.c:281-325`.
//!
//! The *Turán graph* `T(n, r)` is the complete `r`-partite graph on `n`
//! vertices whose partition sizes differ by at most one — i.e. the
//! densest graph on `n` vertices that does not contain a clique of size
//! `r + 1` (Turán's theorem, 1941). Concretely:
//!
//! * Let `q = n / r` and `s = n % r`.
//! * The first `s` partitions get size `q + 1`; the remaining `r − s`
//!   partitions get size `q`.
//! * The result is `full_multipartite(&[q+1; s] ++ &[q; r-s], false,
//!   MultipartiteMode::All)`.
//!
//! Turán graphs are always undirected. The result bundles the graph
//! with a `types` vector recording the partition index of every vertex
//! (partition-major order).
//!
//! Degenerate cases (matching upstream):
//!
//! * `n == 0` → empty graph with empty `types`. The `r` argument is
//!   ignored.
//! * `r > n` → silently capped to `r = n`, yielding `n` singleton
//!   partitions (i.e. the complete graph `K_n` with `types = [0, 1, …,
//!   n − 1]`).
//! * `r == 0` → [`IgraphError::InvalidArgument`] (the partition count
//!   must be positive when `n > 0`).
//!
//! Time complexity: `O(|V| + |E|)`, dominated by the wrapped
//! [`full_multipartite`] call.

use crate::algorithms::constructors::full_multipartite::{
    FullMultipartite, MultipartiteMode, full_multipartite,
};
use crate::core::{IgraphError, IgraphResult};

/// Build the Turán graph `T(n, r)`.
///
/// `n` is the vertex count; `r` is the partition count. Returns a
/// [`FullMultipartite`] whose `graph` is the complete `r'`-partite
/// graph with maximally balanced partition sizes (where
/// `r' = min(r, n)`) and whose `types` records the partition index of
/// every vertex.
///
/// See the module documentation for the precise semantics and
/// degenerate-case handling.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `r == 0` while `n > 0`. The
///   upstream C entry returns `IGRAPH_EINVAL` in this case and we
///   preserve that contract.
/// * Any [`IgraphError`] surfaced by the underlying
///   [`full_multipartite`] call (overflow when the resulting graph
///   would have more than `u32::MAX` vertices or more than
///   `usize::MAX` edges).
///
/// # Examples
///
/// ```
/// use rust_igraph::turan;
///
/// // T(6, 3) — three balanced partitions of size 2, the cocktail-party
/// // graph K_{2,2,2} a.k.a. the octahedron.
/// let r = turan(6, 3).unwrap();
/// assert_eq!(r.graph.vcount(), 6);
/// assert_eq!(r.graph.ecount(), 12);
/// assert_eq!(r.types, vec![0, 0, 1, 1, 2, 2]);
///
/// // r > n → capped at r = n, yielding K_4.
/// let r = turan(4, 6).unwrap();
/// assert_eq!(r.graph.vcount(), 4);
/// assert_eq!(r.graph.ecount(), 6);
/// assert_eq!(r.types, vec![0, 1, 2, 3]);
///
/// // n == 0 → empty graph regardless of r.
/// let r = turan(0, 10).unwrap();
/// assert_eq!(r.graph.vcount(), 0);
/// assert!(r.types.is_empty());
/// ```
pub fn turan(n: u32, r: u32) -> IgraphResult<FullMultipartite> {
    if n == 0 {
        return full_multipartite(&[], false, MultipartiteMode::All);
    }

    if r == 0 {
        return Err(IgraphError::InvalidArgument(
            "turan: number of partitions must be positive when n > 0".into(),
        ));
    }

    let r_effective = r.min(n);
    let quotient = n / r_effective;
    let remainder = n % r_effective;

    // The first `remainder` partitions get one extra vertex (size
    // `quotient + 1`); the remaining `r_effective − remainder`
    // partitions get exactly `quotient`. Matches upstream's
    // `igraph_vector_int_fill(quotient)` + per-index `++` loop.
    let mut partitions: Vec<u32> = Vec::with_capacity(r_effective as usize);
    let big = quotient.checked_add(1).ok_or_else(|| {
        IgraphError::InvalidArgument("turan: partition size n/r + 1 overflows u32".into())
    })?;
    for i in 0..r_effective {
        if i < remainder {
            partitions.push(big);
        } else {
            partitions.push(quotient);
        }
    }

    full_multipartite(&partitions, false, MultipartiteMode::All)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_edges(g: &crate::core::Graph) -> BTreeSet<(u32, u32)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits u32");
        (0..m)
            .map(|e| g.edge(e).expect("edge id in bounds"))
            .map(|(u, v)| if u <= v { (u, v) } else { (v, u) })
            .collect()
    }

    #[test]
    fn empty_n_zero_returns_empty_graph() {
        // Upstream `.out` test 1: igraph_turan(0, 10).
        let r = turan(0, 10).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(!r.graph.is_directed());
        assert!(r.types.is_empty());
    }

    #[test]
    fn empty_n_zero_with_r_one_also_empty() {
        // Edge: n == 0 should ignore r entirely, even r == 1.
        let r = turan(0, 1).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert!(r.types.is_empty());
    }

    #[test]
    fn empty_n_zero_with_r_zero_still_empty_no_error() {
        // n == 0 short-circuits before the r == 0 check, matching
        // upstream's `if (n == 0)` early-return.
        let r = turan(0, 0).expect("ok");
        assert_eq!(r.graph.vcount(), 0);
        assert!(r.types.is_empty());
    }

    #[test]
    fn r_zero_with_positive_n_is_error() {
        let err = turan(5, 0).expect_err("must reject r == 0 when n > 0");
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn r_eq_one_is_n_isolated_vertices() {
        // Upstream `.out` test 2: igraph_turan(10, 1).
        let r = turan(10, 1).expect("ok");
        assert_eq!(r.graph.vcount(), 10);
        assert_eq!(r.graph.ecount(), 0);
        assert_eq!(r.types, vec![0; 10]);
    }

    #[test]
    fn r_exceeds_n_caps_at_n_yielding_kn() {
        // Upstream `.out` test 3: igraph_turan(4, 6) → 4 vertices, 1 partition each.
        let r = turan(4, 6).expect("ok");
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 6);
        assert_eq!(r.types, vec![0, 1, 2, 3]);
        // The full K_4 edge set.
        let expected: BTreeSet<(u32, u32)> = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
            .into_iter()
            .collect();
        assert_eq!(canonical_edges(&r.graph), expected);
    }

    #[test]
    fn turan_13_4_matches_upstream() {
        // Upstream `.out` test 4: igraph_turan(13, 4) — quotient 3,
        // remainder 1 → partitions [4, 3, 3, 3]. types is exactly the
        // .out line `0 0 0 0 1 1 1 2 2 2 3 3 3`.
        let r = turan(13, 4).expect("ok");
        assert_eq!(r.graph.vcount(), 13);
        // E = ½ · (4·9 + 3·10 + 3·10 + 3·10) = ½ · 126 = 63.
        assert_eq!(r.graph.ecount(), 63);
        assert_eq!(r.types, vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn turan_13_4_ecount_matches_closed_form() {
        // E = ½ · Σ_i n_i · (N − n_i). N=13, sizes [4,3,3,3]:
        //   ½ · (4·9 + 3·10 + 3·10 + 3·10) = ½ · (36 + 30 + 30 + 30) = ½ · 126 = 63.
        let r = turan(13, 4).expect("ok");
        let n: u32 = 13;
        let sizes: [u32; 4] = [4, 3, 3, 3];
        let twice_e: u32 = sizes.iter().map(|&s| s * (n - s)).sum();
        let expected_e = (twice_e / 2) as usize;
        assert_eq!(r.graph.ecount(), expected_e);
    }

    #[test]
    fn turan_8_3_matches_upstream() {
        // Upstream `.out` test 5: igraph_turan(8, 3) — quotient 2,
        // remainder 2 → partitions [3, 3, 2]. types is exactly the
        // .out line `0 0 0 1 1 1 2 2`.
        let r = turan(8, 3).expect("ok");
        assert_eq!(r.graph.vcount(), 8);
        assert_eq!(r.types, vec![0, 0, 0, 1, 1, 1, 2, 2]);
        // E = ½ · (3·5 + 3·5 + 2·6) = ½ · 42 = 21.
        assert_eq!(r.graph.ecount(), 21);
    }

    #[test]
    fn turan_6_3_matches_upstream() {
        // Upstream `.out` test 6: igraph_turan(6, 3) — quotient 2,
        // remainder 0 → partitions [2, 2, 2]. The octahedron / cocktail
        // party graph K_{2,2,2}.
        let r = turan(6, 3).expect("ok");
        assert_eq!(r.graph.vcount(), 6);
        assert_eq!(r.graph.ecount(), 12);
        assert_eq!(r.types, vec![0, 0, 1, 1, 2, 2]);
    }

    #[test]
    fn turan_balanced_when_remainder_zero() {
        // n divisible by r → all partitions equal size.
        let r = turan(12, 4).expect("ok");
        assert_eq!(r.graph.vcount(), 12);
        // partitions = [3, 3, 3, 3]; E = ½ · Σ 3·9 = ½ · 108 = 54.
        assert_eq!(r.graph.ecount(), 54);
        assert_eq!(r.types, vec![0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn turan_first_partitions_get_the_extras_when_remainder_nonzero() {
        // n=14, r=4 → q=3, s=2 → partitions [4, 4, 3, 3].
        let r = turan(14, 4).expect("ok");
        assert_eq!(r.graph.vcount(), 14);
        // First two partitions size 4, next two size 3.
        assert_eq!(r.types, vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn turan_no_self_loops_no_intra_partition_edges() {
        let r = turan(15, 5).expect("ok");
        for (u, v) in canonical_edges(&r.graph) {
            assert_ne!(u, v, "no self-loops");
            assert_ne!(
                r.types[u as usize], r.types[v as usize],
                "edge ({u}, {v}) lives inside a single partition"
            );
        }
    }

    #[test]
    fn turan_is_always_undirected() {
        for (n, r) in [(0, 1), (5, 3), (10, 4), (20, 7)] {
            let res = turan(n, r).expect("ok");
            assert!(!res.graph.is_directed(), "T({n}, {r}) must be undirected");
        }
    }

    #[test]
    fn turan_isomorphic_to_full_multipartite_balanced() {
        // From upstream test: turan(13, 4) is isomorphic to
        // full_multipartite([4, 3, 3, 3]). Compare canonical edge
        // multisets — emission order should already match since we
        // share the underlying implementation.
        let t = turan(13, 4).expect("ok");
        let mp = full_multipartite(&[4, 3, 3, 3], false, MultipartiteMode::All).expect("ok");
        assert_eq!(t.graph.vcount(), mp.graph.vcount());
        assert_eq!(t.graph.ecount(), mp.graph.ecount());
        assert_eq!(t.types, mp.types);
        assert_eq!(canonical_edges(&t.graph), canonical_edges(&mp.graph));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn vcount_always_equals_n(n in 0u32..30, r in 1u32..10) {
            let res = turan(n, r).expect("ok for valid inputs");
            prop_assert_eq!(res.graph.vcount(), n);
            prop_assert_eq!(u32::try_from(res.types.len()).unwrap(), n);
        }

        #[test]
        fn always_undirected(n in 0u32..30, r in 1u32..10) {
            let res = turan(n, r).expect("ok");
            prop_assert!(!res.graph.is_directed());
        }

        #[test]
        fn types_partition_major_monotone(n in 1u32..30, r in 1u32..10) {
            let res = turan(n, r).expect("ok");
            for w in res.types.windows(2) {
                prop_assert!(w[0] <= w[1]);
            }
        }

        #[test]
        fn partition_size_differs_by_at_most_one(n in 1u32..30, r in 1u32..10) {
            let res = turan(n, r).expect("ok");
            // Count vertices per partition by scanning types.
            let max_t = *res.types.iter().max().unwrap();
            let mut sizes = vec![0u32; (max_t + 1) as usize];
            for &t in &res.types {
                sizes[t as usize] += 1;
            }
            let lo = *sizes.iter().min().unwrap();
            let hi = *sizes.iter().max().unwrap();
            prop_assert!(hi - lo <= 1, "partition sizes must differ by ≤ 1: {sizes:?}");
        }

        #[test]
        fn no_intra_partition_edges(n in 0u32..20, r in 1u32..8) {
            let res = turan(n, r).expect("ok");
            let m = u32::try_from(res.graph.ecount()).unwrap();
            for e in 0..m {
                let (u, v) = res.graph.edge(e).unwrap();
                prop_assert_ne!(res.types[u as usize], res.types[v as usize]);
            }
        }

        #[test]
        fn r_capped_at_n_when_oversized(n in 1u32..15, extra in 0u32..20) {
            // T(n, n + extra) should equal T(n, n).
            let big = turan(n, n + extra).expect("ok");
            let cap = turan(n, n).expect("ok");
            prop_assert_eq!(big.graph.vcount(), cap.graph.vcount());
            prop_assert_eq!(big.graph.ecount(), cap.graph.ecount());
            prop_assert_eq!(big.types, cap.types);
        }
    }
}
