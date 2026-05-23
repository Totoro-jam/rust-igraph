//! Growing random graph (ALGO-GN-003).
//!
//! Counterpart of `igraph_growing_random_game()` in
//! `references/igraph/src/games/growing_random.c:55-105`.
//!
//! The model starts with a single vertex and grows the graph one step
//! at a time. On each step a fresh vertex is added together with `m`
//! new edges. Two endpoint-selection rules are supported:
//!
//! * **Citation mode** (`citation = true`): every new edge originates
//!   at the freshly-added vertex and connects to a uniformly random
//!   earlier vertex. Each new edge therefore points from `i` (the
//!   freshly-added id) to some `to ∈ [0, i - 1]`.
//! * **Free mode** (`citation = false`): both endpoints are uniformly
//!   sampled — `from` from `[0, i]` (the new vertex itself is
//!   allowed) and `to` from `[1, i]`. The asymmetric bounds are
//!   inherited from upstream, where the closed interval `[0, i]`
//!   includes the new vertex and `[1, i]` excludes vertex 0 from the
//!   sink side to avoid pinning every edge at the seed.
//!
//! Compared with the BAG variant of Barabási–Albert
//! ([`crate::barabasi_game_bag`]) the per-step kernel here is *uniform*
//! rather than degree-proportional — the bag bookkeeping disappears and
//! every step is a constant-time RNG draw. The resulting degree
//! distribution decays exponentially (not as a power law).
//!
//! Time complexity: `O(|V| + |E|) = O(n · m)`. Memory is the edge list
//! only: `O(n · m)`.
//!
//! ## Scope
//!
//! The full upstream call signature is `(graph, n, m, directed,
//! citation)` — exposed here as four arguments plus a `seed`. There are
//! no special parameters or `start_from` hooks to translate.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate_inputs(n: u32, m: u32) -> IgraphResult<()> {
    let n_u64 = u64::from(n);
    let m_u64 = u64::from(m);
    let steps = n_u64.saturating_sub(1);
    let edges = steps
        .checked_mul(m_u64)
        .ok_or(IgraphError::Internal("growing_random edge count overflow"))?;
    // Allocation bound: each (VertexId, VertexId) is 8 bytes; the
    // allocator caps total bytes at isize::MAX. We use a conservative
    // hard cap of u32::MAX edges (matching the upstream IGRAPH_ECOUNT_MAX
    // convention) — well below any platform's allocator limit and still
    // far beyond what practical graphs ever ask for.
    if edges > u64::from(u32::MAX) {
        return Err(IgraphError::Internal(
            "growing_random edge count exceeds u32::MAX",
        ));
    }
    Ok(())
}

/// Generate a growing random graph on `n` vertices, adding `m` new
/// edges per step.
///
/// * `n` — vertex count. `n = 0` returns an empty graph; `n = 1`
///   returns a single isolated vertex.
/// * `m` — number of new edges contributed per added vertex.
///   `m = 0` yields `n` isolated vertices.
/// * `directed` — generate a directed graph.
/// * `citation` — if `true`, every new edge originates at the
///   freshly-added vertex; if `false`, both endpoints are uniformly
///   sampled within the current frontier.
/// * `seed` — initialises an internal [`SplitMix64`] PRNG. Same
///   `(n, m, directed, citation, seed)` always yields the same graph.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] if `(n - 1) · m` overflows `u64`
/// (only possible at `(u32::MAX, u32::MAX)` scale).
///
/// # Examples
///
/// ```
/// use rust_igraph::growing_random_game;
///
/// // 50-vertex directed citation graph with 2 new edges per step.
/// // Exactly (n - 1) · m = 49 · 2 = 98 edges.
/// let g = growing_random_game(50, 2, true, true, 0xCAFE).unwrap();
/// assert_eq!(g.vcount(), 50);
/// assert_eq!(g.ecount(), 98);
/// assert!(g.is_directed());
/// ```
pub fn growing_random_game(
    n: u32,
    m: u32,
    directed: bool,
    citation: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate_inputs(n, m)?;

    if n == 0 {
        return Graph::new(0, directed);
    }
    if n == 1 || m == 0 {
        return Graph::new(n, directed);
    }

    let m_usize = m as usize;
    let steps = (n as usize) - 1;
    let total_edges = steps
        .checked_mul(m_usize)
        .ok_or(IgraphError::Internal("growing_random edge total overflow"))?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);
    let mut rng = SplitMix64::new(seed);

    for i in 1..n {
        for _ in 0..m {
            let (from, to) = if citation {
                // RNG_INTEGER(0, i - 1) in C — closed interval, so 0 ≤ to ≤ i - 1.
                let to = rng.gen_index(i as usize) as VertexId;
                (i, to)
            } else {
                // RNG_INTEGER(0, i) and RNG_INTEGER(1, i) — both closed intervals.
                let from = rng.gen_index((i as usize) + 1) as VertexId;
                let to = (rng.gen_index(i as usize) as VertexId) + 1;
                (from, to)
            };
            edges.push((from, to));
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_edges(g: &Graph) -> Vec<(VertexId, VertexId)> {
        let n_edges = u32::try_from(g.ecount()).expect("ecount fits in u32 in tests");
        (0..n_edges)
            .map(|eid| g.edge(eid).expect("edge id in bounds"))
            .collect()
    }

    #[test]
    fn n_zero_returns_empty_graph() {
        let g = growing_random_game(0, 3, true, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_returns_singleton() {
        let g = growing_random_game(1, 5, true, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn m_zero_returns_edgeless() {
        let g = growing_random_game(20, 0, true, false, 1).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_edge_count_directed_constant_m() {
        for &(n, m) in &[(10u32, 1u32), (10, 3), (50, 2), (200, 5)] {
            let g = growing_random_game(n, m, true, true, 0x00C0_FFEE).unwrap();
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount(), (n as usize - 1) * m as usize);
        }
    }

    #[test]
    fn exact_edge_count_undirected_free_mode() {
        // Edge count is independent of mode/direction.
        let g = growing_random_game(100, 4, false, false, 0xDEAD_BEEF).unwrap();
        assert_eq!(g.vcount(), 100);
        assert_eq!(g.ecount(), 99 * 4);
        assert!(!g.is_directed());
    }

    #[test]
    fn deterministic_with_seed() {
        let a = growing_random_game(80, 3, true, true, 0xABCD).unwrap();
        let b = growing_random_game(80, 3, true, true, 0xABCD).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        let ea: Vec<_> = collect_edges(&a);
        let eb: Vec<_> = collect_edges(&b);
        assert_eq!(ea, eb);
    }

    #[test]
    fn different_seeds_yield_different_graphs() {
        let a = growing_random_game(60, 2, true, false, 1).unwrap();
        let b = growing_random_game(60, 2, true, false, 2).unwrap();
        let ea: Vec<_> = collect_edges(&a);
        let eb: Vec<_> = collect_edges(&b);
        assert_ne!(ea, eb, "different seeds must produce different graphs");
    }

    #[test]
    fn citation_mode_directed_has_strict_temporal_order() {
        // Citation edges: (i, to) with 0 ≤ to < i for every edge.
        let g = growing_random_game(100, 3, true, true, 0xBEEF).unwrap();
        for (src, dst) in collect_edges(&g) {
            assert!(
                dst < src,
                "citation edge ({src} -> {dst}) violates temporal ordering"
            );
        }
    }

    #[test]
    fn citation_mode_directed_no_self_loops() {
        // dst < src ⇒ src ≠ dst.
        let g = growing_random_game(120, 4, true, true, 0xFACE).unwrap();
        for (src, dst) in collect_edges(&g) {
            assert_ne!(src, dst, "citation mode must not produce self-loops");
        }
    }

    #[test]
    fn citation_mode_first_vertex_never_source() {
        // Vertex 0 is the seed; its row in citation mode is always empty.
        let g = growing_random_game(75, 2, true, true, 0x42).unwrap();
        for (src, _dst) in collect_edges(&g) {
            assert_ne!(src, 0, "seed vertex must never be the source in citation");
        }
    }

    #[test]
    fn citation_mode_per_step_outdegree_equals_m() {
        // Citation mode emits exactly m edges with src = i on step i.
        let n: u32 = 50;
        let m: u32 = 3;
        let g = growing_random_game(n, m, true, true, 0xC1A0).unwrap();
        let mut per_src = vec![0u32; n as usize];
        for (src, _dst) in collect_edges(&g) {
            per_src[src as usize] += 1;
        }
        // Vertex 0 = 0 (seed). Vertices 1..n = exactly m each.
        assert_eq!(per_src[0], 0);
        for (i, &out) in per_src.iter().enumerate().skip(1) {
            assert_eq!(out, m, "vertex {i} out-degree should be m={m}");
        }
    }

    #[test]
    fn free_mode_directed_endpoints_within_frontier() {
        // (from, to) generated at step i must satisfy from ∈ [0, i] and
        // to ∈ [1, i]. After construction we can't observe step i
        // directly, but we know that for every edge (f, t):
        //   - t ≥ 1 (sink never lands on vertex 0).
        //   - max(f, t) was the current frontier upper bound at insert
        //     time, so max(f, t) > 0.
        let g = growing_random_game(100, 3, true, false, 0xBABE).unwrap();
        for (src, dst) in collect_edges(&g) {
            assert!(
                dst >= 1,
                "free-mode sink should never be 0 (got edge ({src}, {dst}))"
            );
        }
    }

    #[test]
    fn validate_huge_overflow_is_rejected_or_handled() {
        // Worst case: u32::MAX × u32::MAX would overflow u64 (≈1.8e19 ≈ u64::MAX),
        // so the multiplication is tight. We just assert that very large
        // inputs do not panic — they either error or return.
        let huge = u32::MAX;
        let res = growing_random_game(huge, huge, true, true, 0);
        let _ = res;
    }
}
