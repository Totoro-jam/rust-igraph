//! Edge-switching MCMC degree-sequence simple-graph sampler
//! (ALGO-GN-028).
//!
//! Counterpart of the `IGRAPH_DEGSEQ_EDGE_SWITCHING_SIMPLE` branch of
//! `igraph_degree_sequence_game()` in
//! `references/igraph/src/games/degree_sequence.c`
//! (`edge_switching` lines 712-722). The reference dispatches to
//! `igraph_realize_degree_sequence(..., IGRAPH_SIMPLE_SW,
//! IGRAPH_REALIZE_DEGSEQ_INDEX)` for the deterministic seed, then runs
//! `igraph_rewire(graph, 10 * ecount, IGRAPH_SIMPLE_SW, NULL)` for the
//! MCMC mixing phase. We inline both phases to keep this AWU
//! self-contained.
//!
//! ## Phase 1 — deterministic seed via Havel–Hakimi (undirected) /
//! Kleitman–Wang (directed), INDEX order
//!
//! * **Undirected**: maintain `residual[v] = d_v` (mutable copy). For
//!   each `i ∈ [0, n)`, treat vertex `i` as the hub with residual
//!   degree `r = residual[i]`. Collect the `r` other vertices with the
//!   highest current residual degrees (skipping `i`). Add edges
//!   `(i, spoke)` and decrement each spoke's residual. Set
//!   `residual[i] = 0`. If at any step fewer than `r` valid spokes
//!   exist the sequence is non-graphical (rejected up front by the
//!   shared Erdős–Gallai pre-check, so this is only a guard).
//! * **Directed**: same loop over vertices in index order. At step
//!   `i`, sort the remaining vertices by `(in-residual, out-residual)`
//!   descending and connect vertex `i`'s `out_residual[i]` out-stubs
//!   to the top-`r` distinct vertices (skipping `i` itself),
//!   decrementing their in-residuals. Mirrors `igraph_i_kleitman_wang_index`.
//!
//! These seed builders are RNG-free. The Erdős–Gallai
//! / Fulkerson–Chen–Anstee pre-checks (shared via `pub(crate)` helpers
//! in [`crate::algorithms::games::degree_sequence_fast_heur`]) guarantee
//! they always succeed when called from this entry point.
//!
//! ## Phase 2 — degree-preserving edge-switching MCMC
//!
//! The mixing kernel runs `10 · |E|` trials (matches the upstream
//! constant). Each trial:
//!
//! 1. Sample `e1 ← RNG(0, |E|-1)`, then `e2 ← RNG(0, |E|-1)` until
//!    `e1 ≠ e2`.
//! 2. Let `(a,b)` = edge `e1`, `(c,d)` = edge `e2`.
//! 3. **Undirected only**: with probability 0.5, swap `c` and `d`. This
//!    is what lets the MCMC consider both rewiring orientations from
//!    one pair of edges.
//! 4. Reject if `a == c` or `b == d` (the swap would be a no-op).
//! 5. Reject if `a == d` or `b == c` (the swap would create a
//!    self-loop, and the simple-graph constraint forbids self-loops).
//! 6. Reject if the rewired endpoints `(a,d)` or `(c,b)` already form
//!    an edge in the current graph (would create a multi-edge).
//! 7. Otherwise apply the swap: replace `(a,b),(c,d)` with
//!    `(a,d),(c,b)` in both the edge list and the adjacency tracker.
//!
//! **Both successful and failed trials count toward the `10 · |E|`
//! budget.** This is necessary for the chain to be detailed-balanced;
//! conditioning on success would bias the stationary distribution.
//!
//! ## vs. siblings
//!
//! * [`crate::degree_sequence_game_configuration`] (ALGO-GN-024) — no
//!   simplicity guarantee; produces a multigraph.
//! * [`crate::degree_sequence_game_configuration_simple`] (ALGO-GN-027)
//!   — exact-uniform rejection sampler on the simple-graph space; cost
//!   grows as `exp(O((Σd/n)²))`. This MCMC variant trades exact
//!   uniformity for `O(|E|)` runtime independent of density.
//! * [`crate::degree_sequence_game_fast_heur_simple`] (ALGO-GN-026) —
//!   biased single-pass heuristic; fastest but not asymptotically
//!   uniform.
//! * [`crate::degree_sequence_game_vl`] (ALGO-GN-025) — Viger–Latapy
//!   MCMC on **simple connected** graphs; trades the connectivity
//!   guarantee for tighter mixing bounds.
//!
//! ## Determinism
//!
//! A single `SplitMix64` seed drives both the (RNG-free) seed phase
//! and every rewire trial. The PRNG is not bitwise portable to igraph C
//! / `NumPy` / R, so the three-source conformance harness asserts
//! structural invariants only (vcount, ecount, exact degree match,
//! simplicity).
//!
//! ## Failure modes
//!
//! Non-graphical input is rejected up front by Erdős–Gallai (undirected)
//! or Fulkerson–Chen–Anstee (directed). The seed builder and MCMC kernel
//! never abort after that point: the sampler is best-effort and returns
//! whatever simple graph the chain has reached after `10 · |E|` trials,
//! exactly mirroring upstream semantics.

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

/// Upstream constant: number of rewire trials per edge. The C reference
/// hard-codes `10 * igraph_ecount(graph)` in `edge_switching()` at
/// `references/igraph/src/games/degree_sequence.c:719`.
pub const REWIRE_TRIALS_PER_EDGE: u64 = 10;

/// Sample uniformly from `[low, high]` inclusive. Mirrors the
/// `RNG_INTEGER(low, high)` semantics of the C reference.
fn rng_integer_inclusive(rng: &mut SplitMix64, low: usize, high: usize) -> usize {
    debug_assert!(low <= high, "rng_integer_inclusive: low ≤ high");
    let span = (high - low) as u64 + 1;
    low + (rng.next_u64() % span) as usize
}

/// One unbiased coin flip. Matches `RNG_BOOL()` in the C reference.
fn rng_bool(rng: &mut SplitMix64) -> bool {
    (rng.next_u64() & 1) == 1
}

// ---------------------------------------------------------------- seed: undirected

fn build_seed_undirected(degrees: &[u32]) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let n = degrees.len();
    let total_u64 = checked_sum(degrees)?;
    if total_u64 % 2 != 0 {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_edge_switching_simple: undirected degree sum must be even"
                .to_string(),
        ));
    }
    let ecount = (total_u64 / 2) as usize;
    if ecount == 0 {
        return Ok(Vec::new());
    }

    let mut residual: Vec<i64> = degrees.iter().map(|&d| i64::from(d)).collect();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(ecount);
    // Reused scratch buffer for candidate spokes.
    let mut order: Vec<u32> = Vec::with_capacity(n);

    for i in 0..n {
        let r = residual[i];
        if r <= 0 {
            continue;
        }
        // Hub is fully used after this step; mark it now so it can't be
        // accidentally picked as a spoke.
        residual[i] = 0;

        // Collect all vertices with positive residual, then take the
        // top-r by residual (descending), ties by index ascending.
        order.clear();
        for j in 0..n {
            if j == i {
                continue;
            }
            if residual[j] > 0 {
                order.push(u32::try_from(j).map_err(|_| {
                    IgraphError::Internal("vertex index exceeds u32 (seed builder)")
                })?);
            }
        }
        let r_usize = usize::try_from(r).map_err(|_| {
            IgraphError::Internal(
                "residual degree negative in Havel–Hakimi (should be unreachable)",
            )
        })?;
        if order.len() < r_usize {
            // Should be impossible given the EG pre-check, but the
            // upstream library raises EINVAL here.
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_edge_switching_simple: degree sequence is not simply realisable (Havel–Hakimi)"
                    .to_string(),
            ));
        }
        // Sort descending by residual; ties broken by vertex index
        // ascending (stable, deterministic).
        order.sort_by(|&a, &b| {
            residual[b as usize]
                .cmp(&residual[a as usize])
                .then(a.cmp(&b))
        });

        for &spoke in &order[..r_usize] {
            edges.push((
                u32::try_from(i)
                    .map_err(|_| IgraphError::Internal("vertex index exceeds u32 (seed hub)"))?,
                spoke,
            ));
            residual[spoke as usize] -= 1;
        }
    }

    Ok(edges)
}

// ---------------------------------------------------------------- seed: directed

fn build_seed_directed(
    out_degrees: &[u32],
    in_degrees: &[u32],
) -> IgraphResult<Vec<(VertexId, VertexId)>> {
    let n = out_degrees.len();
    let out_total = checked_sum(out_degrees)? as usize;
    let in_total = checked_sum(in_degrees)? as usize;
    if out_total != in_total {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_edge_switching_simple: directed sums Σout and Σin must match"
                .to_string(),
        ));
    }
    if out_total == 0 {
        return Ok(Vec::new());
    }

    let mut out_residual: Vec<i64> = out_degrees.iter().map(|&d| i64::from(d)).collect();
    let mut in_residual: Vec<i64> = in_degrees.iter().map(|&d| i64::from(d)).collect();
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(out_total);
    let mut order: Vec<u32> = Vec::with_capacity(n);

    for i in 0..n {
        let r = out_residual[i];
        if r <= 0 {
            continue;
        }
        out_residual[i] = 0;

        order.clear();
        for j in 0..n {
            if j == i {
                continue;
            }
            if in_residual[j] > 0 {
                order.push(u32::try_from(j).map_err(|_| {
                    IgraphError::Internal("vertex index exceeds u32 (directed seed)")
                })?);
            }
        }
        let r_usize = usize::try_from(r).map_err(|_| {
            IgraphError::Internal("out residual negative in Kleitman–Wang (should be unreachable)")
        })?;
        if order.len() < r_usize {
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_edge_switching_simple: directed degree pair is not simply realisable (Kleitman–Wang)"
                    .to_string(),
            ));
        }
        // Mirrors `degree_greater<vbd_pair>` ordering used by the
        // upstream Kleitman–Wang INDEX implementation: descending by
        // (in_residual, out_residual). Ties broken by vertex index
        // ascending for deterministic output.
        order.sort_by(|&a, &b| {
            in_residual[b as usize]
                .cmp(&in_residual[a as usize])
                .then(out_residual[b as usize].cmp(&out_residual[a as usize]))
                .then(a.cmp(&b))
        });

        for &target in &order[..r_usize] {
            edges.push((
                u32::try_from(i).map_err(|_| {
                    IgraphError::Internal("vertex index exceeds u32 (directed hub)")
                })?,
                target,
            ));
            in_residual[target as usize] -= 1;
        }
    }

    Ok(edges)
}

// ---------------------------------------------------------------- rewire phase

fn rewire_undirected(
    n_vertices: u32,
    edges: &mut [(VertexId, VertexId)],
    rng: &mut SplitMix64,
    trials: u64,
) {
    let m = edges.len();
    if m < 2 {
        return;
    }
    let vcount = n_vertices as usize;
    let mut adj: Vec<HashSet<u32>> = (0..vcount).map(|_| HashSet::new()).collect();
    for &(u, v) in edges.iter() {
        adj[u as usize].insert(v);
        adj[v as usize].insert(u);
    }

    let mut trial: u64 = 0;
    while trial < trials {
        trial += 1;

        let e1 = rng_integer_inclusive(rng, 0, m - 1);
        let mut e2;
        loop {
            e2 = rng_integer_inclusive(rng, 0, m - 1);
            if e2 != e1 {
                break;
            }
        }

        let (a, b) = edges[e1];
        let (mut c, mut d) = edges[e2];

        // Undirected: with prob 0.5, consider the swapped orientation
        // of the second edge. The rewriting then becomes
        // (a,b),(d,c) → (a,c),(d,b), which covers the second of the
        // two valid degree-preserving rewirings.
        if rng_bool(rng) {
            std::mem::swap(&mut c, &mut d);
        }

        // Reject no-ops and self-loops.
        if a == c || b == d {
            continue;
        }
        if a == d || b == c {
            continue;
        }
        // Reject if the rewired edges would collide with existing ones.
        if adj[a as usize].contains(&d) {
            continue;
        }
        if adj[c as usize].contains(&b) {
            continue;
        }

        // Apply: (a,b),(c,d) → (a,d),(c,b).
        // Adjacency: drop b from a, drop a from b, drop d from c, drop
        // c from d; add d to a, a to d, b to c, c to b.
        adj[a as usize].remove(&b);
        adj[b as usize].remove(&a);
        adj[c as usize].remove(&d);
        adj[d as usize].remove(&c);
        adj[a as usize].insert(d);
        adj[d as usize].insert(a);
        adj[c as usize].insert(b);
        adj[b as usize].insert(c);

        // Preserve the same internal orientation as the original edge so
        // the edge-list direction stays stable across trials (cosmetic;
        // both orientations represent the same undirected edge).
        edges[e1] = (a, d);
        edges[e2] = (c, b);
    }
}

fn rewire_directed(
    n_vertices: u32,
    edges: &mut [(VertexId, VertexId)],
    rng: &mut SplitMix64,
    trials: u64,
) {
    let m = edges.len();
    if m < 2 {
        return;
    }
    let vcount = n_vertices as usize;
    // For directed graphs the multi-arc check is on out-adjacency:
    // (a,d) is a multi-arc only if a→d already exists.
    let mut out_adj: Vec<HashSet<u32>> = (0..vcount).map(|_| HashSet::new()).collect();
    for &(u, v) in edges.iter() {
        out_adj[u as usize].insert(v);
    }

    let mut trial: u64 = 0;
    while trial < trials {
        trial += 1;

        let e1 = rng_integer_inclusive(rng, 0, m - 1);
        let mut e2;
        loop {
            e2 = rng_integer_inclusive(rng, 0, m - 1);
            if e2 != e1 {
                break;
            }
        }

        let (a, b) = edges[e1];
        let (c, d) = edges[e2];

        // Directed: there is only one valid degree-preserving rewiring
        // of two arcs (a→b),(c→d) — namely (a→d),(c→b). No coin flip.
        if a == c || b == d {
            continue; // no-op
        }
        if a == d || b == c {
            continue; // would create a self-loop
        }
        if out_adj[a as usize].contains(&d) {
            continue; // multi-arc
        }
        if out_adj[c as usize].contains(&b) {
            continue; // multi-arc
        }

        out_adj[a as usize].remove(&b);
        out_adj[c as usize].remove(&d);
        out_adj[a as usize].insert(d);
        out_adj[c as usize].insert(b);

        edges[e1] = (a, d);
        edges[e2] = (c, b);
    }
}

// ---------------------------------------------------------------- public API

/// Sample a random simple graph realising the given degree sequence via
/// degree-preserving edge-switching MCMC (ALGO-GN-028).
///
/// Mirrors `IGRAPH_DEGSEQ_EDGE_SWITCHING_SIMPLE`. The algorithm builds
/// a deterministic Havel–Hakimi (undirected) or Kleitman–Wang
/// (directed) seed graph by INDEX order, then runs `10 · |E|`
/// degree-preserving edge swaps (Markov-chain Monte Carlo). Both
/// successful and failed swap trials count toward the budget, which is
/// what makes the chain detailed-balanced.
///
/// The output is guaranteed to be a simple graph (no self-loops, no
/// multi-edges / multi-arcs) realising the prescribed sequence
/// exactly. The output distribution converges to the uniform
/// distribution on simple realisations as the number of trials grows
/// — the upstream `10 · |E|` constant is a pragmatic compromise
/// between mixing quality and wall-clock cost, not a tight bound.
///
/// # Arguments
///
/// * `out_degrees` — undirected mode: the degree of each vertex.
///   Directed mode: the out-degree of each vertex.
/// * `in_degrees`:
///   * `None` → undirected. Requires `Σ out_degrees` even and the
///     sequence to satisfy Erdős–Gallai.
///   * `Some(in_seq)` → directed. Requires equal lengths, equal sums,
///     and the pair to satisfy Fulkerson–Chen–Anstee.
/// * `seed` — drives a `SplitMix64` PRNG. The same
///   `(out_degrees, in_degrees, seed)` triple always produces the same
///   graph.
///
/// # Errors
///
/// Returns `IgraphError::InvalidArgument` if the sequence is
/// non-graphical (rejected by Erdős–Gallai or Fulkerson–Chen–Anstee).
/// The MCMC phase itself cannot fail.
///
/// # Examples
///
/// ```
/// use rust_igraph::degree_sequence_game_edge_switching_simple;
///
/// // 4-cycle: every vertex has degree 2.
/// let g = degree_sequence_game_edge_switching_simple(&[2, 2, 2, 2], None, 17).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// assert!(!g.is_directed());
/// ```
pub fn degree_sequence_game_edge_switching_simple(
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
                "degree_sequence_game_edge_switching_simple: out_degrees and in_degrees must have the same length"
                    .to_string(),
            ));
        }
    }

    if directed {
        let Some(in_seq) = in_degrees else {
            return Err(IgraphError::InvalidArgument(
                "directed graph requires in_degrees".to_string(),
            ));
        };
        if !is_graphical_simple_directed(out_degrees, in_seq) {
            return Err(IgraphError::InvalidArgument(
                "degree_sequence_game_edge_switching_simple: degree pair is not realisable as a simple directed graph (Fulkerson–Chen–Anstee)"
                    .to_string(),
            ));
        }
    } else if !is_graphical_simple_undirected(out_degrees) {
        return Err(IgraphError::InvalidArgument(
            "degree_sequence_game_edge_switching_simple: degree sequence is not realisable as a simple undirected graph (Erdős–Gallai)"
                .to_string(),
        ));
    }

    let mut rng = SplitMix64::new(seed);

    let mut edges = if directed {
        let Some(in_seq) = in_degrees else {
            return Err(IgraphError::InvalidArgument(
                "directed graph requires in_degrees".to_string(),
            ));
        };
        build_seed_directed(out_degrees, in_seq)?
    } else {
        build_seed_undirected(out_degrees)?
    };

    let trials = REWIRE_TRIALS_PER_EDGE
        .checked_mul(edges.len() as u64)
        .ok_or(IgraphError::Internal(
            "rewire trial count overflowed u64 (edge count too large)",
        ))?;

    if directed {
        rewire_directed(n, &mut edges, &mut rng, trials);
    } else {
        rewire_undirected(n, &mut edges, &mut rng, trials);
    }

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
        let g = degree_sequence_game_edge_switching_simple(&[], None, 1).expect("empty ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn undirected_singleton_zero_yields_isolated_vertex() {
        let g = degree_sequence_game_edge_switching_simple(&[0], None, 1).expect("singleton ok");
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_all_isolated_n5_yields_no_edges() {
        let g = degree_sequence_game_edge_switching_simple(&[0; 5], None, 42).expect("ok");
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn undirected_4cycle_preserves_degrees_and_is_simple() {
        let g = degree_sequence_game_edge_switching_simple(&[2, 2, 2, 2], None, 7).expect("ok");
        assert_eq!(g.vcount(), 4);
        assert_eq!(g.ecount(), 4);
        assert_eq!(observed_degrees(&g), vec![2, 2, 2, 2]);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_3regular_n6_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 6];
        let g = degree_sequence_game_edge_switching_simple(&degrees, None, 0xABCD_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_skewed_powerlaw_preserves_degrees() {
        let degrees: Vec<u32> = vec![5, 4, 4, 3, 3, 3, 2, 2, 2, 2];
        let g = degree_sequence_game_edge_switching_simple(&degrees, None, 0xC0FE_u64).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_3regular_n30_preserves_degrees() {
        let degrees: Vec<u32> = vec![3; 30];
        let g = degree_sequence_game_edge_switching_simple(&degrees, None, 0xDEAD_F00D_u64)
            .expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    /// Dense regime that the `configuration_simple` rejection sampler
    /// (ALGO-GN-027) can struggle with. The MCMC variant should handle
    /// it cleanly because cost is linear in `|E|`.
    #[test]
    fn undirected_dense_5regular_n20_preserves_degrees() {
        let degrees: Vec<u32> = vec![5; 20];
        let g = degree_sequence_game_edge_switching_simple(&degrees, None, 9001).expect("ok");
        assert_eq!(observed_degrees(&g), degrees);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn undirected_odd_sum_rejected() {
        let err = degree_sequence_game_edge_switching_simple(&[1, 1, 1], None, 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn undirected_non_graphical_rejected_max_too_large() {
        let err = degree_sequence_game_edge_switching_simple(&[5, 3, 1, 1], None, 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn deterministic_same_seed_undirected() {
        let degrees: Vec<u32> = vec![3; 8];
        let g1 = degree_sequence_game_edge_switching_simple(&degrees, None, 4242).expect("ok");
        let g2 = degree_sequence_game_edge_switching_simple(&degrees, None, 4242).expect("ok");
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
        let g = degree_sequence_game_edge_switching_simple(&[], Some(&[]), 1).expect("ok");
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn directed_2cycle_preserves_in_out() {
        let g = degree_sequence_game_edge_switching_simple(&[1, 1], Some(&[1, 1]), 9).expect("ok");
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
            degree_sequence_game_edge_switching_simple(&out, Some(&inv), 0xC0DE_u64).expect("ok");
        let (got_out, got_in) = observed_out_in(&g);
        assert_eq!(got_out, out);
        assert_eq!(got_in, inv);
        assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
    }

    #[test]
    fn directed_unequal_sums_rejected() {
        let err = degree_sequence_game_edge_switching_simple(&[1, 1, 1], Some(&[1, 1, 0]), 1)
            .unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn directed_length_mismatch_rejected() {
        let err =
            degree_sequence_game_edge_switching_simple(&[1, 1], Some(&[1, 1, 0]), 1).unwrap_err();
        matches!(err, IgraphError::InvalidArgument(_));
    }

    #[test]
    fn directed_deterministic_same_seed() {
        let out = vec![2u32; 5];
        let inv = vec![2u32; 5];
        let g1 = degree_sequence_game_edge_switching_simple(&out, Some(&inv), 12345).expect("ok");
        let g2 = degree_sequence_game_edge_switching_simple(&out, Some(&inv), 12345).expect("ok");
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

    /// Generate graphical undirected degree sequences. We allow a
    /// wider density envelope than the GN-027 strategy because this
    /// sampler's per-call cost is linear in `|E|` regardless of
    /// density (no rejection restarts).
    fn graphical_undirected_strategy() -> impl Strategy<Value = Vec<u32>> {
        (4usize..=12).prop_flat_map(|n| {
            let cap = (n as u32).saturating_sub(1);
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
        fn degrees_preserved_undirected(
            seq in graphical_undirected_strategy(),
            seed in any::<u64>(),
        ) {
            let g = degree_sequence_game_edge_switching_simple(&seq, None, seed)
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
        fn simple_no_loops_no_multi_undirected(
            seq in graphical_undirected_strategy(),
            seed in any::<u64>(),
        ) {
            use crate::algorithms::properties::{SimpleMode, is_simple_with_mode};
            let g = degree_sequence_game_edge_switching_simple(&seq, None, seed)
                .expect("graphical sequence must succeed");
            prop_assert!(is_simple_with_mode(&g, SimpleMode::DirectedAsDirected).unwrap());
        }

        #[test]
        fn same_seed_same_graph(
            seq in graphical_undirected_strategy(),
            seed in any::<u64>(),
        ) {
            let g1 = degree_sequence_game_edge_switching_simple(&seq, None, seed).unwrap();
            let g2 = degree_sequence_game_edge_switching_simple(&seq, None, seed).unwrap();
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
