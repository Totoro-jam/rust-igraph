//! Recent-degree game (ALGO-GN-019).
//!
//! Counterpart of `igraph_recent_degree_game()` from
//! `references/igraph/src/games/recent_degree.c:32-186`. Models a
//! growing network where the probability of citing a vertex is
//! proportional to the number of edges it received in the last
//! `time_window` time steps, raised to `power`, plus a `zero_appeal`
//! floor for never-recently-cited vertices.
//!
//! ## Algorithm
//!
//! Every vertex `v` carries a weight in a prefix-sum tree:
//!   `weight[v] = degree[v]^power + zero_appeal`
//! where `degree[v]` is the count of edges in the sliding window that
//! pointed at `v` (and originated from `v`, when `outpref = true`).
//!
//! The window itself is maintained by a deque (`history`). Entries
//! ordered by time of creation, with a sentinel value separating each
//! step's contributions. On step `i`:
//!
//! 1. **Expire** (only when `i >= time_window`): pop entries from the
//!    front of `history` until a sentinel is popped. For each popped
//!    vertex `j`, decrement `degree[j]` and refresh `psumtree[j]`.
//! 2. **Draw** `m` (or `outseq[i]`) citations, each via
//!    `psumtree.search(uniform(0, sum))` — with a uniform fallback
//!    over `[0, i)` when `sum == 0`. For each citation
//!    `(src=i, dst=to)`: bump `degree[to]`, push `to` to `history`,
//!    record the edge.
//! 3. Push a sentinel to `history`.
//! 4. **Refresh** psumtree entries for each cited vertex with its new
//!    degree.
//! 5. If `outpref`: bump `degree[i] += m` and refresh `psumtree[i]`;
//!    otherwise `psumtree[i] = zero_appeal`.
//!
//! The structure is a binary-indexed (Fenwick) tree with binary-lifting
//! prefix-search, giving O(log n) per update / search.
//!
//! ## Self-loops & multi-edges
//!
//! * **Self-loops**: impossible. The psumtree only ranges over vertices
//!   `[0, i)`; vertex `i` is added *after* its outgoing edges are drawn.
//! * **Multi-edges**: yes, when `m ≥ 2`. Two draws at the same step can
//!   land on the same target. Caller should `simplify()` if simple
//!   output is required (matches upstream behaviour).
//!
//! ## Determinism
//!
//! Reproducible against the shared `SplitMix64`
//! PRNG. Not bit-portable to upstream igraph's GLIBC RNG; conformance
//! asserts structural invariants only.
//!
//! ## Validation
//!
//! * `power` and `zero_appeal` must be finite and non-NaN.
//! * `zero_appeal >= 0`.
//! * `outseq` (when present) must have length `nodes`.
//! * The first entry of `outseq` is conventionally `0` (matching the C
//!   reference, which drops `outseq[0]` from the edge total — vertex 0
//!   has no preceding vertices to cite).
//!
//! ## References
//!
//! * Upstream igraph C `igraph_recent_degree_game`, MIT-licensed
//!   reference port.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::many_single_char_names
)]

use std::collections::VecDeque;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search.
struct PsumTree {
    n: usize,
    bit: Vec<f64>,
    values: Vec<f64>,
    total: f64,
}

impl PsumTree {
    fn new(n: usize) -> Self {
        Self {
            n,
            bit: vec![0.0; n + 1],
            values: vec![0.0; n],
            total: 0.0,
        }
    }

    fn set(&mut self, i: usize, v: f64) {
        let delta = v - self.values[i];
        self.values[i] = v;
        self.total += delta;
        let mut k = i + 1;
        while k <= self.n {
            self.bit[k] += delta;
            k += k & k.wrapping_neg();
        }
    }

    fn total(&self) -> f64 {
        self.total
    }

    fn search(&self, target: f64) -> usize {
        if self.n == 0 {
            return 0;
        }
        let mut idx: usize = 0;
        let mut remaining = target;
        let mut step = (self.n + 1).next_power_of_two() >> 1;
        while step > 0 {
            let next = idx + step;
            if next <= self.n && self.bit[next] <= remaining {
                idx = next;
                remaining -= self.bit[next];
            }
            step >>= 1;
        }
        idx.min(self.n - 1)
    }
}

/// History deque element. Wraps a vertex id; `None` is the
/// per-step sentinel.
type HistoryEntry = Option<u32>;

fn weight_from_degree(degree: u32, power: f64, zero_appeal: f64) -> f64 {
    // Match C `pow(VECTOR(degree)[j], power) + zero_appeal`.
    // `pow(0, positive) == 0`, `pow(0, 0) == 1`.
    let d = f64::from(degree);
    let term = if degree == 0 && power == 0.0 {
        1.0
    } else if degree == 0 {
        0.0
    } else {
        d.powf(power)
    };
    term + zero_appeal
}

fn validate(
    nodes: u32,
    power: f64,
    m: u32,
    outseq: Option<&[u32]>,
    zero_appeal: f64,
) -> IgraphResult<()> {
    if power.is_nan() || !power.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "power must be finite (got {power})"
        )));
    }
    if zero_appeal.is_nan() || !zero_appeal.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "zero_appeal must be finite (got {zero_appeal})"
        )));
    }
    if zero_appeal < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "zero_appeal must be non-negative (got {zero_appeal})"
        )));
    }
    if let Some(seq) = outseq {
        if seq.len() != nodes as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "outseq length ({}) must equal nodes ({nodes})",
                seq.len()
            )));
        }
    }
    let _ = m;
    Ok(())
}

/// Recent-degree game. See module docs.
pub fn recent_degree_game(
    nodes: u32,
    power: f64,
    time_window: u32,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    zero_appeal: f64,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(nodes, power, m, outseq, zero_appeal)?;

    let mut graph = Graph::new(nodes, directed)?;
    if nodes == 0 || nodes < 2 {
        return Ok(graph);
    }
    // If both no outseq AND m == 0, no edges at all.
    if outseq.is_none() && m == 0 {
        return Ok(graph);
    }
    // If outseq is given and the total minus outseq[0] is zero, no edges.
    if let Some(seq) = outseq {
        let total_after_zero: u64 = seq.iter().skip(1).map(|&x| u64::from(x)).sum();
        if total_after_zero == 0 {
            return Ok(graph);
        }
    }

    let n = nodes as usize;
    let mut rng = SplitMix64::new(seed);
    let mut psum = PsumTree::new(n);
    let mut degree: Vec<u32> = vec![0; n];
    let mut history: VecDeque<HistoryEntry> = VecDeque::new();

    // Pre-compute edge capacity to avoid reallocation.
    let edge_capacity = if let Some(seq) = outseq {
        seq.iter().skip(1).map(|&x| u64::from(x)).sum::<u64>() as usize
    } else {
        (n - 1).saturating_mul(m as usize)
    };
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edge_capacity);

    // First vertex enters with degree 0 — weight = zero_appeal (per C: explicit
    // `igraph_psumtree_update(&sumtree, 0, zero_appeal)`).
    psum.set(0, zero_appeal);
    history.push_back(None); // sentinel for step 0

    for i in 1..n {
        let no_of_neighbors = if let Some(seq) = outseq {
            seq[i] as usize
        } else {
            m as usize
        };

        // Expire window: pop entries until a sentinel.
        if i >= time_window as usize {
            while let Some(entry) = history.pop_front() {
                match entry {
                    None => break,
                    Some(j) => {
                        let ju = j as usize;
                        degree[ju] -= 1;
                        psum.set(ju, weight_from_degree(degree[ju], power, zero_appeal));
                    }
                }
            }
        }

        // Draw `no_of_neighbors` citations.
        for _ in 0..no_of_neighbors {
            let sum = psum.total();
            let to = if sum > 0.0 {
                let target = rng.gen_unit() * sum;
                psum.search(target)
            } else {
                // Fallback: uniform over [0, i).
                let span = i as u64;
                (rng.next_u64() % span) as usize
            };
            let src = u32::try_from(i)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            let dst = u32::try_from(to)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            degree[to] = degree[to]
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("degree overflow".into()))?;
            edges.push((src, dst));
            history.push_back(Some(dst));
        }
        history.push_back(None); // sentinel for step i

        // Refresh psum for each cited target.
        // Walk back the last `no_of_neighbors` edges.
        let start = edges.len() - no_of_neighbors;
        for edge in &edges[start..] {
            let nn = edge.1 as usize;
            psum.set(nn, weight_from_degree(degree[nn], power, zero_appeal));
        }

        // Update self entry depending on outpref.
        if outpref {
            // outpref bumps source degree by no_of_neighbors. Use checked
            // add to keep our no-unwrap discipline.
            let bump = u32::try_from(no_of_neighbors)
                .map_err(|_| IgraphError::InvalidArgument("neighbors count overflow".into()))?;
            degree[i] = degree[i]
                .checked_add(bump)
                .ok_or_else(|| IgraphError::InvalidArgument("degree overflow".into()))?;
            psum.set(i, weight_from_degree(degree[i], power, zero_appeal));
        } else {
            psum.set(i, zero_appeal);
        }
    }

    graph.add_edges(edges)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_edges_match(g: &Graph, expected_count: u64) {
        assert_eq!(g.ecount() as u64, expected_count);
    }

    #[test]
    fn empty_graph_returns_empty() {
        let g = recent_degree_game(0, 1.0, 5, 2, None, false, 1.0, true, 42).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_vertex_no_edges() {
        let g = recent_degree_game(1, 1.0, 5, 2, None, false, 1.0, true, 42).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn m_zero_no_edges() {
        let g = recent_degree_game(10, 1.0, 5, 0, None, false, 1.0, true, 42).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn ecount_exact_constant_m() {
        let n = 50u32;
        let m = 3u32;
        let g = recent_degree_game(n, 1.0, 10, m, None, false, 1.0, true, 42).unwrap();
        assert_eq!(g.vcount(), n);
        assert_edges_match(&g, u64::from(n - 1) * u64::from(m));
    }

    #[test]
    fn ecount_exact_outseq() {
        let n = 20u32;
        let outseq: Vec<u32> = (0..n).map(|i| if i == 0 { 0 } else { 2 }).collect();
        let g = recent_degree_game(n, 1.0, 5, 0, Some(&outseq), false, 1.0, true, 42).unwrap();
        let expected: u64 = outseq.iter().skip(1).map(|&x| u64::from(x)).sum();
        assert_edges_match(&g, expected);
    }

    #[test]
    fn determinism_same_seed() {
        let g1 = recent_degree_game(40, 1.5, 8, 2, None, false, 1.0, true, 123).unwrap();
        let g2 = recent_degree_game(40, 1.5, 8, 2, None, false, 1.0, true, 123).unwrap();
        assert_eq!(g1.vcount(), g2.vcount());
        assert_eq!(g1.ecount(), g2.ecount());
        let m = u32::try_from(g1.ecount()).unwrap();
        for eid in 0..m {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn seed_divergence() {
        let g1 = recent_degree_game(40, 1.5, 8, 2, None, false, 1.0, true, 1).unwrap();
        let g2 = recent_degree_game(40, 1.5, 8, 2, None, false, 1.0, true, 2).unwrap();
        let m = u32::try_from(g1.ecount()).unwrap();
        let mut diff = 0;
        for eid in 0..m {
            if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                diff += 1;
            }
        }
        assert!(diff > 0, "two seeds should yield different graphs");
    }

    #[test]
    fn no_self_loops_when_positive_appeal() {
        let g = recent_degree_game(50, 1.0, 5, 3, None, false, 1.0, true, 42).unwrap();
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (s, d) = g.edge(eid).unwrap();
            assert_ne!(s, d, "edge {eid} is a self-loop");
        }
    }

    #[test]
    fn source_is_always_step_index() {
        let n = 30u32;
        let m = 2u32;
        let g = recent_degree_game(n, 1.0, 5, m, None, false, 1.0, true, 42).unwrap();
        let edges_n = u32::try_from(g.ecount()).unwrap();
        for eid in 0..edges_n {
            let (s, _) = g.edge(eid).unwrap();
            let step = eid / m + 1; // step i emits eps edges starting at index (i-1)*m
            assert_eq!(s, step, "edge {eid} src {s} ≠ step {step}");
        }
    }

    #[test]
    fn target_in_zero_to_step_minus_one() {
        let g = recent_degree_game(30, 1.0, 5, 2, None, false, 1.0, true, 42).unwrap();
        let edges_n = u32::try_from(g.ecount()).unwrap();
        for eid in 0..edges_n {
            let (s, d) = g.edge(eid).unwrap();
            assert!(d < s, "target {d} not in [0, src={s})");
        }
    }

    #[test]
    fn directed_flag_propagates() {
        let g1 = recent_degree_game(15, 1.0, 5, 2, None, false, 1.0, true, 42).unwrap();
        let g2 = recent_degree_game(15, 1.0, 5, 2, None, false, 1.0, false, 42).unwrap();
        assert!(g1.is_directed());
        assert!(!g2.is_directed());
    }

    #[test]
    fn outpref_changes_graph() {
        let g1 = recent_degree_game(40, 1.5, 10, 2, None, false, 1.0, true, 42).unwrap();
        let g2 = recent_degree_game(40, 1.5, 10, 2, None, true, 1.0, true, 42).unwrap();
        // Both should have the same ecount but different edge layout due
        // to different psumtree weights.
        assert_eq!(g1.ecount(), g2.ecount());
        let edges_n = u32::try_from(g1.ecount()).unwrap();
        let mut diff = 0;
        for eid in 0..edges_n {
            if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                diff += 1;
            }
        }
        assert!(
            diff > 0,
            "outpref=true and outpref=false should yield different graphs"
        );
    }

    #[test]
    fn time_window_zero_uses_zero_appeal_only() {
        // time_window=0 means edges expire on the very next step, so
        // every draw past the first only sees zero_appeal weights, which
        // are equal → uniform across all alive vertices.
        let g = recent_degree_game(20, 1.0, 0, 2, None, false, 1.0, true, 42).unwrap();
        // Still exactly (n-1)*m edges.
        assert_eq!(g.ecount(), 19 * 2);
        // No self-loops still.
        let edges_n = u32::try_from(g.ecount()).unwrap();
        for eid in 0..edges_n {
            let (s, d) = g.edge(eid).unwrap();
            assert_ne!(s, d);
        }
    }

    #[test]
    fn large_time_window_concentrates_on_early_vertices() {
        // With time_window >= n, no edges expire, so the algorithm
        // becomes plain preferential attachment. Old vertices dominate.
        let n = 200u32;
        let m = 3u32;
        let g = recent_degree_game(n, 1.0, n, m, None, true, 1.0, true, 42).unwrap();
        let mut indeg = vec![0u32; n as usize];
        let edges_n = u32::try_from(g.ecount()).unwrap();
        for eid in 0..edges_n {
            let (_, d) = g.edge(eid).unwrap();
            indeg[d as usize] += 1;
        }
        // The earliest 10% of vertices should hold > 20% of incoming edges.
        let early_cut = (n as usize) / 10;
        let early_sum: u32 = indeg[..early_cut].iter().sum();
        let total: u32 = indeg.iter().sum();
        assert!(
            early_sum * 5 > total,
            "expected early-vertex concentration (early_sum={early_sum}, total={total})"
        );
    }

    #[test]
    fn validation_outseq_wrong_length() {
        let outseq = vec![0u32, 1, 2];
        let err =
            recent_degree_game(10, 1.0, 5, 0, Some(&outseq), false, 1.0, true, 42).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("outseq length")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn validation_nan_power() {
        let err = recent_degree_game(10, f64::NAN, 5, 2, None, false, 1.0, true, 42).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("power must be finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn validation_inf_power() {
        let err =
            recent_degree_game(10, f64::INFINITY, 5, 2, None, false, 1.0, true, 42).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("power must be finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn validation_negative_zero_appeal() {
        let err = recent_degree_game(10, 1.0, 5, 2, None, false, -0.5, true, 42).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => {
                assert!(s.contains("zero_appeal must be non-negative"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn validation_nan_zero_appeal() {
        let err = recent_degree_game(10, 1.0, 5, 2, None, false, f64::NAN, true, 42).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("zero_appeal must be finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn weight_from_degree_zero_power_zero() {
        // pow(0, 0) = 1.0
        let w = weight_from_degree(0, 0.0, 0.5);
        assert_eq!(w, 1.5);
    }

    #[test]
    fn weight_from_degree_zero_positive_power() {
        let w = weight_from_degree(0, 1.0, 0.5);
        assert_eq!(w, 0.5);
    }

    #[test]
    fn weight_from_degree_nonzero() {
        let w = weight_from_degree(4, 0.5, 1.0);
        assert!((w - 3.0).abs() < 1e-12);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn determinism_under_proptest(
            nodes in 1u32..50,
            m in 0u32..4,
            power in 0.0f64..3.0,
            zero_appeal in 0.1f64..5.0,
            time_window in 1u32..20,
            directed in any::<bool>(),
            outpref in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g1 = recent_degree_game(nodes, power, time_window, m, None, outpref, zero_appeal, directed, seed).unwrap();
            let g2 = recent_degree_game(nodes, power, time_window, m, None, outpref, zero_appeal, directed, seed).unwrap();
            prop_assert_eq!(g1.vcount(), g2.vcount());
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let edges_n = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..edges_n {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }

        #[test]
        fn ecount_exact_constant_m(
            nodes in 1u32..40,
            m in 0u32..5,
            time_window in 1u32..20,
            seed in any::<u64>(),
        ) {
            let g = recent_degree_game(nodes, 1.0, time_window, m, None, false, 1.0, true, seed).unwrap();
            let expected = if nodes < 2 { 0 } else { u64::from(nodes - 1) * u64::from(m) };
            prop_assert_eq!(g.ecount() as u64, expected);
        }

        #[test]
        fn no_self_loops_when_zero_appeal_positive(
            nodes in 2u32..40,
            m in 1u32..4,
            power in 0.0f64..2.0,
            zero_appeal in 0.1f64..3.0,
            time_window in 1u32..20,
            seed in any::<u64>(),
        ) {
            let g = recent_degree_game(nodes, power, time_window, m, None, false, zero_appeal, true, seed).unwrap();
            let edges_n = u32::try_from(g.ecount()).unwrap();
            for eid in 0..edges_n {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert_ne!(s, d);
            }
        }

        #[test]
        fn source_is_always_step_index(
            nodes in 2u32..30,
            m in 1u32..4,
            seed in any::<u64>(),
        ) {
            let g = recent_degree_game(nodes, 1.0, 5, m, None, false, 1.0, true, seed).unwrap();
            let edges_n = u32::try_from(g.ecount()).unwrap();
            for eid in 0..edges_n {
                let (s, _) = g.edge(eid).unwrap();
                let step = eid / m + 1;
                prop_assert_eq!(s, step);
            }
        }

        #[test]
        fn target_in_zero_to_step_minus_one(
            nodes in 2u32..30,
            m in 1u32..4,
            seed in any::<u64>(),
        ) {
            let g = recent_degree_game(nodes, 1.0, 5, m, None, false, 1.0, true, seed).unwrap();
            let edges_n = u32::try_from(g.ecount()).unwrap();
            for eid in 0..edges_n {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert!(d < s);
            }
        }
    }
}
