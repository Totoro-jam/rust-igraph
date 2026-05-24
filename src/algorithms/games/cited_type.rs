//! Cited-type citation game (ALGO-GN-017).
//!
//! Counterpart of `igraph_cited_type_game()` from
//! `references/igraph/src/games/citations.c:246-335`. Models a growing
//! citation network where each new vertex `i ∈ [1, nodes)` adds
//! `edges_per_step` outgoing edges, each pointing to a previously-added
//! vertex sampled with probability proportional to the *cited* vertex's
//! type attractivity `pref[type[cited]]`.
//!
//! Vertex types are **input** to this game (pre-assigned by the caller),
//! not sampled internally — that is the key contrast with the related
//! `establishment_game` (GN-015) and `callaway_traits_game` (GN-016)
//! which both draw types from a categorical distribution.
//!
//! ## Algorithm
//!
//! 1. Maintain an incrementally-built cumulative sum
//!    `cumsum[k+1] = Σ_{v < k+1} pref[type[v]]`, seeded with
//!    `cumsum[0] = 0` and `cumsum[1] = pref[type[0]]`.
//! 2. For each step `i ∈ [1, nodes)`:
//!    a. For each of `edges_per_step` candidate edges:
//!       - If `sum > 0`: draw `u ∈ [0, sum)` uniformly and binary-search
//!         `cumsum` for the smallest `k` such that `cumsum[k] > u`; emit
//!         edge `(i, k - 1)`.
//!       - Else (every type so far has zero attractivity): emit edge
//!         `(i, i)` — a self-loop. Matches the C reference behaviour.
//!    - b. Push `pref[type[i]]` onto `cumsum` and update `sum`.
//!
//! ## Self-loops & multi-edges
//!
//! * **Multi-edges**: yes, when `edges_per_step ≥ 2`, two of the `eps`
//!   draws at step `i` can select the same previous vertex. The
//!   upstream docstring explicitly warns about this and recommends
//!   `simplify()` if simple output is required.
//! * **Self-loops**: only via the `sum == 0` fallback (e.g. when
//!   `pref` is identically zero). When at least one previously-assigned
//!   type has `pref > 0`, self-loops are impossible by construction
//!   (the candidate set always excludes `i` itself).
//!
//! ## Determinism
//!
//! Reproducible given the inputs and `seed` against the shared
//! [`crate::core::rng::SplitMix64`] PRNG. The stream is **not** portable
//! to upstream igraph's GLIBC RNG, so conformance assertions are
//! structural (vertex/edge counts, no-self-loops invariant when
//! `sum > 0` is guaranteed) rather than bit-exact.
//!
//! ## References
//!
//! * S. Redner, *"How popular is your paper? An empirical study of the
//!   citation distribution"*, Eur. Phys. J. B **4**, 131-134 (1998).
//! * Upstream igraph C `igraph_cited_type_game`, MIT-licensed
//!   reference port.

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

fn validate(nodes: u32, types: &[u32], pref: &[f64], edges_per_step: u32) -> IgraphResult<()> {
    if types.len() != nodes as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "types vector length ({}) must match nodes ({nodes})",
            types.len()
        )));
    }
    // edges_per_step is u32 so non-negativity is implicit; we keep the
    // upstream error message intent only for the pref validation below.
    let _ = edges_per_step;

    let mut max_type: Option<u32> = None;
    for &t in types {
        max_type = Some(max_type.map_or(t, |m| m.max(t)));
    }
    if let Some(mt) = max_type {
        let required = (mt as usize)
            .checked_add(1)
            .ok_or_else(|| IgraphError::InvalidArgument("type index overflow".into()))?;
        if pref.len() < required {
            return Err(IgraphError::InvalidArgument(format!(
                "pref vector length ({}) must be at least max(types)+1 = {required}",
                pref.len()
            )));
        }
    }

    for (i, &p) in pref.iter().enumerate() {
        if p.is_nan() {
            return Err(IgraphError::InvalidArgument(format!("pref[{i}] is NaN")));
        }
        if !p.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "pref[{i}] is not finite (got {p})"
            )));
        }
        if p < 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "pref[{i}] is negative ({p})"
            )));
        }
    }

    Ok(())
}

/// Smallest `k` such that `cumsum[k] > target`. Mirrors the
/// inverse-transform sampling convention used in the C reference:
/// for `target = u·sum` with `u ∈ [0, 1)`, the chosen vertex is `k-1`.
fn cum_upper(cumsum: &[f64], target: f64) -> usize {
    cumsum.partition_point(|&x| x <= target)
}

/// Cited-type growing citation game. See module docs.
pub fn cited_type_game(
    nodes: u32,
    types: &[u32],
    pref: &[f64],
    edges_per_step: u32,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(nodes, types, pref, edges_per_step)?;

    let mut graph = Graph::new(nodes, directed)?;
    if nodes == 0 || edges_per_step == 0 || nodes < 2 {
        return Ok(graph);
    }

    let mut rng = SplitMix64::new(seed);

    // cumsum[k+1] = Σ_{v=0}^{k} pref[type[v]]; cumsum[0] = 0.
    let n = nodes as usize;
    let mut cumsum: Vec<f64> = Vec::with_capacity(n + 1);
    cumsum.push(0.0);
    let first_type = types[0] as usize;
    let first_pref = pref[first_type];
    cumsum.push(first_pref);
    let mut sum = first_pref;

    let edges_capacity = (n.saturating_sub(1)).saturating_mul(edges_per_step as usize);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(edges_capacity);

    for i in 1..n {
        for _ in 0..edges_per_step {
            let to = if sum > 0.0 {
                let target = rng.gen_unit() * sum;
                let k = cum_upper(&cumsum, target);
                // k is in [1, cumsum.len()) when sum > 0; vertex is k-1.
                k.saturating_sub(1)
            } else {
                // Fallback matching the C reference: when no candidate has
                // positive attractivity, emit a self-loop on i.
                i
            };
            let src = u32::try_from(i)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            let dst = u32::try_from(to)
                .map_err(|_| IgraphError::InvalidArgument("vertex index overflow".into()))?;
            edges.push((src, dst));
        }
        let next_pref = pref[types[i] as usize];
        sum += next_pref;
        cumsum.push(sum);
    }

    graph.add_edges(edges)?;

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nodes_zero_returns_empty_graph() {
        let g = cited_type_game(0, &[], &[1.0], 3, false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn nodes_one_returns_single_vertex_no_edges() {
        let g = cited_type_game(1, &[0], &[1.0], 5, false, 2).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn edges_per_step_zero_yields_edgeless() {
        let g = cited_type_game(50, &[0u32; 50], &[1.0], 0, false, 3).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_ecount_when_pref_positive() {
        // With pref > 0 throughout, every step emits exactly eps edges.
        let n = 40u32;
        let eps = 4u32;
        let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
        let pref = vec![1.0, 2.0, 0.5];
        let g = cited_type_game(n, &types, &pref, eps, false, 4).unwrap();
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), ((n - 1) * eps) as usize);
    }

    #[test]
    fn determinism() {
        let n = 30u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let pref = vec![0.3, 0.7];
        let g1 = cited_type_game(n, &types, &pref, 3, true, 12345).unwrap();
        let g2 = cited_type_game(n, &types, &pref, 3, true, 12345).unwrap();
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
        let pref = vec![1.0, 1.0, 1.0]; // uniform → still randomized targets
        let g1 = cited_type_game(n, &types, &pref, 2, false, 1).unwrap();
        let g2 = cited_type_game(n, &types, &pref, 2, false, 2).unwrap();
        // Edge counts equal (deterministic given inputs), but edge LIST
        // ought to differ on at least one entry.
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
    fn all_zero_pref_emits_self_loops() {
        let n = 10u32;
        let types = vec![0u32; n as usize];
        let pref = vec![0.0];
        let g = cited_type_game(n, &types, &pref, 2, false, 7).unwrap();
        // Every (n-1)*eps candidate falls through to the sum=0 fallback,
        // which writes a self-loop on the current step vertex.
        assert_eq!(g.ecount(), ((n - 1) * 2) as usize);
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_eq!(a, b, "all-zero-pref fallback must yield self-loops");
        }
    }

    #[test]
    fn positive_pref_never_self_loops() {
        // Whenever sum > 0, target is in [0, sum) and binsearch picks k
        // in [1, cumsum.len()), which maps to vertex k-1 ∈ [0, i-1].
        // Hence no self-loop is possible.
        let n = 80u32;
        let types: Vec<u32> = (0..n).map(|v| v % 4).collect();
        let pref = vec![1.0, 2.0, 3.0, 0.5];
        let g = cited_type_game(n, &types, &pref, 3, false, 999).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "positive-pref run must never self-loop");
        }
    }

    #[test]
    fn directed_flag_propagates() {
        let n = 20u32;
        let types = vec![0u32; n as usize];
        let pref = vec![1.0];
        let g = cited_type_game(n, &types, &pref, 2, true, 11).unwrap();
        assert!(g.is_directed());
    }

    #[test]
    fn undirected_flag_propagates() {
        let n = 20u32;
        let types = vec![0u32; n as usize];
        let pref = vec![1.0];
        let g = cited_type_game(n, &types, &pref, 2, false, 12).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn err_types_length_mismatch() {
        let err = cited_type_game(10, &[0, 1], &[1.0, 1.0], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_too_short() {
        let err = cited_type_game(5, &[0, 1, 2, 0, 1], &[1.0, 1.0], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_negative() {
        let err = cited_type_game(5, &[0; 5], &[-1.0], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_nan() {
        let err = cited_type_game(5, &[0; 5], &[f64::NAN], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_inf() {
        let err = cited_type_game(5, &[0; 5], &[f64::INFINITY], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn err_pref_neg_inf() {
        let err = cited_type_game(5, &[0; 5], &[f64::NEG_INFINITY], 1, false, 1);
        assert!(err.is_err());
    }

    #[test]
    fn zero_pref_for_one_type_concentrates_to_others() {
        // type 0 has pref 0, type 1 has pref 1. Citations should
        // only ever target type-1 vertices (after the first one).
        let n = 60u32;
        let types: Vec<u32> = (0..n).map(|v| u32::from(v % 2 != 0)).collect();
        let pref = vec![0.0, 1.0];
        let g = cited_type_game(n, &types, &pref, 1, true, 21).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        // vertex 0 has type 0 (pref 0). For step i=1, sum is still 0,
        // so the fallback fires and we get edge (1, 1). After step 1,
        // type[1] = 1 with pref 1, so sum > 0 from i=2 onward and
        // every subsequent citation must hit a type-1 vertex (an odd
        // index in [0, i-1]).
        let mut hit_self_loop_count = 0u32;
        for eid in 0..n_e {
            let (a, b) = g.edge(eid).unwrap();
            if a == b {
                hit_self_loop_count += 1;
            } else {
                assert_eq!(
                    types[b as usize], 1u32,
                    "non-fallback citation must target a type-1 vertex"
                );
            }
        }
        // Exactly one self-loop expected (the i=1 step).
        assert_eq!(hit_self_loop_count, 1);
    }

    #[test]
    fn high_pref_concentrates_in_degree() {
        // type 0 pref = 100, type 1 pref = 0.01. Most citations should
        // land on type-0 vertices.
        let n = 200u32;
        let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
        let pref = vec![100.0, 0.01];
        let g = cited_type_game(n, &types, &pref, 3, true, 31).unwrap();
        let n_e = u32::try_from(g.ecount()).unwrap();
        let mut to_type0 = 0u32;
        let mut to_type1 = 0u32;
        for eid in 0..n_e {
            let (_, b) = g.edge(eid).unwrap();
            if types[b as usize] == 0 {
                to_type0 += 1;
            } else {
                to_type1 += 1;
            }
        }
        // With a ~10_000:1 pref ratio at parity, > 95% citations to type 0.
        assert!(
            to_type0 > 20 * to_type1,
            "expected heavy concentration on type 0 (got {to_type0} vs {to_type1})"
        );
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
            let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
            let pref = vec![1.0, 2.0, 0.5];
            let g = cited_type_game(n, &types, &pref, eps, false, seed).unwrap();
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
            let pref = vec![1.0, 1.0];
            let g = cited_type_game(n, &types, &pref, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (a, b) = g.edge(eid).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn source_is_always_step_index(
            n in 2u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let pref = vec![1.0, 1.0];
            let g = cited_type_game(n, &types, &pref, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (src, _dst) = g.edge(eid).unwrap();
                // Edges are emitted in step order (i = 1, 2, ..., n-1),
                // eps per step, so src = 1 + eid / eps.
                let expected_src = 1 + eid / eps;
                prop_assert_eq!(src, expected_src);
            }
        }

        #[test]
        fn target_in_zero_to_step_minus_one(
            n in 3u32..25,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 2).collect();
            let pref = vec![1.0, 1.0]; // both positive → no fallback
            let g = cited_type_game(n, &types, &pref, eps, true, seed).unwrap();
            let n_e = u32::try_from(g.ecount()).unwrap();
            for eid in 0..n_e {
                let (src, dst) = g.edge(eid).unwrap();
                prop_assert!(dst < src,
                    "edge {eid}: dst {dst} should be < src {src}");
            }
        }

        #[test]
        fn determinism_under_proptest(
            n in 2u32..30,
            eps in 1u32..4,
            seed in any::<u64>(),
        ) {
            let types: Vec<u32> = (0..n).map(|v| v % 3).collect();
            let pref = vec![1.0, 2.0, 0.5];
            let g1 = cited_type_game(n, &types, &pref, eps, false, seed).unwrap();
            let g2 = cited_type_game(n, &types, &pref, eps, false, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let n_e = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..n_e {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }
    }
}
