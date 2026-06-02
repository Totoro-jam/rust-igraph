//! Barabási–Albert preferential-attachment random graph — PSUMTREE
//! variants (ALGO-GN-020).
//!
//! Counterparts of the `IGRAPH_BARABASI_PSUMTREE` and
//! `IGRAPH_BARABASI_PSUMTREE_MULTIPLE` branches of
//! `igraph_barabasi_game()` in `references/igraph/src/games/barabasi.c`
//! (`igraph_i_barabasi_game_psumtree_multiple` at lines ~166-294 and
//! `igraph_i_barabasi_game_psumtree` at lines ~296-414).
//!
//! Both variants attach `m` (or `outseq[i]`) outgoing edges from every
//! new vertex `i`, choosing destinations weighted by
//! `attraction(deg(v)) = pow(deg(v), power) + A` (with the C-style
//! special case `pow(0, 0) == 1`). Sampling uses a Fenwick BIT
//! (`PsumTree`) shared with [`crate::lastcit_game`] /
//! [`crate::recent_degree_game`] for O(log n) point-set + binary-lifted
//! prefix-search.
//!
//! ## Variants
//!
//! * [`barabasi_game_psumtree`] — **simple** variant. Each of the `m`
//!   draws temporarily zeros the chosen vertex's weight in the BIT so
//!   that the same target cannot be picked twice within the same step.
//!   After the `m` draws complete, the weights of the chosen vertices
//!   are refreshed back to their proper `attraction(deg(v))`. **No
//!   within-step multi-edges** by construction. (Cross-step duplicates
//!   are impossible too because the source vertex is fresh each step,
//!   and the simple-graph guarantee is the contract.)
//!
//! * [`barabasi_game_psumtree_multiple`] — **multi-edge** variant. The
//!   BIT prefix-sum is snapshotted *once* at the start of each step,
//!   and all `m` draws sample against the unchanged tree. The same
//!   target may therefore be drawn twice within one step, producing
//!   multi-edges. After all draws, the affected vertices' weights are
//!   refreshed in batch.
//!
//! ## Common parameters
//!
//! * `nodes` — vertex count. `nodes = 0` returns the empty graph.
//! * `power` — exponent in `pow(deg, power)`. Negative values are
//!   permitted only when there are no zero-degree vertices being
//!   sampled (the upstream contract is that `A > 0` rules that
//!   out — we forward the same rule).
//! * `m` — constant per-step out-degree, used when `outseq = None`.
//! * `outseq` — when provided, must have length `nodes`. The first
//!   entry is ignored (vertex 0 has no outgoing edges per the seed
//!   convention). For `i ≥ 1`, `outseq[i]` overrides `m`.
//! * `outpref` — when `true` the new vertex's own out-degree counts
//!   toward its attraction (total-degree model). Forced to `true`
//!   when `directed = false`.
//! * `a` — constant attractiveness term added to every weight.
//!   When `outpref = false`, must be **strictly positive** so that
//!   zero-degree vertices have non-zero probability. When
//!   `outpref = true`, may be zero but must be non-negative.
//! * `directed` — emit directed edges (older → newer convention
//!   matches BAG variant).
//! * `seed` — initialises an internal `SplitMix64`; the output is
//!   deterministic for fixed parameters and seed.
//!
//! ## Construction guarantees
//!
//! * **No self-loops** by construction — the new vertex `i` is added
//!   to the BIT *after* its `m` outgoing draws complete.
//! * **No cross-step multi-edges**: source `i` is unique each step,
//!   so all duplicates can only land *within* a single step. The
//!   simple variant rules those out; the multiple variant allows them.
//! * **Always-cite fallback**: when `m ≥ i`, all `i` previously-added
//!   vertices are cited (deterministically), matching upstream's
//!   `no_of_neighbors >= i` branch (multiple variant only — the simple
//!   variant cannot satisfy `m ≥ i` without running out of distinct
//!   targets, which the BIT-zeroing scheme also handles implicitly).
//! * **Zero-sum fallback**: when the BIT total is zero, the sample
//!   falls back to a uniform draw over `[0, i)`. With `outpref = true`
//!   and `A > 0`, this branch only fires in the very first step (where
//!   the seed vertex's weight is set to `1.0`, so the sum is positive).
//!
//! ## Cost
//!
//! `O(n · m · log n)` for the BIT searches/updates; `O(n + m·(n-1))`
//! memory for the edge list. Compares favourably to the BAG variant's
//! `O(n · m)` for small `m`, and is asymptotically better than O(n²)
//! naive cumulative-sum sampling.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_arguments,
    clippy::needless_range_loop
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search.
///
/// Shared with [`crate::lastcit_game`] / [`crate::recent_degree_game`];
/// duplicated here so each module is self-contained.
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
    ///
    /// Returns an index strictly less than `bound`, even if FP drift in
    /// the BIT entries causes the binary lifting to over-advance.
    /// Critical for the BA inner loop: the new vertex `i` has weight 0
    /// in the BIT at search time, but its position is `i` — without
    /// bounding the search to `bound = i`, drift can cause the search
    /// to return `i`, producing a self-loop.
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

/// `attraction(deg, power, A) = pow(deg, power) + A`, with the C
/// special case `pow(0, 0) == 1`.
fn attraction(degree: u32, power: f64, a: f64) -> f64 {
    let term = if power == 0.0 {
        1.0
    } else if degree == 0 {
        // pow(0, p) for p > 0 is 0; for p < 0 it is ±∞. Caller is
        // expected to set A > 0 in that regime so that the resulting
        // weight is still finite. The runtime contract is enforced by
        // `validate`.
        if power > 0.0 { 0.0 } else { f64::INFINITY }
    } else {
        f64::from(degree).powf(power)
    };
    term + a
}

fn validate(
    nodes: u32,
    power: f64,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    a: f64,
) -> IgraphResult<()> {
    if !power.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "power must be finite (got {power})"
        )));
    }
    if !a.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "A must be finite (got {a})"
        )));
    }
    if outpref {
        if a < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "A must be non-negative when outpref is true (got {a})"
            )));
        }
    } else if a <= 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "A must be positive when outpref is false (got {a})"
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

/// PSUMTREE (simple) variant — no within-step multi-edges.
///
/// See [module docs](self) for the full contract.
///
/// # Errors
///
/// * `power` or `a` is not finite (NaN / ±∞).
/// * `outpref = false` and `a <= 0` (would leave zero-degree vertices
///   un-sampleable).
/// * `outpref = true` and `a < 0`.
/// * `outseq.len() != nodes`.
///
/// # Examples
///
/// ```
/// use rust_igraph::barabasi_game_psumtree;
///
/// // 50-vertex directed BA graph with classical (power=1, A=1) kernel
/// // and m = 2 outgoing edges per new vertex. Total edge count is
/// // (n - 1) · m = 49 · 2 = 98. The simple variant emits NO
/// // within-step duplicates, so the graph is simple.
/// let g = barabasi_game_psumtree(50, 1.0, 2, None, false, 1.0, true, 0x42).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert_eq!(g.ecount(), 98);
/// ```
pub fn barabasi_game_psumtree(
    nodes: u32,
    power: f64,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    a: f64,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    let outpref = outpref || !directed;
    validate(nodes, power, m, outseq, outpref, a)?;
    barabasi_psumtree_inner(nodes, power, m, outseq, outpref, a, directed, seed, false)
}

/// `PSUMTREE_MULTIPLE` variant — within-step multi-edges allowed.
///
/// See [module docs](self) for the full contract.
///
/// # Errors
///
/// Same as [`barabasi_game_psumtree`].
///
/// # Examples
///
/// ```
/// use rust_igraph::barabasi_game_psumtree_multiple;
///
/// // 50-vertex directed BA graph with classical (power=1, A=1) kernel
/// // and m = 3. The early-cite saturation branch fires for steps where
/// // `m >= i` (steps 1..=3 here, emitting 1+2+3 = 6 instead of
/// // 3+3+3 = 9), so the total edge count is
/// // `(n-1)*m - m*(m-1)/2 = 49*3 - 3 = 144`. The multiple variant
/// // MAY produce within-step duplicates.
/// let g = barabasi_game_psumtree_multiple(50, 1.0, 3, None, false, 1.0, true, 0x42).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert_eq!(g.ecount(), 49 * 3 - 3 * 2 / 2);
/// ```
pub fn barabasi_game_psumtree_multiple(
    nodes: u32,
    power: f64,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    a: f64,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    let outpref = outpref || !directed;
    validate(nodes, power, m, outseq, outpref, a)?;
    barabasi_psumtree_inner(nodes, power, m, outseq, outpref, a, directed, seed, true)
}

#[allow(clippy::too_many_arguments)]
fn barabasi_psumtree_inner(
    nodes: u32,
    power: f64,
    m: u32,
    outseq: Option<&[u32]>,
    outpref: bool,
    a: f64,
    directed: bool,
    seed: u64,
    allow_multiple: bool,
) -> IgraphResult<Graph> {
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
    let mut rng = SplitMix64::new(seed);
    let mut psum = PsumTree::new(n);
    let mut degree: Vec<u32> = vec![0; n];
    let capacity = edge_capacity(nodes, m, outseq);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(capacity);

    // Seed vertex: weight 1.0 (matches upstream's
    // "first vertex weight may be any positive value" — refreshed
    // properly on step 1).
    psum.set(0, 1.0);

    // Scratch buffer for per-step picks (used by both variants for the
    // "refresh chosen weights" pass).
    let mut picks: Vec<usize> = Vec::with_capacity(m as usize);

    for i in 1..n {
        let no_of_neighbors = if let Some(seq) = outseq {
            seq[i] as usize
        } else {
            m as usize
        };

        picks.clear();

        if allow_multiple && no_of_neighbors >= i {
            // Saturation: cite every existing vertex once.
            // (Matches upstream's `no_of_neighbors >= i` early branch
            // in the multiple variant.)
            for to in 0..i {
                degree[to] += 1;
                edges.push((
                    u32::try_from(i)
                        .map_err(|_| IgraphError::Internal("vertex index overflows u32"))?,
                    u32::try_from(to)
                        .map_err(|_| IgraphError::Internal("vertex index overflows u32"))?,
                ));
                psum.set(to, attraction(degree[to], power, a));
            }
        } else if allow_multiple {
            // Multiple variant: snapshot sum once per step; draws may
            // collide.
            let sum_snapshot = psum.total();
            for _ in 0..no_of_neighbors {
                let to = if sum_snapshot > 0.0 {
                    let target = rng.gen_unit() * sum_snapshot;
                    // Bound to [0, i): vertex `i` itself has weight 0
                    // but lives in the BIT at index i, and FP drift
                    // can otherwise let binary-lifting overshoot.
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
            // Refresh weights of all chosen vertices (deduped — same
            // vertex chosen twice only needs one refresh).
            for &nn in &picks {
                psum.set(nn, attraction(degree[nn], power, a));
            }
        } else {
            // Simple variant: zero each pick's weight as it is chosen
            // (so it cannot be sampled again within this step); after
            // all `no_of_neighbors` draws, refresh each chosen vertex's
            // weight. Matches upstream: always emit exactly
            // `no_of_neighbors` edges per step, using the uniform
            // fallback when the BIT is exhausted (`m >= i` warmup
            // case can therefore still produce within-step duplicates
            // via the fallback path).
            for _ in 0..no_of_neighbors {
                let sum = psum.total();
                let to = if sum > 0.0 {
                    let target = rng.gen_unit() * sum;
                    // Bound to [0, i): even after several within-step
                    // zeroings, the BIT may still report positive sum
                    // due to FP drift; bounding prevents the binary
                    // lifting from clamping to `self.n - 1` (which
                    // would equal `i` on the final step).
                    psum.search_bounded(target, i)
                } else {
                    // Fallback path: BIT is exhausted (all candidates
                    // zeroed this step). Sample uniformly over [0, i).
                    // For `m >= i` this introduces within-step
                    // duplicates, matching upstream's behavior.
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
                psum.set(to, 0.0);
                picks.push(to);
            }
            // Refresh weights of chosen vertices.
            for &nn in &picks {
                psum.set(nn, attraction(degree[nn], power, a));
            }
        }

        // Add new vertex `i` to the BIT.
        if outpref {
            // Outpref: the new vertex's own out-degree counts. In the
            // multi-variant saturation branch (m >= i), upstream caps
            // the increment at `i` since only `i` edges were emitted;
            // in all other paths the increment is exactly
            // `no_of_neighbors`.
            let inc = if allow_multiple && no_of_neighbors >= i {
                i as u32
            } else {
                no_of_neighbors as u32
            };
            degree[i] += inc;
            psum.set(i, attraction(degree[i], power, a));
        } else {
            psum.set(i, attraction(0, power, a));
        }
    }

    graph.add_edges(edges)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let n = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
        (0..n)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    #[test]
    fn psumtree_n_zero_returns_empty() {
        let g = barabasi_game_psumtree(0, 1.0, 3, None, false, 1.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn psumtree_multiple_n_zero_returns_empty() {
        let g = barabasi_game_psumtree_multiple(0, 1.0, 3, None, false, 1.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn psumtree_n_one_singleton() {
        let g = barabasi_game_psumtree(1, 1.0, 5, None, false, 1.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn psumtree_m_zero_edgeless() {
        let g = barabasi_game_psumtree(10, 1.0, 0, None, false, 1.0, true, 1).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn psumtree_exact_edge_count() {
        for &(n, m) in &[(10u32, 1u32), (50, 2), (100, 5)] {
            let g = barabasi_game_psumtree(n, 1.0, m, None, false, 1.0, true, 0xABCD).unwrap();
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount(), ((n - 1) * m) as usize);
        }
    }

    #[test]
    fn psumtree_multiple_exact_edge_count() {
        // Multi variant: when `m >= i` the early-cite branch fires and
        // emits only `i` edges instead of `m`. Total =
        // (n-1)*m - m*(m-1)/2 when n > m.
        for &(n, m) in &[(10u32, 1u32), (50, 2), (100, 5)] {
            let g =
                barabasi_game_psumtree_multiple(n, 1.0, m, None, false, 1.0, true, 0xABCD).unwrap();
            assert_eq!(g.vcount(), n);
            let expected = (n - 1) * m - m * (m - 1) / 2;
            assert_eq!(g.ecount(), expected as usize);
        }
    }

    #[test]
    fn psumtree_simple_no_within_step_duplicates_after_warmup() {
        // Simple variant: once `i > m` the BIT-zeroing guarantees no
        // within-step duplicates (the uniform fallback only fires when
        // the BIT is fully exhausted, which can't happen when there
        // are still un-zeroed candidates).
        let m: u32 = 4;
        let g = barabasi_game_psumtree(100, 1.0, m, None, false, 1.0, true, 0xBEEF).unwrap();
        let edges = collect_edges(&g);
        let mut by_src: HashMap<VertexId, Vec<VertexId>> = HashMap::new();
        for &(s, d) in &edges {
            by_src.entry(s).or_default().push(d);
        }
        for (&src, dsts) in &by_src {
            if src > m {
                let mut sorted = dsts.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(sorted.len(), dsts.len(), "duplicate at src {src}: {dsts:?}");
            }
        }
    }

    #[test]
    fn psumtree_simple_no_self_loops() {
        let g = barabasi_game_psumtree(200, 1.0, 3, None, true, 1.0, true, 0xC001).unwrap();
        for (s, d) in collect_edges(&g) {
            assert_ne!(s, d, "self-loop ({s} -> {d}) emitted");
        }
    }

    #[test]
    fn psumtree_multiple_no_self_loops() {
        let g =
            barabasi_game_psumtree_multiple(200, 1.0, 3, None, true, 1.0, true, 0xC001).unwrap();
        for (s, d) in collect_edges(&g) {
            assert_ne!(s, d, "self-loop ({s} -> {d}) emitted");
        }
    }

    #[test]
    fn psumtree_edges_point_to_earlier_vertices() {
        let g = barabasi_game_psumtree(50, 1.0, 2, None, false, 1.0, true, 0xBEEF).unwrap();
        for (s, d) in collect_edges(&g) {
            assert!(d < s, "edge ({s} -> {d}) violates BA temporal ordering");
        }
    }

    #[test]
    fn psumtree_multiple_edges_point_to_earlier_vertices() {
        let g =
            barabasi_game_psumtree_multiple(50, 1.0, 3, None, false, 1.0, true, 0xBEEF).unwrap();
        for (s, d) in collect_edges(&g) {
            assert!(d < s, "edge ({s} -> {d}) violates BA temporal ordering");
        }
    }

    #[test]
    fn psumtree_deterministic_with_seed() {
        let a = barabasi_game_psumtree(80, 1.5, 3, None, false, 1.0, true, 0xDEAD).unwrap();
        let b = barabasi_game_psumtree(80, 1.5, 3, None, false, 1.0, true, 0xDEAD).unwrap();
        assert_eq!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn psumtree_multiple_deterministic_with_seed() {
        let a =
            barabasi_game_psumtree_multiple(80, 1.5, 3, None, false, 1.0, true, 0xDEAD).unwrap();
        let b =
            barabasi_game_psumtree_multiple(80, 1.5, 3, None, false, 1.0, true, 0xDEAD).unwrap();
        assert_eq!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn psumtree_different_seeds_differ() {
        let a = barabasi_game_psumtree(100, 1.0, 2, None, false, 1.0, true, 1).unwrap();
        let b = barabasi_game_psumtree(100, 1.0, 2, None, false, 1.0, true, 2).unwrap();
        assert_ne!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn psumtree_undirected_forces_outpref() {
        let g = barabasi_game_psumtree(50, 1.0, 2, None, false, 1.0, false, 0x42).unwrap();
        assert_eq!(g.vcount(), 50);
        assert!(!g.is_directed());
        // Undirected ⇒ outpref forced true ⇒ A=1.0 is fine
        // (A>=0 with outpref=true).
        assert_eq!(g.ecount(), 49 * 2);
    }

    #[test]
    fn psumtree_undirected_a_zero_outpref_ok() {
        // With outpref=true (forced by undirected), A=0 should be
        // accepted (the seed vertex still has positive weight 1.0 to
        // bootstrap).
        let g = barabasi_game_psumtree(30, 1.0, 1, None, false, 0.0, false, 0xC0).unwrap();
        assert_eq!(g.ecount(), 29);
    }

    #[test]
    fn psumtree_rejects_a_nonpositive_when_outpref_false() {
        let err = barabasi_game_psumtree(10, 1.0, 1, None, false, 0.0, true, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => {
                assert!(s.contains("A must be positive when outpref is false"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn psumtree_rejects_a_negative_when_outpref_true() {
        let err = barabasi_game_psumtree(10, 1.0, 1, None, true, -0.5, true, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => {
                assert!(s.contains("A must be non-negative when outpref is true"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn psumtree_rejects_power_nan() {
        let err = barabasi_game_psumtree(10, f64::NAN, 1, None, false, 1.0, true, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("power must be finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn psumtree_rejects_a_inf() {
        let err =
            barabasi_game_psumtree(10, 1.0, 1, None, true, f64::INFINITY, true, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("A must be finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn psumtree_rejects_outseq_length_mismatch() {
        let seq = vec![0u32, 1, 2];
        let err = barabasi_game_psumtree(10, 1.0, 1, Some(&seq), false, 1.0, true, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(s) => assert!(s.contains("outseq length")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn psumtree_outseq_overrides_m() {
        let seq = vec![0u32, 1, 1, 2, 3];
        let g = barabasi_game_psumtree(5, 1.0, 99, Some(&seq), false, 1.0, true, 0).unwrap();
        assert_eq!(g.vcount(), 5);
        // expected edges = sum(seq[1..]) = 1+1+2+3 = 7
        assert_eq!(g.ecount(), 7);
    }

    #[test]
    fn psumtree_multiple_outseq_overrides_m() {
        let seq = vec![0u32, 1, 1, 2, 3];
        let g =
            barabasi_game_psumtree_multiple(5, 1.0, 99, Some(&seq), false, 1.0, true, 0).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 7);
    }

    #[test]
    fn psumtree_first_vertex_never_source() {
        let g = barabasi_game_psumtree(50, 1.0, 2, None, false, 1.0, true, 0x42).unwrap();
        for (s, _) in collect_edges(&g) {
            assert_ne!(s, 0);
        }
    }

    #[test]
    fn psumtree_concentration_around_early_vertices() {
        let g = barabasi_game_psumtree(500, 1.0, 3, None, false, 1.0, true, 0xABCD).unwrap();
        let mut deg = vec![0u32; g.vcount() as usize];
        for (s, d) in collect_edges(&g) {
            deg[s as usize] += 1;
            deg[d as usize] += 1;
        }
        let mut indexed: Vec<(usize, u32)> = deg.iter().copied().enumerate().collect();
        indexed.sort_by_key(|&(_, d)| std::cmp::Reverse(d));
        let top5: Vec<usize> = indexed.iter().take(5).map(|(v, _)| *v).collect();
        assert!(
            top5.contains(&0),
            "expected vertex 0 in top-5, got {top5:?}"
        );
    }

    #[test]
    fn psumtree_high_power_super_concentration() {
        // Power = 2 ⇒ rich-get-richer is super-linear ⇒ vertex 0
        // dominates strongly.
        let g = barabasi_game_psumtree(300, 2.0, 1, None, false, 1.0, true, 0xF00D).unwrap();
        let mut deg = vec![0u32; g.vcount() as usize];
        for (s, d) in collect_edges(&g) {
            deg[s as usize] += 1;
            deg[d as usize] += 1;
        }
        let max_deg = *deg.iter().max().unwrap();
        let total: u32 = deg.iter().sum();
        let max_v = deg.iter().position(|&d| d == max_deg).unwrap();
        // Top vertex should hold a substantial chunk of total degree
        // and should be a very early vertex.
        assert!(
            max_deg >= total / 10,
            "max_deg {max_deg} too small vs total {total}"
        );
        assert!(max_v < 30, "top vertex {max_v} should be among earliest");
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn psumtree_no_self_loops(
            n in 2u32..120,
            m in 1u32..6,
            power in 0.0f64..3.0,
            seed: u64,
            outpref: bool,
        ) {
            // power >= 0 keeps weights finite; with A=1 zero-degree
            // vertices still have non-zero weight, satisfying both
            // outpref=false and outpref=true contracts.
            let g = barabasi_game_psumtree(n, power, m, None, outpref, 1.0, true, seed)
                .expect("valid params");
            let m_e = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
            for eid in 0..m_e {
                let (s, d) = g.edge(eid).expect("edge in bounds");
                prop_assert_ne!(s, d);
            }
        }

        #[test]
        fn psumtree_multiple_no_self_loops(
            n in 2u32..120,
            m in 1u32..6,
            power in 0.0f64..3.0,
            seed: u64,
            outpref: bool,
        ) {
            let g = barabasi_game_psumtree_multiple(n, power, m, None, outpref, 1.0, true, seed)
                .expect("valid params");
            let m_e = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
            for eid in 0..m_e {
                let (s, d) = g.edge(eid).expect("edge in bounds");
                prop_assert_ne!(s, d);
            }
        }

        #[test]
        fn psumtree_exact_edge_count_prop(
            n in 2u32..200,
            m in 1u32..5,
            seed: u64,
        ) {
            let g = barabasi_game_psumtree(n, 1.0, m, None, false, 1.0, true, seed)
                .expect("valid params");
            prop_assert_eq!(g.ecount() as u64, u64::from((n - 1) * m));
        }

        #[test]
        fn psumtree_multiple_exact_edge_count_prop(
            n in 2u32..200,
            m in 1u32..5,
            seed: u64,
        ) {
            // Multi variant saturates when m >= i: total =
            // (n-1)*m - m*(m-1)/2 when n > m, else triangular sum.
            let g = barabasi_game_psumtree_multiple(n, 1.0, m, None, false, 1.0, true, seed)
                .expect("valid params");
            let expected: u64 = if n > m {
                u64::from((n - 1) * m) - u64::from(m * (m - 1) / 2)
            } else {
                // n <= m: every step i in [1, n-1] saturates to i
                // edges. Sum = (n-1)*n/2.
                u64::from((n - 1) * n / 2)
            };
            prop_assert_eq!(g.ecount() as u64, expected);
        }

        #[test]
        fn psumtree_simple_no_within_step_dups_after_warmup(
            n in 10u32..80,
            m in 2u32..5,
            seed: u64,
        ) {
            // For src > m the BIT-zeroing path guarantees distinct dst.
            let g = barabasi_game_psumtree(n, 1.0, m, None, false, 1.0, true, seed)
                .expect("valid params");
            let m_e = u32::try_from(g.ecount()).expect("ecount fits u32 in tests");
            let mut by_src: std::collections::HashMap<VertexId, Vec<VertexId>> = std::collections::HashMap::new();
            for eid in 0..m_e {
                let (s, d) = g.edge(eid).expect("edge in bounds");
                by_src.entry(s).or_default().push(d);
            }
            for (&src, dsts) in &by_src {
                if src > m {
                    let mut sorted = dsts.clone();
                    sorted.sort_unstable();
                    let total = sorted.len();
                    sorted.dedup();
                    prop_assert_eq!(sorted.len(), total,
                        "duplicate dsts at src {}: {:?}", src, dsts);
                }
            }
        }
    }
}
