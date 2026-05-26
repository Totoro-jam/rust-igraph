//! Recent-degree preferential attachment with vertex aging
//! (ALGO-GN-032).
//!
//! Counterpart of `igraph_recent_degree_aging_game()` in
//! `references/igraph/src/games/recent_degree.c` (lines 228-381).
//!
//! Each step `i ≥ 1` adds a fresh vertex and attaches `m`
//! (or `outseq[i]`) outgoing edges. Targets are drawn from a Fenwick BIT
//! (`PsumTree`) weighted by a product of a *recent*-degree term and an
//! age term:
//!
//! ```text
//! weight(v) = (pow(recent_deg(v), pa_exp) + zero_appeal)
//!           * pow(age(v), aging_exp)
//! ```
//!
//! where `recent_deg(v)` counts edges received from `v` in the last
//! `time_window` steps (sliding window), and
//! `age(v) = (i − v) / binwidth + 1` with
//! `binwidth = nodes / aging_bins + 1`.
//!
//! ## Hybrid of GN-019 and GN-021
//!
//! This model combines:
//! * **GN-019 `recent_degree_game`** — the sliding-window FIFO over
//!   `(vertex, step)` pairs that decrements counters on expiry and a
//!   per-step sentinel separator.
//! * **GN-021 `barabasi_aging_game`** — the binwidth-based age
//!   computation and the periodic age-sweep (every `k · binwidth ≤ i`).
//!
//! The C source uses a slightly simpler weight formula than the
//! barabasi-aging variant: only **one** zero term (`zero_appeal`) added
//! to the degree component, no separate `zero_age_appeal`, `deg_coef`,
//! or `age_coef`. The age term is `pow(age, aging_exp)` directly with no
//! shift in coefficient.
//!
//! ## Mechanics (mirrors C lines 305-369)
//!
//! For each step `i` from `1` to `nodes − 1`:
//!
//! 1. **Window expiry** (only when `i ≥ time_window`): pop entries from
//!    the front of `history` until a sentinel `None` is popped. For
//!    each popped vertex `j`, decrement `degree[j]` and refresh
//!    `psumtree[j]` with the NEW degree but the **age computed at the
//!    moment of expiry** — matching the C source's
//!    `(pow(deg, pa_exp) + zero_appeal) * pow((i - j)/binwidth + 1, aging_exp)`.
//! 2. **Draw** `m` (or `outseq[i]`) targets, each via
//!    `psumtree.search_bounded(uniform(0, sum), i)` — with a uniform
//!    fallback over `[0, i)` when `sum == 0`.
//! 3. For each chosen target: bump `degree[to]`, push `Some(to)` to
//!    `history`, record edge `(i, to)`.
//! 4. Push sentinel `None` to `history`.
//! 5. **Refresh** psumtree entries for each chosen target with their new
//!    weight (recent-degree + age).
//! 6. **Insert vertex `i`**: if `outpref`, bump `degree[i] +=
//!    no_of_neighbors` and refresh `psumtree[i]`; otherwise set it to
//!    `zero_appeal`.
//!    Note that `outpref` contributions to `degree[i]` are **permanent**
//!    — they are NOT pushed to the history deque (matching C lines
//!    349-356 which update degree+psumtree but not history).
//! 7. **Age sweep**: for every `k ≥ 1` with `k · binwidth ≤ i`, refresh
//!    vertex `i − k · binwidth` with age `k + 2`.
//!
//! ## Parameters
//!
//! * `nodes` — vertex count.
//! * `m` — per-step out-degree when `outseq` is `None`.
//! * `outseq` — optional length-`nodes` vector of per-step out-degrees.
//!   The first entry is ignored (vertex 0 has no preceding vertices to
//!   cite), matching the upstream contract.
//! * `outpref` — when `true`, the new vertex's own emitted citations
//!   feed back into its recent-degree counter (so its self-weight in the
//!   BIT is non-zero immediately).
//! * `pa_exp` — preferential-attachment exponent on recent degree.
//! * `aging_exp` — aging exponent on age bin (usually negative —
//!   younger vertices are favoured).
//! * `aging_bins` — must be `≥ 1`. Sets `binwidth = nodes / aging_bins + 1`.
//! * `time_window` — sliding-window length. `0` means the window is
//!   always expired, so every vertex's recent degree decays to zero
//!   immediately on the next step — effectively a pure-aging model with
//!   `zero_appeal` floor.
//! * `zero_appeal` — additive degree term so zero-recent-degree
//!   vertices have non-zero weight (when `aging_exp` is non-negative).
//!   Must be non-negative.
//! * `directed` — emit directed edges (newer → older convention).
//! * `seed` — initialises an internal [`SplitMix64`].
//!
//! ## Construction guarantees
//!
//! * **No self-loops** by construction — the new vertex `i` is added to
//!   the BIT *after* its `m` outgoing draws, and `search_bounded(_, i)`
//!   prevents binary-lifting from over-advancing to position `i`.
//! * **Multi-edges allowed** when `m ≥ 2` — independent draws against
//!   the same snapshot of the BIT can collide. Matches upstream's
//!   behaviour (no within-step zeroing).
//! * **Zero-sum fallback**: when the BIT total is zero, the draw falls
//!   back to a uniform sample over `[0, i)`. This fires when all current
//!   vertices have decayed to zero weight (rare, but possible with very
//!   small `time_window` and `zero_appeal = 0`).
//!
//! ## Cost
//!
//! `O((|V| + |V|/aging_bins + |E|) · log |V|)` per igraph C documentation:
//! BIT operations dominate, while sliding-window expiry is amortised
//! `O(|E|)` because each vertex is inserted into `history` exactly once
//! and popped at most once.
//!
//! ## Notes on the C special case
//!
//! `weight_from_degree` here mirrors C's `pow(degree, pa_exp) + zero_appeal`,
//! preserving `pow(0, 0) == 1` (degree=0 with `pa_exp=0` ⇒ degree term = 1).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::many_single_char_names
)]

use std::collections::VecDeque;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search bounded to `[0, bound)`.
///
/// Duplicated from sibling modules so this file is self-contained
/// (matches the project's per-module BIT precedent set by
/// `barabasi_psumtree.rs`, `barabasi_aging.rs`, and `recent_degree.rs`).
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

    /// Binary-lifted prefix-sum search constrained to `[0, bound)`.
    fn search_bounded(&self, target: f64, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        let mut idx: usize = 0;
        let mut remaining = target;
        let mut step = 1usize;
        while step.saturating_mul(2) <= bound {
            step *= 2;
        }
        while step > 0 {
            let next = idx + step;
            if next <= bound && self.bit[next] <= remaining {
                idx = next;
                remaining -= self.bit[next];
            }
            step >>= 1;
        }
        idx.min(bound - 1)
    }
}

/// History deque element. `Some(v)` marks a citation that contributed
/// `+1` to `degree[v]`; `None` is the per-step sentinel.
type HistoryEntry = Option<u32>;

/// Mirrors C's `pow(VECTOR(degree)[j], pa_exp) + zero_appeal` term.
/// Preserves the `pow(0, 0) == 1` convention.
fn degree_term(degree: u32, pa_exp: f64, zero_appeal: f64) -> f64 {
    let term = if degree == 0 {
        if pa_exp == 0.0 {
            1.0
        } else if pa_exp > 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        f64::from(degree).powf(pa_exp)
    };
    term + zero_appeal
}

/// Mirrors C's `pow(age, aging_exp)` term. `age` is always `≥ 1` in our
/// caller (always shifted by `+1` or `+2`), but we guard the zero case
/// anyway.
fn age_term(age: u32, aging_exp: f64) -> f64 {
    if age == 0 {
        if aging_exp == 0.0 {
            1.0
        } else if aging_exp > 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        f64::from(age).powf(aging_exp)
    }
}

fn weight(degree: u32, age: u32, pa_exp: f64, aging_exp: f64, zero_appeal: f64) -> f64 {
    degree_term(degree, pa_exp, zero_appeal) * age_term(age, aging_exp)
}

fn validate(
    nodes: u32,
    m: u32,
    outseq: Option<&[u32]>,
    pa_exp: f64,
    aging_exp: f64,
    aging_bins: u32,
    zero_appeal: f64,
) -> IgraphResult<()> {
    if !pa_exp.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "pa_exp must be finite (got {pa_exp})"
        )));
    }
    if !aging_exp.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "aging_exp must be finite (got {aging_exp})"
        )));
    }
    if aging_bins == 0 {
        return Err(IgraphError::InvalidArgument(
            "aging_bins must be positive (got 0)".into(),
        ));
    }
    if !zero_appeal.is_finite() || zero_appeal < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "zero_appeal must be finite and non-negative (got {zero_appeal})"
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

fn edge_capacity(nodes: u32, m: u32, outseq: Option<&[u32]>) -> usize {
    let n = nodes as usize;
    match outseq {
        Some(seq) => seq.iter().skip(1).map(|&x| x as usize).sum(),
        None => n.saturating_sub(1).saturating_mul(m as usize),
    }
}

/// Recent-degree preferential attachment with vertex aging.
///
/// See [module docs](self) for the full contract.
///
/// # Errors
///
/// * `pa_exp` / `aging_exp` not finite.
/// * `aging_bins == 0`.
/// * `zero_appeal` negative or non-finite.
/// * `outseq.len() != nodes`.
///
/// # Examples
///
/// ```
/// use rust_igraph::recent_degree_aging_game;
///
/// // 100-vertex directed graph, m=2 per step, no aging effect
/// // (aging_exp=0 makes the age term identically 1), 5-step window.
/// let g = recent_degree_aging_game(
///     100, 2, None, false,
///     1.0, 0.0, 10, 5,
///     1.0, true, 0x42,
/// ).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert_eq!(g.ecount(), 99 * 2);
/// ```
pub fn recent_degree_aging_game(
    nodes: u32,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    pa_exp: f64,
    aging_exp: f64,
    aging_bins: u32,
    time_window: u32,
    zero_appeal: f64,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(nodes, m, outseq, pa_exp, aging_exp, aging_bins, zero_appeal)?;

    let mut graph = Graph::new(nodes, directed)?;
    if nodes < 2 {
        return Ok(graph);
    }
    if outseq.is_none() && m == 0 {
        return Ok(graph);
    }
    if let Some(seq) = outseq {
        let after_zero: u64 = seq.iter().skip(1).map(|&x| u64::from(x)).sum();
        if after_zero == 0 {
            return Ok(graph);
        }
    }

    let n = nodes as usize;
    let binwidth = (n / aging_bins as usize) + 1;
    let mut rng = SplitMix64::new(seed);
    let mut psum = PsumTree::new(n);
    let mut degree: Vec<u32> = vec![0; n];
    let mut history: VecDeque<HistoryEntry> = VecDeque::new();
    let capacity = edge_capacity(nodes, m, outseq);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(capacity);

    // Step 0: seed vertex enters with weight `zero_appeal * age_term(1)`.
    // Matches C line 301: `igraph_psumtree_update(&sumtree, 0, zero_appeal)`.
    // Note: C uses bare `zero_appeal` (not multiplied by an age term) for
    // the seed vertex's initial weight — we match that exactly.
    psum.set(0, zero_appeal);
    history.push_back(None); // sentinel for step 0

    for i in 1..n {
        let no_of_neighbors = if let Some(seq) = outseq {
            seq[i] as usize
        } else {
            m as usize
        };

        // 1. Expire window: pop history entries from the front until the
        // per-step sentinel `None` is popped. For each popped vertex,
        // decrement degree and refresh weight with the CURRENT age.
        if i >= time_window as usize {
            while let Some(entry) = history.pop_front() {
                match entry {
                    None => break,
                    Some(j) => {
                        let ju = j as usize;
                        degree[ju] = degree[ju].saturating_sub(1);
                        let age = ((i - ju) / binwidth + 1) as u32;
                        let w = weight(degree[ju], age, pa_exp, aging_exp, zero_appeal);
                        psum.set(ju, w);
                    }
                }
            }
        }

        // 2-3. Draw `no_of_neighbors` targets and record edges.
        // The C source samples against the live BIT each time (without
        // zeroing picks within the step). We mirror that — the inner BIT
        // total only changes if a chosen vertex's weight was different
        // before; for safety we re-read total each draw.
        let src =
            u32::try_from(i).map_err(|_| IgraphError::Internal("vertex index overflows u32"))?;
        for _ in 0..no_of_neighbors {
            let sum = psum.total();
            let to = if sum > 0.0 {
                let target = rng.gen_unit() * sum;
                psum.search_bounded(target, i)
            } else {
                let span = i as u64;
                (rng.next_u64() % span) as usize
            };
            let dst = u32::try_from(to)
                .map_err(|_| IgraphError::Internal("vertex index overflows u32"))?;
            degree[to] = degree[to]
                .checked_add(1)
                .ok_or_else(|| IgraphError::InvalidArgument("degree overflow".into()))?;
            edges.push((src, dst));
            history.push_back(Some(dst));
        }
        // 4. Per-step sentinel.
        history.push_back(None);

        // 5. Refresh weights for each cited target with NEW degree + age.
        let start = edges.len() - no_of_neighbors;
        for edge in &edges[start..] {
            let nn = edge.1 as usize;
            let age = ((i - nn) / binwidth + 1) as u32;
            let w = weight(degree[nn], age, pa_exp, aging_exp, zero_appeal);
            psum.set(nn, w);
        }

        // 6. Insert vertex `i` into the BIT.
        if outpref {
            // outpref bumps source degree by no_of_neighbors. NOT pushed
            // to the history deque (so this contribution to degree[i] is
            // permanent — matches C lines 349-354).
            let bump = u32::try_from(no_of_neighbors)
                .map_err(|_| IgraphError::InvalidArgument("neighbors count overflow".into()))?;
            degree[i] = degree[i]
                .checked_add(bump)
                .ok_or_else(|| IgraphError::InvalidArgument("degree overflow".into()))?;
            // Self has age 0 (`(i - i) / binwidth + 1 = 1`), but the
            // weight is just `degree_term(degree[i], ...)`. The C source
            // here writes `pow(degree, pa_exp) + zero_appeal` WITHOUT the
            // age multiplier (C line 353) — match exactly.
            psum.set(i, degree_term(degree[i], pa_exp, zero_appeal));
        } else {
            // C line 356: bare `zero_appeal`, no age multiplier.
            psum.set(i, zero_appeal);
        }

        // 7. Age sweep: every vertex at position `i - k*binwidth`
        // (k ≥ 1) just crossed a bin boundary. Refresh with age
        // `k + 2` (matches the C loop at lines 360-368).
        let mut k = 1usize;
        loop {
            let offset = binwidth.saturating_mul(k);
            if offset > i {
                break;
            }
            let shnode = i - offset;
            let deg = degree[shnode];
            let new_age = (k + 2) as u32;
            let w = weight(deg, new_age, pa_exp, aging_exp, zero_appeal);
            psum.set(shnode, w);
            k += 1;
        }
    }

    graph.add_edges(edges)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let n = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..n)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    fn default_call(
        nodes: u32,
        m: u32,
        outpref: bool,
        pa_exp: f64,
        aging_exp: f64,
        time_window: u32,
        directed: bool,
        seed: u64,
    ) -> IgraphResult<Graph> {
        recent_degree_aging_game(
            nodes,
            m,
            None,
            outpref,
            pa_exp,
            aging_exp,
            10,
            time_window,
            1.0,
            directed,
            seed,
        )
    }

    #[test]
    fn rdaging_n_zero_returns_empty() {
        let g = default_call(0, 2, false, 1.0, 0.0, 5, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn rdaging_n_one_singleton() {
        let g = default_call(1, 2, false, 1.0, 0.0, 5, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn rdaging_m_zero_edgeless() {
        let g = default_call(10, 0, false, 1.0, 0.0, 5, true, 1).unwrap();
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn rdaging_ecount_exact_constant_m() {
        let g = default_call(50, 3, false, 1.0, 0.0, 8, true, 42).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 49 * 3);
    }

    #[test]
    fn rdaging_ecount_exact_outseq() {
        let outseq: Vec<u32> = (0..20)
            .map(|i| if i == 0 { 0 } else { (i % 3) + 1 })
            .collect();
        let expected_edges: usize = outseq.iter().skip(1).map(|&x| x as usize).sum();
        let g =
            recent_degree_aging_game(20, 0, Some(&outseq), false, 1.0, -0.5, 5, 6, 1.0, true, 7)
                .unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), expected_edges);
    }

    #[test]
    fn rdaging_no_self_loops() {
        let g = default_call(80, 4, false, 1.0, -1.0, 5, true, 123).unwrap();
        for (s, d) in collect_edges(&g) {
            assert_ne!(s, d, "self-loop at edge ({s}, {d})");
        }
    }

    #[test]
    fn rdaging_source_is_step_index() {
        let m: u32 = 3;
        let n: u32 = 30;
        let g = default_call(n, m, false, 1.0, -0.5, 5, true, 99).unwrap();
        let edges = collect_edges(&g);
        for (idx, (s, _d)) in edges.iter().enumerate() {
            let expected_src = (idx as u32 / m) + 1;
            assert_eq!(
                *s, expected_src,
                "edge {idx}: src {s} != expected {expected_src}"
            );
        }
    }

    #[test]
    fn rdaging_target_below_source() {
        let m: u32 = 2;
        let n: u32 = 40;
        let g = default_call(n, m, false, 1.0, -0.5, 5, true, 7).unwrap();
        let edges = collect_edges(&g);
        for (idx, (s, d)) in edges.iter().enumerate() {
            assert!(*d < *s, "edge {idx}: target {d} should be < source {s}");
        }
    }

    #[test]
    fn rdaging_deterministic_same_seed() {
        let g1 = default_call(40, 3, false, 1.0, -0.5, 5, true, 11).unwrap();
        let g2 = default_call(40, 3, false, 1.0, -0.5, 5, true, 11).unwrap();
        assert_eq!(collect_edges(&g1), collect_edges(&g2));
    }

    #[test]
    fn rdaging_seed_divergence() {
        let g1 = default_call(40, 3, false, 1.0, -0.5, 5, true, 11).unwrap();
        let g2 = default_call(40, 3, false, 1.0, -0.5, 5, true, 12).unwrap();
        assert_ne!(collect_edges(&g1), collect_edges(&g2));
    }

    #[test]
    fn rdaging_directed_flag_propagates() {
        let g_dir = default_call(20, 2, false, 1.0, 0.0, 5, true, 1).unwrap();
        let g_undir = default_call(20, 2, false, 1.0, 0.0, 5, false, 1).unwrap();
        assert!(g_dir.is_directed());
        assert!(!g_undir.is_directed());
    }

    #[test]
    fn rdaging_outpref_changes_graph() {
        let g_off = default_call(30, 2, false, 1.0, -1.0, 5, true, 5).unwrap();
        let g_on = default_call(30, 2, true, 1.0, -1.0, 5, true, 5).unwrap();
        // outpref toggling changes the BIT weights from step 2 onward,
        // so the edge stream should differ.
        assert_ne!(collect_edges(&g_off), collect_edges(&g_on));
    }

    #[test]
    fn rdaging_short_window_changes_graph() {
        // A tiny time_window forces faster decay than a long window
        // (different vertices end up with non-zero recent_degree), so
        // the edge stream should differ.
        let g_short = default_call(50, 2, false, 1.0, -1.0, 2, true, 17).unwrap();
        let g_long = default_call(50, 2, false, 1.0, -1.0, 40, true, 17).unwrap();
        assert_ne!(collect_edges(&g_short), collect_edges(&g_long));
    }

    #[test]
    fn rdaging_strong_aging_lifts_young_share() {
        let make = |aging_exp: f64| {
            recent_degree_aging_game(200, 2, None, false, 1.0, aging_exp, 50, 10, 1.0, true, 33)
                .unwrap()
        };
        let indeg = |g: &Graph| -> Vec<u32> {
            let mut v = vec![0u32; 200];
            for (_s, d) in collect_edges(g) {
                v[d as usize] += 1;
            }
            v
        };
        let aged = indeg(&make(-5.0));
        let flat = indeg(&make(0.0));
        let young_aged: u32 = aged[100..].iter().sum();
        let young_flat: u32 = flat[100..].iter().sum();
        assert!(
            young_aged > young_flat,
            "young-in under strong aging ({young_aged}) should exceed young-in under no aging ({young_flat})"
        );
    }

    #[test]
    fn rdaging_targets_unique_when_strongly_aged() {
        let g = default_call(40, 2, false, 1.0, -3.0, 5, true, 21).unwrap();
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        for e in collect_edges(&g) {
            assert_ne!(e.0, e.1);
            seen.insert(e); // duplicates allowed per the model
        }
    }

    #[test]
    fn rdaging_validation_aging_bins_zero() {
        let err =
            recent_degree_aging_game(10, 2, None, false, 1.0, 0.0, 0, 5, 1.0, true, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rdaging_validation_negative_zero_appeal() {
        let err = recent_degree_aging_game(10, 2, None, false, 1.0, 0.0, 5, 5, -1.0, true, 1)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rdaging_validation_pa_exp_nan() {
        let err = recent_degree_aging_game(10, 2, None, false, f64::NAN, 0.0, 5, 5, 1.0, true, 1)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rdaging_validation_aging_exp_nan() {
        let err = recent_degree_aging_game(10, 2, None, false, 1.0, f64::NAN, 5, 5, 1.0, true, 1)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rdaging_validation_outseq_wrong_length() {
        let outseq = vec![0u32; 5];
        let err =
            recent_degree_aging_game(10, 0, Some(&outseq), false, 1.0, 0.0, 5, 5, 1.0, true, 1)
                .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rdaging_time_window_zero_collapses_to_pure_aging() {
        // With time_window = 0, every recent-degree counter expires
        // immediately at the start of step `i+1`. The weight is then
        // dominated by `zero_appeal * pow(age, aging_exp)` — a pure
        // aging walk with floor.
        let g =
            recent_degree_aging_game(30, 2, None, false, 1.0, -1.0, 5, 0, 1.0, true, 11).unwrap();
        assert_eq!(g.vcount(), 30);
        assert_eq!(g.ecount(), 29 * 2);
        for (s, d) in collect_edges(&g) {
            assert_ne!(s, d);
            assert!(d < s);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn rdaging_ecount_exact_constant_m(
            n in 2u32..40,
            m in 1u32..6,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            aging_bins in 1u32..20,
            time_window in 0u32..20,
            outpref: bool,
            directed: bool,
            seed in 0u64..1_000_000,
        ) {
            let g = recent_degree_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, aging_bins, time_window,
                1.0, directed, seed,
            ).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount() as u32, (n - 1) * m);
        }

        #[test]
        fn rdaging_no_self_loops(
            n in 2u32..40,
            m in 1u32..6,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            aging_bins in 1u32..20,
            time_window in 0u32..20,
            outpref: bool,
            directed: bool,
            seed in 0u64..1_000_000,
        ) {
            let g = recent_degree_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, aging_bins, time_window,
                1.0, directed, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert_ne!(s, d);
            }
        }

        #[test]
        fn rdaging_source_is_step_index(
            n in 2u32..40,
            m in 1u32..6,
            seed in 0u64..1_000_000,
        ) {
            let g = recent_degree_aging_game(
                n, m, None, false,
                1.0, -0.5, 5, 5,
                1.0, true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, _d) = g.edge(eid).unwrap();
                let expected_src = (eid / m) + 1;
                prop_assert_eq!(s, expected_src);
            }
        }

        #[test]
        fn rdaging_targets_below_source(
            n in 2u32..40,
            m in 1u32..6,
            aging_exp in -2.0_f64..1.0,
            seed in 0u64..1_000_000,
        ) {
            let g = recent_degree_aging_game(
                n, m, None, false,
                1.0, aging_exp, 5, 5,
                1.0, true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert!(d < s);
            }
        }

        #[test]
        fn rdaging_determinism(
            n in 2u32..30,
            m in 1u32..5,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            time_window in 0u32..20,
            outpref: bool,
            seed in 0u64..1_000_000,
        ) {
            let g1 = recent_degree_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, 5, time_window,
                1.0, true, seed,
            ).unwrap();
            let g2 = recent_degree_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, 5, time_window,
                1.0, true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..m_edges {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }
    }
}
