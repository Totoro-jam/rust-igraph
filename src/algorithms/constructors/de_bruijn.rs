//! De Bruijn graph constructor (ALGO-CN-012).
//!
//! Counterpart of `igraph_de_bruijn()` in
//! `references/igraph/src/constructors/de_bruijn.c:58-113`.
//!
//! The De Bruijn graph `B(m, n)` is the **directed** graph on `m^n`
//! vertices whose vertices are length-`n` strings drawn from an alphabet
//! of `m` symbols. There is an arc from vertex `v = (a_1, …, a_n)` to
//! vertex `w = (a_2, …, a_n, b)` for every alphabet symbol `b` — that
//! is, `w` is obtained from `v` by dropping the first symbol and
//! appending one. Each vertex therefore has out-degree exactly `m` and
//! in-degree exactly `m` (when `n ≥ 1`); the total arc count is
//! `m^(n+1) = m · vcount`.
//!
//! Vertices are encoded as integers in `[0, m^n)` via the little-endian
//! base-`m` interpretation that upstream uses. The arc-rewrite then
//! reduces to integer arithmetic: from `i`, the `m` successors are
//! `((i * m) mod m^n) + b` for `b ∈ [0, m)`.
//!
//! Degenerate forms:
//!
//! * `n == 0` → exactly `1` directed vertex with no arcs (matches
//!   upstream — there is exactly one length-0 string and the rewrite has
//!   no symbol to append).
//! * `m == 0` → exactly `0` vertices (null graph).
//! * `m == 1, n == 1` → 1 vertex with a single self-loop (the lone
//!   string maps to itself).
//! * `B(2, 1)` is the directed graph on `{0, 1}` with all four arcs
//!   (including both self-loops).
//!
//! Time complexity: `O(|V| + |E|) = O(m^(n+1))`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the De Bruijn graph `B(m, n)`.
///
/// The result is **always directed**, matches the C convention
/// (`IGRAPH_DIRECTED`). Self-loops appear naturally whenever the
/// rewrite `(i, ((i * m) mod m^n) + b)` lands back on `i` — most
/// notably for `m == 1, n ≥ 1`, where the single vertex `0` has a
/// self-loop, and along the main diagonal of `B(m, 1)`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `m.pow(n)` would overflow `u32`
///   (vertex count too large to address with the graph's `u32` vertex
///   id width). The check uses [`u32::checked_pow`] so the failure mode
///   is deterministic and never produces a silently-wrapped graph.
/// * [`IgraphError::InvalidArgument`] — the edge count `m^(n+1)`
///   overflows `usize` so the edge buffer cannot be sized safely.
///
/// # Examples
///
/// ```
/// use rust_igraph::de_bruijn;
///
/// // B(2, 2) — 4 vertices, 8 arcs, 4 of which are self-loops or pairs.
/// let g = de_bruijn(2, 2).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 8);
/// assert!(g.is_directed());
/// ```
pub fn de_bruijn(m: u32, n: u32) -> IgraphResult<Graph> {
    // Degenerate forms first — matches the upstream order in
    // `igraph_de_bruijn`.
    if n == 0 {
        // There is exactly one length-0 string.
        return Graph::new(1, true);
    }
    if m == 0 {
        return Graph::new(0, true);
    }

    let vcount = m.checked_pow(n).ok_or_else(|| {
        IgraphError::InvalidArgument(format!("de_bruijn: m^n overflows u32 (m = {m}, n = {n})"))
    })?;

    // Edge count = vcount * m = m^(n+1). Compute through usize so the
    // overflow check is meaningful for the edge buffer.
    let vcount_us = usize::try_from(vcount).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "de_bruijn: vcount {vcount} cannot fit usize on this target"
        ))
    })?;
    let m_us = usize::try_from(m).map_err(|_| {
        IgraphError::InvalidArgument(format!("de_bruijn: m {m} cannot fit usize on this target"))
    })?;
    let ecount = vcount_us.checked_mul(m_us).ok_or_else(|| {
        IgraphError::InvalidArgument(format!(
            "de_bruijn: m^(n+1) overflows usize (m = {m}, n = {n})"
        ))
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);

    // For each vertex i, the m successors are ((i * m) mod vcount) + b
    // for b ∈ [0, m). Use u64 for the intermediate product so we don't
    // need a checked_mul inside the inner loop.
    let vcount_u64 = u64::from(vcount);
    let m_u64 = u64::from(m);
    for i in 0..vcount {
        let basis = (u64::from(i) * m_u64) % vcount_u64;
        for b in 0..m {
            // `basis + b` is < `vcount` by construction, so the u32 cast
            // is always safe.
            #[allow(clippy::cast_possible_truncation)]
            let target = (basis + u64::from(b)) as u32;
            edges.push((i, target));
        }
    }

    let mut g = Graph::new(vcount, true)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dump_arcs(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut out = Vec::with_capacity(g.ecount());
        for e in 0..ec {
            // Directed graph: preserve source → target orientation.
            let u = g.edge_source(e).expect("edge in range");
            let v = g.edge_target(e).expect("edge in range");
            out.push((u, v));
        }
        out
    }

    fn vcount_us(g: &Graph) -> usize {
        usize::try_from(g.vcount()).expect("vcount fits usize")
    }

    #[test]
    fn n_zero_yields_singleton_directed() {
        let g = de_bruijn(5, 0).expect("n=0 valid");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn m_zero_yields_null_directed() {
        let g = de_bruijn(0, 4).expect("m=0 valid");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn m_one_n_one_is_single_self_loop() {
        let g = de_bruijn(1, 1).expect("B(1,1) valid");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 1);
        assert_eq!(dump_arcs(&g), vec![(0, 0)]);
    }

    #[test]
    fn b_2_1_is_directed_k2_with_loops() {
        // 2 vertices, 4 arcs: 0→0, 0→1, 1→0, 1→1.
        let g = de_bruijn(2, 1).expect("B(2,1) valid");
        assert_eq!(g.vcount(), 2);
        assert_eq!(g.ecount(), 4);
        let arcs: BTreeSet<_> = dump_arcs(&g).into_iter().collect();
        let expected: BTreeSet<_> = [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().collect();
        assert_eq!(arcs, expected);
    }

    #[test]
    fn b_2_2_exact_arc_list() {
        // vcount = 4, basis = (i * 2) mod 4.
        // i=0 basis=0 → (0,0) (0,1)
        // i=1 basis=2 → (1,2) (1,3)
        // i=2 basis=0 → (2,0) (2,1)
        // i=3 basis=2 → (3,2) (3,3)
        let g = de_bruijn(2, 2).expect("B(2,2) valid");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 8);
        let expected = vec![
            (0, 0),
            (0, 1),
            (1, 2),
            (1, 3),
            (2, 0),
            (2, 1),
            (3, 2),
            (3, 3),
        ];
        assert_eq!(dump_arcs(&g), expected);
    }

    #[test]
    fn b_3_2_exact_arc_count_and_degrees() {
        // vcount = 9, ecount = 27.
        let g = de_bruijn(3, 2).expect("B(3,2) valid");
        assert_eq!(g.vcount(), 9);
        assert_eq!(g.ecount(), 27);
        // Each vertex has out-degree 3 and in-degree 3.
        // Count via raw arc list.
        let arcs = dump_arcs(&g);
        let mut out_deg = [0u32; 9];
        let mut in_deg = [0u32; 9];
        for (u, v) in arcs {
            out_deg[u as usize] += 1;
            in_deg[v as usize] += 1;
        }
        for (v, (&o, &i)) in out_deg.iter().zip(in_deg.iter()).enumerate() {
            assert_eq!(o, 3, "out-degree at {v}");
            assert_eq!(i, 3, "in-degree at {v}");
        }
    }

    #[test]
    fn b_2_3_total_counts() {
        let g = de_bruijn(2, 3).expect("B(2,3) valid");
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 16);
    }

    #[test]
    fn b_4_3_total_counts() {
        let g = de_bruijn(4, 3).expect("B(4,3) valid");
        assert_eq!(g.vcount(), 64);
        assert_eq!(g.ecount(), 256);
    }

    #[test]
    fn rewrite_rule_holds() {
        // Verify on B(3, 2): for every arc (i, j), j must lie in
        // [(i * m) mod vcount, (i * m) mod vcount + m).
        let m: u32 = 3;
        let n: u32 = 2;
        let g = de_bruijn(m, n).expect("valid");
        let vcount = g.vcount();
        for (u, v) in dump_arcs(&g) {
            let basis = (u * m) % vcount;
            assert!(
                v >= basis && v < basis + m,
                "arc ({u} → {v}) violates rewrite (basis = {basis}, m = {m})"
            );
        }
    }

    #[test]
    fn vcount_overflow_rejected() {
        // m = 2, n = 32 → 2^32 = 4_294_967_296 > u32::MAX → reject.
        let err = de_bruijn(2, 32).expect_err("vcount overflow");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn out_degree_uniform_for_all_supported_inputs() {
        for m in 1..=4u32 {
            for n in 1..=3u32 {
                let g = de_bruijn(m, n).expect("valid B(m, n)");
                let vc = vcount_us(&g);
                let expected_ecount = (m as usize).pow(n + 1);
                assert_eq!(
                    g.ecount(),
                    expected_ecount,
                    "ecount mismatch for B({m}, {n})"
                );
                let arcs = dump_arcs(&g);
                let mut out_deg = vec![0u32; vc];
                for (u, _) in arcs {
                    out_deg[u as usize] += 1;
                }
                for (v, &d) in out_deg.iter().enumerate() {
                    assert_eq!(d, m, "out-degree mismatch at {v} in B({m}, {n})");
                }
            }
        }
    }

    #[test]
    fn in_degree_uniform_for_all_supported_inputs() {
        for m in 1..=4u32 {
            for n in 1..=3u32 {
                let g = de_bruijn(m, n).expect("valid B(m, n)");
                let vc = vcount_us(&g);
                let arcs = dump_arcs(&g);
                let mut in_deg = vec![0u32; vc];
                for (_, v) in arcs {
                    in_deg[v as usize] += 1;
                }
                for (v, &d) in in_deg.iter().enumerate() {
                    assert_eq!(d, m, "in-degree mismatch at {v} in B({m}, {n})");
                }
            }
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// `vcount == m^n` and `ecount == m^(n+1)` across all supported
        /// `(m, n)` pairs that fit in `u32`.
        #[test]
        fn vcount_and_ecount_are_exact(m in 1u32..=5, n in 1u32..=4) {
            // Skip configurations that would blow past u32 vcount.
            let Some(vcount) = m.checked_pow(n) else { return Ok(()); };
            let Some(ecount) = vcount.checked_mul(m) else { return Ok(()); };
            let g = de_bruijn(m, n).expect("valid input");
            prop_assert_eq!(g.vcount(), vcount);
            prop_assert_eq!(g.ecount() as u64, u64::from(ecount));
        }

        /// Every vertex has out-degree `m` and in-degree `m`.
        #[test]
        fn vertex_in_and_out_degree_equals_m(m in 1u32..=4, n in 1u32..=3) {
            let g = de_bruijn(m, n).expect("valid input");
            let vcount = g.vcount();
            let vc = usize::try_from(vcount).unwrap();
            let mut out_deg = vec![0u32; vc];
            let mut in_deg = vec![0u32; vc];
            let ec = u32::try_from(g.ecount()).unwrap();
            for e in 0..ec {
                let u = g.edge_source(e).expect("edge in range");
                let v = g.edge_target(e).expect("edge in range");
                out_deg[u as usize] += 1;
                in_deg[v as usize] += 1;
            }
            for v in 0..vc {
                prop_assert_eq!(out_deg[v], m);
                prop_assert_eq!(in_deg[v], m);
            }
        }

        /// Every arc `(u, v)` satisfies the rewrite `v ∈ [basis, basis + m)`
        /// where `basis = (u * m) mod vcount`.
        #[test]
        fn rewrite_rule_holds_for_every_arc(m in 1u32..=4, n in 1u32..=3) {
            let g = de_bruijn(m, n).expect("valid input");
            let vcount = g.vcount();
            let ec = u32::try_from(g.ecount()).unwrap();
            for e in 0..ec {
                let u = g.edge_source(e).expect("edge in range");
                let v = g.edge_target(e).expect("edge in range");
                let basis = (u * m) % vcount;
                prop_assert!(v >= basis && v < basis + m);
            }
        }
    }
}
