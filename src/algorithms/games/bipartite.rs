//! Bipartite Erdős–Rényi random graph generators (ALGO-GN-030).
//!
//! Counterpart of `igraph_bipartite_game_gnp()` and
//! `igraph_bipartite_game_gnm()` from
//! `references/igraph/src/misc/bipartite.c:1646-1784` and `:1365-1415`.
//!
//! Two bipartite analogues of the classical Erdős–Rényi models. Every
//! result graph has `n1 + n2` vertices: the **bottom** partition occupies
//! ids `0..n1` (`types[v] = false`), the **top** partition occupies ids
//! `n1..n1+n2` (`types[v] = true`). Bipartite graphs have no self-loops
//! and no within-partition edges; that constraint is enforced
//! structurally by the pair-index decoders.
//!
//! * **G(n1, n2, p)**: every potential cross-partition edge is included
//!   independently with probability `p`. Sampled with the
//!   Batagelj–Brandes 2005 geometric-skip walk so the cost is `O(n1 +
//!   n2 + |E|)` rather than `O(n1·n2)`.
//!
//! * **G(n1, n2, m)**: exactly `m` distinct cross-partition edges drawn
//!   uniformly at random from the `max_edges(n1, n2, directed, mode)`
//!   possible. Sampled with Floyd's algorithm for an `O(m)` distinct
//!   draw.
//!
//! ## Edge direction (directed graphs)
//!
//! [`BipartiteMode`] mirrors igraph's `IGRAPH_OUT` / `IGRAPH_IN` /
//! `IGRAPH_ALL`:
//!
//! * [`BipartiteMode::Out`] — arcs go bottom → top.
//! * [`BipartiteMode::In`] — arcs go top → bottom.
//! * [`BipartiteMode::All`] — each ordered pair is sampled
//!   independently, so mutual pairs `(b → t, t → b)` are possible. The
//!   pair space doubles to `2·n1·n2`.
//!
//! For undirected graphs the mode argument is ignored; the sampler walks
//! the same `n1·n2` pair space and stores each edge canonically.
//!
//! ## Determinism
//!
//! Both functions are deterministic given the `seed` argument and run
//! against the shared [`crate::core::rng::SplitMix64`] PRNG.
//!
//! ## Scope
//!
//! MVP scope ports the most-used path: **simple bipartite graphs**.
//! Upstream's `IGRAPH_MULTI_SW` (multigraphs), `IGRAPH_EDGE_LABELED`
//! (IEA over ordered edge lists), and the `gnp_bipartite_large`
//! overflow path for `n1·n2 > 2^53` are out of scope for this AWU —
//! they will land as follow-up AWUs if real users ask for them.
//!
//! ## References
//!
//! * V. Batagelj and U. Brandes, *"Efficient generation of large random
//!   networks"*, Phys. Rev. E **71**, 036113 (2005).
//! * R. W. Floyd, *Algorithm 489: The algorithm SELECT …*, CACM (1972).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::collections::HashSet;

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Edge-direction policy for the directed variants of the bipartite
/// Erdős–Rényi models. See module docs for the mode semantics.
///
/// `IGRAPH_ALL` translates to [`BipartiteMode::All`]: each ordered pair
/// `(b, t)` and `(t, b)` is sampled *independently*, so mutual edges
/// between the same `b` ∈ bottom and `t` ∈ top can be produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BipartiteMode {
    /// Arcs from bottom to top. Pair space `n1·n2`.
    Out,
    /// Arcs from top to bottom. Pair space `n1·n2`.
    In,
    /// Each ordered direction sampled independently. Pair space
    /// `2·n1·n2`; mutual pairs are possible.
    All,
}

/// Result of [`bipartite_game_gnp`] / [`bipartite_game_gnm`]: the
/// graph together with the boolean type vector that names every vertex
/// as bottom (`false`) or top (`true`).
#[derive(Clone, Debug)]
pub struct BipartiteGraph {
    /// Graph on `n1 + n2` vertices.
    pub graph: Graph,
    /// Per-vertex partition label. `types[v] == false` for
    /// `v ∈ 0..n1` (bottom), `types[v] == true` for `v ∈ n1..n1+n2`
    /// (top).
    pub types: Vec<bool>,
}

/// Number of *possible* edges for a bipartite graph on `n1` bottom and
/// `n2` top vertices. Returned as `u64` so callers walking pair-index
/// space don't silently truncate.
fn max_edges(n1: u32, n2: u32, directed: bool, mode: BipartiteMode) -> u64 {
    let prod = u64::from(n1) * u64::from(n2);
    if directed && matches!(mode, BipartiteMode::All) {
        prod.saturating_mul(2)
    } else {
        prod
    }
}

/// Build the type vector for a bipartite graph: `n1` bottom vertices
/// labelled `false`, then `n2` top vertices labelled `true`.
fn types_vector(n1: u32, n2: u32) -> Vec<bool> {
    let total = (n1 as usize).saturating_add(n2 as usize);
    let mut v = Vec::with_capacity(total);
    v.resize(n1 as usize, false);
    v.resize(total, true);
    v
}

/// Decode a linear pair-index `idx` into a directed `(from, to)` arc
/// already mapped into the joint vertex id space `[0, n1+n2)`.
///
/// The decoder mirrors `references/igraph/src/misc/bipartite.c:1746-1762`
/// byte-for-byte. For directed-with-ALL the first `n1·n2` indices encode
/// bottom→top arcs and the remaining `n1·n2` encode top→bottom arcs.
fn decode_pair(
    idx: u64,
    n1: u32,
    n2: u32,
    directed: bool,
    mode: BipartiteMode,
) -> (VertexId, VertexId) {
    let n1_u64 = u64::from(n1);
    let n2_u64 = u64::from(n2);
    let n1n2 = n1_u64 * n2_u64;

    let (from, to) = if !directed || !matches!(mode, BipartiteMode::All) {
        // Single rectangular cell of size n1 × n2.
        // to walks the top partition, from walks the bottom.
        let to_off = idx / n1_u64;
        let from_off = idx - to_off * n1_u64;
        debug_assert!(from_off < n1_u64 && to_off < n2_u64);
        (from_off, to_off + n1_u64)
    } else if idx < n1n2 {
        // Directed + ALL, bottom→top half.
        let to_off = idx / n1_u64;
        let from_off = idx - to_off * n1_u64;
        debug_assert!(from_off < n1_u64 && to_off < n2_u64);
        (from_off, to_off + n1_u64)
    } else {
        // Directed + ALL, top→bottom half.
        let rel = idx - n1n2;
        let to_off = rel / n2_u64;
        let from_off = rel - to_off * n2_u64;
        debug_assert!(from_off < n2_u64 && to_off < n1_u64);
        (from_off + n1_u64, to_off)
    };

    // For mode == In we swap so that the recorded arc points top→bottom.
    let (from, to) = if matches!(mode, BipartiteMode::In) && directed {
        (to, from)
    } else {
        (from, to)
    };

    #[allow(clippy::cast_possible_truncation)]
    (from as VertexId, to as VertexId)
}

/// Sample `m` distinct integers from `[0, n_pairs)` using Floyd's
/// algorithm. Same routine as in `erdos_renyi`, duplicated here to keep
/// per-module independence (we don't want the bipartite module to
/// reach across `super::erdos_renyi` for a private helper).
fn distinct_sample(rng: &mut SplitMix64, n_pairs: u64, m: u64) -> Vec<u64> {
    debug_assert!(m <= n_pairs);
    let cap = usize::try_from(m).unwrap_or(usize::MAX);
    let mut chosen: HashSet<u64> = HashSet::with_capacity(cap);
    let mut out: Vec<u64> = Vec::with_capacity(cap);
    for j in (n_pairs - m)..n_pairs {
        let span = j + 1;
        let span_usize = usize::try_from(span).unwrap_or(usize::MAX);
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
            "bipartite G(n,p) probability must be finite (got {p})"
        )));
    }
    if !(0.0..=1.0).contains(&p) {
        return Err(IgraphError::InvalidArgument(format!(
            "bipartite G(n,p) probability must be in [0, 1] (got {p})"
        )));
    }
    Ok(())
}

fn validate_gnm(n1: u32, n2: u32, m: u64, directed: bool, mode: BipartiteMode) -> IgraphResult<()> {
    let cap = max_edges(n1, n2, directed, mode);
    if m > cap {
        return Err(IgraphError::InvalidArgument(format!(
            "bipartite G(n,m) requested {m} edges but the {} graph on {n1}+{n2} vertices admits at most {cap}",
            if directed { "directed" } else { "undirected" }
        )));
    }
    if cap == 0 && m > 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "bipartite G(n,m) requested {m} edges but n1={n1} or n2={n2} is zero"
        )));
    }
    Ok(())
}

/// Build the result from an already-decoded edge list.
fn finalize(
    n1: u32,
    n2: u32,
    directed: bool,
    edges: Vec<(VertexId, VertexId)>,
) -> IgraphResult<BipartiteGraph> {
    let n_total = u32::try_from(u64::from(n1) + u64::from(n2)).map_err(|_| {
        IgraphError::InvalidArgument(format!("n1 + n2 overflows u32 (n1={n1}, n2={n2})"))
    })?;
    let mut g = Graph::new(n_total, directed)?;
    g.add_edges(edges)?;
    Ok(BipartiteGraph {
        graph: g,
        types: types_vector(n1, n2),
    })
}

/// Build the complete bipartite graph K_{n1, n2}, with arc orientation
/// determined by `(directed, mode)`. Used for the `p == 1` fast path of
/// `bipartite_game_gnp` and the `m == max_edges` fast path of `_gnm`.
fn complete_bipartite(
    n1: u32,
    n2: u32,
    directed: bool,
    mode: BipartiteMode,
) -> IgraphResult<BipartiteGraph> {
    let cap = max_edges(n1, n2, directed, mode);
    let cap_usize = usize::try_from(cap)
        .map_err(|_| IgraphError::Internal("complete bipartite edge count exceeds usize"))?;
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(cap_usize);
    for idx in 0..cap {
        edges.push(decode_pair(idx, n1, n2, directed, mode));
    }
    finalize(n1, n2, directed, edges)
}

/// Generate a random bipartite graph from the **G(n1, n2, p)** model.
///
/// Every possible cross-partition edge is included independently with
/// probability `p`. The expected number of edges is
/// `p · max_edges(n1, n2, directed, mode)`.
///
/// * `n1` — bottom partition size (vertices `0..n1`).
/// * `n2` — top partition size (vertices `n1..n1+n2`).
/// * `p ∈ [0, 1]` — edge probability.
/// * `directed` — generate a directed bipartite graph; when `false`
///   edges are undirected and `mode` is effectively `All` (pair space
///   `n1·n2`, no mutual pairs because the graph is undirected).
/// * `mode` — see [`BipartiteMode`]. Ignored when `directed == false`.
/// * `seed` — initialises an internal `SplitMix64` PRNG. The same
///   `(n1, n2, p, directed, mode, seed)` always yields the same graph.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `p` is NaN, infinite, or
/// outside `[0, 1]`, or if `n1 + n2` overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{bipartite_game_gnp, BipartiteMode};
/// // Expected edges ≈ 0.3 · 5 · 8 = 12.
/// let bg = bipartite_game_gnp(5, 8, 0.3, false, BipartiteMode::All, 42).unwrap();
/// assert_eq!(bg.graph.vcount(), 13);
/// assert_eq!(bg.types.len(), 13);
/// // Every edge crosses the partition.
/// for eid in 0..bg.graph.ecount() {
///     let (u, v) = bg.graph.edge(eid as u32).unwrap();
///     assert_ne!(bg.types[u as usize], bg.types[v as usize]);
/// }
/// ```
pub fn bipartite_game_gnp(
    n1: u32,
    n2: u32,
    p: f64,
    directed: bool,
    mode: BipartiteMode,
    seed: u64,
) -> IgraphResult<BipartiteGraph> {
    validate_gnp(p)?;

    let n_total = u32::try_from(u64::from(n1) + u64::from(n2)).map_err(|_| {
        IgraphError::InvalidArgument(format!("n1 + n2 overflows u32 (n1={n1}, n2={n2})"))
    })?;

    let cap = max_edges(n1, n2, directed, mode);
    if cap == 0 || p == 0.0 {
        let g = Graph::new(n_total, directed)?;
        return Ok(BipartiteGraph {
            graph: g,
            types: types_vector(n1, n2),
        });
    }
    if p == 1.0 {
        return complete_bipartite(n1, n2, directed, mode);
    }

    let mut rng = SplitMix64::new(seed);
    // Batagelj–Brandes geometric-skip walk over the linear pair-index
    // space [0, cap). At each step, advance by `RNG_GEOM(p) + 1` (the
    // +1 enforces strictly-increasing indices so the sample is a
    // *simple* bipartite graph).
    #[allow(clippy::cast_precision_loss)]
    let cap_f = cap as f64;
    let mut last = rng.gen_geom(p);
    let mut indices: Vec<u64> = Vec::new();
    while last < cap_f {
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
        .map(|idx| decode_pair(idx, n1, n2, directed, mode))
        .collect();
    finalize(n1, n2, directed, edges)
}

/// Generate a random bipartite graph from the **G(n1, n2, m)** model.
///
/// Exactly `m` cross-partition edges are drawn uniformly at random
/// from the `max_edges(n1, n2, directed, mode)` possible ones.
/// Sampling is without replacement (simple bipartite graph).
///
/// * `n1`, `n2`, `directed`, `mode`, `seed` — see [`bipartite_game_gnp`].
/// * `m` — exact edge count.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `m` exceeds the
/// `max_edges(n1, n2, directed, mode)` capacity, or if `n1 + n2`
/// overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{bipartite_game_gnm, BipartiteMode};
/// let bg = bipartite_game_gnm(4, 6, 10, false, BipartiteMode::All, 7).unwrap();
/// assert_eq!(bg.graph.vcount(), 10);
/// assert_eq!(bg.graph.ecount(), 10);
/// // Every edge crosses the partition.
/// for eid in 0..bg.graph.ecount() {
///     let (u, v) = bg.graph.edge(eid as u32).unwrap();
///     assert_ne!(bg.types[u as usize], bg.types[v as usize]);
/// }
/// ```
pub fn bipartite_game_gnm(
    n1: u32,
    n2: u32,
    m: u64,
    directed: bool,
    mode: BipartiteMode,
    seed: u64,
) -> IgraphResult<BipartiteGraph> {
    validate_gnm(n1, n2, m, directed, mode)?;

    let n_total = u32::try_from(u64::from(n1) + u64::from(n2)).map_err(|_| {
        IgraphError::InvalidArgument(format!("n1 + n2 overflows u32 (n1={n1}, n2={n2})"))
    })?;

    if m == 0 {
        let g = Graph::new(n_total, directed)?;
        return Ok(BipartiteGraph {
            graph: g,
            types: types_vector(n1, n2),
        });
    }

    let cap = max_edges(n1, n2, directed, mode);
    if m == cap {
        return complete_bipartite(n1, n2, directed, mode);
    }

    let mut rng = SplitMix64::new(seed);
    let picks = distinct_sample(&mut rng, cap, m);
    let edges: Vec<(VertexId, VertexId)> = picks
        .into_iter()
        .map(|idx| decode_pair(idx, n1, n2, directed, mode))
        .collect();
    finalize(n1, n2, directed, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ------- types_vector / max_edges sanity -------

    #[test]
    fn types_vector_bottom_then_top() {
        let t = types_vector(3, 5);
        assert_eq!(t.len(), 8);
        for (i, &ty) in t.iter().enumerate() {
            if i < 3 {
                assert!(!ty, "vertex {i} should be bottom (false)");
            } else {
                assert!(ty, "vertex {i} should be top (true)");
            }
        }
    }

    #[test]
    fn max_edges_undirected_is_product() {
        assert_eq!(max_edges(3, 5, false, BipartiteMode::All), 15);
        assert_eq!(max_edges(0, 5, false, BipartiteMode::All), 0);
        assert_eq!(max_edges(3, 0, false, BipartiteMode::Out), 0);
    }

    #[test]
    fn max_edges_directed_out_or_in_is_product() {
        assert_eq!(max_edges(3, 5, true, BipartiteMode::Out), 15);
        assert_eq!(max_edges(3, 5, true, BipartiteMode::In), 15);
    }

    #[test]
    fn max_edges_directed_all_doubles() {
        assert_eq!(max_edges(3, 5, true, BipartiteMode::All), 30);
    }

    // ------- decode_pair sanity -------

    #[test]
    fn decode_undirected_covers_every_cross_pair_once() {
        let (n1, n2) = (3u32, 4u32);
        let cap = max_edges(n1, n2, false, BipartiteMode::All);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n1, n2, false, BipartiteMode::All);
            assert!(u < n1 && v >= n1 && v < n1 + n2);
            assert!(seen.insert((u, v)), "duplicate at idx {idx}: {u},{v}");
        }
        assert_eq!(seen.len(), (n1 * n2) as usize);
    }

    #[test]
    fn decode_directed_out_emits_bottom_to_top() {
        let (n1, n2) = (3u32, 4u32);
        let cap = max_edges(n1, n2, true, BipartiteMode::Out);
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n1, n2, true, BipartiteMode::Out);
            assert!(u < n1, "from must be bottom, got {u}");
            assert!(v >= n1 && v < n1 + n2, "to must be top, got {v}");
        }
    }

    #[test]
    fn decode_directed_in_emits_top_to_bottom() {
        let (n1, n2) = (3u32, 4u32);
        let cap = max_edges(n1, n2, true, BipartiteMode::In);
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n1, n2, true, BipartiteMode::In);
            assert!(u >= n1 && u < n1 + n2, "from must be top, got {u}");
            assert!(v < n1, "to must be bottom, got {v}");
        }
    }

    #[test]
    fn decode_directed_all_emits_each_ordered_pair_once() {
        let (n1, n2) = (3u32, 4u32);
        let cap = max_edges(n1, n2, true, BipartiteMode::All);
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for idx in 0..cap {
            let (u, v) = decode_pair(idx, n1, n2, true, BipartiteMode::All);
            // Crosses partition, in either direction.
            let u_bot = u < n1;
            let v_bot = v < n1;
            assert_ne!(u_bot, v_bot, "endpoints must cross partition");
            assert!(seen.insert((u, v)), "duplicate ordered pair at idx {idx}");
        }
        assert_eq!(seen.len(), 2 * (n1 * n2) as usize);
    }

    // ------- gnp boundary cases -------

    #[test]
    fn gnp_p_zero_is_empty() {
        let bg = bipartite_game_gnp(5, 6, 0.0, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.vcount(), 11);
        assert_eq!(bg.graph.ecount(), 0);
    }

    #[test]
    fn gnp_p_one_undirected_is_complete_bipartite() {
        let bg = bipartite_game_gnp(3, 4, 1.0, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.ecount(), 12);
    }

    #[test]
    fn gnp_p_one_directed_out_is_complete_bipartite() {
        let bg = bipartite_game_gnp(3, 4, 1.0, true, BipartiteMode::Out, 1).unwrap();
        assert_eq!(bg.graph.ecount(), 12);
        for eid in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(eid as u32).unwrap();
            assert!(u < 3, "Out arc must start in bottom");
            assert!(v >= 3, "Out arc must end in top");
        }
    }

    #[test]
    fn gnp_p_one_directed_all_doubles_edge_count() {
        let bg = bipartite_game_gnp(3, 4, 1.0, true, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.ecount(), 24);
    }

    #[test]
    fn gnp_invalid_p_rejected() {
        assert!(bipartite_game_gnp(3, 4, -0.1, false, BipartiteMode::All, 1).is_err());
        assert!(bipartite_game_gnp(3, 4, 1.1, false, BipartiteMode::All, 1).is_err());
        assert!(bipartite_game_gnp(3, 4, f64::NAN, false, BipartiteMode::All, 1).is_err());
        assert!(bipartite_game_gnp(3, 4, f64::INFINITY, false, BipartiteMode::All, 1).is_err());
    }

    #[test]
    fn gnp_zero_n1_is_edgeless_with_n2_vertices() {
        let bg = bipartite_game_gnp(0, 5, 0.5, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.vcount(), 5);
        assert_eq!(bg.graph.ecount(), 0);
        for &t in &bg.types {
            assert!(t, "all 5 must be top");
        }
    }

    #[test]
    fn gnp_zero_n2_is_edgeless_with_n1_vertices() {
        let bg = bipartite_game_gnp(7, 0, 0.5, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.vcount(), 7);
        assert_eq!(bg.graph.ecount(), 0);
        for &t in &bg.types {
            assert!(!t, "all 7 must be bottom");
        }
    }

    #[test]
    fn gnp_deterministic_with_seed() {
        let a = bipartite_game_gnp(10, 15, 0.4, false, BipartiteMode::All, 12345).unwrap();
        let b = bipartite_game_gnp(10, 15, 0.4, false, BipartiteMode::All, 12345).unwrap();
        assert_eq!(a.graph.vcount(), b.graph.vcount());
        assert_eq!(a.graph.ecount(), b.graph.ecount());
        let edges_a: Vec<_> = (0..a.graph.ecount())
            .map(|e| a.graph.edge(e as u32).unwrap())
            .collect();
        let edges_b: Vec<_> = (0..b.graph.ecount())
            .map(|e| b.graph.edge(e as u32).unwrap())
            .collect();
        assert_eq!(edges_a, edges_b);
    }

    #[test]
    fn gnp_different_seeds_typically_differ() {
        let a = bipartite_game_gnp(40, 60, 0.05, false, BipartiteMode::All, 1).unwrap();
        let b = bipartite_game_gnp(40, 60, 0.05, false, BipartiteMode::All, 2).unwrap();
        assert_ne!(a.graph.ecount(), b.graph.ecount());
    }

    #[test]
    fn gnp_expected_ecount_in_band() {
        // E[m] = p · n1·n2 = 0.2 · 30·30 = 180; stddev ≈ sqrt(np(1-p))
        // ≈ sqrt(180·0.8) ≈ 12; ±60 is ~5σ — generous, catches a
        // broken sampler.
        let bg = bipartite_game_gnp(30, 30, 0.2, false, BipartiteMode::All, 31_415).unwrap();
        let m = bg.graph.ecount();
        assert!(m > 120 && m < 240, "ecount = {m}");
    }

    #[test]
    fn gnp_is_simple_and_bipartite() {
        let bg = bipartite_game_gnp(20, 20, 0.3, false, BipartiteMode::All, 7).unwrap();
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            assert_ne!(u, v, "self-loop in bipartite output");
            // Must cross partition.
            assert_ne!(bg.types[u as usize], bg.types[v as usize]);
            let canon = if u <= v { (u, v) } else { (v, u) };
            assert!(seen.insert(canon), "parallel edge {canon:?}");
        }
    }

    #[test]
    fn gnp_directed_in_arcs_top_to_bottom() {
        let bg = bipartite_game_gnp(5, 6, 0.5, true, BipartiteMode::In, 999).unwrap();
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            assert!((5..11).contains(&u), "In source must be top, got {u}");
            assert!(v < 5, "In target must be bottom, got {v}");
        }
    }

    // ------- gnm boundary cases -------

    #[test]
    fn gnm_m_zero_is_empty() {
        let bg = bipartite_game_gnm(5, 6, 0, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.vcount(), 11);
        assert_eq!(bg.graph.ecount(), 0);
    }

    #[test]
    fn gnm_m_max_is_complete_bipartite() {
        let bg = bipartite_game_gnm(3, 4, 12, false, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.ecount(), 12);
    }

    #[test]
    fn gnm_m_max_directed_all_doubles() {
        let bg = bipartite_game_gnm(3, 4, 24, true, BipartiteMode::All, 1).unwrap();
        assert_eq!(bg.graph.ecount(), 24);
    }

    #[test]
    fn gnm_m_exceeds_capacity_rejected() {
        assert!(bipartite_game_gnm(5, 6, 31, false, BipartiteMode::All, 1).is_err());
    }

    #[test]
    fn gnm_zero_partition_with_m_positive_is_error() {
        assert!(bipartite_game_gnm(0, 5, 1, false, BipartiteMode::All, 1).is_err());
        assert!(bipartite_game_gnm(5, 0, 1, false, BipartiteMode::All, 1).is_err());
    }

    #[test]
    fn gnm_exact_edge_count() {
        let bg = bipartite_game_gnm(10, 20, 50, false, BipartiteMode::All, 42).unwrap();
        assert_eq!(bg.graph.ecount(), 50);
        assert_eq!(bg.graph.vcount(), 30);
    }

    #[test]
    fn gnm_deterministic_with_seed() {
        let a = bipartite_game_gnm(15, 20, 80, false, BipartiteMode::All, 7).unwrap();
        let b = bipartite_game_gnm(15, 20, 80, false, BipartiteMode::All, 7).unwrap();
        let edges_a: Vec<_> = (0..a.graph.ecount())
            .map(|e| a.graph.edge(e as u32).unwrap())
            .collect();
        let edges_b: Vec<_> = (0..b.graph.ecount())
            .map(|e| b.graph.edge(e as u32).unwrap())
            .collect();
        assert_eq!(edges_a, edges_b);
    }

    #[test]
    fn gnm_is_simple_and_bipartite() {
        let bg = bipartite_game_gnm(12, 10, 40, false, BipartiteMode::All, 99).unwrap();
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            assert_ne!(u, v);
            assert_ne!(bg.types[u as usize], bg.types[v as usize]);
            let canon = if u <= v { (u, v) } else { (v, u) };
            assert!(seen.insert(canon), "parallel edge {canon:?}");
        }
    }

    #[test]
    fn gnm_directed_out_arcs_bottom_to_top() {
        let bg = bipartite_game_gnm(4, 5, 15, true, BipartiteMode::Out, 33).unwrap();
        assert_eq!(bg.graph.ecount(), 15);
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            assert!(u < 4, "Out source must be bottom, got {u}");
            assert!((4..9).contains(&v), "Out target must be top, got {v}");
        }
    }

    #[test]
    fn gnm_directed_all_allows_mutual_pairs() {
        // m=24 = 2*3*4 forces every ordered cross-pair to appear, so the
        // canonical-pair (lo, hi) multiset has each unordered cross
        // pair exactly twice.
        let bg = bipartite_game_gnm(3, 4, 24, true, BipartiteMode::All, 1).unwrap();
        let mut canonical_counts: std::collections::HashMap<(u32, u32), u32> =
            std::collections::HashMap::new();
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            let key = if u <= v { (u, v) } else { (v, u) };
            *canonical_counts.entry(key).or_insert(0) += 1;
        }
        for (_, c) in canonical_counts {
            assert_eq!(
                c, 2,
                "every unordered cross pair appears twice in K_(3,4)·2"
            );
        }
    }

    #[test]
    fn gnp_full_directed_all_is_bipartite_and_has_2n1n2_edges() {
        // Sanity: p=1 with directed+All must include both
        // (bottom_i, top_j) and (top_j, bottom_i) — 2 · n1 · n2 arcs.
        let bg = bipartite_game_gnp(2, 3, 1.0, true, BipartiteMode::All, 0).unwrap();
        assert_eq!(bg.graph.ecount(), 12);
        let mut bottom_to_top = 0u32;
        let mut top_to_bottom = 0u32;
        for e in 0..bg.graph.ecount() {
            let (u, v) = bg.graph.edge(e as u32).unwrap();
            if u < 2 && v >= 2 {
                bottom_to_top += 1;
            } else if u >= 2 && v < 2 {
                top_to_bottom += 1;
            } else {
                panic!("non-bipartite arc {u}->{v}");
            }
        }
        assert_eq!(bottom_to_top, 6);
        assert_eq!(top_to_bottom, 6);
    }

    // ------- proptest harness (gated; runs under `cargo test --features proptest-harness`) -------

    #[cfg(feature = "proptest-harness")]
    mod prop {
        use super::super::*;
        use proptest::prelude::*;

        fn arb_mode() -> impl Strategy<Value = BipartiteMode> {
            prop_oneof![
                Just(BipartiteMode::Out),
                Just(BipartiteMode::In),
                Just(BipartiteMode::All),
            ]
        }

        proptest! {
            #[test]
            fn gnp_vcount_always_matches_sum(
                n1 in 0u32..15,
                n2 in 0u32..15,
                p in 0.0..=1.0,
                directed in any::<bool>(),
                mode in arb_mode(),
                seed in any::<u64>(),
            ) {
                let bg = bipartite_game_gnp(n1, n2, p, directed, mode, seed).unwrap();
                prop_assert_eq!(bg.graph.vcount(), n1 + n2);
                prop_assert_eq!(bg.types.len() as u32, n1 + n2);
                prop_assert!(bg.graph.ecount() as u64 <= max_edges(n1, n2, directed, mode));
            }

            #[test]
            fn gnp_simple_and_bipartite(
                n1 in 1u32..12,
                n2 in 1u32..12,
                p in 0.0..=0.6,
                directed in any::<bool>(),
                mode in arb_mode(),
                seed in any::<u64>(),
            ) {
                let bg = bipartite_game_gnp(n1, n2, p, directed, mode, seed).unwrap();
                // Distinct-edge multiset: for undirected use canonical
                // (lo, hi); for directed All allow mutual pairs so
                // dedup on ordered pair instead.
                let mut seen: std::collections::HashSet<(u32, u32)> =
                    std::collections::HashSet::new();
                for e in 0..bg.graph.ecount() {
                    let (u, v) = bg.graph.edge(e as u32).unwrap();
                    prop_assert!(u != v);
                    prop_assert!(bg.types[u as usize] != bg.types[v as usize]);
                    let key = if directed { (u, v) } else if u <= v { (u, v) } else { (v, u) };
                    prop_assert!(seen.insert(key));
                }
            }

            #[test]
            fn gnm_exact_count_and_bipartite(
                n1 in 1u32..10,
                n2 in 1u32..10,
                seed in any::<u64>(),
            ) {
                let cap = max_edges(n1, n2, false, BipartiteMode::All);
                if cap == 0 { return Ok(()); }
                let m = (seed as u64) % (cap + 1);
                let bg = bipartite_game_gnm(n1, n2, m, false, BipartiteMode::All, seed).unwrap();
                prop_assert_eq!(bg.graph.ecount() as u64, m);
                let mut seen: std::collections::HashSet<(u32, u32)> =
                    std::collections::HashSet::new();
                for e in 0..bg.graph.ecount() {
                    let (u, v) = bg.graph.edge(e as u32).unwrap();
                    prop_assert!(u != v);
                    prop_assert!(bg.types[u as usize] != bg.types[v as usize]);
                    let key = if u <= v { (u, v) } else { (v, u) };
                    prop_assert!(seen.insert(key));
                }
            }

            #[test]
            fn gnp_determinism(
                n1 in 1u32..10,
                n2 in 1u32..10,
                p in 0.05..0.95,
                directed in any::<bool>(),
                mode in arb_mode(),
                seed in any::<u64>(),
            ) {
                let a = bipartite_game_gnp(n1, n2, p, directed, mode, seed).unwrap();
                let b = bipartite_game_gnp(n1, n2, p, directed, mode, seed).unwrap();
                prop_assert_eq!(a.graph.ecount(), b.graph.ecount());
                for e in 0..a.graph.ecount() {
                    prop_assert_eq!(
                        a.graph.edge(e as u32).unwrap(),
                        b.graph.edge(e as u32).unwrap()
                    );
                }
            }
        }
    }
}
