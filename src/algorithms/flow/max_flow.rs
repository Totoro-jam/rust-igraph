//! `max_flow_value` (ALGO-FL-002) — Dinic's algorithm.
//!
//! Counterpart of `igraph_maxflow_value` in
//! `references/igraph/src/flow/flow.c` (`igraph_maxflow_value` is a
//! thin wrapper around `igraph_maxflow`; the C implementation is
//! Goldberg-Tarjan push-relabel, ~600 LOC). We implement Dinic's
//! algorithm here instead — O(V²·E) worst-case, but with much smaller
//! constants on the sparse networks rust-igraph targets, and a far
//! simpler control-flow that fits the self-roll preference of the
//! project. The scalar max-flow value is unique (Ford-Fulkerson
//! correctness + max-flow / min-cut duality), so cross-backend
//! conformance against the C reference is exact on integer capacities
//! and within 1e-9 on `f64`.
//!
//! ## Pre-conditions
//!
//! - `source < vcount()` and `target < vcount()`, else
//!   [`IgraphError::VertexOutOfRange`].
//! - `source != target`, else [`IgraphError::InvalidArgument`] (matches
//!   igraph C's `IGRAPH_EINVAL`).
//! - When `capacity` is supplied, its length must equal `ecount()`,
//!   else [`IgraphError::InvalidArgument`]. Each capacity must be
//!   finite and non-negative. Negative or non-finite capacities raise
//!   [`IgraphError::InvalidArgument`].
//! - When `capacity` is `None`, each edge contributes unit capacity
//!   (the igraph C default when the capacity vector is `NULL`).
//!
//! ## Undirected handling
//!
//! igraph C converts each undirected edge `(i, j)` of capacity `c`
//! into two directed arcs `i → j` and `j → i`, both with capacity
//! `c`, before running the directed algorithm. We follow the same
//! pattern: we always materialise a directed residual network whose
//! arc count is `2 · ecount()` (forward) plus `2 · ecount()`
//! (reverse residual arcs). For directed input, each input edge
//! contributes one forward arc with capacity `c[e]` and one zero-cap
//! reverse arc; for undirected input, each input edge contributes two
//! forward arcs (each with capacity `c[e]`) and two reverse arcs (the
//! "reverse" arc of the second forward is the first forward, so the
//! residual structure naturally encodes the undirected semantics).

// Arc indices fit in u32 by the residual-network construction (each
// input edge contributes two arcs, so the arc count is `2 * ecount()`
// and `ecount() <= u32::MAX / 2` is an invariant inherited from
// `Graph`). Vertex ids are u32 by definition. Inner-loop casts here are
// either provably bounded (2 * ecount() or 2 * ecount() + 1) or
// round-trips of values that were already u32 in storage.
#![allow(clippy::cast_possible_truncation)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Scalar maximum-flow value from `source` to `target`.
///
/// Counterpart of `igraph_maxflow_value` in
/// `references/igraph/src/flow/flow.c` (igraph C uses Goldberg-Tarjan
/// push-relabel; this implementation uses Dinic's algorithm — the
/// scalar value matches by the max-flow / min-cut uniqueness theorem).
///
/// # Arguments
///
/// * `graph` — input graph (directed or undirected).
/// * `source` — source vertex id (`0 ≤ source < vcount()`).
/// * `target` — sink vertex id (`0 ≤ target < vcount()`, `target != source`).
/// * `capacity` — optional per-edge capacity in the graph's edge-id
///   order. When `None`, each edge contributes unit capacity. When
///   `Some(c)`, `c.len()` must equal `graph.ecount()`, and every entry
///   must be finite and `≥ 0`.
///
/// # Returns
///
/// The maximum total flow value as `f64`, summing to `0.0` when source
/// and target are in disjoint connected components or when every
/// `source → target` path is blocked by zero-capacity edges.
///
/// # Errors
///
/// * [`IgraphError::VertexOutOfRange`] if `source` or `target` is
///   outside `[0, vcount())`.
/// * [`IgraphError::InvalidArgument`] if `source == target`, the
///   capacity slice length differs from `ecount()`, or any capacity is
///   negative / non-finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, max_flow_value};
///
/// // Two parallel paths of capacity 1 each → max flow = 2.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let cap = vec![1.0, 1.0, 1.0, 1.0];
/// let f = max_flow_value(&g, 0, 3, Some(&cap)).unwrap();
/// assert!((f - 2.0).abs() < 1e-12);
/// ```
pub fn max_flow_value(
    graph: &Graph,
    source: VertexId,
    target: VertexId,
    capacity: Option<&[f64]>,
) -> IgraphResult<f64> {
    let n = graph.vcount();
    if n == 0 || source >= n {
        return Err(IgraphError::VertexOutOfRange { id: source, n });
    }
    if target >= n {
        return Err(IgraphError::VertexOutOfRange { id: target, n });
    }
    if source == target {
        return Err(IgraphError::InvalidArgument(
            "source and target must be distinct".to_string(),
        ));
    }

    let m = graph.ecount();
    if let Some(c) = capacity {
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

    let net = Network::build(graph, capacity)?;
    let mut state = DinicState::new(net);
    Ok(state.run(source, target))
}

/// Residual network in flat-CSR form with paired arcs.
///
/// Arcs are stored in pairs: for each input forward arc at index `2k`,
/// its reverse residual sits at index `2k + 1` (and vice versa). The
/// XOR `idx ^ 1` recovers the paired arc.
struct Network {
    n: usize,
    /// Head (destination vertex) of each arc.
    head: Vec<u32>,
    /// Residual capacity of each arc (may be modified during the flow
    /// computation). For input forward arcs this starts at `capacity[e]`
    /// (or `1.0` if `capacity` is `None`); for the reverse residual
    /// this starts at `0.0` on directed input, or also at `capacity[e]`
    /// on undirected input.
    cap: Vec<f64>,
    /// CSR-style adjacency: `arcs_out[v]` lists every arc index whose
    /// tail is `v`. Built once at construction.
    arcs_out: Vec<Vec<u32>>,
}

impl Network {
    fn build(graph: &Graph, capacity: Option<&[f64]>) -> IgraphResult<Self> {
        let n = graph.vcount() as usize;
        let m = graph.ecount();
        let directed = graph.is_directed();

        // Two arcs per input edge in either case: directed → (fwd, rev0);
        // undirected → (fwd_u→v, fwd_v→u). The "reverse residual" arc
        // for undirected is simply the other forward arc with the same
        // capacity, since the XOR pairing matches our layout.
        let arc_count = m
            .checked_mul(2)
            .ok_or(IgraphError::Internal("arc count overflows usize"))?;

        let mut head = vec![0_u32; arc_count];
        let mut cap = vec![0.0_f64; arc_count];
        let mut arcs_out: Vec<Vec<u32>> = vec![Vec::new(); n];

        let edge_count =
            u32::try_from(m).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
        for eid in 0..edge_count {
            let (src, dst) = graph.edge(eid)?;
            let e_us = eid as usize;
            let cap_val = capacity.map_or(1.0, |c| c[e_us]);

            let fwd = 2 * e_us;
            let rev = fwd + 1;
            head[fwd] = dst;
            head[rev] = src;
            cap[fwd] = cap_val;
            cap[rev] = if directed { 0.0 } else { cap_val };
            // Both arcs are usable for traversal in either mode. For
            // directed input, `cap[rev] == 0.0` so the BFS/DFS skip it
            // until augmenting flow gives it residual capacity.
            arcs_out[src as usize].push(fwd as u32);
            arcs_out[dst as usize].push(rev as u32);
        }

        Ok(Self {
            n,
            head,
            cap,
            arcs_out,
        })
    }
}

struct DinicState {
    net: Network,
    /// BFS distance from source; `u32::MAX` means unreachable.
    level: Vec<u32>,
    /// DFS current-arc pointer into `arcs_out[v]` so we don't revisit
    /// saturated arcs within the same blocking-flow phase.
    iter: Vec<u32>,
    /// BFS queue scratch buffer.
    queue: Vec<u32>,
}

impl DinicState {
    fn new(net: Network) -> Self {
        let n = net.n;
        Self {
            net,
            level: vec![u32::MAX; n],
            iter: vec![0_u32; n],
            queue: Vec::with_capacity(n),
        }
    }

    fn run(&mut self, source: u32, target: u32) -> f64 {
        let mut total = 0.0_f64;
        let src = source as usize;
        let dst = target as usize;
        while self.bfs(src, dst) {
            for it in &mut self.iter {
                *it = 0;
            }
            loop {
                let pushed = self.dfs(src, dst, f64::INFINITY);
                if pushed <= 0.0 {
                    break;
                }
                total += pushed;
            }
        }
        total
    }

    /// BFS in the residual graph (only arcs with positive capacity).
    /// Returns true iff `target` is reachable.
    fn bfs(&mut self, source: usize, target: usize) -> bool {
        for l in &mut self.level {
            *l = u32::MAX;
        }
        self.level[source] = 0;
        self.queue.clear();
        self.queue.push(source as u32);
        let mut head_ptr = 0_usize;
        while head_ptr < self.queue.len() {
            let v = self.queue[head_ptr] as usize;
            head_ptr += 1;
            let next_level = self.level[v].saturating_add(1);
            for &a in &self.net.arcs_out[v] {
                let a_us = a as usize;
                if self.net.cap[a_us] <= 0.0 {
                    continue;
                }
                let w = self.net.head[a_us] as usize;
                if self.level[w] == u32::MAX {
                    self.level[w] = next_level;
                    self.queue.push(w as u32);
                }
            }
        }
        self.level[target] != u32::MAX
    }

    /// DFS that pushes up to `limit` of residual capacity from `v` to
    /// `target` along level-monotone arcs. Returns the actual amount
    /// pushed (0 if no augmenting path remains from `v`).
    fn dfs(&mut self, v: usize, target: usize, limit: f64) -> f64 {
        if v == target {
            return limit;
        }
        let next_level = self.level[v].saturating_add(1);
        while (self.iter[v] as usize) < self.net.arcs_out[v].len() {
            let arc_idx = self.net.arcs_out[v][self.iter[v] as usize] as usize;
            let w = self.net.head[arc_idx] as usize;
            let cap_here = self.net.cap[arc_idx];
            if cap_here > 0.0 && self.level[w] == next_level {
                let send = limit.min(cap_here);
                let pushed = self.dfs(w, target, send);
                if pushed > 0.0 {
                    self.net.cap[arc_idx] -= pushed;
                    self.net.cap[arc_idx ^ 1] += pushed;
                    return pushed;
                }
            }
            self.iter[v] += 1;
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() < tol,
            "actual = {actual}, expected = {expected}"
        );
    }

    #[test]
    fn rejects_out_of_range_source() {
        let g = Graph::with_vertices(2);
        let err = max_flow_value(&g, 5, 1, None).unwrap_err();
        match err {
            IgraphError::VertexOutOfRange { id, n } => {
                assert_eq!(id, 5);
                assert_eq!(n, 2);
            }
            _ => panic!("expected VertexOutOfRange"),
        }
    }

    #[test]
    fn rejects_out_of_range_target() {
        let g = Graph::with_vertices(2);
        let err = max_flow_value(&g, 0, 9, None).unwrap_err();
        assert!(matches!(err, IgraphError::VertexOutOfRange { id: 9, n: 2 }));
    }

    #[test]
    fn rejects_source_equals_target() {
        let g = Graph::with_vertices(2);
        let err = max_flow_value(&g, 0, 0, None).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_wrong_capacity_length() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let cap = vec![1.0, 2.0];
        let err = max_flow_value(&g, 0, 1, Some(&cap)).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_negative_capacity() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let cap = vec![-1.0];
        let err = max_flow_value(&g, 0, 1, Some(&cap)).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn isolated_source_and_target() {
        let g = Graph::with_vertices(2);
        let f = max_flow_value(&g, 0, 1, None).unwrap();
        assert_close(f, 0.0, 1e-12);
    }

    #[test]
    fn single_edge_directed_unit() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let f = max_flow_value(&g, 0, 1, None).unwrap();
        assert_close(f, 1.0, 1e-12);
    }

    #[test]
    fn single_edge_directed_wrong_direction() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let f = max_flow_value(&g, 1, 0, None).unwrap();
        assert_close(f, 0.0, 1e-12);
    }

    #[test]
    fn single_edge_undirected_unit() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let f = max_flow_value(&g, 0, 1, None).unwrap();
        assert_close(f, 1.0, 1e-12);
        let f_rev = max_flow_value(&g, 1, 0, None).unwrap();
        assert_close(f_rev, 1.0, 1e-12);
    }

    #[test]
    fn two_parallel_paths_directed() {
        // 0 → 1 → 3,  0 → 2 → 3, all unit capacity.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let f = max_flow_value(&g, 0, 3, None).unwrap();
        assert_close(f, 2.0, 1e-12);
    }

    #[test]
    fn bottleneck_directed() {
        // 0 → 1 (cap 5), 1 → 2 (cap 2), 2 → 3 (cap 5) → bottleneck = 2.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let cap = vec![5.0, 2.0, 5.0];
        let f = max_flow_value(&g, 0, 3, Some(&cap)).unwrap();
        assert_close(f, 2.0, 1e-12);
    }

    #[test]
    fn classic_max_flow_textbook() {
        // CLRS classic flow network: 6 vertices s = 0, t = 5.
        // Arcs and capacities:
        //   0→1:16  0→2:13  1→3:12  2→1:4  2→4:14  3→2:9  3→5:20  4→3:7  4→5:4
        // Max flow = 23.
        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 2).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(4, 3).unwrap();
        g.add_edge(4, 5).unwrap();
        let cap = vec![16.0, 13.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0];
        let f = max_flow_value(&g, 0, 5, Some(&cap)).unwrap();
        assert_close(f, 23.0, 1e-12);
    }

    #[test]
    fn multigraph_parallel_edges() {
        // Three parallel arcs 0→1 with capacities 1, 2, 4 → total 7.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![1.0, 2.0, 4.0];
        let f = max_flow_value(&g, 0, 1, Some(&cap)).unwrap();
        assert_close(f, 7.0, 1e-12);
    }

    #[test]
    fn self_loop_does_not_contribute() {
        // A self-loop on the source can't add flow to the sink.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![100.0, 3.0];
        let f = max_flow_value(&g, 0, 1, Some(&cap)).unwrap();
        assert_close(f, 3.0, 1e-12);
    }

    #[test]
    fn undirected_two_paths_share_capacity() {
        // Undirected edges 0-1, 1-2, all unit capacity. The two
        // forward orientations of {0,1} share a single capacity unit
        // (igraph C semantics: convert to two directed arcs of cap c
        // each, but only one unit of net flow can move from 0 to 1 at
        // a time before residuals cancel).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let f = max_flow_value(&g, 0, 2, None).unwrap();
        assert_close(f, 1.0, 1e-12);
    }

    #[test]
    fn weighted_fractional_flow() {
        // Two parallel paths with fractional capacities → max flow = 0.75.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let cap = vec![0.5, 0.5, 0.25, 0.25];
        let f = max_flow_value(&g, 0, 3, Some(&cap)).unwrap();
        assert_close(f, 0.75, 1e-12);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    /// Independent reference: plain Edmonds-Karp BFS-augmentation on
    /// the same residual structure. O(V·E²) — perfectly fine for the
    /// tiny proptest graphs but algorithmically distinct from Dinic's
    /// blocking-flow strategy, so agreement on the scalar value
    /// cross-validates both implementations.
    fn edmonds_karp(graph: &Graph, source: u32, target: u32, cap: &[f64]) -> f64 {
        let net = Network::build(graph, Some(cap)).expect("net builds");
        let n = net.n;
        let mut residual = net.cap.clone();
        let mut total = 0.0_f64;
        loop {
            let mut parent_arc = vec![u32::MAX; n];
            let mut visited = vec![false; n];
            visited[source as usize] = true;
            let mut queue = vec![source];
            let mut head = 0_usize;
            let mut found = false;
            'bfs: while head < queue.len() {
                let v = queue[head] as usize;
                head += 1;
                for &a in &net.arcs_out[v] {
                    let a_us = a as usize;
                    let w = net.head[a_us] as usize;
                    if !visited[w] && residual[a_us] > 0.0 {
                        visited[w] = true;
                        parent_arc[w] = a;
                        if w == target as usize {
                            found = true;
                            break 'bfs;
                        }
                        queue.push(w as u32);
                    }
                }
            }
            if !found {
                break;
            }
            let mut bottleneck = f64::INFINITY;
            let mut cur = target as usize;
            while cur != source as usize {
                let a = parent_arc[cur] as usize;
                if residual[a] < bottleneck {
                    bottleneck = residual[a];
                }
                cur = net.head[a ^ 1] as usize;
            }
            let mut cur = target as usize;
            while cur != source as usize {
                let a = parent_arc[cur] as usize;
                residual[a] -= bottleneck;
                residual[a ^ 1] += bottleneck;
                cur = net.head[a ^ 1] as usize;
            }
            total += bottleneck;
        }
        total
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(80))]

        #[test]
        fn matches_edmonds_karp_directed(
            n in 2u32..8,
            seed in any::<u64>(),
        ) {
            // Build a random directed graph + capacities deterministically from `seed`.
            let mut rng_state = seed | 1;
            let mut next = || {
                rng_state ^= rng_state << 13;
                rng_state ^= rng_state >> 7;
                rng_state ^= rng_state << 17;
                rng_state
            };
            let mut g = Graph::new(n, true).unwrap();
            let edge_count = next() as u32 % (n * 3 + 1);
            let mut caps = Vec::with_capacity(edge_count as usize);
            for _ in 0..edge_count {
                let u = (next() as u32) % n;
                let v = (next() as u32) % n;
                g.add_edge(u, v).unwrap();
                let c = f64::from((next() as u32 % 16) + 1);
                caps.push(c);
            }
            let source = 0;
            let target = n - 1;
            if source == target {
                return Ok(());
            }
            let dinic = max_flow_value(&g, source, target, Some(&caps)).unwrap();
            let ref_val = edmonds_karp(&g, source, target, &caps);
            prop_assert!(
                (dinic - ref_val).abs() < 1e-9,
                "dinic={dinic} ref={ref_val}"
            );
        }

        #[test]
        fn matches_edmonds_karp_undirected(
            n in 2u32..8,
            seed in any::<u64>(),
        ) {
            let mut rng_state = seed | 1;
            let mut next = || {
                rng_state ^= rng_state << 13;
                rng_state ^= rng_state >> 7;
                rng_state ^= rng_state << 17;
                rng_state
            };
            let mut g = Graph::with_vertices(n);
            let edge_count = next() as u32 % (n * 3 + 1);
            let mut caps = Vec::with_capacity(edge_count as usize);
            for _ in 0..edge_count {
                let u = (next() as u32) % n;
                let v = (next() as u32) % n;
                g.add_edge(u, v).unwrap();
                let c = f64::from((next() as u32 % 16) + 1);
                caps.push(c);
            }
            let source = 0;
            let target = n - 1;
            if source == target {
                return Ok(());
            }
            let dinic = max_flow_value(&g, source, target, Some(&caps)).unwrap();
            let ref_val = edmonds_karp(&g, source, target, &caps);
            prop_assert!(
                (dinic - ref_val).abs() < 1e-9,
                "dinic={dinic} ref={ref_val}"
            );
        }
    }
}
