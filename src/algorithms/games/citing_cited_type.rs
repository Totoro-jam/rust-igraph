//! Citing-cited type citation game (ALGO-GN-029).
//!
//! Counterpart of `igraph_citing_cited_type_game()` from
//! `references/igraph/src/games/citations.c:387-498`. Generalises
//! [`crate::cited_type_game`] (GN-017) so that the **category of the
//! citing vertex** also influences the choice of cited vertex: the new
//! edge `(i, to)` is sampled from a distribution whose weight depends
//! on `pref[type[i]][type[to]]`.
//!
//! Vertex types are **input** to this game (pre-assigned by the caller),
//! mirroring `cited_type_game`. The contrast with that algorithm is the
//! data structure: one Fenwick BIT (`PsumTree`) is maintained **per
//! citing type** indexed by candidate cited vertex, so a step with
//! citing type `t` runs `edges_per_step` independent samples against
//! `sumtrees[t]`. The per-step accumulator `sums[t]` is incrementally
//! updated when each new vertex `i` is added to every tree with weight
//! `pref[t][type[i]]`.
//!
//! ## Algorithm
//!
//! Let `T = max(types) + 1` (or `T = 0` when `nodes == 0`).
//!
//! 1. Initialise `T` Fenwick BITs `sumtrees[0..T]` of size `nodes` and
//!    a length-`T` accumulator `sums`.
//! 2. **First vertex setup**: for each `t ∈ [0, T)`:
//!    - `sumtrees[t].set(0, pref[t][type[0]])`
//!    - `sums[t] = pref[t][type[0]]`
//! 3. For each step `i ∈ [1, nodes)` with citing type `t = type[i]`:
//!    - For each of `edges_per_step` candidate edges:
//!       - If `sums[t] > 0`: draw `u ∈ [0, sums[t])` uniformly and binary-
//!         lift on `sumtrees[t]` (bounded by `i`, since vertices ≥ i have
//!         weight 0) to pick the cited vertex `to`.
//!       - Else (every previously-added vertex has zero weight under the
//!         citing-type row `pref[t]`): draw `to ∈ [0, i)` uniformly.
//!         Matches the C reference (`RNG_INTEGER(0, i-1)`).
//!       - Push the edge `(i, to)`.
//!    - Per-cell non-negativity check of `pref[j][type[i]]` for each row
//!      `j ∈ [0, T)`, then `sumtrees[j].set(i, pref[j][type[i]])` and
//!      `sums[j] += pref[j][type[i]]`.
//!
//! ## Self-loops & multi-edges
//!
//! * **Self-loops**: impossible by construction. The candidate set at
//!   step `i` is `[0, i)` (BIT search is bounded by `i`, fallback draws
//!   `RNG_INTEGER(0, i-1)`). Vertex `i` is added to the BITs *after* its
//!   `edges_per_step` draws complete.
//! * **Multi-edges**: yes, when `edges_per_step ≥ 2`, two of the per-step
//!   draws can collide on the same cited vertex. The upstream docstring
//!   explicitly warns about this and suggests `simplify()` if simple
//!   output is needed.
//!
//! ## Determinism
//!
//! Reproducible given the inputs and `seed` against the shared
//! [`SplitMix64`] PRNG. The stream is **not** portable to upstream
//! igraph's GLIBC RNG, so conformance assertions are structural rather
//! than bit-exact.
//!
//! ## Cost
//!
//! `O(nodes · (edges_per_step · log nodes + T · log nodes))` —
//! `T = max(types) + 1` BIT updates per step + `edges_per_step` BIT
//! searches per step.
//!
//! ## References
//!
//! * Upstream igraph C `igraph_citing_cited_type_game`,
//!   GPL-2.0-or-later reference port.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::many_single_char_names
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Fenwick-tree-based prefix-sum store with O(log n) point set and
/// O(log n) prefix-target search.
///
/// Local duplicate of the BIT used by [`crate::barabasi_game_psumtree`],
/// [`crate::lastcit_game`], [`crate::recent_degree_game`], and friends —
/// each module is kept self-contained per project convention.
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

    /// Binary-lifted prefix-sum search constrained to `[0, bound)`.
    /// `bound = 0` returns 0. The result is always `< bound` even under
    /// FP drift (the binary lifting is clamped at the end).
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

fn validate(
    nodes: u32,
    types: &[u32],
    pref: &[&[f64]],
    edges_per_step: u32,
) -> IgraphResult<usize> {
    if types.len() != nodes as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "types vector length ({}) must match nodes ({nodes})",
            types.len()
        )));
    }
    let _ = edges_per_step;

    // T = max(types) + 1 when nodes > 0, else 0 (matches upstream guard).
    let no_of_types: usize = if nodes == 0 {
        0
    } else {
        let mx = *types.iter().max().ok_or_else(|| {
            IgraphError::InvalidArgument("types vector empty but nodes > 0".into())
        })?;
        (mx as usize)
            .checked_add(1)
            .ok_or_else(|| IgraphError::InvalidArgument("type index overflow".into()))?
    };

    if pref.len() != no_of_types {
        return Err(IgraphError::InvalidArgument(format!(
            "preference matrix row count ({}) must equal number of types ({no_of_types})",
            pref.len()
        )));
    }
    for (r, row) in pref.iter().enumerate() {
        if row.len() != no_of_types {
            return Err(IgraphError::InvalidArgument(format!(
                "preference matrix row {r} has {} columns, expected {no_of_types}",
                row.len()
            )));
        }
        for (c, &v) in row.iter().enumerate() {
            if v.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "pref[{r}][{c}] is NaN"
                )));
            }
            if !v.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "pref[{r}][{c}] is not finite (got {v})"
                )));
            }
            if v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix contains negative entry: {v} at [{r}][{c}]"
                )));
            }
        }
    }

    Ok(no_of_types)
}

/// Citing-cited type growing citation game. See module docs.
///
/// # Errors
///
/// * `types.len() != nodes`.
/// * `pref` is not square `T × T` where `T = max(types) + 1`.
/// * `pref` contains any negative, NaN, or non-finite entry.
///
/// # Examples
///
/// ```
/// use rust_igraph::citing_cited_type_game;
///
/// // Two type buckets, even citing vertices prefer odd cited and vice
/// // versa. 50 vertices alternating types ⇒ (nodes-1)*eps edges in
/// // total since pref is strictly positive throughout.
/// let types: Vec<u32> = (0..50).map(|v| v % 2).collect();
/// let pref_rows: Vec<Vec<f64>> = vec![vec![0.1, 1.0], vec![1.0, 0.1]];
/// let pref_refs: Vec<&[f64]> = pref_rows.iter().map(Vec::as_slice).collect();
/// let g = citing_cited_type_game(50, &types, &pref_refs, 3, true, 0xCAFE).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert_eq!(g.ecount(), 49 * 3);
/// ```
pub fn citing_cited_type_game(
    nodes: u32,
    types: &[u32],
    pref: &[&[f64]],
    edges_per_step: u32,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    let no_of_types = validate(nodes, types, pref, edges_per_step)?;

    let mut graph = Graph::new(nodes, directed)?;
    if nodes == 0 || nodes < 2 || edges_per_step == 0 {
        return Ok(graph);
    }
    // no_of_types == 0 only when nodes == 0 (already returned).

    let n = nodes as usize;
    let mut rng = SplitMix64::new(seed);

    // One BIT per citing type; sums[t] mirrors sumtrees[t].total() but is
    // kept as a fast scalar accumulator (matches the C reference).
    let mut sumtrees: Vec<PsumTree> = (0..no_of_types).map(|_| PsumTree::new(n)).collect();
    let mut sums: Vec<f64> = vec![0.0; no_of_types];

    // First-vertex setup.
    let t0 = types[0] as usize;
    for j in 0..no_of_types {
        let w = pref[j][t0];
        sumtrees[j].set(0, w);
        sums[j] = w;
    }

    let edges_capacity = (n.saturating_sub(1)).saturating_mul(edges_per_step as usize);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edges_capacity);

    for (i, &type_i) in types.iter().enumerate().take(n).skip(1) {
        let ti = type_i as usize;
        let sum_ti = sums[ti];
        for _ in 0..edges_per_step {
            let to = if sum_ti > 0.0 {
                let target = rng.gen_unit() * sum_ti;
                // Bound to [0, i): vertices ≥ i are not yet in the tree
                // (they sit at zero weight) but might be reached via FP
                // drift through the binary lifting.
                sumtrees[ti].search_bounded(target, i)
            } else {
                let span = i as u64;
                (rng.next_u64() % span) as usize
            };
            let src = u32::try_from(i)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            let dst = u32::try_from(to)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            edges.push((src, dst));
        }
        // Add vertex i to every BIT under its true (citing-type, cited-type)
        // weight. Non-negativity was already enforced by validate().
        for j in 0..no_of_types {
            let w = pref[j][ti];
            sumtrees[j].set(i, w);
            sums[j] += w;
        }
    }

    graph.add_edges(edges)?;

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pref_refs(rows: &[Vec<f64>]) -> Vec<&[f64]> {
        rows.iter().map(Vec::as_slice).collect()
    }

    #[test]
    fn nodes_zero_returns_empty_graph() {
        let rows: Vec<Vec<f64>> = vec![];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(0, &[], &refs, 3, false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn nodes_one_returns_single_vertex_no_edges() {
        let rows = vec![vec![1.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(1, &[0], &refs, 5, false, 2).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn edges_per_step_zero_yields_edgeless() {
        let rows = vec![vec![1.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(50, &[0u32; 50], &refs, 0, false, 3).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_ecount_when_pref_positive() {
        let n = 40u32;
        let eps = 4u32;
        let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
        let rows = vec![
            vec![1.0, 2.0, 0.5],
            vec![0.7, 1.1, 0.3],
            vec![0.2, 0.4, 0.9],
        ];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, eps, false, 4).unwrap();
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), ((n - 1) * eps) as usize);
    }

    #[test]
    fn determinism() {
        let n = 30u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let rows = vec![vec![0.3, 0.7], vec![0.5, 0.5]];
        let refs = pref_refs(&rows);
        let g1 = citing_cited_type_game(n, &types, &refs, 3, true, 12345).unwrap();
        let g2 = citing_cited_type_game(n, &types, &refs, 3, true, 12345).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        let n_e = u32::try_from(g1.ecount()).unwrap();
        for eid in 0..n_e {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let n = 40u32;
        let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
        let rows = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let refs = pref_refs(&rows);
        let g1 = citing_cited_type_game(n, &types, &refs, 2, false, 1).unwrap();
        let g2 = citing_cited_type_game(n, &types, &refs, 2, false, 2).unwrap();
        let n_e = u32::try_from(g1.ecount()).unwrap();
        let mut any_diff = false;
        for eid in 0..n_e {
            if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                any_diff = true;
                break;
            }
        }
        assert!(any_diff);
    }

    #[test]
    fn all_zero_pref_falls_back_to_uniform_no_self_loops() {
        let n = 10u32;
        let types = vec![0u32; n as usize];
        let rows = vec![vec![0.0]];
        let refs = pref_refs(&rows);
        // Use directed=true so the temporal ordering invariant (dst < src)
        // survives Graph::add_edges canonicalisation.
        let g = citing_cited_type_game(n, &types, &refs, 2, true, 7).unwrap();
        assert_eq!(g.ecount(), ((n - 1) * 2) as usize);
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(
                a, b,
                "uniform fallback (RNG_INTEGER(0, i-1)) must never self-loop"
            );
            assert!(b < a, "target {b} should be < source {a}");
        }
    }

    #[test]
    fn positive_pref_never_self_loops() {
        let n = 80u32;
        let types: Vec<u32> = (0..n).map(|v| v % 4).collect();
        let rows = vec![
            vec![1.0, 2.0, 3.0, 0.5],
            vec![0.4, 1.5, 0.7, 1.2],
            vec![2.1, 0.8, 1.0, 1.1],
            vec![0.6, 0.9, 1.3, 0.4],
        ];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 3, true, 999).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b);
            assert!(b < a);
        }
    }

    #[test]
    fn directed_flag_propagates() {
        let n = 20u32;
        let types = vec![0u32; n as usize];
        let rows = vec![vec![1.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 2, true, 11).unwrap();
        assert!(g.is_directed());
    }

    #[test]
    fn undirected_flag_propagates() {
        let n = 20u32;
        let types = vec![0u32; n as usize];
        let rows = vec![vec![1.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 2, false, 12).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn err_types_length_mismatch() {
        let rows = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(10, &[0, 1], &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_not_square() {
        let rows = vec![vec![1.0, 0.5]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(5, &[0; 5], &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_wrong_row_count() {
        let types: Vec<u32> = vec![0, 1, 2, 0, 1];
        let rows = vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(5, &types, &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_negative() {
        let rows = vec![vec![-1.0]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(5, &[0; 5], &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_nan() {
        let rows = vec![vec![f64::NAN]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(5, &[0; 5], &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_inf() {
        let rows = vec![vec![f64::INFINITY]];
        let refs = pref_refs(&rows);
        let err = citing_cited_type_game(5, &[0; 5], &refs, 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn citing_type_blocks_all_zeros_row_falls_back_uniform() {
        // citing type 0 has all-zero weights ⇒ for citing vertices of
        // type 0 the BIT-sum is zero ⇒ uniform fallback. citing type 1
        // has positive weights ⇒ structural sampling.
        let n = 12u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let rows = vec![vec![0.0, 0.0], vec![1.0, 1.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 1, true, 17).unwrap();
        assert_eq!(g.ecount(), (n - 1) as usize);
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b);
            assert!(b < a);
        }
    }

    #[test]
    fn row_concentrates_on_preferred_cited_type() {
        // citing type 0 prefers cited type 0 strongly; citing type 1
        // prefers cited type 1 strongly. Count cross-type ties.
        let n = 200u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let rows = vec![vec![100.0, 0.01], vec![0.01, 100.0]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 2, true, 31).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        let mut same = 0u32;
        let mut diff = 0u32;
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            if types[a as usize] == types[b as usize] {
                same += 1;
            } else {
                diff += 1;
            }
        }
        // With a 10_000:1 row preference, > 95% of edges should be
        // same-type. Use 20× as a robust threshold.
        assert!(
            same > 20 * diff,
            "expected heavy same-type concentration (got {same} vs {diff})"
        );
    }

    #[test]
    fn off_diagonal_concentrates_on_cross_type() {
        // citing type 0 prefers cited type 1; citing type 1 prefers cited
        // type 0. Edges should mostly cross.
        let n = 200u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let rows = vec![vec![0.01, 100.0], vec![100.0, 0.01]];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, 2, true, 67).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        let mut same = 0u32;
        let mut diff = 0u32;
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            if types[a as usize] == types[b as usize] {
                same += 1;
            } else {
                diff += 1;
            }
        }
        assert!(
            diff > 20 * same,
            "expected heavy cross-type concentration (got {diff} vs {same})"
        );
    }

    #[test]
    fn source_is_step_index_dst_less_than_src() {
        let n = 40u32;
        let eps = 3u32;
        let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
        let rows = vec![
            vec![1.0, 2.0, 0.5],
            vec![0.7, 1.1, 0.3],
            vec![0.2, 0.4, 0.9],
        ];
        let refs = pref_refs(&rows);
        let g = citing_cited_type_game(n, &types, &refs, eps, true, 0xABCD).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (src, dst) = g.edge(eid).unwrap();
            let expected_src = 1 + eid / eps;
            assert_eq!(src, expected_src);
            assert!(dst < src);
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ecount_exact_when_pref_positive(
            n in 2u32..40,
            eps in 1u32..6,
            seed in any::<u64>(),
        ) {
            // Build types as `v % 2` so max-type is always 1 ⇒ T = 2,
            // independent of how small n shrinks.
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let rows: Vec<Vec<f64>> = vec![vec![1.0, 2.0], vec![0.7, 1.1]];
            let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
            let g = citing_cited_type_game(n, &types, &refs, eps, false, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), ((n - 1) * eps) as usize);
        }

        #[test]
        fn no_self_loops_when_pref_positive(
            n in 2u32..30,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let rows: Vec<Vec<f64>> = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
            let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
            let g = citing_cited_type_game(n, &types, &refs, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (a, b) = g.edge(eid).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn no_self_loops_in_all_zero_fallback(
            n in 2u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            // All-zero rows ⇒ uniform fallback on every draw. The fallback
            // uses RNG_INTEGER(0, i-1), so target is strictly less than i.
            let types = vec![0u32; n as usize];
            let rows: Vec<Vec<f64>> = vec![vec![0.0]];
            let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
            let g = citing_cited_type_game(n, &types, &refs, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (a, b) = g.edge(eid).unwrap();
                prop_assert_ne!(a, b);
                prop_assert!(b < a);
            }
        }

        #[test]
        fn source_is_step_index(
            n in 2u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let rows: Vec<Vec<f64>> = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
            let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
            let g = citing_cited_type_game(n, &types, &refs, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (src, _dst) = g.edge(eid).unwrap();
                let expected_src = 1 + eid / eps;
                prop_assert_eq!(src, expected_src);
            }
        }

        #[test]
        fn determinism_under_proptest(
            n in 2u32..30,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let rows: Vec<Vec<f64>> = vec![vec![1.0, 2.0], vec![0.7, 1.1]];
            let refs: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
            let g1 = citing_cited_type_game(n, &types, &refs, eps, false, seed).unwrap();
            let g2 = citing_cited_type_game(n, &types, &refs, eps, false, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let n_e = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..n_e {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }
    }
}
