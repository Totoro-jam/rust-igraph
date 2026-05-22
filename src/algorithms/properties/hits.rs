//! Kleinberg's hub and authority scores — HITS (ALGO-PR-017 / PR-017b).
//!
//! Counterpart of `igraph_hub_and_authority_scores()` from
//! `references/igraph/src/centrality/hub_authority.c`. The hub score
//! `h[v]` is the `v`-th component of the principal eigenvector of
//! `A·Aᵀ`; the authority score `a[v]` is the principal eigenvector of
//! `Aᵀ·A`. The two are tied by `a = Aᵀ·h` and `h = A·a` once both have
//! converged.
//!
//! ## Unweighted ([`hub_and_authority_scores`])
//! - **Directed**: power iteration on `A·Aᵀ` (symmetric PSD ⇒ no
//!   bipartite ±λ shift trick needed; non-negative initial vector
//!   preserves Perron-Frobenius non-negativity).
//! - **Undirected**: delegate to [`eigenvector_centrality`], matching
//!   upstream's documented behaviour ("In undirected graphs, both the
//!   hub and authority scores are equal to the eigenvector
//!   centrality"). The reported eigenvalue is `λ²` because if
//!   `A·v = λ·v` then `A²·v = λ²·v`.
//! - **Empty edges**: vectors filled with `1.0`, eigenvalue `0`,
//!   matching upstream.
//!
//! ## Weighted ([`hub_and_authority_scores_weighted`])
//! - **Directed**: same power iteration, with weighted matrix
//!   `W[i,j] = Σ_{e: i→j} w_e`. Two-step `tmp = Wᵀ·h`, `h_new = W·tmp`,
//!   walking edge-incidence lists rather than vertex-neighbour lists.
//! - **Undirected**: shifted power iteration on `(W + I)` where
//!   `W` is the symmetric weighted adjacency. Self-rolled (avoids
//!   waiting on PR-012b's weighted eigenvector centrality); reported
//!   eigenvalue is squared per upstream convention.
//! - **Negative weights**: allowed but a warning is appropriate
//!   (Perron-Frobenius doesn't apply, sign of entries is no longer
//!   guaranteed). We don't zero-clip negatives in this case.
//! - **Length / all-zero / empty-edge** validation mirrors upstream.
//!
//! ARPACK backend ships later (PR-017c) once the LA-IRLM scaffolding
//! lands, paralleling PR-011c.

use crate::algorithms::properties::eigenvector::eigenvector_centrality;
use crate::core::{Graph, IgraphError, IgraphResult};

const DEFAULT_EPS: f64 = 1e-12;
const DEFAULT_MAX_ITER: usize = 5000;

/// Output of [`hub_and_authority_scores`]: scaled hub and authority
/// vectors and the dominant eigenvalue of `A·Aᵀ`.
///
/// Both vectors are normalised so that their max-absolute element is
/// exactly `1.0`, matching python-igraph's reporting convention.
#[derive(Debug, Clone, PartialEq)]
pub struct HitsScores {
    /// Hub score per vertex, length `vcount()`. Max-absolute element is `1.0`.
    pub hub: Vec<f64>,
    /// Authority score per vertex, length `vcount()`. Max-absolute element is `1.0`.
    pub authority: Vec<f64>,
    /// Dominant eigenvalue of `A·Aᵀ` (= square of dominant `A`-eigenvalue
    /// for the undirected delegation path). Returned as `0.0` for the
    /// empty-edge case.
    pub eigenvalue: f64,
}

/// Compute Kleinberg's hub and authority scores.
///
/// Returns `Ok(HitsScores)` containing both vectors and the dominant
/// eigenvalue of `A·Aᵀ`. The empty graph yields empty vectors and a
/// `0.0` eigenvalue.
///
/// On undirected graphs the routine delegates to
/// [`eigenvector_centrality`], so `hub == authority` exactly and the
/// reported `eigenvalue` is `λ²` (the square of the dominant
/// adjacency-matrix eigenvalue).
///
/// Counterpart of `igraph_hub_and_authority_scores(g, h, a, &val,
/// /*weights=*/NULL, /*options=*/NULL)` from `references/igraph/src/centrality/hub_authority.c`.
///
/// # Examples
///
/// Pure hub / authority partition on a bipartite directed graph:
///
/// ```
/// use rust_igraph::{Graph, hub_and_authority_scores};
///
/// // 0,1 → 2,3 — vertices 0,1 are pure hubs, 2,3 pure authorities.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 2u32), (0, 3), (1, 2), (1, 3)]).unwrap();
/// let s = hub_and_authority_scores(&g).unwrap();
/// assert!((s.hub[0] - 1.0).abs() < 1e-9);
/// assert!((s.hub[1] - 1.0).abs() < 1e-9);
/// assert!(s.hub[2].abs() < 1e-9);
/// assert!(s.hub[3].abs() < 1e-9);
/// assert!((s.authority[2] - 1.0).abs() < 1e-9);
/// assert!((s.authority[3] - 1.0).abs() < 1e-9);
/// // Largest eigenvalue of A·Aᵀ for this 2x2 hub-authority block is 4.
/// assert!((s.eigenvalue - 4.0).abs() < 1e-6);
/// ```
///
/// Cross-relation invariant `h ∝ A·a` after convergence:
///
/// ```
/// use rust_igraph::{Graph, hub_and_authority_scores};
///
/// let mut g = Graph::new(5, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (0, 3), (1, 2), (1, 3), (2, 0), (2, 4), (3, 2), (4, 0), (4, 1)])
///     .unwrap();
/// let s = hub_and_authority_scores(&g).unwrap();
///
/// // Compute A·authority by walking the edge list, then verify it lines
/// // up with hub after max-norming.
/// let n = g.vcount() as usize;
/// let mut a_auth = vec![0.0_f64; n];
/// for e in 0..g.ecount() {
///     let (u, v) = g.edge(e as u32).unwrap();
///     a_auth[u as usize] += s.authority[v as usize];
/// }
/// let max = a_auth.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
/// for x in &mut a_auth {
///     *x /= max;
/// }
/// for u in 0..n {
///     assert!((a_auth[u] - s.hub[u]).abs() < 1e-6);
/// }
/// ```
pub fn hub_and_authority_scores(graph: &Graph) -> IgraphResult<HitsScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(HitsScores {
            hub: Vec::new(),
            authority: Vec::new(),
            eigenvalue: 0.0,
        });
    }

    // Undirected → eigenvector centrality (mode = ALL); hub = auth.
    if !graph.is_directed() {
        let ec = eigenvector_centrality(graph)?;
        let lambda = dominant_eigenvalue_undirected(graph, &ec);
        return Ok(HitsScores {
            hub: ec.clone(),
            authority: ec,
            eigenvalue: lambda * lambda,
        });
    }

    // Empty-edge directed graph → fill with 1.0, eigenvalue 0.
    if graph.ecount() == 0 {
        return Ok(HitsScores {
            hub: vec![1.0_f64; n_us],
            authority: vec![1.0_f64; n_us],
            eigenvalue: 0.0,
        });
    }

    // Pre-cache out- and in-neighbour lists; both are O(V + E).
    let mut out_adj: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    let mut in_adj: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        out_adj.push(graph.out_neighbors_vec(v)?);
        in_adj.push(graph.in_neighbors_vec(v)?);
    }

    // Seed the hub vector with out-degrees, mirroring upstream — this
    // is correlated with the dominant A·Aᵀ eigenvector and gives faster
    // convergence than a uniform start. Vertices with zero out-degree
    // (sinks) start at 0 and stay 0 through iteration: a sink can
    // never be a hub.
    #[allow(clippy::cast_precision_loss)]
    let mut h: Vec<f64> = out_adj.iter().map(|nei| nei.len() as f64).collect();
    // Normalise initial seed so the first iteration's eigenvalue
    // estimate is meaningful.
    rescale_max_abs(&mut h);
    let mut tmp = vec![0.0_f64; n_us];
    let mut h_new = vec![0.0_f64; n_us];

    let mut eigenvalue = 0.0_f64;
    for _ in 0..DEFAULT_MAX_ITER {
        // tmp = Aᵀ h  →  tmp[v] = Σ_{u ∈ in(v)} h[u].
        for v in 0..n_us {
            let mut s = 0.0_f64;
            for &u in &in_adj[v] {
                s += h[u as usize];
            }
            tmp[v] = s;
        }
        // h_new = A tmp  →  h_new[u] = Σ_{v ∈ out(u)} tmp[v].
        for u in 0..n_us {
            let mut s = 0.0_f64;
            for &v in &out_adj[u] {
                s += tmp[v as usize];
            }
            h_new[u] = s;
        }

        // Rayleigh-style estimate: with `max|h| = 1`, the unnormalised
        // `max|A·Aᵀ·h|` equals the dominant eigenvalue at convergence.
        let max = h_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()));
        if max > 0.0 {
            eigenvalue = max;
            for slot in &mut h_new {
                *slot /= max;
            }
        }

        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (h_new[v] - h[v]).abs();
        }
        std::mem::swap(&mut h, &mut h_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    // Eliminate -0.0 from numerical drift.
    for slot in &mut h {
        if *slot < 0.0 {
            *slot = 0.0;
        }
    }

    // authority = Aᵀ · h, then rescale.
    let mut authority = vec![0.0_f64; n_us];
    for v in 0..n_us {
        let mut s = 0.0_f64;
        for &u in &in_adj[v] {
            s += h[u as usize];
        }
        authority[v] = s;
    }
    rescale_max_abs(&mut authority);
    for slot in &mut authority {
        if *slot < 0.0 {
            *slot = 0.0;
        }
    }

    Ok(HitsScores {
        hub: h,
        authority,
        eigenvalue,
    })
}

/// Weighted Kleinberg hub and authority scores.
///
/// `weights[e]` is the weight of edge id `e`; must have length
/// `graph.ecount()` or this returns [`IgraphError::InvalidArgument`].
/// The weighted adjacency matrix is `W[i,j] = Σ_{e: i→j} w_e`; the
/// returned hub/authority vectors approximate the principal
/// eigenvectors of `W·Wᵀ` and `Wᵀ·W` respectively.
///
/// Behaviour mirrors upstream `igraph_hub_and_authority_scores` when
/// `weights` is non-NULL:
/// - Length must match `ecount()`.
/// - All-zero weights → both vectors filled with `1.0`, eigenvalue `0`.
/// - Empty edges → same as the unweighted empty case.
/// - Negative weights are *accepted*: the sign-cleanup pass that
///   normally zeros tiny negative drifts is skipped, since
///   Perron-Frobenius no longer guarantees a non-negative eigenvector.
/// - Undirected graphs delegate to a self-rolled shifted power
///   iteration on `(W + I)`, and the reported eigenvalue is `λ²`
///   (square of the dominant adjacency eigenvalue).
///
/// Counterpart of `igraph_hub_and_authority_scores(g, h, a, &val,
/// weights, /*options=*/NULL)` from
/// `references/igraph/src/centrality/hub_authority.c`.
///
/// # Examples
///
/// Weighted 2×2 bipartite block where weights tilt the authority:
///
/// ```
/// use rust_igraph::{Graph, hub_and_authority_scores_weighted};
///
/// // 0→2 (w=1), 0→3 (w=4), 1→2 (w=2), 1→3 (w=3).
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 2u32), (0, 3), (1, 2), (1, 3)]).unwrap();
/// let weights = vec![1.0, 4.0, 2.0, 3.0];
/// let s = hub_and_authority_scores_weighted(&g, &weights).unwrap();
/// // Both vertex 0 and 1 are hubs; vertex 2,3 are authorities.
/// assert!(s.hub[2].abs() < 1e-9);
/// assert!(s.hub[3].abs() < 1e-9);
/// assert!(s.authority[0].abs() < 1e-9);
/// assert!(s.authority[1].abs() < 1e-9);
/// // Authority 3 has heavier in-weight than authority 2.
/// assert!(s.authority[3] > s.authority[2]);
/// ```
#[allow(clippy::too_many_lines)] // Mirrors upstream's tightly-coupled two-step power iter.
pub fn hub_and_authority_scores_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<HitsScores> {
    let n = graph.vcount();
    let n_us = n as usize;
    let m = graph.ecount();

    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights length {} does not match edge count {}",
            weights.len(),
            m
        )));
    }

    if n == 0 {
        return Ok(HitsScores {
            hub: Vec::new(),
            authority: Vec::new(),
            eigenvalue: 0.0,
        });
    }

    if m == 0 {
        return Ok(HitsScores {
            hub: vec![1.0_f64; n_us],
            authority: vec![1.0_f64; n_us],
            eigenvalue: 0.0,
        });
    }

    let (min_w, max_w) = weights
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &w| {
            (lo.min(w), hi.max(w))
        });
    let negative_weights = min_w < 0.0;
    if min_w == 0.0 && max_w == 0.0 {
        return Ok(HitsScores {
            hub: vec![1.0_f64; n_us],
            authority: vec![1.0_f64; n_us],
            eigenvalue: 0.0,
        });
    }

    if !graph.is_directed() {
        let ec = weighted_undirected_eigenvector(graph, weights, negative_weights)?;
        let lambda = weighted_rayleigh_undirected(graph, weights, &ec);
        return Ok(HitsScores {
            hub: ec.clone(),
            authority: ec,
            eigenvalue: lambda * lambda,
        });
    }

    // Cache out- and in-incident edge ids; both are O(V + E).
    let mut out_inc: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    let mut in_inc: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        out_inc.push(graph.incident(v)?);
        in_inc.push(graph.incident_in(v)?);
    }

    // Seed with out-strengths; sinks (zero out-strength, non-negative
    // weights) stay 0. With negative weights, fall back to degree-based
    // sentinel to avoid spurious zeros.
    let mut h: Vec<f64> = (0..n_us)
        .map(|u| {
            let strength: f64 = out_inc[u].iter().map(|&e| weights[e as usize]).sum();
            if strength != 0.0 {
                strength
            } else if negative_weights && !out_inc[u].is_empty() {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    rescale_max_abs(&mut h);

    let mut tmp = vec![0.0_f64; n_us];
    let mut h_new = vec![0.0_f64; n_us];

    let mut eigenvalue = 0.0_f64;
    for _ in 0..DEFAULT_MAX_ITER {
        // tmp = Wᵀ h  →  tmp[v] = Σ_{e=(u→v)} w_e · h[u]
        for v in 0..n {
            let v_us = v as usize;
            let mut s = 0.0_f64;
            for &e in &in_inc[v_us] {
                let other = graph.edge_other(e, v)?;
                s += weights[e as usize] * h[other as usize];
            }
            tmp[v_us] = s;
        }
        // h_new = W tmp  →  h_new[u] = Σ_{e=(u→v)} w_e · tmp[v]
        for u in 0..n {
            let u_us = u as usize;
            let mut s = 0.0_f64;
            for &e in &out_inc[u_us] {
                let other = graph.edge_other(e, u)?;
                s += weights[e as usize] * tmp[other as usize];
            }
            h_new[u_us] = s;
        }

        // Dominant eigenvalue estimate: with max|h|=1, max|W·Wᵀ·h| ≈ λ
        // at convergence. For negative weights we track the signed
        // component of greatest magnitude so we converge on the
        // *largest* (not largest-magnitude) eigenvalue, matching the
        // C side's `which = 'LA'` setting.
        let scale = if negative_weights {
            let (mut best_mag, mut signed) = (0.0_f64, 0.0_f64);
            for &v in &h_new {
                let mag = v.abs();
                if mag > best_mag {
                    best_mag = mag;
                    signed = v;
                }
            }
            signed
        } else {
            h_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()))
        };
        if scale != 0.0 {
            eigenvalue = scale;
            for slot in &mut h_new {
                *slot /= scale;
            }
        }

        let mut diff = 0.0_f64;
        for v in 0..n_us {
            diff += (h_new[v] - h[v]).abs();
        }
        std::mem::swap(&mut h, &mut h_new);
        if diff < DEFAULT_EPS {
            break;
        }
    }

    if !negative_weights {
        for slot in &mut h {
            if *slot < 0.0 {
                *slot = 0.0;
            }
        }
    }

    // authority = Wᵀ · h, then rescale.
    let mut authority = vec![0.0_f64; n_us];
    for v in 0..n {
        let v_us = v as usize;
        let mut s = 0.0_f64;
        for &e in &in_inc[v_us] {
            let other = graph.edge_other(e, v)?;
            s += weights[e as usize] * h[other as usize];
        }
        authority[v_us] = s;
    }
    rescale_max_abs(&mut authority);
    if !negative_weights {
        for slot in &mut authority {
            if *slot < 0.0 {
                *slot = 0.0;
            }
        }
    }

    Ok(HitsScores {
        hub: h,
        authority,
        eigenvalue,
    })
}

/// Shifted power iteration on `(W + I)` for the symmetric weighted
/// adjacency `W`. Returns the max-norm-scaled principal eigenvector.
/// Stand-in for the not-yet-landed weighted
/// `eigenvector_centrality_weighted` (PR-012b); kept private so it can
/// later be promoted.
fn weighted_undirected_eigenvector(
    graph: &Graph,
    weights: &[f64],
    negative_weights: bool,
) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;

    let mut inc: Vec<Vec<u32>> = Vec::with_capacity(n_us);
    for v in 0..n {
        inc.push(graph.incident(v)?);
    }

    // Seed with vertex strengths (sum of incident-edge weights) so the
    // first iteration is already in the dominant subspace. Loops
    // contribute once (incident lists count them once on undirected
    // graphs in this codebase; mirror that).
    let mut x: Vec<f64> = (0..n_us)
        .map(|v| {
            let strength: f64 = inc[v].iter().map(|&e| weights[e as usize]).sum();
            if strength != 0.0 {
                strength
            } else if negative_weights && !inc[v].is_empty() {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    if x.iter().all(|&v| v == 0.0) {
        x.fill(1.0);
    }
    rescale_max_abs(&mut x);

    let mut x_new = vec![0.0_f64; n_us];

    for _ in 0..DEFAULT_MAX_ITER {
        // x_new = (W + I) x
        for v in 0..n {
            let v_us = v as usize;
            let mut s = x[v_us];
            for &e in &inc[v_us] {
                let other = graph.edge_other(e, v)?;
                s += weights[e as usize] * x[other as usize];
            }
            x_new[v_us] = s;
        }

        let scale = if negative_weights {
            let (mut best_mag, mut signed) = (0.0_f64, 0.0_f64);
            for &v in &x_new {
                let mag = v.abs();
                if mag > best_mag {
                    best_mag = mag;
                    signed = v;
                }
            }
            signed
        } else {
            x_new.iter().fold(0.0_f64, |acc, &v| acc.max(v.abs()))
        };
        if scale != 0.0 {
            for slot in &mut x_new {
                *slot /= scale;
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

    if !negative_weights {
        for slot in &mut x {
            if *slot < 0.0 {
                *slot = 0.0;
            }
        }
    }
    Ok(x)
}

/// Rayleigh quotient `vᵀ·W·v / vᵀ·v` on the symmetric weighted
/// adjacency, used to recover the eigenvalue along the undirected
/// fallback path.
fn weighted_rayleigh_undirected(graph: &Graph, weights: &[f64], v: &[f64]) -> f64 {
    let n = graph.vcount();
    if n == 0 {
        return 0.0;
    }
    let mut numer = 0.0_f64;
    let mut denom = 0.0_f64;
    for u in 0..n {
        let vu = v[u as usize];
        denom += vu * vu;
        if let Ok(inc) = graph.incident(u) {
            let mut acc = 0.0_f64;
            for &e in &inc {
                if let Ok(other) = graph.edge_other(e, u) {
                    acc += weights[e as usize] * v[other as usize];
                }
            }
            numer += vu * acc;
        }
    }
    if denom > 0.0 { numer / denom } else { 0.0 }
}

/// Rayleigh quotient `vᵀ·A·v / vᵀ·v` on the underlying undirected
/// adjacency, used to recover the eigenvalue along the undirected
/// fallback path.
fn dominant_eigenvalue_undirected(graph: &Graph, v: &[f64]) -> f64 {
    let n = graph.vcount();
    if n == 0 {
        return 0.0;
    }
    let mut numer = 0.0_f64;
    let mut denom = 0.0_f64;
    for u in 0..n {
        let vu = v[u as usize];
        denom += vu * vu;
        if let Ok(nei) = graph.neighbors(u) {
            let mut acc = 0.0_f64;
            for &w in &nei {
                acc += v[w as usize];
            }
            numer += vu * acc;
        }
    }
    if denom > 0.0 { numer / denom } else { 0.0 }
}

fn rescale_max_abs(v: &mut [f64]) {
    let max = v.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
    if max > 0.0 {
        for slot in v {
            *slot /= max;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "index {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        assert!(s.hub.is_empty());
        assert!(s.authority.is_empty());
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn directed_no_edges_fills_ones() {
        let g = Graph::new(3, true).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-12);
        close(&s.authority, &[1.0, 1.0, 1.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn single_directed_edge() {
        // 0 → 1: 0 is a pure hub (auth=0), 1 is a pure authority (hub=0).
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 0.0], 1e-9);
        close(&s.authority, &[0.0, 1.0], 1e-9);
        assert!((s.eigenvalue - 1.0).abs() < 1e-6);
    }

    #[test]
    fn two_to_two_bipartite_hub_auth() {
        // Doctest scenario: 0,1 → 2,3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (0, 3), (1, 2), (1, 3)])
            .unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 1.0, 0.0, 0.0], 1e-9);
        close(&s.authority, &[0.0, 0.0, 1.0, 1.0], 1e-9);
        // Largest eigenvalue of A·Aᵀ for this 2x2 block is 4.
        assert!((s.eigenvalue - 4.0).abs() < 1e-6);
    }

    #[test]
    fn directed_triangle_uniform_one() {
        // 0→1→2→0: every vertex is symmetrically a hub and an authority.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-9);
        close(&s.authority, &[1.0, 1.0, 1.0], 1e-9);
        assert!((s.eigenvalue - 1.0).abs() < 1e-6);
    }

    #[test]
    fn undirected_delegates_to_eigenvector() {
        // Undirected triangle: hub == auth == eigenvector centrality.
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-9);
        close(&s.authority, &s.hub, 1e-15);
    }

    #[test]
    fn undirected_star_hub_equals_eigenvector() {
        // Undirected 4-star: centre = 1, leaves = 1/sqrt(3).
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let s = hub_and_authority_scores(&g).unwrap();
        let inv_sqrt3 = 1.0 / 3f64.sqrt();
        close(&s.hub, &[1.0, inv_sqrt3, inv_sqrt3, inv_sqrt3], 1e-9);
        close(&s.authority, &s.hub, 1e-15);
    }

    #[test]
    fn sink_has_zero_hub() {
        // 0→2, 1→2: 2 is a sink (out-degree 0) → hub[2] = 0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        assert!(s.hub[2].abs() < 1e-9);
        // 2 is the only authority.
        assert!((s.authority[2] - 1.0).abs() < 1e-9);
        assert!(s.authority[0].abs() < 1e-9);
        assert!(s.authority[1].abs() < 1e-9);
    }

    #[test]
    fn source_has_zero_authority() {
        // 0→1, 0→2: 0 is a source (in-degree 0) → authority[0] = 0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2)]).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        assert!(s.authority[0].abs() < 1e-9);
        assert!((s.hub[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn formula_h_eq_a_a_authority() {
        // After convergence, h ∝ A · authority. Verify on a small
        // directed graph using the returned (normalised) vectors and
        // the eigenvalue.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (1, 4)])
            .unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        // Verify: A·a should be parallel to h (up to normalisation by max).
        let n = g.vcount();
        let mut a_a = vec![0.0_f64; n as usize];
        for u in 0..n {
            let mut acc = 0.0_f64;
            for &v in &g.out_neighbors_vec(u).unwrap() {
                acc += s.authority[v as usize];
            }
            a_a[u as usize] = acc;
        }
        let max = a_a.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
        if max > 0.0 {
            for slot in &mut a_a {
                *slot /= max;
            }
        }
        for (u, &val) in a_a.iter().enumerate() {
            assert!(
                (val - s.hub[u]).abs() < 1e-6,
                "vertex {u}: A·a={val} hub={}",
                s.hub[u]
            );
        }
    }

    // -------- PR-017b: weighted variant --------

    #[test]
    fn weighted_length_mismatch_errors() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let result = hub_and_authority_scores_weighted(&g, &[1.0]);
        assert!(matches!(result, Err(IgraphError::InvalidArgument(_))));
    }

    #[test]
    fn weighted_empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[]).unwrap();
        assert!(s.hub.is_empty());
        assert!(s.authority.is_empty());
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_no_edges_fills_ones() {
        let g = Graph::new(3, true).unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[]).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-12);
        close(&s.authority, &[1.0, 1.0, 1.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_all_zero_weights_fills_ones() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[0.0, 0.0]).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-12);
        close(&s.authority, &[1.0, 1.0, 1.0], 1e-12);
        assert!(s.eigenvalue.abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_uniform_matches_unweighted() {
        // With unit weights, weighted and unweighted HITS must agree.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (1, 4)])
            .unwrap();
        let unweighted = hub_and_authority_scores(&g).unwrap();
        let weighted = hub_and_authority_scores_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
        close(&weighted.hub, &unweighted.hub, 1e-6);
        close(&weighted.authority, &unweighted.authority, 1e-6);
        assert!((weighted.eigenvalue - unweighted.eigenvalue).abs() < 1e-6);
    }

    #[test]
    fn weighted_bipartite_authority_tilt() {
        // 0,1 → 2,3 with weights tilting authority 3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (0, 3), (1, 2), (1, 3)])
            .unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[1.0, 4.0, 2.0, 3.0]).unwrap();
        // Hubs are the source partition, authorities are the sink one.
        assert!(s.hub[2].abs() < 1e-9);
        assert!(s.hub[3].abs() < 1e-9);
        assert!(s.authority[0].abs() < 1e-9);
        assert!(s.authority[1].abs() < 1e-9);
        // Authority 3 receives heavier weights (4+3 = 7) than 2 (1+2 = 3).
        assert!(s.authority[3] > s.authority[2]);
    }

    #[test]
    #[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
    fn weighted_cross_relation_invariant() {
        // After convergence, hub ∝ W·authority (max-normalised both sides).
        let mut g = Graph::new(5, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (4, 0)])
            .unwrap();
        let weights = vec![1.5, 0.5, 2.0, 1.0, 0.75, 1.25];
        let s = hub_and_authority_scores_weighted(&g, &weights).unwrap();

        let n = g.vcount() as usize;
        let mut w_auth = vec![0.0_f64; n];
        for (e, &w) in weights.iter().enumerate() {
            let (u, v) = g.edge(e as u32).unwrap();
            w_auth[u as usize] += w * s.authority[v as usize];
        }
        let max = w_auth.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
        if max > 0.0 {
            for slot in &mut w_auth {
                *slot /= max;
            }
        }
        for (u, &val) in w_auth.iter().enumerate() {
            assert!(
                (val - s.hub[u]).abs() < 1e-6,
                "vertex {u}: W·a={val} hub={}",
                s.hub[u]
            );
        }
    }

    #[test]
    fn weighted_undirected_matches_unweighted_under_unit_weights() {
        // Undirected delegation: unit weights ⇒ same as unweighted path.
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (0, 3)])
            .unwrap();
        let unweighted = hub_and_authority_scores(&g).unwrap();
        let weighted = hub_and_authority_scores_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
        close(&weighted.hub, &unweighted.hub, 1e-6);
        close(&weighted.authority, &unweighted.authority, 1e-6);
        assert!((weighted.eigenvalue - unweighted.eigenvalue).abs() < 1e-6);
    }

    #[test]
    fn weighted_negative_weights_no_zero_clip() {
        // With negative weights, sign-cleanup is skipped: at least one
        // vector entry may be < 0 (and we don't reject the input).
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[1.0, -1.0, 1.0]).unwrap();
        // Result must still be finite and one component must hit ±1.
        assert!(s.hub.iter().all(|x| x.is_finite()));
        assert!(s.authority.iter().all(|x| x.is_finite()));
        let max_hub = s.hub.iter().fold(0.0_f64, |a, &x| a.max(x.abs()));
        assert!((max_hub - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weighted_sink_has_zero_hub() {
        // 0→2 (w=2), 1→2 (w=3): 2 is a sink → hub[2] = 0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        let s = hub_and_authority_scores_weighted(&g, &[2.0, 3.0]).unwrap();
        assert!(s.hub[2].abs() < 1e-9);
        assert!((s.authority[2] - 1.0).abs() < 1e-9);
    }
}
