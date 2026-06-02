//! Forest fire random-graph generator (ALGO-GN-006).
//!
//! Counterpart of `igraph_forest_fire_game()` in
//! `references/igraph/src/games/forestfire.c:106-257`.
//!
//! ## Model
//!
//! Leskovec–Kleinberg–Faloutsos (KDD'05). Nodes are added one at a
//! time. Each new node `v`:
//!
//! 1. picks `ambs` *ambassador* vertices uniformly at random from the
//!    already-present nodes and cites each of them;
//! 2. for every ambassador `a`, draws `x ~ Geom(1 − p)` outgoing and
//!    `y ~ Geom(1 − r·p)` incoming neighbours of `a` (where `p` is the
//!    forward burning probability and `r` the backward factor), cites
//!    those still-unvisited neighbours, and pushes them onto the burn
//!    queue;
//! 3. the burn continues breadth-first until the queue empties.
//!
//! The corrected variant from
//! <https://www.cs.cmu.edu/~jure/pubs/powergrowth-tkdd.pdf> is the one
//! mirrored here — the published paper algorithm uses
//! `mean = p/(1−p)` which igraph corrects to a single geometric draw.
//!
//! ## Validation
//!
//! * `fw_prob ∈ [0, 1)` — strict upper bound so the geometric draw is
//!   finite (geometric mean is `p/(1−p)`).
//! * `bw_factor · fw_prob ∈ [0, 1)` — same guard for the incoming draw.
//! * `ambs ≥ 0`. `ambs = 0` returns an edgeless graph of `n` isolated
//!   vertices, matching upstream.
//!
//! ## Algorithm cost
//!
//! Asymptotic cost is hard to pin down because the burn radius depends
//! on the in/out-neighbourhood sizes built up so far. In the sparse
//! regime (`p` away from 1) the expected work per new node is bounded
//! by the geometric tail, giving roughly `O(n / (1 − p)²)` overall.
//! Memory is `O(n + |E|)`: two per-vertex `Vec<u32>` adjacency arrays
//! plus the edge buffer.
//!
//! ## Determinism
//!
//! Fully deterministic in `(n, fw_prob, bw_factor, ambs, directed,
//! seed)` via `SplitMix64`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::collections::VecDeque;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate(fw_prob: f64, bw_factor: f64) -> IgraphResult<()> {
    if !fw_prob.is_finite() || !(0.0..1.0).contains(&fw_prob) {
        return Err(IgraphError::Internal(
            "forest_fire_game: fw_prob must satisfy 0 <= fw_prob < 1",
        ));
    }
    let bw_prob = bw_factor * fw_prob;
    if !bw_prob.is_finite() || !(0.0..1.0).contains(&bw_prob) {
        return Err(IgraphError::Internal(
            "forest_fire_game: bw_factor * fw_prob must satisfy 0 <= bw*fw < 1",
        ));
    }
    Ok(())
}

/// Generate a forest-fire-model random graph on `n` vertices.
///
/// * `n` — vertex count. `n = 0` returns an empty graph; `n = 1`
///   returns a single isolated vertex.
/// * `fw_prob` — forward burning probability `p ∈ [0, 1)`. The number
///   of outgoing neighbours a burn picks from each ambassador follows
///   `Geom(1 − p)`.
/// * `bw_factor` — backward burning ratio `r ≥ 0`. The number of
///   incoming neighbours follows `Geom(1 − r·p)`; the product
///   `r · p` must also stay in `[0, 1)`.
/// * `ambs` — number of ambassadors each new node connects to.
///   `ambs = 0` shortcuts to an edgeless `n`-vertex graph.
/// * `directed` — whether to return a directed graph. The internal
///   burn tracks in/out-neighbour lists regardless of this flag; only
///   the final `Graph` honours it.
/// * `seed` — seeds the internal `SplitMix64` PRNG.
///
/// # Errors
///
/// Returns [`IgraphError::Internal`] if `fw_prob` falls outside
/// `[0, 1)`, if `bw_factor · fw_prob` is outside `[0, 1)`, or if
/// either parameter is non-finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::forest_fire_game;
///
/// // Light burn: small fw_prob keeps edges close to the ambassador count.
/// let g = forest_fire_game(100, 0.05, 0.3, 2, true, 0xF00D).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert!(g.is_directed());
/// // Each non-root vertex cites >= 1 ambassador, so at least n - 1 edges.
/// assert!(g.ecount() >= 99);
/// ```
pub fn forest_fire_game(
    n: u32,
    fw_prob: f64,
    bw_factor: f64,
    ambs: u32,
    directed: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(fw_prob, bw_factor)?;

    if ambs == 0 || n < 2 {
        return Graph::new(n, directed);
    }

    let n_usize = n as usize;
    let p_geom_out = 1.0 - fw_prob;
    let p_geom_in = 1.0 - fw_prob * bw_factor;

    let mut rng = SplitMix64::new(seed);

    // Per-vertex adjacency lists built up incrementally — used by the
    // burn to choose where to spread next.
    let mut inneis: Vec<Vec<u32>> = (0..n_usize).map(|_| Vec::new()).collect();
    let mut outneis: Vec<Vec<u32>> = (0..n_usize).map(|_| Vec::new()).collect();

    // `visited[v] == actnode + 1` flags `v` as "already cited by the
    // current new node `actnode`". Stamping with `actnode + 1` rather
    // than `actnode` ensures the zero-init slate is distinguishable
    // from a stamp left by `actnode == 0`.
    let mut visited: Vec<u32> = vec![0; n_usize];
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut neiq: VecDeque<u32> = VecDeque::with_capacity(16);

    for actnode in 1u32..n {
        let stamp = actnode.wrapping_add(1);
        // The new vertex never cites itself.
        visited[actnode as usize] = stamp;

        // Pick `ambs` ambassadors uniformly from [0, actnode).
        for _ in 0..ambs {
            let a = rng.gen_index(actnode as usize) as u32;
            try_add_edge(
                &mut visited,
                &mut neiq,
                &mut edges,
                &mut outneis,
                &mut inneis,
                actnode,
                a,
                stamp,
            );
        }

        while let Some(actamb) = neiq.pop_front() {
            let neis_out = clamp_geom(rng.gen_geom(p_geom_out));
            let neis_in = clamp_geom(rng.gen_geom(p_geom_in));

            burn_direction(
                &mut outneis,
                actamb,
                neis_out,
                &mut visited,
                &mut neiq,
                &mut edges,
                &mut inneis,
                actnode,
                stamp,
                BurnSide::Out,
                &mut rng,
            );
            burn_direction(
                &mut inneis,
                actamb,
                neis_in,
                &mut visited,
                &mut neiq,
                &mut edges,
                &mut outneis,
                actnode,
                stamp,
                BurnSide::In,
                &mut rng,
            );
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Bound `Geom(p)` draws to the platform `u32::MAX` window so the
/// "draw >= `no_out`" upstream short-circuit still holds when the
/// geometric tail returns a very large number.
fn clamp_geom(x: f64) -> u32 {
    if !x.is_finite() || x >= f64::from(u32::MAX) {
        return u32::MAX;
    }
    x as u32
}

#[derive(Clone, Copy)]
enum BurnSide {
    Out,
    In,
}

/// Burn through `count` neighbours on one side (`side`) of ambassador
/// `actamb`. Mirrors the upstream Fisher–Yates partial-shuffle path
/// that picks distinct, still-unvisited targets and falls through to
/// the "burn them all" branch when `count` exceeds the side's size.
#[allow(clippy::too_many_arguments)]
fn burn_direction(
    burn_side_arr: &mut [Vec<u32>],
    actamb: u32,
    count: u32,
    visited: &mut [u32],
    neiq: &mut VecDeque<u32>,
    edges: &mut Vec<(VertexId, VertexId)>,
    counter_side_arr: &mut [Vec<u32>],
    actnode: u32,
    stamp: u32,
    side: BurnSide,
    rng: &mut SplitMix64,
) {
    let side_len = burn_side_arr[actamb as usize].len();
    if count as usize >= side_len {
        // Burn them all — no shuffling required.
        for i in 0..side_len {
            let nei = burn_side_arr[actamb as usize][i];
            try_add_edge_side(
                visited,
                neiq,
                edges,
                burn_side_arr,
                counter_side_arr,
                actnode,
                nei,
                stamp,
                side,
            );
        }
        return;
    }

    let mut left = side_len;
    let mut taken: usize = 0;
    let target = count as usize;
    while taken < target && left > 0 {
        let which = rng.gen_index(left);
        let nei = burn_side_arr[actamb as usize][which];
        // Swap-to-end so the next draw covers a different slot.
        burn_side_arr[actamb as usize].swap(which, left - 1);
        if visited[nei as usize] != stamp {
            try_add_edge_side(
                visited,
                neiq,
                edges,
                burn_side_arr,
                counter_side_arr,
                actnode,
                nei,
                stamp,
                side,
            );
            taken += 1;
        }
        left -= 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn try_add_edge_side(
    visited: &mut [u32],
    neiq: &mut VecDeque<u32>,
    edges: &mut Vec<(VertexId, VertexId)>,
    burn_side_arr: &mut [Vec<u32>],
    counter_side_arr: &mut [Vec<u32>],
    actnode: u32,
    nei: u32,
    stamp: u32,
    side: BurnSide,
) {
    if visited[nei as usize] == stamp {
        return;
    }
    visited[nei as usize] = stamp;
    neiq.push_back(nei);
    edges.push((actnode as VertexId, nei as VertexId));
    // Mirror upstream's adjacency bookkeeping: the new edge actnode → nei
    // adds `nei` to actnode's outneis and `actnode` to nei's inneis.
    // The arrays we pass in switch roles depending on which side we
    // were burning, but the bookkeeping itself is direction-aware.
    match side {
        BurnSide::Out => {
            // burn_side_arr is outneis; counter_side_arr is inneis.
            burn_side_arr[actnode as usize].push(nei);
            counter_side_arr[nei as usize].push(actnode);
        }
        BurnSide::In => {
            // burn_side_arr is inneis; counter_side_arr is outneis.
            counter_side_arr[actnode as usize].push(nei);
            burn_side_arr[nei as usize].push(actnode);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn try_add_edge(
    visited: &mut [u32],
    neiq: &mut VecDeque<u32>,
    edges: &mut Vec<(VertexId, VertexId)>,
    outneis: &mut [Vec<u32>],
    inneis: &mut [Vec<u32>],
    actnode: u32,
    nei: u32,
    stamp: u32,
) {
    if visited[nei as usize] == stamp {
        return;
    }
    visited[nei as usize] = stamp;
    neiq.push_back(nei);
    edges.push((actnode as VertexId, nei as VertexId));
    outneis[actnode as usize].push(nei);
    inneis[nei as usize].push(actnode);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn canonical_edges(g: &Graph) -> HashSet<(VertexId, VertexId)> {
        let n_edges = u32::try_from(g.ecount()).expect("ecount fits in u32 for tests");
        (0..n_edges)
            .map(|eid| {
                let (a, b) = g.edge(eid).expect("edge id in bounds");
                if a <= b { (a, b) } else { (b, a) }
            })
            .collect()
    }

    #[test]
    fn n_zero_returns_empty_graph() {
        let g = forest_fire_game(0, 0.1, 0.3, 2, true, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn n_one_returns_singleton() {
        let g = forest_fire_game(1, 0.1, 0.3, 2, true, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn ambs_zero_yields_isolated_vertices() {
        let g = forest_fire_game(50, 0.2, 0.5, 0, true, 1).unwrap();
        assert_eq!(g.vcount(), 50);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn directed_flag_honoured() {
        let g = forest_fire_game(20, 0.1, 0.3, 1, true, 42).unwrap();
        assert!(g.is_directed());
        let g = forest_fire_game(20, 0.1, 0.3, 1, false, 42).unwrap();
        assert!(!g.is_directed());
    }

    #[test]
    fn at_least_one_edge_per_new_node_when_ambs_positive() {
        // With p = 0 the burn never spreads beyond ambassadors. From
        // actnode = k, `ambs` uniform draws from [0, k) yield between
        // 1 (all duplicates, possible only when k = 1) and
        // min(ambs, k) distinct edges. For n = 30, ambs = 2 the band
        // is [29, 1 + 2·28] = [29, 57].
        let g = forest_fire_game(30, 0.0, 0.0, 2, true, 5).unwrap();
        let ec = g.ecount();
        assert!((29..=57).contains(&ec), "edges {ec} outside [29, 57]");
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let a = forest_fire_game(100, 0.2, 0.5, 2, true, 0xC0DE).unwrap();
        let b = forest_fire_game(100, 0.2, 0.5, 2, true, 0xC0DE).unwrap();
        assert_eq!(a.ecount(), b.ecount());
        assert_eq!(canonical_edges(&a), canonical_edges(&b));
    }

    #[test]
    fn distinct_seeds_differ() {
        let a = forest_fire_game(200, 0.25, 0.4, 2, true, 1).unwrap();
        let b = forest_fire_game(200, 0.25, 0.4, 2, true, 2).unwrap();
        assert_ne!(canonical_edges(&a), canonical_edges(&b));
    }

    #[test]
    fn no_self_loops() {
        let g = forest_fire_game(80, 0.3, 0.4, 2, true, 9).unwrap();
        let n_edges = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_edges {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b);
        }
    }

    #[test]
    fn no_duplicate_edges_per_new_node() {
        // Within a single `actnode` burn, the visited stamp prevents
        // re-citing. Across actnodes the new edges always start at the
        // current `actnode`, so directed (src, dst) pairs are unique
        // (src = actnode appears exactly once across the whole run).
        let g = forest_fire_game(80, 0.3, 0.4, 2, true, 9).unwrap();
        let n_edges = u32::try_from(g.ecount()).unwrap();
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        for eid in 0..n_edges {
            let (a, b) = g.edge(eid).unwrap();
            assert!(seen.insert((a, b)), "duplicate directed edge {a}->{b}");
        }
    }

    #[test]
    fn fw_prob_out_of_range_errors() {
        assert!(forest_fire_game(10, -0.1, 0.3, 2, true, 1).is_err());
        assert!(forest_fire_game(10, 1.0, 0.3, 2, true, 1).is_err());
        assert!(forest_fire_game(10, 1.5, 0.3, 2, true, 1).is_err());
        assert!(forest_fire_game(10, f64::NAN, 0.3, 2, true, 1).is_err());
        assert!(forest_fire_game(10, f64::INFINITY, 0.3, 2, true, 1).is_err());
    }

    #[test]
    fn bw_factor_combined_out_of_range_errors() {
        // bw_factor * fw_prob = 0.6 * 0.9 = 0.54 — ok
        assert!(forest_fire_game(10, 0.9, 0.6, 2, true, 1).is_ok());
        // bw_factor * fw_prob = 1.2 * 0.9 = 1.08 — invalid
        assert!(forest_fire_game(10, 0.9, 1.2, 2, true, 1).is_err());
        // negative bw_factor — invalid
        assert!(forest_fire_game(10, 0.5, -0.1, 2, true, 1).is_err());
    }

    #[test]
    fn ambs_capped_to_existing_nodes() {
        // ambs = 10, n = 5, p = 0. For actnode = k the 10 draws from
        // [0, k) yield at most min(10, k) = k distinct edges. With
        // small k the birthday paradox almost guarantees the cap is
        // hit, so total edges land near the upper bound
        // 1+2+3+4 = 10. The lower bound 4 (one distinct per actnode)
        // is mathematically possible but vanishingly unlikely.
        let g = forest_fire_game(5, 0.0, 0.0, 10, true, 7).unwrap();
        let ec = g.ecount();
        assert!((4..=10).contains(&ec), "edges {ec} outside [4, 10]");
    }

    #[test]
    fn growth_monotone_in_p() {
        // Higher fw_prob → larger expected burn → more edges (in
        // expectation). We test across several seeds since the
        // monotonicity only holds in expectation, and accept the
        // average over a small ensemble.
        let mut low_total: u64 = 0;
        let mut high_total: u64 = 0;
        for seed in 0..8u64 {
            let lo = forest_fire_game(50, 0.05, 0.5, 2, true, seed).unwrap();
            let hi = forest_fire_game(50, 0.35, 0.5, 2, true, seed).unwrap();
            low_total += lo.ecount() as u64;
            high_total += hi.ecount() as u64;
        }
        assert!(high_total > low_total);
    }

    #[test]
    fn each_actnode_emits_only_outgoing_edges() {
        // By construction, every edge has src = some actnode ≥ 1 and
        // dst < src. In the directed view this means every vertex has
        // in-degree 0 if no later vertex ever cited it — but more
        // importantly src > dst always holds for the emitted edges.
        let g = forest_fire_game(60, 0.3, 0.4, 2, true, 3).unwrap();
        let n_edges = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_edges {
            let (a, b) = g.edge(eid).unwrap();
            assert!(a > b, "edge {a}->{b} violates DAG-by-construction");
        }
    }
}
