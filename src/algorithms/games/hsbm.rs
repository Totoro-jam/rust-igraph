//! Hierarchical Stochastic Block Model (ALGO-GN-011).
//!
//! Counterparts of `igraph_hsbm_game()` and `igraph_hsbm_list_game()`
//! from `references/igraph/src/games/sbm.c:284-648`. Both arrange `n`
//! vertices into `K` *macro-blocks*. Each macro-block runs its own
//! internal Stochastic Block Model over `k_b` *micro-blocks*; in
//! addition every pair of macro-blocks is connected by a Bernoulli(p)
//! sampler on the rectangular product of their vertex sets.
//!
//! * [`hsbm_game`] — the **uniform** variant: every macro-block has the
//!   same size `m`, the same micro-block proportions `rho`, and the
//!   same internal preference matrix `c`. Requires `n % m == 0` so the
//!   number of macro-blocks is exactly `n / m`.
//! * [`hsbm_list_game`] — the **per-macro** variant: each macro-block
//!   `b` carries its own size `m_list[b]`, proportions `rho_list[b]`,
//!   and preference matrix `c_list[b]`. The sizes must sum to `n`.
//!
//! Both functions always produce an **undirected, simple** graph
//! (no self-loops, no multi-edges) — matching the upstream C
//! signatures, which fix `directed=0` and do not expose a `loops`
//! or `multiple` flag.
//!
//! ## Algorithm
//!
//! Each macro-block is sampled independently via the same
//! Batagelj–Brandes geometric-skip path used by [`super::sbm`], reused
//! through [`crate::algorithms::games::sbm`] internals. The inter-
//! macro layer adds, for every pair of macros, a single rectangular
//! geometric-skip pass at rate `p`. Two fast paths:
//!
//! * `p == 1` emits the full bipartite edge set without consulting the
//!   RNG.
//! * `p == 0` is skipped entirely.
//!
//! Total cost is `O(n + m + sum k_b² + K²)` where `m` is the realised
//! edge count and `K` is the number of macro-blocks.
//!
//! ## Determinism
//!
//! Given the same `(args, seed)` the output is bit-exact reproducible
//! via the shared [`crate::core::rng::SplitMix64`] PRNG.
//!
//! ## References
//!
//! * V. Batagelj and U. Brandes, *"Efficient generation of large
//!   random networks"*, Phys. Rev. E **71**, 036113 (2005).
//! * Hierarchical-SBM literature, e.g. Lyzinski et al., *"Community
//!   detection and classification in hierarchical stochastic
//!   blockmodels"*, IEEE Trans. Net. Sci. Eng. **4** (2017), 13-26.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::manual_midpoint,
    clippy::many_single_char_names
)]

use crate::algorithms::games::sbm::{PairShape, block_offsets, sample_pair_with_max};
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Tolerance for "sum equals one" and "rho·m is integer" checks. The
/// upstream C uses `sqrt(DBL_EPSILON)` which is ~1.49e-8.
const SUM_TOL: f64 = 1.490_116_119_384_765_6e-8;

// ----- validation -----------------------------------------------------

/// Verify a single Bernoulli rate matrix is square `k×k`, symmetric,
/// and lies in `[0, 1]`.
fn validate_pref(c: &[Vec<f64>], k: usize, label: &str) -> IgraphResult<()> {
    if c.len() != k {
        return Err(IgraphError::InvalidArgument(format!(
            "{label}: pref matrix has {} rows but rho has length {k}",
            c.len()
        )));
    }
    for (i, row) in c.iter().enumerate() {
        if row.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}: pref matrix row {i} has length {} (expected {k})",
                row.len()
            )));
        }
        for (j, &val) in row.iter().enumerate() {
            if !val.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "{label}: pref[{i}][{j}] = {val} is not finite"
                )));
            }
            if !(0.0..=1.0).contains(&val) {
                return Err(IgraphError::InvalidArgument(format!(
                    "{label}: pref[{i}][{j}] = {val} must lie in [0, 1]"
                )));
            }
        }
    }
    for (i, row_i) in c.iter().enumerate() {
        for (j, row_j) in c.iter().enumerate().skip(i + 1) {
            let pij = row_i[j];
            let pji = row_j[i];
            if pij != pji {
                return Err(IgraphError::InvalidArgument(format!(
                    "{label}: pref matrix is not symmetric: pref[{i}][{j}] = {pij}, \
                     pref[{j}][{i}] = {pji}"
                )));
            }
        }
    }
    Ok(())
}

/// Verify `rho ⊂ [0, 1]` and `sum(rho) == 1` within `SUM_TOL`.
fn validate_rho(rho: &[f64], label: &str) -> IgraphResult<()> {
    if rho.is_empty() {
        return Err(IgraphError::InvalidArgument(format!(
            "{label}: rho must contain at least one entry"
        )));
    }
    let mut acc = 0.0;
    for (i, &val) in rho.iter().enumerate() {
        if !val.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}: rho[{i}] = {val} is not finite"
            )));
        }
        if !(0.0..=1.0).contains(&val) {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}: rho[{i}] = {val} must lie in [0, 1]"
            )));
        }
        acc += val;
    }
    if (acc - 1.0).abs() > SUM_TOL {
        return Err(IgraphError::InvalidArgument(format!(
            "{label}: sum(rho) = {acc} must equal 1 (tolerance {SUM_TOL})"
        )));
    }
    Ok(())
}

/// Verify that every `rho[i] * m` is (close to) an integer, and return
/// the resulting csizes vector summing to `m`. Distribute any
/// rounding residue onto the last positive-rho entry so the sum is
/// exactly `m`, mirroring how the upstream tolerance check would
/// accept the inputs but the final sum needs to be exact.
fn csizes_for(rho: &[f64], m: u32, label: &str) -> IgraphResult<Vec<u32>> {
    let m_f = f64::from(m);
    let mut sizes = Vec::with_capacity(rho.len());
    for (i, &r) in rho.iter().enumerate() {
        let s = r * m_f;
        let rounded = s.round();
        if (rounded - s).abs() > SUM_TOL {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}: rho[{i}] * m = {s} is not an integer (tolerance {SUM_TOL})"
            )));
        }
        if rounded < 0.0 || rounded > f64::from(u32::MAX) {
            return Err(IgraphError::InvalidArgument(format!(
                "{label}: rho[{i}] * m = {rounded} is out of u32 range"
            )));
        }
        sizes.push(rounded as u32);
    }
    let sum: u64 = sizes.iter().copied().map(u64::from).sum();
    if sum != u64::from(m) {
        return Err(IgraphError::InvalidArgument(format!(
            "{label}: sum of round(rho[i] * m) = {sum} but m = {m}"
        )));
    }
    Ok(sizes)
}

// ----- intra-macro sampling -------------------------------------------

/// Run a single macro-block's intra-SBM, appending edges to `edges`
/// with vertex ids offset by `macro_off`.
fn sample_intra_macro(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    macro_off: u32,
    csizes: &[u32],
    c: &[Vec<f64>],
) {
    let k = csizes.len();
    // Cumulative offsets within this macro-block.
    let mut intra_off = vec![0u32; k + 1];
    let mut acc = 0u32;
    for (i, &s) in csizes.iter().enumerate() {
        acc += s;
        intra_off[i + 1] = acc;
    }
    for from in 0..k {
        let fromsize = csizes[from];
        if fromsize == 0 {
            continue;
        }
        let fromoff = macro_off + intra_off[from];
        for to in from..k {
            let tosize = csizes[to];
            if tosize == 0 {
                continue;
            }
            let tooff = macro_off + intra_off[to];
            let prob = c[from][to];
            if prob <= 0.0 {
                continue;
            }
            let (shape, maxedges) = if from == to {
                let fs = u64::from(fromsize);
                (PairShape::TriExclDiag, fs * fs.saturating_sub(1) / 2)
            } else {
                (PairShape::Rect, u64::from(fromsize) * u64::from(tosize))
            };
            sample_pair_with_max(
                rng, edges, fromsize, fromoff, tooff, shape, false, prob, maxedges,
            );
        }
    }
}

// ----- inter-macro sampling -------------------------------------------

/// Emit every edge between two disjoint macro-block vertex ranges
/// (full bipartite). Used only when `p == 1`.
fn full_bipartite(
    edges: &mut Vec<(VertexId, VertexId)>,
    fromoff: u32,
    fromsize: u32,
    tooff: u32,
    tosize: u32,
) {
    for u in 0..fromsize {
        for v in 0..tosize {
            edges.push((fromoff + u, tooff + v));
        }
    }
}

/// Rectangular geometric-skip pass between two disjoint macro-block
/// vertex ranges at rate `p`. Caller must ensure `0 < p < 1`.
fn rect_between_macros(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    fromoff: u32,
    fromsize: u32,
    tooff: u32,
    tosize: u32,
    p: f64,
) {
    if fromsize == 0 || tosize == 0 || p <= 0.0 {
        return;
    }
    let maxedges = u64::from(fromsize) * u64::from(tosize);
    sample_pair_with_max(
        rng,
        edges,
        fromsize,
        fromoff,
        tooff,
        PairShape::Rect,
        false,
        p,
        maxedges,
    );
}

/// Add inter-macro edges for every pair `(b1, b2)` with `b1 < b2`.
fn add_inter_macro(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    macro_offsets: &[u32],
    macro_sizes: &[u32],
    p: f64,
) {
    if p <= 0.0 {
        return;
    }
    let k = macro_sizes.len();
    if p >= 1.0 {
        for b1 in 0..k {
            let fromoff = macro_offsets[b1];
            let fromsize = macro_sizes[b1];
            if fromsize == 0 {
                continue;
            }
            for b2 in (b1 + 1)..k {
                let tosize = macro_sizes[b2];
                if tosize == 0 {
                    continue;
                }
                full_bipartite(edges, fromoff, fromsize, macro_offsets[b2], tosize);
            }
        }
        return;
    }
    for b1 in 0..k {
        let fromoff = macro_offsets[b1];
        let fromsize = macro_sizes[b1];
        if fromsize == 0 {
            continue;
        }
        for b2 in (b1 + 1)..k {
            let tosize = macro_sizes[b2];
            if tosize == 0 {
                continue;
            }
            rect_between_macros(rng, edges, fromoff, fromsize, macro_offsets[b2], tosize, p);
        }
    }
}

// ----- public API -----------------------------------------------------

/// Generate a graph from the **uniform Hierarchical Stochastic Block
/// Model**: every macro-block is the same size `m` and shares the same
/// internal preference matrix.
///
/// * `n` — total vertex count. Must satisfy `n >= 1` and `n % m == 0`.
/// * `m` — micro-vertex count per macro-block. Must satisfy `m >= 1`.
///   The number of macro-blocks is `n / m`.
/// * `rho` — micro-block proportions inside each macro-block. Length
///   `k`, entries in `[0, 1]`, sum equal to `1` within
///   `√DBL_EPSILON`. Each `rho[j] * m` must also be (within tolerance)
///   an integer.
/// * `c` — `k × k` symmetric Bernoulli pref matrix on the micro-blocks,
///   entries in `[0, 1]`.
/// * `p` — Bernoulli rate for every edge crossing two distinct
///   macro-blocks. Must lie in `[0, 1]`.
/// * `seed` — initialises an internal `SplitMix64` PRNG.
///
/// The returned graph is always undirected, simple (no loops, no
/// multi-edges). Vertex ordering follows macro-block then micro-block:
/// macro `b` occupies the range `[b*m, (b+1)*m)`, and inside it
/// micro-block `j` occupies `[b*m + offset_j, b*m + offset_{j+1})`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when any of the constraints
/// above is violated.
///
/// # Examples
///
/// ```
/// use rust_igraph::hsbm_game;
///
/// // 2 macro-blocks of size 10, each with 2 micro-blocks of 5,
/// // strong within-cluster ties and a 1 % between-macro rate.
/// let rho = vec![0.5, 0.5];
/// let c = vec![vec![0.8, 0.05], vec![0.05, 0.8]];
/// let g = hsbm_game(20, 10, &rho, &c, 0.01, 42).unwrap();
/// assert_eq!(g.vcount(), 20);
/// assert!(!g.is_directed());
/// ```
pub fn hsbm_game(
    n: u32,
    m: u32,
    rho: &[f64],
    c: &[Vec<f64>],
    p: f64,
    seed: u64,
) -> IgraphResult<Graph> {
    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "hsbm_game: n must be at least 1".into(),
        ));
    }
    if m == 0 {
        return Err(IgraphError::InvalidArgument(
            "hsbm_game: m must be at least 1".into(),
        ));
    }
    if n % m != 0 {
        return Err(IgraphError::InvalidArgument(format!(
            "hsbm_game: n ({n}) must be a multiple of m ({m})"
        )));
    }
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(IgraphError::InvalidArgument(format!(
            "hsbm_game: p = {p} must lie in [0, 1]"
        )));
    }
    let k = rho.len();
    validate_rho(rho, "hsbm_game")?;
    validate_pref(c, k, "hsbm_game")?;
    let csizes = csizes_for(rho, m, "hsbm_game")?;
    let no_macros = n / m;

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    // Phase 1: intra-macro.
    for b in 0..no_macros {
        let macro_off = b * m;
        sample_intra_macro(&mut rng, &mut edges, macro_off, &csizes, c);
    }

    // Phase 2: inter-macro.
    let mut macro_offsets = Vec::with_capacity(no_macros as usize + 1);
    let mut macro_sizes = Vec::with_capacity(no_macros as usize);
    for b in 0..no_macros {
        macro_offsets.push(b * m);
        macro_sizes.push(m);
    }
    macro_offsets.push(no_macros * m);
    add_inter_macro(&mut rng, &mut edges, &macro_offsets, &macro_sizes, p);

    let mut g = Graph::new(n, false)?;
    g.add_edges(edges)?;
    Ok(g)
}

/// Generate a graph from the **per-macro Hierarchical Stochastic Block
/// Model**: every macro-block carries its own size, its own
/// micro-block proportions, and its own internal preference matrix.
///
/// * `n` — total vertex count. Must equal `m_list.iter().sum()`.
/// * `m_list` — length-`K` vector of macro-block sizes. Each entry
///   must be at least 1.
/// * `rho_list` — length-`K` list; entry `b` is the micro-block
///   proportion vector for macro-block `b`. Each `rho_list[b]` must
///   sum to 1 within `√DBL_EPSILON`, and every `rho_list[b][j] *
///   m_list[b]` must be (within tolerance) an integer.
/// * `c_list` — length-`K` list; entry `b` is the
///   `k_b × k_b` symmetric Bernoulli pref matrix for macro-block `b`.
///   Entries in `[0, 1]`.
/// * `p` — Bernoulli rate for every edge crossing two distinct
///   macro-blocks. Must lie in `[0, 1]`.
/// * `seed` — initialises an internal `SplitMix64` PRNG.
///
/// The returned graph is always undirected, simple. Vertex ordering
/// follows the macro-block order: macro `b` occupies
/// `[sum_{i<b} m_list[i], sum_{i<=b} m_list[i])`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when any of the constraints
/// above is violated, including length mismatches between `m_list`,
/// `rho_list`, and `c_list`.
///
/// # Examples
///
/// ```
/// use rust_igraph::hsbm_list_game;
///
/// // 3 macro-blocks of size 4, 6, 10. Each runs its own inner SBM.
/// let m_list = vec![4u32, 6, 10];
/// let rho_list = vec![
///     vec![1.0],          // single cluster
///     vec![0.5, 0.5],     // two equal clusters of 3
///     vec![0.5, 0.5],     // two equal clusters of 5
/// ];
/// let c_list = vec![
///     vec![vec![0.5]],
///     vec![vec![0.4, 0.1], vec![0.1, 0.4]],
///     vec![vec![0.3, 0.05], vec![0.05, 0.3]],
/// ];
/// let g = hsbm_list_game(20, &m_list, &rho_list, &c_list, 0.02, 7).unwrap();
/// assert_eq!(g.vcount(), 20);
/// ```
pub fn hsbm_list_game(
    n: u32,
    m_list: &[u32],
    rho_list: &[Vec<f64>],
    c_list: &[Vec<Vec<f64>>],
    p: f64,
    seed: u64,
) -> IgraphResult<Graph> {
    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "hsbm_list_game: n must be at least 1".into(),
        ));
    }
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(IgraphError::InvalidArgument(format!(
            "hsbm_list_game: p = {p} must lie in [0, 1]"
        )));
    }
    let no_macros = rho_list.len();
    if no_macros == 0 {
        return Err(IgraphError::InvalidArgument(
            "hsbm_list_game: rho_list must contain at least one entry".into(),
        ));
    }
    if m_list.len() != no_macros || c_list.len() != no_macros {
        return Err(IgraphError::InvalidArgument(format!(
            "hsbm_list_game: rho_list ({no_macros}), m_list ({}) and c_list ({}) \
             must have the same length",
            m_list.len(),
            c_list.len()
        )));
    }
    if m_list.contains(&0) {
        return Err(IgraphError::InvalidArgument(
            "hsbm_list_game: every m_list[b] must be at least 1".into(),
        ));
    }
    let sum_m: u64 = m_list.iter().copied().map(u64::from).sum();
    if sum_m != u64::from(n) {
        return Err(IgraphError::InvalidArgument(format!(
            "hsbm_list_game: sum(m_list) = {sum_m} but n = {n}"
        )));
    }
    let (macro_offsets, _total) = block_offsets(m_list)?;

    let mut all_csizes: Vec<Vec<u32>> = Vec::with_capacity(no_macros);
    for b in 0..no_macros {
        let k = rho_list[b].len();
        let label = format!("hsbm_list_game (macro {b})");
        validate_rho(&rho_list[b], &label)?;
        validate_pref(&c_list[b], k, &label)?;
        let csizes = csizes_for(&rho_list[b], m_list[b], &label)?;
        all_csizes.push(csizes);
    }

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    // Phase 1: intra-macro.
    for b in 0..no_macros {
        sample_intra_macro(
            &mut rng,
            &mut edges,
            macro_offsets[b],
            &all_csizes[b],
            &c_list[b],
        );
    }

    // Phase 2: inter-macro.
    add_inter_macro(&mut rng, &mut edges, &macro_offsets, m_list, p);

    let mut g = Graph::new(n, false)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- validation: hsbm_game ------------------------------------

    #[test]
    fn hsbm_rejects_n_not_multiple_of_m() {
        let rho = vec![1.0];
        let c = vec![vec![0.5]];
        let res = hsbm_game(10, 3, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_zero_n() {
        let rho = vec![1.0];
        let c = vec![vec![0.5]];
        let res = hsbm_game(0, 1, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_zero_m() {
        let rho = vec![1.0];
        let c = vec![vec![0.5]];
        let res = hsbm_game(10, 0, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_p_out_of_range() {
        let rho = vec![1.0];
        let c = vec![vec![0.5]];
        let res = hsbm_game(10, 5, &rho, &c, 1.5, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_rho_not_summing_to_one() {
        let rho = vec![0.3, 0.3];
        let c = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let res = hsbm_game(10, 5, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_rho_times_m_not_integer() {
        // rho[0]*m = 0.5*7 = 3.5, not integer.
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let res = hsbm_game(14, 7, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_asymmetric_c() {
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.5, 0.2], vec![0.3, 0.5]];
        let res = hsbm_game(10, 5, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_c_out_of_range() {
        let rho = vec![1.0];
        let c = vec![vec![1.5]];
        let res = hsbm_game(10, 5, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_rejects_pref_wrong_shape() {
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.5]];
        let res = hsbm_game(10, 5, &rho, &c, 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    // -- structural correctness: hsbm_game ------------------------

    #[test]
    fn hsbm_trivial_one_vertex() {
        // Reproduces igraph_hsbm_game.out fixture #1: 1 vertex, 0 edges.
        let rho = vec![1.0];
        let c = vec![vec![1.0]];
        let g = hsbm_game(1, 1, &rho, &c, 0.0, 0).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn hsbm_complete_bipartite_within_macro() {
        // Reproduces fixture #2: one macro of 10, three clusters
        // sized 6, 4, 0; off-diagonal pref = 1 → complete bipartite
        // K_{6,4}; p=0 → no inter-macro work.
        let rho = vec![0.6, 0.4, 0.0];
        let c = vec![
            vec![0.0, 1.0, 0.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        let g = hsbm_game(10, 10, &rho, &c, 0.0, 0).unwrap();
        assert_eq!(g.vcount(), 10);
        // K_{6,4} has 24 edges.
        assert_eq!(g.ecount(), 24);
        // Every edge crosses the {0..6} vs {6..10} partition.
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            let (lo, hi) = if u < v { (u, v) } else { (v, u) };
            assert!(lo < 6 && (6..10).contains(&hi));
        }
    }

    #[test]
    fn hsbm_full_inter_macro_complete_minus_clusters() {
        // Reproduces fixture #3: two macros of 5, intra pref [0 1; 1 0]
        // (3,2 clusters → 6 intra-macro edges per macro), p=1
        // → complete bipartite between macros = 25 edges.
        // Total expected = 2*6 + 25 = 37 edges.
        let rho = vec![0.6, 0.4, 0.0];
        let c = vec![
            vec![0.0, 1.0, 0.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        let g = hsbm_game(10, 5, &rho, &c, 1.0, 0).unwrap();
        assert_eq!(g.vcount(), 10);
        assert_eq!(g.ecount(), 37);
    }

    #[test]
    fn hsbm_p_zero_isolates_macros() {
        // p = 0 means no inter-macro edges.
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.5, 0.1], vec![0.1, 0.5]];
        let g = hsbm_game(20, 10, &rho, &c, 0.0, 0xDEAD).unwrap();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            let bu = u / 10;
            let bv = v / 10;
            assert_eq!(bu, bv, "edge ({u}, {v}) crosses macros with p=0");
        }
    }

    #[test]
    fn hsbm_p_one_emits_all_inter_macro_edges() {
        // p=1 with c=0 inside macros: expect every cross-macro pair.
        let rho = vec![1.0];
        let c = vec![vec![0.0]];
        let g = hsbm_game(20, 5, &rho, &c, 1.0, 0).unwrap();
        // 4 macros of 5; cross-macro edges = C(4,2)*5*5 = 6*25 = 150.
        assert_eq!(g.ecount(), 150);
    }

    #[test]
    fn hsbm_no_self_loops_and_no_multi() {
        let rho = vec![0.5, 0.5];
        let c = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let g = hsbm_game(20, 10, &rho, &c, 0.5, 0).unwrap();
        let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v, "self-loop in HSBM");
            let key = if u <= v { (u, v) } else { (v, u) };
            assert!(seen.insert(key), "duplicate edge {u}-{v}");
        }
    }

    #[test]
    fn hsbm_determinism_same_seed_same_graph() {
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.3, 0.05], vec![0.05, 0.3]];
        let g1 = hsbm_game(40, 10, &rho, &c, 0.05, 0x00C0_FFEE).unwrap();
        let g2 = hsbm_game(40, 10, &rho, &c, 0.05, 0x00C0_FFEE).unwrap();
        let edges1: Vec<_> = (0..g1.ecount() as u32)
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let edges2: Vec<_> = (0..g2.ecount() as u32)
            .map(|e| g2.edge(e).unwrap())
            .collect();
        assert_eq!(edges1, edges2);
    }

    // -- validation: hsbm_list_game --------------------------------

    #[test]
    fn hsbm_list_rejects_length_mismatch() {
        let res = hsbm_list_game(10, &[10, 0], &[vec![1.0]], &[vec![vec![0.5]]], 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_list_rejects_sum_m_not_n() {
        let res = hsbm_list_game(
            10,
            &[5, 4],
            &[vec![1.0], vec![1.0]],
            &[vec![vec![0.5]], vec![vec![0.5]]],
            0.0,
            0,
        );
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_list_rejects_empty_macros() {
        let res = hsbm_list_game(10, &[], &[], &[], 0.0, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn hsbm_list_rejects_zero_size_macro() {
        let res = hsbm_list_game(
            5,
            &[5, 0],
            &[vec![1.0], vec![1.0]],
            &[vec![vec![0.5]], vec![vec![0.5]]],
            0.0,
            0,
        );
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    // -- structural correctness: hsbm_list_game --------------------

    #[test]
    fn hsbm_list_trivial_one_vertex() {
        let g = hsbm_list_game(1, &[1], &[vec![1.0]], &[vec![vec![1.0]]], 0.0, 0).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn hsbm_list_complete_bipartite_within_one_macro() {
        // Same as the C #2 fixture but expressed in list form.
        let m_list = vec![10u32];
        let rho_list = vec![vec![0.6, 0.4, 0.0]];
        let c_list = vec![vec![
            vec![0.0, 1.0, 0.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ]];
        let g = hsbm_list_game(10, &m_list, &rho_list, &c_list, 0.0, 0).unwrap();
        assert_eq!(g.ecount(), 24);
    }

    #[test]
    fn hsbm_list_two_macros_p_one_full_inter() {
        // Two macros of size 5; intra pref off-diagonal = 1 → K_{3,2}
        // per macro (6 edges each); p=1 → 25 inter edges. Total 37.
        let m_list = vec![5u32, 5];
        let rho_list = vec![vec![0.6, 0.4, 0.0], vec![0.6, 0.4, 0.0]];
        let c_list = vec![
            vec![
                vec![0.0, 1.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0],
            ],
            vec![
                vec![0.0, 1.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0],
            ],
        ];
        let g = hsbm_list_game(10, &m_list, &rho_list, &c_list, 1.0, 0).unwrap();
        assert_eq!(g.ecount(), 37);
    }

    #[test]
    fn hsbm_list_heterogeneous_macros_no_panic() {
        // Macros of different size and different inner k.
        let m_list = vec![3u32, 10, 5, 3];
        let rho_list = vec![
            vec![1.0 / 3.0, 2.0 / 3.0],
            vec![3.0 / 10.0, 3.0 / 10.0, 4.0 / 10.0],
            vec![1.0],
            vec![2.0 / 3.0, 1.0 / 3.0],
        ];
        let c_list = vec![
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
            vec![
                vec![0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0],
            ],
            vec![vec![0.0]],
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
        ];
        let g = hsbm_list_game(21, &m_list, &rho_list, &c_list, 1.0, 0).unwrap();
        assert_eq!(g.vcount(), 21);
        // p=1, all intra=0 → expected edges = sum over (b1<b2) m_b1 * m_b2.
        let mut expected = 0u64;
        for i in 0..m_list.len() {
            for j in (i + 1)..m_list.len() {
                expected += u64::from(m_list[i]) * u64::from(m_list[j]);
            }
        }
        assert_eq!(u64::try_from(g.ecount()).unwrap(), expected);
    }

    #[test]
    fn hsbm_list_uniform_matches_hsbm_game() {
        // hsbm_list_game with uniform args must produce the same graph
        // as hsbm_game given the same seed.
        let rho = vec![0.5, 0.5];
        let c = vec![vec![0.3, 0.05], vec![0.05, 0.3]];
        let g1 = hsbm_game(40, 10, &rho, &c, 0.05, 0xABCD).unwrap();

        let m_list = vec![10u32; 4];
        let rho_list = vec![rho.clone(); 4];
        let c_list = vec![c.clone(); 4];
        let g2 = hsbm_list_game(40, &m_list, &rho_list, &c_list, 0.05, 0xABCD).unwrap();

        let e1: Vec<_> = (0..g1.ecount() as u32)
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let e2: Vec<_> = (0..g2.ecount() as u32)
            .map(|e| g2.edge(e).unwrap())
            .collect();
        assert_eq!(e1, e2);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// HSBM uniform: vertex count is `n` regardless of args.
        #[test]
        fn hsbm_vcount_matches_n(
            no_macros in 1u32..5,
            cluster_count in 1usize..4,
            seed: u64,
        ) {
            let m = (cluster_count as u32) * 3;
            let n = no_macros * m;
            let rho = vec![1.0 / cluster_count as f64; cluster_count];
            let p_in = 0.2_f64;
            let p_off = 0.05_f64;
            let mut c = vec![vec![p_off; cluster_count]; cluster_count];
            for (i, row) in c.iter_mut().enumerate() {
                row[i] = p_in;
            }
            let g = hsbm_game(n, m, &rho, &c, 0.02, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
        }

        /// HSBM uniform: never self-loops, never multi-edges.
        #[test]
        fn hsbm_is_simple(
            no_macros in 1u32..4,
            seed: u64,
        ) {
            let m = 6u32;
            let n = no_macros * m;
            let rho = vec![0.5_f64, 0.5];
            let c = vec![vec![0.3, 0.05], vec![0.05, 0.3]];
            let g = hsbm_game(n, m, &rho, &c, 0.05, seed).unwrap();
            let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_ne!(u, v);
                let key = if u <= v { (u, v) } else { (v, u) };
                prop_assert!(seen.insert(key));
            }
        }

        /// HSBM uniform: with `p = 0` every edge stays within its
        /// macro-block (i.e. `u / m == v / m`).
        #[test]
        fn hsbm_p_zero_keeps_edges_inside_macros(
            no_macros in 2u32..5,
            seed: u64,
        ) {
            let m = 6u32;
            let n = no_macros * m;
            let rho = vec![0.5_f64, 0.5];
            let c = vec![vec![0.4, 0.1], vec![0.1, 0.4]];
            let g = hsbm_game(n, m, &rho, &c, 0.0, seed).unwrap();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_eq!(u / m, v / m);
            }
        }

        /// HSBM list: vertex count equals sum of `m_list`.
        #[test]
        fn hsbm_list_vcount_matches_sum_m(
            sizes in prop::collection::vec(1u32..7, 1usize..4),
            seed: u64,
        ) {
            let n: u32 = sizes.iter().sum();
            let rho_list: Vec<Vec<f64>> = sizes.iter().map(|_| vec![1.0]).collect();
            let c_list: Vec<Vec<Vec<f64>>> = sizes.iter().map(|_| vec![vec![0.2_f64]]).collect();
            let g = hsbm_list_game(n, &sizes, &rho_list, &c_list, 0.05, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
        }
    }
}
