//! `st_mincut_value` (ALGO-FL-010) — scalar s-t minimum-cut value.
//! `st_mincut`       (ALGO-FL-018) — full s-t minimum-cut partition.
//!
//! Counterpart of `igraph_st_mincut_value` / `igraph_st_mincut` in
//! `references/igraph/src/flow/flow.c` (lines 1127-1138 and
//! 1140-1186 respectively). Both C entries are thin wrappers around
//! `igraph_maxflow` — the value variant requests only the scalar flow,
//! while the partition variant additionally requests the cut edge list
//! and the source-side / sink-side partitions. Correctness rests on
//! the max-flow / min-cut theorem (Ford-Fulkerson, 1956): after the
//! flow saturates, the set of vertices reachable from the source in
//! the residual network is *exactly* the source-side of a minimum
//! `s-t` cut, and the edges crossing that frontier are saturated in
//! the original flow.
//!
//! We follow the same delegation pattern. [`st_mincut_value`] is a
//! thin wrapper over [`super::max_flow::max_flow_value`].
//! [`st_mincut`] uses the crate-private
//! [`super::max_flow::max_flow_with_residual`] entry point (a sibling
//! of `max_flow_value` that also returns the post-augmentation
//! residual network) and then does one BFS from the source in the
//! residual to materialise the partition + cut edge list. All input
//! validation (vertex-id bounds, `source != target`, capacity length,
//! capacity finiteness / non-negativity) is delegated to the
//! max-flow layer, so the error contract of both functions matches
//! that of `max_flow_value`.

// Vertex-id casts in the proptest helper compute `state % u64::from(n)`
// before truncating to `u32` — the result is always `< n <= u32::MAX`,
// so no truncation occurs at runtime. Same dispensation as
// `max_flow.rs`; keep both modules in sync.
#![allow(clippy::cast_possible_truncation)]

use crate::core::{Graph, IgraphResult, VertexId};

use super::max_flow::{max_flow_value, max_flow_with_residual};

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

/// Full s-t minimum-cut: scalar value, the cut edge list, and the
/// source-side / sink-side vertex partitions.
///
/// Counterpart of `igraph_st_mincut` in
/// `references/igraph/src/flow/flow.c:1140`. The C entry is a
/// 47-line wrapper around `igraph_maxflow` that requests the optional
/// `cut`, `partition`, and `partition2` outputs alongside the flow
/// value. Our implementation:
///
/// 1. Calls the crate-private `max_flow_with_residual` backend (the
///    shared Dinic entry point that also powers [`max_flow_value`]) to
///    obtain both the flow value and the final residual network.
/// 2. Runs one BFS from `source` in the residual graph (following only
///    arcs with strictly positive residual capacity). The set of
///    reachable vertices is precisely the source-side `S` of a
///    minimum `s-t` cut (max-flow / min-cut duality, Ford-Fulkerson
///    1956). The complement is `partition2`.
/// 3. Walks the original edge list once: an original edge `(u, v)` is
///    in the cut iff it crosses the partition boundary — for directed
///    input the cut edge points `S → V \ S`; for undirected input,
///    either endpoint side suffices.
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
/// An [`StMincut`] whose
///
/// * `value` equals the maximum `source → target` flow (matches
///   [`max_flow_value`] / [`st_mincut_value`] within `1e-12`).
/// * `cut` lists original edge ids whose removal disconnects `source`
///   from `target`. Sum of `capacity[cut[i]]` (or `cut.len()` when
///   `capacity` is `None`) equals `value` within tolerance.
/// * `partition` is the source-side `S` (always contains `source`,
///   never contains `target` unless the graph is so degenerate that
///   they're already separated by the empty cut).
/// * `partition2` is `V \ S` (always contains `target`).
///
/// Both partitions list vertex ids in ascending order. `partition` and
/// `partition2` together cover every vertex exactly once.
///
/// # Errors
///
/// Same as [`max_flow_value`]:
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
/// use rust_igraph::{Graph, st_mincut};
///
/// // 0 → 1 (cap 5) → 2 (cap 2) → 3 (cap 7).
/// // Unique bottleneck arc (1, 2): cut = [1], partition = [0, 1].
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cut = st_mincut(&g, 0, 3, Some(&[5.0, 2.0, 7.0])).unwrap();
/// assert!((cut.value - 2.0).abs() < 1e-12);
/// assert_eq!(cut.cut, vec![1]);
/// assert_eq!(cut.partition, vec![0, 1]);
/// assert_eq!(cut.partition2, vec![2, 3]);
/// ```
pub fn st_mincut(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    capacity: Option<&[f64]>,
) -> IgraphResult<StMincut> {
    let (value, net) = max_flow_with_residual(graph, source, target, capacity)?;

    // BFS in the residual graph (only arcs with strictly positive
    // residual capacity). The reachable set is exactly the source-side
    // of a minimum s-t cut by max-flow / min-cut duality. We compare
    // against zero (not against a tolerance) because Dinic only ever
    // subtracts pushed flow from a residual that already had at least
    // that much capacity, so a saturated arc lands at *exactly* 0.0 and
    // any positive value means a legitimate residual path; using a
    // tolerance here would incorrectly include arcs that were just
    // saturated to within fp noise. See `references/igraph/src/flow/
    // flow.c:1015` for the equivalent C check.
    let n = net.n;
    let mut in_source = vec![false; n];
    in_source[source as usize] = true;
    let mut queue: Vec<u32> = Vec::with_capacity(n);
    queue.push(source);
    let mut head_ptr = 0_usize;
    while head_ptr < queue.len() {
        let v = queue[head_ptr] as usize;
        head_ptr += 1;
        for &arc in &net.arcs_out[v] {
            let arc_us = arc as usize;
            if net.cap[arc_us] <= 0.0 {
                continue;
            }
            let w = net.head[arc_us] as usize;
            if !in_source[w] {
                in_source[w] = true;
                queue.push(w as u32);
            }
        }
    }

    // Materialise the two partitions in ascending vertex-id order.
    let mut partition: Vec<u32> = Vec::with_capacity(queue.len());
    let mut partition2: Vec<u32> = Vec::with_capacity(n.saturating_sub(queue.len()));
    for (v, &is_src) in in_source.iter().enumerate() {
        if is_src {
            partition.push(v as u32);
        } else {
            partition2.push(v as u32);
        }
    }

    // Walk original edges once. An edge belongs to the cut iff it
    // crosses the partition boundary. For directed input the cut must
    // point S → V\S (the reverse direction lies in the opposite cut and
    // is not saturated by this flow). For undirected input either
    // crossing is legitimate — the underlying residual carries flow in
    // whichever direction the network demanded.
    let m = graph.ecount();
    let directed = graph.is_directed();
    let edge_count =
        u32::try_from(m).map_err(|_| crate::core::IgraphError::Internal("ecount overflows u32"))?;
    let mut cut: Vec<u32> = Vec::new();
    for eid in 0..edge_count {
        let (u, v) = graph.edge(eid)?;
        let u_in = in_source[u as usize];
        let v_in = in_source[v as usize];
        let crosses = if directed {
            u_in && !v_in
        } else {
            u_in != v_in
        };
        if crosses {
            cut.push(eid);
        }
    }

    Ok(StMincut {
        value,
        cut,
        partition,
        partition2,
    })
}

/// Output of [`st_mincut`]: scalar value, cut edge ids, and the
/// source-side / sink-side vertex partitions.
///
/// Mirrors the four output parameters of `igraph_st_mincut` in
/// `references/igraph/src/flow/flow.c:1140` (`value`, `cut`,
/// `partition`, `partition2`) — bundled into one return type for
/// ergonomic Rust call sites.
#[derive(Debug, Clone)]
pub struct StMincut {
    /// Capacity of the minimum `source → target` cut. Equals the
    /// scalar value returned by [`st_mincut_value`] /
    /// [`max_flow_value`] within `1e-12`.
    pub value: f64,
    /// Edge ids (in `graph`'s ecount-order) whose removal disconnects
    /// `source` from `target`. Sum of capacities equals `value`.
    pub cut: Vec<u32>,
    /// Source-side partition `S` (vertices reachable from `source` in
    /// the residual network after saturation). Always contains
    /// `source`. Sorted ascending.
    pub partition: Vec<u32>,
    /// Sink-side partition `V \ S`. Always contains `target` (unless
    /// `target` is itself unreachable from `source` even before any
    /// flow is pushed, in which case the empty cut suffices). Sorted
    /// ascending.
    pub partition2: Vec<u32>,
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

    // ----- FL-018 (st_mincut, the partition variant) -----------------

    fn sum_cut_caps(cut: &[u32], caps: Option<&[f64]>) -> f64 {
        if let Some(c) = caps {
            cut.iter().map(|&e| c[e as usize]).sum()
        } else {
            // cut.len() is bounded by ecount(); test fixtures keep it
            // well under 2^53 so the f64 round-trip is exact.
            #[allow(clippy::cast_precision_loss)]
            let n = cut.len() as f64;
            n
        }
    }

    fn assert_partition_well_formed(n: u32, part: &[u32], part2: &[u32], src: u32, tgt: u32) {
        // Two partitions cover every vertex exactly once; sorted ascending.
        assert!(part.windows(2).all(|w| w[0] < w[1]), "partition not sorted");
        assert!(
            part2.windows(2).all(|w| w[0] < w[1]),
            "partition2 not sorted"
        );
        assert_eq!(
            part.len() + part2.len(),
            n as usize,
            "partitions must cover V"
        );
        let mut seen = vec![false; n as usize];
        for &v in part.iter().chain(part2.iter()) {
            assert!(!seen[v as usize], "vertex {v} appears in both partitions");
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&b| b), "some vertex missing");
        assert!(
            part.contains(&src),
            "source {src} not on source side {part:?}"
        );
        // The sink is on the opposite side as long as the flow is non-trivial.
        // For the empty / disconnected case the sink may also live in
        // partition2 trivially (the BFS never reached it).
        assert!(
            part2.contains(&tgt),
            "target {tgt} not on sink side {part2:?}"
        );
    }

    #[test]
    fn st_mincut_bottleneck_directed_partition() {
        // 0 → 1 (cap 5) → 2 (cap 2) → 3 (cap 7).
        // Unique bottleneck arc id 1; partition = {0,1}, partition2 = {2,3}.
        let mut g = Graph::new(4, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 2).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let caps = [5.0, 2.0, 7.0];
        let r = st_mincut(&g, 0, 3, Some(&caps)).expect("st_mincut");
        assert!(approx(r.value, 2.0));
        assert_eq!(r.cut, vec![1]);
        assert_eq!(r.partition, vec![0, 1]);
        assert_eq!(r.partition2, vec![2, 3]);
        assert!(approx(sum_cut_caps(&r.cut, Some(&caps)), r.value));
        assert_partition_well_formed(4, &r.partition, &r.partition2, 0, 3);
    }

    #[test]
    fn st_mincut_two_parallel_paths_partition() {
        // 0→1→3 and 0→2→3 unit caps. The Dinic BFS saturates both
        // outgoing arcs from 0 first, so the only residual frontier
        // sits at the source — partition = {0}, cut = first arcs of
        // each path.
        let mut g = Graph::new(4, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(1, 3).expect("edge");
        g.add_edge(0, 2).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let r = st_mincut(&g, 0, 3, None).expect("st_mincut");
        assert!(approx(r.value, 2.0));
        assert!(approx(sum_cut_caps(&r.cut, None), r.value));
        assert_partition_well_formed(4, &r.partition, &r.partition2, 0, 3);
        // value equals the value-only oracle.
        let v = st_mincut_value(&g, 0, 3, None).expect("value");
        assert!(approx(v, r.value));
    }

    #[test]
    fn st_mincut_isolated_endpoints() {
        // No path source→target → empty cut, partition = {source}.
        let g = Graph::new(4, true).expect("graph");
        let r = st_mincut(&g, 0, 3, None).expect("st_mincut");
        assert!(approx(r.value, 0.0));
        assert!(r.cut.is_empty(), "cut must be empty when value = 0");
        assert_eq!(r.partition, vec![0]);
        assert_eq!(r.partition2, vec![1, 2, 3]);
    }

    #[test]
    fn st_mincut_single_edge() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        let r = st_mincut(&g, 0, 1, None).expect("st_mincut");
        assert!(approx(r.value, 1.0));
        assert_eq!(r.cut, vec![0]);
        assert_eq!(r.partition, vec![0]);
        assert_eq!(r.partition2, vec![1]);
    }

    #[test]
    fn st_mincut_undirected_partition_crossings() {
        // 4v undirected: edges (0,1), (0,2), (1,3), (2,3); caps 1,1,1,1.
        // Two unit-capacity paths from 0 to 3, so cut value = 2.
        let mut g = Graph::new(4, false).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 2).expect("edge");
        g.add_edge(1, 3).expect("edge");
        g.add_edge(2, 3).expect("edge");
        let caps = [1.0, 1.0, 1.0, 1.0];
        let r = st_mincut(&g, 0, 3, Some(&caps)).expect("st_mincut");
        assert!(approx(r.value, 2.0));
        assert!(approx(sum_cut_caps(&r.cut, Some(&caps)), 2.0));
        assert_partition_well_formed(4, &r.partition, &r.partition2, 0, 3);
        // Each cut edge has one endpoint on each side.
        for &eid in &r.cut {
            let (u, v) = g.edge(eid).expect("edge");
            let u_in = r.partition.contains(&u);
            let v_in = r.partition.contains(&v);
            assert_ne!(
                u_in, v_in,
                "cut edge {eid} ({u}-{v}) does not cross the partition"
            );
        }
    }

    #[test]
    fn st_mincut_multigraph_parallel_arcs() {
        // Two parallel arcs 0→1: cut must list both edge ids.
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge");
        let r = st_mincut(&g, 0, 1, None).expect("st_mincut");
        assert!(approx(r.value, 2.0));
        let mut cut_sorted = r.cut.clone();
        cut_sorted.sort_unstable();
        assert_eq!(cut_sorted, vec![0, 1]);
        assert_eq!(r.partition, vec![0]);
        assert_eq!(r.partition2, vec![1]);
    }

    #[test]
    fn st_mincut_classic_textbook_partition() {
        // CLRS 26.1-1: max flow = min cut = 23. The unique min cut
        // here isolates source 0 with cap 16 + 13 = 29 on its outgoing
        // arcs — that's NOT the min cut. The min cut sits at the
        // {0,1,2,4} | {3,5} boundary (12 + 7 + 4 = 23). We don't pin
        // the exact edge id list (multiple min cuts may exist with the
        // same value), only invariants.
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
        let r = st_mincut(&g, 0, 5, Some(&caps)).expect("st_mincut");
        assert!(approx(r.value, 23.0));
        assert!(approx(sum_cut_caps(&r.cut, Some(&caps)), 23.0));
        assert_partition_well_formed(6, &r.partition, &r.partition2, 0, 5);
        // All cut arcs go S → V\S (directed).
        for &eid in &r.cut {
            let (u, v) = g.edge(eid).expect("edge");
            assert!(
                r.partition.contains(&u) && r.partition2.contains(&v),
                "directed cut arc {eid} ({u}→{v}) must point S→V\\S"
            );
        }
    }

    #[test]
    fn st_mincut_capacity_validation_propagates() {
        let mut g = Graph::new(2, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        // Mismatched capacity length → InvalidArgument.
        let err = st_mincut(&g, 0, 1, Some(&[1.0, 2.0])).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
        // Negative capacity → InvalidArgument.
        let err = st_mincut(&g, 0, 1, Some(&[-1.0])).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // source == target → InvalidArgument (delegated).
        let err = st_mincut(&g, 0, 0, None).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // source out of range.
        let err = st_mincut(&g, 5, 1, None).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 2 }));
        // target out of range.
        let err = st_mincut(&g, 0, 5, None).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 5, n: 2 }));
    }

    #[test]
    fn st_mincut_value_matches_max_flow_value() {
        // Triangulate against both peers on the same non-trivial graph.
        let mut g = Graph::new(5, true).expect("graph");
        for (s, t) in [(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (1, 4)] {
            g.add_edge(s, t).expect("edge");
        }
        let caps = [3.0, 5.0, 2.0, 4.0, 6.0, 1.0];
        let flow = max_flow_value(&g, 0, 4, Some(&caps)).expect("flow");
        let val = st_mincut_value(&g, 0, 4, Some(&caps)).expect("value");
        let part = st_mincut(&g, 0, 4, Some(&caps)).expect("partition");
        assert!(approx(flow, val));
        assert!(approx(flow, part.value));
        assert!(approx(sum_cut_caps(&part.cut, Some(&caps)), flow));
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

        // FL-018: st_mincut partition invariants.
        //   1. value equals st_mincut_value;
        //   2. partition ∪ partition2 = V, disjoint, source ∈ partition,
        //      target ∈ partition2;
        //   3. sum of cut capacities equals the flow value;
        //   4. removing the cut edges disconnects source from target in
        //      the underlying (directed) reachability — verified by a
        //      plain BFS that respects the directed-vs-undirected arc
        //      orientation.
        #[test]
        fn st_mincut_partition_invariants(
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

            let result = st_mincut(&g, s, t, Some(&caps)).expect("st_mincut");
            let value = st_mincut_value(&g, s, t, Some(&caps)).expect("value");

            // Invariant 1: value matches the scalar oracle.
            let scale = value.abs().max(result.value.abs()).max(1.0);
            prop_assert!(
                (value - result.value).abs() <= 1e-12_f64 * scale,
                "values disagree: scalar={value} partition={}", result.value
            );

            // Invariant 2: partition / partition2 well-formedness.
            prop_assert_eq!(
                result.partition.len() + result.partition2.len(),
                n as usize,
                "partitions do not cover V"
            );
            let mut seen = vec![false; n as usize];
            for &v in result.partition.iter().chain(result.partition2.iter()) {
                prop_assert!(!seen[v as usize], "vertex {} duplicated", v);
                seen[v as usize] = true;
            }
            prop_assert!(seen.iter().all(|&b| b), "some vertex unaccounted for");
            prop_assert!(result.partition.contains(&s), "source missing from partition");
            prop_assert!(result.partition2.contains(&t), "target missing from partition2");
            prop_assert!(
                result.partition.windows(2).all(|w| w[0] < w[1]),
                "partition not sorted"
            );
            prop_assert!(
                result.partition2.windows(2).all(|w| w[0] < w[1]),
                "partition2 not sorted"
            );

            // Invariant 3: cut capacity sum = value.
            let mut sum_caps = 0.0_f64;
            for &eid in &result.cut {
                sum_caps += caps[eid as usize];
            }
            let scale = value.abs().max(sum_caps.abs()).max(1.0);
            prop_assert!(
                (sum_caps - value).abs() <= 1e-9_f64 * scale,
                "cut capacity {sum_caps} does not match value {value}"
            );

            // Invariant 4: removing cut edges disconnects s from t.
            // Build adjacency over the surviving edges and BFS.
            let cut_set: std::collections::HashSet<u32> = result.cut.iter().copied().collect();
            let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n as usize];
            for eid in 0..g.ecount() as u32 {
                if cut_set.contains(&eid) {
                    continue;
                }
                let (u, v) = g.edge(eid).expect("edge");
                adj[u as usize].push(v);
                if !directed {
                    adj[v as usize].push(u);
                }
            }
            let mut visited = vec![false; n as usize];
            visited[s as usize] = true;
            let mut q = vec![s];
            let mut hp = 0;
            while hp < q.len() {
                let v = q[hp] as usize;
                hp += 1;
                for &w in &adj[v] {
                    if !visited[w as usize] {
                        visited[w as usize] = true;
                        q.push(w);
                    }
                }
            }
            prop_assert!(
                !visited[t as usize],
                "removing cut did not disconnect {} from {} (graph: n={}, m={}, directed={}, seed={}, cut={:?})",
                s, t, n, m, directed, seed, result.cut
            );
        }
    }
}
