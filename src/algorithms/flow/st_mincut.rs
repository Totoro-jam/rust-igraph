//! `st_mincut_value` (ALGO-FL-010) — scalar s-t minimum-cut value.
//!
//! Counterpart of `igraph_st_mincut_value` in
//! `references/igraph/src/flow/flow.c` (lines 1127-1138). The C source
//! is a 5-line wrapper that rejects `source == target` and then
//! delegates to `igraph_maxflow_value`; the value returned is the
//! capacity of the minimum s-t edge cut, which equals the maximum s-t
//! flow by the max-flow / min-cut theorem (Ford-Fulkerson, 1956).
//!
//! We follow the same pattern: this is a thin wrapper over
//! [`super::max_flow::max_flow_value`]. All input validation
//! (vertex-id bounds, `source != target`, capacity length, capacity
//! finiteness / non-negativity) is delegated to `max_flow_value`, so
//! the error contract is identical to that function.

// Vertex-id casts in the proptest helper compute `state % u64::from(n)`
// before truncating to `u32` — the result is always `< n <= u32::MAX`,
// so no truncation occurs at runtime. Same dispensation as
// `max_flow.rs`; keep both modules in sync.
#![allow(clippy::cast_possible_truncation)]

use crate::core::{Graph, IgraphResult, VertexId};

use super::max_flow::max_flow_value;

/// Scalar s-t minimum-cut value (capacity of the minimum edge set
/// whose removal disconnects `source` from `target`).
///
/// Counterpart of `igraph_st_mincut_value` in
/// `references/igraph/src/flow/flow.c:1127`. By the max-flow /
/// min-cut theorem (Ford-Fulkerson, 1956) the value returned equals
/// [`max_flow_value`](super::max_flow::max_flow_value); this function
/// is a thin wrapper that exists for naming parity with igraph C and
/// to make call sites intent-revealing when the caller wants the
/// cut interpretation rather than the flow one.
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — sink vertex id (`0 ≤ target < vcount()`,
///   `target != source`).
/// * `capacity` — optional per-edge capacity in the graph's edge-id
///   order. When `None`, each edge contributes unit capacity. When
///   `Some(c)`, `c.len()` must equal `graph.ecount()`, and every entry
///   must be finite and `≥ 0`.
///
/// # Returns
///
/// The minimum s-t cut capacity as `f64`. Returns `0.0` when no
/// `source → target` path exists in the residual network (the empty
/// cut already disconnects them).
///
/// # Errors
///
/// Same as [`max_flow_value`](super::max_flow::max_flow_value):
///
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::InvalidArgument`] if `source == target`, the
///   capacity slice length differs from `ecount()`, or any capacity
///   is negative / non-finite.
///
/// [`IgraphError::VertexOutOfRange`]: crate::core::IgraphError::VertexOutOfRange
/// [`IgraphError::InvalidArgument`]: crate::core::IgraphError::InvalidArgument
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, st_mincut_value};
///
/// // Two parallel paths of capacity 1 each → min s-t cut = 2
/// // (must cut both bottleneck edges to disconnect 0 from 3).
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cap = vec![1.0, 1.0, 1.0, 1.0];
/// let cut = st_mincut_value(&g, 0, 3, Some(&cap)).unwrap();
/// assert!((cut - 2.0).abs() < 1e-12);
/// ```
pub fn st_mincut_value(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    capacity: Option<&[f64]>,
) -> IgraphResult<f64> {
    max_flow_value(graph, source, target, capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::IgraphError;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12_f64 * a.abs().max(b.abs()).max(1.0)
    }

    #[test]
    fn rejects_source_equals_target() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let err = st_mincut_value(&g, 0, 0, None).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::new(2, true).expect("graph");
        let err = st_mincut_value(&g, 5, 0, None).unwrap_err();
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
        let err = st_mincut_value(&g, 0, 5, None).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            other => panic!("expected VertexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn isolated_endpoints_have_zero_cut() {
        // Disconnected source and target — the empty cut already
        // separates them.
        let g = Graph::new(4, true).expect("graph");
        let cut = st_mincut_value(&g, 0, 3, None).expect("cut");
        assert!(approx(cut, 0.0));
    }

    #[test]
    fn single_edge_unit_cut() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let cut = st_mincut_value(&g, 0, 1, None).expect("cut");
        assert!(approx(cut, 1.0));
    }

    #[test]
    fn two_parallel_paths_cut_equals_two() {
        // 0→1→3 and 0→2→3, unit caps. Min cut = 2.
        let mut g = Graph::new(4, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 3).expect("edge");
        g.add_edge(0, 2).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let cut = st_mincut_value(&g, 0, 3, Some(&[1.0, 1.0, 1.0, 1.0])).expect("cut");
        assert!(approx(cut, 2.0));
    }

    #[test]
    fn bottleneck_directed() {
        // 0 → 1 (cap 5) → 2 (cap 2) → 3 (cap 7).
        // Min cut = 2 (the (1,2) edge is the unique bottleneck).
        let mut g = Graph::new(4, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 2).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let cut = st_mincut_value(&g, 0, 3, Some(&[5.0, 2.0, 7.0])).expect("cut");
        assert!(approx(cut, 2.0));
    }

    #[test]
    fn classic_max_flow_textbook_cut() {
        // CLRS 26.1-1 — max flow = 23, so min s-t cut = 23 by duality.
        let mut g = Graph::new(6, true).expect("graph");
        let arcs = [
            (0u32, 1u32),
            (0, 2),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 4),
            (3, 2),
            (3, 5),
            (4, 3),
            (4, 5),
        ];
        let caps = [16.0, 13.0, 10.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0];
        for (u, v) in arcs {
            g.add_edge(u, v).expect("edge");
        }
        let cut = st_mincut_value(&g, 0, 5, Some(&caps)).expect("cut");
        assert!(approx(cut, 23.0));
    }

    #[test]
    fn undirected_cut_matches_max_flow() {
        // igraph_maxflow.c:213 4-vertex undirected reference: max flow = 4.
        let mut g = Graph::new(4, false).expect("graph");
        for (a, b) in [(0u32, 1u32), (0, 2), (1, 2), (1, 3), (2, 3)] {
            g.add_edge(a, b).expect("edge");
        }
        let cut = st_mincut_value(&g, 0, 3, Some(&[4.0, 2.0, 10.0, 2.0, 2.0])).expect("cut");
        assert!(approx(cut, 4.0));
    }

    #[test]
    fn equals_max_flow_value() {
        // Belt-and-suspenders: assert wrapper agrees with the
        // delegate on a non-trivial fixture.
        let mut g = Graph::new(5, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (1, 4)] {
            g.add_edge(s, t).expect("edge");
        }
        let caps = [3.0, 5.0, 2.0, 4.0, 6.0, 1.0];
        let flow = max_flow_value(&g, 0, 4, Some(&caps)).expect("flow");
        let cut = st_mincut_value(&g, 0, 4, Some(&caps)).expect("cut");
        assert!(approx(flow, cut));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest cross-validates the wrapper invariant: for every legal
    //! input, `st_mincut_value(g, s, t, c) == max_flow_value(g, s, t, c)`.
    //! This is the duality theorem at the value level — and since
    //! `max_flow_value` is itself proptest-cross-validated against an
    //! independent Edmonds-Karp reference (see `max_flow.rs`), the
    //! mincut value transitively inherits that cross-validation.

    use super::*;
    use crate::core::Graph;
    use proptest::prelude::*;

    fn xorshift(mut r: u64) -> u64 {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        r
    }

    fn build_random(seed: u64, n: u32, m_max: u32, directed: bool) -> (Graph, Vec<f64>) {
        let mut g = Graph::new(n, directed).expect("graph");
        let mut state = seed | 1;
        let mut caps: Vec<f64> = Vec::new();
        for _ in 0..m_max {
            state = xorshift(state);
            let u = (state % u64::from(n)) as u32;
            state = xorshift(state);
            let v = (state % u64::from(n)) as u32;
            if u == v {
                continue;
            }
            state = xorshift(state);
            let cap = f64::from((state % 16) as u32) + 1.0;
            g.add_edge(u, v).expect("edge");
            caps.push(cap);
        }
        (g, caps)
    }

    proptest! {
        #[test]
        fn mincut_equals_maxflow(
            seed in any::<u64>(),
            n in 2u32..8,
            m in 1u32..16,
            directed in any::<bool>(),
        ) {
            let (g, caps) = build_random(seed, n, m, directed);
            let s = (seed % u64::from(n)) as u32;
            let t_raw = xorshift(seed) % u64::from(n);
            let t = if t_raw as u32 == s { (s + 1) % n } else { t_raw as u32 };
            prop_assume!(s != t);

            let flow = max_flow_value(&g, s, t, Some(&caps)).expect("flow");
            let cut = st_mincut_value(&g, s, t, Some(&caps)).expect("cut");
            let scale = flow.abs().max(cut.abs()).max(1.0);
            prop_assert!(
                (flow - cut).abs() <= 1e-12_f64 * scale,
                "duality violated: flow {flow} cut {cut} (n={n}, m={m}, directed={directed}, seed={seed})"
            );
        }
    }
}
