//! Independent Edge Allocation random multigraph generator (ALGO-GN-031).
//!
//! Counterpart of `igraph_iea_game()` from
//! `references/igraph/src/games/erdos_renyi.c:480-542`.
//!
//! The IEA model generates a random multigraph on `n` vertices with
//! exactly `m` edges. Each edge is independently assigned to an *ordered*
//! vertex pair drawn uniformly at random; when `loops` is `false` the
//! pair `(u, u)` is excluded from the sample space.
//!
//! Because every edge is sampled independently, the model:
//!
//! * Always produces exactly `m` edges (no rejection sampling against
//!   simplicity).
//! * Admits multi-edges by construction — the same vertex pair can be
//!   drawn many times.
//! * Optionally admits self-loops (when `loops = true`).
//!
//! This sampler is **not** uniform over the space of multigraphs. As
//! upstream's docstring notes, the probability of a directed multigraph
//! is proportional to `1 / Π_{i,j} A_ij!` (with the usual double-factorial
//! correction for the undirected diagonal). Two equivalent
//! interpretations:
//!
//! * It uniformly samples **edge-labelled** graphs in which the `m` edges
//!   are distinguishable.
//! * For the special case of simple graphs (i.e. multigraph realisations
//!   without repeated pairs) the IEA model gives every simple graph the
//!   same probability — but conditional on simplicity, the sampler is
//!   equivalent to G(n, m) without replacement.
//!
//! Use [`crate::erdos_renyi_gnm`] when uniform sampling over **simple**
//! graphs is required, or when avoiding multi-edges is a hard constraint.
//!
//! Time complexity: `O(|V| + |E|)` — one PRNG draw (two for `directed &&
//! loops`) per edge plus the final `Graph::add_edges` insertion.
//!
//! ## Scope
//!
//! Direct port of `igraph_iea_game`. No special arguments; mirrors the
//! upstream `(n, m, directed, loops)` signature plus our standard `seed`.
//! The `IGRAPH_EXPERIMENTAL` tag upstream applies to the C API, not to
//! the algorithm — the model itself is well-established (independent
//! edge allocation, sometimes called "edge-labelled Erdős–Rényi" or the
//! "random multigraph with given size").
//!
//! ## References
//!
//! * Janson, S. *"Random graphs and trees"*. The independent-edge-
//!   assignment construction is the textbook starting point for many
//!   multigraph results.
//! * igraph C documentation for `igraph_iea_game` (experimental API).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Maximum addressable edge count (mirrors `IGRAPH_ECOUNT_MAX` —
/// upstream's per-graph edge cap). Keeps allocator pressure bounded.
const IEA_ECOUNT_MAX: u64 = u32::MAX as u64;

fn validate(n: u32, m: u64, loops: bool) -> IgraphResult<()> {
    if m > IEA_ECOUNT_MAX {
        return Err(IgraphError::InvalidArgument(format!(
            "iea_game requested {m} edges, exceeds the {IEA_ECOUNT_MAX} cap"
        )));
    }
    if m == 0 {
        return Ok(());
    }
    if n == 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "iea_game cannot place {m} edges on 0 vertices"
        )));
    }
    if !loops && n < 2 {
        return Err(IgraphError::InvalidArgument(format!(
            "iea_game without loops requires n >= 2 when m > 0 (got n = {n})"
        )));
    }
    Ok(())
}

/// Generate a random multigraph through independent edge allocation.
///
/// * `n` — number of vertices.
/// * `m` — number of edges to draw. The result graph has exactly `m`
///   edges (including any multi-edges or self-loops that occur).
/// * `directed` — generate a directed graph. When `false`, the sampled
///   ordered pair is stored as an undirected edge.
/// * `loops` — when `true`, the sampler may place self-loop edges
///   `(u, u)`. When `false`, the diagonal of the pair space is excluded
///   and every edge connects two distinct vertices.
/// * `seed` — initialises an internal [`SplitMix64`] PRNG. Same
///   `(n, m, directed, loops, seed)` always yields the same graph.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when:
///
/// * `m` exceeds the internal `u32::MAX` edge cap.
/// * `m > 0` and `n == 0` (no vertices to place edges on).
/// * `m > 0`, `loops == false`, and `n < 2` (no non-self pairs exist).
///
/// # Examples
///
/// ```
/// use rust_igraph::iea_game;
///
/// // Directed multigraph: 100 vertices, 500 edges, self-loops allowed.
/// let g = iea_game(100, 500, true, true, 0xDEAD_BEEF).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert_eq!(g.ecount(), 500);
/// assert!(g.is_directed());
/// ```
pub fn iea_game(n: u32, m: u64, directed: bool, loops: bool, seed: u64) -> IgraphResult<Graph> {
    validate(n, m, loops)?;

    if m == 0 {
        return Graph::new(n, directed);
    }

    let mut rng = SplitMix64::new(seed);
    let m_usize = usize::try_from(m).map_err(|_| {
        IgraphError::Internal("iea_game edge count does not fit in usize on this target")
    })?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(m_usize);

    let n_usize = n as usize;
    if loops {
        for _ in 0..m {
            let u = rng.gen_index(n_usize) as VertexId;
            let v = rng.gen_index(n_usize) as VertexId;
            edges.push((u, v));
        }
    } else {
        // Sample u uniformly in [0, n), then v uniformly in [0, n-1).
        // Re-map `v >= u` → `v + 1` to skip the self-loop slot, yielding
        // a uniform draw over [0, n) \ {u}. This is the natural Rust
        // analogue of upstream's `if (c == r) c = n-1;` trick — both
        // achieve a uniform sample of (n-1) distinct values from [0, n)
        // with a single RNG draw, but the shift-by-one variant keeps the
        // sampling argument transparent without leaning on the
        // remap-the-diagonal-row construction the C version uses to
        // amortise an unrelated rejection trick.
        let n_minus_1 = n_usize - 1;
        for _ in 0..m {
            let u = rng.gen_index(n_usize) as VertexId;
            let mut v_idx = rng.gen_index(n_minus_1) as u32;
            if v_idx >= u {
                v_idx += 1;
            }
            edges.push((u, v_idx as VertexId));
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        (0..m)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    #[test]
    fn empty_request_returns_empty_graph() {
        let g = iea_game(50, 0, true, true, 42).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn zero_vertices_with_zero_edges_is_allowed() {
        let g = iea_game(0, 0, false, false, 7).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn zero_vertices_with_edges_is_rejected() {
        let err = iea_game(0, 1, true, true, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn no_loops_requires_two_vertices() {
        let err = iea_game(1, 1, false, false, 1).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
        // With m=0 the constraint relaxes.
        let g = iea_game(1, 0, false, false, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_edge_count_holds_directed_with_loops() {
        let g = iea_game(80, 1234, true, true, 0xBEEF).unwrap();
        assert_eq!(g.vcount(), 80);
        assert_eq!(g.ecount(), 1234);
        assert!(g.is_directed());
    }

    #[test]
    fn exact_edge_count_holds_undirected_no_loops() {
        let g = iea_game(80, 500, false, false, 0x00C0_FFEE).unwrap();
        assert_eq!(g.vcount(), 80);
        assert_eq!(g.ecount(), 500);
        assert!(!g.is_directed());
    }

    #[test]
    fn no_loops_branch_yields_no_self_loops() {
        let g = iea_game(50, 4000, true, false, 0x9999).unwrap();
        for (u, v) in collect_edges(&g) {
            assert_ne!(u, v, "no-loops branch produced self-loop ({u}, {v})");
        }
    }

    #[test]
    fn loops_branch_can_produce_self_loops_eventually() {
        // For n = 5 with 1000 edges and loops allowed, the expected
        // number of self-loops is 1000 / 5 = 200 — overwhelmingly likely
        // to be > 0.
        let g = iea_game(5, 1000, true, true, 0x1234_5678).unwrap();
        let any_self = collect_edges(&g).iter().any(|(u, v)| u == v);
        assert!(any_self, "loops=true should sometimes produce self-loops");
    }

    #[test]
    fn deterministic_with_seed() {
        let a = iea_game(40, 200, true, true, 0xCAFE).unwrap();
        let b = iea_game(40, 200, true, true, 0xCAFE).unwrap();
        assert_eq!(collect_edges(&a), collect_edges(&b));
    }

    #[test]
    fn different_seeds_yield_different_graphs() {
        let a = iea_game(40, 200, true, true, 1).unwrap();
        let b = iea_game(40, 200, true, true, 2).unwrap();
        assert_ne!(
            collect_edges(&a),
            collect_edges(&b),
            "different seeds must produce different graphs"
        );
    }

    #[test]
    fn endpoint_indices_in_range() {
        let n = 30u32;
        let g = iea_game(n, 1000, true, true, 0x42).unwrap();
        for (u, v) in collect_edges(&g) {
            assert!(u < n && v < n, "endpoint out of range: ({u}, {v})");
        }
    }

    #[test]
    fn marginal_endpoint_distribution_is_roughly_uniform() {
        // Each draw uniformly picks an endpoint from [0, n). Across many
        // samples, the per-vertex hit count should concentrate near the
        // expected value n_hits / n. We use a relaxed χ²-style check:
        // every bin must be within ±25% of the mean.
        let n = 8u32;
        let m = 200_000u64;
        let g = iea_game(n, m, true, true, 0xC0DE).unwrap();
        let mut hits = vec![0u64; n as usize];
        for (u, v) in collect_edges(&g) {
            hits[u as usize] += 1;
            hits[v as usize] += 1;
        }
        let expected = (2 * m) as f64 / f64::from(n);
        for (i, &h) in hits.iter().enumerate() {
            let dev = ((h as f64) - expected).abs() / expected;
            assert!(
                dev < 0.05,
                "endpoint {i} deviation {dev} exceeds 5%: hits={h}, expected={expected}"
            );
        }
    }

    #[test]
    fn no_loops_marginal_distribution_is_roughly_uniform() {
        let n = 8u32;
        let m = 200_000u64;
        let g = iea_game(n, m, true, false, 0xD00D).unwrap();
        let mut hits = vec![0u64; n as usize];
        for (u, v) in collect_edges(&g) {
            assert_ne!(u, v);
            hits[u as usize] += 1;
            hits[v as usize] += 1;
        }
        let expected = (2 * m) as f64 / f64::from(n);
        for (i, &h) in hits.iter().enumerate() {
            let dev = ((h as f64) - expected).abs() / expected;
            assert!(
                dev < 0.05,
                "endpoint {i} deviation {dev} exceeds 5%: hits={h}, expected={expected}"
            );
        }
    }

    #[test]
    fn allows_multi_edges() {
        // With n=4 and m=1000, the number of edges far exceeds the 16
        // distinct ordered pairs — multi-edges must appear.
        let g = iea_game(4, 1000, true, true, 0x1357).unwrap();
        let mut counts: HashMap<(VertexId, VertexId), u32> = HashMap::new();
        for e in collect_edges(&g) {
            *counts.entry(e).or_default() += 1;
        }
        let max_mult = counts.values().copied().max().unwrap_or(0);
        assert!(
            max_mult > 1,
            "expected multi-edges with m=1000 on n=4, got max multiplicity {max_mult}"
        );
    }

    #[test]
    fn cap_rejects_overlarge_m() {
        let err = iea_game(2, u64::from(u32::MAX) + 1, true, true, 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(60))]

        #[test]
        fn vcount_and_ecount_invariants(
            n in 2u32..40,
            m in 0u64..400,
            directed in any::<bool>(),
            loops in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = iea_game(n, m, directed, loops, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(g.ecount(), m as usize);
            prop_assert_eq!(g.is_directed(), directed);
        }

        #[test]
        fn no_loops_branch_has_no_self_loops(
            n in 2u32..40,
            m in 1u64..400,
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = iea_game(n, m, directed, false, seed).unwrap();
            let m_u32 = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_u32 {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        #[test]
        fn determinism_is_seed_only(
            n in 2u32..30,
            m in 0u64..150,
            directed in any::<bool>(),
            loops in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let a = iea_game(n, m, directed, loops, seed).unwrap();
            let b = iea_game(n, m, directed, loops, seed).unwrap();
            let am = u32::try_from(a.ecount()).unwrap();
            let bm = u32::try_from(b.ecount()).unwrap();
            prop_assert_eq!(am, bm);
            for eid in 0..am {
                prop_assert_eq!(a.edge(eid).unwrap(), b.edge(eid).unwrap());
            }
        }

        #[test]
        fn endpoints_within_range(
            n in 2u32..40,
            m in 0u64..200,
            directed in any::<bool>(),
            loops in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = iea_game(n, m, directed, loops, seed).unwrap();
            let m_u32 = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m_u32 {
                let (u, v) = g.edge(eid).unwrap();
                prop_assert!(u < n);
                prop_assert!(v < n);
            }
        }
    }
}
