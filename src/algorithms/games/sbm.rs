//! Stochastic Block Model (ALGO-GN-010).
//!
//! Counterpart of `igraph_sbm_game()` from
//! `references/igraph/src/games/sbm.c:78` — the canonical
//! community-aware generalisation of Erdős–Rényi G(n,p).
//!
//! Given a `k × k` *preference matrix* `P` and a `k`-vector of
//! `block_sizes` summing to `n`, every pair `(u, v)` is connected
//! independently with probability `P[block(u)][block(v)]` (or, in
//! multigraph mode, with `Pascal`-distributed multiplicity of mean
//! `P[..][..]`). The vertex order is determined by `block_sizes`: the
//! first `block_sizes[0]` vertices form block 0, the next
//! `block_sizes[1]` form block 1, and so on.
//!
//! Each block-pair is sampled independently with Batagelj–Brandes
//! geometric skip over the local pair-index space; total cost is
//! `O(n + m + k²)` where `m` is the realised edge count and `k` is the
//! block count.
//!
//! ## Scope
//!
//! Ports the full `igraph_sbm_game` surface:
//!
//! * `directed`: directed vs undirected graph. When `false`, `P` must
//!   be symmetric.
//! * `loops`: when `true`, self-loops are allowed within blocks.
//! * `multiple`: when `true`, pref-matrix entries are expected edge
//!   multiplicities (each pair may receive multiple edges following a
//!   geometric distribution); when `false`, entries are connection
//!   probabilities in `[0, 1]`.
//!
//! The hierarchical variants `igraph_hsbm_game` and
//! `igraph_hsbm_list_game` are deferred to a follow-up AWU.
//!
//! ## Determinism
//!
//! The output is fully reproducible given `(pref_matrix, block_sizes,
//! directed, loops, multiple, seed)` via the shared
//! [`crate::core::rng::SplitMix64`] PRNG.
//!
//! ## References
//!
//! * K. Faust and S. Wasserman, *"Blockmodels: Interpretation and
//!   evaluation"*, Social Networks **14** (1992), 5-61.
//! * V. Batagelj and U. Brandes, *"Efficient generation of large
//!   random networks"*, Phys. Rev. E **71**, 036113 (2005) — the
//!   geometric-skip sampler reused for each block-pair.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::manual_midpoint
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Per-block-pair sampling kind. Determines the local `maxedges`
/// formula and the index → `(vfrom, vto)` decoder.
#[derive(Clone, Copy)]
enum PairShape {
    /// Rectangular `fromsize × tosize` grid (off-diagonal block, or
    /// directed-with-loops diagonal block).
    Rect,
    /// Rectangular `fromsize × (fromsize - 1)` grid with the diagonal
    /// remapped to the last column (directed, no loops, on-diagonal).
    RectNoDiag,
    /// Triangular `fromsize × (fromsize + 1) / 2` grid including the
    /// diagonal (undirected, loops, on-diagonal).
    TriInclDiag,
    /// Triangular `fromsize × (fromsize - 1) / 2` grid excluding the
    /// diagonal (undirected, no loops, on-diagonal).
    TriExclDiag,
}

impl PairShape {
    /// Decode a pair-index into `(vfrom, vto)` within the block pair's
    /// **local** coordinate frame. Caller adds the block offsets.
    fn decode(self, idx: u64, fromsize: u32) -> (u32, u32) {
        let fs = u64::from(fromsize);
        match self {
            Self::Rect => {
                let vto = idx / fs;
                let vfrom = idx - vto * fs;
                debug_assert!(vfrom < fs);
                (vfrom as u32, vto as u32)
            }
            Self::RectNoDiag => {
                let vto = idx / fs;
                let vfrom = idx - vto * fs;
                let vto = if vfrom == vto { fs - 1 } else { vto };
                debug_assert!(vfrom < fs && vto < fs && vfrom != vto);
                (vfrom as u32, vto as u32)
            }
            Self::TriInclDiag => {
                // vto = floor((sqrt(8·idx + 1) - 1) / 2);
                // vfrom = idx - vto·(vto + 1) / 2
                // Yields 0 <= vfrom <= vto < fs.
                let idx_f = idx as f64;
                let vto_f = ((8.0 * idx_f + 1.0).sqrt() - 1.0) / 2.0;
                let mut vto = vto_f.trunc() as u64;
                let mut vfrom = idx - vto * (vto + 1) / 2;
                while vfrom > vto {
                    vto += 1;
                    vfrom = idx - vto * (vto + 1) / 2;
                }
                debug_assert!(vfrom <= vto && vto < fs);
                (vfrom as u32, vto as u32)
            }
            Self::TriExclDiag => {
                // vto = floor((sqrt(8·idx + 1) + 1) / 2);
                // vfrom = idx - vto·(vto - 1) / 2
                // Yields 0 <= vfrom < vto < fs.
                let idx_f = idx as f64;
                let vto_f = ((8.0 * idx_f + 1.0).sqrt() + 1.0) / 2.0;
                let mut vto = vto_f.trunc() as u64;
                if vto < 1 {
                    vto = 1;
                }
                let mut vfrom = idx - vto * (vto - 1) / 2;
                while vfrom >= vto {
                    vto += 1;
                    vfrom = idx - vto * (vto - 1) / 2;
                }
                debug_assert!(vfrom < vto && vto < fs);
                (vfrom as u32, vto as u32)
            }
        }
    }
}

/// Classify a single block-pair sample task based on the global flags
/// and whether the pair is on-diagonal.
fn pair_shape(directed: bool, loops: bool, on_diagonal: bool) -> PairShape {
    match (directed, loops, on_diagonal) {
        (true, false, true) => PairShape::RectNoDiag,
        (false, true, true) => PairShape::TriInclDiag,
        (false, false, true) => PairShape::TriExclDiag,
        _ => PairShape::Rect,
    }
}

fn validate(
    pref_matrix: &[Vec<f64>],
    block_sizes: &[u32],
    directed: bool,
    multiple: bool,
) -> IgraphResult<()> {
    let k = pref_matrix.len();
    if block_sizes.len() != k {
        return Err(IgraphError::InvalidArgument(format!(
            "block_sizes length ({}) does not match preference matrix size ({k})",
            block_sizes.len()
        )));
    }
    for (i, row) in pref_matrix.iter().enumerate() {
        if row.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "preference matrix row {i} has length {} (expected {k} for square matrix)",
                row.len()
            )));
        }
        for (j, &val) in row.iter().enumerate() {
            if !val.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] must be finite (got {val})"
                )));
            }
            if multiple {
                if val < 0.0 {
                    return Err(IgraphError::InvalidArgument(format!(
                        "preference matrix entry [{i}][{j}] = {val} must be non-negative \
                         (multigraph SBM uses expected multiplicities)"
                    )));
                }
            } else if !(0.0..=1.0).contains(&val) {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] = {val} must lie in [0, 1] \
                     (simple SBM uses connection probabilities)"
                )));
            }
        }
    }
    if !directed {
        for (i, row_i) in pref_matrix.iter().enumerate() {
            for (j, row_j) in pref_matrix.iter().enumerate().skip(i + 1) {
                let pij = row_i[j];
                let pji = row_j[i];
                if pij != pji {
                    return Err(IgraphError::InvalidArgument(format!(
                        "preference matrix must be symmetric for undirected SBM: \
                         pref[{i}][{j}] = {pij} but pref[{j}][{i}] = {pji}"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Build cumulative block offsets and return the total vertex count.
fn block_offsets(block_sizes: &[u32]) -> IgraphResult<(Vec<u32>, u32)> {
    let mut offsets: Vec<u32> = Vec::with_capacity(block_sizes.len() + 1);
    offsets.push(0);
    let mut acc: u32 = 0;
    for &s in block_sizes {
        acc = acc.checked_add(s).ok_or_else(|| {
            IgraphError::InvalidArgument("sum of block_sizes overflows u32".into())
        })?;
        offsets.push(acc);
    }
    Ok((offsets, acc))
}

/// Walk pair-index space `[0, maxedges)` via the Batagelj–Brandes
/// geometric skip and emit decoded `(vfrom + fromoff, vto + tooff)`
/// edges into the buffer.
#[allow(clippy::too_many_arguments)]
fn sample_pair_with_max(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    fromsize: u32,
    fromoff: u32,
    tooff: u32,
    shape: PairShape,
    multiple: bool,
    prob: f64,
    maxedges: u64,
) {
    if maxedges == 0 || prob <= 0.0 {
        return;
    }
    let prob_step = if multiple { prob / (1.0 + prob) } else { prob };
    if prob_step <= 0.0 {
        return;
    }
    let max_f = maxedges as f64;
    // Simple-graph step adds 1.0 to enforce strictly-increasing
    // indices; multigraph step adds 0.0 (Pascal sampling).
    let step_extra: f64 = if multiple { 0.0 } else { 1.0 };

    let mut last = rng.gen_geom(prob_step);
    while last < max_f {
        let idx = last.trunc() as u64;
        if idx >= maxedges {
            break;
        }
        let (vfrom, vto) = shape.decode(idx, fromsize);
        edges.push((fromoff + vfrom, tooff + vto));
        last += rng.gen_geom(prob_step);
        last += step_extra;
    }
}

/// Generate a random graph from the **Stochastic Block Model**.
///
/// Each pair `(u, v)` is sampled with probability
/// `pref_matrix[block(u)][block(v)]` (`!multiple` mode) or with
/// `Pascal`-distributed multiplicity (`multiple` mode). Sampling
/// happens block-pair by block-pair using the Batagelj–Brandes
/// geometric-skip algorithm — total cost is `O(n + m + k²)`.
///
/// * `pref_matrix` — `k × k` preference matrix. Each row must have the
///   same length as the outer vector. In `!multiple` mode every entry
///   must lie in `[0, 1]`; in `multiple` mode every entry must be a
///   non-negative finite real.
/// * `block_sizes` — length-`k` vector of per-block vertex counts. The
///   resulting graph has `n = block_sizes.iter().sum()` vertices.
/// * `directed` — generate a directed graph. When `false`,
///   `pref_matrix` must be symmetric.
/// * `loops` — allow self-loop edges.
/// * `multiple` — switch to multigraph semantics (pref entries are
///   expected multiplicities, not probabilities).
/// * `seed` — initialises an internal `SplitMix64` PRNG.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if:
/// * a row of `pref_matrix` has the wrong length,
/// * `block_sizes.len() != pref_matrix.len()`,
/// * any pref-matrix entry is non-finite,
/// * any entry is outside `[0, 1]` when `!multiple`,
/// * any entry is negative when `multiple`,
/// * `pref_matrix` is not symmetric when `!directed`,
/// * `sum(block_sizes)` overflows `u32`.
///
/// # Examples
///
/// ```
/// use rust_igraph::sbm_game;
///
/// // Two equally-sized blocks: dense inside, sparse between.
/// let pref = vec![vec![0.2, 0.01], vec![0.01, 0.2]];
/// let sizes = vec![50, 50];
/// let g = sbm_game(&pref, &sizes, false, false, false, 42).unwrap();
/// assert_eq!(g.vcount(), 100);
/// // Expected ecount ≈ 0.2·(50·49/2) + 0.2·(50·49/2) + 0.01·(50·50)
/// //               = 245 + 245 + 25 = 515.
/// let m = g.ecount();
/// assert!((350..650).contains(&m), "ecount = {m}");
/// ```
pub fn sbm_game(
    pref_matrix: &[Vec<f64>],
    block_sizes: &[u32],
    directed: bool,
    loops: bool,
    multiple: bool,
    seed: u64,
) -> IgraphResult<Graph> {
    validate(pref_matrix, block_sizes, directed, multiple)?;

    let no_blocks = block_sizes.len();
    let (offsets, n) = block_offsets(block_sizes)?;

    if no_blocks == 0 || n == 0 {
        return Graph::new(n, directed);
    }

    let mut rng = SplitMix64::new(seed);
    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    for from in 0..no_blocks {
        let fromsize = block_sizes[from];
        if fromsize == 0 {
            continue;
        }
        let fromoff = offsets[from];
        let start = if directed { 0 } else { from };
        for to in start..no_blocks {
            let tosize = block_sizes[to];
            if tosize == 0 {
                continue;
            }
            let tooff = offsets[to];
            let prob = pref_matrix[from][to];
            if prob <= 0.0 {
                continue;
            }

            let on_diagonal = from == to;
            let shape = pair_shape(directed, loops, on_diagonal);
            let maxedges = match shape {
                PairShape::Rect => u64::from(fromsize) * u64::from(tosize),
                PairShape::RectNoDiag => {
                    let fs = u64::from(fromsize);
                    fs * fs.saturating_sub(1)
                }
                PairShape::TriInclDiag => {
                    let fs = u64::from(fromsize);
                    fs * (fs + 1) / 2
                }
                PairShape::TriExclDiag => {
                    let fs = u64::from(fromsize);
                    fs * fs.saturating_sub(1) / 2
                }
            };

            sample_pair_with_max(
                &mut rng, &mut edges, fromsize, fromoff, tooff, shape, multiple, prob, maxedges,
            );
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- shape decoders --------------------------------------------

    #[test]
    fn decode_rect_basic() {
        let shape = PairShape::Rect;
        assert_eq!(shape.decode(0, 3), (0, 0));
        assert_eq!(shape.decode(1, 3), (1, 0));
        assert_eq!(shape.decode(2, 3), (2, 0));
        assert_eq!(shape.decode(3, 3), (0, 1));
        assert_eq!(shape.decode(8, 3), (2, 2));
    }

    #[test]
    fn decode_rect_no_diag_remaps() {
        // fromsize = 3, the "diagonal" cells (0,0), (1,1), (2,2)
        // are remapped to column fromsize-1 = 2.
        let shape = PairShape::RectNoDiag;
        // idx 0 → vfrom = 0, vto = 0 (raw) → remap to (0, 2)
        assert_eq!(shape.decode(0, 3), (0, 2));
        // idx 1 → vfrom = 1, vto = 0 (raw, no remap)
        assert_eq!(shape.decode(1, 3), (1, 0));
        // idx 4 → vfrom = 1, vto = 1 (raw) → remap to (1, 2)
        assert_eq!(shape.decode(4, 3), (1, 2));
        // idx 8 → vfrom = 2, vto = 2 (raw) → remap to (2, 2). But
        // vfrom == fromsize - 1, so we land on (2, 2) which violates
        // the no-loop invariant. The C indexing convention ensures
        // this case is never visited when maxedges = fromsize*(fromsize-1) = 6,
        // so we only walk idx ∈ [0, 6).
        // idx 5 → vfrom = 2, vto = 1; no remap → (2, 1)
        assert_eq!(shape.decode(5, 3), (2, 1));
    }

    #[test]
    fn decode_tri_incl_diag_covers_all_pairs() {
        // fromsize = 4, maxedges = 4*5/2 = 10. Indices 0..10 should
        // produce every (vfrom, vto) with 0 <= vfrom <= vto < 4.
        let shape = PairShape::TriInclDiag;
        let mut seen: Vec<(u32, u32)> = (0..10).map(|i| shape.decode(i, 4)).collect();
        seen.sort_unstable();
        let mut expected: Vec<(u32, u32)> = (0..4)
            .flat_map(|to| (0..=to).map(move |from| (from, to)))
            .collect();
        expected.sort_unstable();
        assert_eq!(seen, expected);
    }

    #[test]
    fn decode_tri_excl_diag_covers_all_pairs() {
        let shape = PairShape::TriExclDiag;
        let mut seen: Vec<(u32, u32)> = (0..6).map(|i| shape.decode(i, 4)).collect();
        seen.sort_unstable();
        let mut expected: Vec<(u32, u32)> = (0..4)
            .flat_map(|to| (0..to).map(move |from| (from, to)))
            .collect();
        expected.sort_unstable();
        assert_eq!(seen, expected);
    }

    // -- validation -----------------------------------------------

    #[test]
    fn empty_pref_and_blocks_gives_empty_graph() {
        let g = sbm_game(&[], &[], false, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn rejects_non_square_pref() {
        let pref: Vec<Vec<f64>> = vec![vec![0.1, 0.2], vec![0.2]];
        let res = sbm_game(&pref, &[3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_mismatched_block_sizes() {
        let pref = vec![vec![0.1, 0.2], vec![0.2, 0.3]];
        let res = sbm_game(&pref, &[3, 3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_prob_above_one() {
        let pref = vec![vec![0.5, 1.2], vec![1.2, 0.5]];
        let res = sbm_game(&pref, &[3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_negative_prob() {
        let pref = vec![vec![0.5, -0.01], vec![-0.01, 0.5]];
        let res = sbm_game(&pref, &[3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_non_finite() {
        let pref = vec![vec![0.5, f64::NAN], vec![f64::NAN, 0.5]];
        let res = sbm_game(&pref, &[3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_asymmetric_for_undirected() {
        let pref = vec![vec![0.1, 0.2], vec![0.3, 0.1]];
        let res = sbm_game(&pref, &[3, 3], false, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn accepts_asymmetric_for_directed() {
        let pref = vec![vec![0.1, 0.2], vec![0.3, 0.1]];
        let res = sbm_game(&pref, &[3, 3], true, false, false, 0);
        assert!(res.is_ok());
    }

    #[test]
    fn rejects_multiple_negative() {
        let pref = vec![vec![0.5, -0.1], vec![-0.1, 0.5]];
        let res = sbm_game(&pref, &[3, 3], false, false, true, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn accepts_multiple_above_one() {
        // In multigraph mode, entries are multiplicities — values > 1
        // are legal.
        let pref = vec![vec![0.5, 2.5], vec![2.5, 0.5]];
        let g = sbm_game(&pref, &[3, 3], false, false, true, 0).unwrap();
        assert_eq!(g.vcount(), 6);
    }

    // -- structural correctness ------------------------------------

    fn block_of(v: u32, block_sizes: &[u32]) -> usize {
        let mut acc = 0u32;
        for (i, &s) in block_sizes.iter().enumerate() {
            acc += s;
            if v < acc {
                return i;
            }
        }
        block_sizes.len()
    }

    #[test]
    fn single_block_reduces_to_er_undirected() {
        // k = 1, p = 0.5, 30 vertices. Expect ecount close to
        // 0.5 · (30·29/2) = 217.5.
        let pref = vec![vec![0.5]];
        let g = sbm_game(&pref, &[30], false, false, false, 0xA5A5).unwrap();
        assert_eq!(g.vcount(), 30);
        let m = g.ecount();
        assert!(
            (100..350).contains(&m),
            "single-block undirected ecount = {m}"
        );
        // Endpoints all in the only block.
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert!(u < 30 && v < 30);
            assert_ne!(u, v, "no self-loops when loops=false");
        }
    }

    #[test]
    fn diagonal_only_pref_gives_per_block_er() {
        // Block-diagonal pref: edges only inside each block.
        let pref = vec![
            vec![0.5, 0.0, 0.0],
            vec![0.0, 0.5, 0.0],
            vec![0.0, 0.0, 0.5],
        ];
        let sizes = [20u32, 20, 20];
        let g = sbm_game(&pref, &sizes, false, false, false, 0x00C0_FFEE).unwrap();
        assert_eq!(g.vcount(), 60);
        let m = g.ecount() as u32;
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            let bu = block_of(u, &sizes);
            let bv = block_of(v, &sizes);
            assert_eq!(
                bu, bv,
                "edge {u}-{v} crosses blocks ({bu} vs {bv}) under diagonal pref"
            );
        }
    }

    #[test]
    fn off_diagonal_only_pref_gives_bipartite_blocks() {
        // pref[0][1] = pref[1][0] = 0.4, diagonal = 0.
        // All edges should run between block 0 and block 1.
        let pref = vec![vec![0.0, 0.4], vec![0.4, 0.0]];
        let sizes = [15u32, 15];
        let g = sbm_game(&pref, &sizes, false, false, false, 0xDEAD_BEEF).unwrap();
        assert_eq!(g.vcount(), 30);
        let m = g.ecount() as u32;
        for e in 0..m {
            let (u, v) = g.edge(e).unwrap();
            let bu = block_of(u, &sizes);
            let bv = block_of(v, &sizes);
            assert_ne!(bu, bv, "edge {u}-{v} stays inside block {bu}");
        }
    }

    #[test]
    fn no_self_loops_when_loops_false() {
        // Probability 1.0 on diagonal but loops disabled.
        let pref = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let sizes = [5u32, 5];
        let g = sbm_game(&pref, &sizes, false, false, false, 0x42).unwrap();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v, "self-loop on edge {e} = ({u}, {v})");
        }
        // Each block 5×5/2 = 10 edges (excluding self-loops): K_5 has 10 edges.
        assert_eq!(g.ecount(), 20);
    }

    #[test]
    fn self_loops_when_loops_true() {
        // 1-block, probability 1.0 with loops; expect K_n plus all
        // n self-loops = n(n+1)/2 edges.
        let pref = vec![vec![1.0]];
        let g = sbm_game(&pref, &[4], false, true, false, 0).unwrap();
        // 4*5/2 = 10 edges expected.
        assert_eq!(g.ecount(), 10);
        let mut loop_count = 0;
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            if u == v {
                loop_count += 1;
            }
        }
        assert_eq!(loop_count, 4, "expected one self-loop per vertex");
    }

    #[test]
    fn directed_complete_no_loops() {
        // p=1, directed, no loops on a single block. Expected ecount = n*(n-1).
        let pref = vec![vec![1.0]];
        let g = sbm_game(&pref, &[5], true, false, false, 0).unwrap();
        assert_eq!(g.ecount(), 5 * 4);
        // Every ordered pair (u, v) with u != v appears exactly once.
        let mut seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v);
            assert!(seen.insert((u, v)), "duplicate ordered pair ({u}, {v})");
        }
        assert_eq!(seen.len(), 5 * 4);
    }

    #[test]
    fn directed_asymmetric_pref_only_one_direction() {
        // pref[0][1] = 0.8, pref[1][0] = 0.0. Edges should only go
        // 0→1, never 1→0.
        let pref = vec![vec![0.0, 0.8], vec![0.0, 0.0]];
        let sizes = [8u32, 8];
        let g = sbm_game(&pref, &sizes, true, false, false, 0xBEEF).unwrap();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert!(u < 8 && v >= 8, "edge ({u}, {v}) should run 0→1 only");
        }
    }

    #[test]
    fn determinism_same_seed_same_graph() {
        let pref = vec![vec![0.3, 0.05], vec![0.05, 0.3]];
        let sizes = [30u32, 30];
        let g1 = sbm_game(&pref, &sizes, false, false, false, 0xAB_CDEF).unwrap();
        let g2 = sbm_game(&pref, &sizes, false, false, false, 0xAB_CDEF).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        let edges1: Vec<_> = (0..g1.ecount() as u32)
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let edges2: Vec<_> = (0..g2.ecount() as u32)
            .map(|e| g2.edge(e).unwrap())
            .collect();
        assert_eq!(edges1, edges2);
    }

    #[test]
    fn zero_size_block_does_not_break() {
        // Block 1 has zero vertices; expect a graph on just the
        // remaining 5 vertices.
        let pref = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let g = sbm_game(&pref, &[5, 0], false, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 5);
    }

    #[test]
    fn zero_probability_block_yields_no_edges() {
        let pref = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let g = sbm_game(&pref, &[10, 10], false, false, false, 42).unwrap();
        assert_eq!(g.vcount(), 20);
        assert_eq!(g.ecount(), 0);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    /// Build a uniform-`p` symmetric pref matrix on `k` blocks.
    fn uniform_pref(k: usize, p: f64) -> Vec<Vec<f64>> {
        vec![vec![p; k]; k]
    }

    /// Build a block-diagonal-only pref matrix: `pref[i][i] = p`,
    /// off-diagonal entries are zero.
    fn diag_pref(k: usize, p: f64) -> Vec<Vec<f64>> {
        let mut m = vec![vec![0.0; k]; k];
        for (i, row) in m.iter_mut().enumerate() {
            row[i] = p;
        }
        m
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn vcount_equals_block_sum(
            sizes in prop::collection::vec(1u32..8, 1usize..5),
            seed: u64,
        ) {
            let pref = uniform_pref(sizes.len(), 0.2);
            let g = sbm_game(&pref, &sizes, false, false, false, seed).unwrap();
            let expected: u32 = sizes.iter().sum();
            prop_assert_eq!(g.vcount(), expected);
        }

        #[test]
        fn no_self_loops_when_loops_false_undirected(
            sizes in prop::collection::vec(1u32..6, 1usize..4),
            seed: u64,
        ) {
            let pref = uniform_pref(sizes.len(), 0.25);
            let g = sbm_game(&pref, &sizes, false, false, false, seed).unwrap();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        #[test]
        fn edges_respect_block_pref_support(
            sizes in prop::collection::vec(1u32..6, 2usize..4),
            seed: u64,
        ) {
            let pref = diag_pref(sizes.len(), 0.3);
            let g = sbm_game(&pref, &sizes, false, false, false, seed).unwrap();
            let mut offsets = vec![0u32];
            let mut acc = 0u32;
            for &s in &sizes {
                acc += s;
                offsets.push(acc);
            }
            let block_of = |v: u32| -> usize {
                offsets.iter().position(|&o| v < o).map(|i| i - 1).unwrap_or(0)
            };
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_eq!(block_of(u), block_of(v));
            }
        }
    }
}
