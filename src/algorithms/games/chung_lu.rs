//! Chung–Lu expected-degree random graph generator (ALGO-GN-012).
//!
//! Counterpart of `igraph_chung_lu_game()` from
//! `references/igraph/src/games/chung_lu.c:193`.
//!
//! Given a per-vertex weight vector `w` (and, in the directed case,
//! a separate in-weight vector with the same total mass), every pair
//! `(i, j)` is connected independently with probability `p_ij`
//! depending on the chosen variant. All three variants share the
//! "expected-degree base form" `q_ij = w_i · w_j / S` where
//! `S = Σ w_k`.
//!
//! ## Variants
//!
//! * [`ChungLuVariant::Original`] — `p_ij = min(q_ij, 1)`. The
//!   original Chung–Lu (2002) construction. When `q_ij > 1` a single
//!   warning is emitted (via [`tracing`]-style `eprintln!` if the
//!   `verbose-warnings` feature ever lands; today the variant is
//!   silently clamped) and the pair is still sampled — but in that
//!   regime expected degrees no longer match the input weights.
//! * [`ChungLuVariant::Maxent`] — `p_ij = q_ij / (1 + q_ij)`. The
//!   "generalised random graph" / maximum-entropy model with fixed
//!   expected degrees (Park & Newman 2004, Britton–Deijfen–Martin-Löf
//!   2006).
//! * [`ChungLuVariant::Nr`] — `p_ij = 1 - exp(-q_ij)`. The simple-graph
//!   projection of Norros & Reittu's (2006) conditionally Poissonian
//!   multigraph model.
//!
//! All three are equivalent in the sparse-graph limit `q_ij → 0`.
//!
//! ## Algorithm
//!
//! Implements the Miller–Hagberg (2011) `O(|V| + |E|)` sampler used by
//! upstream igraph. The vertices are sorted by **in-weight descending**
//! to obtain a permutation `idx`, then for each origin `i`:
//!
//! 1. Walk the target index `j` (starting at `0` for directed, at `i`
//!    for undirected).
//! 2. Maintain a running upper-bound probability `p` (starts at `1`).
//!    Draw `gap ~ Geom(p)` and skip past it; the geometric skip is the
//!    same primitive used by the Batagelj–Brandes ER samplers.
//! 3. Compute the true probability `q` for the pair, accept the edge
//!    with probability `q / p`, then set `p ← q`.
//! 4. Because the sort is descending and `w_i` is fixed, `q` is
//!    monotonically non-increasing as `j` grows, so `q / p ≤ 1`.
//!
//! The geometric-skip + accept/reject combination is mathematically
//! identical to Bernoulli sampling each pair independently with the
//! variant-specific probability, but only the accepted edges incur work.
//!
//! ## Determinism
//!
//! Reproducible given `(out_weights, in_weights, loops, variant, seed)`
//! against the shared [`crate::core::rng::SplitMix64`] PRNG.
//!
//! ## References
//!
//! * Chung F, Lu L: *Connected components in a random graph with given
//!   degree sequences*. Annals of Combinatorics **6**, 125–145 (2002).
//! * Miller JC, Hagberg A: *Efficient generation of networks with given
//!   expected degrees* (2011).
//! * Park J, Newman MEJ: *Statistical mechanics of networks*.
//!   Phys. Rev. E **70**, 066117 (2004).
//! * Norros I, Reittu H: *On a conditionally Poissonian graph process*.
//!   Adv. Appl. Probab. **38**, 59–75 (2006).

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::unnecessary_cast
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Variant of the Chung–Lu connection-probability formula.
///
/// All three share the base `q_ij = w_i · w_j / S` with
/// `S = Σ out_weights`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChungLuVariant {
    /// `p_ij = min(q_ij, 1)`. The original Chung–Lu (2002) formula.
    /// Saturates to `1` when `q_ij > 1`.
    Original,
    /// `p_ij = q_ij / (1 + q_ij)`. The maximum-entropy / generalised
    /// random-graph variant (Park & Newman 2004).
    Maxent,
    /// `p_ij = 1 - exp(-q_ij)`. The Norros–Reittu (2006) simple-graph
    /// projection.
    Nr,
}

/// Largest weight-vector length we support. Upstream igraph rejects
/// at `IGRAPH_MAX_EXACT_REAL` (= `2^53`), which is the largest integer
/// exactly representable as `f64`. We pick the same bound, expressed
/// in `usize` terms so the check is portable.
const MAX_NODES: usize = 1usize << 53;

fn check_weights(label: &str, weights: &[f64]) -> IgraphResult<()> {
    let mut max_w = f64::NEG_INFINITY;
    let mut min_w = f64::INFINITY;
    for (i, &w) in weights.iter().enumerate() {
        if w.is_nan() {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}[{i}] is NaN; Chung–Lu weights must be finite"
            )));
        }
        if w < min_w {
            min_w = w;
        }
        if w > max_w {
            max_w = w;
        }
    }
    if weights.is_empty() {
        return Ok(());
    }
    if min_w < 0.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "{label} must be non-negative; got minimum {min_w}"
        )));
    }
    if !max_w.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "{label} must be finite; got maximum {max_w}"
        )));
    }
    Ok(())
}

/// Sort indices `0..n` descending by `key[idx[k]]`. Ties broken by the
/// natural index order to keep the output deterministic across runs of
/// the same input.
fn sort_indices_desc(key: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..key.len()).collect();
    idx.sort_by(|&a, &b| {
        key[b]
            .partial_cmp(&key[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    idx
}

#[inline]
fn apply_variant(q: f64, variant: ChungLuVariant) -> f64 {
    match variant {
        ChungLuVariant::Original => {
            if q > 1.0 {
                1.0
            } else {
                q
            }
        }
        ChungLuVariant::Maxent => q / (1.0 + q),
        ChungLuVariant::Nr => 1.0 - (-q).exp(),
    }
}

/// Sample a random graph from the **Chung–Lu** expected-degree model.
///
/// * `out_weights` — non-negative, finite per-vertex weights. Length is
///   the resulting graph's vertex count `n`. In the sparse-graph limit
///   each `out_weights[i]` is approximately the expected (out-)degree of
///   vertex `i`.
/// * `in_weights` — when `Some`, generates a **directed** graph; must
///   have the same length as `out_weights` and the same total mass
///   (within `f64` exact equality, per upstream igraph). When `None`,
///   generates an **undirected** graph with `in_weights = out_weights`.
/// * `loops` — when `true`, self-loop edges are allowed (sampled
///   independently with `p_ii`).
/// * `variant` — which connection-probability formula to use; see
///   [`ChungLuVariant`].
/// * `seed` — initialises an internal [`SplitMix64`] PRNG so output is
///   reproducible given the inputs.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if:
/// * any weight is negative, NaN, or non-finite (`±∞`);
/// * `in_weights` has a different length than `out_weights`;
/// * `in_weights` and `out_weights` have a different sum (directed
///   case);
/// * `out_weights.len()` exceeds `2^53` (the largest integer exactly
///   representable as `f64`, matching upstream igraph's
///   `IGRAPH_MAX_EXACT_REAL` guard).
///
/// # Complexity
///
/// `O(|V| log |V| + |E|)` — the `log |V|` factor comes from the
/// descending in-weight sort; the sampler itself is linear in the
/// realised edge count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{chung_lu_game, ChungLuVariant};
///
/// // Power-law-style weights: a handful of "hubs" plus many low-degree
/// // vertices. The expected degree of vertex i ≈ out_weights[i].
/// let weights: Vec<f64> = (0..50).map(|i| 1.0 + (i as f64) * 0.2).collect();
/// let g = chung_lu_game(&weights, None, false, ChungLuVariant::Maxent, 42).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert!(!g.is_directed());
/// ```
pub fn chung_lu_game(
    out_weights: &[f64],
    in_weights: Option<&[f64]>,
    loops: bool,
    variant: ChungLuVariant,
    seed: u64,
) -> IgraphResult<Graph> {
    let n = out_weights.len();
    let directed = in_weights.is_some();

    if n > MAX_NODES {
        return Err(IgraphError::InvalidArgument(format!(
            "Chung–Lu vertex count {n} exceeds the largest exactly representable f64 integer (2^53)"
        )));
    }

    if let Some(inw) = in_weights {
        if inw.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "in_weights length {} does not match out_weights length {n}",
                inw.len()
            )));
        }
    }

    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument(format!("Chung–Lu vertex count {n} exceeds u32::MAX"))
    })?;

    if n == 0 {
        return Graph::new(0, directed);
    }

    check_weights("out_weights", out_weights)?;
    let wsum: f64 = out_weights.iter().sum();

    let in_view: &[f64] = match in_weights {
        Some(inw) => {
            check_weights("in_weights", inw)?;
            let in_sum: f64 = inw.iter().sum();
            if in_sum != wsum {
                return Err(IgraphError::InvalidArgument(format!(
                    "Sum of out- and in-weights must be the same; got out_sum = {wsum} and in_sum = {in_sum}"
                )));
            }
            inw
        }
        None => out_weights,
    };

    // S == 0 means every q_ij is 0 ⇒ empty graph regardless of variant.
    if wsum == 0.0 {
        return Graph::new(n_u32, directed);
    }

    let idx = sort_indices_desc(in_view);
    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    for i in 0..n {
        let vi = idx[i];
        let wi = out_weights[vi];
        if wi == 0.0 {
            if directed {
                continue;
            }
            break;
        }

        let mut j = if directed { 0 } else { i };
        let mut p: f64 = 1.0;

        loop {
            let remaining = n - j;
            let gap = rng.gen_geom(p);
            // Terminate when the skip overshoots; also protects against
            // overflow when `p` is tiny.
            if !gap.is_finite() || gap >= remaining as f64 {
                break;
            }
            // gen_geom returns a non-negative float; safe to cast.
            j += gap as usize;
            if j >= n {
                break;
            }

            let vj = idx[j];
            let wj = in_view[vj];

            // q = w_i · w_j / S with the variant-specific transform.
            let q_raw = wi * wj / wsum;
            let q = apply_variant(q_raw, variant);

            if q == 0.0 {
                break;
            }

            // p is the running upper-bound probability used for the
            // geometric skip; q / p is always in [0, 1] because q is
            // monotonically non-increasing along the descending-weight
            // sort and the previous step set p to its previous q value.
            let ratio = if p > 0.0 { q / p } else { 0.0 };
            let accept = rng.gen_unit() < ratio;
            if accept && (loops || vi != vj) {
                let from = u32::try_from(vi).map_err(|_| {
                    IgraphError::InvalidArgument(format!(
                        "Chung–Lu source vertex index {vi} overflows u32"
                    ))
                })?;
                let to = u32::try_from(vj).map_err(|_| {
                    IgraphError::InvalidArgument(format!(
                        "Chung–Lu target vertex index {vj} overflows u32"
                    ))
                })?;
                edges.push((from, to));
            }

            p = q;
            j += 1;
        }
    }

    let mut g = Graph::new(n_u32, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecount(g: &Graph) -> usize {
        g.ecount() as usize
    }

    #[test]
    fn empty_weights_undirected() {
        let g = chung_lu_game(&[], None, false, ChungLuVariant::Original, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert!(!g.is_directed());
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn empty_weights_directed() {
        let in_w: [f64; 0] = [];
        let g = chung_lu_game(&[], Some(&in_w), true, ChungLuVariant::Maxent, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn zero_weights_yields_empty_graph() {
        let w = vec![0.0; 8];
        for variant in [
            ChungLuVariant::Original,
            ChungLuVariant::Maxent,
            ChungLuVariant::Nr,
        ] {
            let g = chung_lu_game(&w, None, true, variant, 1).unwrap();
            assert_eq!(g.vcount(), 8);
            assert_eq!(g.ecount(), 0, "variant {variant:?} should give empty graph");
        }
    }

    #[test]
    fn single_vertex_no_loops_is_empty() {
        let g = chung_lu_game(&[5.0], None, false, ChungLuVariant::Maxent, 7).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn rejects_negative_weight() {
        let w = vec![1.0, -0.5, 2.0];
        let err = chung_lu_game(&w, None, false, ChungLuVariant::Original, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => {
                assert!(msg.contains("non-negative"), "msg = {msg}");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_non_finite_weight() {
        let w = vec![1.0, f64::INFINITY];
        let err = chung_lu_game(&w, None, false, ChungLuVariant::Original, 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));

        let w_nan = vec![1.0, f64::NAN];
        let err = chung_lu_game(&w_nan, None, false, ChungLuVariant::Maxent, 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn rejects_mismatched_in_out_lengths() {
        let out_w = vec![1.0, 2.0, 3.0];
        let in_w = vec![1.0, 5.0];
        let err =
            chung_lu_game(&out_w, Some(&in_w), false, ChungLuVariant::Original, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("length"), "msg = {msg}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rejects_mismatched_in_out_sums() {
        let out_w = vec![1.0, 2.0, 3.0];
        let in_w = vec![1.0, 1.0, 1.0];
        let err = chung_lu_game(&out_w, Some(&in_w), false, ChungLuVariant::Maxent, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("sum"), "msg = {msg}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let w: Vec<f64> = (0..40).map(|i| 1.0 + f64::from(i) * 0.1).collect();
        let g1 = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 0xABC).unwrap();
        let g2 = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 0xABC).unwrap();
        assert_eq!(ecount(&g1), ecount(&g2));
        let m = ecount(&g1);
        for eid in 0..m as u32 {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn different_seeds_differ_or_match_size_only() {
        let w: Vec<f64> = (0..40).map(|i| 1.0 + f64::from(i) * 0.1).collect();
        let g1 = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 1).unwrap();
        let g2 = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 2).unwrap();
        // Vertex counts always match.
        assert_eq!(g1.vcount(), g2.vcount());
        // Probability that 40-vertex outputs share edge lists exactly is
        // astronomically low; assert difference.
        let m1 = ecount(&g1);
        let m2 = ecount(&g2);
        if m1 == m2 && m1 > 0 {
            let mut any_diff = false;
            for eid in 0..m1 as u32 {
                if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                    any_diff = true;
                    break;
                }
            }
            assert!(
                any_diff,
                "two seeds happened to produce identical edge sequences"
            );
        }
    }

    #[test]
    fn no_loops_when_loops_false() {
        let w = vec![5.0; 20];
        let g = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 9).unwrap();
        let m = ecount(&g) as u32;
        for eid in 0..m {
            let (u, v) = g.edge(eid).unwrap();
            assert!(u != v, "found self-loop at edge {eid}: ({u},{v})");
        }
    }

    #[test]
    fn directed_graph_has_directed_edges() {
        let w = vec![3.0; 15];
        let g = chung_lu_game(&w, Some(&w), false, ChungLuVariant::Maxent, 11).unwrap();
        assert!(g.is_directed());
        assert_eq!(g.vcount(), 15);
    }

    #[test]
    fn maxent_mean_degree_matches_weight_in_sparse_limit() {
        // E[deg(i)] ≈ w_i (S - w_i) / S for undirected, no loops; with
        // identical weights w_i = w, this is w (n-1) w / (n w) = w (n-1) / n.
        let n = 200_usize;
        let w_val = 4.0_f64;
        let w = vec![w_val; n];
        let g = chung_lu_game(&w, None, false, ChungLuVariant::Maxent, 0x5EE_D001).unwrap();
        // Sum of degrees = 2|E| (undirected).
        let total_deg = 2.0 * ecount(&g) as f64;
        let mean_deg = total_deg / n as f64;
        // For Maxent: p = q / (1 + q) with q = w²/(n·w) = w/n.
        // So mean degree ≈ (n-1) · (w/n) / (1 + w/n)
        //                ≈ w · (n-1)/(n + w)
        let expected = w_val * (n as f64 - 1.0) / (n as f64 + w_val);
        let tol = 1.0_f64; // very generous; only sanity that it's near.
        assert!(
            (mean_deg - expected).abs() < tol,
            "mean_deg = {mean_deg}, expected ≈ {expected}"
        );
    }

    #[test]
    fn nr_and_original_all_zero_probability_yields_no_edges() {
        // Very low weights mean q ≈ 0 for all pairs → tiny graph.
        let w = vec![1e-3; 50];
        for variant in [
            ChungLuVariant::Original,
            ChungLuVariant::Maxent,
            ChungLuVariant::Nr,
        ] {
            let g = chung_lu_game(&w, None, false, variant, 13).unwrap();
            // Expected edges = C(50,2) · q with q = (1e-3)² / (50·1e-3) = 2e-5
            // => about 0.02 in expectation; almost certainly 0 in any draw.
            assert!(
                ecount(&g) <= 3,
                "variant {variant:?} produced too many edges"
            );
        }
    }

    #[test]
    fn vertex_count_is_input_length() {
        for n in [1, 2, 5, 20] {
            let w = vec![2.0; n];
            let g = chung_lu_game(&w, None, true, ChungLuVariant::Maxent, 1).unwrap();
            assert_eq!(g.vcount() as usize, n);
        }
    }

    #[test]
    fn original_with_very_large_weights_is_dense() {
        // Pick weights so that q_ij >> 1 for most pairs in the original
        // variant. p_ij gets clamped to 1, so we should approach the
        // complete graph.
        let n = 30;
        let w = vec![100.0; n];
        let g = chung_lu_game(&w, None, false, ChungLuVariant::Original, 19).unwrap();
        let m = ecount(&g);
        // Complete graph has n·(n-1)/2 = 435 edges; we should be ≥ 95%
        // of that thanks to clamp-to-1.
        let cmax = n * (n - 1) / 2;
        assert!(
            m as f64 >= 0.95 * cmax as f64,
            "expected near-complete graph, got {m}/{cmax}"
        );
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Vertex count is always exactly the weight-vector length.
        #[test]
        fn vcount_matches_weight_len(
            n in 1usize..40,
            w_max in 0.0_f64..5.0,
            seed: u64,
            variant_pick in 0u8..3,
        ) {
            let weights: Vec<f64> = (0..n).map(|i| w_max * (i as f64 + 1.0) / n as f64).collect();
            let variant = match variant_pick {
                0 => ChungLuVariant::Original,
                1 => ChungLuVariant::Maxent,
                _ => ChungLuVariant::Nr,
            };
            let g = chung_lu_game(&weights, None, false, variant, seed).unwrap();
            prop_assert_eq!(g.vcount() as usize, n);
            prop_assert!(!g.is_directed());
        }

        /// Determinism: same seed + same inputs ⇒ identical edge sequence.
        #[test]
        fn determinism(
            n in 1usize..40,
            w_max in 0.0_f64..3.0,
            seed: u64,
            loops: bool,
        ) {
            let weights: Vec<f64> = (0..n).map(|i| w_max * (i as f64 + 1.0) / n as f64).collect();
            let g1 = chung_lu_game(&weights, None, loops, ChungLuVariant::Maxent, seed).unwrap();
            let g2 = chung_lu_game(&weights, None, loops, ChungLuVariant::Maxent, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let m = g1.ecount();
            for eid in 0..m as u32 {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }

        /// `loops = false` ⇒ no self-loop edges in the output.
        #[test]
        fn no_self_loops_when_disabled(
            n in 2usize..30,
            w_max in 0.5_f64..4.0,
            seed: u64,
        ) {
            let weights: Vec<f64> = (0..n).map(|i| w_max * (i as f64 + 1.0) / n as f64).collect();
            let g = chung_lu_game(&weights, None, false, ChungLuVariant::Maxent, seed).unwrap();
            let m = g.ecount() as u32;
            for eid in 0..m {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_ne!(u, v);
            }
        }
    }
}
