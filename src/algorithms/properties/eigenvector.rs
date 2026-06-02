//! Eigenvector centrality (ALGO-PR-012 / PR-012b).
//!
//! Counterpart of `igraph_eigenvector_centrality()` from
//! `references/igraph/src/centrality/eigenvector.c`. The eigenvector
//! centrality of `v` is its component in the dominant eigenvector of
//! the (possibly weighted, possibly directed) adjacency matrix —
//! largest real-part eigenvalue, real and non-negative by
//! Perron-Frobenius for strongly-connected non-negative weights.
//!
//! Implemented via shifted power iteration on `(M + σI)`:
//! `x_new[v] = σ·x_old[v] + Σ_{u ~ v} m_{u,v}·x_old[u]`, then normalize
//! so `max|x| == 1`. The shift breaks the `±λ` bipartite symmetry of
//! symmetric matrices and turns the "largest-real-part" eigenvalue of
//! `M` into the unique "largest-magnitude" eigenvalue of `M + σI`, so
//! plain power iteration converges to it for non-negative `M`. Iterate
//! until L1 change < `1e-12` or 5000 iterations.
//!
//! ## Variants
//!
//! - [`eigenvector_centrality`] — undirected, unweighted; backward
//!   compatible (returns `Vec<f64>` for the vector only).
//! - [`eigenvector_centrality_weighted`] — undirected, weighted.
//! - [`eigenvector_centrality_directed`] — directed, unweighted.
//! - [`eigenvector_centrality_directed_weighted`] — directed, weighted.
//! - [`eigenvector_centrality_full`] — single-entry master function
//!   matching upstream's signature `(g, mode, weights?) -> {vec, λ}`.
//!
//! ## Directed-mode convention
//!
//! For mode = [`EigenvectorMode::Out`] (the standard convention; matches
//! upstream's `IGRAPH_OUT`), the centrality of `v` is proportional to
//! the sum of centralities of vertices pointing into `v`, i.e. we walk
//! `v`'s incoming edges. For [`EigenvectorMode::In`] we walk outgoing
//! edges. [`EigenvectorMode::All`] treats the graph as undirected and
//! falls through to the symmetric path.
//!
//! ## DAG short-circuit
//!
//! When the graph is a DAG and weights are non-negative, the adjacency
//! matrix is nilpotent and every eigenvalue is zero. ARPACK behaviour
//! is then non-deterministic; matching upstream, we return `eigenvalue
//! = 0` and a vector of `1`s on sinks (for [`EigenvectorMode::Out`]) /
//! sources (for [`EigenvectorMode::In`]) and `0`s elsewhere. See
//! <https://github.com/igraph/igraph/issues/2679> for upstream
//! rationale.

use crate::core::{Graph, IgraphError, IgraphResult};

// Tighter than the L1-convergence default we use elsewhere because
// rustdoc/conformance compare bit-precise vs python-igraph's ARPACK.
const DEFAULT_EPS: f64 = 1e-14;
const DEFAULT_MAX_ITER: usize = 5000;

// Tolerance for the shifted-power-iter (slightly looser since we run on
// `(M + σI)` and need to recover λ ≈ ρ - σ which loses ~log10(σ/ρ) bits).
const SHIFTED_EPS: f64 = 1e-12;
const SHIFTED_MAX_ITER: usize = 5000;

/// How to consider edge directions for [`eigenvector_centrality_full`]
/// and friends. Mirrors upstream's `IGRAPH_OUT` / `IGRAPH_IN` /
/// `IGRAPH_ALL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EigenvectorMode {
    /// Standard convention: centrality of `v` is proportional to the
    /// sum of centralities of vertices pointing to `v` (we walk
    /// incoming edges of `v`). Equivalent to ARPACK's "left
    /// eigenvector of the adjacency matrix".
    Out,
    /// Reverse convention: centrality of `v` is proportional to the
    /// sum of centralities of vertices `v` points to (we walk outgoing
    /// edges of `v`). Equivalent to ARPACK's "right eigenvector".
    In,
    /// Treat the graph as undirected and use the symmetric eigenvector
    /// path. Mandatory for undirected input.
    All,
}

/// Output of [`eigenvector_centrality_full`]: the normalised
/// centrality vector and the dominant eigenvalue of the (possibly
/// shifted-back) adjacency matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct EigenvectorScores {
    /// Centrality per vertex, length `vcount()`. Max-absolute element
    /// is exactly `1.0` (matching python-igraph), unless all entries
    /// are zero (empty / DAG / all-zero-weight sentinels).
    pub vector: Vec<f64>,
    /// Largest-real-part eigenvalue of the adjacency / weighted
    /// adjacency matrix. `0.0` for the empty-edge, all-zero-weight,
    /// and DAG sentinel cases.
    pub eigenvalue: f64,
}

/// Backward-compatible undirected, unweighted entry point.
///
/// Returns just the centrality vector (max-1 normalised). Directed
/// graphs return [`IgraphError::Unsupported`] — use
/// [`eigenvector_centrality_directed`] or [`eigenvector_centrality_full`]
/// for the directed paths.
///
/// Counterpart of `igraph_eigenvector_centrality(g, &v, NULL, IGRAPH_ALL,
/// NULL, NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigenvector_centrality};
///
/// // Triangle: every vertex has identical centrality 1.0.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let ec = eigenvector_centrality(&g).unwrap();
/// assert!((ec[0] - 1.0).abs() < 1e-9);
/// assert!((ec[1] - 1.0).abs() < 1e-9);
/// assert!((ec[2] - 1.0).abs() < 1e-9);
/// ```
pub fn eigenvector_centrality(graph: &Graph) -> IgraphResult<Vec<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed eigenvector_centrality — use eigenvector_centrality_directed or _full",
        ));
    }
    undirected_unweighted(graph).map(|s| s.vector)
}

/// Undirected weighted eigenvector centrality.
///
/// `weights.len()` must equal `graph.ecount()`. Returns the
/// max-1-normalised vector and the dominant eigenvalue of the weighted
/// symmetric adjacency `W[i,j] = Σ_{e: i~j} w_e`. Non-negative weights
/// are assumed (negative weights are accepted but a Perron-Frobenius
/// guarantee no longer applies and entries may be negative; matches
/// upstream).
///
/// All-zero weights and edge-less graphs both return a vector of `1`s
/// and `eigenvalue = 0`, matching upstream's sentinel.
///
/// Counterpart of `igraph_eigenvector_centrality(g, &v, &λ, IGRAPH_ALL,
/// weights, NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigenvector_centrality_weighted};
///
/// // Weighted star K_{1,4} with unit weights → leaf:centre = 1:2.
/// let mut g = Graph::with_vertices(5);
/// for v in 1..5 {
///     g.add_edge(0, v).unwrap();
/// }
/// let s = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
/// assert!((s.vector[0] - 1.0).abs() < 1e-9);
/// assert!((s.vector[1] - 0.5).abs() < 1e-9);
/// assert!((s.eigenvalue - 2.0).abs() < 1e-6);
/// ```
pub fn eigenvector_centrality_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<EigenvectorScores> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed eigenvector_centrality_weighted — use eigenvector_centrality_directed_weighted",
        ));
    }
    validate_weights(graph, weights)?;
    undirected_weighted(graph, Some(weights))
}

/// Directed unweighted eigenvector centrality.
///
/// `mode` controls which side of the adjacency matrix's left/right
/// eigenvector is returned. Undirected input is rejected — use
/// [`eigenvector_centrality`] instead.
///
/// Counterpart of `igraph_eigenvector_centrality(g, &v, &λ, mode, NULL,
/// NULL)` on directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, EigenvectorMode, eigenvector_centrality_directed};
///
/// // Directed 4-cycle 0→1→2→3→0 plus chord 1→3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (1, 3)]).unwrap();
/// let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
/// // Eigenvalue ≈ 1.22074 (real root of x³ = 1 + x²).
/// assert!((s.eigenvalue - 1.22074).abs() < 1e-3);
/// ```
pub fn eigenvector_centrality_directed(
    graph: &Graph,
    mode: EigenvectorMode,
) -> IgraphResult<EigenvectorScores> {
    eigenvector_centrality_full(graph, mode, None)
}

/// Directed weighted eigenvector centrality.
///
/// Like [`eigenvector_centrality_directed`] but with per-edge weights.
/// Weights of parallel edges are summed (`W[i,j] = Σ_{e: i→j} w_e`).
///
/// Counterpart of `igraph_eigenvector_centrality(g, &v, &λ, mode,
/// weights, NULL)` on directed graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigenvector_centrality_directed_weighted, EigenvectorMode};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = eigenvector_centrality_directed_weighted(&g, EigenvectorMode::All, &[1.0, 1.0, 1.0]).unwrap();
/// assert_eq!(r.vector.len(), 3);
/// ```
pub fn eigenvector_centrality_directed_weighted(
    graph: &Graph,
    mode: EigenvectorMode,
    weights: &[f64],
) -> IgraphResult<EigenvectorScores> {
    eigenvector_centrality_full(graph, mode, Some(weights))
}

/// Master entry point matching upstream's signature.
///
/// Dispatches to the undirected / directed and weighted / unweighted
/// branches based on `graph.is_directed()`, `mode`, and `weights`.
/// Validates `weights.len() == graph.ecount()` when supplied.
///
/// Counterpart of `igraph_eigenvector_centrality(g, &v, &λ, mode,
/// weights, NULL)` — the C-level "do everything" signature.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, eigenvector_centrality_full, EigenvectorMode};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = eigenvector_centrality_full(&g, EigenvectorMode::All, None).unwrap();
/// assert_eq!(r.vector.len(), 3);
/// ```
pub fn eigenvector_centrality_full(
    graph: &Graph,
    mode: EigenvectorMode,
    weights: Option<&[f64]>,
) -> IgraphResult<EigenvectorScores> {
    if let Some(w) = weights {
        validate_weights(graph, w)?;
    }
    // Undirected input forces ALL mode (matches upstream).
    let effective_mode = if graph.is_directed() {
        mode
    } else {
        EigenvectorMode::All
    };
    match effective_mode {
        EigenvectorMode::All => match weights {
            None => undirected_unweighted(graph),
            Some(w) => undirected_weighted(graph, Some(w)),
        },
        EigenvectorMode::Out | EigenvectorMode::In => directed_path(graph, effective_mode, weights),
    }
}

// ---------- internal: shared validation ----------

fn validate_weights(graph: &Graph, weights: &[f64]) -> IgraphResult<()> {
    let m = graph.ecount();
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector length ({}) not equal to number of edges ({})",
            weights.len(),
            m
        )));
    }
    Ok(())
}

// ---------- internal: undirected unweighted ----------

fn undirected_unweighted(graph: &Graph) -> IgraphResult<EigenvectorScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(EigenvectorScores {
            vector: Vec::new(),
            eigenvalue: 0.0,
        });
    }
    if graph.ecount() == 0 {
        return Ok(EigenvectorScores {
            vector: vec![1.0; n_us],
            eigenvalue: 0.0,
        });
    }
    if n == 1 {
        return Ok(EigenvectorScores {
            vector: vec![1.0],
            eigenvalue: 0.0,
        });
    }

    let mut adj: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        adj.push(graph.neighbors(v)?);
    }

    let mut x = vec![1.0_f64; n_us];
    let mut x_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        // x_new = (A + I) · x — shift breaks ±λ bipartite symmetry.
        for v in 0..n_us {
            let mut sum = x[v];
            for &u in &adj[v] {
                sum += x[u as usize];
            }
            x_new[v] = sum;
        }
        let max = x_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if max > 0.0 {
            for slot in &mut x_new {
                *slot /= max;
            }
        }
        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (x_new[v] - x[v]).abs();
        }
        std::mem::swap(&mut x, &mut x_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    // Eigenvalue from Rayleigh quotient: λ = (xᵀ·A·x) / (xᵀ·x).
    let mut ax = vec![0.0_f64; n_us];
    for v in 0..n_us {
        let mut sum = 0.0_f64;
        for &u in &adj[v] {
            sum += x[u as usize];
        }
        ax[v] = sum;
    }
    let num: f64 = x.iter().zip(ax.iter()).map(|(a, b)| a * b).sum();
    let den: f64 = x.iter().map(|a| a * a).sum();
    let eigenvalue = if den > 0.0 { num / den } else { 0.0 };

    // Sign cleanup: drop tiny negatives that come from numerical drift
    // (only valid because A here is non-negative).
    for slot in &mut x {
        if *slot < 0.0 {
            *slot = 0.0;
        }
    }

    Ok(EigenvectorScores {
        vector: x,
        eigenvalue,
    })
}

// ---------- internal: undirected weighted ----------

#[allow(
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::cast_possible_truncation
)]
fn undirected_weighted(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<EigenvectorScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    if n == 0 {
        return Ok(EigenvectorScores {
            vector: Vec::new(),
            eigenvalue: 0.0,
        });
    }
    if m == 0 {
        return Ok(EigenvectorScores {
            vector: vec![1.0; n_us],
            eigenvalue: 0.0,
        });
    }
    if let Some(w) = weights {
        if w.iter().all(|&x| x == 0.0) {
            return Ok(EigenvectorScores {
                vector: vec![1.0; n_us],
                eigenvalue: 0.0,
            });
        }
    }
    if n == 1 {
        return Ok(EigenvectorScores {
            vector: vec![1.0],
            eigenvalue: 0.0,
        });
    }

    let negative_weights = weights.is_some_and(|w| w.iter().any(|&x| x < 0.0));

    // Pre-cache incident edges per vertex (undirected → both endpoints
    // appear in their respective incidence lists).
    let mut inc: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        inc.push(graph.incident(v)?);
    }

    // Seed: weighted strength fallback to degree if zero strength (matches C).
    let mut x = vec![0.0_f64; n_us];
    for v in 0..n {
        let v_us = v as usize;
        let mut strength = 0.0_f64;
        for &e in &inc[v_us] {
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            strength += w;
        }
        if strength != 0.0 {
            x[v_us] = strength.abs();
        } else if !negative_weights {
            x[v_us] = 0.0;
        } else {
            // Negative weights can produce zero strength even on
            // non-zero-degree vertices; seed those with 1.
            x[v_us] = if inc[v_us].is_empty() { 0.0 } else { 1.0 };
        }
    }
    // Make sure at least one entry is non-zero so power iter has signal.
    if x.iter().all(|&y| y == 0.0) {
        x.fill(1.0);
    }
    // Normalise seed to max=1 to keep numerics tame.
    let max0 = x.iter().fold(0.0_f64, |acc, &y| acc.max(y.abs()));
    if max0 > 0.0 {
        for slot in &mut x {
            *slot /= max0;
        }
    }

    // Shifted power iter on (W + σI), σ = 1 suffices since W is bounded
    // and we just need to break ±λ symmetry. For negative weights, this
    // doesn't guarantee convergence to largest-real, but matches the
    // upstream "best effort" contract.
    let shift = 1.0_f64;
    let mut x_new = vec![0.0_f64; n_us];

    for _ in 0..SHIFTED_MAX_ITER {
        // x_new = σ·x + W·x  (W·x walks incident edges with weight w_e)
        for v in 0..n_us {
            let mut sum = shift * x[v];
            for &e in &inc[v] {
                let other = graph.edge_other(e, v as u32)?;
                let w = weights.map_or(1.0, |ws| ws[e as usize]);
                sum += w * x[other as usize];
            }
            x_new[v] = sum;
        }
        // For negative-weight case, track signed component with greatest
        // magnitude so we converge to largest real eigenvalue, not just
        // largest magnitude. For positive weights this is equivalent to
        // max abs.
        let pivot = if negative_weights {
            let mut best = 0.0_f64;
            for &v in &x_new {
                if v.abs() > best.abs() {
                    best = v;
                }
            }
            best
        } else {
            x_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()))
        };
        if pivot != 0.0 {
            for slot in &mut x_new {
                *slot /= pivot;
            }
        }
        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (x_new[v] - x[v]).abs();
        }
        std::mem::swap(&mut x, &mut x_new);
        if diff < SHIFTED_EPS {
            break;
        }
    }

    // Rayleigh quotient on the original W (no shift): λ = xᵀ·W·x / xᵀ·x
    let mut wx = vec![0.0_f64; n_us];
    for v in 0..n_us {
        let mut sum = 0.0_f64;
        for &e in &inc[v] {
            let other = graph.edge_other(e, v as u32)?;
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            sum += w * x[other as usize];
        }
        wx[v] = sum;
    }
    let num: f64 = x.iter().zip(wx.iter()).map(|(a, b)| a * b).sum();
    let den: f64 = x.iter().map(|a| a * a).sum();
    let eigenvalue = if den > 0.0 { num / den } else { 0.0 };

    // Renormalise to max-abs = 1 (Rayleigh-quotient pass may have
    // produced a vector whose dominant component is signed).
    let max = x.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
    if max > 0.0 {
        for slot in &mut x {
            *slot /= max;
        }
    }
    if !negative_weights {
        for slot in &mut x {
            if *slot < 0.0 {
                *slot = 0.0;
            }
        }
    }

    Ok(EigenvectorScores {
        vector: x,
        eigenvalue,
    })
}

// ---------- internal: directed (with optional weights) ----------

#[allow(
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::needless_range_loop
)]
fn directed_path(
    graph: &Graph,
    mode: EigenvectorMode,
    weights: Option<&[f64]>,
) -> IgraphResult<EigenvectorScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    if n == 0 {
        return Ok(EigenvectorScores {
            vector: Vec::new(),
            eigenvalue: 0.0,
        });
    }
    if m == 0 {
        return Ok(EigenvectorScores {
            vector: vec![1.0; n_us],
            eigenvalue: 0.0,
        });
    }
    if let Some(w) = weights {
        if w.iter().all(|&x| x == 0.0) {
            return Ok(EigenvectorScores {
                vector: vec![1.0; n_us],
                eigenvalue: 0.0,
            });
        }
    }

    let negative_weights = weights.is_some_and(|w| w.iter().any(|&x| x < 0.0));

    // DAG short-circuit (only valid for non-negative weights — see
    // upstream comment block on lines 350-369). Returns 1s on sinks
    // (Out) / sources (In) and 0s elsewhere.
    if !negative_weights && crate::algorithms::properties::is_dag::is_dag(graph) {
        return dag_sentinel(graph, mode, weights);
    }

    // For Out: centrality of v ∝ Σ_{u: u→v} c_u, so we walk IN-edges of v.
    // For In:  centrality of v ∝ Σ_{u: v→u} c_u, so we walk OUT-edges of v.
    // Pre-build per-vertex (edge_id, neighbour) list.
    let mut walk: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n_us];
    let m_u32 = u32::try_from(m)
        .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32::MAX".into()))?;
    match mode {
        EigenvectorMode::Out => {
            for e in 0..m_u32 {
                let (u, v) = graph.edge(e)?;
                walk[v as usize].push((e, u));
            }
        }
        EigenvectorMode::In => {
            for e in 0..m_u32 {
                let (u, v) = graph.edge(e)?;
                walk[u as usize].push((e, v));
            }
        }
        EigenvectorMode::All => unreachable!("All is dispatched to undirected_*"),
    }

    // Seed: in-strength (Out) / out-strength (In). Fallback to in-degree
    // when negative weights produce zero strength on a non-empty list.
    let mut x = vec![0.0_f64; n_us];
    for v in 0..n_us {
        let mut strength = 0.0_f64;
        for &(e, _) in &walk[v] {
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            strength += w;
        }
        if strength != 0.0 {
            x[v] = strength.abs();
        } else if !negative_weights {
            x[v] = 0.0;
        } else {
            x[v] = if walk[v].is_empty() { 0.0 } else { 1.0 };
        }
    }
    if x.iter().all(|&y| y == 0.0) {
        x.fill(1.0);
    }
    let max0 = x.iter().fold(0.0_f64, |acc, &y| acc.max(y.abs()));
    if max0 > 0.0 {
        for slot in &mut x {
            *slot /= max0;
        }
    }

    // Shift: σ = max absolute row sum of M ensures (M + σI) has its
    // largest-magnitude eigenvalue equal to largest-real-part of M + σ,
    // because for non-negative M the largest-real eigenvalue is real
    // and non-negative (Perron-Frobenius). We pad by 1 to also break
    // exact tie configurations.
    let mut row_norm: f64 = 0.0;
    for v in 0..n_us {
        let mut s = 0.0_f64;
        for &(e, _) in &walk[v] {
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            s += w.abs();
        }
        if s > row_norm {
            row_norm = s;
        }
    }
    let shift = row_norm.max(1.0) + 1.0;

    let mut x_new = vec![0.0_f64; n_us];
    for _ in 0..SHIFTED_MAX_ITER {
        for v in 0..n_us {
            let mut sum = shift * x[v];
            for &(e, nei) in &walk[v] {
                let w = weights.map_or(1.0, |ws| ws[e as usize]);
                sum += w * x[nei as usize];
            }
            x_new[v] = sum;
        }
        let pivot = if negative_weights {
            let mut best = 0.0_f64;
            for &v in &x_new {
                if v.abs() > best.abs() {
                    best = v;
                }
            }
            best
        } else {
            x_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()))
        };
        if pivot != 0.0 {
            for slot in &mut x_new {
                *slot /= pivot;
            }
        }
        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (x_new[v] - x[v]).abs();
        }
        std::mem::swap(&mut x, &mut x_new);
        if diff < SHIFTED_EPS {
            break;
        }
    }

    // Rayleigh quotient on unshifted M.
    let mut mx = vec![0.0_f64; n_us];
    for v in 0..n_us {
        let mut sum = 0.0_f64;
        for &(e, nei) in &walk[v] {
            let w = weights.map_or(1.0, |ws| ws[e as usize]);
            sum += w * x[nei as usize];
        }
        mx[v] = sum;
    }
    let num: f64 = x.iter().zip(mx.iter()).map(|(a, b)| a * b).sum();
    let den: f64 = x.iter().map(|a| a * a).sum();
    let mut eigenvalue = if den > 0.0 { num / den } else { 0.0 };

    // Pathological case: dominant eigenvalue is zero (i.e. M is
    // nilpotent yet we slipped past the DAG fast-path due to numerical
    // noise). Fall back to upstream's "all zeros" output.
    if !negative_weights && eigenvalue <= SHIFTED_EPS {
        eigenvalue = 0.0;
        x.fill(0.0);
    } else {
        let max = x.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if max > 0.0 {
            for slot in &mut x {
                *slot /= max;
            }
        }
        if !negative_weights {
            for slot in &mut x {
                if *slot < 0.0 {
                    *slot = 0.0;
                }
            }
        }
    }

    Ok(EigenvectorScores {
        vector: x,
        eigenvalue,
    })
}

// DAG sentinel: 1s on sinks (Out mode) / sources (In mode), 0s elsewhere.
#[allow(clippy::many_single_char_names)]
fn dag_sentinel(
    graph: &Graph,
    mode: EigenvectorMode,
    weights: Option<&[f64]>,
) -> IgraphResult<EigenvectorScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();
    // Compute the strength in the "outgoing" direction (Out → vertex
    // with no outgoing edges; In → vertex with no incoming edges).
    // Upstream's logic: sinks for Out mode = vertices where out-strength
    // is zero.
    let mut strength = vec![0.0_f64; n_us];
    let m_u32 = u32::try_from(m)
        .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32::MAX".into()))?;
    for e in 0..m_u32 {
        let (u, v) = graph.edge(e)?;
        let w = weights.map_or(1.0, |ws| ws[e as usize]);
        match mode {
            EigenvectorMode::Out => strength[u as usize] += w,
            EigenvectorMode::In => strength[v as usize] += w,
            EigenvectorMode::All => unreachable!(),
        }
    }
    let vector: Vec<f64> = strength
        .iter()
        .map(|&s| if s == 0.0 { 1.0 } else { 0.0 })
        .collect();
    Ok(EigenvectorScores {
        vector,
        eigenvalue: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "vertex {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(eigenvector_centrality(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(eigenvector_centrality(&g).unwrap(), vec![1.0]);
    }

    #[test]
    fn isolated_vertices_yield_uniform_one() {
        let g = Graph::with_vertices(3);
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn triangle_uniform_one() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn star_4_centre_one_leaves_inverse_sqrt_three() {
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let ec = eigenvector_centrality(&g).unwrap();
        let inv_sqrt3 = 1.0 / 3f64.sqrt();
        close(&ec, &[1.0, inv_sqrt3, inv_sqrt3, inv_sqrt3], 1e-9);
    }

    #[test]
    fn k4_uniform_one() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let ec = eigenvector_centrality(&g).unwrap();
        close(&ec, &[1.0, 1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn directed_graph_returns_unsupported() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(eigenvector_centrality(&g).is_err());
    }

    // ---- weighted undirected ----

    #[test]
    fn weighted_star_unit_matches_unweighted() {
        let mut g = Graph::with_vertices(5);
        for v in 1..5 {
            g.add_edge(0, v).unwrap();
        }
        let s = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
        close(&s.vector, &[1.0, 0.5, 0.5, 0.5, 0.5], 1e-9);
        assert!((s.eigenvalue - 2.0).abs() < 1e-6);
    }

    #[test]
    fn weighted_k5_unit_eigenvalue_four() {
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        let s = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
        close(&s.vector, &[1.0, 1.0, 1.0, 1.0, 1.0], 1e-9);
        assert!((s.eigenvalue - 4.0).abs() < 1e-6);
    }

    #[test]
    fn weighted_empty_returns_ones() {
        let g = Graph::with_vertices(4);
        let s = eigenvector_centrality_weighted(&g, &[]).unwrap();
        close(&s.vector, &[1.0; 4], 1e-15);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_all_zero_returns_ones() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let s = eigenvector_centrality_weighted(&g, &[0.0, 0.0]).unwrap();
        close(&s.vector, &[1.0; 3], 1e-15);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_length_mismatch_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(eigenvector_centrality_weighted(&g, &[1.0]).is_err());
    }

    // ---- directed ----

    #[test]
    fn directed_full_eigenvalue_n_minus_one() {
        // K_5 directed (no loops) has dominant eigenvalue n-1=4.
        let mut g = Graph::new(5, true).unwrap();
        for u in 0..5u32 {
            for v in 0..5u32 {
                if u != v {
                    g.add_edge(u, v).unwrap();
                }
            }
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        close(&s.vector, &[1.0, 1.0, 1.0, 1.0, 1.0], 1e-9);
        assert!((s.eigenvalue - 4.0).abs() < 1e-6);
    }

    #[test]
    fn directed_dag_out_star_returns_leaves_one() {
        // out-star: 0 → 1, 2, 3, 4. Sinks (Out mode) are 1..4.
        let mut g = Graph::new(5, true).unwrap();
        for v in 1..5u32 {
            g.add_edge(0, v).unwrap();
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        close(&s.vector, &[0.0, 1.0, 1.0, 1.0, 1.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn directed_dag_in_star_in_mode_marks_leaves() {
        // In-star: 1, 2, 3, 4 → 0. With In mode (centrality ∝ vertices
        // we point TO), upstream's DAG sentinel marks sources of the
        // adjacency matrix — vertices with no incoming edges, i.e. the
        // leaves 1..4. Vertex 0 has 4 incoming edges, no outgoing, so
        // it gets 0. Symmetric with the out-star + Out-mode case.
        let mut g = Graph::new(5, true).unwrap();
        for v in 1..5u32 {
            g.add_edge(v, 0).unwrap();
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::In).unwrap();
        close(&s.vector, &[0.0, 1.0, 1.0, 1.0, 1.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn directed_dag_in_star_out_mode_marks_centre() {
        // In-star: 1, 2, 3, 4 → 0. With default Out mode, upstream
        // marks SINKS (no outgoing edges) = vertex 0 only. Matches the
        // golden case at references/igraph/tests/unit/igraph_eigenvector_centrality.out.
        let mut g = Graph::new(5, true).unwrap();
        for v in 1..5u32 {
            g.add_edge(v, 0).unwrap();
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        close(&s.vector, &[1.0, 0.0, 0.0, 0.0, 0.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn directed_small_cycle_chord_eigenvalue_matches_arpack() {
        // 0→1, 1→2, 2→3, 3→0, 1→3 — from upstream's golden test.
        // Expected eigenvalue ≈ 1.22074 (real root of x³ = 1 + x²).
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (1, 3)])
            .unwrap();
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        assert!(
            (s.eigenvalue - 1.220_744).abs() < 1e-3,
            "expected ≈ 1.22074, got {}",
            s.eigenvalue
        );
        // Vertex 3 should be most central (max-1 by construction).
        let max_idx = s
            .vector
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(max_idx, 3);
    }

    #[test]
    fn directed_empty_directed_returns_ones() {
        let g = Graph::new(5, true).unwrap();
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        close(&s.vector, &[1.0; 5], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    // ---- master function dispatch ----

    #[test]
    fn full_undirected_with_all_mode_works() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let s = eigenvector_centrality_full(&g, EigenvectorMode::All, None).unwrap();
        close(&s.vector, &[1.0, 1.0, 1.0], 1e-9);
    }

    #[test]
    fn full_undirected_forces_all_mode() {
        // Passing Out on undirected graph still hits the symmetric path.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let a = eigenvector_centrality_full(&g, EigenvectorMode::Out, None).unwrap();
        let b = eigenvector_centrality_full(&g, EigenvectorMode::All, None).unwrap();
        close(&a.vector, &b.vector, 1e-12);
        assert!((a.eigenvalue - b.eigenvalue).abs() < 1e-12);
    }

    // ---- stress / hard spectrum cases ----

    #[test]
    fn directed_cycle_50_converges_to_unit_eigenvalue() {
        // 0→1→2→...→49→0. Adjacency eigenvalues are 50th roots of
        // unity; largest-real = 1 (only λ=1 itself is real). With our
        // shift σ = max_row_norm+1 = 2, shifted eigenvalues are
        // ω_k + 2, of which |ω_0 + 2| = 3 is the strict maximum.
        // Power iter should converge in O(1) iterations.
        let n: u32 = 50;
        let mut g = Graph::new(n, true).unwrap();
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).unwrap();
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        assert!(
            (s.eigenvalue - 1.0).abs() < 1e-9,
            "expected λ ≈ 1.0, got {}",
            s.eigenvalue
        );
        // Circulant graph: vector should be constant.
        let mx = s.vector.iter().copied().fold(0.0_f64, f64::max);
        let mn = s.vector.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(mx - mn < 1e-9, "vec spread {} too large", mx - mn);
    }

    #[test]
    fn directed_k10_eigenvalue_n_minus_one() {
        // Complete digraph K_10 (no loops): eigenvalue = n - 1 = 9,
        // uniform vector.
        let n: u32 = 10;
        let mut g = Graph::new(n, true).unwrap();
        for u in 0..n {
            for v in 0..n {
                if u != v {
                    g.add_edge(u, v).unwrap();
                }
            }
        }
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        assert!(
            (s.eigenvalue - 9.0).abs() < 1e-9,
            "expected λ = 9, got {}",
            s.eigenvalue
        );
        close(&s.vector, &[1.0; 10], 1e-9);
    }

    #[test]
    fn directed_cycle_chord_eigenvector_components_match_arpack() {
        // Upstream golden: cycle 4 + chord 1→3, mode = Out.
        // Vector (max-1) = [0.819173, 0.671044, 0.549700, 1.000000]
        // λ ≈ 1.220744.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (1, 3)])
            .unwrap();
        let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
        assert!((s.eigenvalue - 1.220_744).abs() < 1e-4);
        close(
            &s.vector,
            &[0.819_173, 0.671_044, 0.549_700, 1.000_000],
            1e-3,
        );
    }
}
