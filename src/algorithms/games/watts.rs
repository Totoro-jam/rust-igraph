//! Watts–Strogatz small-world model (ALGO-GN-009).
//!
//! Counterpart of `igraph_watts_strogatz_game()` in
//! `references/igraph/src/games/watts_strogatz.c:75-118` for the
//! `dim = 1` case (1-D periodic ring) — by far the dominant use case
//! and the original Watts & Strogatz Nature '98 model. The upstream
//! C entry point delegates to `igraph_square_lattice` plus
//! `igraph_rewire_edges`; both helpers are folded directly into this
//! module rather than ported as standalone APIs.
//!
//! ## Model
//!
//! Step 1: build a periodic 1-D ring on `size` vertices where every
//! vertex is connected to the `nei` nearest neighbours on each side.
//! Each vertex therefore has degree `2·nei` exactly (the validation
//! step rejects `2·nei + 1 > size`, so the ring is always "wide
//! enough" that the two sides do not overlap).
//!
//! Step 2: walk the edge list and, independently for each endpoint of
//! each edge, rewire with probability `p`. The new endpoint is sampled
//! uniformly from the vertex set with the optional rejection rules:
//!   * `loops = false` — a candidate equal to the other endpoint of
//!     the same edge is rejected, mirroring upstream's "draw from
//!     `[0, n-2]` and remap to skip the kept endpoint" trick;
//!   * `multiple = false` — a candidate that would create a duplicate
//!     of an existing edge is rejected. The C reference uses a custom
//!     linked-list adjacency for O(1) duplicate detection; we use a
//!     `HashSet<(u32, u32)>` of canonical pairs for the same effect.
//!
//! ## Validation
//!
//! * `size ≥ 1`.
//! * `nei ≥ 0`.
//! * `2·nei + 1 ≤ size` (so the ring lattice has no overlap; for
//!   `nei = 0` this collapses to `size ≥ 1` which is already required).
//! * `p ∈ [0, 1]`, NaN / infinity rejected.
//!
//! ## Determinism
//!
//! Fully deterministic in `(size, nei, p, loops, multiple, seed)` via
//! `SplitMix64`.
//!
//! ## Reference
//!
//! Duncan J. Watts and Steven H. Strogatz, *Collective dynamics of
//! "small-world" networks*, Nature **393**, 440-442 (1998).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate(size: u32, nei: u32, p: f64) -> IgraphResult<()> {
    if size == 0 {
        return Err(IgraphError::InvalidArgument(
            "watts_strogatz_game: size must be at least 1".into(),
        ));
    }
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(IgraphError::InvalidArgument(
            "watts_strogatz_game: p must be a finite real in [0, 1]".into(),
        ));
    }
    // 2·nei + 1 ≤ size ⇔ 2·nei ≤ size − 1. We compute on u64 to dodge
    // 32-bit overflow even when nei is u32::MAX.
    let two_nei = u64::from(nei).saturating_mul(2);
    if two_nei + 1 > u64::from(size) {
        return Err(IgraphError::InvalidArgument(format!(
            "watts_strogatz_game: 2*nei + 1 = {} must not exceed size = {}",
            two_nei + 1,
            size,
        )));
    }
    Ok(())
}

/// Build the 1-D periodic ring with `nei`-radius neighbourhood, in
/// edge-list form. Each vertex `v` emits `nei` forward edges to
/// `(v + j) mod size` for `j = 1..=nei`. Total edges = `size · nei`.
fn build_ring_lattice_edges(size: u32, nei: u32) -> Vec<(u32, u32)> {
    let n = size as usize;
    let k = nei as usize;
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(n * k);
    for v in 0..size {
        for j in 1..=nei {
            let u = (v + j) % size;
            edges.push((v, u));
        }
    }
    edges
}

/// Canonical undirected pair: smaller vertex first.
#[inline]
fn canonical(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Draw a uniform random vertex in `[0, size)` that is not equal to
/// `forbidden` (when `loops == false`). The reduction trick — draw
/// from `[0, size-1)` and remap the forbidden value to `size-1` —
/// matches `references/igraph/src/operators/rewire_edges.c:111-116`
/// and yields a uniform distribution over `[0, size) \ {forbidden}`
/// in O(1).
#[inline]
fn draw_vertex(size: u32, forbidden: u32, loops: bool, rng: &mut SplitMix64) -> u32 {
    if loops {
        rng.gen_index(size as usize) as u32
    } else {
        let r = rng.gen_index((size - 1) as usize) as u32;
        if r == forbidden { size - 1 } else { r }
    }
}

/// Rewire a single endpoint of edge `eid`. `which == 0` rewires the
/// "from" endpoint; `which == 1` rewires the "to" endpoint.
///
/// `adj`, when present, holds canonical undirected pairs already
/// represented in the edge list. With `adj == None` the multi-edge
/// constraint is skipped (the `multiple == true` fast path).
fn rewire_one_endpoint(
    eid: usize,
    which: u8,
    edges: &mut [(u32, u32)],
    adj: Option<&mut HashSet<(u32, u32)>>,
    size: u32,
    loops: bool,
    rng: &mut SplitMix64,
) {
    let (a, b) = edges[eid];
    let (old_v, kept) = if which == 0 { (a, b) } else { (b, a) };

    if let Some(set) = adj.as_ref() {
        debug_assert!(set.contains(&canonical(a, b)));
    }

    if let Some(set) = adj {
        // No-multi-edge path. Remove the current edge from the
        // canonical set before drawing so a candidate equal to old_v
        // (no-op move) is also legal.
        set.remove(&canonical(a, b));

        // Bounded rejection: at most `size` candidates exist, and at
        // least one — old_v itself — is always feasible (kept != old_v
        // unless it's a self-loop and self-loops are allowed, in
        // which case (kept, kept) was removed above). So the loop
        // terminates in expected O(1 / (1 − density)).
        let mut tries = 0u32;
        let cap = size.saturating_mul(4).max(64);
        let pick = loop {
            let cand = draw_vertex(size, kept, loops, rng);
            let pair = canonical(cand, kept);
            if !set.contains(&pair) {
                break cand;
            }
            tries += 1;
            if tries >= cap {
                // Density too high to find a fresh vertex within the
                // budget — keep the original endpoint. Same fallback
                // shape as the C `while (… && pot != v)` loop, which
                // also yields `pot == v` (no rewrite) on saturation.
                break old_v;
            }
        };
        let new_pair = canonical(pick, kept);
        set.insert(new_pair);
        if which == 0 {
            edges[eid] = (pick, kept);
        } else {
            edges[eid] = (kept, pick);
        }
    } else {
        // Multi-edges allowed: single uniform draw, no rejection.
        let pick = draw_vertex(size, kept, loops, rng);
        if which == 0 {
            edges[eid] = (pick, kept);
        } else {
            edges[eid] = (kept, pick);
        }
    }
}

/// Generate a 1-D Watts–Strogatz small-world graph.
///
/// Builds a periodic ring lattice of `size` vertices where every
/// vertex connects to its `nei` nearest neighbours on each side
/// (total degree `2·nei`), then independently rewires each endpoint
/// of each edge with probability `p`. The candidate replacement is
/// uniformly sampled from the vertex set, optionally subject to a
/// no-self-loop / no-multi-edge rejection rule.
///
/// `loops = false` rejects any candidate equal to the kept endpoint,
/// `multiple = false` additionally rejects candidates that would
/// duplicate an existing edge. Both default to `false` in the canonical
/// Watts–Strogatz Nature '98 model and in python-igraph's
/// `Graph.Watts_Strogatz`. The output is always undirected.
///
/// # Errors
///
/// Returns `IgraphError::InvalidArgument` if:
/// * `size == 0`, or
/// * `2·nei + 1 > size` (the ring would self-overlap), or
/// * `p` is NaN, infinite, or outside `[0, 1]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::watts_strogatz_game;
/// let g = watts_strogatz_game(10, 2, 0.0, false, false, 42).unwrap();
/// assert_eq!(g.vcount(), 10);
/// // p = 0 leaves the lattice untouched: 10 vertices × 2 neighbours
/// // per side / 2 (undirected) × 2 sides = 20 edges.
/// assert_eq!(g.ecount(), 20);
/// ```
pub fn watts_strogatz_game(
    size: u32,
    nei: u32,
    p: f64,
    loops: bool,
    multiple: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(size, nei, p)?;

    let mut edges = build_ring_lattice_edges(size, nei);
    let m = edges.len();

    // p == 0 short-circuit: no rewire, ring stays intact.
    if p > 0.0 && m > 0 {
        let mut rng = SplitMix64::new(seed);

        // Build adjacency set for the no-multi-edge path.
        let mut adj_set: Option<HashSet<(u32, u32)>> = if multiple {
            None
        } else {
            let mut s = HashSet::with_capacity(m);
            for &(a, b) in &edges {
                s.insert(canonical(a, b));
            }
            Some(s)
        };

        // Pass 1: rewire the "from" endpoint of each edge with
        // probability p, using geometric skips per upstream's
        // `references/igraph/src/operators/rewire_edges.c:99-129`.
        let mut to_rewire = rng.gen_geom(p);
        while to_rewire.is_finite() && (to_rewire as usize) < m {
            let eid = to_rewire as usize;
            rewire_one_endpoint(eid, 0, &mut edges, adj_set.as_mut(), size, loops, &mut rng);
            to_rewire += rng.gen_geom(p) + 1.0;
        }

        // Pass 2: rewire the "to" endpoint of each edge.
        let mut to_rewire = rng.gen_geom(p);
        while to_rewire.is_finite() && (to_rewire as usize) < m {
            let eid = to_rewire as usize;
            rewire_one_endpoint(eid, 1, &mut edges, adj_set.as_mut(), size, loops, &mut rng);
            to_rewire += rng.gen_geom(p) + 1.0;
        }
    }

    let mut g = Graph::with_vertices(size);
    let edges_iter = edges
        .into_iter()
        .map(|(a, b)| (a as VertexId, b as VertexId));
    g.add_edges(edges_iter)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn count_unique_edges(g: &Graph) -> usize {
        let mut canonical: HashSet<(u32, u32)> = HashSet::with_capacity(g.ecount());
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (a, b) = g.edge(eid).expect("edge id in bounds");
            canonical.insert(if a <= b { (a, b) } else { (b, a) });
        }
        canonical.len()
    }

    fn degrees(g: &Graph) -> Vec<u64> {
        let mut d = vec![0u64; g.vcount() as usize];
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (a, b) = g.edge(eid).expect("edge id in bounds");
            d[a as usize] += 1;
            d[b as usize] += 1;
        }
        d
    }

    #[test]
    fn ring_lattice_p0_undirected_undirected_count() {
        // p = 0 ⇒ unchanged ring. size = 8, nei = 2 → 8·2 = 16 edges,
        // every vertex has degree 4.
        let g = watts_strogatz_game(8, 2, 0.0, false, false, 1).unwrap();
        assert_eq!(g.vcount(), 8);
        assert_eq!(g.ecount(), 16);
        assert!(!g.is_directed());
        let d = degrees(&g);
        for (v, deg) in d.iter().enumerate() {
            assert_eq!(*deg, 4, "vertex {v} has degree {deg}, expected 4");
        }
    }

    #[test]
    fn ring_lattice_nei_zero_is_edgeless() {
        let g = watts_strogatz_game(10, 0, 0.5, false, false, 7).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn p_one_rewires_every_endpoint_undirected() {
        // p = 1 means *every* endpoint is rewired. Edge count is
        // preserved (rewire never adds or drops edges, only mutates
        // endpoints). With multiple=false, the result is still simple.
        let g = watts_strogatz_game(20, 3, 1.0, false, false, 0x_C0FFEE).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 20 * 3); // 60 edges = size·nei
        let n_unique = count_unique_edges(&g);
        assert_eq!(
            n_unique,
            g.ecount(),
            "rewire should not introduce multi-edges"
        );
        for eid in 0..u32::try_from(g.ecount()).unwrap() {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "no self-loops when loops=false (edge {eid})");
        }
    }

    #[test]
    fn rewire_preserves_edge_count() {
        for &p in &[0.0, 0.1, 0.3, 0.5, 0.9, 1.0] {
            let g = watts_strogatz_game(50, 3, p, false, false, 0xABCD).unwrap();
            assert_eq!(
                g.ecount(),
                50 * 3,
                "edge count should equal size·nei at p={p}"
            );
        }
    }

    #[test]
    fn simple_output_has_no_loops_or_multi() {
        let g = watts_strogatz_game(30, 4, 0.5, false, false, 0xDEAD).unwrap();
        let n_unique = count_unique_edges(&g);
        assert_eq!(n_unique, g.ecount(), "no multi-edges");
        for eid in 0..u32::try_from(g.ecount()).unwrap() {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "no self-loops");
        }
    }

    #[test]
    fn multigraph_path_runs_without_panic() {
        // multiple = true → no rejection; loops = true → self-loops OK.
        let g = watts_strogatz_game(20, 2, 0.7, true, true, 0xBEEF).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 20 * 2);
    }

    #[test]
    fn loops_true_multiple_false_may_self_loop() {
        // Carve a regime where the candidate-equal-to-kept-endpoint
        // case is reachable. We don't assert that a self-loop *exists*
        // (it depends on RNG) but the run must succeed.
        let g = watts_strogatz_game(10, 1, 1.0, true, false, 0x1357).unwrap();
        assert_eq!(g.ecount(), 10);
        // is_simple may be false here, but every edge must be unique
        // because multiple=false still rejects duplicates.
        let n_unique = count_unique_edges(&g);
        assert_eq!(n_unique, g.ecount());
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let a = watts_strogatz_game(30, 3, 0.2, false, false, 123_456).unwrap();
        let b = watts_strogatz_game(30, 3, 0.2, false, false, 123_456).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        let ea: Vec<(u32, u32)> = (0..u32::try_from(a.ecount()).unwrap())
            .map(|eid| {
                let (u, v) = a.edge(eid).unwrap();
                if u <= v { (u, v) } else { (v, u) }
            })
            .collect();
        let eb: Vec<(u32, u32)> = (0..u32::try_from(b.ecount()).unwrap())
            .map(|eid| {
                let (u, v) = b.edge(eid).unwrap();
                if u <= v { (u, v) } else { (v, u) }
            })
            .collect();
        let mut sa = ea.clone();
        sa.sort_unstable();
        let mut sb = eb.clone();
        sb.sort_unstable();
        assert_eq!(sa, sb);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = watts_strogatz_game(30, 3, 0.5, false, false, 1).unwrap();
        let b = watts_strogatz_game(30, 3, 0.5, false, false, 2).unwrap();
        let ea: HashSet<(u32, u32)> = (0..u32::try_from(a.ecount()).unwrap())
            .map(|eid| {
                let (u, v) = a.edge(eid).unwrap();
                if u <= v { (u, v) } else { (v, u) }
            })
            .collect();
        let eb: HashSet<(u32, u32)> = (0..u32::try_from(b.ecount()).unwrap())
            .map(|eid| {
                let (u, v) = b.edge(eid).unwrap();
                if u <= v { (u, v) } else { (v, u) }
            })
            .collect();
        assert_ne!(
            ea, eb,
            "different seeds should yield different rewired graphs"
        );
    }

    #[test]
    fn size_zero_rejected() {
        let err = watts_strogatz_game(0, 0, 0.0, false, false, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("size")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn nei_too_large_rejected() {
        // 2·nei + 1 > size: nei = 3, size = 6 → 7 > 6.
        let err = watts_strogatz_game(6, 3, 0.0, false, false, 0).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("nei")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn p_out_of_range_rejected() {
        for &p in &[-0.1_f64, 1.1, f64::NAN, f64::INFINITY] {
            let err = watts_strogatz_game(10, 2, p, false, false, 0).unwrap_err();
            matches!(err, IgraphError::InvalidArgument(_));
        }
    }

    #[test]
    fn ring_lattice_2_nei_equals_size_minus_one_is_almost_complete() {
        // size = 5, nei = 2 → 2·nei + 1 = 5 = size: the ring lattice
        // becomes K_5 (every vertex connects to the other 4).
        let g = watts_strogatz_game(5, 2, 0.0, false, false, 0).unwrap();
        assert_eq!(g.ecount(), 10); // C(5, 2) = 10
        let d = degrees(&g);
        for deg in d {
            assert_eq!(deg, 4);
        }
    }

    #[test]
    fn p_zero_seed_independence() {
        let a = watts_strogatz_game(20, 2, 0.0, false, false, 1).unwrap();
        let b = watts_strogatz_game(20, 2, 0.0, false, false, 999).unwrap();
        // p=0 means the seed should not be touched at all → identical
        // graphs regardless of seed.
        assert_eq!(a.ecount(), b.ecount());
        for eid in 0..u32::try_from(a.ecount()).unwrap() {
            assert_eq!(a.edge(eid).unwrap(), b.edge(eid).unwrap());
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    fn canonical_set(g: &Graph) -> HashSet<(u32, u32)> {
        let mut s = HashSet::with_capacity(g.ecount());
        let m = u32::try_from(g.ecount()).expect("ecount fits in u32");
        for eid in 0..m {
            let (a, b) = g.edge(eid).expect("edge id in bounds");
            s.insert(if a <= b { (a, b) } else { (b, a) });
        }
        s
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64, .. ProptestConfig::default()
        })]

        #[test]
        fn invariant_simple_graph_no_loops_no_multi(
            size in 4u32..30,
            nei_raw in 1u32..5,
            p in 0.0_f64..=1.0,
            seed in any::<u64>(),
        ) {
            // Ensure 2·nei + 1 ≤ size.
            let nei = std::cmp::min(nei_raw, (size - 1) / 2);
            prop_assume!(nei >= 1);

            let g = watts_strogatz_game(size, nei, p, false, false, seed).unwrap();
            prop_assert_eq!(g.vcount(), size);
            prop_assert_eq!(g.ecount(), (size * nei) as usize);

            // No self-loops, no multi-edges.
            let canonical = canonical_set(&g);
            prop_assert_eq!(canonical.len(), g.ecount());
            for eid in 0..u32::try_from(g.ecount()).unwrap() {
                let (a, b) = g.edge(eid).unwrap();
                prop_assert_ne!(a, b);
            }
        }

        #[test]
        fn invariant_edge_count_preserved_under_any_p(
            size in 4u32..30,
            nei_raw in 1u32..5,
            p in 0.0_f64..=1.0,
            loops in any::<bool>(),
            multiple in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let nei = std::cmp::min(nei_raw, (size - 1) / 2);
            prop_assume!(nei >= 1);
            let g = watts_strogatz_game(size, nei, p, loops, multiple, seed).unwrap();
            prop_assert_eq!(g.ecount(), (size * nei) as usize);
            prop_assert_eq!(g.vcount(), size);
        }

        #[test]
        fn invariant_determinism(
            size in 4u32..30,
            nei_raw in 1u32..5,
            p in 0.0_f64..=1.0,
            loops in any::<bool>(),
            multiple in any::<bool>(),
            seed in any::<u64>(),
        ) {
            let nei = std::cmp::min(nei_raw, (size - 1) / 2);
            prop_assume!(nei >= 1);
            let a = watts_strogatz_game(size, nei, p, loops, multiple, seed).unwrap();
            let b = watts_strogatz_game(size, nei, p, loops, multiple, seed).unwrap();
            prop_assert_eq!(a.vcount(), b.vcount());
            prop_assert_eq!(a.ecount(), b.ecount());

            let mut ea: Vec<(u32, u32)> = (0..u32::try_from(a.ecount()).unwrap())
                .map(|eid| {
                    let (u, v) = a.edge(eid).unwrap();
                    if u <= v { (u, v) } else { (v, u) }
                })
                .collect();
            let mut eb: Vec<(u32, u32)> = (0..u32::try_from(b.ecount()).unwrap())
                .map(|eid| {
                    let (u, v) = b.edge(eid).unwrap();
                    if u <= v { (u, v) } else { (v, u) }
                })
                .collect();
            ea.sort_unstable();
            eb.sort_unstable();
            prop_assert_eq!(ea, eb);
        }
    }
}
