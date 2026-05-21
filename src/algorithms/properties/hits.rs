//! Kleinberg's hub and authority scores — HITS (ALGO-PR-017).
//!
//! Counterpart of `igraph_hub_and_authority_scores()` from
//! `references/igraph/src/centrality/hub_authority.c`. The hub score
//! `h[v]` is the `v`-th component of the principal eigenvector of
//! `A·Aᵀ`; the authority score `a[v]` is the principal eigenvector of
//! `Aᵀ·A`. The two are tied by `a = Aᵀ·h` and `h = A·a` once both have
//! converged.
//!
//! Phase-1 minimal slice: unweighted graphs only.
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
//! Weighted variants and an ARPACK backend ship later (PR-017b /
//! PR-017c, paralleling PR-011/PR-011c and PR-012/PR-012b).

use crate::algorithms::properties::eigenvector::eigenvector_centrality;
use crate::core::{Graph, IgraphResult};

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
/// Counterpart of `igraph_hub_and_authority_scores(g, h, a, &val,
/// /*weights=*/NULL, /*options=*/NULL)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, hub_and_authority_scores};
///
/// // Bipartite hubs→authorities pattern: 0 and 1 both point to 2 and 3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 2u32), (0, 3), (1, 2), (1, 3)]).unwrap();
/// let s = hub_and_authority_scores(&g).unwrap();
/// // 0 and 1 are hubs (point to authorities), 2 and 3 are authorities.
/// assert!((s.hub[0] - 1.0).abs() < 1e-9);
/// assert!((s.hub[1] - 1.0).abs() < 1e-9);
/// assert!(s.hub[2].abs() < 1e-9);
/// assert!(s.hub[3].abs() < 1e-9);
/// assert!((s.authority[2] - 1.0).abs() < 1e-9);
/// assert!((s.authority[3] - 1.0).abs() < 1e-9);
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

/// Rayleigh quotient `vᵀ·A·v / vᵀ·v` on the underlying undirected
/// adjacency, used to recover the eigenvalue along the undirected
/// fallback path.
fn dominant_eigenvalue_undirected(graph: &Graph, v: &[f64]) -> f64 {
    let n = graph.vcount();
    let n_us = n as usize;
    if n_us == 0 {
        return 0.0;
    }
    let mut numer = 0.0_f64;
    let mut denom = 0.0_f64;
    for u in 0..n_us {
        denom += v[u] * v[u];
        if let Ok(nei) = graph.neighbors(u as u32) {
            let mut acc = 0.0_f64;
            for &w in &nei {
                acc += v[w as usize];
            }
            numer += v[u] * acc;
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
        assert_eq!(s.eigenvalue, 0.0);
    }

    #[test]
    fn directed_no_edges_fills_ones() {
        let g = Graph::new(3, true).unwrap();
        let s = hub_and_authority_scores(&g).unwrap();
        close(&s.hub, &[1.0, 1.0, 1.0], 1e-12);
        close(&s.authority, &[1.0, 1.0, 1.0], 1e-12);
        assert_eq!(s.eigenvalue, 0.0);
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
        let n = g.vcount() as usize;
        let mut a_a = vec![0.0_f64; n];
        for u in 0..n {
            let mut acc = 0.0_f64;
            for &v in &g.out_neighbors_vec(u as u32).unwrap() {
                acc += s.authority[v as usize];
            }
            a_a[u] = acc;
        }
        let max = a_a.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
        if max > 0.0 {
            for slot in &mut a_a {
                *slot /= max;
            }
        }
        for u in 0..n {
            assert!(
                (a_a[u] - s.hub[u]).abs() < 1e-6,
                "vertex {u}: A·a={} hub={}",
                a_a[u],
                s.hub[u]
            );
        }
    }
}
