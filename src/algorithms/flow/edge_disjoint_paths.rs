//! `edge_disjoint_paths` (ALGO-FL-012) — maximum number of
//! edge-disjoint `source → target` paths.
//!
//! Counterpart of `igraph_edge_disjoint_paths` in
//! `references/igraph/src/flow/flow.c` (lines 2326-2343). The C source
//! is a 15-line wrapper that rejects `source == target` and then
//! delegates to `igraph_maxflow_value` with `NULL` capacity (unit
//! capacities per edge), casting the resulting `igraph_real_t` to
//! `igraph_int_t` (int64). By Menger's theorem (1927) the maximum
//! number of pairwise edge-disjoint `s → t` paths equals the
//! cardinality of the minimum s-t edge cut, which on unit capacities
//! equals the s-t maximum flow.
//!
//! Note that `edge_disjoint_paths(g, s, t) == st_edge_connectivity(g,
//! s, t)` whenever both functions are defined; both are aliases for
//! `round(max_flow_value(g, s, t, None))`. We keep them as separate
//! Rust functions for naming parity with igraph C (so call sites can
//! pick the name that matches their intent — "paths" vs.
//! "connectivity").

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

use super::max_flow::max_flow_value;

/// Maximum number of pairwise edge-disjoint paths from `source` to
/// `target`.
///
/// Counterpart of `igraph_edge_disjoint_paths` in
/// `references/igraph/src/flow/flow.c:2326`. By Menger's theorem
/// (1927) this equals the s-t edge connectivity (and, on unit
/// capacities, the s-t maximum flow); see
/// [`st_edge_connectivity`](super::st_edge_connectivity::st_edge_connectivity)
/// for the connectivity-flavoured spelling.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — sink vertex id (`0 ≤ target < vcount()`,
///   `target != source`).
///
/// # Returns
///
/// The maximum number of edge-disjoint `source → target` paths as
/// `i64`. Returns `0` when no `source → target` path exists.
///
/// # Errors
///
/// Same as [`max_flow_value`](super::max_flow::max_flow_value):
///
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::InvalidArgument`] if `source == target`. (igraph
///   C returns `IGRAPH_UNIMPLEMENTED` for this case; we use
///   `InvalidArgument` for parity with the rest of our flow module.)
/// * [`IgraphError::Internal`] if the unit-capacity max-flow value is
///   not representable as `i64` (unreachable in practice).
///
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
/// [`IgraphError::Internal`]: crate::core::IgraphError::Internal
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_disjoint_paths};
///
/// // Two parallel unit-cap paths 0→1→3 and 0→2→3 → 2 edge-disjoint paths.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert_eq!(edge_disjoint_paths(&g, 0, 3).unwrap(), 2);
/// ```
pub fn edge_disjoint_paths(graph: &Graph, source: VertexId, target: VertexId) -> IgraphResult<i64> {
    let flow = max_flow_value(graph, source, target, None)?;
    // Unit-capacity max-flow is integral and bounded above by `ecount()`
    // (≤ u32::MAX), so the f64 → i64 round + cast is lossless in every
    // reachable case. The clippy allows below are *known* properties of
    // this conversion, not bugs.
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
        let err = edge_disjoint_paths(&g, 0, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::new(2, true).expect("graph");
        let err = edge_disjoint_paths(&g, 5, 0).unwrap_err();
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
        let err = edge_disjoint_paths(&g, 0, 5).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn isolated_endpoints_have_no_paths() {
        let g = Graph::new(4, true).expect("graph");
        assert_eq!(edge_disjoint_paths(&g, 0, 3).expect("paths"), 0);
    }

    #[test]
    fn single_edge_one_path() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert_eq!(edge_disjoint_paths(&g, 0, 1).expect("paths"), 1);
    }

    #[test]
    fn two_parallel_paths() {
        // 0→1→3 and 0→2→3 → 2 edge-disjoint paths.
        let mut g = Graph::new(4, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 3), (0, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(edge_disjoint_paths(&g, 0, 3).expect("paths"), 2);
    }

    #[test]
    fn parallel_arcs_count_each_as_path() {
        let mut g = Graph::new(2, true).expect("graph");
        for _ in 0..4 {
            g.add_edge(0, 1).expect("edge");
        }
        assert_eq!(edge_disjoint_paths(&g, 0, 1).expect("paths"), 4);
    }

    #[test]
    fn directed_no_reverse_path() {
        // 0 → 1 → 2 → 3 directed chain.
        let mut g = Graph::new(4, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (1, 2), (2, 3)] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(edge_disjoint_paths(&g, 0, 3).expect("paths"), 1);
        // Reverse direction has no paths.
        assert_eq!(edge_disjoint_paths(&g, 3, 0).expect("paths"), 0);
    }

    #[test]
    fn c_unit_fixture_directed_0_to_5() {
        // Mirrors igraph_edge_disjoint_paths.c — directed 6-vertex
        // graph with a self-loop at vertex 3. ep(0→5) = 2.
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
            (3, 3),
        ];
        for (s, t) in arcs {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(edge_disjoint_paths(&g, 0, 5).expect("paths"), 2);
        assert_eq!(edge_disjoint_paths(&g, 0, 3).expect("paths"), 1);
        assert_eq!(edge_disjoint_paths(&g, 3, 0).expect("paths"), 0);
        assert_eq!(edge_disjoint_paths(&g, 3, 5).expect("paths"), 2);
    }

    #[test]
    fn c_unit_fixture_undirected_4_to_3() {
        // After igraph_to_undirected the same fixture has 3 vertex-paths
        // 4→3 (direct, via 1, via 2). ep(4, 3) = 3.
        let mut g = Graph::new(6, false).expect("graph");
        for (s, t) in [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (3, 5),
            (4, 5),
            (3, 3),
        ] {
            g.add_edge(s, t).expect("edge");
        }
        assert_eq!(edge_disjoint_paths(&g, 4, 3).expect("paths"), 3);
    }

    #[test]
    fn matches_st_edge_connectivity() {
        // Belt-and-suspenders: every fixture should agree with
        // st_edge_connectivity by Menger's theorem.
        use super::super::st_edge_connectivity::st_edge_connectivity;
        let mut g = Graph::new(5, false).expect("graph");
        for i in 0u32..5 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).expect("edge");
            }
        }
        assert_eq!(
            edge_disjoint_paths(&g, 0, 4).expect("paths"),
            st_edge_connectivity(&g, 0, 4).expect("ec")
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest cross-validates the Menger equality:
    //! `edge_disjoint_paths(g, s, t) == st_edge_connectivity(g, s, t)`
    //! for every random unit-cap graph. Both functions are themselves
    //! cross-validated against `max_flow_value` (see `max_flow.rs`
    //! proptests), so this transitively inherits the Edmonds-Karp
    //! independent reference.

    use super::super::st_edge_connectivity::st_edge_connectivity;
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
        fn menger_equals_st_edge_connectivity(
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

            let paths = edge_disjoint_paths(&g, s, t).expect("paths");
            let ec = st_edge_connectivity(&g, s, t).expect("ec");
            prop_assert_eq!(
                paths,
                ec,
                "Menger violated: paths={} ec={} (n={}, m={}, directed={}, seed={})",
                paths, ec, n, m, directed, seed
            );
        }
    }
}
