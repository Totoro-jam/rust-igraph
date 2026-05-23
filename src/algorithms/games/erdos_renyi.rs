//! Erdős–Rényi random graph generators (ALGO-GN-001).
//!
//! Counterpart of `igraph_erdos_renyi_game_gnp()` and
//! `igraph_erdos_renyi_game_gnm()` from
//! `references/igraph/src/games/erdos_renyi.c:715` and `:429`.
//!
//! Two classic models — both fix the vertex count `n`:
//!
//! * **G(n, p)**: every possible vertex pair is independently connected
//!   with probability `p`. The expected edge count is
//!   `p · max_edges`. Sampled via Batagelj & Brandes 2005's
//!   geometric-skip algorithm so the cost is `O(|V| + |E|)` rather than
//!   `O(|V|²)`.
//!
//! * **G(n, m)**: exactly `m` distinct edges are chosen uniformly at
//!   random from the `max_edges` possible. Sampled with Floyd's
//!   algorithm for an `O(m)` distinct-pair draw, then each pair-index is
//!   decoded into a `(from, to)` pair using the same formulas the
//!   upstream C implementation uses.
//!
//! Both functions are deterministic given the `seed` argument and run
//! against the shared [`crate::core::rng::SplitMix64`] PRNG.
//!
//! ## Scope
//!
//! MVP scope ports the most-used path: **simple graphs**, with `loops`
//! optionally enabled. Multigraphs (`multiple = true`) and the
//! edge-labeled IEA variant are out of scope for this AWU — they will
//! land as follow-up AWUs if real users ask for them. Returning an
//! `IgraphError::Unimplemented`-style variant from a hypothetical
//! multigraph flag would be the wrong API shape (the flag could never
//! be true), so we simply omit it from the public surface.
//!
//! ## References
//!
//! * V. Batagelj and U. Brandes,
//!   *"Efficient generation of large random networks"*,
//!   Phys. Rev. E **71**, 036113 (2005).
//! * R. W. Floyd, *Algorithm 489: The algorithm `SELECT` …*, CACM (1972)
//!   — the distinct-sample technique used in G(n, m).

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::manual_midpoint
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Decode a linear pair-index `idx ∈ [0, max_edges)` into a `(from, to)`
/// vertex pair. The four cases mirror the upstream C decoders in
/// `erdos_renyi.c:339-374` and `:806-841`.
fn decode_pair(idx: u64, n: u32, directed: bool, loops: bool) -> (VertexId, VertexId) {
    let n_u64 = u64::from(n);
    if directed && loops {
        // n × n grid, row = `to`, col = `from`.
        let to = idx / n_u64;
        let from = idx - to * n_u64;
        debug_assert!(from < n_u64 && to < n_u64);
        #[allow(clippy::cast_possible_truncation)]
        ((from as VertexId), (to as VertexId))
    } else if directed && !loops {
        // n × n grid sampled at n*(n-1) cells; diagonal elements get
        // remapped to column `n-1`. Matches `erdos_renyi.c:814-824`.
        let to = idx / n_u64;
        let from = idx - to * n_u64;
        let to = if from == to { n_u64 - 1 } else { to };
        debug_assert!(from < n_u64 && to < n_u64 && from != to);
        #[allow(clippy::cast_possible_truncation)]
        ((from as VertexId), (to as VertexId))
    } else if !directed && loops {
        // Triangular numbering INCLUDING the diagonal:
        //   to = floor((sqrt(8·idx + 1) - 1) / 2)
        //   from = idx - to·(to+1)/2
        // produces `0 <= from <= to < n`.
        #[allow(clippy::cast_precision_loss)]
        let idx_f = idx as f64;
        let to_f = ((8.0 * idx_f + 1.0).sqrt() - 1.0) / 2.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut to = to_f.trunc() as u64;
        // Recompute `from` exactly in integer arithmetic; the float
        // truncation may have rounded down by one, so verify the
        // result lies in [0, to] and bump if needed.
        let mut from = idx - to * (to + 1) / 2;
        while from > to {
            to += 1;
            from = idx - to * (to + 1) / 2;
        }
        debug_assert!(from <= to && to < n_u64);
        #[allow(clippy::cast_possible_truncation)]
        ((from as VertexId), (to as VertexId))
    } else {
        // !directed && !loops — triangular numbering EXCLUDING the diagonal:
        //   to = floor((sqrt(8·idx + 1) + 1) / 2)
        //   from = idx - to·(to-1)/2
        // produces `0 <= from < to < n`.
        #[allow(clippy::cast_precision_loss)]
        let idx_f = idx as f64;
        let to_f = ((8.0 * idx_f + 1.0).sqrt() + 1.0) / 2.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut to = to_f.trunc() as u64;
        if to < 1 {
            to = 1;
        }
        let mut from = idx - to * (to - 1) / 2;
        while from >= to {
            to += 1;
            from = idx - to * (to - 1) / 2;
        }
        debug_assert!(from < to && to < n_u64);
        #[allow(clippy::cast_possible_truncation)]
        ((from as VertexId), (to as VertexId))
    }
}

/// Number of *possible* edges for the four `(directed, loops)` cases.
/// Returned as `u64` so callers walking pair-index space don't
/// silently truncate at `n ≈ 65536` (where `n² > u32::MAX`).
fn max_edges(n: u32, directed: bool, loops: bool) -> u64 {
    let n = u64::from(n);
    match (directed, loops) {
        (true, true) => n * n,
        (true, false) => n * n.saturating_sub(1),
        (false, true) => n * (n + 1) / 2,
        (false, false) => n * n.saturating_sub(1) / 2,
    }
}

/// Sample `m` distinct integers from `[0, n_pairs)` using Floyd's
/// algorithm.
///
/// Floyd's invariant: after step `j ∈ [n-m, n)`, the set `S` contains a
/// uniform random `m`-subset of `[0, j+1)`. The procedure performs
/// exactly `m` PRNG draws and `m` `HashSet` insertions — no rejection
/// retries — and uses `O(m)` memory.
///
/// Returns the sampled indices in *sorted* order for deterministic
/// edge ordering (handy for property tests and reproducible bench
/// inputs).
fn distinct_sample(rng: &mut SplitMix64, n_pairs: u64, m: u64) -> Vec<u64> {
    debug_assert!(m <= n_pairs);
    let cap = usize::try_from(m).unwrap_or(usize::MAX);
    let mut chosen: HashSet<u64> = HashSet::with_capacity(cap);
    let mut out: Vec<u64> = Vec::with_capacity(cap);
    for j in (n_pairs - m)..n_pairs {
        // Uniform in [0, j+1). For typical graphs `j+1 < 2^53` so the
        // double-precision intermediate is exact.
        let span = j + 1;
        let span_usize = usize::try_from(span).unwrap_or(usize::MAX);
        // gen_index takes usize. For n_pairs > usize::MAX we'd overflow;
        // we already gate that out below by capping m.
        let t_usize = rng.gen_index(span_usize);
        let t = t_usize as u64;
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

fn validate_gnp(p: f64) -> IgraphResult<()> {
    if !p.is_finite() {
        return Err(IgraphError::InvalidArgument(format!(
            "G(n,p) probability must be finite (got {p})"
        )));
    }
    if !(0.0..=1.0).contains(&p) {
        return Err(IgraphError::InvalidArgument(format!(
            "G(n,p) probability must be in [0, 1] (got {p})"
        )));
    }
    Ok(())
}

fn validate_gnm(n: u32, m: u64, directed: bool, loops: bool) -> IgraphResult<()> {
    let cap = max_edges(n, directed, loops);
    if m > cap {
        return Err(IgraphError::InvalidArgument(format!(
            "G(n,m) requested {m} edges but the {} graph on {n} vertices admits at most {cap}",
            if directed { "directed" } else { "undirected" }
        )));
    }
    Ok(())
}

/// Build a `Graph` with `n` vertices, `directed`, and the given edge
/// list (already decoded from pair-indices).
fn finalize(n: u32, directed: bool, edges: Vec<(VertexId, VertexId)>) -> IgraphResult<Graph> {
    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Generate a complete graph `K_n` (with self-loops if `loops`).
fn complete(n: u32, directed: bool, loops: bool) -> IgraphResult<Graph> {
    let cap = max_edges(n, directed, loops);
    let cap_usize = usize::try_from(cap)
        .map_err(|_| IgraphError::Internal("complete graph edge count exceeds usize"))?;
    let mut edges = Vec::with_capacity(cap_usize);
    for i in 0..n {
        if directed {
            for j in 0..n {
                if i == j && !loops {
                    continue;
                }
                edges.push((i, j));
            }
        } else {
            let start = if loops { i } else { i + 1 };
            for j in start..n {
                edges.push((i, j));
            }
        }
    }
    finalize(n, directed, edges)
}

/// Generate a random graph from the **G(n, p)** Erdős–Rényi model.
///
/// Every possible edge is included independently with probability `p`.
/// The expected number of edges is `p · max_edges(n, directed, loops)`.
///
/// * `n` — vertex count.
/// * `p ∈ [0, 1]` — edge probability.
/// * `directed` — generate a directed graph (ordered pairs); when
///   `false`, edges are undirected.
/// * `loops` — when `true`, self-loops are allowed; when `false`, no
///   self-loop edges are sampled.
/// * `seed` — initialises an internal `SplitMix64` PRNG. Same
///   `(n, p, directed, loops, seed)` always yields the same graph.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `p` is NaN, infinite,
/// or outside `[0, 1]`.
///
/// # Examples
///
/// ```
/// use rust_igraph::erdos_renyi_gnp;
/// // Expected edges ≈ p · n(n-1)/2 = 0.5 · 100·99/2 = 2475.
/// let g = erdos_renyi_gnp(100, 0.5, false, false, 42).unwrap();
/// assert_eq!(g.vcount(), 100);
/// // The actual count fluctuates around the expectation, but for
/// // p = 0.5 it lies in a narrow band.
/// let m = g.ecount();
/// assert!(m > 2200 && m < 2700, "ecount={m}");
/// ```
pub fn erdos_renyi_gnp(
    n: u32,
    p: f64,
    directed: bool,
    loops: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate_gnp(p)?;

    let cap = max_edges(n, directed, loops);
    if n == 0 || p == 0.0 || cap == 0 {
        return Graph::new(n, directed);
    }
    if p == 1.0 {
        return complete(n, directed, loops);
    }

    let mut rng = SplitMix64::new(seed);
    // Batagelj–Brandes geometric-skip walk over the linear pair-index
    // space [0, cap). At each step, advance by `RNG_GEOM(p) + 1` (the
    // +1 enforces strictly-increasing indices so the sample is a
    // *simple* graph).
    #[allow(clippy::cast_precision_loss)]
    let cap_f = cap as f64;
    let mut last = rng.gen_geom(p);
    let mut indices: Vec<u64> = Vec::new();
    while last < cap_f {
        // Convert via trunc — last is always non-negative.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx = last.trunc() as u64;
        if idx >= cap {
            break;
        }
        indices.push(idx);
        last += rng.gen_geom(p);
        last += 1.0; // simple-graph step
    }

    let edges: Vec<(VertexId, VertexId)> = indices
        .into_iter()
        .map(|idx| decode_pair(idx, n, directed, loops))
        .collect();
    finalize(n, directed, edges)
}

/// Generate a random graph from the **G(n, m)** Erdős–Rényi model.
///
/// Exactly `m` edges are drawn uniformly at random from the
/// `max_edges(n, directed, loops)` possible edges. Sampling is
/// without replacement (simple graph).
///
/// * `n` — vertex count.
/// * `m` — edge count.
/// * `directed`, `loops`, `seed` — see [`erdos_renyi_gnp`].
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `m` exceeds
/// `max_edges(n, directed, loops)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::erdos_renyi_gnm;
/// let g = erdos_renyi_gnm(20, 30, false, false, 7).unwrap();
/// assert_eq!(g.vcount(), 20);
/// assert_eq!(g.ecount(), 30);
/// ```
pub fn erdos_renyi_gnm(
    n: u32,
    m: u64,
    directed: bool,
    loops: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate_gnm(n, m, directed, loops)?;

    if m == 0 {
        return Graph::new(n, directed);
    }

    let cap = max_edges(n, directed, loops);
    if m == cap {
        return complete(n, directed, loops);
    }

    let mut rng = SplitMix64::new(seed);
    let picks = distinct_sample(&mut rng, cap, m);
    let edges: Vec<(VertexId, VertexId)> = picks
        .into_iter()
        .map(|idx| decode_pair(idx, n, directed, loops))
        .collect();
    finalize(n, directed, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ------- decode_pair sanity -------

    #[test]
    fn decode_undirected_no_loops_covers_strict_upper_triangle() {
        // For n=5, max_edges = 10. Every idx in [0, 10) must produce a
        // pair (from < to) with both vertices in [0, 5).
        let n = 5u32;
        let cap = max_edges(n, false, false);
        assert_eq!(cap, 10);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n, false, false);
            assert!(u < v && v < n);
            assert!(seen.insert((u, v)), "duplicate pair at idx {idx}");
        }
        assert_eq!(seen.len(), cap as usize);
    }

    #[test]
    fn decode_undirected_with_loops_covers_lower_triangle_incl_diagonal() {
        let n = 4u32;
        let cap = max_edges(n, false, true);
        assert_eq!(cap, 10);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n, false, true);
            assert!(u <= v && v < n);
            assert!(seen.insert((u, v)));
        }
        assert_eq!(seen.len(), cap as usize);
    }

    #[test]
    fn decode_directed_no_loops_covers_off_diagonal() {
        let n = 5u32;
        let cap = max_edges(n, true, false);
        assert_eq!(cap, 20);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n, true, false);
            assert!(u < n && v < n && u != v);
            assert!(seen.insert((u, v)));
        }
        assert_eq!(seen.len(), cap as usize);
    }

    #[test]
    fn decode_directed_with_loops_covers_full_grid() {
        let n = 4u32;
        let cap = max_edges(n, true, true);
        assert_eq!(cap, 16);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n, true, true);
            assert!(u < n && v < n);
            assert!(seen.insert((u, v)));
        }
        assert_eq!(seen.len(), cap as usize);
    }

    // ------- gnp boundary cases -------

    #[test]
    fn gnp_p_zero_is_empty() {
        let g = erdos_renyi_gnp(10, 0.0, false, false, 1).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn gnp_p_one_is_complete() {
        let g = erdos_renyi_gnp(6, 1.0, false, false, 1).unwrap();
        assert_eq!(g.vcount(), 6);
        assert_eq!(g.ecount(), 15); // 6 choose 2
    }

    #[test]
    fn gnp_directed_p_one_loops_is_full_grid() {
        let g = erdos_renyi_gnp(4, 1.0, true, true, 1).unwrap();
        assert_eq!(g.ecount(), 16);
    }

    #[test]
    fn gnp_invalid_p_rejected() {
        assert!(erdos_renyi_gnp(5, -0.1, false, false, 1).is_err());
        assert!(erdos_renyi_gnp(5, 1.1, false, false, 1).is_err());
        assert!(erdos_renyi_gnp(5, f64::NAN, false, false, 1).is_err());
        assert!(erdos_renyi_gnp(5, f64::INFINITY, false, false, 1).is_err());
    }

    #[test]
    fn gnp_zero_vertices_is_empty() {
        let g = erdos_renyi_gnp(0, 0.5, false, false, 1).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
    }

    // ------- gnp determinism + distribution sanity -------

    #[test]
    fn gnp_deterministic_with_seed() {
        let a = erdos_renyi_gnp(50, 0.3, false, false, 12345).unwrap();
        let b = erdos_renyi_gnp(50, 0.3, false, false, 12345).unwrap();
        assert_eq!(a.vcount(), b.vcount());
        assert_eq!(a.ecount(), b.ecount());
        // Edge sets must match exactly.
        let edges_a: Vec<_> = (0..a.ecount()).map(|e| a.edge(e as u32).unwrap()).collect();
        let edges_b: Vec<_> = (0..b.ecount()).map(|e| b.edge(e as u32).unwrap()).collect();
        assert_eq!(edges_a, edges_b);
    }

    #[test]
    fn gnp_different_seeds_differ() {
        let a = erdos_renyi_gnp(200, 0.05, false, false, 1).unwrap();
        let b = erdos_renyi_gnp(200, 0.05, false, false, 2).unwrap();
        // Tiny chance of accidental equality but vanishingly small here.
        assert_ne!(a.ecount(), b.ecount());
    }

    #[test]
    fn gnp_expected_ecount_in_band() {
        // E[m] = p · n(n-1)/2 = 0.2 · 100·99/2 = 990. Sample stddev ≈
        // sqrt(p(1-p) · n(n-1)/2) ≈ sqrt(0.16 · 4950) ≈ 28; ±100 is
        // ~3.5σ — generous but tight enough to catch a broken sampler.
        let g = erdos_renyi_gnp(100, 0.2, false, false, 31_415).unwrap();
        let m = g.ecount();
        assert!(m > 890 && m < 1090, "ecount = {m}");
    }

    #[test]
    fn gnp_is_simple_no_loops() {
        let g = erdos_renyi_gnp(50, 0.4, false, false, 7).unwrap();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            assert_ne!(u, v, "self-loop generated at edge {e}");
        }
    }

    #[test]
    fn gnp_no_parallel_edges_simple() {
        let g = erdos_renyi_gnp(40, 0.3, false, false, 11).unwrap();
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..g.ecount() {
            let pair = g.edge(e as u32).unwrap();
            assert!(seen.insert(pair), "parallel edge at {e}: {pair:?}");
        }
    }

    // ------- gnm boundary cases -------

    #[test]
    fn gnm_m_zero_is_empty() {
        let g = erdos_renyi_gnm(10, 0, false, false, 1).unwrap();
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn gnm_m_max_is_complete() {
        let g = erdos_renyi_gnm(6, 15, false, false, 1).unwrap();
        assert_eq!(g.ecount(), 15);
    }

    #[test]
    fn gnm_m_exceeds_capacity_rejected() {
        // K_5 has 10 undirected edges; asking for 11 is impossible.
        assert!(erdos_renyi_gnm(5, 11, false, false, 1).is_err());
    }

    #[test]
    fn gnm_exact_edge_count() {
        let g = erdos_renyi_gnm(50, 100, false, false, 42).unwrap();
        assert_eq!(g.ecount(), 100);
        assert_eq!(g.vcount(), 50);
    }

    #[test]
    fn gnm_deterministic_with_seed() {
        let a = erdos_renyi_gnm(30, 60, false, false, 7).unwrap();
        let b = erdos_renyi_gnm(30, 60, false, false, 7).unwrap();
        let edges_a: Vec<_> = (0..a.ecount()).map(|e| a.edge(e as u32).unwrap()).collect();
        let edges_b: Vec<_> = (0..b.ecount()).map(|e| b.edge(e as u32).unwrap()).collect();
        assert_eq!(edges_a, edges_b);
    }

    #[test]
    fn gnm_is_simple_no_loops() {
        let g = erdos_renyi_gnm(20, 40, false, false, 99).unwrap();
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            assert_ne!(u, v);
            assert!(seen.insert((u, v)), "parallel edge: {u} {v}");
        }
    }

    #[test]
    fn gnm_directed_loops_full_grid() {
        // n=3, max directed-with-loops edges = 9. Drawing all 9 must
        // yield exactly one of each (i, j) pair.
        let g = erdos_renyi_gnm(3, 9, true, true, 0).unwrap();
        assert_eq!(g.ecount(), 9);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..g.ecount() {
            seen.insert(g.edge(e as u32).unwrap());
        }
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn gnm_directed_no_loops() {
        // n=4 directed without loops: max = 12. Drawing all 12.
        let g = erdos_renyi_gnm(4, 12, true, false, 0).unwrap();
        assert_eq!(g.ecount(), 12);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..g.ecount() {
            let (u, v) = g.edge(e as u32).unwrap();
            assert_ne!(u, v);
            seen.insert((u, v));
        }
        assert_eq!(seen.len(), 12);
    }

    // ------- proptest harness (gated; runs under `cargo test --features proptest-harness`) -------

    #[cfg(feature = "proptest-harness")]
    mod prop {
        use super::super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn gnp_vcount_always_matches_n(
                n in 0u32..50,
                p in 0.0..=1.0,
                directed in any::<bool>(),
                loops in any::<bool>(),
                seed in any::<u64>(),
            ) {
                let g = erdos_renyi_gnp(n, p, directed, loops, seed).unwrap();
                prop_assert_eq!(g.vcount(), n);
                prop_assert!(g.ecount() as u64 <= max_edges(n, directed, loops));
            }

            #[test]
            fn gnp_simple_no_self_loops_no_multi(
                n in 1u32..30,
                p in 0.0..=0.7,
                seed in any::<u64>(),
            ) {
                let g = erdos_renyi_gnp(n, p, false, false, seed).unwrap();
                let mut seen = std::collections::HashSet::new();
                for e in 0..g.ecount() {
                    let (u, v) = g.edge(e as u32).unwrap();
                    prop_assert!(u != v);
                    prop_assert!(seen.insert((u, v)));
                }
            }

            #[test]
            fn gnm_exact_count_and_simple(
                n in 2u32..30,
                seed in any::<u64>(),
            ) {
                // Pick m up to max-1 to exercise non-trivial Floyd path.
                let cap = max_edges(n, false, false);
                if cap == 0 { return Ok(()); }
                // Bound m so we stay deterministic w.r.t. seed.
                let m = (seed as u64) % cap;
                let g = erdos_renyi_gnm(n, m, false, false, seed).unwrap();
                prop_assert_eq!(g.ecount(), m as usize);
                let mut seen = std::collections::HashSet::new();
                for e in 0..g.ecount() {
                    let (u, v) = g.edge(e as u32).unwrap();
                    prop_assert!(u != v);
                    prop_assert!(seen.insert((u, v)));
                }
            }

            #[test]
            fn gnp_determinism(
                n in 1u32..40,
                p in 0.05..0.95,
                directed in any::<bool>(),
                loops in any::<bool>(),
                seed in any::<u64>(),
            ) {
                let a = erdos_renyi_gnp(n, p, directed, loops, seed).unwrap();
                let b = erdos_renyi_gnp(n, p, directed, loops, seed).unwrap();
                prop_assert_eq!(a.ecount(), b.ecount());
                for e in 0..a.ecount() {
                    prop_assert_eq!(a.edge(e as u32).unwrap(), b.edge(e as u32).unwrap());
                }
            }
        }
    }
}
