//! Configuration-model **simple-graph** degree-sequence generator
//! (ALGO-GN-027).
//!
//! Counterpart of the `IGRAPH_DEGSEQ_CONFIGURATION_SIMPLE` branch of
//! `igraph_degree_sequence_game()` in
//! `references/igraph/src/games/degree_sequence.c`
//! (`configuration_simple_undirected` lines 552-594,
//! `configuration_simple_directed` lines 597-708).
//!
//! Like the multigraph configuration model (ALGO-GN-024) the algorithm
//! builds a flat **stub bag** of size `Σd` and pairs stubs by an
//! incremental Fisher–Yates shuffle. The key difference is that any
//! self-loop or multi-edge produced during the inner shuffle aborts the
//! attempt: the adjacency tracker is cleared and the entire attempt is
//! restarted. The resulting distribution is exactly uniform on the set
//! of simple realisations of the prescribed sequence — at the cost of an
//! unbounded expected restart count for sequences that are hard to
//! realise simply.
//!
//! ## Undirected branch
//!
//! Build `stubs = [v repeated d_v times]` of length `Σd`. Then for each
//! `i ∈ [0, |E|)`:
//!
//! 1. Pick `k ← RNG(2i, |stubs|−1)`, swap `stubs[2i]` with `stubs[k]`.
//! 2. Pick `k ← RNG(2i+1, |stubs|−1)`, swap `stubs[2i+1]` with `stubs[k]`.
//! 3. Let `from = stubs[2i]`, `to = stubs[2i+1]`. If `from == to`
//!    (self-loop) **or** the pair is already adjacent, fail the attempt,
//!    clear adjacency, restart the inner loop. Otherwise record the
//!    edge in the adjacency tracker and continue.
//!
//! This mirrors the C `configuration_simple_undirected_set` /
//! `configuration_simple_undirected_bitset` helpers; we use a single
//! `Vec<HashSet<u32>>` backend, which is what `_set` uses internally
//! (the `_bitset` variant in C is only a memory-saving alternative for
//! `vcount ≤ 1024` and offers no algorithmic advantage).
//!
//! ## Directed branch
//!
//! Build `out_stubs` and `in_stubs` separately. Only `out_stubs` is
//! shuffled (one FY swap per edge) — the in-bag is consumed in
//! construction order, so `to = in_stubs[i]` is monotonically
//! non-decreasing. This allows an `O(1)` multi-arc test via a
//! `vertex_done_mark` counter trick: whenever `to` advances, bump the
//! mark; a duplicate source is detected when `vertex_done[from] ==
//! current_mark`.
//!
//! ## vs. siblings
//!
//! * [`crate::degree_sequence_game_configuration`] (ALGO-GN-024) — no
//!   simplicity check; faster but produces a multigraph.
//! * [`crate::degree_sequence_game_fast_heur_simple`] (ALGO-GN-026) —
//!   biased single-pass heuristic with back-off-and-continue; 21–54×
//!   faster but NOT uniform on the simple-graph space.
//! * [`crate::degree_sequence_game_vl`] (ALGO-GN-025) — Viger–Latapy
//!   uniform sampler on the **simple connected** subspace; trades the
//!   connectivity guarantee for an MCMC mixing phase.
//!
//! ## Determinism
//!
//! A single [`SplitMix64`] seed drives every shuffle and every retry.
//! The PRNG is not bitwise portable to igraph C / `NumPy` / R, so the
//! three-source conformance harness asserts structural invariants only
//! (vcount, ecount = Σd/2 or Σout, exact out/in degree match,
//! simplicity).
//!
//! ## Failure modes
//!
//! Non-graphical input is rejected up front by Erdős–Gallai (undirected)
//! or Fulkerson–Chen–Anstee (directed). For sequences that pass the
//! graphicality test but happen to be hostile to the rejection sampler
//! (very dense, near-regular sequences), the attempt-restart counter is
//! bounded by [`MAX_OUTER_ATTEMPTS`]; the function returns
//! `InvalidArgument` once exhausted.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop
)]

use std::collections::HashSet;

use crate::algorithms::games::degree_sequence_fast_heur::{
    checked_sum, is_graphical_simple_directed, is_graphical_simple_undirected,
};
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Cap on attempt restarts before giving up.
pub const MAX_OUTER_ATTEMPTS: u32 = 1024;

/// Sample `k` uniformly from `[low, high]` inclusive. Mirrors the C
/// reference's `RNG_INTEGER(low, high)` semantics.
fn rng_integer_inclusive(rng: &mut SplitMix64, low: usize, high: usize) -> usize {
    debug_assert!(low <= high, "rng_integer_inclusive: low ≤ high");
    let span = (high - low) as u64 + 1;
    low + (rng.next_u64() % span) as usize
}

fn build_stubs(degrees: &[u32], total: usize) -> Vec<u32> {
    let mut stubs: Vec<u32> = Vec::with_capacity(total);
    for (i, &d) in degrees.iter().enumerate() {
        let v = i as u32;
        for _ in 0..d {
            stubs.push(v);
        }
    }
    stubs
}

fn run_undirected(
    degrees: &[u32],
    n: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let stub_count_u64 = checked_sum(degrees)?;
    if stub_count_u64 % 2 != 0 {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_configuration_simple: undirected degree sum must be even"
                .to_string(),
        ));
    }
    let stub_count = stub_count_u64 as usize;
    let ecount = stub_count / 2;
    if ecount == 0 {
        return Ok(Vec::new());
    }

    let mut stubs = build_stubs(degrees, stub_count);
    let vcount = n as usize;
    let mut adjacency: Vec<HashSet<u32>> = (0..vcount)
        .map(|i| HashSet::with_capacity(degrees[i] as usize))
        .collect();

    for _attempt in 0..MAX_OUTER_ATTEMPTS {
        let mut success = true;

        for i in 0..ecount {
            // Two independent Fisher–Yates picks in the un-consumed tail.
            let k1 = rng_integer_inclusive(rng, 2 * i, stub_count - 1);
            stubs.swap(2 * i, k1);
            let k2 = rng_integer_inclusive(rng, 2 * i + 1, stub_count - 1);
            stubs.swap(2 * i + 1, k2);

            let from = stubs[2 * i];
            let to = stubs[2 * i + 1];

            if from == to {
                success = false;
                break;
            }
            if adjacency[to as usize].contains(&from) {
                success = false;
                break;
            }
            adjacency[to as usize].insert(from);
            adjacency[from as usize].insert(to);
        }

        if success {
            let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
            for i in 0..ecount {
                edges.push((stubs[2 * i], stubs[2 * i + 1]));
            }
            return Ok(edges);
        }

        for set in &mut adjacency {
            set.clear();
        }
    }

    Err(IgraphError::InvalidArgument(format!(
        "degree_sequence_game_configuration_simple: exhausted {MAX_OUTER_ATTEMPTS} rejection-sampling restarts; the sequence is graphical but very hostile to rejection sampling — try degree_sequence_game_fast_heur_simple or degree_sequence_game_vl",
    )))
}

fn run_directed(
    out_degrees: &[u32],
    in_degrees: &[u32],
    n: u32,
    rng: &mut SplitMix64,
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let out_total = checked_sum(out_degrees)? as usize;
    let in_total = checked_sum(in_degrees)? as usize;
    if out_total != in_total {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_configuration_simple: directed sums Σout and Σin must match"
                .to_string(),
        ));
    }
    let ecount = out_total;
    if ecount == 0 {
        return Ok(Vec::new());
    }

    let mut out_stubs = build_stubs(out_degrees, ecount);
    let in_stubs = build_stubs(in_degrees, ecount);
    let vcount = n as usize;

    // Per-source mark: bumped whenever `to` advances. A source is a
    // duplicate of the current target iff its stored mark equals the
    // current `vertex_done_mark`. This avoids clearing the array between
    // targets — clear via a fresh mark instead.
    let mut vertex_done: Vec<u64> = vec![0u64; vcount];
    let mut vertex_done_mark: u64 = 0;

    for _attempt in 0..MAX_OUTER_ATTEMPTS {
        let mut success = true;
        let mut previous_to: i64 = -1;

        for i in 0..ecount {
            let k = rng_integer_inclusive(rng, i, ecount - 1);
            out_stubs.swap(i, k);

            let from = out_stubs[i];
            let to = in_stubs[i];

            if to == from {
                success = false;
                break;
            }

            if i64::from(to) != previous_to {
                vertex_done_mark = vertex_done_mark.wrapping_add(1);
                previous_to = i64::from(to);
            }

            if vertex_done[from as usize] == vertex_done_mark {
                success = false;
                break;
            }
            vertex_done[from as usize] = vertex_done_mark;
        }

        if success {
            let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
            for i in 0..ecount {
                edges.push((out_stubs[i], in_stubs[i]));
            }
            return Ok(edges);
        }
        // No need to clear `vertex_done` between attempts: the mark is
        // bumped per target, and the wrap-on-overflow guarantee makes
        // false positives statistically negligible until 2^64 marks
        // (which we cannot reach within MAX_OUTER_ATTEMPTS · ecount).
    }

    Err(IgraphError::InvalidArgument(format!(
        "degree_sequence_game_configuration_simple: exhausted {MAX_OUTER_ATTEMPTS} rejection-sampling restarts; the directed sequence pair is graphical but very hostile to rejection sampling",
    )))
}

/// Uniform random simple graph realising the given degree sequence via
/// the configuration-model rejection sampler (ALGO-GN-027).
///
/// Returns a [`Graph`] guaranteed to be simple (no self-loops, no
/// multi-edges / multi-arcs) and to exactly match the prescribed
/// out-degrees (and in-degrees in the directed case). Connectivity is
/// *not* guaranteed — use [`crate::degree_sequence_game_vl`] for
/// uniform-connected sampling.
///
/// # Arguments
///
/// * `out_degrees` — for undirected mode, the degree of each vertex;
///   for directed mode, the out-degree of each vertex.
/// * `in_degrees`:
///   * `None` → undirected graph; requires `Σ out_degrees` to be even
///     and the sequence to satisfy Erdős–Gallai.
///   * `Some(in_seq)` → directed graph; requires `in_seq.len() ==
///     out_degrees.len()`, `Σ in_seq == Σ out_degrees`, and the pair to
///     satisfy Fulkerson–Chen–Anstee.
/// * `seed` — drives a [`SplitMix64`] PRNG; the same `(out_degrees,
///   in_degrees, seed)` triple always produces the same graph.
///
/// # Errors
///
/// Returns `IgraphError::InvalidArgument` if the input sequence is
/// non-graphical or if the rejection sampler exhausts
/// [`MAX_OUTER_ATTEMPTS`] without finding a simple realisation.
///
/// # Examples
///
/// ```
/// use rust_igraph::degree_sequence_game_configuration_simple;
///
/// // 4-cycle: every vertex degree 2 ⇒ 4 simple edges.
/// let g = degree_sequence_game_configuration_simple(&[2, 2, 2, 2], None, 17).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// assert!(!g.is_directed());
/// ```
pub fn degree_sequence_game_configuration_simple(
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
                "degree_sequence_game_configuration_simple: out_degrees and in_degrees must have the same length".to_string(),
            ));
        }
    }

    if directed {
        let in_seq = in_degrees.expect("directed branch implies Some(in_degrees)");
        if !is_graphical_simple_directed(out_degrees, in_seq) {
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_configuration_simple: degree pair is not realisable as a simple directed graph (Fulkerson–Chen–Anstee)"
                    .to_string(),
            ));
        }
    } else if !is_graphical_simple_undirected(out_degrees) {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_configuration_simple: degree sequence is not realisable as a simple undirected graph (Erdős–Gallai)"
                .to_string(),
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
        let g = degree_sequence_game_configuration_simple(&[], None, 1).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn undirected_singleton_zero_yields_isolated_vertex() {
        let g = degree_sequence_game_configuration_simple(&[0], None, 1).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_all_isolated_n5_yields_no_edges() {
        let g = degree_sequence_game_configuration_simple(&[0; 5], None, 42).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_4cycle_preserves_degrees_and_is_simple() {
        let g = degree_sequence_game_configuration_simple(&[2, 2, 2, 2], None, 7).expect("ok");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(observed_degrees(&g), vec![2, 2, 2, 2]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_3regular_n6_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 6];
        let g = degree_sequence_game_configuration_simple(&degrees, None, 0xABCD_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_skewed_powerlaw_preserves_degrees_sorted() {
        let degrees: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let g = degree_sequence_game_configuration_simple(&degrees, None, 0xC0FE_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_3regular_n30_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 30];
        let g =
            degree_sequence_game_configuration_simple(&degrees, None, 0xDEAD_F00D_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_odd_sum_rejected() {
        let err = degree_sequence_game_configuration_simple(&[1, 1, 1], None, 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn undirected_negative_eg_rejected_max_too_large() {
        // n=4, max degree 5 > n-1=3 — EG fails at k=1.
        let err = degree_sequence_game_configuration_simple(&[5, 3, 1, 1], None, 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn deterministic_same_seed_undirected() {
        let degrees: Vec<u32> = vec![3; 8];
        let g1 = degree_sequence_game_configuration_simple(&degrees, None, 4242).expect("ok");
        let g2 = degree_sequence_game_configuration_simple(&degrees, None, 4242).expect("ok");
        let mut e1: Vec<(u32, u32)> = (0..u32::try_from(g1.ecount()).unwrap())
            .map(|i| {
                let (a, b) = g1.edge(i).unwrap();
                if a < b { (a, b) } else { (b, a) }
            })
            .collect();
        let mut e2: Vec<(u32, u32)> = (0..u32::try_from(g2.ecount()).unwrap())
            .map(|i| {
                let (a, b) = g2.edge(i).unwrap();
                if a < b { (a, b) } else { (b, a) }
            })
            .collect();
        e1.sort_unstable();
        e2.sort_unstable();
        assert_eq!(e1, e2);
    }

    #[test]
    fn directed_empty_sequence_yields_empty_graph() {
        let g = degree_sequence_game_configuration_simple(&[], Some(&[]), 1).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn directed_2cycle_preserves_in_out() {
        let g = degree_sequence_game_configuration_simple(&[1, 1], Some(&[1, 1]), 9).expect("ok");
        let (out, inv) = observed_out_in(&g);
        assert_eq!(out, vec![1, 1]);
        assert_eq!(inv, vec![1, 1]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_balanced_n6_d2_preserves_degrees() {
        let n = 6;
        let out = vec![2u32; n];
        let inv = vec![2u32; n];
        let g =
            degree_sequence_game_configuration_simple(&out, Some(&inv), 0xC0DE_u64).expect("ok");
        let (got_out, got_in) = observed_out_in(&g);
        assert_eq!(got_out, out);
        assert_eq!(got_in, inv);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_unequal_sums_rejected() {
        let err =
            degree_sequence_game_configuration_simple(&[1, 1, 1], Some(&[1, 1, 0]), 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn directed_length_mismatch_rejected() {
        let err =
            degree_sequence_game_configuration_simple(&[1, 1], Some(&[1, 1, 0]), 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn directed_deterministic_same_seed() {
        let out = vec![2u32; 5];
        let inv = vec![2u32; 5];
        let g1 = degree_sequence_game_configuration_simple(&out, Some(&inv), 12345).expect("ok");
        let g2 = degree_sequence_game_configuration_simple(&out, Some(&inv), 12345).expect("ok");
        let e1: Vec<(u32, u32)> = (0..u32::try_from(g1.ecount()).unwrap())
            .map(|i| g1.edge(i).unwrap())
            .collect();
        let e2: Vec<(u32, u32)> = (0..u32::try_from(g2.ecount()).unwrap())
            .map(|i| g2.edge(i).unwrap())
            .collect();
        assert_eq!(e1, e2);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    // Restrict the strategy to *moderate-density* graphical sequences. The
    // configuration_simple rejection sampler is exact-uniform but its
    // expected attempt count grows roughly as `exp(O((Σd/n)²))`, so dense
    // near-regular sequences blow past MAX_OUTER_ATTEMPTS = 1024 in
    // practice. The dense regime is exactly what the fast-heur (GN-026)
    // and VL (GN-025) siblings cover; the proptest fixtures here check
    // the property contract in the regime where this sampler is the
    // right tool.
    fn graphical_undirected_strategy() -> impl Strategy<Value = Vec<u32>> {
        (4usize..=8).prop_flat_map(|n| {
            let cap = ((n as u32) / 2).max(1);
            prop::collection::vec(0u32..=cap, n).prop_filter(
                "must be graphical (even sum + EG)",
                move |seq| {
                    let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
                    sum % 2 == 0 && is_graphical_simple_undirected(seq)
                },
            )
        })
    }

    proptest! {
        #[test]
        fn degrees_preserved_undirected(seq in graphical_undirected_strategy(), seed in any::<u64>()) {
            let g = degree_sequence_game_configuration_simple(&seq, None, seed)
                .expect("graphical sequence must succeed");
            let n = g.vcount() as usize;
            let mut deg = vec![0u32; n];
            let ec = u32::try_from(g.ecount()).unwrap();
            for eid in 0..ec {
                let (s, t) = g.edge(eid).unwrap();
                deg[s as usize] += 1;
                deg[t as usize] += 1;
            }
            prop_assert_eq!(deg, seq);
        }

        #[test]
        fn simple_no_loops_no_multi_undirected(seq in graphical_undirected_strategy(), seed in any::<u64>()) {
            use crate::algorithms::properties::{SimpleMode, is_simple_with_mode};
            let g = degree_sequence_game_configuration_simple(&seq, None, seed)
                .expect("graphical sequence must succeed");
            prop_assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
        }

        #[test]
        fn same_seed_same_graph(seq in graphical_undirected_strategy(), seed in any::<u64>()) {
            let g1 = degree_sequence_game_configuration_simple(&seq, None, seed).unwrap();
            let g2 = degree_sequence_game_configuration_simple(&seq, None, seed).unwrap();
            let e1: Vec<(u32, u32)> = (0..u32::try_from(g1.ecount()).unwrap())
                .map(|i| g1.edge(i).unwrap())
                .collect();
            let e2: Vec<(u32, u32)> = (0..u32::try_from(g2.ecount()).unwrap())
                .map(|i| g2.edge(i).unwrap())
                .collect();
            prop_assert_eq!(e1, e2);
        }
    }
}
