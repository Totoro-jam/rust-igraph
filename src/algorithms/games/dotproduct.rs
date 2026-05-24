//! Random dot-product graph generator (ALGO-GN-022).
//!
//! Counterpart of `igraph_dot_product_game()` from
//! `references/igraph/src/games/dotproduct.c:59-102`.
//!
//! ## Model — Random Dot-Product Graph (RDPG)
//!
//! Every vertex `i ∈ [0, n)` carries a latent position vector
//! `v_i ∈ R^d`. For every unordered pair `(i, j)` with `i < j`
//! (undirected) or ordered pair `(i, j)` with `i ≠ j` (directed), the
//! edge probability is
//!
//! ```text
//!     p_{i,j} = v_i · v_j   (Euclidean inner product, length d)
//! ```
//!
//! and the edge is realised by a single `Bernoulli(p_{i,j})` draw. The
//! latent positions are caller-supplied — this generator only consumes
//! them and never tries to fit them. See Nickel (2006) and
//! Young–Scheinerman (2007) for the construction's properties (degree
//! distribution as a function of position-vector marginals, etc.).
//!
//! ## Clamp semantics (mirroring the C reference exactly)
//!
//! Latent inner products are not guaranteed to live in `[0, 1]`. The
//! C reference resolves the three regimes deterministically:
//!
//! * `prob < 0`  → no edge added. Sets the `negative_warning` flag on
//!   the first occurrence so callers can audit (caller-side warning,
//!   no `eprintln!` here).
//! * `prob > 1`  → edge always added, *no* RNG draw consumed (the C
//!   reference deliberately short-circuits past `RNG_UNIF01`). Sets the
//!   `over_one_warning` flag on the first occurrence.
//! * `0 ≤ prob ≤ 1` → a single `RNG_UNIF01() < prob` draw decides.
//!
//! The PRNG-draw count therefore depends on how many pairs fall in the
//! `[0, 1]` band — exactly mirroring the C kernel. Conformance is
//! structural (vcount, directed flag, simple-by-construction, no
//! self-loops, edge-count bound) rather than bit-exact because
//! [`crate::core::rng::SplitMix64`] is not bit-portable to upstream's
//! glibc-style stream.
//!
//! ## Construction guarantees
//!
//! * **Never** produces a self-loop. The `i == j` slot is short-
//!   circuited explicitly in the directed loop; the undirected loop
//!   starts `j` at `i + 1` so it cannot reach `i`.
//! * **Always** simple. Each (un)ordered pair is inspected exactly
//!   once, so multi-edges are impossible by construction.
//! * Bounded edge count: at most `n · (n − 1) / 2` undirected, at most
//!   `n · (n − 1)` directed.
//!
//! ## Determinism
//!
//! Reproducible given `(vecs, directed, seed)` against the shared
//! [`crate::core::rng::SplitMix64`] PRNG.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Outcome of [`dot_product_game_with_warnings`].
#[derive(Debug, Clone, Copy)]
pub struct DotProductWarnings {
    /// `true` iff at least one pair `(i, j)` had `v_i · v_j < 0`. Those
    /// pairs contribute no edge to the output graph.
    pub had_negative: bool,
    /// `true` iff at least one pair `(i, j)` had `v_i · v_j > 1`. Those
    /// pairs contribute an edge unconditionally (no RNG draw).
    pub had_over_one: bool,
}

fn validate_vecs(vecs: &[Vec<f64>]) -> IgraphResult<(usize, usize)> {
    let n = vecs.len();
    if n == 0 {
        return Ok((0, 0));
    }
    let d = vecs[0].len();
    for (i, v) in vecs.iter().enumerate() {
        if v.len() != d {
            return Err(IgraphError::InvalidArgument(format!(
                "dot_product_game vecs[{i}] has length {} but vecs[0] has length {d}; \
                 every latent position vector must have the same dimension",
                v.len()
            )));
        }
        for (k, &x) in v.iter().enumerate() {
            if !x.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "dot_product_game vecs[{i}][{k}] = {x} is not finite; \
                     NaN/±∞ entries are rejected so the inner-product clamp is well-defined"
                )));
            }
        }
    }
    Ok((n, d))
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    // Vectors are validated to have equal length and finite entries.
    let mut acc = 0.0_f64;
    for k in 0..a.len() {
        acc += a[k] * b[k];
    }
    acc
}

/// Sample a random dot-product graph from caller-supplied latent
/// position vectors and report whether clamp regimes were hit.
///
/// * `vecs[i]` is the latent position vector for vertex `i`. The slice
///   length determines the resulting graph's vertex count `n`.
///   All entries must be finite (no NaN, no `±∞`); every vector must
///   share the same dimension `d`.
/// * `directed` — when `true`, both `(i, j)` and `(j, i)` are sampled
///   independently with their own dot product (the matrix need not be
///   symmetric — even though the dot product itself is, the *draws* are
///   distinct). When `false`, only `i < j` pairs are sampled.
/// * `seed` initialises an internal [`SplitMix64`] PRNG so the output
///   is reproducible given the inputs.
///
/// Returns the generated graph together with [`DotProductWarnings`]
/// flags so the caller can decide whether to log/abort/clip when the
/// latent positions stray outside `[0, 1]`.
///
/// # Errors
///
/// * `vecs[i].len() != vecs[0].len()` for some `i` — every latent
///   vector must share the same dimension.
/// * Any entry is non-finite (NaN or `±∞`).
/// * `vecs.len() > u32::MAX`.
///
/// # Complexity
///
/// `O(n² · d)` — every pair is inspected exactly once, each with a
/// length-`d` dot product. The RNG-draw count is at most one per pair
/// and is strictly less when the `prob > 1` short-circuit fires.
///
/// # Examples
///
/// ```
/// use rust_igraph::dot_product_game_with_warnings;
///
/// // Three vertices on the unit interval (1-D latent space).
/// // p_{0,1} = 0.4·0.6 = 0.24; p_{0,2} = 0.4·0.5 = 0.20; p_{1,2} = 0.30.
/// let vecs = vec![vec![0.4], vec![0.6], vec![0.5]];
/// let (g, warn) = dot_product_game_with_warnings(&vecs, false, 42).unwrap();
/// assert_eq!(g.vcount(), 3);
/// assert!(!g.is_directed());
/// assert!(!warn.had_negative);
/// assert!(!warn.had_over_one);
/// ```
pub fn dot_product_game_with_warnings(
    vecs: &[Vec<f64>],
    directed: bool,
    seed: u64,
) -> IgraphResult<(Graph, DotProductWarnings)> {
    let (n, _d) = validate_vecs(vecs)?;
    let n_u32 = u32::try_from(n).map_err(|_| {
        IgraphError::InvalidArgument(format!(
            "dot_product_game vertex count {n} exceeds u32::MAX"
        ))
    })?;
    if n == 0 {
        return Ok((
            Graph::new(0, directed)?,
            DotProductWarnings {
                had_negative: false,
                had_over_one: false,
            },
        ));
    }

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut had_negative = false;
    let mut had_over_one = false;

    for i in 0..n {
        let i_id = i as VertexId;
        let j_start = if directed { 0 } else { i + 1 };
        for j in j_start..n {
            if i == j {
                continue;
            }
            let prob = dot(&vecs[i], &vecs[j]);
            let j_id = j as VertexId;
            if prob > 1.0 {
                had_over_one = true;
                edges.push((i_id, j_id));
            } else if prob < 0.0 {
                had_negative = true;
                // no edge added, no RNG draw consumed
            } else if rng.gen_unit() < prob {
                edges.push((i_id, j_id));
            }
        }
    }

    let mut g = Graph::new(n_u32, directed)?;
    g.add_edges(edges)?;
    Ok((
        g,
        DotProductWarnings {
            had_negative,
            had_over_one,
        },
    ))
}

/// Sample a random dot-product graph (Nickel 2006) — silent variant.
///
/// Equivalent to [`dot_product_game_with_warnings`] but discards the
/// clamp-regime flags. Use this when the latent positions are known to
/// live in `[0, 1]` and the warning bits are not useful.
///
/// See [`dot_product_game_with_warnings`] for full semantics, error
/// modes, and complexity.
///
/// # Examples
///
/// ```
/// use rust_igraph::dot_product_game;
///
/// let vecs = vec![vec![0.5, 0.1], vec![0.4, 0.2], vec![0.3, 0.3]];
/// let g = dot_product_game(&vecs, false, 7).unwrap();
/// assert_eq!(g.vcount(), 3);
/// // Simple by construction: at most n(n-1)/2 = 3 edges, no self-loops.
/// assert!(g.ecount() <= 3);
/// for e in 0..g.ecount() {
///     let (u, v) = g.edge(e as u32).unwrap();
///     assert_ne!(u, v);
/// }
/// ```
pub fn dot_product_game(vecs: &[Vec<f64>], directed: bool, seed: u64) -> IgraphResult<Graph> {
    dot_product_game_with_warnings(vecs, directed, seed).map(|(g, _)| g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_self_loop(g: &Graph) -> bool {
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            if u == v {
                return true;
            }
        }
        false
    }

    fn is_simple_undirected(g: &Graph) -> bool {
        assert!(!g.is_directed());
        let mut seen: std::collections::HashSet<(VertexId, VertexId)> =
            std::collections::HashSet::new();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            let key = if u <= v { (u, v) } else { (v, u) };
            if !seen.insert(key) {
                return false;
            }
        }
        true
    }

    fn is_simple_directed(g: &Graph) -> bool {
        assert!(g.is_directed());
        let mut seen: std::collections::HashSet<(VertexId, VertexId)> =
            std::collections::HashSet::new();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            if !seen.insert((u, v)) {
                return false;
            }
        }
        true
    }

    #[test]
    fn empty_vecs_produces_empty_graph() {
        let vecs: Vec<Vec<f64>> = Vec::new();
        let g = dot_product_game(&vecs, false, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());

        let g_dir = dot_product_game(&vecs, true, 0).unwrap();
        assert!(g_dir.is_directed());
        assert_eq!(g_dir.vcount(), 0);
    }

    #[test]
    fn single_vertex_no_edges() {
        let vecs = vec![vec![0.5, 0.5]];
        let g = dot_product_game(&vecs, false, 7).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn all_zero_probs_gives_empty_edges() {
        // Every entry zero → every dot product is zero → no edges.
        let vecs = vec![vec![0.0; 3]; 5];
        let g = dot_product_game(&vecs, false, 99).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn unit_probs_gives_complete_graph_undirected() {
        // (1) · (1) = 1.0 — boundary case. With strict `<` Bernoulli,
        // prob == 1 means rng.gen_unit() < 1.0 (always true since
        // gen_unit lives in [0, 1)). Expected: complete graph K_n.
        let n = 6u32;
        let vecs = vec![vec![1.0]; n as usize];
        let g = dot_product_game(&vecs, false, 31).unwrap();
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), (n as usize) * ((n as usize) - 1) / 2);
        assert!(!has_self_loop(&g));
        assert!(is_simple_undirected(&g));
    }

    #[test]
    fn unit_probs_gives_complete_graph_directed() {
        let n = 5u32;
        let vecs = vec![vec![1.0]; n as usize];
        let g = dot_product_game(&vecs, true, 31).unwrap();
        assert_eq!(g.ecount(), (n as usize) * ((n as usize) - 1));
        assert!(!has_self_loop(&g));
        assert!(is_simple_directed(&g));
    }

    #[test]
    fn over_one_short_circuit_adds_edge_no_warn_negative() {
        // vecs[i] = [1.5] => dot = 2.25 > 1. Every pair must be added,
        // had_over_one set, had_negative clear, no RNG draw consumed.
        let vecs = vec![vec![1.5]; 4];
        let (g, warn) = dot_product_game_with_warnings(&vecs, false, 0).unwrap();
        assert_eq!(g.ecount(), 6);
        assert!(warn.had_over_one);
        assert!(!warn.had_negative);
    }

    #[test]
    fn negative_dot_skips_and_warns() {
        // (+1) · (-0.5) = -0.5 < 0. Every cross-pair is negative.
        // We need a layout where every pair has negative dot. Two
        // groups [+1, +1] and [-0.5, -0.5]: + . + = 1 (edge), - . - =
        // 0.25 (chance), + . - = -0.5 (skip). Sandwich two of each so
        // some pairs are negative.
        let vecs = vec![vec![1.0], vec![1.0], vec![-0.5], vec![-0.5]];
        let (_, warn) = dot_product_game_with_warnings(&vecs, false, 11).unwrap();
        assert!(warn.had_negative);
    }

    #[test]
    fn directed_matrix_need_not_be_symmetric() {
        // Directed mode iterates *all* ordered i!=j pairs. Even though
        // dot(v_i, v_j) == dot(v_j, v_i) here, each ordered pair gets
        // its OWN RNG draw so the graph is not forced to be symmetric.
        let vecs = vec![vec![0.5]; 8];
        let g = dot_product_game(&vecs, true, 12345).unwrap();
        let n = vecs.len();
        assert!(g.is_directed());
        assert!(g.ecount() <= n * (n - 1));
        assert!(!has_self_loop(&g));
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let vecs = vec![
            vec![0.7, 0.2],
            vec![0.3, 0.4],
            vec![0.1, 0.5],
            vec![0.6, 0.6],
        ];
        let g1 = dot_product_game(&vecs, false, 0xDEAD_BEEF).unwrap();
        let g2 = dot_product_game(&vecs, false, 0xDEAD_BEEF).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        for e in 0..g1.ecount() {
            assert_eq!(g1.edge(e as u32).unwrap(), g2.edge(e as u32).unwrap());
        }
    }

    #[test]
    fn determinism_different_seed_likely_differs() {
        // Probabilities sit well inside (0, 1) so different seeds give
        // different realisations almost surely.
        let vecs = vec![
            vec![0.5, 0.4],
            vec![0.3, 0.5],
            vec![0.4, 0.3],
            vec![0.6, 0.2],
            vec![0.2, 0.6],
            vec![0.5, 0.5],
            vec![0.3, 0.3],
            vec![0.4, 0.4],
        ];
        let g1 = dot_product_game(&vecs, false, 1).unwrap();
        let g2 = dot_product_game(&vecs, false, 2).unwrap();
        let edges_of = |g: &Graph| {
            let mut v: Vec<(VertexId, VertexId)> =
                (0..g.ecount()).map(|e| g.edge(e as u32).unwrap()).collect();
            v.sort_unstable();
            v
        };
        assert_ne!(edges_of(&g1), edges_of(&g2));
    }

    #[test]
    fn mismatched_dim_errors() {
        let vecs = vec![vec![0.1, 0.2], vec![0.3]];
        let err = dot_product_game(&vecs, false, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("dimension")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn nan_in_vec_errors() {
        let vecs = vec![vec![0.1, f64::NAN], vec![0.2, 0.3]];
        let err = dot_product_game(&vecs, false, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("finite")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn inf_in_vec_errors() {
        let vecs = vec![vec![f64::INFINITY], vec![0.5]];
        assert!(dot_product_game(&vecs, false, 0).is_err());
    }

    #[test]
    fn zero_dimension_yields_zero_dot_products() {
        // d = 0: every dot product is the empty sum = 0 → no edges.
        let vecs = vec![Vec::<f64>::new(); 4];
        let g = dot_product_game(&vecs, false, 0).unwrap();
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 0);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn vecs_strategy() -> impl Strategy<Value = Vec<Vec<f64>>> {
        // n ∈ [0, 8], d ∈ [1, 4], entries ∈ [-0.5, 1.5] so all three
        // clamp regimes are exercised.
        (1usize..=4).prop_flat_map(|d| {
            prop::collection::vec(prop::collection::vec(-0.5f64..1.5, d..=d), 0usize..=8)
        })
    }

    proptest! {
        #[test]
        fn never_self_loop(
            vecs in vecs_strategy(),
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = dot_product_game(&vecs, directed, seed).unwrap();
            for e in 0..g.ecount() {
                let (u, v) = g.edge(e as u32).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        #[test]
        fn always_simple(
            vecs in vecs_strategy(),
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = dot_product_game(&vecs, directed, seed).unwrap();
            let mut seen: std::collections::HashSet<(VertexId, VertexId)> =
                std::collections::HashSet::new();
            for e in 0..g.ecount() {
                let (u, v) = g.edge(e as u32).unwrap();
                let key = if directed {
                    (u, v)
                } else if u <= v {
                    (u, v)
                } else {
                    (v, u)
                };
                prop_assert!(seen.insert(key));
            }
        }

        #[test]
        fn vcount_matches_input(
            vecs in vecs_strategy(),
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = dot_product_game(&vecs, directed, seed).unwrap();
            prop_assert_eq!(g.vcount() as usize, vecs.len());
            prop_assert_eq!(g.is_directed(), directed);
        }

        #[test]
        fn edge_count_within_bounds(
            vecs in vecs_strategy(),
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g = dot_product_game(&vecs, directed, seed).unwrap();
            let n = vecs.len();
            let bound = if directed {
                n.saturating_mul(n.saturating_sub(1))
            } else {
                n.saturating_mul(n.saturating_sub(1)) / 2
            };
            prop_assert!(g.ecount() <= bound);
        }

        #[test]
        fn determinism(
            vecs in vecs_strategy(),
            directed in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let g1 = dot_product_game(&vecs, directed, seed).unwrap();
            let g2 = dot_product_game(&vecs, directed, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            for e in 0..g1.ecount() {
                prop_assert_eq!(
                    g1.edge(e as u32).unwrap(),
                    g2.edge(e as u32).unwrap()
                );
            }
        }
    }
}
