//! `mincut_value` (ALGO-FL-017) — global minimum-cut value: the
//! weighted generalisation of [`super::edge_connectivity`].
//!
//! Counterpart of `igraph_mincut_value` in
//! `references/igraph/src/flow/flow.c:1692`. Equivalent to
//! python-igraph's `Graph.mincut_value()` and R-igraph's `min_cut()`
//! (called with no `source`/`target`).
//!
//! ## Algorithm
//!
//! Definition: the minimum total capacity of an edge set whose
//! removal makes the graph not strongly connected (for undirected
//! input, strong and weak connectedness coincide). With unit
//! capacities this collapses to the edge connectivity from FL-016.
//!
//! Same fixed-vertex strategy as FL-016: pick vertex `0`, and for
//! every other vertex `v` compute the maximum flow `0 → v`
//! (undirected) or both `0 → v` and `v → 0` (directed). The minimum
//! over all such pairs equals the global minimum cut, because every
//! global min-cut separates `0` from at least one vertex on the
//! other side.
//!
//! Unlike igraph C, this port does **not** branch on directedness:
//! the same fixed-vertex iteration is run for both undirected and
//! directed graphs. igraph's undirected branch is a
//! Stoer–Wagner-style algorithm with better asymptotics on dense
//! weighted inputs; porting it is left for a future AWU.
//!
//! ## Corner cases
//!
//! * `vcount ≤ 1` → `f64::INFINITY`. Mirrors `IGRAPH_INFINITY` init
//!   at `flow.c:1699` — there are no pairs to separate, so the
//!   minimum over an empty set is `+∞`.
//! * Disconnected graph (some pair has no `0 → v` path) → `0.0`.
//!
//! ## Complexity
//!
//! `V - 1` (undirected) or `2 (V - 1)` (directed) calls into FL-002
//! `max_flow_value`. Each call is `O(V·E²)` on Dinic (the FL-002
//! backend), so the total is `O(V²·E²)` worst case. The igraph C
//! docstring reports `O(log V · V²)` for the (not-ported)
//! Stoer-Wagner branch and `O(V⁴)` for the directed branch.

use crate::core::{Graph, IgraphError, IgraphResult};

use super::max_flow::max_flow_value;

/// Global minimum-cut value of a (possibly weighted) graph.
///
/// Returns the minimum total capacity of an edge set whose removal
/// disconnects some pair of distinct vertices. For undirected input,
/// equivalent to the global min-cut over weak components; for
/// directed input, over strongly connected components.
///
/// Mirrors `igraph_mincut_value`
/// (`references/igraph/src/flow/flow.c:1692`).
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `capacity` — optional per-edge capacity vector. `None` means
///   every edge has unit capacity (in which case the result equals
///   `edge_connectivity(graph, true) as f64`). When `Some(c)`,
///   `c.len()` must equal `graph.ecount()` and each entry must be a
///   finite non-negative number — these checks are delegated to
///   [`max_flow_value`].
///
/// # Returns
///
/// * `f64::INFINITY` if `vcount() ≤ 1` (no pair to separate).
/// * `0.0` if the graph is not strongly connected (directed) or not
///   connected (undirected).
/// * Otherwise the minimum value of `max_flow_value(graph, 0, v,
///   capacity)` (plus `max_flow_value(graph, v, 0, capacity)` for
///   directed) over `v ∈ 1..vcount()`.
///
/// # Errors
///
/// Propagates errors from [`max_flow_value`] — chiefly
/// [`IgraphError::InvalidArgument`] for bad capacity input (length
/// mismatch, NaN, negative).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, mincut_value};
///
/// // Undirected ring on 5 vertices, unit caps — global min cut = 2.
/// let mut g = Graph::new(5, false).unwrap();
/// for (u, v) in [(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)] {
///     g.add_edge(u, v).unwrap();
/// }
/// assert!((mincut_value(&g, None).unwrap() - 2.0).abs() < 1e-12);
///
/// // Same ring, weighted: each edge weight 0.5 → min cut = 1.0.
/// let caps = vec![0.5_f64; 5];
/// let mc = mincut_value(&g, Some(&caps)).unwrap();
/// assert!((mc - 1.0).abs() < 1e-12);
/// ```
pub fn mincut_value(graph: &Graph, capacity: Option<&[f64]>) -> IgraphResult<f64> {
    let n = graph.vcount();

    // Mirrors `IGRAPH_INFINITY` init at flow.c:1699 — for n ≤ 1 the
    // minimum over an empty pair-set is +∞. Validate capacity length
    // up front so callers always get the same error for n=0 and n=2.
    if let Some(c) = capacity {
        let m = graph.ecount();
        if c.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "capacity length {} does not match edge count {}",
                c.len(),
                m
            )));
        }
        for (i, &v) in c.iter().enumerate() {
            if !v.is_finite() || v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "capacity[{i}] = {v} is not a finite non-negative number"
                )));
            }
        }
    }
    if n <= 1 {
        return Ok(f64::INFINITY);
    }

    // Fixed-vertex iteration — see module doc and flow.c:1706-1723.
    let directed = graph.is_directed();
    let mut min_cut = f64::INFINITY;
    for v in 1..n {
        let f = max_flow_value(graph, 0, v, capacity)?;
        if f < min_cut {
            min_cut = f;
            if min_cut == 0.0 {
                return Ok(0.0);
            }
        }
        if directed {
            let f2 = max_flow_value(graph, v, 0, capacity)?;
            if f2 < min_cut {
                min_cut = f2;
                if min_cut == 0.0 {
                    return Ok(0.0);
                }
            }
        }
    }

    Ok(min_cut)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-12;

    fn approx_eq(a: f64, b: f64) -> bool {
        if a.is_infinite() && b.is_infinite() {
            a.is_sign_positive() == b.is_sign_positive()
        } else {
            (a - b).abs() < TOL
        }
    }

    fn ring_n(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..n {
            let j = (i + 1) % n;
            g.add_edge(i, j).expect("edge");
        }
        g
    }

    fn path_n(n: u32, directed: bool) -> Graph {
        let mut g = Graph::new(n, directed).expect("graph");
        for i in 0..(n - 1) {
            g.add_edge(i, i + 1).expect("edge");
        }
        g
    }

    fn complete_undirected(n: u32) -> Graph {
        let mut g = Graph::new(n, false).expect("graph");
        for i in 0..n {
            for j in (i + 1)..n {
                g.add_edge(i, j).expect("edge");
            }
        }
        g
    }

    fn complete_directed_mutual(n: u32) -> Graph {
        let mut g = Graph::new(n, true).expect("graph");
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    g.add_edge(i, j).expect("edge");
                }
            }
        }
        g
    }

    // --- IGRAPH_INFINITY init parity (flow.c:1699) ---

    #[test]
    fn empty_graph_returns_infinity() {
        let g = Graph::new(0, false).expect("graph");
        let mc = mincut_value(&g, None).expect("mc");
        assert!(mc.is_infinite() && mc.is_sign_positive());
    }

    #[test]
    fn single_vertex_returns_infinity() {
        let g = Graph::new(1, false).expect("graph");
        assert!(mincut_value(&g, None).expect("mc").is_infinite());
        let g2 = Graph::new(1, true).expect("graph");
        assert!(mincut_value(&g2, None).expect("mc").is_infinite());
    }

    // --- Disconnected ---

    #[test]
    fn two_disconnected_vertices_return_zero() {
        let g = Graph::new(2, false).expect("graph");
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 0.0));
    }

    #[test]
    fn two_isolated_edges_return_zero() {
        let mut g = Graph::new(4, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(2, 3).expect("edge");
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 0.0));
    }

    // --- Unit-capacity parity with edge_connectivity (FL-016) ---

    #[test]
    fn unit_caps_ring_5v_undirected() {
        let g = ring_n(5, false);
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 2.0));
    }

    #[test]
    fn unit_caps_path_5v_undirected() {
        let g = path_n(5, false);
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 1.0));
    }

    #[test]
    fn unit_caps_k5_undirected() {
        let g = complete_undirected(5);
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 4.0));
    }

    #[test]
    fn unit_caps_directed_3cycle() {
        let g = ring_n(3, true);
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 1.0));
    }

    #[test]
    fn unit_caps_complete_directed_mutual_4v() {
        let g = complete_directed_mutual(4);
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 3.0));
    }

    // --- Weighted fixtures ---

    #[test]
    fn weighted_k2_returns_capacity() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let caps = vec![7.5];
        assert!(approx_eq(mincut_value(&g, Some(&caps)).expect("mc"), 7.5));
    }

    #[test]
    fn weighted_ring_5v_undirected_min_edge() {
        // Bridge edge with weight 0.25; rest 10.0. The cheapest
        // 2-edge cut is the bridge + its predecessor: 0.25 + 10 = 10.25,
        // which is the minimum (any cut without the bridge requires
        // weight ≥ 20).
        let g = ring_n(5, false);
        let caps = vec![0.25, 10.0, 10.0, 10.0, 10.0];
        let mc = mincut_value(&g, Some(&caps)).expect("mc");
        assert!(approx_eq(mc, 10.25), "got {mc}, want 10.25");
    }

    #[test]
    fn weighted_path_directed_zero_capacity_returns_zero() {
        // Path with the first capacity zero — flow 0 → 4 = 0, so
        // min cut = 0.
        let g = path_n(5, true);
        let caps = vec![0.0, 1.0, 1.0, 1.0];
        assert!(approx_eq(mincut_value(&g, Some(&caps)).expect("mc"), 0.0));
    }

    #[test]
    fn weighted_directed_3cycle_min_arc() {
        // Directed 3-cycle 0→1→2→0 with caps [3, 1, 2]. The min cut
        // 0→2 path-wise: bottleneck arc 1→2 weight 1. Also need to
        // check v→0 directions: 1→0 not present; 2→0 weight 2.
        // min_{v} min(maxflow(0→v), maxflow(v→0)) = min(3, 1, 2, 2) = 1.
        let g = ring_n(3, true);
        let caps = vec![3.0, 1.0, 2.0];
        assert!(approx_eq(mincut_value(&g, Some(&caps)).expect("mc"), 1.0));
    }

    // --- Multigraph: parallel edges raise the min cut ---

    #[test]
    fn multigraph_two_parallel_edges_returns_two() {
        let mut g = Graph::new(2, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        assert!(approx_eq(mincut_value(&g, None).expect("mc"), 2.0));
    }

    // --- Capacity = None must match capacity = Some(all ones) ---

    #[test]
    fn none_matches_unit_caps() {
        let fixtures: Vec<Graph> = vec![
            ring_n(6, false),
            ring_n(6, true),
            path_n(5, false),
            complete_undirected(4),
            complete_directed_mutual(4),
        ];
        for g in fixtures {
            let caps = vec![1.0_f64; g.ecount()];
            let a = mincut_value(&g, None).expect("mc");
            let b = mincut_value(&g, Some(&caps)).expect("mc");
            assert!(
                approx_eq(a, b),
                "None={a} != Some(1.0..)={b} (n={}, dir={})",
                g.vcount(),
                g.is_directed()
            );
        }
    }

    // --- Error paths ---

    #[test]
    fn capacity_wrong_length_errors() {
        let g = ring_n(4, false);
        let caps = vec![1.0_f64; 99];
        let err = mincut_value(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn capacity_negative_errors() {
        let g = ring_n(4, false);
        let mut caps = vec![1.0_f64; g.ecount()];
        caps[1] = -0.5;
        let err = mincut_value(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn capacity_nan_errors() {
        let g = ring_n(4, false);
        let mut caps = vec![1.0_f64; g.ecount()];
        caps[0] = f64::NAN;
        let err = mincut_value(&g, Some(&caps)).expect_err("must err");
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    //! Proptest invariants:
    //! * `mincut_value >= 0` for any valid input with `vcount ≥ 2`.
    //! * `mincut_value` is finite for `vcount ≥ 2`.
    //! * `mincut_value(g, None) == mincut_value(g, Some(unit caps))`
    //!   to within tolerance.
    //! * On unit capacities, `mincut_value == edge_connectivity` as
    //!   `f64`.

    use super::*;
    use crate::algorithms::flow::edge_connectivity::edge_connectivity;
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
        fn nonneg_and_finite(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..10,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let mc = mincut_value(&g, None).expect("mc");
            prop_assert!(mc.is_finite(), "expected finite mc, got {mc}");
            prop_assert!(mc >= 0.0, "expected mc >= 0, got {mc}");
        }

        #[test]
        fn none_matches_unit_caps(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..10,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let caps = vec![1.0_f64; g.ecount()];
            let a = mincut_value(&g, None).expect("mc");
            let b = mincut_value(&g, Some(&caps)).expect("mc");
            prop_assert!((a - b).abs() < 1e-12,
                "None={a} != Some(1.0..)={b} (n={n}, m={m}, dir={directed}, seed={seed})");
        }

        #[test]
        fn unit_caps_match_edge_connectivity(
            seed in any::<u64>(),
            n in 2u32..6,
            m in 0u32..10,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let mc = mincut_value(&g, None).expect("mc");
            let ec = edge_connectivity(&g, true).expect("ec") as f64;
            prop_assert!((mc - ec).abs() < 1e-12,
                "mincut={mc} != edge_conn={ec} (n={n}, m={m}, dir={directed}, seed={seed})");
        }

        #[test]
        fn doubling_capacities_doubles_mincut(
            seed in any::<u64>(),
            n in 2u32..5,
            m in 1u32..8,
            directed in any::<bool>(),
        ) {
            let g = build_random(seed, n, m, directed);
            let caps1 = vec![1.0_f64; g.ecount()];
            let caps2 = vec![2.0_f64; g.ecount()];
            let mc1 = mincut_value(&g, Some(&caps1)).expect("mc1");
            let mc2 = mincut_value(&g, Some(&caps2)).expect("mc2");
            prop_assert!((mc2 - 2.0 * mc1).abs() < 1e-12,
                "doubling caps should double mc: mc1={mc1}, mc2={mc2}");
        }
    }
}
