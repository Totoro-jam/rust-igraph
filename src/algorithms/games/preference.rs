//! Preference-based block model (ALGO-GN-014).
//!
//! Counterparts of `igraph_preference_game()` and
//! `igraph_asymmetric_preference_game()` from
//! `references/igraph/src/games/preference.c:83` and
//! `references/igraph/src/games/preference.c:381`.
//!
//! The *symmetric* variant is essentially the non-growing version of
//! the establishment game / a stochastic block model with a randomly
//! assigned per-vertex type:
//!
//! 1. Each vertex draws a type from the categorical distribution
//!    `type_dist`, or — when `fixed_sizes = true` — gets pinned to a
//!    deterministic block-by-position assignment.
//! 2. Per type-pair `(i, j)`, edges are sampled with probability
//!    `pref_matrix[i][j]` over the rectangular `|V_i| × |V_j|` (or
//!    triangular) slot space using Batagelj–Brandes geometric skip,
//!    then mapped back to global vertex IDs through `vids_by_type`.
//!
//! The *asymmetric* variant gives every vertex an `(out_type, in_type)`
//! pair drawn jointly from `type_dist_matrix`. Sampling is per
//! `(out_type i, in_type j)` cell of the connection matrix; with
//! `loops = false` the sampler subtracts the count of vertices that
//! appear in both `vids_by_outtype[i]` and `vids_by_intype[j]` from the
//! local `maxedges`, then uses an end-corner remap to reroute any
//! sampled diagonal slot to a unique non-loop slot.
//!
//! Both variants share `PairShape::decode` and the geometric-skip
//! pattern with the SBM module ([`crate::algorithms::games::sbm`]); the
//! only addition is the indirection through `vids_by_type` (since
//! same-type vertices are not contiguous in vertex-id space) and the
//! intersection-based loop handling for the asymmetric path.
//!
//! ## Determinism
//!
//! Reproducible given the inputs and `seed` against the shared
//! `SplitMix64` PRNG. The stream is **not** portable
//! to upstream igraph's GLIBC RNG, so conformance assertions are
//! structural (vertex/edge counts, type-vector range, `loops` respect,
//! pref-matrix support) rather than bit-exact.
//!
//! ## References
//!
//! * V. Batagelj and U. Brandes, *"Efficient generation of large random
//!   networks"*, Phys. Rev. E **71**, 036113 (2005) — the geometric-skip
//!   sampler reused for each block-pair.
//! * K. Faust and S. Wasserman, *"Blockmodels: Interpretation and
//!   evaluation"*, Social Networks **14** (1992), 5–61.

#![allow(
    unknown_lints,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::needless_range_loop
)]

use crate::algorithms::games::sbm::PairShape;
use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Largest vertex count we accept; bounded by the largest `f64`-exact
/// integer (`2^53 - 1`), which is the `IGRAPH_MAX_EXACT_REAL` ceiling
/// upstream uses for the `maxedges` overflow check.
const MAX_NODES: u64 = 1u64 << 53;

// -------------------------------------------------------------------
// Shared shape selector (mirrors sbm::pair_shape but local for clarity)
// -------------------------------------------------------------------

fn pair_shape(directed: bool, loops: bool, on_diagonal: bool) -> PairShape {
    match (directed, loops, on_diagonal) {
        (true, false, true) => PairShape::RectNoDiag,
        (false, true, true) => PairShape::TriInclDiag,
        (false, false, true) => PairShape::TriExclDiag,
        _ => PairShape::Rect,
    }
}

fn shape_maxedges(shape: PairShape, fromsize: u32, tosize: u32) -> u64 {
    match shape {
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
    }
}

// -------------------------------------------------------------------
// Symmetric: igraph_preference_game
// -------------------------------------------------------------------

fn validate_symmetric(
    nodes: u32,
    types: u32,
    type_dist: Option<&[f64]>,
    fixed_sizes: bool,
    pref_matrix: &[Vec<f64>],
    directed: bool,
) -> IgraphResult<()> {
    if types < 1 {
        return Err(IgraphError::InvalidArgument(
            "The number of vertex types must be at least 1.".into(),
        ));
    }
    let k = types as usize;
    if let Some(td) = type_dist {
        if td.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "type_dist length ({}) does not match number of types ({k})",
                td.len()
            )));
        }
        for (i, &v) in td.iter().enumerate() {
            if v.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "type_dist[{i}] must not be NaN"
                )));
            }
            if v < 0.0 {
                return Err(IgraphError::InvalidArgument(format!(
                    "type_dist[{i}] = {v} must be non-negative"
                )));
            }
        }
    }
    if pref_matrix.len() != k {
        return Err(IgraphError::InvalidArgument(format!(
            "preference matrix has {} rows (expected {k})",
            pref_matrix.len()
        )));
    }
    for (i, row) in pref_matrix.iter().enumerate() {
        if row.len() != k {
            return Err(IgraphError::InvalidArgument(format!(
                "preference matrix row {i} has length {} (expected {k})",
                row.len()
            )));
        }
        for (j, &p) in row.iter().enumerate() {
            if p.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] must not be NaN"
                )));
            }
            if !(0.0..=1.0).contains(&p) {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] = {p} must lie in [0, 1]"
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
                        "preference matrix must be symmetric for undirected graphs: \
                         pref[{i}][{j}] = {pij} but pref[{j}][{i}] = {pji}"
                    )));
                }
            }
        }
    }
    if fixed_sizes {
        if let Some(td) = type_dist {
            // Sum must equal nodes (treated as integer counts).
            let mut sum: u64 = 0;
            for (i, &v) in td.iter().enumerate() {
                if v < 0.0 || !v.is_finite() {
                    return Err(IgraphError::InvalidArgument(format!(
                        "type_dist[{i}] = {v} must be a non-negative finite count when fixed_sizes is set"
                    )));
                }
                let c = v as u64;
                #[allow(clippy::float_cmp)]
                if (c as f64) != v {
                    return Err(IgraphError::InvalidArgument(format!(
                        "type_dist[{i}] = {v} must be an integer count when fixed_sizes is set"
                    )));
                }
                sum = sum.checked_add(c).ok_or_else(|| {
                    IgraphError::InvalidArgument("sum of type_dist overflows u64".into())
                })?;
            }
            if sum != u64::from(nodes) {
                return Err(IgraphError::InvalidArgument(format!(
                    "fixed_sizes requires sum(type_dist) = nodes; got {sum} vs {nodes}"
                )));
            }
        }
    }
    if u64::from(nodes) > MAX_NODES {
        return Err(IgraphError::InvalidArgument(format!(
            "nodes = {nodes} exceeds f64-exact ceiling 2^53"
        )));
    }
    Ok(())
}

/// Smallest `k` such that `cumdist[k] > u`. Mirrors
/// `igraph_vector_binsearch` for the cumulative-distribution case.
fn cumdist_lookup(cumdist: &[f64], u: f64) -> usize {
    // cumdist is non-decreasing with cumdist[0] == 0. We want the
    // smallest k >= 1 such that u < cumdist[k]. Equivalent to
    // upper_bound (first k with cumdist[k] > u).
    let mut lo = 1usize;
    let mut hi = cumdist.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cumdist[mid] > u {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo.min(cumdist.len() - 1).max(1)
}

fn assign_types_random(
    rng: &mut SplitMix64,
    nodes: u32,
    types: u32,
    type_dist: Option<&[f64]>,
    node_types: &mut [u32],
    vids_by_type: &mut [Vec<u32>],
) -> IgraphResult<()> {
    let k = types as usize;
    let mut cumdist = vec![0.0f64; k + 1];
    if let Some(td) = type_dist {
        for i in 0..k {
            cumdist[i + 1] = cumdist[i] + td[i];
        }
    } else {
        for i in 0..k {
            cumdist[i + 1] = (i + 1) as f64;
        }
    }
    let maxcum = cumdist[k];
    if maxcum <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "type_dist must have a positive total mass".into(),
        ));
    }
    for v in 0..nodes {
        let u = rng.gen_unit() * maxcum;
        let pos = cumdist_lookup(&cumdist, u);
        let t = (pos - 1).min(k - 1);
        node_types[v as usize] = t as u32;
        vids_by_type[t].push(v);
    }
    Ok(())
}

fn assign_types_fixed(
    nodes: u32,
    types: u32,
    type_dist: Option<&[f64]>,
    node_types: &mut [u32],
    vids_by_type: &mut [Vec<u32>],
) {
    let mut an: u32 = 0;
    if let Some(td) = type_dist {
        for (i, &cnt_f) in td.iter().enumerate() {
            let cnt = cnt_f as u32;
            for _ in 0..cnt {
                if an >= nodes {
                    break;
                }
                node_types[an as usize] = i as u32;
                vids_by_type[i].push(an);
                an += 1;
            }
        }
    } else {
        let size_one = nodes / types;
        let extra = nodes - size_one * types;
        for (i, slot) in vids_by_type.iter_mut().enumerate() {
            for _ in 0..size_one {
                node_types[an as usize] = i as u32;
                slot.push(an);
                an += 1;
            }
            if (i as u32) < extra {
                node_types[an as usize] = i as u32;
                slot.push(an);
                an += 1;
            }
        }
    }
}

/// Geometric-skip sampler over a single type-pair block. Decodes each
/// sampled index through `PairShape` and emits global edges via the
/// `v1` / `v2` indirection arrays.
fn sample_block_indirect(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    v1: &[u32],
    v2: &[u32],
    shape: PairShape,
    p: f64,
    maxedges: u64,
) {
    if maxedges == 0 || p <= 0.0 {
        return;
    }
    let v1size = v1.len() as u32;
    let max_f = maxedges as f64;
    let mut last = rng.gen_geom(p);
    while last < max_f {
        let idx = last.trunc() as u64;
        if idx >= maxedges {
            break;
        }
        let (vfrom, vto) = shape.decode(idx, v1size);
        edges.push((v1[vfrom as usize], v2[vto as usize]));
        last += rng.gen_geom(p);
        last += 1.0;
    }
}

/// Generate a random graph from a **type-aware preference (block) model**.
///
/// Each vertex is assigned to one of `types` types (uniformly, by
/// `type_dist`, or pinned by `fixed_sizes`); each ordered/unordered
/// vertex pair `(u, v)` is then connected with probability
/// `pref_matrix[type(u)][type(v)]`.
///
/// * `nodes` — number of vertices in the graph.
/// * `types` — number of vertex types (≥ 1).
/// * `type_dist` — categorical weights over types; `None` ⇒ uniform.
/// * `fixed_sizes` — when `true`, types are assigned by deterministic
///   block-by-position rather than sampled. With `type_dist = Some(td)`
///   the block sizes are `td[i]` (must be integer and sum to `nodes`).
///   With `type_dist = None` the function tries to make groups of
///   equal size.
/// * `pref_matrix` — `types × types` connection probabilities in
///   `[0, 1]`. Must be symmetric when `directed = false`.
/// * `directed` — generate a directed graph.
/// * `loops` — allow self-loop edges.
/// * `seed` — initialises an internal `SplitMix64` PRNG.
///
/// # Returns
///
/// `(graph, node_types)` where `node_types[i]` is the type of vertex
/// `i` in `0..types`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when:
/// * `types < 1`,
/// * `type_dist` length disagrees with `types`, contains NaN or a
///   negative value,
/// * `pref_matrix` is non-square or has the wrong dimensions,
/// * any pref entry is NaN or outside `[0, 1]`,
/// * `pref_matrix` is not symmetric when `!directed`,
/// * `fixed_sizes` and `type_dist` are both supplied but the entries
///   are not integer or do not sum to `nodes`,
/// * `nodes` exceeds the `2^53` `f64`-exact ceiling.
///
/// # Examples
///
/// ```
/// use rust_igraph::preference_game;
///
/// // Three types, equal mixture; dense within types, sparse between.
/// let pref = vec![
///     vec![0.20, 0.02, 0.02],
///     vec![0.02, 0.20, 0.02],
///     vec![0.02, 0.02, 0.20],
/// ];
/// let (g, types) = preference_game(60, 3, None, false, &pref, false, false, 42).unwrap();
/// assert_eq!(g.vcount(), 60);
/// assert_eq!(types.len(), 60);
/// assert!(types.iter().all(|&t| t < 3));
/// ```
pub fn preference_game(
    nodes: u32,
    types: u32,
    type_dist: Option<&[f64]>,
    fixed_sizes: bool,
    pref_matrix: &[Vec<f64>],
    directed: bool,
    loops: bool,
    seed: u64,
) -> IgraphResult<(Graph, Vec<u32>)> {
    validate_symmetric(nodes, types, type_dist, fixed_sizes, pref_matrix, directed)?;

    let k = types as usize;
    let mut node_types = vec![0u32; nodes as usize];
    let mut vids_by_type: Vec<Vec<u32>> = vec![Vec::new(); k];
    let mut rng = SplitMix64::new(seed);

    if fixed_sizes {
        assign_types_fixed(nodes, types, type_dist, &mut node_types, &mut vids_by_type);
    } else {
        assign_types_random(
            &mut rng,
            nodes,
            types,
            type_dist,
            &mut node_types,
            &mut vids_by_type,
        )?;
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    for i in 0..k {
        for j in 0..k {
            if !directed && i > j {
                continue;
            }
            let p = pref_matrix[i][j];
            if p <= 0.0 {
                continue;
            }
            let v1 = &vids_by_type[i];
            let v2 = &vids_by_type[j];
            let v1s = v1.len() as u32;
            let v2s = v2.len() as u32;
            if v1s == 0 || v2s == 0 {
                continue;
            }
            let on_diag = i == j;
            let shape = pair_shape(directed, loops, on_diag);
            let maxedges = shape_maxedges(shape, v1s, v2s);
            if maxedges as f64 > MAX_NODES as f64 {
                return Err(IgraphError::InvalidArgument(
                    "block maxedges overflows f64-exact representation".into(),
                ));
            }
            sample_block_indirect(&mut rng, &mut edges, v1, v2, shape, p, maxedges);
        }
    }

    let mut g = Graph::new(nodes, directed)?;
    g.add_edges(edges)?;
    Ok((g, node_types))
}

// -------------------------------------------------------------------
// Asymmetric: igraph_asymmetric_preference_game
// -------------------------------------------------------------------

fn validate_asymmetric(
    nodes: u32,
    no_out_types: u32,
    no_in_types: u32,
    type_dist_matrix: Option<&[Vec<f64>]>,
    pref_matrix: &[Vec<f64>],
) -> IgraphResult<()> {
    if no_out_types < 1 {
        return Err(IgraphError::InvalidArgument(
            "no_out_types must be at least 1".into(),
        ));
    }
    if no_in_types < 1 {
        return Err(IgraphError::InvalidArgument(
            "no_in_types must be at least 1".into(),
        ));
    }
    let no_out = no_out_types as usize;
    let no_in = no_in_types as usize;
    if let Some(tdm) = type_dist_matrix {
        if tdm.len() != no_out {
            return Err(IgraphError::InvalidArgument(format!(
                "type_dist_matrix has {} rows (expected {no_out})",
                tdm.len()
            )));
        }
        for (i, row) in tdm.iter().enumerate() {
            if row.len() != no_in {
                return Err(IgraphError::InvalidArgument(format!(
                    "type_dist_matrix row {i} has length {} (expected {no_in})",
                    row.len()
                )));
            }
            for (j, &v) in row.iter().enumerate() {
                if v.is_nan() {
                    return Err(IgraphError::InvalidArgument(format!(
                        "type_dist_matrix[{i}][{j}] must not be NaN"
                    )));
                }
                if v < 0.0 {
                    return Err(IgraphError::InvalidArgument(format!(
                        "type_dist_matrix[{i}][{j}] = {v} must be non-negative"
                    )));
                }
            }
        }
    }
    if pref_matrix.len() != no_out {
        return Err(IgraphError::InvalidArgument(format!(
            "preference matrix has {} rows (expected {no_out})",
            pref_matrix.len()
        )));
    }
    for (i, row) in pref_matrix.iter().enumerate() {
        if row.len() != no_in {
            return Err(IgraphError::InvalidArgument(format!(
                "preference matrix row {i} has length {} (expected {no_in})",
                row.len()
            )));
        }
        for (j, &p) in row.iter().enumerate() {
            if p.is_nan() {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] must not be NaN"
                )));
            }
            if !(0.0..=1.0).contains(&p) {
                return Err(IgraphError::InvalidArgument(format!(
                    "preference matrix entry [{i}][{j}] = {p} must lie in [0, 1]"
                )));
            }
        }
    }
    if u64::from(nodes) > MAX_NODES {
        return Err(IgraphError::InvalidArgument(format!(
            "nodes = {nodes} exceeds f64-exact ceiling 2^53"
        )));
    }
    Ok(())
}

/// Sorted-merge intersection of two ascending `&[u32]` slices.
fn sorted_intersection(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    out
}

/// Position of `target` in the sorted slice `xs`, or `None`.
fn binsearch_pos(xs: &[u32], target: u32) -> Option<usize> {
    xs.binary_search(&target).ok()
}

/// Sample one asymmetric type-pair block, emitting edges into `edges`.
/// Uses the upstream end-corner remap to reroute loop slots when
/// `loops = false` and `intersect` is non-empty.
fn sample_asym_pair(
    rng: &mut SplitMix64,
    edges: &mut Vec<(VertexId, VertexId)>,
    v1: &[u32],
    v2: &[u32],
    intersect: &[u32],
    p: f64,
    maxedges: u64,
    loops: bool,
) {
    if maxedges == 0 || p <= 0.0 {
        return;
    }
    let v1size = v1.len() as u32;
    let v2size = v2.len() as u32;
    let max_f = maxedges as f64;
    let intersect_count = intersect.len();

    let mut last = rng.gen_geom(p);
    while last < max_f {
        let idx = last.trunc() as u64;
        if idx >= maxedges {
            break;
        }
        let to = idx / u64::from(v1size);
        let from = idx - to * u64::from(v1size);
        let (mut vfrom, mut vto) = (from as u32, to as u32);

        if !loops && intersect_count > 0 && v1[vfrom as usize] == v2[vto as usize] {
            // Remap this loop slot to a non-loop slot in the rightmost
            // column. The k-th loop in `intersect` (0-indexed) is
            // mapped to a unique non-loop slot at column v2_size - 1.
            vto = v2size - 1;
            let loop_vid = v1[vfrom as usize];
            let mut c = binsearch_pos(intersect, loop_vid).unwrap_or(intersect_count);
            let mut from_idx: i64 = i64::from(v1size) - 1;
            // Skip the rightmost-column slot if it would itself be a loop.
            if from_idx >= 0 && v1[from_idx as usize] == v2[vto as usize] {
                from_idx -= 1;
            }
            while c > 0 && from_idx >= 0 {
                c -= 1;
                from_idx -= 1;
                if from_idx >= 0 && v1[from_idx as usize] == v2[vto as usize] {
                    from_idx -= 1;
                }
            }
            // After the walk, from_idx should point to a non-loop slot.
            // If we ran off the end (no remaining valid slot), skip
            // emitting this edge — the index is at the cap and the next
            // sample will bring `last >= max_f`.
            if from_idx < 0 {
                last += rng.gen_geom(p);
                last += 1.0;
                continue;
            }
            vfrom = from_idx as u32;
        }
        edges.push((v1[vfrom as usize], v2[vto as usize]));
        last += rng.gen_geom(p);
        last += 1.0;
    }
}

/// Generate a random directed graph with **asymmetric vertex types and
/// connection preferences**.
///
/// Each vertex is assigned an `(out_type, in_type)` pair drawn jointly
/// from `type_dist_matrix` (or uniformly when `None`). A directed edge
/// `u → v` is then sampled with probability
/// `pref_matrix[out_type(u)][in_type(v)]`.
///
/// * `nodes` — number of vertices.
/// * `no_out_types`, `no_in_types` — number of out- and in-types
///   (≥ 1).
/// * `type_dist_matrix` — `out_types × in_types` joint weights;
///   `None` ⇒ uniform.
/// * `pref_matrix` — `out_types × in_types` connection probabilities
///   in `[0, 1]`.
/// * `loops` — allow self-loop edges.
/// * `seed` — initialises an internal `SplitMix64` PRNG.
///
/// The resulting graph is **always directed**.
///
/// # Returns
///
/// `(graph, node_type_out, node_type_in)` — the per-vertex out- and
/// in-type assignments.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] when:
/// * `no_out_types < 1` or `no_in_types < 1`,
/// * `type_dist_matrix` has wrong dimensions, NaN, or negative entries,
/// * `pref_matrix` has wrong dimensions, NaN, or entries outside `[0, 1]`,
/// * `nodes` exceeds the `2^53` `f64`-exact ceiling.
///
/// # Examples
///
/// ```
/// use rust_igraph::asymmetric_preference_game;
///
/// // 2 out-types × 2 in-types, uniform mixing, p ≈ 0.1 across the board.
/// let pref = vec![vec![0.1, 0.1], vec![0.1, 0.1]];
/// let (g, out, inp) =
///     asymmetric_preference_game(40, 2, 2, None, &pref, true, 42).unwrap();
/// assert_eq!(g.vcount(), 40);
/// assert!(g.is_directed());
/// assert!(out.iter().all(|&t| t < 2));
/// assert!(inp.iter().all(|&t| t < 2));
/// ```
pub fn asymmetric_preference_game(
    nodes: u32,
    no_out_types: u32,
    no_in_types: u32,
    type_dist_matrix: Option<&[Vec<f64>]>,
    pref_matrix: &[Vec<f64>],
    loops: bool,
    seed: u64,
) -> IgraphResult<(Graph, Vec<u32>, Vec<u32>)> {
    validate_asymmetric(
        nodes,
        no_out_types,
        no_in_types,
        type_dist_matrix,
        pref_matrix,
    )?;

    let no_out = no_out_types as usize;
    let no_in = no_in_types as usize;
    let mut node_out = vec![0u32; nodes as usize];
    let mut node_in = vec![0u32; nodes as usize];
    let mut vids_by_outtype: Vec<Vec<u32>> = vec![Vec::new(); no_out];
    let mut vids_by_intype: Vec<Vec<u32>> = vec![Vec::new(); no_in];
    let mut rng = SplitMix64::new(seed);

    // Joint cumulative distribution: cumdist[k+1] = cumdist[k] +
    // matrix[out, in]; iteration order matches upstream so the decoded
    // type cell is `(cell % no_out_types, cell / no_out_types)` for
    // (out_type, in_type).
    let total_cells = no_out * no_in;
    let mut cumdist = vec![0.0f64; total_cells + 1];
    let mut k = 0usize;
    if let Some(tdm) = type_dist_matrix {
        // Iteration order matches upstream so the decoded type cell is
        // (cell % no_out_types, cell / no_out_types) for (out, in).
        for j_in in 0..no_in {
            for (i_out, row) in tdm.iter().enumerate().take(no_out) {
                cumdist[k + 1] = cumdist[k] + row[j_in];
                k += 1;
                let _ = i_out; // index used implicitly by enumerate
            }
        }
    } else {
        for i in 0..total_cells {
            cumdist[i + 1] = (i + 1) as f64;
        }
    }
    let maxcum = cumdist[total_cells];
    if maxcum <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "type_dist_matrix must have a positive total mass".into(),
        ));
    }

    for v in 0..nodes {
        let u = rng.gen_unit() * maxcum;
        let pos = cumdist_lookup(&cumdist, u);
        let cell = (pos - 1).min(total_cells - 1);
        let out_t = (cell % no_out) as u32;
        let in_t = (cell / no_out) as u32;
        node_out[v as usize] = out_t;
        node_in[v as usize] = in_t;
        vids_by_outtype[out_t as usize].push(v);
        vids_by_intype[in_t as usize].push(v);
    }

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    for i in 0..no_out {
        for j in 0..no_in {
            let p = pref_matrix[i][j];
            if p <= 0.0 {
                continue;
            }
            let v1 = &vids_by_outtype[i];
            let v2 = &vids_by_intype[j];
            let v1s = v1.len() as u32;
            let v2s = v2.len() as u32;
            if v1s == 0 || v2s == 0 {
                continue;
            }
            let mut maxedges: u64 = u64::from(v1s) * u64::from(v2s);
            if maxedges as f64 > MAX_NODES as f64 {
                return Err(IgraphError::InvalidArgument(
                    "asymmetric block maxedges overflows f64-exact representation".into(),
                ));
            }
            let intersect = if loops {
                Vec::new()
            } else {
                sorted_intersection(v1, v2)
            };
            if !loops {
                maxedges -= intersect.len() as u64;
            }
            sample_asym_pair(&mut rng, &mut edges, v1, v2, &intersect, p, maxedges, loops);
        }
    }

    let mut g = Graph::new(nodes, true)?;
    g.add_edges(edges)?;
    Ok((g, node_out, node_in))
}

// ===================================================================
// Tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn diag_pref_sym(k: usize, p: f64) -> Vec<Vec<f64>> {
        let mut m = vec![vec![0.0; k]; k];
        for (i, row) in m.iter_mut().enumerate() {
            row[i] = p;
        }
        m
    }

    fn uniform_pref_sym(k: usize, p: f64) -> Vec<Vec<f64>> {
        vec![vec![p; k]; k]
    }

    // -- symmetric: validation ----------------------------------------

    #[test]
    fn rejects_zero_types() {
        let res = preference_game(10, 0, None, false, &[], false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_pref_wrong_rows() {
        let pref = vec![vec![0.1, 0.1]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_pref_wrong_cols() {
        let pref = vec![vec![0.1], vec![0.1, 0.1]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_nan_pref() {
        let pref = vec![vec![f64::NAN, 0.1], vec![0.1, 0.1]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_pref_above_one() {
        let pref = vec![vec![0.5, 1.5], vec![1.5, 0.5]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_negative_pref() {
        let pref = vec![vec![0.5, -0.1], vec![-0.1, 0.5]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_asymmetric_pref_when_undirected() {
        let pref = vec![vec![0.1, 0.2], vec![0.3, 0.1]];
        let res = preference_game(10, 2, None, false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn accepts_asymmetric_pref_when_directed() {
        let pref = vec![vec![0.1, 0.2], vec![0.3, 0.1]];
        let g = preference_game(20, 2, None, false, &pref, true, false, 7).unwrap();
        assert_eq!(g.0.vcount(), 20);
        assert!(g.0.is_directed());
    }

    #[test]
    fn rejects_nan_type_dist() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![1.0, f64::NAN];
        let res = preference_game(10, 2, Some(&td), false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_negative_type_dist() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![1.0, -0.5];
        let res = preference_game(10, 2, Some(&td), false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_type_dist_length_mismatch() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![1.0, 1.0, 1.0];
        let res = preference_game(10, 2, Some(&td), false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn rejects_zero_total_mass() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![0.0, 0.0];
        let res = preference_game(10, 2, Some(&td), false, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn fixed_sizes_rejects_wrong_sum() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![3.0, 3.0]; // sum 6 != nodes 10
        let res = preference_game(10, 2, Some(&td), true, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn fixed_sizes_rejects_non_integer_counts() {
        let pref = uniform_pref_sym(2, 0.1);
        let td = vec![5.5, 4.5];
        let res = preference_game(10, 2, Some(&td), true, &pref, false, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    // -- symmetric: structural ----------------------------------------

    #[test]
    fn empty_pref_yields_no_edges() {
        let pref = uniform_pref_sym(3, 0.0);
        let (g, types) = preference_game(15, 3, None, false, &pref, false, false, 42).unwrap();
        assert_eq!(g.vcount(), 15);
        assert_eq!(g.ecount(), 0);
        assert!(types.iter().all(|&t| t < 3));
    }

    #[test]
    fn no_self_loops_when_loops_false_undirected() {
        let pref = uniform_pref_sym(3, 0.5);
        let (g, _types) = preference_game(20, 3, None, false, &pref, false, false, 7).unwrap();
        assert!(!g.is_directed());
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v);
        }
    }

    #[test]
    fn no_self_loops_when_loops_false_directed() {
        let pref = uniform_pref_sym(3, 0.5);
        let (g, _types) = preference_game(20, 3, None, false, &pref, true, false, 7).unwrap();
        assert!(g.is_directed());
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v);
        }
    }

    #[test]
    fn loops_allowed_when_loops_true() {
        // p = 1.0 + loops = true ⇒ every diagonal slot becomes an edge,
        // including the self-loops.
        let pref = diag_pref_sym(2, 1.0);
        let (g, _types) = preference_game(8, 2, None, false, &pref, false, true, 1).unwrap();
        let mut found_loop = false;
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            if u == v {
                found_loop = true;
                break;
            }
        }
        assert!(
            found_loop,
            "expected at least one self-loop with p=1, loops=true"
        );
    }

    #[test]
    fn fixed_sizes_equal_split_no_dist() {
        let pref = uniform_pref_sym(3, 0.0);
        let (g, types) = preference_game(9, 3, None, true, &pref, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 9);
        // Equal split: 3, 3, 3
        let mut counts = [0u32; 3];
        for &t in &types {
            counts[t as usize] += 1;
        }
        assert_eq!(counts, [3, 3, 3]);
    }

    #[test]
    fn fixed_sizes_explicit_distribution() {
        let pref = uniform_pref_sym(3, 0.0);
        let td = vec![5.0, 3.0, 2.0];
        let (g, types) = preference_game(10, 3, Some(&td), true, &pref, false, false, 0).unwrap();
        assert_eq!(g.vcount(), 10);
        let mut counts = [0u32; 3];
        for &t in &types {
            counts[t as usize] += 1;
        }
        assert_eq!(counts, [5, 3, 2]);
    }

    #[test]
    fn deterministic_same_seed() {
        let pref = uniform_pref_sym(3, 0.3);
        let (g1, t1) = preference_game(20, 3, None, false, &pref, false, false, 12345).unwrap();
        let (g2, t2) = preference_game(20, 3, None, false, &pref, false, false, 12345).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        assert_eq!(t1, t2);
        let edges1: Vec<_> = (0..g1.ecount() as u32)
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let edges2: Vec<_> = (0..g2.ecount() as u32)
            .map(|e| g2.edge(e).unwrap())
            .collect();
        assert_eq!(edges1, edges2);
    }

    #[test]
    fn diag_pref_keeps_edges_within_type() {
        let pref = diag_pref_sym(3, 0.5);
        let (g, types) = preference_game(30, 3, None, false, &pref, false, false, 42).unwrap();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_eq!(types[u as usize], types[v as usize]);
        }
    }

    #[test]
    fn type_dist_skews_assignment() {
        // Heavily skewed dist: type 0 should dominate.
        let pref = uniform_pref_sym(2, 0.0);
        let td = vec![100.0, 1.0];
        let (_g, types) =
            preference_game(50, 2, Some(&td), false, &pref, false, false, 99).unwrap();
        let count0 = types.iter().filter(|&&t| t == 0).count();
        let count1 = types.iter().filter(|&&t| t == 1).count();
        assert!(
            count0 > count1,
            "expected type 0 to dominate, got {count0} vs {count1}"
        );
    }

    // -- asymmetric: validation ---------------------------------------

    #[test]
    fn asym_rejects_zero_out_types() {
        let pref: Vec<Vec<f64>> = vec![];
        let res = asymmetric_preference_game(10, 0, 2, None, &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_zero_in_types() {
        let pref = vec![vec![]];
        let res = asymmetric_preference_game(10, 1, 0, None, &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_pref_wrong_dim() {
        let pref = vec![vec![0.1, 0.1]];
        let res = asymmetric_preference_game(10, 2, 2, None, &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_nan_pref() {
        let pref = vec![vec![0.1, f64::NAN], vec![0.1, 0.1]];
        let res = asymmetric_preference_game(10, 2, 2, None, &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_pref_out_of_range() {
        let pref = vec![vec![0.1, 1.5], vec![0.1, 0.1]];
        let res = asymmetric_preference_game(10, 2, 2, None, &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_tdm_wrong_dim() {
        let pref = vec![vec![0.1, 0.1], vec![0.1, 0.1]];
        let tdm = vec![vec![1.0, 1.0]]; // 1×2, expected 2×2
        let res = asymmetric_preference_game(10, 2, 2, Some(&tdm), &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_negative_tdm() {
        let pref = vec![vec![0.1, 0.1], vec![0.1, 0.1]];
        let tdm = vec![vec![1.0, -1.0], vec![1.0, 1.0]];
        let res = asymmetric_preference_game(10, 2, 2, Some(&tdm), &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn asym_rejects_zero_total_mass() {
        let pref = vec![vec![0.1, 0.1], vec![0.1, 0.1]];
        let tdm = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let res = asymmetric_preference_game(10, 2, 2, Some(&tdm), &pref, false, 0);
        assert!(matches!(res, Err(IgraphError::InvalidArgument(_))));
    }

    // -- asymmetric: structural ---------------------------------------

    #[test]
    fn asym_empty_pref_yields_no_edges() {
        let pref = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let (g, out, inp) = asymmetric_preference_game(15, 2, 2, None, &pref, false, 42).unwrap();
        assert_eq!(g.vcount(), 15);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
        assert!(out.iter().all(|&t| t < 2));
        assert!(inp.iter().all(|&t| t < 2));
    }

    #[test]
    fn asym_always_directed() {
        let pref = vec![vec![0.3, 0.1], vec![0.1, 0.3]];
        let (g, _, _) = asymmetric_preference_game(20, 2, 2, None, &pref, false, 1).unwrap();
        assert!(g.is_directed());
    }

    #[test]
    fn asym_no_self_loops_when_loops_false() {
        let pref = vec![vec![0.7, 0.3], vec![0.3, 0.7]];
        let (g, _out, _inp) = asymmetric_preference_game(30, 2, 2, None, &pref, false, 11).unwrap();
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v);
        }
    }

    #[test]
    fn asym_loops_allowed_when_loops_true() {
        // Single (out, in) cell at p = 1.0 ⇒ all v1 × v2 slots filled,
        // which include self-pairs.
        let pref = vec![vec![1.0]];
        let (g, _out, _inp) = asymmetric_preference_game(5, 1, 1, None, &pref, true, 0).unwrap();
        // 5 vertices, all 25 directed slots present (with loops).
        assert_eq!(g.ecount(), 25);
        let mut found_loop = false;
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            if u == v {
                found_loop = true;
                break;
            }
        }
        assert!(found_loop);
    }

    #[test]
    fn asym_loops_false_full_pref_yields_no_loops_full_off_diag() {
        let pref = vec![vec![1.0]];
        let (g, _out, _inp) = asymmetric_preference_game(5, 1, 1, None, &pref, false, 0).unwrap();
        // 5 vertices, no loops ⇒ 5*5 - 5 = 20 directed off-diagonal edges.
        assert_eq!(g.ecount(), 20);
        for e in 0..g.ecount() as u32 {
            let (u, v) = g.edge(e).unwrap();
            assert_ne!(u, v);
        }
    }

    #[test]
    fn asym_deterministic_same_seed() {
        let pref = vec![vec![0.3, 0.1], vec![0.1, 0.3]];
        let (g1, o1, i1) = asymmetric_preference_game(20, 2, 2, None, &pref, false, 9999).unwrap();
        let (g2, o2, i2) = asymmetric_preference_game(20, 2, 2, None, &pref, false, 9999).unwrap();
        assert_eq!(g1.ecount(), g2.ecount());
        assert_eq!(o1, o2);
        assert_eq!(i1, i2);
        let e1: Vec<_> = (0..g1.ecount() as u32)
            .map(|e| g1.edge(e).unwrap())
            .collect();
        let e2: Vec<_> = (0..g2.ecount() as u32)
            .map(|e| g2.edge(e).unwrap())
            .collect();
        assert_eq!(e1, e2);
    }

    // -- helpers tested directly -------------------------------------

    #[test]
    fn cumdist_lookup_handles_endpoints() {
        let cd = vec![0.0, 1.0, 3.0, 6.0];
        assert_eq!(cumdist_lookup(&cd, 0.0), 1);
        assert_eq!(cumdist_lookup(&cd, 0.5), 1);
        assert_eq!(cumdist_lookup(&cd, 1.0), 2);
        assert_eq!(cumdist_lookup(&cd, 2.99), 2);
        assert_eq!(cumdist_lookup(&cd, 3.0), 3);
        assert_eq!(cumdist_lookup(&cd, 5.5), 3);
    }

    #[test]
    fn sorted_intersection_basic() {
        assert_eq!(
            sorted_intersection(&[1, 3, 5, 7], &[2, 3, 5, 9]),
            vec![3, 5]
        );
        assert_eq!(sorted_intersection(&[], &[1, 2]), Vec::<u32>::new());
        assert_eq!(sorted_intersection(&[1, 2, 3], &[4, 5]), Vec::<u32>::new());
        assert_eq!(sorted_intersection(&[1, 2, 3], &[1, 2, 3]), vec![1, 2, 3]);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptest_invariants {
    use super::*;
    use proptest::prelude::*;

    fn uniform_pref_sym(k: usize, p: f64) -> Vec<Vec<f64>> {
        vec![vec![p; k]; k]
    }

    fn diag_pref_sym(k: usize, p: f64) -> Vec<Vec<f64>> {
        let mut m = vec![vec![0.0; k]; k];
        for (i, row) in m.iter_mut().enumerate() {
            row[i] = p;
        }
        m
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn vcount_equals_nodes(
            n in 1u32..40,
            k in 1u32..5,
            seed: u64,
            directed: bool,
            loops: bool,
        ) {
            let pref = uniform_pref_sym(k as usize, 0.2);
            let (g, types) = preference_game(n, k, None, false, &pref, directed, loops, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert_eq!(types.len(), n as usize);
            for &t in &types {
                prop_assert!(t < k);
            }
        }

        #[test]
        fn no_self_loops_when_loops_false(
            n in 1u32..30,
            k in 1u32..4,
            seed: u64,
            directed: bool,
        ) {
            let pref = uniform_pref_sym(k as usize, 0.4);
            let (g, _types) = preference_game(n, k, None, false, &pref, directed, false, seed).unwrap();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        #[test]
        fn diag_pref_keeps_edges_within_type(
            n in 4u32..30,
            k in 2u32..4,
            seed: u64,
        ) {
            let pref = diag_pref_sym(k as usize, 0.5);
            let (g, types) = preference_game(n, k, None, false, &pref, false, false, seed).unwrap();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_eq!(types[u as usize], types[v as usize]);
            }
        }

        #[test]
        fn fixed_sizes_equal_split_counts_match(
            size_one in 1u32..6,
            k in 1u32..5,
            seed: u64,
        ) {
            let pref = uniform_pref_sym(k as usize, 0.0);
            let n = size_one * k;
            let (_g, types) = preference_game(n, k, None, true, &pref, false, false, seed).unwrap();
            let mut counts = vec![0u32; k as usize];
            for &t in &types {
                counts[t as usize] += 1;
            }
            for &c in &counts {
                prop_assert_eq!(c, size_one);
            }
        }

        #[test]
        fn determinism_seed(
            n in 1u32..25,
            k in 1u32..4,
            seed: u64,
        ) {
            let pref = uniform_pref_sym(k as usize, 0.3);
            let (g1, t1) = preference_game(n, k, None, false, &pref, false, false, seed).unwrap();
            let (g2, t2) = preference_game(n, k, None, false, &pref, false, false, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            prop_assert_eq!(t1, t2);
        }

        #[test]
        fn asym_vcount_matches(
            n in 1u32..30,
            no_out in 1u32..4,
            no_in in 1u32..4,
            seed: u64,
            loops: bool,
        ) {
            let pref = vec![vec![0.2; no_in as usize]; no_out as usize];
            let (g, out, inp) =
                asymmetric_preference_game(n, no_out, no_in, None, &pref, loops, seed).unwrap();
            prop_assert_eq!(g.vcount(), n);
            prop_assert!(g.is_directed());
            prop_assert_eq!(out.len(), n as usize);
            prop_assert_eq!(inp.len(), n as usize);
            for &t in &out {
                prop_assert!(t < no_out);
            }
            for &t in &inp {
                prop_assert!(t < no_in);
            }
        }

        #[test]
        fn asym_no_self_loops_when_loops_false(
            n in 1u32..30,
            no_out in 1u32..3,
            no_in in 1u32..3,
            seed: u64,
        ) {
            let pref = vec![vec![0.5; no_in as usize]; no_out as usize];
            let (g, _out, _inp) =
                asymmetric_preference_game(n, no_out, no_in, None, &pref, false, seed).unwrap();
            for e in 0..g.ecount() as u32 {
                let (u, v) = g.edge(e).unwrap();
                prop_assert_ne!(u, v);
            }
        }

        #[test]
        fn asym_determinism_seed(
            n in 1u32..25,
            no_out in 1u32..3,
            no_in in 1u32..3,
            seed: u64,
        ) {
            let pref = vec![vec![0.3; no_in as usize]; no_out as usize];
            let (g1, o1, i1) =
                asymmetric_preference_game(n, no_out, no_in, None, &pref, false, seed).unwrap();
            let (g2, o2, i2) =
                asymmetric_preference_game(n, no_out, no_in, None, &pref, false, seed).unwrap();
            prop_assert_eq!(g1.ecount(), g2.ecount());
            prop_assert_eq!(o1, o2);
            prop_assert_eq!(i1, i2);
        }
    }
}
