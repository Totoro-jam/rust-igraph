//! Fast-heuristic simple-graph degree-sequence generator (ALGO-GN-026).
//!
//! Counterpart of the `IGRAPH_DEGSEQ_FAST_HEUR_SIMPLE` branch of
//! `igraph_degree_sequence_game()` in
//! `references/igraph/src/games/degree_sequence.c`
//! (`fast_heur_undirected` lines 125-261, `fast_heur_directed`
//! lines 263-402).
//!
//! The routine samples a **simple** graph (no self-loops, no multi-edges)
//! with a prescribed degree sequence using a one-pass stub-matching
//! heuristic with single-cycle backtracking:
//!
//! 1. Build a stub bag from the residual degree sequence and shuffle it.
//! 2. Walk pairs `(stubs[2i], stubs[2i+1])`; accept the edge if it would
//!    not create a self-loop and the endpoint pair is not yet incident.
//!    Otherwise bump residual degrees on both endpoints and mark them as
//!    "incomplete".
//! 3. If the incomplete set is empty, the attempt succeeded.
//! 4. Else, scan the incomplete vertices for a feasible non-edge pair.
//!    If at least one exists, repeat from (1) using only the residual
//!    stubs. If none exists, restart the entire attempt from scratch.
//!
//! Unlike the configuration-model generator (ALGO-GN-024), the output is
//! guaranteed to be simple. Unlike the Viger–Latapy sampler
//! (ALGO-GN-025), the result is **not** uniform on the space of simple
//! graphs realising the sequence: the bias of the heuristic is acceptable
//! for many applications but is the price for a single-pass attempt with
//! `O(Σd · log Σd)` complexity on the median attempt.
//!
//! ## Directed vs undirected
//!
//! * **Undirected** (`in_degrees = None`): one stub bag, paired as
//!   `(stubs[2i], stubs[2i+1])` after a Fisher–Yates shuffle.
//! * **Directed** (`in_degrees = Some(in_seq)`): two stub bags
//!   (out + in). Only the out-bag is shuffled; the in-bag is consumed in
//!   construction order, matching the C reference.
//!
//! ## Connectivity
//!
//! This generator does **not** enforce connectivity. For uniformly random
//! *simple connected* samples use `degree_sequence_game_vl` (ALGO-GN-025).
//!
//! ## Determinism
//!
//! A single [`SplitMix64`] seed drives every shuffle. The PRNG is not
//! bitwise portable to igraph C / `NumPy` / R, so the three-source
//! conformance harness asserts structural invariants only (vcount,
//! ecount = Σd/2, exact degree match, simplicity).
//!
//! ## Failure modes
//!
//! Non-graphical input is rejected up front. For sequences that pass the
//! graphicality test but happen to be especially hostile to the heuristic
//! (e.g. very dense, near-regular sequences with low slack), the
//! attempt-restart counter is bounded by [`MAX_OUTER_ATTEMPTS`]; the
//! function returns `InvalidArgument` once exhausted.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::manual_contains
)]

use std::collections::BTreeSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Cap on outer attempt restarts before giving up.
const MAX_OUTER_ATTEMPTS: u32 = 1024;

/// Sum a `u32` slice into a `u64` with overflow checking.
pub(crate) fn checked_sum(degrees: &[u32]) -> IgraphResult<u64> {
    let mut acc: u64 = 0;
    for &d in degrees {
        acc = acc
            .checked_add(u64::from(d))
            .ok_or(IgraphError::Internal("degree-sum overflow"))?;
    }
    Ok(acc)
}

/// Erdős–Gallai test (undirected simple graph).
pub(crate) fn is_graphical_simple_undirected(degrees: &[u32]) -> bool {
    let n = degrees.len();
    if n == 0 {
        return true;
    }
    let sum: u64 = degrees.iter().map(|&d| u64::from(d)).sum();
    if sum % 2 != 0 {
        return false;
    }
    let n_u64 = n as u64;
    if degrees.iter().any(|&d| u64::from(d) >= n_u64) {
        return false;
    }
    let mut d_desc: Vec<u32> = degrees.to_vec();
    d_desc.sort_by(|a, b| b.cmp(a));
    let mut left_sum: u64 = 0;
    for k in 1..=n {
        left_sum = left_sum.saturating_add(u64::from(d_desc[k - 1]));
        let k_u64 = k as u64;
        let bound_right: u64 = d_desc[k..].iter().map(|&d| u64::from(d).min(k_u64)).sum();
        let rhs = k_u64
            .saturating_mul(k_u64.saturating_sub(1))
            .saturating_add(bound_right);
        if left_sum > rhs {
            return false;
        }
    }
    true
}

/// Fulkerson–Chen–Anstee test for directed simple graph (no self-loops).
///
/// A pair of sequences `(a_i, b_i)` with `Σa = Σb` is realisable as a
/// simple directed graph iff, after re-ordering indices `π` so that
/// `a_{π(1)} ≥ … ≥ a_{π(n)}`, for every `k = 1..n`,
/// `Σ_{i≤k} a_{π(i)} ≤ Σ_{i≤k} min(b_{π(i)}, k − 1) + Σ_{i>k} min(b_{π(i)}, k)`.
pub(crate) fn is_graphical_simple_directed(out_degrees: &[u32], in_degrees: &[u32]) -> bool {
    let n = out_degrees.len();
    if n != in_degrees.len() {
        return false;
    }
    if n == 0 {
        return true;
    }
    let out_sum: u64 = out_degrees.iter().map(|&d| u64::from(d)).sum();
    let in_sum: u64 = in_degrees.iter().map(|&d| u64::from(d)).sum();
    if out_sum != in_sum {
        return false;
    }
    let n_u64 = n as u64;
    if out_degrees.iter().any(|&d| u64::from(d) >= n_u64) {
        return false;
    }
    if in_degrees.iter().any(|&d| u64::from(d) >= n_u64) {
        return false;
    }
    // Sort indices by out-degree descending; break ties by larger in-degree
    // (Fulkerson–Chen–Anstee uses this canonical "corrected" permutation).
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| {
        out_degrees[j]
            .cmp(&out_degrees[i])
            .then_with(|| in_degrees[j].cmp(&in_degrees[i]))
    });
    let a: Vec<u32> = idx.iter().map(|&i| out_degrees[i]).collect();
    let b: Vec<u32> = idx.iter().map(|&i| in_degrees[i]).collect();
    let mut left_sum: u64 = 0;
    for k in 1..=n {
        left_sum = left_sum.saturating_add(u64::from(a[k - 1]));
        let k_u64 = k as u64;
        // Σ_{i ≤ k} min(b_i, k-1) + Σ_{i > k} min(b_i, k)
        let cap_left: u64 = (0..k)
            .map(|i| u64::from(b[i]).min(k_u64.saturating_sub(1)))
            .sum();
        let cap_right: u64 = (k..n).map(|i| u64::from(b[i]).min(k_u64)).sum();
        let rhs = cap_left.saturating_add(cap_right);
        if left_sum > rhs {
            return false;
        }
    }
    true
}

/// Fisher–Yates shuffle in place using the supplied PRNG (range
/// `[0, bound)` from a 64-bit draw, biased-but-uniform-enough for our
/// stub-shuffle needs).
fn fisher_yates<T>(slice: &mut [T], rng: &mut SplitMix64) {
    let n = slice.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        let bound = (i + 1) as u64;
        let j = (rng.next_u64() % bound) as usize;
        slice.swap(i, j);
    }
}

/// Per-vertex adjacency stored as a sorted `Vec<u32>` of neighbours. Edge
/// membership tests are `O(log d)` via binary search; insertions preserve
/// the sort order. We index the same way the C reference does so that the
/// undirected/directed branches share code paths.
type Adj = Vec<Vec<u32>>;

fn adj_contains(neis: &[u32], target: u32) -> bool {
    neis.binary_search(&target).is_ok()
}

fn adj_insert(neis: &mut Vec<u32>, target: u32) {
    match neis.binary_search(&target) {
        Ok(_) => {} // already present (shouldn't happen if caller checked)
        Err(pos) => neis.insert(pos, target),
    }
}

/// Run the undirected fast-heuristic loop. On success returns the edge
/// list with `src ≤ dst` canonicalisation.
fn run_undirected(
    degrees: &[u32],
    n: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let nu = n as usize;
    let mut residual: Vec<u32> = degrees.to_vec();
    let mut adj: Adj = vec![Vec::new(); nu];
    let mut stubs: Vec<u32> = Vec::with_capacity(checked_sum(degrees)? as usize);
    let mut incomplete: BTreeSet<u32> = BTreeSet::new();

    for outer in 0..MAX_OUTER_ATTEMPTS {
        let _ = outer;
        adj.iter_mut().for_each(Vec::clear);
        residual.copy_from_slice(degrees);
        let mut failed = false;
        let mut finished;

        loop {
            // Build stubs from current residual_degrees.
            stubs.clear();
            for (i, &r) in residual.iter().enumerate() {
                for _ in 0..r {
                    stubs.push(i as u32);
                }
            }
            // Reset residuals to zero; they'll be re-bumped on collision.
            residual.fill(0);
            incomplete.clear();

            fisher_yates(&mut stubs, rng);

            let k = stubs.len();
            // Walk in pairs. Note: odd k would have been rejected earlier
            // (graphical implies even Σd).
            let mut i = 0;
            while i + 1 < k {
                let mut from = stubs[i];
                let mut to = stubs[i + 1];
                i += 2;
                if from > to {
                    std::mem::swap(&mut from, &mut to);
                }
                if from == to || adj_contains(&adj[from as usize], to) {
                    residual[from as usize] = residual[from as usize].saturating_add(1);
                    residual[to as usize] = residual[to as usize].saturating_add(1);
                    incomplete.insert(from);
                    incomplete.insert(to);
                } else {
                    adj_insert(&mut adj[from as usize], to);
                }
            }

            finished = incomplete.is_empty();
            if finished {
                break;
            }

            // Check feasibility: can any pair of distinct incomplete
            // vertices be connected by a non-existing edge?
            let mut feasible = false;
            let incs: Vec<u32> = incomplete.iter().copied().collect();
            'outer_check: for ia in 0..incs.len() {
                let mut from = incs[ia];
                for ib in (ia + 1)..incs.len() {
                    let mut to = incs[ib];
                    if from > to {
                        std::mem::swap(&mut from, &mut to);
                    }
                    if !adj_contains(&adj[from as usize], to) {
                        feasible = true;
                        break 'outer_check;
                    }
                    // restore (we mutated `from` via swap; reset per loop)
                    from = incs[ia];
                }
            }
            if !feasible {
                failed = true;
                break;
            }
            // Else loop again: rebuild stubs from current residuals.
        }

        if finished {
            // Materialise sorted edge list.
            let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(nu * 2);
            for (u_idx, neis) in adj.iter().enumerate() {
                let u = u_idx as u32;
                for &v in neis {
                    edges.push((u, v));
                }
            }
            return Ok(edges);
        }
        let _ = failed;
        // restart outer
    }

    Err(IgraphError::InvalidArgument(
        "degree_sequence_game_fast_heur_simple: heuristic exhausted attempt cap; sequence appears to be hostile to the fast-heuristic; try the VL sampler".to_string(),
    ))
}

/// Run the directed fast-heuristic loop. On success returns the directed
/// edge list `(src, dst)` in adjacency-scan order (src ascending, dst
/// ascending within each src).
fn run_directed(
    out_degrees: &[u32],
    in_degrees: &[u32],
    n: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let nu = n as usize;
    let mut residual_out: Vec<u32> = out_degrees.to_vec();
    let mut residual_in: Vec<u32> = in_degrees.to_vec();
    let mut adj: Adj = vec![Vec::new(); nu];
    let out_total = checked_sum(out_degrees)? as usize;
    let in_total = checked_sum(in_degrees)? as usize;
    if out_total != in_total {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_fast_heur_simple: directed mode requires Σout == Σin".to_string(),
        ));
    }
    let mut out_stubs: Vec<u32> = Vec::with_capacity(out_total);
    let mut in_stubs: Vec<u32> = Vec::with_capacity(in_total);
    let mut incomplete_out: BTreeSet<u32> = BTreeSet::new();
    let mut incomplete_in: BTreeSet<u32> = BTreeSet::new();

    for outer in 0..MAX_OUTER_ATTEMPTS {
        let _ = outer;
        adj.iter_mut().for_each(Vec::clear);
        residual_out.copy_from_slice(out_degrees);
        residual_in.copy_from_slice(in_degrees);
        let mut failed = false;
        let mut finished;

        loop {
            out_stubs.clear();
            in_stubs.clear();
            for (i, &r) in residual_out.iter().enumerate() {
                for _ in 0..r {
                    out_stubs.push(i as u32);
                }
            }
            for (i, &r) in residual_in.iter().enumerate() {
                for _ in 0..r {
                    in_stubs.push(i as u32);
                }
            }
            residual_out.fill(0);
            residual_in.fill(0);
            incomplete_out.clear();
            incomplete_in.clear();

            fisher_yates(&mut out_stubs, rng);

            let k = out_stubs.len();
            for i in 0..k {
                let from = out_stubs[i];
                let to = in_stubs[i];
                if from == to || adj_contains(&adj[from as usize], to) {
                    residual_out[from as usize] = residual_out[from as usize].saturating_add(1);
                    residual_in[to as usize] = residual_in[to as usize].saturating_add(1);
                    incomplete_out.insert(from);
                    incomplete_in.insert(to);
                } else {
                    adj_insert(&mut adj[from as usize], to);
                }
            }

            finished = incomplete_out.is_empty();
            if finished {
                break;
            }
            // Feasibility test: any (from, to) with from ∈ inc_out,
            // to ∈ inc_in, from != to, and edge not yet present.
            let mut feasible = false;
            'outer_check: for &from in &incomplete_out {
                for &to in &incomplete_in {
                    if from != to && !adj_contains(&adj[from as usize], to) {
                        feasible = true;
                        break 'outer_check;
                    }
                }
            }
            if !feasible {
                failed = true;
                break;
            }
        }

        if finished {
            let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(out_total);
            for (u_idx, neis) in adj.iter().enumerate() {
                let u = u_idx as u32;
                for &v in neis {
                    edges.push((u, v));
                }
            }
            return Ok(edges);
        }
        let _ = failed;
    }

    Err(IgraphError::InvalidArgument(
        "degree_sequence_game_fast_heur_simple: heuristic exhausted attempt cap (directed); sequence appears to be hostile to the fast-heuristic".to_string(),
    ))
}

/// Sample a simple graph realising the given degree sequence(s) via the
/// fast-heuristic single-pass + restart algorithm.
///
/// * `out_degrees` — undirected degree of every vertex, or out-degree of
///   every vertex when `in_degrees = Some(...)`. Vertex count is the
///   slice length.
/// * `in_degrees` — when `Some(seq)`, switches to directed mode with
///   `seq[i]` as the in-degree of vertex `i`; must equal `out_degrees`
///   in length and sum.
/// * `seed` — drives the internal [`SplitMix64`] PRNG.
///
/// Output is a simple graph (no self-loops, no multi-edges). The sampled
/// graph is *not* uniformly distributed on the space of simple
/// realisations — see [`degree_sequence_game_vl`] for a uniform sampler.
///
/// # Errors
///
/// * `n > u32::MAX`.
/// * `in_degrees.len() != out_degrees.len()`.
/// * Undirected mode with a non-graphical sequence (Erdős–Gallai fails).
/// * Directed mode with `Σ out ≠ Σ in` or a non-graphical pair
///   (Fulkerson–Chen–Anstee fails).
/// * Heuristic exhausted [`MAX_OUTER_ATTEMPTS`] restarts.
///
/// [`degree_sequence_game_vl`]: crate::degree_sequence_game_vl
///
/// # Examples
///
/// ```
/// use rust_igraph::degree_sequence_game_fast_heur_simple;
///
/// // 4-cycle: every vertex degree 2 ⇒ 4 simple edges.
/// let g = degree_sequence_game_fast_heur_simple(&[2, 2, 2, 2], None, 7).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// assert!(!g.is_directed());
/// ```
pub fn degree_sequence_game_fast_heur_simple(
    out_degrees: &[u32],
    in_degrees: Option<&[u32]>,
    seed: u64,
) -> IgraphResult<Graph> {
    let directed = in_degrees.is_some();
    let n = u32::try_from(out_degrees.len())
        .map_err(|_| IgraphError::Internal("vertex count exceeds u32"))?;
    if let Some(in_seq) = in_degrees {
        if in_seq.len() != out_degrees.len() {
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_fast_heur_simple: out_degrees and in_degrees must have the same length".to_string(),
            ));
        }
    }

    // Graphicality checks.
    if directed {
        let in_seq = in_degrees.expect("directed branch implies Some(in_degrees)");
        if !is_graphical_simple_directed(out_degrees, in_seq) {
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_fast_heur_simple: degree pair is not realisable as a simple directed graph (Fulkerson–Chen–Anstee)"
                    .to_string(),
            ));
        }
    } else if !is_graphical_simple_undirected(out_degrees) {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_fast_heur_simple: degree sequence is not realisable as a simple undirected graph (Erdős–Gallai)".to_string(),
        ));
    }

    let mut rng = SplitMix64::new(seed);

    let edges = if directed {
        let in_seq = in_degrees.expect("directed branch implies Some(in_degrees)");
        run_directed(out_degrees, in_seq, n, &mut rng)?
    } else {
        run_undirected(out_degrees, n, &mut rng)?
    };

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::properties::{SimpleMode, is_simple_with_mode};

    fn observed_degrees(g: &Graph) -> Vec<u32> {
        let n = g.vcount() as usize;
        let mut deg = vec![0u32; n];
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32");
        for eid in 0..ec {
            let (s, t) = g.edge(eid).expect("eid in bounds");
            deg[s as usize] = deg[s as usize].saturating_add(1);
            deg[t as usize] = deg[t as usize].saturating_add(1);
        }
        deg
    }

    fn observed_out_in(g: &Graph) -> (Vec<u32>, Vec<u32>) {
        let n = g.vcount() as usize;
        let mut out = vec![0u32; n];
        let mut inv = vec![0u32; n];
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32");
        for eid in 0..ec {
            let (s, t) = g.edge(eid).expect("eid in bounds");
            out[s as usize] = out[s as usize].saturating_add(1);
            inv[t as usize] = inv[t as usize].saturating_add(1);
        }
        (out, inv)
    }

    #[test]
    fn undirected_empty_sequence_yields_empty_graph() {
        let g = degree_sequence_game_fast_heur_simple(&[], None, 1).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn undirected_singleton_zero_yields_isolated_vertex() {
        let g = degree_sequence_game_fast_heur_simple(&[0], None, 1).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_4cycle_preserves_degrees_and_is_simple() {
        let g = degree_sequence_game_fast_heur_simple(&[2, 2, 2, 2], None, 7).expect("ok");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(observed_degrees(&g), vec![2, 2, 2, 2]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_3regular_n6_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 6];
        let g = degree_sequence_game_fast_heur_simple(&degrees, None, 0xABCD_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_complete_k4_preserves_degrees() {
        let g = degree_sequence_game_fast_heur_simple(&[3, 3, 3, 3], None, 42).expect("ok");
        assert_eq!(g.ecount(), 6);
        assert_eq!(observed_degrees(&g), vec![3, 3, 3, 3]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_odd_sum_rejected() {
        let err = degree_sequence_game_fast_heur_simple(&[1, 1, 1], None, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("Erdős–Gallai")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn undirected_degree_exceeds_n_minus_1_rejected() {
        // n=4, max simple-graph degree = n-1 = 3.
        // [5,3,1,1] has Σ=10 (even) but vertex 0's degree 5 > 3.
        // EG, k=1: 5 ≤ 0 + min(3,1)+min(1,1)+min(1,1) = 3 → fails.
        let err = degree_sequence_game_fast_heur_simple(&[5, 3, 1, 1], None, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("Erdős–Gallai")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn undirected_non_graphical_rejected() {
        // [3,3,3,1] has sum 10 (even), max=3, n=4. Sorted desc: [3,3,3,1].
        // k=1: 3 ≤ 0 + min(3,1)+min(3,1)+min(1,1) = 1+1+1 = 3, ok.
        // k=2: 6 ≤ 2 + min(3,2)+min(1,2) = 2 + 3 = 5. FAILS.
        let err = degree_sequence_game_fast_heur_simple(&[3, 3, 3, 1], None, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("Erdős–Gallai")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn undirected_skewed_sequence_preserves_degrees() {
        let degrees: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let g = degree_sequence_game_fast_heur_simple(&degrees, None, 123).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_same_seed_same_graph() {
        let degrees: Vec<u32> = vec![3, 3, 3, 3, 2, 2];
        let a = degree_sequence_game_fast_heur_simple(&degrees, None, 9).expect("a");
        let b = degree_sequence_game_fast_heur_simple(&degrees, None, 9).expect("b");
        let ea: Vec<(u32, u32)> = (0..a.ecount() as u32)
            .map(|e| a.edge(e).expect("eid"))
            .collect();
        let eb: Vec<(u32, u32)> = (0..b.ecount() as u32)
            .map(|e| b.edge(e).expect("eid"))
            .collect();
        assert_eq!(ea, eb);
    }

    #[test]
    fn undirected_distinct_seeds_likely_differ() {
        let degrees: Vec<u32> = vec![3; 10];
        let mut a_edges: Vec<(u32, u32)> = (0..)
            .take(
                degree_sequence_game_fast_heur_simple(&degrees, None, 1)
                    .expect("a")
                    .ecount() as usize,
            )
            .map(|_| (0u32, 0u32))
            .collect();
        let g_a = degree_sequence_game_fast_heur_simple(&degrees, None, 1).expect("a");
        for e in 0..g_a.ecount() as u32 {
            a_edges[e as usize] = g_a.edge(e).expect("eid");
        }
        let g_b = degree_sequence_game_fast_heur_simple(&degrees, None, 2).expect("b");
        let b_edges: Vec<(u32, u32)> = (0..g_b.ecount() as u32)
            .map(|e| g_b.edge(e).expect("eid"))
            .collect();
        // They might match by chance but with 3-regular n=10 the chance is vanishing.
        assert_ne!(a_edges, b_edges);
    }

    // --- directed ---

    #[test]
    fn directed_balanced_d1_n4_cycle() {
        // out=[1,1,1,1], in=[1,1,1,1] ⇒ one 4-cycle realisation up to permutation.
        let g = degree_sequence_game_fast_heur_simple(&[1, 1, 1, 1], Some(&[1, 1, 1, 1]), 1)
            .expect("ok");
        assert!(g.is_directed());
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        let (out, inv) = observed_out_in(&g);
        assert_eq!(out, vec![1, 1, 1, 1]);
        assert_eq!(inv, vec![1, 1, 1, 1]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_skewed_n5_preserves_both_degrees() {
        let out_seq: Vec<u32> = vec![2, 2, 2, 1, 1];
        let in_seq: Vec<u32> = vec![1, 1, 2, 2, 2];
        let g = degree_sequence_game_fast_heur_simple(&out_seq, Some(&in_seq), 0x5EED).expect("ok");
        let (out, inv) = observed_out_in(&g);
        assert_eq!(out, out_seq);
        assert_eq!(inv, in_seq);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_sum_mismatch_rejected() {
        let err =
            degree_sequence_game_fast_heur_simple(&[1, 1, 1], Some(&[1, 1, 0]), 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => {
                assert!(msg.contains("Fulkerson") || msg.contains("Σout"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn directed_length_mismatch_rejected() {
        let err = degree_sequence_game_fast_heur_simple(&[1, 1, 1], Some(&[1, 1]), 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("length")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn directed_self_loops_avoided_when_residuals_permit() {
        // Force the only realisation to be 0→1, 1→0 (no self-loops).
        let g = degree_sequence_game_fast_heur_simple(&[1, 1], Some(&[1, 1]), 1).expect("ok");
        assert_eq!(g.ecount(), 2);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_3regular_n5_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 5];
        let g =
            degree_sequence_game_fast_heur_simple(&degrees, Some(&degrees), 0xC0DE).expect("ok");
        let (out, inv) = observed_out_in(&g);
        assert_eq!(out, degrees);
        assert_eq!(inv, degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use crate::algorithms::properties::{SimpleMode, is_simple_with_mode};
    use proptest::prelude::*;

    fn observed_degrees(g: &Graph) -> Vec<u32> {
        let n = g.vcount() as usize;
        let mut deg = vec![0u32; n];
        let ec = u32::try_from(g.ecount()).expect("ecount fits u32");
        for eid in 0..ec {
            let (s, t) = g.edge(eid).expect("eid in bounds");
            deg[s as usize] = deg[s as usize].saturating_add(1);
            deg[t as usize] = deg[t as usize].saturating_add(1);
        }
        deg
    }

    fn arb_undirected_degseq() -> impl Strategy<Value = Vec<u32>> {
        (3usize..=10).prop_flat_map(|n| prop::collection::vec(0u32..(n as u32), n))
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        #[test]
        fn degrees_preserved_when_graphical(seq in arb_undirected_degseq(), seed in any::<u64>()) {
            let result = degree_sequence_game_fast_heur_simple(&seq, None, seed);
            match result {
                Ok(g) => {
                    prop_assert_eq!(observed_degrees(&g), seq);
                    prop_assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
                }
                Err(IgraphError::InvalidArgument(msg)) => {
                    // Acceptable only when truly non-graphical or the heuristic capped out.
                    prop_assert!(
                        msg.contains("Erdős–Gallai") || msg.contains("attempt cap"),
                        "unexpected rejection: {}",
                        msg
                    );
                }
                Err(other) => prop_assert!(false, "unexpected error: {:?}", other),
            }
        }

        #[test]
        fn same_seed_same_graph(seq in arb_undirected_degseq(), seed in any::<u64>()) {
            let a = degree_sequence_game_fast_heur_simple(&seq, None, seed);
            let b = degree_sequence_game_fast_heur_simple(&seq, None, seed);
            match (a, b) {
                (Ok(ga), Ok(gb)) => {
                    let ea: Vec<(u32, u32)> = (0..ga.ecount() as u32)
                        .map(|e| ga.edge(e).expect("eid"))
                        .collect();
                    let eb: Vec<(u32, u32)> = (0..gb.ecount() as u32)
                        .map(|e| gb.edge(e).expect("eid"))
                        .collect();
                    prop_assert_eq!(ea, eb);
                }
                (Err(_), Err(_)) => {}
                (a, b) => prop_assert!(false, "determinism violated: {:?} vs {:?}", a, b),
            }
        }
    }
}
