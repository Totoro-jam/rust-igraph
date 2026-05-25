//! Full graph (complete graph) constructor (ALGO-CN-014).
//!
//! Counterpart of `igraph_full()` in
//! `references/igraph/src/constructors/full.c:54-124`.
//!
//! In a *full* graph every possible edge is present: every vertex is
//! connected to every other vertex. The two boolean flags shape the
//! ecount:
//!
//! | directed | loops | ecount         | emission pattern                       |
//! |----------|-------|----------------|----------------------------------------|
//! | false    | false | `n·(n−1)/2`    | `(i, j)` for every `0 ≤ i < j < n`     |
//! | false    | true  | `n·(n+1)/2`    | `(i, j)` for every `0 ≤ i ≤ j < n`     |
//! | true     | false | `n·(n−1)`      | `(i, j)` for every `i ≠ j`             |
//! | true     | true  | `n²`           | `(i, j)` for every `0 ≤ i, j < n`      |
//!
//! Emission order matches upstream byte-for-byte (source-major,
//! target-ascending; for the `directed && !loops` case the inner loop
//! walks `j ∈ [0, i)` first then `j ∈ (i, n)`).
//!
//! Time complexity: `O(|V|^2) = O(|E|)`.
//!
//! Degenerate inputs:
//!
//! * `n == 0` → empty graph (vcount = 0, ecount = 0).
//! * `n == 1 ∧ loops == false` → singleton, no edges.
//! * `n == 1 ∧ loops == true` → singleton with a single self-loop
//!   `(0, 0)`.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Build the full (complete) graph on `n` vertices.
///
/// The semantics follow `igraph_full(graph, n, directed, loops)`:
///
/// * `directed = false, loops = false` → `K_n` with `n·(n−1)/2` edges.
/// * `directed = false, loops = true`  → `K_n` plus one self-loop per
///   vertex (`n·(n+1)/2` edges total).
/// * `directed = true,  loops = false` → directed `K_n` with both
///   `(i, j)` and `(j, i)` arcs for every `i ≠ j` (`n·(n−1)` arcs).
/// * `directed = true,  loops = true`  → above plus a self-loop on every
///   vertex (`n²` arcs total).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — the upper bound on the edge
///   count (`n²` in the worst case) overflows `usize`, so the edge
///   buffer cannot be sized safely. Computed via [`usize::checked_mul`]
///   so the failure mode is deterministic.
///
/// # Examples
///
/// ```
/// use rust_igraph::full_graph;
///
/// // K_4 — undirected complete graph on 4 vertices, 6 edges.
/// let k4 = full_graph(4, false, false).unwrap();
/// assert_eq!(k4.vcount(), 4);
/// assert_eq!(k4.ecount(), 6);
/// assert!(!k4.is_directed());
///
/// // Directed K_3 with loops — 9 arcs (3 × 3).
/// let dk3 = full_graph(3, true, true).unwrap();
/// assert_eq!(dk3.vcount(), 3);
/// assert_eq!(dk3.ecount(), 9);
/// assert!(dk3.is_directed());
/// ```
pub fn full_graph(n: u32, directed: bool, loops: bool) -> IgraphResult<Graph> {
    if n == 0 {
        return Graph::new(0, directed);
    }

    let n_us = usize::try_from(n)
        .map_err(|_| IgraphError::InvalidArgument(format!("full_graph: n {n} cannot fit usize")))?;
    let ecount = expected_ecount(n_us, directed, loops)?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);

    match (directed, loops) {
        (true, true) => {
            // n² arcs, source-major, j walks the full row.
            for i in 0..n {
                for j in 0..n {
                    edges.push((i, j));
                }
            }
        }
        (true, false) => {
            // n·(n−1) arcs, source-major; j walks [0, i) then (i, n).
            for i in 0..n {
                for j in 0..i {
                    edges.push((i, j));
                }
                for j in (i + 1)..n {
                    edges.push((i, j));
                }
            }
        }
        (false, true) => {
            // n·(n+1)/2 edges, canonical (i ≤ j).
            for i in 0..n {
                for j in i..n {
                    edges.push((i, j));
                }
            }
        }
        (false, false) => {
            // n·(n−1)/2 edges, canonical (i < j).
            for i in 0..n {
                for j in (i + 1)..n {
                    edges.push((i, j));
                }
            }
        }
    }

    debug_assert_eq!(edges.len(), ecount);

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Worst-case (= exact) edge count for the requested `(directed, loops)`
/// configuration. Returns `InvalidArgument` if the product would
/// overflow `usize`.
fn expected_ecount(n: usize, directed: bool, loops: bool) -> IgraphResult<usize> {
    match (directed, loops) {
        (true, true) => n.checked_mul(n).ok_or_else(|| {
            IgraphError::InvalidArgument(format!("full_graph: n² overflows usize (n = {n})"))
        }),
        (true, false) => {
            // n·(n−1). When n == 0 we already returned above.
            n.checked_mul(n - 1).ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "full_graph: n·(n−1) overflows usize (n = {n})"
                ))
            })
        }
        (false, true) => {
            // n·(n+1)/2. Compute n·(n+1) safely, then halve.
            let prod = n
                .checked_mul(n.checked_add(1).ok_or_else(|| {
                    IgraphError::InvalidArgument(format!(
                        "full_graph: n+1 overflows usize (n = {n})"
                    ))
                })?)
                .ok_or_else(|| {
                    IgraphError::InvalidArgument(format!(
                        "full_graph: n·(n+1) overflows usize (n = {n})"
                    ))
                })?;
            Ok(prod / 2)
        }
        (false, false) => {
            // n·(n−1)/2.
            let prod = n.checked_mul(n - 1).ok_or_else(|| {
                IgraphError::InvalidArgument(format!(
                    "full_graph: n·(n−1) overflows usize (n = {n})"
                ))
            })?;
            Ok(prod / 2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dump_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        let mut out = Vec::with_capacity(g.ecount());
        for e in 0..ec {
            let u = g.edge_source(e).expect("edge in range");
            let v = g.edge_target(e).expect("edge in range");
            out.push((u, v));
        }
        out
    }

    #[test]
    fn null_graph_zero_vertices() {
        let g = full_graph(0, false, false).expect("K_0");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn singleton_no_loops() {
        let g = full_graph(1, false, false).expect("K_1, no loops");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn singleton_with_loops() {
        let g = full_graph(1, false, true).expect("K_1, with loops");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 1);
        assert_eq!(dump_edges(&g), vec![(0, 0)]);
    }

    #[test]
    fn undirected_no_loops_k10_emission_order() {
        // Cross-reference with references/igraph/tests/unit/full.out
        // "Undirected, no loops".
        let g = full_graph(10, false, false).expect("K_10");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 45);
        let edges = dump_edges(&g);
        // First five edges in upstream emission order.
        assert_eq!(edges[0], (0, 1));
        assert_eq!(edges[1], (0, 2));
        assert_eq!(edges[8], (0, 9));
        assert_eq!(edges[9], (1, 2));
        // Last edge: (8, 9).
        assert_eq!(*edges.last().unwrap(), (8, 9));
    }

    #[test]
    fn directed_no_loops_k10_emission_order() {
        let g = full_graph(10, true, false).expect("directed K_10");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 90);
        let arcs = dump_edges(&g);
        // For source i, arcs are (i, 0..i) followed by (i, i+1..n).
        assert_eq!(arcs[0], (0, 1));
        assert_eq!(arcs[8], (0, 9));
        // Source 1's row: (1, 0), (1, 2), …, (1, 9).
        assert_eq!(arcs[9], (1, 0));
        assert_eq!(arcs[10], (1, 2));
        // Last arc: (9, 8).
        assert_eq!(*arcs.last().unwrap(), (9, 8));
    }

    #[test]
    fn undirected_with_loops_k10_counts() {
        let g = full_graph(10, false, true).expect("undirected K_10 + loops");
        assert_eq!(g.vcount(), 10);
        // n(n+1)/2 = 55.
        assert_eq!(g.ecount(), 55);
        let edges: BTreeSet<_> = dump_edges(&g).into_iter().collect();
        let mut expected: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();
        for i in 0u32..10 {
            for j in i..10 {
                expected.insert((i, j));
            }
        }
        assert_eq!(edges, expected);
    }

    #[test]
    fn directed_with_loops_k10_counts() {
        let g = full_graph(10, true, true).expect("directed K_10 + loops");
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 100);
        let arcs: BTreeSet<_> = dump_edges(&g).into_iter().collect();
        let mut expected: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();
        for i in 0u32..10 {
            for j in 0u32..10 {
                expected.insert((i, j));
            }
        }
        assert_eq!(arcs, expected);
    }

    #[test]
    fn directed_no_loops_matches_directed_complete_kautz_n_zero() {
        // K_{m+1} from CN-013 (kautz with n=0) is exactly
        // full_graph(m+1, directed=true, loops=false). Verify the same
        // arc multiset over the m+1 = 4 case.
        let g_full = full_graph(4, true, false).expect("directed K_4");
        let arcs: BTreeSet<_> = dump_edges(&g_full).into_iter().collect();
        let mut expected = BTreeSet::new();
        for i in 0u32..4 {
            for j in 0u32..4 {
                if i != j {
                    expected.insert((i, j));
                }
            }
        }
        assert_eq!(arcs, expected);
    }

    #[test]
    fn ecount_overflow_rejected() {
        // n² ≥ usize::MAX is impossible on 64-bit, but on 32-bit hosts
        // `n = 65_537` would overflow. Pick a value that overflows both:
        // n² overflows u32 at n ≥ 65_536, but we test the u32 → usize
        // path is solid by feeding a parameter that fits u32 yet would
        // produce an absurdly large edge buffer. For portability we
        // limit the test to the `(true, true)` branch by exercising the
        // helper directly.
        let r = expected_ecount(usize::MAX / 2, true, true);
        assert!(matches!(r, Err(IgraphError::InvalidArgument(_))));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Edge count matches the closed-form for every `(directed, loops)`
        /// combination.
        #[test]
        fn ecount_matches_closed_form(n in 0u32..=12u32, directed in proptest::bool::ANY, loops in proptest::bool::ANY) {
            let g = full_graph(n, directed, loops).expect("valid input");
            let n_us = n as usize;
            let expected = match (directed, loops) {
                (true, true) => n_us * n_us,
                (true, false) => n_us * n_us.saturating_sub(1),
                (false, true) => n_us * (n_us + 1) / 2,
                (false, false) => n_us * n_us.saturating_sub(1) / 2,
            };
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), expected);
            prop_assert_eq!(g.is_directed(), directed);
        }

        /// Without loops, no `(v, v)` edge ever appears. With loops, every
        /// vertex has exactly one self-loop.
        #[test]
        fn loop_flag_controls_self_loops(n in 1u32..=10u32, directed in proptest::bool::ANY) {
            for &loops in &[false, true] {
                let g = full_graph(n, directed, loops).expect("valid input");
                let ec = u32::try_from(g.ecount()).unwrap();
                let mut loop_count = 0u32;
                for e in 0..ec {
                    let u = g.edge_source(e).expect("edge in range");
                    let v = g.edge_target(e).expect("edge in range");
                    if u == v {
                        loop_count += 1;
                    }
                }
                if loops {
                    prop_assert_eq!(loop_count, n, "with loops every vertex has exactly one self-loop");
                } else {
                    prop_assert_eq!(loop_count, 0u32, "loops=false must yield no self-loops");
                }
            }
        }

        /// The undirected non-looped variant is exactly the unordered pair set
        /// `{ (i, j) : 0 ≤ i < j < n }` — emission order is checked separately
        /// in the deterministic tests above, here we only check the multiset.
        #[test]
        fn undirected_no_loops_yields_pair_set(n in 0u32..=8u32) {
            let g = full_graph(n, false, false).expect("valid input");
            let ec = u32::try_from(g.ecount()).unwrap();
            let mut got: BTreeSet<(VertexId, VertexId)> = BTreeSet::new();
            for e in 0..ec {
                let u = g.edge_source(e).expect("edge in range");
                let v = g.edge_target(e).expect("edge in range");
                let canon = if u <= v { (u, v) } else { (v, u) };
                prop_assert!(got.insert(canon), "duplicate undirected edge ({u}, {v})");
            }
            let mut expected = BTreeSet::new();
            for i in 0u32..n {
                for j in (i + 1)..n {
                    expected.insert((i, j));
                }
            }
            prop_assert_eq!(got, expected);
        }
    }
}
