//! Infomap community detection (ALGO-CO-018).
//!
//! Rosvall & Bergstrom (2008): *Maps of information flow reveal community
//! structure in complex networks.* PNAS 105(4):1118-1123.
//! <https://doi.org/10.1073/pnas.0706851105>
//!
//! Minimises the two-level *map equation* — the expected per-step
//! description length of a random walker that uses module-level
//! codebooks. A lower codelength means the partition captures more
//! of the graph's flow structure.
//!
//! Algorithm (greedy, Louvain-style):
//!
//! 1. Compute PageRank (ergodic visit frequencies) with teleportation
//!    τ = 0.15.
//! 2. Place each vertex in its own module.
//! 3. Greedily move vertices to neighbouring modules that decrease the
//!    map equation the most.
//! 4. Repeat until no improvement.
//! 5. (Optional) Run `nb_trials` independent attempts and keep the
//!    partition with the shortest codelength.
//!
//! Self-rolled, dependency-free. Supports directed and undirected
//! graphs, optional edge weights and vertex weights (teleportation
//! targets).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use crate::core::{Graph, IgraphError, IgraphResult};

const TAU: f64 = 0.15;
const PR_EPS: f64 = 1e-10;
const PR_MAX_ITER: usize = 1000;
const MAX_PASSES: usize = 256;
const DL_EPS: f64 = 1e-12;

/// Result of an Infomap run.
#[derive(Debug, Clone)]
pub struct InfomapResult {
    /// Community assignment for each vertex (0-indexed, contiguous).
    pub membership: Vec<u32>,
    /// Map equation codelength of the best partition found.
    pub codelength: f64,
}

/// Run Infomap with the default options (unweighted, 1 trial).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, infomap};
///
/// // Two triangles connected by a bridge — two obvious communities.
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let r = infomap(&g).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[3], r.membership[4]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn infomap(graph: &Graph) -> IgraphResult<InfomapResult> {
    infomap_with_options(graph, None, None, 1, 0)
}

/// Run Infomap with per-edge weights (1 trial).
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `weights.len() != ecount` or
///   any weight is negative or non-finite.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, infomap_weighted};
///
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let w = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
/// let r = infomap_weighted(&g, &w).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_ne!(r.membership[0], r.membership[3]);
/// ```
pub fn infomap_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<InfomapResult> {
    infomap_with_options(graph, Some(weights), None, 1, 0)
}

/// Run Infomap with full control over parameters.
///
/// - `edge_weights`: per-edge weights; `None` for unit weights. Must be
///   non-negative and finite.
/// - `vertex_weights`: teleportation target weights; `None` for uniform.
///   Must be non-negative and finite; at least one must be positive.
/// - `nb_trials`: number of independent optimisation attempts (≥ 1).
/// - `seed`: PRNG seed for the node-order shuffle.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] for malformed weights or
///   `nb_trials < 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, infomap_with_options};
///
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let a = infomap_with_options(&g, None, None, 3, 42).unwrap();
/// let b = infomap_with_options(&g, None, None, 3, 42).unwrap();
/// assert_eq!(a.membership, b.membership);
/// ```
pub fn infomap_with_options(
    graph: &Graph,
    edge_weights: Option<&[f64]>,
    vertex_weights: Option<&[f64]>,
    nb_trials: u32,
    seed: u64,
) -> IgraphResult<InfomapResult> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    if nb_trials < 1 {
        return Err(IgraphError::InvalidArgument(
            "nb_trials must be at least 1".to_string(),
        ));
    }
    validate_edge_weights(m, edge_weights)?;
    validate_vertex_weights(n, vertex_weights)?;

    if n == 0 {
        return Ok(InfomapResult {
            membership: Vec::new(),
            codelength: f64::NAN,
        });
    }

    let fs = build_flow_state(graph, edge_weights, vertex_weights)?;

    let mut rng = SplitMix64::new(seed);
    let mut best: Option<InfomapResult> = None;

    for _ in 0..nb_trials {
        let result = run_trial(&fs, &mut rng);
        let dominated = match &best {
            None => false,
            Some(b) => result.codelength >= b.codelength,
        };
        if !dominated {
            best = Some(result);
        }
    }

    Ok(best.unwrap_or(InfomapResult {
        membership: vec![0; n],
        codelength: f64::NAN,
    }))
}

// ──────────────────── Validation ────────────────────

fn validate_edge_weights(m: usize, weights: Option<&[f64]>) -> IgraphResult<()> {
    if let Some(ew) = weights {
        if ew.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "edge weight vector length {} does not match edge count {m}",
                ew.len()
            )));
        }
        for (i, &w) in ew.iter().enumerate() {
            if w < 0.0 || !w.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "edge weight[{i}] = {w} is invalid (must be non-negative and finite)"
                )));
            }
        }
    }
    Ok(())
}

fn validate_vertex_weights(n: usize, weights: Option<&[f64]>) -> IgraphResult<()> {
    if let Some(vw) = weights {
        if vw.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "vertex weight vector length {} does not match vertex count {n}",
                vw.len()
            )));
        }
        let mut any_positive = false;
        for (i, &w) in vw.iter().enumerate() {
            if w < 0.0 || !w.is_finite() {
                return Err(IgraphError::InvalidArgument(format!(
                    "vertex weight[{i}] = {w} is invalid"
                )));
            }
            if w > 0.0 {
                any_positive = true;
            }
        }
        if !any_positive && n > 0 {
            return Err(IgraphError::InvalidArgument(
                "all vertex weights are zero".to_string(),
            ));
        }
    }
    Ok(())
}

// ──────────────────── Internal types ────────────────────

struct FlowState {
    n: usize,
    node_flow: Vec<f64>,
    out_flow: Vec<Vec<(u32, f64)>>,
    in_flow: Vec<Vec<(u32, f64)>>,
    tele_weight: Vec<f64>,
}

// ──────────────────── Flow computation ────────────────────

fn build_flow_state(
    graph: &Graph,
    edge_weights: Option<&[f64]>,
    vertex_weights: Option<&[f64]>,
) -> IgraphResult<FlowState> {
    let n = graph.vcount() as usize;
    let directed = graph.is_directed();
    let ecount = graph.ecount();
    let m32 = u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;

    // Build weighted out-adjacency and compute out-strength
    let mut out_adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut out_strength = vec![0.0_f64; n];

    for e in 0..m32 {
        let (u, v) = graph.edge(e)?;
        let w = edge_weights.map_or(1.0, |ew| ew[e as usize]);
        if directed {
            out_adj[u as usize].push((v, w));
            out_strength[u as usize] += w;
        } else {
            out_adj[u as usize].push((v, w));
            out_adj[v as usize].push((u, w));
            out_strength[u as usize] += w;
            out_strength[v as usize] += w;
        }
    }

    // Teleportation weights
    let tele_weight: Vec<f64> = if let Some(vw) = vertex_weights {
        let s: f64 = vw.iter().sum();
        if s > 0.0 {
            vw.iter().map(|&w| w / s).collect()
        } else {
            vec![1.0 / n as f64; n]
        }
    } else {
        vec![1.0 / n as f64; n]
    };

    // Compute PageRank
    let node_flow = power_iteration_pagerank(n, &out_adj, &out_strength, &tele_weight)?;

    // Build edge flows: for each edge u→v, flow = node_flow[u] * (1-τ) * w / out_strength[u]
    let mut out_flow: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut in_flow: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];

    for u in 0..n {
        if out_strength[u] > 0.0 {
            for &(v, w) in &out_adj[u] {
                let ef = node_flow[u] * (1.0 - TAU) * w / out_strength[u];
                out_flow[u].push((v, ef));
                in_flow[v as usize].push((u as u32, ef));
            }
        }
    }

    Ok(FlowState {
        n,
        node_flow,
        out_flow,
        in_flow,
        tele_weight,
    })
}

fn power_iteration_pagerank(
    n: usize,
    out_adj: &[Vec<(u32, f64)>],
    out_strength: &[f64],
    tele_weight: &[f64],
) -> IgraphResult<Vec<f64>> {
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![1.0]);
    }

    // Build in-adjacency for iteration: in_adj[v] = list of (u, w/out_strength[u])
    let mut in_adj: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    for u in 0..n {
        if out_strength[u] > 0.0 {
            for &(v, w) in &out_adj[u] {
                in_adj[v as usize].push((u as u32, w / out_strength[u]));
            }
        }
    }

    let dangling: Vec<bool> = out_strength.iter().map(|&s| s <= 0.0).collect();
    let mut pr = vec![1.0 / n as f64; n];
    let mut buf = vec![0.0_f64; n];

    for _ in 0..PR_MAX_ITER {
        let dangle_sum: f64 = (0..n).filter(|&i| dangling[i]).map(|i| pr[i]).sum();

        for v in 0..n {
            let link: f64 = in_adj[v]
                .iter()
                .map(|&(u, w_norm)| pr[u as usize] * w_norm)
                .sum();
            buf[v] = TAU * tele_weight[v] + (1.0 - TAU) * link + dangle_sum * tele_weight[v];
        }

        let s: f64 = buf.iter().sum();
        if s > 0.0 {
            for p in &mut buf {
                *p /= s;
            }
        }

        let diff: f64 = pr.iter().zip(buf.iter()).map(|(a, b)| (a - b).abs()).sum();
        std::mem::swap(&mut pr, &mut buf);

        if diff < PR_EPS {
            break;
        }
    }

    Ok(pr)
}

// ──────────────────── Map equation ────────────────────

fn plogp(x: f64) -> f64 {
    if x > 0.0 { x * x.log2() } else { 0.0 }
}

/// Compute the map equation codelength from per-module exit flows and
/// node flows.
///
/// L(M) = plogp(q↷) − 2·Σᵢ plogp(qᵢ) + Σᵢ plogp(qᵢ + pᵢ) − Σ_α plogp(p_α)
fn codelength(
    _membership: &[u32],
    module_exit: &[f64],
    module_flow: &[f64],
    node_flow: &[f64],
    n_modules: usize,
) -> f64 {
    let total_exit: f64 = module_exit[..n_modules].iter().sum();

    let mut sum_plogp_exit = 0.0_f64;
    let mut sum_plogp_exit_plus_flow = 0.0_f64;

    for i in 0..n_modules {
        sum_plogp_exit += plogp(module_exit[i]);
        sum_plogp_exit_plus_flow += plogp(module_exit[i] + module_flow[i]);
    }

    let sum_plogp_node: f64 = node_flow.iter().copied().map(plogp).sum();

    plogp(total_exit) - 2.0 * sum_plogp_exit + sum_plogp_exit_plus_flow - sum_plogp_node
}

/// Compute the exit flow for a single module from scratch.
fn module_exit_flow(mod_id: u32, membership: &[u32], fs: &FlowState) -> f64 {
    let mut edge_exit = 0.0_f64;
    let mut mod_flow = 0.0_f64;
    let mut tele_in_mod = 0.0_f64;
    let mut any = false;

    for v in 0..fs.n {
        if membership[v] != mod_id {
            continue;
        }
        any = true;
        mod_flow += fs.node_flow[v];
        tele_in_mod += fs.tele_weight[v];

        for &(u, ef) in &fs.out_flow[v] {
            if membership[u as usize] != mod_id {
                edge_exit += ef;
            }
        }
    }

    if !any {
        return 0.0;
    }

    // Teleportation exit: each node α in module i teleports outside with
    // probability τ * (1 - tele_weight_sum_in_module). Weighted by node flow.
    let tele_exit = TAU * mod_flow * (1.0 - tele_in_mod);
    edge_exit + tele_exit
}

// ──────────────────── Greedy optimisation ────────────────────

fn run_trial(fs: &FlowState, rng: &mut SplitMix64) -> InfomapResult {
    let n = fs.n;
    if n == 0 {
        return InfomapResult {
            membership: Vec::new(),
            codelength: f64::NAN,
        };
    }

    // Each node starts in its own module
    let mut membership: Vec<u32> = (0..n as u32).collect();
    let mut module_flow: Vec<f64> = fs.node_flow.clone();
    let mut module_size: Vec<u32> = vec![1; n];
    let mut module_exit: Vec<f64> = (0..n)
        .map(|v| module_exit_flow(v as u32, &membership, fs))
        .collect();

    let mut cl = codelength(&membership, &module_exit, &module_flow, &fs.node_flow, n);

    let mut node_order: Vec<usize> = (0..n).collect();
    if n > 1 {
        shuffle(&mut node_order, rng);
    }

    for _pass in 0..MAX_PASSES {
        let prev_cl = cl;
        let mut changed = false;

        if n > 1 {
            let a = rng.gen_index(n);
            let b = rng.gen_index(n);
            node_order.swap(a, b);
        }

        for &v in &node_order {
            let old_mod = membership[v];

            // Collect unique neighbour modules
            let mut nbr_mods: Vec<u32> = Vec::new();
            for &(u, _) in &fs.out_flow[v] {
                let um = membership[u as usize];
                if um != old_mod && !nbr_mods.contains(&um) {
                    nbr_mods.push(um);
                }
            }
            for &(u, _) in &fs.in_flow[v] {
                let um = membership[u as usize];
                if um != old_mod && !nbr_mods.contains(&um) {
                    nbr_mods.push(um);
                }
            }

            if nbr_mods.is_empty() {
                continue;
            }

            // Current codelength contributions of the affected modules
            let best = try_moves(
                v,
                old_mod,
                &nbr_mods,
                fs,
                &membership,
                &module_exit,
                &module_flow,
                &module_size,
            );

            if let Some((new_mod, delta)) = best {
                if delta < -DL_EPS {
                    // Apply the move
                    let p_v = fs.node_flow[v];
                    membership[v] = new_mod;

                    module_flow[old_mod as usize] -= p_v;
                    module_flow[new_mod as usize] += p_v;
                    module_size[old_mod as usize] -= 1;
                    module_size[new_mod as usize] += 1;

                    // Recompute exit for affected modules
                    // (old_mod, new_mod, and any module with edges to/from v)
                    let mut to_update: Vec<u32> = vec![old_mod, new_mod];
                    for &(u, _) in &fs.out_flow[v] {
                        let um = membership[u as usize];
                        if !to_update.contains(&um) {
                            to_update.push(um);
                        }
                    }
                    for &(u, _) in &fs.in_flow[v] {
                        let um = membership[u as usize];
                        if !to_update.contains(&um) {
                            to_update.push(um);
                        }
                    }
                    for &mid in &to_update {
                        module_exit[mid as usize] = module_exit_flow(mid, &membership, fs);
                    }

                    changed = true;
                }
            }
        }

        cl = codelength(&membership, &module_exit, &module_flow, &fs.node_flow, n);

        if !changed || prev_cl - cl < DL_EPS {
            break;
        }
    }

    InfomapResult {
        membership: reindex(&membership),
        codelength: cl,
    }
}

/// Try moving node v from old_mod to each candidate module.
/// Returns `Some((best_mod, delta_cl))` if an improvement exists.
fn try_moves(
    v: usize,
    old_mod: u32,
    candidates: &[u32],
    fs: &FlowState,
    membership: &[u32],
    module_exit: &[f64],
    module_flow: &[f64],
    module_size: &[u32],
) -> Option<(u32, f64)> {
    let p_v = fs.node_flow[v];

    // State before: module_exit[old_mod], module_flow[old_mod], etc.
    let total_exit_before: f64 = module_exit.iter().sum();
    let old_exit_before = module_exit[old_mod as usize];
    let old_flow_before = module_flow[old_mod as usize];

    // If v is the only node in old_mod, moving it would empty the module.
    let old_becomes_empty = module_size[old_mod as usize] == 1;

    // Compute what old_mod's exit would be after removing v
    let old_exit_after = if old_becomes_empty {
        0.0
    } else {
        exit_after_remove(v, old_mod, fs, membership)
    };
    let old_flow_after = old_flow_before - p_v;

    let mut best_mod = old_mod;
    let mut best_delta = 0.0_f64;

    for &new_mod in candidates {
        let new_exit_before = module_exit[new_mod as usize];
        let new_flow_before = module_flow[new_mod as usize];

        // What new_mod's exit would be after adding v
        let new_exit_after = exit_after_add(v, new_mod, fs, membership);
        let new_flow_after = new_flow_before + p_v;

        // Change in total exit
        let new_total_exit =
            total_exit_before - old_exit_before + old_exit_after - new_exit_before + new_exit_after;

        // Delta codelength:
        // ΔL = plogp(new_total) - plogp(old_total)
        //    + [-2·plogp(old_exit_after) + 2·plogp(old_exit_before)]
        //    + [-2·plogp(new_exit_after) + 2·plogp(new_exit_before)]
        //    + [plogp(old_exit_after + old_flow_after) - plogp(old_exit_before + old_flow_before)]
        //    + [plogp(new_exit_after + new_flow_after) - plogp(new_exit_before + new_flow_before)]
        let mut delta = plogp(new_total_exit) - plogp(total_exit_before);
        delta += 2.0 * (plogp(old_exit_before) - plogp(old_exit_after));
        delta += 2.0 * (plogp(new_exit_before) - plogp(new_exit_after));
        delta += plogp(old_exit_after + old_flow_after) - plogp(old_exit_before + old_flow_before);
        delta += plogp(new_exit_after + new_flow_after) - plogp(new_exit_before + new_flow_before);

        if delta < best_delta - DL_EPS {
            best_delta = delta;
            best_mod = new_mod;
        }
    }

    if best_mod == old_mod {
        None
    } else {
        Some((best_mod, best_delta))
    }
}

fn exit_after_remove(v: usize, mod_id: u32, fs: &FlowState, membership: &[u32]) -> f64 {
    let mut edge_exit = 0.0_f64;
    let mut mod_flow = 0.0_f64;
    let mut tele_in = 0.0_f64;
    let mut any = false;

    for u in 0..fs.n {
        if membership[u] != mod_id || u == v {
            continue;
        }
        any = true;
        mod_flow += fs.node_flow[u];
        tele_in += fs.tele_weight[u];

        for &(w, ef) in &fs.out_flow[u] {
            let wm = if w as usize == v {
                u32::MAX
            } else {
                membership[w as usize]
            };
            if wm != mod_id {
                edge_exit += ef;
            }
        }
    }

    if !any {
        return 0.0;
    }

    edge_exit + TAU * mod_flow * (1.0 - tele_in)
}

fn exit_after_add(v: usize, mod_id: u32, fs: &FlowState, membership: &[u32]) -> f64 {
    let mut edge_exit = 0.0_f64;
    let mut mod_flow = 0.0_f64;
    let mut tele_in = 0.0_f64;

    for u in 0..fs.n {
        if membership[u] != mod_id {
            continue;
        }
        mod_flow += fs.node_flow[u];
        tele_in += fs.tele_weight[u];

        for &(w, ef) in &fs.out_flow[u] {
            let wm = if w as usize == v {
                mod_id
            } else {
                membership[w as usize]
            };
            if wm != mod_id {
                edge_exit += ef;
            }
        }
    }

    // Add v itself
    mod_flow += fs.node_flow[v];
    tele_in += fs.tele_weight[v];

    for &(w, ef) in &fs.out_flow[v] {
        if membership[w as usize] != mod_id && w as usize != v {
            edge_exit += ef;
        }
    }

    edge_exit + TAU * mod_flow * (1.0 - tele_in)
}

// ──────────────────── Helpers ────────────────────

fn reindex(membership: &[u32]) -> Vec<u32> {
    let mut map: Vec<Option<u32>> = Vec::new();
    let mut next_id = 0u32;
    let mut result = Vec::with_capacity(membership.len());

    for &m in membership {
        let idx = m as usize;
        while map.len() <= idx {
            map.push(None);
        }
        let new_id = if let Some(id) = map[idx] {
            id
        } else {
            let id = next_id;
            next_id = next_id.saturating_add(1);
            map[idx] = Some(id);
            id
        };
        result.push(new_id);
    }

    result
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn gen_index(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n
    }
}

fn shuffle<T>(slice: &mut [T], rng: &mut SplitMix64) {
    for i in (1..slice.len()).rev() {
        let j = rng.gen_index(i + 1);
        slice.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangles() -> Graph {
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn basic_two_communities() {
        let g = two_triangles();
        let r = infomap(&g).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = infomap(&g).unwrap();
        assert!(r.membership.is_empty());
        assert!(r.codelength.is_nan());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let r = infomap(&g).unwrap();
        assert_eq!(r.membership, vec![0]);
    }

    #[test]
    fn disconnected_components() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        let r = infomap(&g).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[0], r.membership[2]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_eq!(r.membership[3], r.membership[5]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn deterministic_with_seed() {
        let g = two_triangles();
        let a = infomap_with_options(&g, None, None, 3, 42).unwrap();
        let b = infomap_with_options(&g, None, None, 3, 42).unwrap();
        assert_eq!(a.membership, b.membership);
    }

    #[test]
    fn weighted_edges() {
        let g = two_triangles();
        let w = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
        let r = infomap_weighted(&g, &w).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn directed_graph() {
        let g = Graph::from_edges(
            &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)],
            true,
            Some(6),
        )
        .unwrap();
        let r = infomap(&g).unwrap();
        assert_eq!(r.membership[0], r.membership[1]);
        assert_eq!(r.membership[3], r.membership[4]);
        assert_ne!(r.membership[0], r.membership[3]);
    }

    #[test]
    fn invalid_edge_weights() {
        let g = two_triangles();
        let w = vec![-1.0; 7];
        assert!(infomap_weighted(&g, &w).is_err());
    }

    #[test]
    fn invalid_nb_trials() {
        let g = two_triangles();
        assert!(infomap_with_options(&g, None, None, 0, 0).is_err());
    }

    #[test]
    fn karate_club() {
        let g = Graph::from_edges(
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (0, 5),
                (0, 6),
                (0, 7),
                (0, 8),
                (0, 10),
                (0, 11),
                (0, 12),
                (0, 13),
                (0, 17),
                (0, 19),
                (0, 21),
                (0, 31),
                (1, 2),
                (1, 3),
                (1, 7),
                (1, 13),
                (1, 17),
                (1, 19),
                (1, 21),
                (1, 30),
                (2, 3),
                (2, 7),
                (2, 8),
                (2, 9),
                (2, 13),
                (2, 27),
                (2, 28),
                (2, 32),
                (3, 7),
                (3, 12),
                (3, 13),
                (4, 6),
                (4, 10),
                (5, 6),
                (5, 10),
                (5, 16),
                (6, 16),
                (8, 30),
                (8, 32),
                (8, 33),
                (9, 33),
                (13, 33),
                (14, 32),
                (14, 33),
                (15, 32),
                (15, 33),
                (18, 32),
                (18, 33),
                (19, 33),
                (20, 32),
                (20, 33),
                (22, 32),
                (22, 33),
                (23, 25),
                (23, 27),
                (23, 29),
                (23, 32),
                (23, 33),
                (24, 25),
                (24, 27),
                (24, 31),
                (25, 31),
                (26, 29),
                (26, 33),
                (27, 33),
                (28, 31),
                (28, 33),
                (29, 32),
                (29, 33),
                (30, 32),
                (30, 33),
                (31, 32),
                (31, 33),
                (32, 33),
            ],
            false,
            None,
        )
        .unwrap();
        let r = infomap(&g).unwrap();
        // Should find at least 2 communities
        let k = *r.membership.iter().max().unwrap_or(&0) + 1;
        assert!(
            k >= 2,
            "karate club should have at least 2 communities, got {k}"
        );
        assert!(r.codelength.is_finite());
    }
}
