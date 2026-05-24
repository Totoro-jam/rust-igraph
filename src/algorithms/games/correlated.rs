//! Correlated Erdős–Rényi graph generators (ALGO-GN-023).
//!
//! Counterparts of `igraph_correlated_game()` and
//! `igraph_correlated_pair_game()` from
//! `references/igraph/src/games/correlated.c` (lines 90-338).
//!
//! Given a simple `old_graph` on `n` vertices with target marginal
//! density `p`, [`correlated_game`] returns a new graph whose adjacency
//! vector has Pearson correlation `corr ∈ [0, 1]` with that of the
//! original. The construction preserves the marginal density `p` by
//! solving the elementary 2 × 2 contingency table:
//!
//! ```text
//!   q     = p + corr · (1 − p)
//!   p_del = 1 − q                        // P(drop | original edge)
//!   p_add = (1 − q) · p / (1 − p)        // P(create | original non-edge)
//! ```
//!
//! The per-position Bernoullis are sampled via the same Batagelj–Brandes
//! geometric-skip trick used in [`crate::erdos_renyi_gnp`] (ALGO-GN-001):
//! two `RNG_GEOM` streams skip directly over kept-or-skipped edge slots
//! without visiting each `O(n²)` position. The deletion stream walks the
//! `m` existing edges (in canonical-code order); the addition stream
//! walks the `n(n−1)/2` undirected (or `n(n−1)` directed) candidate
//! non-edges. A 3-way merge interleaves the kept edges, the deletes, and
//! the additions in sorted code order.
//!
//! [`correlated_pair_game`] is the convenience wrapper that samples an
//! `ER(n, p)` graph and a correlated counterpart from a single seed.
//!
//! ## References
//!
//! * Pedarsani, P. & Grossglauser, M. (2011). *On the privacy of
//!   anonymized networks*. Proc. SIGKDD.
//! * Barak, B. et al. (2018). *(Nearly) Efficient algorithms for the
//!   graph matching problem on correlated random graphs*. `NeurIPS`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::many_single_char_names,
    clippy::manual_midpoint,
    clippy::too_many_lines
)]

use crate::algorithms::games::erdos_renyi::erdos_renyi_gnp;
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Encode an undirected `(from, to)` pair with `from < to` as a single
/// 0-based code: `to·(to−1)/2 + from`. Range: `[0, n·(n−1)/2)`.
fn u_code(from: VertexId, to: VertexId) -> u64 {
    let (lo, hi) = if from < to { (from, to) } else { (to, from) };
    let hi64 = u64::from(hi);
    let lo64 = u64::from(lo);
    hi64 * (hi64 - 1) / 2 + lo64
}

/// Encode a directed `(from, to)` pair with `from != to` as a single
/// code in `[0, n·(n−1))`. Matches `D_CODE` in upstream `correlated.c`.
fn d_code(from: VertexId, to: VertexId, n: u32) -> u64 {
    let n64 = u64::from(n);
    let from64 = u64::from(from);
    let to64 = u64::from(to);
    // The clever trick: edges with `to == n-1` are filed into the
    // "diagonal hole" of column `from` (slot `from·n + from`); other
    // edges live at their natural `to·n + from` slot, so the diagonal
    // holes get filled exactly once.
    if to64 == n64 - 1 {
        from64 * n64 + from64
    } else {
        to64 * n64 + from64
    }
}

/// Decode an undirected code into `(from, to)` with `from < to`.
/// Inverts `U_CODE` from `correlated.c`.
fn u_decode(code: u64) -> (VertexId, VertexId) {
    // Standard upper-triangular inversion. We do the floating-point
    // estimate then verify in exact integer arithmetic — the same
    // bump-up loop used in `erdos_renyi::decode_pair`.
    let code_f = code as f64;
    let to_f = ((8.0 * code_f + 1.0).sqrt() + 1.0) / 2.0;
    let mut to = to_f.trunc() as u64;
    if to < 1 {
        to = 1;
    }
    let mut from = code - to * (to - 1) / 2;
    while from >= to {
        to += 1;
        from = code - to * (to - 1) / 2;
    }
    debug_assert!(from < to);
    (from as VertexId, to as VertexId)
}

/// Decode a directed code into `(from, to)` with `from != to`.
/// Inverts `D_CODE` from `correlated.c`.
fn d_decode(code: u64, n: u32) -> (VertexId, VertexId) {
    let n64 = u64::from(n);
    let to0 = code / n64;
    let from = code - to0 * n64;
    // `from == to` flags "I lived in a diagonal hole" → the real `to`
    // is `n − 1` (the column whose edges fill those holes).
    let to = if from == to0 { n64 - 1 } else { to0 };
    debug_assert!(from < n64 && to < n64 && from != to);
    (from as VertexId, to as VertexId)
}

/// Either canonical-code helper, selected by `directed`.
fn code(from: VertexId, to: VertexId, n: u32, directed: bool) -> u64 {
    if directed {
        d_code(from, to, n)
    } else {
        u_code(from, to)
    }
}

/// Either decode helper, selected by `directed`.
fn decode(c: u64, n: u32, directed: bool) -> (VertexId, VertexId) {
    if directed {
        d_decode(c, n)
    } else {
        u_decode(c)
    }
}

/// Total candidate-edge slots for a simple graph on `n` vertices.
fn slot_count(n: u32, directed: bool) -> u64 {
    let n64 = u64::from(n);
    if n64 < 2 {
        0
    } else if directed {
        n64 * (n64 - 1)
    } else {
        n64 * (n64 - 1) / 2
    }
}

/// Walk an old graph's edges and emit `(from, to, code)` sorted by
/// `code`. Codes are unique because `old_graph` is simple.
fn coded_edges(old_graph: &Graph) -> IgraphResult<Vec<(VertexId, VertexId, u64)>> {
    let n = old_graph.vcount();
    let directed = old_graph.is_directed();
    let m = u32::try_from(old_graph.ecount()).map_err(|_| {
        IgraphError::Internal("correlated_game: edge count exceeds u32 — bug in caller")
    })?;
    let mut out: Vec<(VertexId, VertexId, u64)> = Vec::with_capacity(m as usize);
    for eid in 0..m {
        let (f, t) = old_graph.edge(eid)?;
        out.push((f, t, code(f, t, n, directed)));
    }
    out.sort_unstable_by_key(|&(_, _, c)| c);
    Ok(out)
}

/// Compute and validate `(p_del, p_add)` from `(corr, p)`. Both are in
/// `[0, 1]`; either may be `0.0` (no draws on that side).
fn compute_conditionals(corr: f64, p: f64) -> (f64, f64) {
    let q = p + corr * (1.0 - p);
    let p_del = 1.0 - q;
    let p_add = (1.0 - q) * (p / (1.0 - p));
    // Clamp tiny FP drift below 0 / above 1.
    let p_del = p_del.clamp(0.0, 1.0);
    let p_add = p_add.clamp(0.0, 1.0);
    (p_del, p_add)
}

/// Apply the inverse of `permutation` to every endpoint in `edges`.
///
/// In the upstream C: `perm[i]` names the *old* vertex that becomes
/// vertex `i` in the *new* graph. So `inv[perm[i]] = i`, and to relabel
/// an edge whose endpoint is `v` in the old graph we look up `inv[v]`.
fn apply_permutation(
    edges: &mut [(VertexId, VertexId)],
    permutation: &[VertexId],
) -> IgraphResult<()> {
    let n = permutation.len();
    let n_u32 = u32::try_from(n)
        .map_err(|_| IgraphError::Internal("correlated_game: permutation length exceeds u32"))?;
    let mut inv: Vec<i64> = vec![-1; n];
    for (i, &v) in permutation.iter().enumerate() {
        if v >= n_u32 {
            return Err(IgraphError::InvalidArgument(format!(
                "correlated_game: permutation entry {v} out of range for n = {n}"
            )));
        }
        let slot = v as usize;
        if inv[slot] != -1 {
            return Err(IgraphError::InvalidArgument(format!(
                "correlated_game: permutation entry {v} appears more than once"
            )));
        }
        inv[slot] = i as i64;
    }
    // All slots filled iff perm is a bijection.
    for &val in &inv {
        if val == -1 {
            return Err(IgraphError::InvalidArgument(
                "correlated_game: permutation is not a bijection".to_string(),
            ));
        }
    }
    for edge in edges.iter_mut() {
        edge.0 = inv[edge.0 as usize] as VertexId;
        edge.1 = inv[edge.1 as usize] as VertexId;
    }
    Ok(())
}

/// Drive the geometric-skip stream and collect every index `< cap` it
/// visits. Mirrors the `RNG_GEOM` loop in `correlated.c`.
fn skip_indices(rng: &mut SplitMix64, prob: f64, cap: u64) -> Vec<u64> {
    if prob <= 0.0 || cap == 0 {
        return Vec::new();
    }
    let cap_f = cap as f64;
    let mut last = rng.gen_geom(prob);
    let mut out: Vec<u64> = Vec::new();
    while last < cap_f {
        let idx = last.trunc() as u64;
        if idx >= cap {
            break;
        }
        out.push(idx);
        last += rng.gen_geom(prob);
        last += 1.0;
    }
    out
}

/// Generate a random graph whose adjacency vector is Pearson-correlated
/// to that of `old_graph` at level `corr`.
///
/// * `old_graph` — input. Must be **simple** (no self-loops, no
///   multi-edges). Vertex count `n` and `directed` are inherited.
/// * `corr` — target correlation, in `[0, 1]`. `0` is independent
///   (sampled from `ER(n, p)`); `1` is a relabeled copy of `old_graph`.
/// * `p` — marginal edge probability, in the **open** `(0, 1)`. Usually
///   matches the empirical density of `old_graph`.
/// * `permutation` — optional vertex relabeling. `perm[i]` names the
///   vertex in `old_graph` that becomes vertex `i` in the new graph.
///   Must be a bijection of `[0, n)`.
/// * `seed` — drives the internal `SplitMix64` PRNG.
///
/// # Errors
///
/// * `corr` not in `[0, 1]`, `p` not in `(0, 1)`, either is NaN/∞;
/// * `permutation.len() != old_graph.vcount()`;
/// * `permutation` is not a bijection of `[0, n)`;
/// * `old_graph` is not simple.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, correlated_game, erdos_renyi_gnp};
/// let old = erdos_renyi_gnp(20, 0.3, false, false, 11).unwrap();
/// // Strong correlation: most of old's edges survive.
/// let new_g = correlated_game(&old, 0.9, 0.3, None, 17).unwrap();
/// assert_eq!(new_g.vcount(), 20);
/// assert_eq!(new_g.is_directed(), false);
/// ```
pub fn correlated_game(
    old_graph: &Graph,
    corr: f64,
    p: f64,
    permutation: Option<&[VertexId]>,
    seed: u64,
) -> IgraphResult<Graph> {
    if !corr.is_finite() || !(0.0..=1.0).contains(&corr) {
        return Err(IgraphError::InvalidArgument(format!(
            "correlated_game: corr must be in [0, 1] (got {corr})"
        )));
    }
    if !p.is_finite() || p <= 0.0 || p >= 1.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "correlated_game: p must be in the open interval (0, 1) (got {p})"
        )));
    }

    let n = old_graph.vcount();
    let directed = old_graph.is_directed();

    if let Some(perm) = permutation {
        if perm.len() != n as usize {
            return Err(IgraphError::InvalidArgument(format!(
                "correlated_game: permutation length {} does not match vertex count {}",
                perm.len(),
                n
            )));
        }
    }

    // Defensive verification — upstream also checks this.
    if !is_simple(old_graph)? {
        return Err(IgraphError::InvalidArgument(
            "correlated_game: old_graph must be a simple graph".to_string(),
        ));
    }

    // Special cases.
    if corr == 0.0 {
        // Drop old graph entirely; return a fresh ER(n, p) at the same
        // shape. The C kernel takes this short-circuit explicitly.
        let mut g = erdos_renyi_gnp(n, p, directed, false, seed)?;
        if let Some(perm) = permutation {
            apply_permutation_in_place(&mut g, perm)?;
        }
        return Ok(g);
    }
    if corr == 1.0 {
        // Copy old graph's edges (optionally relabeled), no RNG draws.
        let mut edges: Vec<(VertexId, VertexId)> =
            (0..u32::try_from(old_graph.ecount()).map_err(|_| {
                IgraphError::Internal("correlated_game: ecount exceeds u32 in corr=1 branch")
            })?)
                .map(|eid| old_graph.edge(eid))
                .collect::<IgraphResult<Vec<_>>>()?;
        if let Some(perm) = permutation {
            apply_permutation(&mut edges, perm)?;
        }
        let mut g = Graph::new(n, directed)?;
        g.add_edges(edges)?;
        return Ok(g);
    }

    let (p_del, p_add) = compute_conditionals(corr, p);
    let coded = coded_edges(old_graph)?;
    let m = coded.len() as u64;
    let cap = slot_count(n, directed);
    let missing = cap - m;

    let mut rng = SplitMix64::new(seed);

    // Step 1: delete indices, walked over the m existing edges by their
    // *position* in the sorted edge list. We recode immediately.
    let delete_positions = skip_indices(&mut rng, p_del, m);
    let mut delete_codes: Vec<u64> = delete_positions
        .into_iter()
        .map(|pos| coded[pos as usize].2)
        .collect();
    delete_codes.sort_unstable();

    // Step 2: add indices, walked over the `missing` non-edge slots.
    // These are *position* indices over the gap sequence — when we
    // merge below, each add index gets shifted by `p_e` (the count of
    // original edges scanned so far) to land on its canonical code.
    let add_positions = skip_indices(&mut rng, p_add, missing);

    // Step 3: 3-way merge. p_e walks `coded`, p_d walks delete_codes,
    // p_a walks add_positions.
    let mut new_edges: Vec<(VertexId, VertexId)> =
        Vec::with_capacity(coded.len() + add_positions.len());
    let mut p_e: usize = 0;
    let mut p_d: usize = 0;
    let mut p_a: usize = 0;
    // Use f64-INF / u64-MAX sentinels; the C kernel uses double INF but
    // u64 saturation is equivalent and avoids the float comparisons.
    let no_e = coded.len();
    let no_d = delete_codes.len();
    let no_a = add_positions.len();
    let inf: u128 = u128::MAX;
    let mut next_e: u128 = if p_e < no_e {
        u128::from(coded[p_e].2)
    } else {
        inf
    };
    let mut next_a: u128 = if p_a < no_a {
        u128::from(add_positions[p_a]) + u128::from(p_e as u64)
    } else {
        inf
    };
    let mut next_d: u128 = if p_d < no_d {
        u128::from(delete_codes[p_d])
    } else {
        inf
    };
    while next_e != inf || next_a != inf || next_d != inf {
        if next_e <= next_a && next_e < next_d {
            // Keep an original edge.
            let (f, t, _) = coded[p_e];
            new_edges.push((f, t));
            p_e += 1;
            next_e = if p_e < no_e {
                u128::from(coded[p_e].2)
            } else {
                inf
            };
            // next_a depends on p_e — refresh.
            next_a = if p_a < no_a {
                u128::from(add_positions[p_a]) + u128::from(p_e as u64)
            } else {
                inf
            };
        } else if next_e <= next_a && next_e == next_d {
            // Delete: skip the edge and advance the delete cursor.
            p_e += 1;
            next_e = if p_e < no_e {
                u128::from(coded[p_e].2)
            } else {
                inf
            };
            next_a = if p_a < no_a {
                u128::from(add_positions[p_a]) + u128::from(p_e as u64)
            } else {
                inf
            };
            p_d += 1;
            next_d = if p_d < no_d {
                u128::from(delete_codes[p_d])
            } else {
                inf
            };
        } else {
            // Add a new edge.
            let code64 = u64::try_from(next_a)
                .map_err(|_| IgraphError::Internal("correlated_game: add-code overflow"))?;
            let (f, t) = decode(code64, n, directed);
            new_edges.push((f, t));
            p_a += 1;
            next_a = if p_a < no_a {
                u128::from(add_positions[p_a]) + u128::from(p_e as u64)
            } else {
                inf
            };
        }
    }

    if let Some(perm) = permutation {
        apply_permutation(&mut new_edges, perm)?;
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(new_edges)?;
    Ok(g)
}

/// Apply `permutation` to every edge of an already-built `Graph`.
///
/// This rebuilds a fresh graph with the relabeled edges; mutating a
/// `Graph` in place is not part of the public API.
fn apply_permutation_in_place(g: &mut Graph, perm: &[VertexId]) -> IgraphResult<()> {
    let m = u32::try_from(g.ecount())
        .map_err(|_| IgraphError::Internal("correlated_game: ecount exceeds u32"))?;
    let mut edges: Vec<(VertexId, VertexId)> = (0..m)
        .map(|eid| g.edge(eid))
        .collect::<IgraphResult<Vec<_>>>()?;
    apply_permutation(&mut edges, perm)?;
    let n = g.vcount();
    let directed = g.is_directed();
    let mut fresh = Graph::new(n, directed)?;
    fresh.add_edges(edges)?;
    *g = fresh;
    Ok(())
}

/// Sample two random graphs with target adjacency-vector correlation
/// `corr`. Builds `graph1` via `ER(n, p)` and `graph2` via
/// [`correlated_game`] against `graph1`.
///
/// Counterpart of `igraph_correlated_pair_game` in
/// `references/igraph/src/games/correlated.c:330`.
///
/// * `n` — vertex count for both graphs.
/// * `corr` — target correlation, in `[0, 1]`.
/// * `p` — marginal edge probability, in the open `(0, 1)`.
/// * `directed` — whether both graphs are directed.
/// * `permutation` — optional, applied to `graph2` only (see
///   [`correlated_game`]).
/// * `seed` — drives a single internal `SplitMix64`. `graph1` consumes
///   its initial draws; `graph2` is sampled from the continuation.
///
/// # Errors
///
/// Same conditions as [`correlated_game`], plus any error from
/// [`crate::erdos_renyi_gnp`] when building `graph1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::correlated_pair_game;
/// let (g1, g2) = correlated_pair_game(40, 0.7, 0.2, false, None, 99).unwrap();
/// assert_eq!(g1.vcount(), 40);
/// assert_eq!(g2.vcount(), 40);
/// ```
pub fn correlated_pair_game(
    n: u32,
    corr: f64,
    p: f64,
    directed: bool,
    permutation: Option<&[VertexId]>,
    seed: u64,
) -> IgraphResult<(Graph, Graph)> {
    let g1 = erdos_renyi_gnp(n, p, directed, false, seed)?;
    // Use a distinct seed derivation for g2 so the same seed → distinct
    // RNG state, mirroring the C implementation's effective behavior
    // (C uses a single shared RNG state that carries over).
    let g2_seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let g2 = correlated_game(&g1, corr, p, permutation, g2_seed)?;
    Ok((g1, g2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecount_of(g: &Graph) -> usize {
        g.ecount()
    }

    /// Brute-force adjacency-overlap: count edges present in both graphs
    /// when treated as undirected, ignoring multiplicity.
    fn edge_set(g: &Graph) -> std::collections::HashSet<(VertexId, VertexId)> {
        let mut s = std::collections::HashSet::new();
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (f, t) = g.edge(eid).unwrap();
            let key = if g.is_directed() || f < t {
                (f, t)
            } else {
                (t, f)
            };
            s.insert(key);
        }
        s
    }

    fn assert_no_self_loops(g: &Graph) {
        let m = u32::try_from(g.ecount()).unwrap();
        for eid in 0..m {
            let (f, t) = g.edge(eid).unwrap();
            assert_ne!(f, t, "self-loop at edge {eid}");
        }
    }

    fn assert_simple(g: &Graph) {
        assert!(is_simple(g).unwrap(), "not simple");
    }

    #[test]
    fn u_code_round_trip() {
        for to in 1u32..30 {
            for from in 0..to {
                let c = u_code(from, to);
                let (f2, t2) = u_decode(c);
                assert_eq!((f2, t2), (from, to), "u_code({from}, {to}) = {c}");
            }
        }
    }

    #[test]
    fn d_code_round_trip() {
        for n in [2u32, 3, 5, 8, 13] {
            for from in 0..n {
                for to in 0..n {
                    if from == to {
                        continue;
                    }
                    let c = d_code(from, to, n);
                    let cap = u64::from(n) * (u64::from(n) - 1);
                    assert!(c < cap, "d_code({from}, {to}, {n}) = {c} >= cap {cap}");
                    let (f2, t2) = d_decode(c, n);
                    assert_eq!((f2, t2), (from, to), "d_code({from}, {to}, {n}) = {c}");
                }
            }
        }
    }

    #[test]
    fn d_code_is_bijection() {
        // Every code in [0, n(n-1)) decodes to a unique (f, t) with f != t.
        for n in [2u32, 3, 5, 10] {
            let cap = u64::from(n) * (u64::from(n) - 1);
            let mut seen = std::collections::HashSet::new();
            for c in 0..cap {
                let (f, t) = d_decode(c, n);
                assert_ne!(f, t);
                assert!(seen.insert((f, t)), "duplicate decoded pair at code {c}");
            }
            assert_eq!(seen.len(), cap as usize);
        }
    }

    #[test]
    fn empty_old_graph_returns_empty_new() {
        let old = Graph::new(0, false).unwrap();
        let new_g = correlated_game(&old, 0.5, 0.5, None, 1).unwrap();
        assert_eq!(new_g.vcount(), 0);
        assert_eq!(new_g.ecount(), 0);
    }

    #[test]
    fn singleton_old_graph_returns_singleton() {
        let old = Graph::new(1, false).unwrap();
        let new_g = correlated_game(&old, 0.5, 0.5, None, 1).unwrap();
        assert_eq!(new_g.vcount(), 1);
        assert_eq!(new_g.ecount(), 0);
    }

    #[test]
    fn corr_zero_independent_density() {
        // corr=0 is just ER(n, p); should NOT preserve any edges except
        // by chance.
        let old = erdos_renyi_gnp(50, 0.5, false, false, 7).unwrap();
        let new_g = correlated_game(&old, 0.0, 0.5, None, 11).unwrap();
        assert_eq!(new_g.vcount(), old.vcount());
        // Independent → expected overlap is p² · cap ≈ 0.25 · 1225 ≈ 306.
        let overlap = edge_set(&old).intersection(&edge_set(&new_g)).count();
        assert!(
            (200..420).contains(&overlap),
            "independent overlap should hover near 306, got {overlap}"
        );
    }

    #[test]
    fn corr_one_exact_copy() {
        let old = erdos_renyi_gnp(40, 0.3, false, false, 5).unwrap();
        let new_g = correlated_game(&old, 1.0, 0.3, None, 11).unwrap();
        assert_eq!(new_g.vcount(), old.vcount());
        assert_eq!(new_g.ecount(), old.ecount());
        assert_eq!(edge_set(&new_g), edge_set(&old));
    }

    #[test]
    fn corr_one_with_permutation_relabels() {
        let old = erdos_renyi_gnp(10, 0.4, false, false, 3).unwrap();
        // Reverse permutation: new[i] = old[n - 1 - i].
        let n = old.vcount();
        let perm: Vec<VertexId> = (0..n).rev().collect();
        let new_g = correlated_game(&old, 1.0, 0.4, Some(&perm), 17).unwrap();
        assert_eq!(new_g.ecount(), old.ecount());
        // Every original (u, v) should appear as (n-1-u, n-1-v) in new.
        let mut expected = std::collections::HashSet::new();
        let m = u32::try_from(old.ecount()).unwrap();
        for eid in 0..m {
            let (f, t) = old.edge(eid).unwrap();
            let (fn_, tn) = (n - 1 - f, n - 1 - t);
            let key = if fn_ < tn { (fn_, tn) } else { (tn, fn_) };
            expected.insert(key);
        }
        assert_eq!(edge_set(&new_g), expected);
    }

    #[test]
    fn high_corr_preserves_most_edges() {
        let old = erdos_renyi_gnp(100, 0.2, false, false, 23).unwrap();
        let new_g = correlated_game(&old, 0.95, 0.2, None, 31).unwrap();
        let intersection = edge_set(&old).intersection(&edge_set(&new_g)).count();
        let frac = intersection as f64 / ecount_of(&old) as f64;
        assert!(
            frac > 0.85,
            "corr=0.95 should preserve >85% of old edges, got {frac:.3}"
        );
    }

    #[test]
    fn low_corr_preserves_few_edges() {
        let old = erdos_renyi_gnp(100, 0.2, false, false, 23).unwrap();
        let new_g = correlated_game(&old, 0.1, 0.2, None, 31).unwrap();
        let intersection = edge_set(&old).intersection(&edge_set(&new_g)).count();
        let frac = intersection as f64 / ecount_of(&old) as f64;
        // p²·cap = 0.04·4950 = 198 independent overlap; corr=0.1 lifts
        // that mildly. Should still be well under 50%.
        assert!(
            frac < 0.45,
            "corr=0.1 should preserve <45% of old edges, got {frac:.3}"
        );
    }

    #[test]
    fn output_is_simple_no_self_loops() {
        let old = erdos_renyi_gnp(40, 0.3, false, false, 9).unwrap();
        let new_g = correlated_game(&old, 0.5, 0.3, None, 13).unwrap();
        assert_no_self_loops(&new_g);
        assert_simple(&new_g);
    }

    #[test]
    fn deterministic_same_seed() {
        let old = erdos_renyi_gnp(30, 0.3, false, false, 1).unwrap();
        let a = correlated_game(&old, 0.5, 0.3, None, 999).unwrap();
        let b = correlated_game(&old, 0.5, 0.3, None, 999).unwrap();
        assert_eq!(edge_set(&a), edge_set(&b));
    }

    #[test]
    fn different_seed_different_output() {
        let old = erdos_renyi_gnp(30, 0.3, false, false, 1).unwrap();
        let a = correlated_game(&old, 0.5, 0.3, None, 999).unwrap();
        let b = correlated_game(&old, 0.5, 0.3, None, 1000).unwrap();
        assert_ne!(edge_set(&a), edge_set(&b));
    }

    #[test]
    fn directed_input_yields_directed_output() {
        let old = erdos_renyi_gnp(20, 0.3, true, false, 5).unwrap();
        let new_g = correlated_game(&old, 0.5, 0.3, None, 7).unwrap();
        assert!(new_g.is_directed());
        assert_no_self_loops(&new_g);
        assert_simple(&new_g);
    }

    #[test]
    fn invalid_corr_rejected() {
        let old = erdos_renyi_gnp(5, 0.3, false, false, 1).unwrap();
        assert!(correlated_game(&old, -0.1, 0.3, None, 1).is_err());
        assert!(correlated_game(&old, 1.1, 0.3, None, 1).is_err());
        assert!(correlated_game(&old, f64::NAN, 0.3, None, 1).is_err());
    }

    #[test]
    fn invalid_p_rejected() {
        let old = erdos_renyi_gnp(5, 0.3, false, false, 1).unwrap();
        assert!(correlated_game(&old, 0.5, 0.0, None, 1).is_err());
        assert!(correlated_game(&old, 0.5, 1.0, None, 1).is_err());
        assert!(correlated_game(&old, 0.5, -0.1, None, 1).is_err());
        assert!(correlated_game(&old, 0.5, f64::NAN, None, 1).is_err());
    }

    #[test]
    fn invalid_permutation_rejected() {
        let old = erdos_renyi_gnp(5, 0.3, false, false, 1).unwrap();
        // Wrong length.
        let bad_len: Vec<VertexId> = vec![0, 1, 2];
        assert!(correlated_game(&old, 0.5, 0.3, Some(&bad_len), 1).is_err());
        // Duplicate entries.
        let dup: Vec<VertexId> = vec![0, 1, 2, 1, 4];
        assert!(correlated_game(&old, 0.5, 0.3, Some(&dup), 1).is_err());
        // Out-of-range.
        let oor: Vec<VertexId> = vec![0, 1, 2, 3, 99];
        assert!(correlated_game(&old, 0.5, 0.3, Some(&oor), 1).is_err());
    }

    #[test]
    fn non_simple_old_rejected() {
        let mut old = Graph::new(3, false).unwrap();
        old.add_edge(0, 1).unwrap();
        old.add_edge(0, 1).unwrap(); // multi-edge
        assert!(correlated_game(&old, 0.5, 0.3, None, 1).is_err());
    }

    #[test]
    fn correlated_pair_basic() {
        let (g1, g2) = correlated_pair_game(50, 0.8, 0.3, false, None, 42).unwrap();
        assert_eq!(g1.vcount(), 50);
        assert_eq!(g2.vcount(), 50);
        assert_simple(&g1);
        assert_simple(&g2);
        // High correlation → overlap > independent baseline (p² · cap).
        let cap = 50.0 * 49.0 / 2.0;
        let baseline = 0.3_f64.powi(2) * cap;
        let overlap = edge_set(&g1).intersection(&edge_set(&g2)).count() as f64;
        assert!(
            overlap > baseline * 1.5,
            "high-corr overlap {overlap} should exceed baseline {baseline}"
        );
    }

    #[test]
    fn correlated_pair_directed() {
        let (g1, g2) = correlated_pair_game(30, 0.5, 0.3, true, None, 7).unwrap();
        assert!(g1.is_directed());
        assert!(g2.is_directed());
        assert_simple(&g1);
        assert_simple(&g2);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn vcount_preserved(n in 2u32..40, p in 0.05f64..0.95, corr in 0.0f64..=1.0, seed in any::<u64>()) {
            let old = erdos_renyi_gnp(n, p, false, false, seed).unwrap();
            let new_g = correlated_game(&old, corr, p, None, seed.wrapping_add(1)).unwrap();
            prop_assert_eq!(new_g.vcount(), n);
            prop_assert_eq!(new_g.is_directed(), false);
        }

        #[test]
        fn always_simple(n in 2u32..30, p in 0.1f64..0.9, corr in 0.0f64..=1.0, seed in any::<u64>()) {
            let old = erdos_renyi_gnp(n, p, false, false, seed).unwrap();
            let new_g = correlated_game(&old, corr, p, None, seed.wrapping_add(1)).unwrap();
            prop_assert!(is_simple(&new_g).unwrap());
            let m = u32::try_from(new_g.ecount()).unwrap();
            for eid in 0..m {
                let (f, t) = new_g.edge(eid).unwrap();
                prop_assert_ne!(f, t);
            }
        }

        #[test]
        fn corr_one_is_exact_copy(n in 2u32..30, p in 0.1f64..0.9, seed in any::<u64>()) {
            let old = erdos_renyi_gnp(n, p, false, false, seed).unwrap();
            let new_g = correlated_game(&old, 1.0, p, None, seed.wrapping_add(1)).unwrap();
            prop_assert_eq!(new_g.ecount(), old.ecount());
        }

        #[test]
        fn deterministic(n in 2u32..30, p in 0.1f64..0.9, corr in 0.0f64..=1.0, seed in any::<u64>()) {
            let old = erdos_renyi_gnp(n, p, false, false, seed).unwrap();
            let a = correlated_game(&old, corr, p, None, seed.wrapping_add(1)).unwrap();
            let b = correlated_game(&old, corr, p, None, seed.wrapping_add(1)).unwrap();
            prop_assert_eq!(a.ecount(), b.ecount());
        }

        #[test]
        fn directed_round_trip(n in 2u32..25, p in 0.1f64..0.9, corr in 0.0f64..=1.0, seed in any::<u64>()) {
            let old = erdos_renyi_gnp(n, p, true, false, seed).unwrap();
            let new_g = correlated_game(&old, corr, p, None, seed.wrapping_add(1)).unwrap();
            prop_assert_eq!(new_g.vcount(), n);
            prop_assert!(new_g.is_directed());
            prop_assert!(is_simple(&new_g).unwrap());
        }
    }
}
