//! Establishment / sample-traits game (ALGO-GN-015).
//!
//! Counterpart of `igraph_establishment_game()` from
//! `references/igraph/src/games/establishment.c:57`. Models a growing
//! graph with vertex types: a single vertex is added at each step and
//! tries to connect to `k` already-present vertices, with each candidate
//! edge accepted independently with probability
//! `pref_matrix[type_new][type_old]`.
//!
//! ## Algorithm
//!
//! 1. Build a cumulative distribution from `type_dist` (or the uniform
//!    `[1, 1, …, 1]` distribution when `type_dist == None`).
//! 2. Assign every vertex `v ∈ [0, nodes)` a type by drawing a uniform
//!    sample in `[0, total)` and binary-searching the cumulative table.
//! 3. For each `v ∈ [k, nodes)`, draw `k` distinct previous vertices
//!    `[0, v)` via Floyd's distinct-sample. For each picked neighbour
//!    `u`, accept the edge `(v, u)` with probability
//!    `pref_matrix[type[v]][type[u]]` using a single uniform draw.
//!
//! ## Determinism
//!
//! Reproducible given the inputs and `seed` against the shared
//! `SplitMix64` PRNG. The stream is **not** portable
//! to upstream igraph's GLIBC RNG, so conformance assertions are
//! structural (vertex/edge counts, type-vector range, support of the
//! preference matrix) rather than bit-exact.
//!
//! ## References
//!
//! * G. Caldarelli, A. Capocci, P. De Los Rios, and M. A. Muñoz, *"Scale-free
//!   networks from varying vertex intrinsic fitness"*, Phys. Rev. Lett.
//!   **89**, 258702 (2002).

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::needless_range_loop
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate(
    nodes: u32,
    types: u32,
    type_dist: Option<&[f64]>,
    pref_matrix: &[Vec<f64>],
    directed: bool,
) -> IgraphResult<()> {
    if types < 1 {
        return Err(IgraphError::InvalidArgument(
            "The number of vertex types must be at least 1.".into(),
        ));
    }
    let k = types as usize;
    if let Some(td) = type_dist {
        if td.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "type_dist length ({}) does not match number of types ({k})",
                td.len()
            )));
        }
        for (i, &v) in td.iter().enumerate() {
            if v.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "type_dist[{i}] must not be NaN"
                )));
            }
            if v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "type_dist[{i}] = {v} must be non-negative"
                )));
            }
        }
    }
    if pref_matrix.len() != k {
        return Err(IgraphError::InvalidArgument(format!(
            "preference matrix has {} rows (expected {k})",
            pref_matrix.len()
        )));
    }
    for (i, row) in pref_matrix.iter().enumerate() {
        if row.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "preference matrix row {i} has length {} (expected {k})",
                row.len()
            )));
        }
        for (j, &p) in row.iter().enumerate() {
            if p.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] must not be NaN"
                )));
            }
            if !(0.0..=1.0).contains(&p) {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] = {p} must lie in [0, 1]"
                )));
            }
        }
    }
    if !directed {
        for (i, row_i) in pref_matrix.iter().enumerate() {
            for (j, row_j) in pref_matrix.iter().enumerate().skip(i + 1) {
                if row_i[j] != row_j[i] {
                    return Err(IgraphError::InvalidArgument(format!(
                        "preference matrix must be symmetric for undirected graphs: \
                         pref[{i}][{j}] = {} but pref[{j}][{i}] = {}",
                        row_i[j], row_j[i],
                    )));
                }
            }
        }
    }
    // nodes is u32, automatically non-negative; sentinel u32 ceiling
    // is below `2^53`, so the f64-exact concern from preference.rs does
    // not apply here.
    let _ = nodes;
    Ok(())
}

/// Smallest `k ≥ 1` such that `cumdist[k] > u`. Mirrors
/// `igraph_vector_binsearch` for the cumulative-distribution case.
fn cumdist_lookup(cumdist: &[f64], u: f64) -> usize {
    let mut lo = 1usize;
    let mut hi = cumdist.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cumdist[mid] > u {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo.min(cumdist.len() - 1).max(1)
}

/// Floyd distinct-sample of `m` integers from `[0, n)`. Result is sorted
/// ascending. Caller must ensure `m <= n`.
fn floyd_distinct_sample(rng: &mut SplitMix64, n: u32, m: u32) -> Vec<u32> {
    debug_assert!(m <= n);
    let cap = m as usize;
    let mut chosen: HashSet<u32> = HashSet::with_capacity(cap);
    let mut out: Vec<u32> = Vec::with_capacity(cap);
    for j in (n - m)..n {
        let span = (j as usize) + 1;
        let t = rng.gen_index(span) as u32;
        let pick = if chosen.insert(t) {
            t
        } else {
            chosen.insert(j);
            j
        };
        out.push(pick);
    }
    out.sort_unstable();
    out
}

/// Generates a random graph with vertex types via the establishment
/// model.
///
/// Vertices are added one-by-one. Once `i ≥ k`, a new vertex tries to
/// connect to `k` previously added vertices uniformly at random; for
/// each candidate, the edge is accepted with probability
/// `pref_matrix[type[i]][type[j]]`. The result is a simple (no parallel
/// edges, no self-loops by construction) graph whose density is shaped
/// jointly by the type distribution and the preference matrix.
///
/// * `nodes` — number of vertices.
/// * `types ≥ 1` — number of vertex types.
/// * `k` — connections attempted per new vertex.
/// * `type_dist` — categorical weights over types; `None` means uniform.
///   Must be the same length as `types`, all entries non-negative,
///   total mass strictly positive.
/// * `pref_matrix` — `types × types` matrix of acceptance probabilities
///   in `[0, 1]`. When `directed = false` it must be symmetric.
/// * `directed` — generate a directed graph.
/// * `seed` — `SplitMix64` seed.
///
/// Returns the generated [`Graph`] and the per-vertex type vector.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when the preference matrix
/// or type distribution is malformed (wrong size, NaN, out-of-range
/// probability, asymmetric for an undirected graph) or when the type
/// distribution is non-positive everywhere.
///
/// # Example
///
/// ```
/// use rust_igraph::establishment_game;
///
/// // Two types, only-cross prefs ⇒ bipartite directed graph.
/// let pref = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
/// let (g, types) =
///     establishment_game(20, 2, 5, Some(&[1.0, 1.0]), &pref, true, 42).unwrap();
/// assert_eq!(g.vcount(), 20);
/// assert_eq!(types.len(), 20);
/// assert!(types.iter().all(|&t| t < 2));
/// ```
pub fn establishment_game(
    nodes: u32,
    types: u32,
    k: u32,
    type_dist: Option<&[f64]>,
    pref_matrix: &[Vec<f64>],
    directed: bool,
    seed: u64,
) -> IgraphResult<(Graph, Vec<u32>)> {
    validate(nodes, types, type_dist, pref_matrix, directed)?;

    let n_types = types as usize;
    let mut rng = SplitMix64::new(seed);

    let mut cumdist = vec![0.0f64; n_types + 1];
    if let Some(td) = type_dist {
        for i in 0..n_types {
            cumdist[i + 1] = cumdist[i] + td[i];
        }
    } else {
        for i in 0..n_types {
            cumdist[i + 1] = (i + 1) as f64;
        }
    }
    let maxcum = cumdist[n_types];
    if maxcum <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "type_dist must contain at least one positive value".into(),
        ));
    }

    let mut node_types = vec![0u32; nodes as usize];
    for v in 0..(nodes as usize) {
        let u = rng.gen_unit() * maxcum;
        let pos = cumdist_lookup(&cumdist, u);
        let t = (pos - 1).min(n_types - 1);
        node_types[v] = t as u32;
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    if k > 0 && nodes > k {
        for i in k..nodes {
            let type1 = node_types[i as usize] as usize;
            let picks = floyd_distinct_sample(&mut rng, i, k);
            for &j in &picks {
                let type2 = node_types[j as usize] as usize;
                let p = pref_matrix[type1][type2];
                if p > 0.0 && rng.gen_unit() < p {
                    edges.push((i, j));
                }
            }
        }
    }

    let mut g = Graph::new(nodes, directed)?;
    g.add_edges(edges)?;
    Ok((g, node_types))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet as Set;

    fn diag_pref(types: usize, p: f64) -> Vec<Vec<f64>> {
        (0..types)
            .map(|i| (0..types).map(|j| if i == j { p } else { 0.0 }).collect())
            .collect()
    }

    fn full_pref(types: usize, p: f64) -> Vec<Vec<f64>> {
        vec![vec![p; types]; types]
    }

    #[test]
    fn nodes_zero_returns_empty_graph() {
        let pref = full_pref(2, 0.5);
        let (g, types) = establishment_game(0, 2, 5, None, &pref, false, 42).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert_eq!(types.len(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn nodes_le_k_yields_no_edges() {
        let pref = full_pref(2, 1.0);
        let (g, types) = establishment_game(5, 2, 5, None, &pref, false, 7).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn k_zero_yields_no_edges() {
        let pref = full_pref(2, 1.0);
        let (g, _) = establishment_game(20, 2, 0, None, &pref, false, 7).unwrap();
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_pref_means_no_edges() {
        // Single vertex type, p_00 = 0 ⇒ growth never accepts.
        let pref = vec![vec![0.0]];
        let (g, types) = establishment_game(20, 1, 5, Some(&[1.0]), &pref, false, 11).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 0);
        assert!(types.iter().all(|&t| t == 0));
    }

    #[test]
    fn full_pref_saturates_edges() {
        // p == 1 across all type pairs ⇒ every Floyd pick becomes an edge.
        let pref = full_pref(2, 1.0);
        let (g, _) = establishment_game(30, 2, 5, None, &pref, false, 13).unwrap();
        // Vertices [k, n) each contribute exactly k edges.
        let n: usize = 30;
        let k: usize = 5;
        assert_eq!(g.ecount(), (n - k) * k);
    }

    #[test]
    fn cross_only_pref_yields_bipartite_directed() {
        // Two types, MOAT-like: only cross pairs accept.
        let pref = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let (g, types) = establishment_game(40, 2, 5, Some(&[1.0, 1.0]), &pref, true, 19).unwrap();
        assert_eq!(g.vcount(), 40);
        assert!(g.is_directed());
        // All edges must cross types.
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (u, v) = g.edge(eid).expect("edge id in bounds");
            assert_ne!(types[u as usize], types[v as usize]);
        }
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let pref = diag_pref(3, 0.4);
        let (g1, t1) =
            establishment_game(50, 3, 4, Some(&[1.0, 2.0, 3.0]), &pref, false, 0xABCDu64).unwrap();
        let (g2, t2) =
            establishment_game(50, 3, 4, Some(&[1.0, 2.0, 3.0]), &pref, false, 0xABCDu64).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        assert_eq!(t1, t2);
        let m = u32::try_from(g1.ecount()).expect("fits");
        for eid in 0..m {
            assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
        }
    }

    #[test]
    fn distinct_seeds_differ() {
        let pref = full_pref(2, 0.4);
        let (g1, _) = establishment_game(50, 2, 4, None, &pref, false, 1).unwrap();
        let (g2, _) = establishment_game(50, 2, 4, None, &pref, false, 2).unwrap();
        // Strong probability the edge counts or per-edge content differ.
        let m1 = u32::try_from(g1.ecount()).unwrap();
        let m2 = u32::try_from(g2.ecount()).unwrap();
        let same_edges = m1 == m2 && {
            let mut all_eq = true;
            for eid in 0..m1 {
                if g1.edge(eid).unwrap() != g2.edge(eid).unwrap() {
                    all_eq = false;
                    break;
                }
            }
            all_eq
        };
        assert!(!same_edges);
    }

    #[test]
    fn diagonal_pref_keeps_edges_within_types() {
        // p > 0 only on the diagonal ⇒ every accepted edge connects
        // same-type vertices.
        let pref = diag_pref(3, 0.6);
        let (g, types) =
            establishment_game(80, 3, 4, Some(&[1.0, 1.0, 1.0]), &pref, false, 23).unwrap();
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (u, v) = g.edge(eid).unwrap();
            assert_eq!(types[u as usize], types[v as usize]);
        }
    }

    #[test]
    fn graph_is_simple_no_loops_no_multi() {
        // Floyd's sample picks distinct neighbours, and edges only go
        // i → j with j < i, so no self-loops and no parallel edges.
        let pref = full_pref(3, 0.7);
        let (g, _) = establishment_game(60, 3, 4, None, &pref, false, 31).unwrap();
        let m = u32::try_from(g.ecount()).unwrap();
        let mut seen: Set<(VertexId, VertexId)> = Set::with_capacity(m as usize);
        for eid in 0..m {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "no self-loops");
            let key = if a <= b { (a, b) } else { (b, a) };
            assert!(seen.insert(key), "edge {key:?} appears twice");
        }
    }

    #[test]
    fn types_within_range_uniform() {
        let pref = full_pref(4, 0.0);
        let (_g, types) = establishment_game(200, 4, 1, None, &pref, false, 99).unwrap();
        assert!(types.iter().all(|&t| t < 4));
        // With uniform None and 200 nodes, all 4 types should appear.
        let mut seen = [false; 4];
        for &t in &types {
            seen[t as usize] = true;
        }
        assert!(seen.iter().all(|&x| x));
    }

    #[test]
    fn types_zero_rejected() {
        let pref: Vec<Vec<f64>> = vec![];
        let err = establishment_game(5, 0, 1, None, &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn type_dist_wrong_length_rejected() {
        let pref = full_pref(2, 0.1);
        let err = establishment_game(10, 2, 1, Some(&[1.0]), &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn type_dist_negative_rejected() {
        let pref = full_pref(2, 0.1);
        let err = establishment_game(10, 2, 1, Some(&[1.0, -0.1]), &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn type_dist_all_zero_rejected() {
        let pref = full_pref(2, 0.1);
        let err = establishment_game(10, 2, 1, Some(&[0.0, 0.0]), &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn pref_out_of_range_rejected() {
        let pref = vec![vec![1.5, 0.1], vec![0.1, 0.5]];
        let err = establishment_game(10, 2, 1, None, &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        let pref = vec![vec![-0.1, 0.1], vec![0.1, 0.5]];
        let err = establishment_game(10, 2, 1, None, &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn pref_nan_rejected() {
        let pref = vec![vec![f64::NAN, 0.1], vec![0.1, 0.5]];
        let err = establishment_game(10, 2, 1, None, &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn pref_asymmetric_undirected_rejected() {
        let pref = vec![vec![0.5, 0.1], vec![0.2, 0.5]];
        let err = establishment_game(10, 2, 1, None, &pref, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn pref_asymmetric_directed_ok() {
        let pref = vec![vec![0.5, 0.1], vec![0.2, 0.5]];
        let (g, _) = establishment_game(10, 2, 1, None, &pref, true, 1).unwrap();
        assert_eq!(g.vcount(), 10);
        assert!(g.is_directed());
    }

    #[test]
    fn pref_wrong_shape_rejected() {
        let pref = vec![vec![0.5, 0.1, 0.1], vec![0.1, 0.5, 0.1]];
        let err = establishment_game(10, 2, 1, None, &pref, true, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn directed_no_loops_construction_invariant() {
        // i > j by construction for every edge, so no self-loops even
        // when `directed = true` and pref allows them.
        let pref = full_pref(2, 1.0);
        let (g, _) = establishment_game(30, 2, 3, None, &pref, true, 5).unwrap();
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b);
            assert!(a > b, "growth direction always points back in time");
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_harness {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(40))]

        #[test]
        fn types_in_range(
            seed in any::<u64>(),
            n in 0u32..120,
            t in 1u32..5,
            k in 0u32..6,
            directed in any::<bool>(),
        ) {
            let types_usize = t as usize;
            let pref = vec![vec![0.5; types_usize]; types_usize];
            let (g, types) = establishment_game(n, t, k, None, &pref, directed, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(types.len() as u32, n);
            for &tt in &types {
                prop_assert!(tt < t);
            }
        }

        #[test]
        fn ecount_band(
            seed in any::<u64>(),
            n in 5u32..80,
            t in 1u32..4,
            k in 0u32..5,
            p in 0.0f64..=1.0f64,
        ) {
            let types_usize = t as usize;
            let pref = vec![vec![p; types_usize]; types_usize];
            let (g, _) = establishment_game(n, t, k, None, &pref, false, seed).unwrap();
            // 0 ≤ m ≤ (n - k) * k when n > k, else 0.
            if n > k {
                let bound = u64::from(n - k) * u64::from(k);
                prop_assert!(g.ecount() as u64 <= bound);
            } else {
                prop_assert_eq!(g.ecount(), 0);
            }
        }

        #[test]
        fn deterministic(
            seed in any::<u64>(),
            n in 0u32..60,
            t in 1u32..4,
            k in 0u32..5,
        ) {
            let types_usize = t as usize;
            let pref = vec![vec![0.3; types_usize]; types_usize];
            let (g1, types1) = establishment_game(n, t, k, None, &pref, false, seed).unwrap();
            let (g2, types2) = establishment_game(n, t, k, None, &pref, false, seed).unwrap();
            prop_assert_eq!(types1, types2);
            prop_assert_eq!(g1.ecount(), g2.ecount());
            let m = u32::try_from(g1.ecount()).unwrap();
            for eid in 0..m {
                prop_assert_eq!(g1.edge(eid).unwrap(), g2.edge(eid).unwrap());
            }
        }

        #[test]
        fn diagonal_pref_stays_within_types(
            seed in any::<u64>(),
            n in 10u32..80,
            t in 2u32..5,
            k in 1u32..5,
        ) {
            let types_usize = t as usize;
            let pref: Vec<Vec<f64>> = (0..types_usize)
                .map(|i| {
                    (0..types_usize)
                        .map(|j| if i == j { 0.6 } else { 0.0 })
                        .collect()
                })
                .collect();
            let (g, types) =
                establishment_game(n, t, k, None, &pref, false, seed).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_eq!(types[u as usize], types[v as usize]);
            }
        }

        #[test]
        fn cross_only_pref_yields_cross_edges(
            seed in any::<u64>(),
            n in 10u32..80,
            k in 1u32..5,
        ) {
            // 2 types, only off-diagonal accepts.
            let pref = vec![vec![0.0, 0.7], vec![0.7, 0.0]];
            let (g, types) =
                establishment_game(n, 2, k, Some(&[1.0, 1.0]), &pref, false, seed).unwrap();
            let m = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_ne!(types[u as usize], types[v as usize]);
            }
        }
    }
}
