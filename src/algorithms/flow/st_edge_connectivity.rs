//! `st_edge_connectivity` (ALGO-FL-011) — minimum number of edges
//! whose removal disconnects `source` from `target`.
//!
//! Counterpart of `igraph_st_edge_connectivity` in
//! `references/igraph/src/flow/flow.c` (lines 2219-2234). The C source
//! is a 15-line wrapper that rejects `source == target` and then
//! delegates to `igraph_maxflow_value` with `NULL` capacity (unit
//! capacities per edge), casting the resulting `igraph_real_t` to
//! `igraph_int_t` (int64). Unit-capacity max-flow equals the
//! cardinality of the minimum edge cut by the max-flow / min-cut
//! theorem (Ford-Fulkerson, 1956), so the truncation is lossless on
//! integer outputs.
//!
//! We follow the same pattern: this is a thin wrapper over
//! [`super::max_flow::max_flow_value`] with `capacity = None`.
//! Validation of vertex-id bounds and `source != target` is delegated
//! to `max_flow_value`, so the error contract is identical.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::max_flow::max_flow_value;

/// Edge connectivity between two vertices: the minimum number of edges
/// whose removal disconnects `source` from `target`.
///
/// Counterpart of `igraph_st_edge_connectivity` in
/// `references/igraph/src/flow/flow.c:2219`. By the max-flow / min-cut
/// theorem with unit capacities this equals
/// [`max_flow_value`](super::max_flow::max_flow_value)`(g, s, t, None)`
/// (cast to an integer). This function exists for naming parity with
/// igraph C and to give call sites a typed integer answer when the
/// caller wants the connectivity interpretation rather than the flow
/// one.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected). For undirected
///   graphs both arc directions are open, matching igraph C.
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — sink vertex id (`0 ≤ target < vcount()`,
///   `target != source`).
///
/// # Returns
///
/// The number of edges in a minimum `source → target` edge cut as
/// `i64`. Returns `0` when no `source → target` path exists (the empty
/// cut already disconnects them).
///
/// # Errors
///
/// Same as [`max_flow_value`](super::max_flow::max_flow_value):
///
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::InvalidArgument`] if `source == target`.
/// * [`IgraphError::Internal`] if the unit-capacity max-flow value is
///   not representable as `i64` (in practice unreachable: it is
///   bounded by `ecount()`, which fits in `u32`).
///
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::Internal`]: crate::core::IgraphError::Internal
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, st_edge_connectivity};
///
/// // Two parallel unit-cap paths 0→1→3 and 0→2→3 → connectivity = 2.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert_eq!(st_edge_connectivity(&g, 0, 3).unwrap(), 2);
/// ```
pub fn st_edge_connectivity(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
) -> IgraphResult<i64> {
    let flow = max_flow_value(graph, source, target, None)?;
    // Unit-capacity max-flow is integral and bounded above by `ecount()`
    // (≤ u32::MAX), so the f64 → i64 round + cast is lossless in every
    // reachable case. The two lints below are *known* properties of
    // that conversion, not bugs: we want the truncation (rounded) and
    // the precision-loss bound (compared against `i64::MAX`) precisely
    // to defend against numerical drift.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    {
        if !flow.is_finite() || flow < 0.0 || flow > i64::MAX as f64 {
            return Err(IgraphError::Internal(
                "unit-capacity max-flow value is not representable as i64",
            ));
        }
        Ok(flow.round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::IgraphError;

    #[test]
    fn rejects_source_equals_target() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let err = st_edge_connectivity(&g, 0, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::new(2, true).expect("graph");
        let err = st_edge_connectivity(&g, 5, 0).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_target() {
        let g = Graph::new(2, true).expect("graph");
        let err = st_edge_connectivity(&g, 0, 5).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn isolated_endpoints_have_zero_connectivity() {
        let g = Graph::new(4, true).expect("graph");
        assert_eq!(st_edge_connectivity(&g, 0, 3).expect("ec"), 0);
    }

    #[test]
    fn single_edge_unit() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(st_edge_connectivity(&g, 0, 1).expect("ec"), 1);
    }

    #[test]
    fn two_parallel_paths() {
        // 0→1→3 and 0→2→3 → ec(0,3) = 2.
        let mut g = Graph::new(4, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(st_edge_connectivity(&g, 0, 3).expect("ec"), 2);
    }

    #[test]
    fn three_parallel_arcs_directed() {
        let mut g = Graph::new(2, true).expect("graph");
        for _ in 0..3 {
            g.add_edge(0, 1).expect("edge");
        }
        assert_eq!(st_edge_connectivity(&g, 0, 1).expect("ec"), 3);
    }

    #[test]
    fn directed_anti_parallel_arc_does_not_count() {
        // 0 → 1 → 0 (anti-parallel). ec(0,1) only counts arcs going
        // s → t direction = 1.
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 0).expect("edge");
        assert_eq!(st_edge_connectivity(&g, 0, 1).expect("ec"), 1);
    }

    #[test]
    fn undirected_bottleneck() {
        // Path 0 — 1 — 2 — 3 (undirected). Any single edge separates
        // endpoints → ec(0, 3) = 1.
        let mut g = Graph::new(4, false).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(st_edge_connectivity(&g, 0, 3).expect("ec"), 1);
    }

    #[test]
    fn c_unit_test_fixture() {
        // Mirrors references/igraph/tests/unit/igraph_st_edge_connectivity.c
        // 6 vertices, 8 directed arcs, ec(0, 5) = 2.
        let mut g = Graph::new(6, true).expect("graph");
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
        ];
        for (s, t) in arcs {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(st_edge_connectivity(&g, 0, 5).expect("ec"), 2);
    }

    #[test]
    fn full_graph_5v_undirected() {
        // K_5 undirected. ec(any-pair) = 4 (each vertex has degree 4).
        let mut g = Graph::new(5, false).expect("graph");
        for i in 0u32..5 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).expect("edge");
            }
        }
        assert_eq!(st_edge_connectivity(&g, 0, 1).expect("ec"), 4);
        assert_eq!(st_edge_connectivity(&g, 2, 4).expect("ec"), 4);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest cross-validates the wrapper invariant: for every legal
    //! random unit-capacity input,
    //! `st_edge_connectivity(g, s, t) ==
    //!  round(max_flow_value(g, s, t, None))`.
    //! Since `max_flow_value` is itself proptest-cross-validated
    //! against an independent Edmonds-Karp reference (see
    //! `max_flow.rs`), this transitively inherits that cross-check.

    use super::*;
    use crate::core::Graph;
    use proptest::prelude::*;

    fn xorshift(mut r: u64) -> u64 {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        r
    }

    fn build_random(seed: u64, n: u32, m_max: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        let mut state = seed | 1;
        for _ in 0..m_max {
            state = xorshift(state);
            let u = u32::try_from(state % u64::from(n)).expect("modulo fits");
            state = xorshift(state);
            let v = u32::try_from(state % u64::from(n)).expect("modulo fits");
            if u == v {
                continue;
            }
            g.add_edge(u, v).expect("edge");
        }
        g
    }

    proptest! {
        #[test]
        fn ec_equals_unit_maxflow(
            seed in any::<u64>(),
            n in 2u32..8,
            m in 1u32..16,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let s = u32::try_from(seed % u64::from(n)).expect("modulo fits");
            let t_raw = u32::try_from(xorshift(seed) % u64::from(n)).expect("modulo fits");
            let t = if t_raw == s { (s + 1) % n } else { t_raw };
            prop_assume!(s != t);

            let flow = max_flow_value(&g, s, t, None).expect("flow");
            let ec = st_edge_connectivity(&g, s, t).expect("ec");
            let want = flow.round() as i64;
            prop_assert_eq!(
                ec,
                want,
                "ec/flow mismatch: ec={} flow={} (n={}, m={}, directed={}, seed={})",
                ec, flow, n, m, directed, seed
            );
            // Sanity: connectivity is bounded by min(out-deg(s), in-deg(t))
            // when directed, and degrees when undirected. We can't easily
            // cheap-check this here, but assert it's >= 0 and <= m.
            prop_assert!(ec >= 0);
            prop_assert!(ec as u64 <= u64::from(m));
        }
    }
}
