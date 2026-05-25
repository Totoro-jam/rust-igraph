//! Hamming graph constructor (ALGO-CN-008).
//!
//! Counterpart of `igraph_hamming()` in
//! `references/igraph/src/constructors/regular.c:1070-1153`.
//!
//! The d-dimensional **Hamming graph** `H(n, q)` over an alphabet of
//! size `q` has `q^n` vertices indexed by all length-`n` strings in
//! `{0, 1, …, q-1}^n`. Two vertices are connected iff their strings
//! differ in **exactly one position**.
//!
//! The string `(x_0, x_1, …, x_{n-1})` maps to vertex id
//! `Σ_i x_i · q^i` (little-endian base-`q` representation), matching
//! the upstream convention.
//!
//! Edge count: `q^n · (q − 1) · n / 2`.
//!
//! Special cases:
//!
//! * `n = 0` → singleton (one vertex), even when `q = 0`.
//! * `n > 0` and `q = 0` → null graph (zero vertices).
//! * `q = 1` → singleton (the only string is the all-zero one, no edges).
//! * `q = 2` → identical to the n-dimensional hypercube `Q_n` (see
//!   [`crate::hypercube`]).
//!
//! Algorithm: enumerate every vertex `v ∈ [0, q^n)`; for each digit
//! position `i ∈ [0, n)` extract `dig = (v / q^i) mod q`, then for
//! every higher digit value `dig + j` with `j ∈ [1, q − dig)` emit the
//! canonical edge `(v, v + j · q^i)`. Because we only walk upward in
//! each digit, every edge is produced exactly once (lower endpoint
//! first). Mirrors the upstream C loop.
//!
//! Time complexity: `O(n · q^(n+1))` — for each of the `q^n` vertices
//! we iterate `n` digit positions and emit up to `q − 1` outgoing
//! upward-pointing edges per digit.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the d-dimensional Hamming graph `H(n, q)`.
///
/// Returns a graph with `q^n` vertices where two vertices share an
/// edge iff their base-`q` digit representations differ in exactly one
/// position. For the directed variant every edge points from the
/// lower-indexed endpoint to the higher-indexed one (the canonical
/// orientation used by upstream igraph).
///
/// # Edge cases
///
/// * `n = 0` returns the singleton (one vertex, zero edges), regardless
///   of `q`.
/// * `n > 0` and `q = 0` returns the null graph (zero vertices, zero
///   edges).
/// * `q = 1` returns the singleton.
/// * `q = 2` produces `Q_n`, equivalent to [`crate::hypercube`].
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `q^n` overflows `u32`, or the
///   edge count overflows `usize`.
///
/// # Examples
///
/// ```
/// use rust_igraph::hamming;
///
/// // H(2, 3) — 9 vertices, 18 edges, 4-regular.
/// let g = hamming(2, 3, false).unwrap();
/// assert_eq!(g.vcount(), 9);
/// assert_eq!(g.ecount(), 18);
/// ```
pub fn hamming(n: u32, q: u32, directed: bool) -> IgraphResult<Graph> {
    // Singleton in the zero-dimensional case, even when q == 0.
    if n == 0 {
        return Graph::new(1, directed);
    }
    // Null graph for an empty alphabet (with n > 0).
    if q == 0 {
        return Graph::new(0, directed);
    }
    // q == 1: every "string" is the all-zero one — singleton.
    if q == 1 {
        return Graph::new(1, directed);
    }

    // vcount = q^n; overflow-safe via checked_pow.
    let vcount: u32 = q.checked_pow(n).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "hamming: parameters n={n}, q={q} overflow u32 vertex count (q^n)"
        ))
    })?;

    // ecount = q^n · (q − 1) · n / 2. The product (q−1)·n fits because
    // (q−1)·n < q^n once n ≥ 1 and q ≥ 2 (so it cannot overflow at this
    // point, mirroring the upstream comment). The full multiplication
    // is still guarded with checked ops in case future bound changes
    // shift the invariant.
    let qm1: u64 = u64::from(q) - 1;
    let n_u64: u64 = u64::from(n);
    let vc_u64: u64 = u64::from(vcount);
    let prod = qm1
        .checked_mul(n_u64)
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!("hamming: (q-1)*n overflows u64 for n={n}, q={q}"))
        })?
        .checked_mul(vc_u64)
        .ok_or_else(|| {
            IgraphError::InvalidArgument(format!(
                "hamming: ecount product overflows u64 for n={n}, q={q}"
            ))
        })?;
    // Always even because either q is even (vcount even) or (q-1) is
    // even — but we still divide rather than rely on it.
    debug_assert!(prod % 2 == 0, "hamming ecount product must be even");
    let ecount: usize = usize::try_from(prod / 2).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "hamming: edge count {} overflows usize for n={n}, q={q}",
            prod / 2
        ))
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    for v in 0..vcount {
        // Walk digits of v in base q. `pos = q^i` selects the i-th digit.
        let mut pos: u32 = 1;
        for _i in 0..n {
            let dig = (v / pos) % q;
            // For each higher value of this digit, emit canonical edge
            // (v, v + j*pos) — guaranteed to land within [0, q^n) and
            // strictly greater than v because only this digit changes
            // and it grows.
            let mut j: u32 = 1;
            while j < q - dig {
                let u = v + j * pos;
                edges.push((v, u));
                j += 1;
            }
            // pos *= q. At the last iteration this can overflow u32
            // (when q^n hits the boundary), so use checked_mul; if it
            // overflows, we are at the very last digit and won't use
            // the result again — guard accordingly.
            if let Some(next_pos) = pos.checked_mul(q) {
                pos = next_pos;
            } else {
                break;
            }
        }
    }

    let mut g = Graph::new(vcount, directed)?;
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

    fn degree_of(g: &Graph, v: VertexId) -> usize {
        g.degree(v).expect("vertex in range")
    }

    #[test]
    fn n0_is_singleton() {
        // n=0 — singleton regardless of q.
        for q in [0u32, 1, 2, 5, 100] {
            let g = hamming(0, q, false).expect("n=0");
            assert_eq!(g.vcount(), 1, "n=0, q={q}");
            assert_eq!(g.ecount(), 0, "n=0, q={q}");
        }
    }

    #[test]
    fn q0_with_positive_n_is_null_graph() {
        let g = hamming(3, 0, false).expect("q=0");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn q1_is_singleton() {
        // Only one string (the all-zero one), no edges.
        for n in [1u32, 3, 7] {
            let g = hamming(n, 1, false).expect("q=1");
            assert_eq!(g.vcount(), 1, "n={n}, q=1");
            assert_eq!(g.ecount(), 0, "n={n}, q=1");
        }
    }

    #[test]
    fn h_n1_q3_is_complete_k3() {
        // H(1, 3): 3 strings of length 1, every two differ in their
        // single position — that's K_3.
        let g = hamming(1, 3, false).expect("H(1,3)");
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 3);
        assert_eq!(dump_edges(&g), vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn h_n2_q3_matches_upstream_edges() {
        // H(2, 3): 9 vertices, q^n·(q-1)·n/2 = 9·2·2/2 = 18 edges.
        let g = hamming(2, 3, false).expect("H(2,3)");
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 18);
        // Hand-computed canonical edge list following the upstream
        // digit-walk order: for v=0..8, digit 0 (j=1,2), then digit 1
        // (j=1,2) where applicable.
        assert_eq!(
            dump_edges(&g),
            vec![
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 6), // v=0
                (1, 2),
                (1, 4),
                (1, 7), // v=1
                (2, 5),
                (2, 8), // v=2
                (3, 4),
                (3, 5),
                (3, 6), // v=3
                (4, 5),
                (4, 7), // v=4
                (5, 8), // v=5
                (6, 7),
                (6, 8), // v=6
                (7, 8), // v=7
                        // v=8 → no higher edge
            ]
        );
    }

    #[test]
    fn h_n2_q3_is_four_regular() {
        // Every vertex in H(n, q) has degree n*(q-1).
        let g = hamming(2, 3, false).expect("H(2,3)");
        for v in 0..g.vcount() {
            assert_eq!(
                degree_of(&g, v),
                4,
                "vertex {v} of H(2,3) should have degree 4"
            );
        }
    }

    #[test]
    fn q2_equals_hypercube_q3() {
        // H(n, 2) ≡ Q_n. Compare against the standalone hypercube.
        use crate::hypercube;
        for n in 0u32..=4 {
            let h = hamming(n, 2, false).expect("H(n,2)");
            let q = hypercube(n, false).expect("Q_n");
            assert_eq!(h.vcount(), q.vcount(), "n={n} vcount");
            assert_eq!(h.ecount(), q.ecount(), "n={n} ecount");
            assert_eq!(dump_edges(&h), dump_edges(&q), "n={n} edge sequence");
        }
    }

    #[test]
    fn directed_emits_low_to_high_arcs() {
        let g = hamming(2, 3, true).expect("H(2,3) directed");
        assert!(g.is_directed());
        assert_eq!(g.ecount(), 18);
        let m = u32::try_from(g.ecount()).unwrap();
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            assert!(u < v, "edge ({u}, {v}) must be canonically ordered");
        }
    }

    #[test]
    fn h_n3_q2_matches_hypercube_q3() {
        // Spot check: H(3, 2) === Q_3 (8 vertices, 12 edges).
        let g = hamming(3, 2, false).expect("H(3,2)");
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 12);
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn edges_differ_in_one_digit() {
        // For every edge of H(2, 4), the two endpoints differ in exactly
        // one base-4 digit.
        let n: u32 = 2;
        let q: u32 = 4;
        let g = hamming(n, q, false).expect("H(2,4)");
        let m = u32::try_from(g.ecount()).unwrap();
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            let mut diffs = 0u32;
            let mut a = u;
            let mut b = v;
            for _ in 0..n {
                if a % q != b % q {
                    diffs += 1;
                }
                a /= q;
                b /= q;
            }
            assert_eq!(
                diffs, 1,
                "edge ({u}, {v}) in H({n},{q}) must differ in one digit"
            );
        }
    }

    #[test]
    fn vertex_count_overflow_rejected() {
        // q = 2, n = 32 → 2^32 overflows u32.
        let err = hamming(32, 2, false).expect_err("2^32 overflow");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("overflow")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn vertex_count_overflow_high_alphabet_rejected() {
        // q = 100, n = 5 → 100^5 = 10^10 overflows u32.
        let err = hamming(5, 100, false).expect_err("100^5 overflow");
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("overflow")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // For every reasonable (n, q), the Hamming graph H(n, q) is
        // n*(q-1)-regular (or 0-regular when n=0 or q≤1).
        #[test]
        fn n_q_minus_one_regular(n in 0u32..=4, q in 2u32..=5) {
            let g = hamming(n, q, false).unwrap();
            let expected_degree = (n as usize) * (q as usize - 1);
            for v in 0..g.vcount() {
                prop_assert_eq!(g.degree(v).unwrap(), expected_degree);
            }
        }

        // Vertex count is q^n.
        #[test]
        fn vertex_count_is_q_to_the_n(n in 0u32..=5, q in 2u32..=4) {
            let g = hamming(n, q, false).unwrap();
            let expected = (q as u64).pow(n);
            prop_assert_eq!(g.vcount() as u64, expected);
        }

        // Edge count is q^n * (q-1) * n / 2.
        #[test]
        fn edge_count_formula(n in 0u32..=4, q in 2u32..=5) {
            let g = hamming(n, q, false).unwrap();
            let vc = (q as u64).pow(n);
            let expected = (vc * (q as u64 - 1) * (n as u64) / 2) as usize;
            prop_assert_eq!(g.ecount(), expected);
        }

        // H(n, 2) ≡ Q_n: matches hypercube exactly.
        #[test]
        fn q2_equals_hypercube(n in 0u32..=6) {
            use crate::hypercube;
            let h = hamming(n, 2, false).unwrap();
            let q = hypercube(n, false).unwrap();
            prop_assert_eq!(h.vcount(), q.vcount());
            prop_assert_eq!(h.ecount(), q.ecount());
        }
    }
}
