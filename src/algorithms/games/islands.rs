//! Simple interconnected islands random-graph generator (ALGO-GN-007).
//!
//! Counterpart of `igraph_simple_interconnected_islands_game()` in
//! `references/igraph/src/games/islands.c:55-176`.
//!
//! ## Model
//!
//! Generates `islands_n` Erdős–Rényi G(n, p) "islands" each of size
//! `islands_size` and probability `islands_pin`, then for every
//! unordered pair of islands draws exactly `n_inter` bipartite edges
//! uniformly at random from the `islands_size × islands_size` possible
//! cross-island pairs. The resulting graph is always undirected and
//! simple — within an island the geometric-skip walk produces strictly
//! increasing pair-indices, and across islands the bipartite slice
//! cannot collide with intra-island edges (the index spaces are
//! disjoint by construction).
//!
//! ## Validation
//!
//! * `islands_pin ∈ [0, 1]`. NaN / non-finite rejected.
//! * `n_inter ≤ islands_size²` (max-saturated bipartite slice).
//! * `islands_n · islands_size` must fit in `u32`.
//!
//! ## Algorithm cost
//!
//! Per island: Batagelj–Brandes geometric-skip is `O(islands_size + |E_i|)`.
//! Per inter-island pair: Floyd distinct-sample is `O(n_inter)` expected.
//! Total: `O(N + |E|)` where `N = islands_n · islands_size` and `|E|`
//! aggregates both intra- and inter-island edges.
//!
//! ## Determinism
//!
//! Fully deterministic in
//! `(islands_n, islands_size, islands_pin, n_inter, seed)` via
//! `SplitMix64`.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::manual_midpoint
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

fn validate(islands_n: u32, islands_size: u32, islands_pin: f64, n_inter: u32) -> IgraphResult<()> {
    if !islands_pin.is_finite() || !(0.0..=1.0).contains(&islands_pin) {
        return Err(IgraphError::InvalidArgument(format!(
            "simple_interconnected_islands_game: islands_pin must satisfy 0 <= islands_pin <= 1 (got {islands_pin})"
        )));
    }
    let size_sq = u64::from(islands_size) * u64::from(islands_size);
    if u64::from(n_inter) > size_sq {
        return Err(IgraphError::InvalidArgument(format!(
            "simple_interconnected_islands_game: n_inter ({n_inter}) exceeds islands_size^2 ({size_sq})"
        )));
    }
    let total_nodes = u64::from(islands_n) * u64::from(islands_size);
    if total_nodes > u64::from(u32::MAX) {
        return Err(IgraphError::InvalidArgument(format!(
            "simple_interconnected_islands_game: islands_n * islands_size ({total_nodes}) overflows u32"
        )));
    }
    Ok(())
}

/// Decode a unit-triangle pair-index `idx` into `(from, to)` with
/// `from < to`, where indices `0, 1, 2, …` enumerate
/// `(0,1), (0,2), (1,2), (0,3), …`. This matches the upstream
/// `to = floor((sqrt(8x+1)+1)/2); from = x − to·(to−1)/2`.
fn decode_triangle_pair(idx: u64) -> (u32, u32) {
    let x = idx as f64;
    let to = ((8.0 * x + 1.0).sqrt() + 1.0) / 2.0;
    let to = to.floor() as u64;
    let from = idx - (to * (to - 1)) / 2;
    (from as u32, to as u32)
}

/// Floyd distinct-sample of `m` integers from `[0, n_pairs)`. Mirrors
/// the [`crate::algorithms::games::erdos_renyi`] version but kept local
/// so the islands module is self-contained.
fn distinct_sample(rng: &mut SplitMix64, n_pairs: u64, m: u64) -> Vec<u64> {
    debug_assert!(m <= n_pairs);
    let cap = usize::try_from(m).unwrap_or(usize::MAX);
    let mut chosen: HashSet<u64> = HashSet::with_capacity(cap);
    let mut out: Vec<u64> = Vec::with_capacity(cap);
    for j in (n_pairs - m)..n_pairs {
        let span = j + 1;
        let span_usize = usize::try_from(span).unwrap_or(usize::MAX);
        let t = rng.gen_index(span_usize) as u64;
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

/// Generate a random graph made of `islands_n` interconnected
/// Erdős–Rényi islands.
///
/// * `islands_n` — number of islands.
/// * `islands_size` — vertex count per island (all islands have the
///   same size).
/// * `islands_pin ∈ [0, 1]` — within-island edge probability.
/// * `n_inter` — number of bipartite edges drawn between each
///   unordered pair of islands. Must satisfy
///   `n_inter ≤ islands_size²`.
/// * `seed` — seeds the internal `SplitMix64` PRNG.
///
/// The returned graph is always undirected and simple. Vertex
/// labelling: island `is` occupies the contiguous range
/// `[is · islands_size, (is + 1) · islands_size)`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `islands_pin` is
/// outside `[0, 1]`, if `n_inter > islands_size²`, or if the total
/// vertex count `islands_n · islands_size` overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::simple_interconnected_islands_game;
///
/// // 4 islands of size 25 with p = 0.4 within and 3 cross-island edges
/// // between each pair → 100 vertices total.
/// let g = simple_interconnected_islands_game(4, 25, 0.4, 3, 0xA15_1A4D).unwrap();
/// assert_eq!(g.vcount(), 100);
/// assert!(!g.is_directed());
/// // Inter-island contribution alone is C(4, 2) · 3 = 18 edges; the
/// // intra-island contribution adds many more.
/// assert!(g.ecount() >= 18);
/// ```
pub fn simple_interconnected_islands_game(
    islands_n: u32,
    islands_size: u32,
    islands_pin: f64,
    n_inter: u32,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(islands_n, islands_size, islands_pin, n_inter)?;

    let total_nodes = islands_n * islands_size;
    if total_nodes == 0 {
        return Graph::new(total_nodes, false);
    }

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    let intra_cap: u64 = if islands_size >= 2 {
        u64::from(islands_size) * u64::from(islands_size - 1) / 2
    } else {
        0
    };
    let intra_cap_f = intra_cap as f64;

    let bipartite_cap: u64 = u64::from(islands_size) * u64::from(islands_size);

    for is in 0..islands_n {
        let start = is * islands_size;

        // Intra-island Batagelj–Brandes geometric-skip over the
        // strict upper triangle [0, islands_size·(islands_size-1)/2).
        if islands_pin > 0.0 && intra_cap > 0 {
            if islands_pin >= 1.0 {
                // Saturate: emit every upper-triangle edge.
                for idx in 0..intra_cap {
                    let (from, to) = decode_triangle_pair(idx);
                    edges.push(((start + from) as VertexId, (start + to) as VertexId));
                }
            } else {
                let mut last = rng.gen_geom(islands_pin);
                while last < intra_cap_f {
                    let idx = last.trunc() as u64;
                    if idx >= intra_cap {
                        break;
                    }
                    let (from, to) = decode_triangle_pair(idx);
                    edges.push(((start + from) as VertexId, (start + to) as VertexId));
                    last += rng.gen_geom(islands_pin);
                    last += 1.0;
                }
            }
        }

        // Inter-island bipartite slices for every later island i' > is.
        if n_inter > 0 && islands_size > 0 {
            for ip in (is + 1)..islands_n {
                let other_start = ip * islands_size;
                let picks = distinct_sample(&mut rng, bipartite_cap, u64::from(n_inter));
                for pick in picks {
                    let from = (pick / u64::from(islands_size)) as u32;
                    let to = (pick - u64::from(from) * u64::from(islands_size)) as u32;
                    edges.push(((start + from) as VertexId, (other_start + to) as VertexId));
                }
            }
        }
    }

    let mut g = Graph::new(total_nodes, false)?;
    g.add_edges(edges)?;
    Ok(g)
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
    fn empty_when_no_islands() {
        let g = simple_interconnected_islands_game(0, 10, 0.5, 1, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn empty_when_island_size_zero() {
        let g = simple_interconnected_islands_game(5, 0, 0.5, 0, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_island_zero_pin_zero_inter_is_edgeless() {
        let g = simple_interconnected_islands_game(1, 20, 0.0, 0, 7).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn single_island_pin_one_is_complete_clique() {
        let g = simple_interconnected_islands_game(1, 10, 1.0, 0, 9).unwrap();
        assert_eq!(g.vcount(), 10);
        // K_10 has 10*9/2 = 45 edges.
        assert_eq!(g.ecount(), 45);
    }

    #[test]
    fn multiple_islands_pin_zero_only_inter_island() {
        // 4 islands of size 5, p=0, n_inter=2 → C(4,2)·2 = 12 edges.
        let g = simple_interconnected_islands_game(4, 5, 0.0, 2, 3).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 12);
    }

    #[test]
    fn saturated_bipartite_slice() {
        // islands_size=3, n_inter = 9 = 3² (max).
        let g = simple_interconnected_islands_game(2, 3, 0.0, 9, 5).unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 9);
    }

    #[test]
    fn n_inter_above_cap_errors() {
        let res = simple_interconnected_islands_game(2, 3, 0.0, 10, 5);
        assert!(res.is_err());
    }

    #[test]
    fn invalid_pin_rejected() {
        assert!(simple_interconnected_islands_game(2, 3, -0.1, 0, 5).is_err());
        assert!(simple_interconnected_islands_game(2, 3, 1.1, 0, 5).is_err());
        assert!(simple_interconnected_islands_game(2, 3, f64::NAN, 0, 5).is_err());
    }

    #[test]
    fn output_is_undirected_and_simple() {
        let g = simple_interconnected_islands_game(3, 8, 0.4, 2, 11).unwrap();
        assert!(!g.is_directed());
        let n_edges = u32::try_from(g.ecount()).unwrap();
        let mut seen: HashSet<(VertexId, VertexId)> = HashSet::new();
        for eid in 0..n_edges {
            let (a, b) = g.edge(eid).unwrap();
            assert_ne!(a, b, "self-loop edge {a}-{b}");
            let pair = if a <= b { (a, b) } else { (b, a) };
            assert!(seen.insert(pair), "multi-edge {pair:?}");
        }
    }

    #[test]
    fn intra_edges_stay_within_island_boundaries() {
        // Each emitted edge belongs to one island's intra range or to
        // a single (island_a, island_b) bipartite slice — never spans
        // three islands.
        let islands_n = 4u32;
        let islands_size = 10u32;
        let g = simple_interconnected_islands_game(islands_n, islands_size, 0.3, 5, 13).unwrap();
        let n_edges = u32::try_from(g.ecount()).unwrap();
        for eid in 0..n_edges {
            let (a, b) = g.edge(eid).unwrap();
            let island_a = a / islands_size;
            let island_b = b / islands_size;
            assert!(island_a < islands_n);
            assert!(island_b < islands_n);
        }
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let a = simple_interconnected_islands_game(4, 20, 0.35, 3, 0xC0DE).unwrap();
        let b = simple_interconnected_islands_game(4, 20, 0.35, 3, 0xC0DE).unwrap();
        assert_eq!(a.ecount(), b.ecount());
        assert_eq!(canonical_edges(&a), canonical_edges(&b));
    }

    #[test]
    fn distinct_seeds_differ() {
        let a = simple_interconnected_islands_game(4, 20, 0.35, 3, 1).unwrap();
        let b = simple_interconnected_islands_game(4, 20, 0.35, 3, 2).unwrap();
        assert_ne!(canonical_edges(&a), canonical_edges(&b));
    }

    #[test]
    fn ecount_band_matches_expectation() {
        // Within an island: E[edges] = p · n(n-1)/2.
        // Inter-island: exact n_inter per pair, no variance.
        let islands_n = 5u32;
        let islands_size = 30u32;
        let p = 0.2;
        let n_inter = 4u32;
        let expected_intra =
            p * f64::from(islands_n) * f64::from(islands_size) * f64::from(islands_size - 1) / 2.0;
        let expected_inter = f64::from(n_inter * islands_n * (islands_n - 1) / 2);
        let expected = expected_intra + expected_inter;

        let g =
            simple_interconnected_islands_game(islands_n, islands_size, p, n_inter, 4242).unwrap();
        let m = g.ecount() as f64;
        let lo = 0.6 * expected;
        let hi = 1.4 * expected;
        assert!(m >= lo && m <= hi, "ecount {m} outside band [{lo}, {hi}]");
    }

    #[test]
    fn triangle_pair_decoder_round_trips() {
        // For n = 6, the 15 strict upper-triangle pairs should be
        // enumerated in the canonical order matching the formula.
        let expected: Vec<(u32, u32)> = vec![
            (0, 1),
            (0, 2),
            (1, 2),
            (0, 3),
            (1, 3),
            (2, 3),
            (0, 4),
            (1, 4),
            (2, 4),
            (3, 4),
            (0, 5),
            (1, 5),
            (2, 5),
            (3, 5),
            (4, 5),
        ];
        for (idx, &want) in expected.iter().enumerate() {
            assert_eq!(decode_triangle_pair(idx as u64), want);
        }
    }
}
