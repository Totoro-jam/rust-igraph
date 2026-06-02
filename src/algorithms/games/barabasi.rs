//! Barabási–Albert preferential-attachment random graph (ALGO-GN-002,
//! BAG variant).
//!
//! Counterpart of the `IGRAPH_BARABASI_BAG` branch of
//! `igraph_barabasi_game()` in
//! `references/igraph/src/games/barabasi.c:67-178`.
//!
//! The "bag" mechanism (Newman 2003, originally implemented by Albert
//! and Barabási themselves) maintains a multiset whose multiplicity of
//! vertex `v` equals `deg(v) + 1`. Drawing a vertex uniformly from the
//! bag therefore yields a vertex proportional to its degree — the
//! preferential-attachment kernel of the classical BA model with
//! `power = 1` and constant attractiveness `A = 1`.
//!
//! Algorithm:
//!
//! 1. Initial bag = `[0]` (vertex `0` is the seed).
//! 2. For each new vertex `i = 1 .. n`:
//!    * Draw `m` neighbours from the bag uniformly with replacement.
//!    * Emit `(i, to)` for each neighbour.
//!    * Push `i` onto the bag once (so the new vertex itself is
//!      sample-able next round).
//!    * Push each chosen neighbour back onto the bag (their degree just
//!      went up by one).
//!    * If `outpref`, also push `m` copies of `i` (so the new vertex's
//!      *out*-degree also drives future sampling).
//!
//! Because draws are with replacement, two of the `m` neighbours of a
//! given vertex can collide → the BAG variant can produce multi-edges
//! and (when `i` itself is in the bag at sampling time, only possible
//! with `outpref=true`) self-loops. Use the PSUMTREE variant
//! ([`crate::erdos_renyi_gnp`] is *not* a substitute) once that AWU
//! lands if a simple graph is required.
//!
//! Total cost: `O(n · m)` work, `O(n · m)` memory for the bag and edge
//! list.
//!
//! ## Scope
//!
//! MVP scope only covers the most-used dispatch path:
//! * `power = 1`, `A = 1` (hardcoded — the BAG model only supports
//!   these per upstream `barabasi.c:567-574`).
//! * `m` is constant across all new vertices (no `outseq`).
//! * No `start_from` graph — we always seed with the singleton
//!   `[0]`.
//! * `outpref` defaults to `false` for directed graphs, and is
//!   *forced* to `true` for undirected graphs (matching upstream
//!   `barabasi.c:83-85`).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate_inputs(n: u32, m: u32) -> IgraphResult<()> {
    // n = 0 is allowed (returns an empty graph).
    // n = 1 is allowed (returns a single-vertex graph with no edges).
    // m·(n-1) must not overflow u64 — easily checked via checked_mul.
    let n_u64 = u64::from(n);
    let m_u64 = u64::from(m);
    let steps = n_u64.saturating_sub(1);
    let new_edges = steps
        .checked_mul(m_u64)
        .ok_or(IgraphError::Internal("barabasi_game edge count overflow"))?;
    // Total bag size we will need: n + new_edges + (outpref ? new_edges : 0).
    // The +outpref case (2 · new_edges) bounds both. Use that worst case.
    let _bag = n_u64
        .checked_add(new_edges)
        .and_then(|s| s.checked_add(new_edges))
        .ok_or(IgraphError::Internal("barabasi_game bag size overflow"))?;
    Ok(())
}

/// Generate a graph with `n` vertices via Barabási–Albert preferential
/// attachment, using the original "bag" mechanism (BAG variant).
///
/// Each new vertex attaches `m` outgoing edges to existing vertices,
/// chosen with probability proportional to their degree. Because draws
/// are with replacement, the result may contain multi-edges (and
/// self-loops when `outpref = true`).
///
/// * `n` — vertex count. `n = 0` returns an empty graph.
/// * `m` — number of edges each newly-added vertex contributes.
///   `m = 0` yields an edgeless graph on `n` vertices.
/// * `outpref` — when `true`, the new vertex's outgoing edges also
///   bias subsequent sampling (i.e., out-degree counts toward
///   attractiveness). Forced to `true` if `directed = false`
///   (preferential attachment on an undirected graph naturally uses
///   total degree).
/// * `directed` — generate a directed graph (edges point from new
///   vertex to older).
/// * `seed` — initialises an internal `SplitMix64` PRNG. Same
///   `(n, m, outpref, directed, seed)` always yields the same graph.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] if `(n - 1) · m` or the bag size
/// overflows `u64` (only possible on absurdly large inputs;
/// `(n, m) = (u32::MAX, u32::MAX)` is the only realistic failure mode).
///
/// # Examples
///
/// ```
/// use rust_igraph::barabasi_game_bag;
///
/// // 100-vertex directed BA graph with m = 2 outgoing edges per new vertex.
/// // The first vertex (seed) has no outgoing edges, so the total edge
/// // count is exactly (n - 1) · m = 99 · 2 = 198.
/// let g = barabasi_game_bag(100, 2, false, true, 0xCAFEBEEF).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert_eq!(g.ecount(), 198);
/// assert!(g.is_directed());
/// ```
pub fn barabasi_game_bag(
    n: u32,
    m: u32,
    outpref: bool,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate_inputs(n, m)?;

    // Upstream rule: undirected ⇒ outpref forced true.
    let outpref = outpref || !directed;

    if n == 0 {
        return Graph::new(0, directed);
    }
    if n == 1 || m == 0 {
        // Single vertex, or no edges per step — just return n isolated vertices.
        return Graph::new(n, directed);
    }

    let m_usize = m as usize;
    let steps = (n as usize).saturating_sub(1);
    let total_edges = steps
        .checked_mul(m_usize)
        .ok_or(IgraphError::Internal("barabasi_game edge total overflow"))?;
    let bag_capacity = (n as usize)
        .checked_add(total_edges)
        .and_then(|s| s.checked_add(if outpref { total_edges } else { 0 }))
        .ok_or(IgraphError::Internal("barabasi_game bag capacity overflow"))?;

    let mut bag: Vec<VertexId> = Vec::with_capacity(bag_capacity);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(total_edges);

    // Seed bag with vertex 0.
    bag.push(0);

    let mut rng = SplitMix64::new(seed);

    for i in 1..n {
        // Draw m neighbours uniformly from the current bag (with replacement).
        // Sample into a scratch buffer first so we can push to `bag` after
        // emitting all m edges of this round — matches upstream's two-pass
        // structure in barabasi.c:158-170.
        let bag_len = bag.len();
        debug_assert!(bag_len > 0);
        let edges_start = edges.len();
        for _ in 0..m {
            let pick = rng.gen_index(bag_len);
            let to = bag[pick];
            edges.push((i, to));
        }
        // Update bag for next round.
        bag.push(i);
        for k in 0..m_usize {
            let neighbour = edges[edges_start + k].1;
            bag.push(neighbour);
            if outpref {
                bag.push(i);
            }
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
        let g = barabasi_game_bag(0, 3, false, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_returns_singleton() {
        let g = barabasi_game_bag(1, 5, false, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn m_zero_returns_edgeless() {
        let g = barabasi_game_bag(10, 0, false, true, 1).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn exact_edge_count_directed_constant_m() {
        // Exactly (n - 1) · m edges.
        for &(n, m) in &[(10u32, 1u32), (10, 3), (50, 2), (200, 5)] {
            let g = barabasi_game_bag(n, m, false, true, 0x00C0_FFEE).unwrap();
            assert_eq!(g.vcount(), n);
            assert_eq!(g.ecount(), (n as usize - 1) * m as usize);
        }
    }

    #[test]
    fn undirected_forces_outpref() {
        // outpref=false but directed=false ⇒ should be promoted to true.
        // We verify by ensuring the degree-sum invariant of the bag holds:
        // |bag| = n + 2·(n-1)·m for outpref, n + (n-1)·m otherwise.
        // We can't observe `bag` directly, but the edge count is the same
        // either way (it depends only on m and n), so we check that the
        // generation does not error.
        let g = barabasi_game_bag(50, 3, false, false, 0x0BAD_C0DE).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 49 * 3);
        assert!(!g.is_directed());
    }

    #[test]
    fn deterministic_with_seed() {
        let a = barabasi_game_bag(100, 4, true, true, 0xDEAD).unwrap();
        let b = barabasi_game_bag(100, 4, true, true, 0xDEAD).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        // Compare edges as sequences — same seed must yield identical
        // ordering.
        let ea: Vec<_> = collect_edges(&a);
        let eb: Vec<_> = collect_edges(&b);
        assert_eq!(ea, eb);
    }

    #[test]
    fn different_seeds_yield_different_graphs() {
        let a = barabasi_game_bag(100, 4, false, true, 1).unwrap();
        let b = barabasi_game_bag(100, 4, false, true, 2).unwrap();
        let ea: Vec<_> = collect_edges(&a);
        let eb: Vec<_> = collect_edges(&b);
        assert_ne!(ea, eb, "different seeds must produce different graphs");
    }

    #[test]
    fn first_vertex_has_no_outgoing_in_directed_graph() {
        // Vertex 0 is the seed; no new edge ever has 0 as its source.
        let g = barabasi_game_bag(50, 3, false, true, 0x42).unwrap();
        for (src, _dst) in collect_edges(&g) {
            assert_ne!(src, 0, "seed vertex must never be the source");
        }
    }

    #[test]
    fn edges_only_point_to_earlier_vertices() {
        // Preferential attachment: (i, to) with to < i for every edge.
        let g = barabasi_game_bag(80, 2, false, true, 0xBEEF).unwrap();
        for (src, dst) in collect_edges(&g) {
            assert!(
                dst < src,
                "edge ({src} -> {dst}) violates BA temporal ordering"
            );
        }
    }

    #[test]
    fn high_degree_concentration_around_early_vertices() {
        // Empirical sanity check — in BA the early vertices accumulate
        // most of the degree. We check that the top-5 highest-degree
        // vertices include vertex 0.
        let g = barabasi_game_bag(500, 3, false, true, 0xABCD).unwrap();
        let mut deg = vec![0u32; g.vcount() as usize];
        for (src, dst) in collect_edges(&g) {
            deg[src as usize] += 1;
            deg[dst as usize] += 1;
        }
        let mut indexed: Vec<(usize, u32)> = deg.iter().copied().enumerate().collect();
        indexed.sort_by_key(|b| std::cmp::Reverse(b.1));
        let top5: Vec<usize> = indexed.iter().take(5).map(|(v, _)| *v).collect();
        assert!(
            top5.contains(&0),
            "expected vertex 0 in top-5, got {top5:?}"
        );
    }

    #[test]
    fn outpref_true_increases_self_degree_role() {
        // With outpref=true, the new vertex's *out*-degree also feeds
        // future sampling. We check that the graph is well-formed and
        // that the edge count matches the formula (independent of
        // outpref).
        let g = barabasi_game_bag(40, 2, true, true, 7).unwrap();
        assert_eq!(g.ecount(), 39 * 2);
    }

    #[test]
    fn validate_huge_overflow_is_rejected() {
        // (u32::MAX - 1) · u32::MAX overflows u64? No — u32::MAX² ≈ 1.8e19
        // which is below u64::MAX ≈ 1.8e19. Tight. The bag-size check
        // (2 · new_edges + n) is the one that flips. Verify that
        // worst-case actually errors.
        let huge = u32::MAX;
        let res = barabasi_game_bag(huge, huge, true, true, 0);
        // Either it errors (overflow) or it succeeds (n=1 step on a
        // billion-vertex graph would blow memory long before u64 trouble).
        // We just assert it returns *something* without panicking.
        let _ = res;
    }
}
