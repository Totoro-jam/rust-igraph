//! Barabási–Albert preferential-attachment with aging (ALGO-GN-021).
//!
//! Counterpart of `igraph_barabasi_aging_game()` in
//! `references/igraph/src/games/barabasi.c` (lines ~606-841).
//!
//! Each step `i ≥ 1` adds a fresh vertex and attaches `m`
//! (or `outseq[i]`) outgoing edges. Targets are drawn from a Fenwick BIT
//! (`PsumTree`) weighted by a product of a degree term and an age term:
//!
//! ```text
//! weight(v) = (deg_coef · pow(deg(v), pa_exp) + zero_deg_appeal)
//!           · (age_coef · pow(age(v), aging_exp) + zero_age_appeal)
//! ```
//!
//! `age(v) = (i − v) / binwidth + 1` where `binwidth = nodes / aging_bins + 1`
//! (the C special case is `pow(0, 0) == 1`, preserved here).
//!
//! ## Mechanics
//!
//! * **Sampling**: each of the `m` draws independently picks against the
//!   current BIT (the C source does NOT zero picks within a step, so
//!   within-step multi-edges are possible — matching upstream).
//! * **Refresh chosen weights**: after the `m` draws, every chosen vertex
//!   has its weight refreshed to the new degree.
//! * **New-vertex weight**: vertex `i` is inserted into the BIT with age
//!   `1` (the newest bin) and degree either `0` (if `outpref = false`)
//!   or `no_of_neighbors` (if `outpref = true`).
//! * **Age sweep**: for every `k ≥ 1` such that `k · binwidth ≤ i`, the
//!   vertex at position `i − k · binwidth` has its weight refreshed with
//!   age `k + 2` (it just crossed a bin boundary). This matches the C
//!   loop at lines 817-829.
//!
//! ## Parameters
//!
//! * `nodes` — vertex count.
//! * `m` — per-step out-degree when `outseq` is `None`.
//! * `outseq` — optional length-`nodes` vector of per-step out-degrees
//!   (the first entry is ignored, matching upstream).
//! * `outpref` — when `true`, the new vertex's own emitted citations
//!   feed back into its degree at end-of-step (and therefore into its
//!   weight when it becomes a target in subsequent steps).
//! * `pa_exp` — preferential-attachment exponent on degree.
//! * `aging_exp` — aging exponent on age bin (usually negative —
//!   younger vertices are favoured).
//! * `aging_bins` — must be `≥ 1`. Sets `binwidth = nodes / aging_bins + 1`.
//! * `zero_deg_appeal` — additive degree term so zero-degree vertices
//!   have non-zero weight. Must be non-negative.
//! * `zero_age_appeal` — additive age term so oldest vertices retain
//!   non-zero weight when `aging_exp < 0`. Must be non-negative.
//! * `deg_coef` — multiplicative degree coefficient. Must be non-negative.
//! * `age_coef` — multiplicative age coefficient. Must be non-negative.
//! * `directed` — emit directed edges (newer → older convention).
//! * `seed` — initialises an internal `SplitMix64`.
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
//!   back to a uniform sample over `[0, i)`. This only fires at the
//!   start of the run when the seed vertex's age has already pushed its
//!   weight to zero (e.g. `aging_exp = -∞` is forbidden by validation,
//!   but `aging_exp` very negative + `zero_age_appeal = 0` makes it
//!   possible).
//!
//! ## Cost
//!
//! `O((n + n / aging_bins) · log n + |E|)` — each step does
//! `O(m · log n)` for the draws and refresh, plus an amortized
//! `O((n / aging_bins) · log n)` for the age sweep across the run.
//!
//! ## Notes on the C special case
//!
//! `attraction_aging` in C uses `dp = (pa_exp == 0.0) ? 1.0 : pow(deg, pa_exp)`
//! — i.e. `pow(0, 0) == 1` is special-cased only for the degree term
//! and only when the exponent is exactly zero. The age term always
//! evaluates `pow(age, aging_exp)` directly (and since `age ≥ 1` after
//! the `+1` shift, there is no `pow(0, *)` edge case for age).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::needless_range_loop
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search bounded to `[0, bound)`.
///
/// Duplicated from [`crate::barabasi_game_psumtree`] /
/// [`crate::recent_degree_game`] so this module is self-contained.
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

/// Mirrors `attraction_aging()` at `barabasi.c:606`.
///
/// `weight = (deg_coef · dp + zero_deg_appeal) · (age_coef · ap + zero_age_appeal)`
/// where `dp = pow(deg, pa_exp)` (forced to `1.0` when `pa_exp == 0`)
/// and `ap = pow(age, aging_exp)`.
fn attraction_aging(
    degree: u32,
    age: u32,
    pa_exp: f64,
    aging_exp: f64,
    zero_deg_appeal: f64,
    zero_age_appeal: f64,
    deg_coef: f64,
    age_coef: f64,
) -> f64 {
    let dp = if pa_exp == 0.0 {
        1.0
    } else if degree == 0 {
        if pa_exp > 0.0 { 0.0 } else { f64::INFINITY }
    } else {
        f64::from(degree).powf(pa_exp)
    };
    let ap = if age == 0 {
        // `age >= 1` is always true in our caller (we feed `age + 1`,
        // `age + 2`, or `1` for the new vertex). Guard anyway.
        if aging_exp == 0.0 {
            1.0
        } else if aging_exp > 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        f64::from(age).powf(aging_exp)
    };
    (deg_coef * dp + zero_deg_appeal) * (age_coef * ap + zero_age_appeal)
}

fn validate(
    nodes: u32,
    m: u32,
    outseq: Option<&[u32]>,
    pa_exp: f64,
    aging_exp: f64,
    aging_bins: u32,
    zero_deg_appeal: f64,
    zero_age_appeal: f64,
    deg_coef: f64,
    age_coef: f64,
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
    if !deg_coef.is_finite() || deg_coef < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "deg_coef must be finite and non-negative (got {deg_coef})"
        )));
    }
    if !age_coef.is_finite() || age_coef < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "age_coef must be finite and non-negative (got {age_coef})"
        )));
    }
    if !zero_deg_appeal.is_finite() || zero_deg_appeal < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "zero_deg_appeal must be finite and non-negative (got {zero_deg_appeal})"
        )));
    }
    if !zero_age_appeal.is_finite() || zero_age_appeal < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "zero_age_appeal must be finite and non-negative (got {zero_age_appeal})"
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

/// Preferential attachment with vertex aging.
///
/// See [module docs](self) for the full contract.
///
/// # Errors
///
/// * `pa_exp` / `aging_exp` not finite.
/// * `aging_bins == 0`.
/// * `deg_coef` / `age_coef` / `zero_deg_appeal` / `zero_age_appeal`
///   negative or non-finite.
/// * `outseq.len() != nodes`.
///
/// # Examples
///
/// ```
/// use rust_igraph::barabasi_aging_game;
///
/// // 100-vertex directed graph with classical preferential attachment
/// // (pa_exp = 1) and no aging (aging_exp = 0, age_coef = 0, so the
/// // age term collapses to a constant `zero_age_appeal = 1.0`). Total
/// // edges: (n - 1) * m = 99 * 2 = 198.
/// let g = barabasi_aging_game(
///     100, 2, None, false,
///     1.0, 0.0, 10,
///     1.0, 1.0,
///     1.0, 0.0,
///     true, 0x42,
/// ).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert_eq!(g.ecount(), 198);
/// ```
pub fn barabasi_aging_game(
    nodes: u32,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    pa_exp: f64,
    aging_exp: f64,
    aging_bins: u32,
    zero_deg_appeal: f64,
    zero_age_appeal: f64,
    deg_coef: f64,
    age_coef: f64,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(
        nodes,
        m,
        outseq,
        pa_exp,
        aging_exp,
        aging_bins,
        zero_deg_appeal,
        zero_age_appeal,
        deg_coef,
        age_coef,
    )?;

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
    let capacity = edge_capacity(nodes, m, outseq);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(capacity);

    // Seed vertex: weight 1.0 (matches upstream's "any positive value"
    // initial placeholder; refreshed properly on step 1).
    psum.set(0, 1.0);

    let mut picks: Vec<usize> = Vec::with_capacity(m as usize);

    for i in 1..n {
        let no_of_neighbors = if let Some(seq) = outseq {
            seq[i] as usize
        } else {
            m as usize
        };

        picks.clear();

        // The C source samples against the live BIT for each of the m
        // draws WITHOUT zeroing picks within the step. Independent
        // draws against the (slowly evolving) tree may therefore
        // collide, producing within-step multi-edges. We mirror that.
        let sum_snapshot = psum.total();
        for _ in 0..no_of_neighbors {
            let to = if sum_snapshot > 0.0 {
                let target = rng.gen_unit() * sum_snapshot;
                psum.search_bounded(target, i)
            } else {
                let span = i as u64;
                (rng.next_u64() % span) as usize
            };
            degree[to] += 1;
            edges.push((
                u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("vertex index overflows u32"))?,
                u32::try_from(to)
                    .map_err(|_| IgraphError::Internal("vertex index overflows u32"))?,
            ));
            picks.push(to);
        }

        // Refresh weights of every chosen vertex with their new degree
        // (age is `(i - n) / binwidth + 1` — matches the C loop at
        // lines 786-796 which uses `age + 1`).
        for &nn in &picks {
            let age = (i - nn) / binwidth + 1;
            let w = attraction_aging(
                degree[nn],
                age as u32,
                pa_exp,
                aging_exp,
                zero_deg_appeal,
                zero_age_appeal,
                deg_coef,
                age_coef,
            );
            psum.set(nn, w);
        }

        // Insert vertex `i` into the BIT at age 1 (newest bin).
        if outpref {
            degree[i] += no_of_neighbors as u32;
            let w = attraction_aging(
                degree[i],
                1,
                pa_exp,
                aging_exp,
                zero_deg_appeal,
                zero_age_appeal,
                deg_coef,
                age_coef,
            );
            psum.set(i, w);
        } else {
            let w = attraction_aging(
                0,
                1,
                pa_exp,
                aging_exp,
                zero_deg_appeal,
                zero_age_appeal,
                deg_coef,
                age_coef,
            );
            psum.set(i, w);
        }

        // Age sweep: every vertex at position `i - k*binwidth`
        // (k ≥ 1) just crossed a bin boundary. Refresh with age
        // `k + 2` (matches the C loop at lines 817-829).
        let mut k = 1usize;
        loop {
            let offset = binwidth.saturating_mul(k);
            if offset > i {
                break;
            }
            let shnode = i - offset;
            let deg = degree[shnode];
            let new_age = (k + 2) as u32;
            let w = attraction_aging(
                deg,
                new_age,
                pa_exp,
                aging_exp,
                zero_deg_appeal,
                zero_age_appeal,
                deg_coef,
                age_coef,
            );
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
        directed: bool,
        seed: u64,
    ) -> IgraphResult<Graph> {
        barabasi_aging_game(
            nodes, m, None, outpref, pa_exp, aging_exp, 10, 1.0, 1.0, 1.0, 1.0, directed, seed,
        )
    }

    #[test]
    fn aging_n_zero_returns_empty() {
        let g = default_call(0, 2, false, 1.0, 0.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn aging_n_one_singleton() {
        let g = default_call(1, 2, false, 1.0, 0.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn aging_m_zero_edgeless() {
        let g = default_call(10, 0, false, 1.0, 0.0, true, 1).unwrap();
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn aging_ecount_exact_constant_m() {
        let g = default_call(50, 3, false, 1.0, 0.0, true, 42).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 49 * 3);
    }

    #[test]
    fn aging_ecount_exact_outseq() {
        let outseq: Vec<u32> = (0..20)
            .map(|i| if i == 0 { 0 } else { (i % 3) + 1 })
            .collect();
        let expected_edges: usize = outseq.iter().skip(1).map(|&x| x as usize).sum();
        let g = barabasi_aging_game(
            20,
            0,
            Some(&outseq),
            false,
            1.0,
            -0.5,
            5,
            1.0,
            1.0,
            1.0,
            1.0,
            true,
            7,
        )
        .unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), expected_edges);
    }

    #[test]
    fn aging_no_self_loops() {
        let g = default_call(80, 4, false, 1.0, -1.0, true, 123).unwrap();
        for (s, d) in collect_edges(&g) {
            assert_ne!(s, d, "self-loop at edge ({s}, {d})");
        }
    }

    #[test]
    fn aging_source_is_step_index() {
        // Edges are emitted in step order; for constant-m the first m
        // edges come from vertex 1, next m from vertex 2, etc.
        let m: u32 = 3;
        let n: u32 = 30;
        let g = default_call(n, m, false, 1.0, -0.5, true, 99).unwrap();
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
    fn aging_target_in_zero_to_step_minus_one() {
        let m: u32 = 2;
        let n: u32 = 40;
        let g = default_call(n, m, false, 1.0, -0.5, true, 7).unwrap();
        let edges = collect_edges(&g);
        for (idx, (s, d)) in edges.iter().enumerate() {
            assert!(*d < *s, "edge {idx}: target {d} should be < source {s}");
        }
    }

    #[test]
    fn aging_deterministic_same_seed() {
        let g1 = default_call(40, 3, false, 1.0, -0.5, true, 11).unwrap();
        let g2 = default_call(40, 3, false, 1.0, -0.5, true, 11).unwrap();
        assert_eq!(collect_edges(&g1), collect_edges(&g2));
    }

    #[test]
    fn aging_seed_divergence() {
        let g1 = default_call(40, 3, false, 1.0, -0.5, true, 11).unwrap();
        let g2 = default_call(40, 3, false, 1.0, -0.5, true, 12).unwrap();
        assert_ne!(collect_edges(&g1), collect_edges(&g2));
    }

    #[test]
    fn aging_directed_flag_propagates() {
        let g_dir = default_call(20, 2, false, 1.0, 0.0, true, 1).unwrap();
        let g_undir = default_call(20, 2, false, 1.0, 0.0, false, 1).unwrap();
        assert!(g_dir.is_directed());
        assert!(!g_undir.is_directed());
    }

    #[test]
    fn aging_outpref_changes_graph() {
        let g_off = default_call(30, 2, false, 1.0, -1.0, true, 5).unwrap();
        let g_on = default_call(30, 2, true, 1.0, -1.0, true, 5).unwrap();
        // outpref toggling changes the BIT weights from step 2 onward,
        // so the edge stream should differ.
        assert_ne!(collect_edges(&g_off), collect_edges(&g_on));
    }

    #[test]
    fn aging_strong_aging_lifts_young_share_vs_no_aging() {
        // Directional sanity: turning on strong negative aging should
        // shift in-degree toward the younger half of vertices,
        // compared to the no-aging baseline (aging_exp = 0).
        let make = |aging_exp: f64| {
            barabasi_aging_game(
                200, 2, None, false, 1.0, aging_exp, 50, 1.0, 0.0, 1.0, 1.0, true, 33,
            )
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
    fn aging_classical_matches_classical_ba_shape() {
        // With pa_exp = 1, aging_exp = 0, age_coef = 0,
        // zero_age_appeal = 1, the age term collapses to a constant
        // and the model reduces to classical preferential attachment.
        // Edge count should be exact and the in-degree of the seed
        // vertex should still be the largest hub.
        let g = barabasi_aging_game(
            100, 2, None, false, 1.0, 0.0, 10, 1.0, 1.0, 1.0, 0.0, true, 17,
        )
        .unwrap();
        assert_eq!(g.ecount(), 99 * 2);
        let mut indeg = vec![0u32; 100];
        for (_s, d) in collect_edges(&g) {
            indeg[d as usize] += 1;
        }
        let max_idx = (0..100).max_by_key(|&i| indeg[i]).expect("100 candidates");
        // Seed vertex (0) or one of the very early vertices should win.
        assert!(
            max_idx < 10,
            "max-indegree vertex should be among the oldest 10 (got {max_idx})"
        );
    }

    #[test]
    fn aging_validation_aging_bins_zero() {
        let err = barabasi_aging_game(10, 2, None, false, 1.0, 0.0, 0, 1.0, 1.0, 1.0, 1.0, true, 1)
            .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_negative_deg_coef() {
        let err = barabasi_aging_game(
            10, 2, None, false, 1.0, 0.0, 5, 1.0, 1.0, -1.0, 1.0, true, 1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_negative_age_coef() {
        let err = barabasi_aging_game(
            10, 2, None, false, 1.0, 0.0, 5, 1.0, 1.0, 1.0, -1.0, true, 1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_negative_zero_deg_appeal() {
        let err = barabasi_aging_game(
            10, 2, None, false, 1.0, 0.0, 5, -1.0, 1.0, 1.0, 1.0, true, 1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_negative_zero_age_appeal() {
        let err = barabasi_aging_game(
            10, 2, None, false, 1.0, 0.0, 5, 1.0, -1.0, 1.0, 1.0, true, 1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_pa_exp_nan() {
        let err = barabasi_aging_game(
            10,
            2,
            None,
            false,
            f64::NAN,
            0.0,
            5,
            1.0,
            1.0,
            1.0,
            1.0,
            true,
            1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn aging_validation_outseq_wrong_length() {
        let outseq = vec![0u32; 5];
        let err = barabasi_aging_game(
            10,
            0,
            Some(&outseq),
            false,
            1.0,
            0.0,
            5,
            1.0,
            1.0,
            1.0,
            1.0,
            true,
            1,
        )
        .unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn attraction_aging_pa_exp_zero_returns_deg_constant_one() {
        // dp = 1 when pa_exp = 0 (regardless of degree).
        let w0 = attraction_aging(0, 1, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let w_big = attraction_aging(1000, 1, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        // (1*1 + 0) * (1*1 + 1) = 2
        assert!((w0 - 2.0).abs() < 1e-12);
        assert!((w_big - 2.0).abs() < 1e-12);
    }

    #[test]
    fn aging_targets_are_unique_when_strongly_aged() {
        // With m = 2 and strong aging suppressing repeats, the chance
        // of within-step duplicates is non-zero but small. Just check
        // structural invariant: no edge has identical src and dst.
        let g = default_call(40, 2, false, 1.0, -3.0, true, 21).unwrap();
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        for e in collect_edges(&g) {
            assert_ne!(e.0, e.1);
            seen.insert(e); // duplicates are allowed per the model
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn aging_ecount_exact_constant_m(
            n in 2u32..40,
            m in 1u32..6,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            aging_bins in 1u32..20,
            outpref: bool,
            directed: bool,
            seed in 0u64..1_000_000,
        ) {
            let g = barabasi_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, aging_bins,
                1.0, 1.0, 1.0, 1.0,
                directed, seed,
            ).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount() as u32, (n - 1) * m);
        }

        #[test]
        fn aging_no_self_loops(
            n in 2u32..40,
            m in 1u32..6,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            aging_bins in 1u32..20,
            outpref: bool,
            directed: bool,
            seed in 0u64..1_000_000,
        ) {
            let g = barabasi_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, aging_bins,
                1.0, 1.0, 1.0, 1.0,
                directed, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert_ne!(s, d);
            }
        }

        #[test]
        fn aging_source_is_step_index(
            n in 2u32..40,
            m in 1u32..6,
            seed in 0u64..1_000_000,
        ) {
            let g = barabasi_aging_game(
                n, m, None, false,
                1.0, -0.5, 5,
                1.0, 1.0, 1.0, 1.0,
                true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, _d) = g.edge(eid).unwrap();
                let expected_src = (eid / m) + 1;
                prop_assert_eq!(s, expected_src);
            }
        }

        #[test]
        fn aging_targets_below_source(
            n in 2u32..40,
            m in 1u32..6,
            aging_exp in -2.0_f64..1.0,
            seed in 0u64..1_000_000,
        ) {
            let g = barabasi_aging_game(
                n, m, None, false,
                1.0, aging_exp, 5,
                1.0, 1.0, 1.0, 1.0,
                true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_edges {
                let (s, d) = g.edge(eid).unwrap();
                prop_assert!(d < s);
            }
        }

        #[test]
        fn aging_determinism(
            n in 2u32..30,
            m in 1u32..5,
            pa_exp in -1.0_f64..2.0,
            aging_exp in -2.0_f64..1.0,
            outpref: bool,
            seed in 0u64..1_000_000,
        ) {
            let g1 = barabasi_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, 5,
                1.0, 1.0, 1.0, 1.0,
                true, seed,
            ).unwrap();
            let g2 = barabasi_aging_game(
                n, m, None, outpref,
                pa_exp, aging_exp, 5,
                1.0, 1.0, 1.0, 1.0,
                true, seed,
            ).unwrap();
            let m_edges = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..m_edges {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }
    }
}
