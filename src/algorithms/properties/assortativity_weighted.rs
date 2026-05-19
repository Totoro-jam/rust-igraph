//! Weighted degree assortativity coefficient (ALGO-PR-006b).
//!
//! Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/false,
//! &weights)` from `references/igraph/src/misc/mixing.c`. Same Pearson
//! framework as PR-006 but each vertex's "type" is its **strength**
//! (weighted degree) and each edge's contribution is multiplied by
//! its weight `w`:
//!
//! ```text
//! For each edge e = (u, v) with weight w:
//!   num1 += w * (s_u * s_v)
//!   num2 += w * (s_u + s_v)
//!   den1 += w * (s_u^2 + s_v^2)
//! W = Σ w_i
//! num1 /= W;  den1 /= 2W;  num2 /= 2W;  num2 *= num2
//! r = (num1 - num2) / (den1 - num2)
//! ```
//!
//! Returns `None` for graphs with no edges, zero total weight, or zero
//! variance (matches upstream's `IGRAPH_NAN`). Phase-1 minimal slice:
//! undirected only; directed weighted variant ships in PR-006c.
//!
//! Strength accounting for undirected self-loops follows
//! `igraph_strength(_, _, _, IGRAPH_ALL, IGRAPH_LOOPS, &weights)` —
//! a self-loop with weight `w` contributes `2w` to its vertex's
//! strength.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Weighted degree assortativity coefficient.
///
/// `weights[e]` must be non-negative and finite; `weights.len()` must
/// equal `graph.ecount()`. Returns `None` when the metric is undefined
/// (no edges, zero total weight, or all-equal strength → zero
/// variance). Directed graphs return [`IgraphError::Unsupported`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity_degree_weighted};
///
/// // Path 0-1-2 with unit weights → strengths [1, 2, 1]; r = -1.0
/// // (matches the unweighted PR-006 result on the same graph).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let r = assortativity_degree_weighted(&g, &[1.0, 1.0]).unwrap();
/// assert!((r.unwrap() - (-1.0_f64)).abs() < 1e-12);
/// ```
pub fn assortativity_degree_weighted(graph: &Graph, weights: &[f64]) -> IgraphResult<Option<f64>> {
    if graph.is_directed() {
        return Err(IgraphError::Unsupported(
            "directed weighted assortativity is PR-006c; not yet ported",
        ));
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() || w < 0.0 || !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} must be non-negative and finite (got {w})"
            )));
        }
    }

    // Per-vertex strength: sum of incident-edge weights, with self-loops
    // contributing 2*w to match upstream's LOOPS_TWICE convention.
    let n = graph.vcount();
    let n_us = n as usize;
    let mut strength = vec![0.0_f64; n_us];

    let m_u = u32::try_from(m).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    for e in 0..m_u {
        let (u, v) = graph.edge(e as EdgeId)?;
        let edge_w = weights[e as usize];
        if u == v {
            strength[u as usize] += 2.0 * edge_w;
        } else {
            strength[u as usize] += edge_w;
            strength[v as usize] += edge_w;
        }
    }

    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut den1 = 0.0_f64;
    let mut total_w = 0.0_f64;

    for e in 0..m_u {
        let (u, v) = graph.edge(e as EdgeId)?;
        let edge_w = weights[e as usize];
        let su = strength[u as usize];
        let sv = strength[v as usize];
        num1 += edge_w * (su * sv);
        num2 += edge_w * (su + sv);
        den1 += edge_w * (su * su + sv * sv);
        total_w += edge_w;
    }

    if total_w == 0.0 {
        return Ok(None);
    }

    num1 /= total_w;
    den1 /= 2.0 * total_w;
    num2 /= 2.0 * total_w;
    num2 *= num2;

    let denom = den1 - num2;
    if denom == 0.0 {
        return Ok(None);
    }
    Ok(Some((num1 - num2) / denom))
}

/// Directed weighted degree assortativity (PR-006d).
///
/// Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/true,
/// &weights)` from `references/igraph/src/misc/mixing.c:351-405`. Pearson
/// correlation between out-strength of source and in-strength of target,
/// each edge weighted by `w`:
///
/// ```text
/// For each edge (u → v) with weight w:
///   num1 += w * (s_out[u] * s_in[v])
///   num2 += w * s_out[u]
///   num3 += w * s_in[v]
///   den1 += w * s_out[u]^2
///   den2 += w * s_in[v]^2
/// W = Σ w
/// r = (num1 - num2*num3/W) / (sqrt(den1 - num2^2/W) * sqrt(den2 - num3^2/W))
/// ```
///
/// Returns `None` for graphs with no edges, zero total weight, or zero
/// variance (matches upstream's `IGRAPH_NAN`). Undirected graphs route
/// to the symmetric formula via [`assortativity_degree_weighted`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity_degree_directed_weighted};
///
/// // Directed 3-cycle 0→1→2→0 with unit weights: every vertex has
/// // out-strength 1 and in-strength 1, so both variance terms vanish.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(
///     assortativity_degree_directed_weighted(&g, &[1.0, 1.0, 1.0]).unwrap(),
///     None
/// );
/// ```
pub fn assortativity_degree_directed_weighted(
    graph: &Graph,
    weights: &[f64],
) -> IgraphResult<Option<f64>> {
    if !graph.is_directed() {
        return assortativity_degree_weighted(graph, weights);
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }
    if weights.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "weights vector size ({}) differs from edge count ({})",
            weights.len(),
            m
        )));
    }
    for (e, &w) in weights.iter().enumerate() {
        if w.is_nan() || w < 0.0 || !w.is_finite() {
            return Err(IgraphError::InvalidArgument(format!(
                "weight at edge {e} must be non-negative and finite (got {w})"
            )));
        }
    }

    let n = graph.vcount();
    let n_us = n as usize;
    let mut out_strength = vec![0.0_f64; n_us];
    let mut in_strength = vec![0.0_f64; n_us];

    let m_u = u32::try_from(m).map_err(|_| IgraphError::Internal("ecount overflows u32"))?;
    for e in 0..m_u {
        let (u, v) = graph.edge(e as EdgeId)?;
        let edge_w = weights[e as usize];
        out_strength[u as usize] += edge_w;
        in_strength[v as usize] += edge_w;
    }

    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut num3 = 0.0_f64;
    let mut den1 = 0.0_f64;
    let mut den2 = 0.0_f64;
    let mut total_w = 0.0_f64;

    for e in 0..m_u {
        let (u, v) = graph.edge(e as EdgeId)?;
        let edge_w = weights[e as usize];
        let so = out_strength[u as usize];
        let si = in_strength[v as usize];
        num1 += edge_w * so * si;
        num2 += edge_w * so;
        num3 += edge_w * si;
        den1 += edge_w * so * so;
        den2 += edge_w * si * si;
        total_w += edge_w;
    }

    if total_w == 0.0 {
        return Ok(None);
    }
    let num = num1 - num2 * num3 / total_w;
    let var_from = den1 - num2 * num2 / total_w;
    let var_to = den2 - num3 * num3 / total_w;
    if var_from <= 0.0 || var_to <= 0.0 {
        return Ok(None);
    }
    let den = var_from.sqrt() * var_to.sqrt();
    if den == 0.0 {
        return Ok(None);
    }
    Ok(Some(num / den))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} vs {b}");
    }

    #[test]
    fn empty_graph_is_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(assortativity_degree_weighted(&g, &[]).unwrap(), None);
    }

    #[test]
    fn no_edges_is_none() {
        let g = Graph::with_vertices(5);
        assert_eq!(assortativity_degree_weighted(&g, &[]).unwrap(), None);
    }

    #[test]
    fn unit_weights_match_unweighted_path_3() {
        // Path 0-1-2 with unit weights → -1.0 (same as unweighted PR-006).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = assortativity_degree_weighted(&g, &[1.0, 1.0])
            .unwrap()
            .unwrap();
        close(r, -1.0, 1e-12);
    }

    #[test]
    fn unit_weights_match_unweighted_diamond() {
        // K4 minus an edge → -2/3 (matches PR-006's diamond test).
        let mut g = Graph::with_vertices(4);
        for &(u, v) in &[(0u32, 1), (0, 2), (1, 2), (1, 3), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let weights = vec![1.0; 5];
        let r = assortativity_degree_weighted(&g, &weights)
            .unwrap()
            .unwrap();
        close(r, -2.0 / 3.0, 1e-12);
    }

    #[test]
    fn k4_regular_unit_weights_returns_none() {
        // K4 every vertex same strength → variance zero → None.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(assortativity_degree_weighted(&g, &[1.0; 6]).unwrap(), None);
    }

    #[test]
    fn weighted_path_breaks_perfect_disassortativity() {
        // Path 0-1-2 with weights (1, 4): strengths [1, 5, 4].
        // Edge 0=(0,1) w=1: num1 += 1*(1*5)=5; num2 += 1*(1+5)=6;
        //                   den1 += 1*(1+25)=26
        // Edge 1=(1,2) w=4: num1 += 4*(5*4)=80; num2 += 4*(5+4)=36;
        //                   den1 += 4*(25+16)=164
        // W = 5; num1=85/5=17; num2=42/10=4.2; num2^2=17.64; den1=190/10=19
        // r = (17 - 17.64) / (19 - 17.64) = -0.64 / 1.36 ≈ -0.470588...
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = assortativity_degree_weighted(&g, &[1.0, 4.0])
            .unwrap()
            .unwrap();
        close(r, -0.64 / 1.36, 1e-12);
    }

    #[test]
    fn weights_size_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_weighted(&g, &[]).is_err());
    }

    #[test]
    fn negative_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn nan_weight_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_weighted(&g, &[f64::NAN]).is_err());
    }

    #[test]
    fn directed_returns_unsupported() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_weighted(&g, &[1.0]).is_err());
    }

    #[test]
    fn zero_total_weight_returns_none() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // All weights zero → W = 0 → None (avoid division by zero).
        assert_eq!(
            assortativity_degree_weighted(&g, &[0.0, 0.0]).unwrap(),
            None
        );
    }

    // ---- PR-006d: Directed weighted assortativity ----------------

    #[test]
    fn directed_weighted_3_cycle_uniform_returns_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(
            assortativity_degree_directed_weighted(&g, &[1.0, 1.0, 1.0]).unwrap(),
            None
        );
    }

    #[test]
    fn directed_weighted_unit_weights_match_unweighted_directed() {
        // Directed chain 0→1→2→3→4 with unit weights should match the
        // unweighted directed assortativity (formula collapses identically).
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let unweighted =
            crate::algorithms::properties::assortativity::assortativity_degree_directed(&g)
                .unwrap();
        let weighted = assortativity_degree_directed_weighted(&g, &[1.0; 4]).unwrap();
        match (unweighted, weighted) {
            (Some(u), Some(w)) => close(u, w, 1e-12),
            (None, None) => {}
            _ => panic!("disagreement between unweighted={unweighted:?} weighted={weighted:?}"),
        }
    }

    #[test]
    fn directed_weighted_undirected_routes_to_undirected_weighted() {
        // For an undirected input, the directed function should behave
        // exactly like the undirected weighted variant.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r1 = assortativity_degree_directed_weighted(&g, &[1.0, 1.0]).unwrap();
        let r2 = assortativity_degree_weighted(&g, &[1.0, 1.0]).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn directed_weighted_empty_no_edges_is_none() {
        let g = Graph::new(3, true).unwrap();
        assert_eq!(
            assortativity_degree_directed_weighted(&g, &[]).unwrap(),
            None
        );
    }

    #[test]
    fn directed_weighted_negative_weight_errors() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_directed_weighted(&g, &[-1.0]).is_err());
    }

    #[test]
    fn directed_weighted_size_mismatch_errors() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(assortativity_degree_directed_weighted(&g, &[]).is_err());
        assert!(assortativity_degree_directed_weighted(&g, &[1.0, 2.0]).is_err());
    }
}
