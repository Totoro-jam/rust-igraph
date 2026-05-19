//! Degree assortativity coefficient (ALGO-PR-006).
//!
//! Counterpart of `igraph_assortativity_degree()` from
//! `references/igraph/src/misc/mixing.c:443` and the underlying
//! `igraph_assortativity()` (`mixing.c:273`). The metric is the Pearson
//! correlation of endpoint degrees over the edge list — high positive
//! values mean high-degree vertices tend to connect to each other
//! (assortative mixing); negative values mean opposite.
//!
//! Phase-1 covers undirected unweighted ([`assortativity_degree`] —
//! ALGO-PR-006), undirected weighted (`assortativity_degree_weighted`
//! in [`super::assortativity_weighted`] — ALGO-PR-006b), and now
//! [`assortativity_degree_directed`] (ALGO-PR-006c — directed Pearson
//! correlation of source out-degree against target in-degree).
//! Weighted-directed ships when needed.
//!
//! Formula (matches upstream's float ordering at `mixing.c:306-349`):
//! - For each edge `e = (u, v)` over the m edges of the graph:
//!   - `num1 += deg(u) * deg(v)`
//!   - `num2 += deg(u) + deg(v)`
//!   - `den1 += deg(u)^2 + deg(v)^2`
//! - `num1 /= m`
//! - `den1 /= 2 * m`
//! - `num2 /= 2 * m`; `num2 = num2 * num2`
//! - `r = (num1 - num2) / (den1 - num2)`
//! - When `den1 == num2` (regular graphs — every vertex has the same
//!   degree, so the variance is zero), `r` is undefined → `None`.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphResult};

/// Degree assortativity coefficient of `graph` (undirected, unweighted).
/// Returns `None` for graphs with no edges or for regular graphs
/// (all vertices same degree — the variance denominator vanishes,
/// matching upstream's `IGRAPH_NAN`).
///
/// Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/false)`.
/// Directed graphs return [`crate::IgraphError::Unsupported`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity_degree};
///
/// // 4-cycle: all vertices have degree 2 → regular graph → None.
/// let mut g = Graph::with_vertices(4);
/// for i in 0..4u32 { g.add_edge(i, (i + 1) % 4).unwrap(); }
/// assert_eq!(assortativity_degree(&g).unwrap(), None);
///
/// // Path 0-1-2: deg=[1,2,1]. Two edges, both connect a deg-1 vertex
/// // to the deg-2 centre. The metric is well-defined here:
/// // num1 = (1*2 + 2*1) / 2 = 2
/// // num2 = ((1+2 + 2+1) / 4)^2 = (6/4)^2 = 2.25
/// // den1 = (1+4 + 4+1) / 4 = 2.5
/// // r = (2 - 2.25) / (2.5 - 2.25) = -1.0  (perfectly disassortative)
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert_eq!(assortativity_degree(&g).unwrap(), Some(-1.0));
/// ```
pub fn assortativity_degree(graph: &Graph) -> IgraphResult<Option<f64>> {
    if graph.is_directed() {
        // Directed graphs route through the directed Pearson formula
        // (PR-006c) — `assortativity_degree_directed` is the canonical
        // entry but `assortativity_degree(g)` doing the natural thing
        // matches python-igraph's `Graph.assortativity_degree()` default.
        return assortativity_degree_directed(graph);
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }

    // Per-vertex degree vector. `Graph::degree` already counts self-loops
    // twice for undirected graphs (LOOPS_TWICE), matching upstream's
    // `igraph_strength(_, _, _, IGRAPH_ALL, IGRAPH_LOOPS, NULL)`.
    let n = graph.vcount();
    let mut deg = Vec::with_capacity(n as usize);
    for v in 0..n {
        let d = graph.degree(v)?;
        // d is bounded by ecount * 2 in the undirected LOOPS_TWICE case;
        // fits in u32 for any practical graph that survives our u32
        // edge-id encoding.
        #[allow(clippy::cast_precision_loss)]
        deg.push(d as f64);
    }

    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut den1 = 0.0_f64;

    let m_u32 = u32::try_from(m).map_err(|_| {
        crate::core::IgraphError::Internal("ecount overflows u32 for assortativity")
    })?;
    for e in 0..m_u32 {
        let (u, v) = graph.edge(e as EdgeId)?;
        let du = deg[u as usize];
        let dv = deg[v as usize];
        num1 += du * dv;
        num2 += du + dv;
        den1 += du * du + dv * dv;
    }

    #[allow(clippy::cast_precision_loss)]
    let total = m as f64;
    num1 /= total;
    den1 /= total * 2.0;
    num2 /= total * 2.0;
    num2 *= num2;

    let denom = den1 - num2;
    if denom == 0.0 {
        // Regular graph → upstream returns NaN; we encode as None.
        return Ok(None);
    }
    Ok(Some((num1 - num2) / denom))
}

/// Directed degree assortativity coefficient (ALGO-PR-006c).
///
/// Counterpart of `igraph_assortativity_degree(_, _, /*directed=*/true)`
/// (the directed branch of `mixing.c:443`). For each directed edge
/// `e = (u → v)` Pearson-correlates the source's out-degree against
/// the target's in-degree:
///
/// ```text
/// num1 = Σ out_deg(u) * in_deg(v)
/// num2 = Σ out_deg(u)
/// num3 = Σ in_deg(v)
/// den1 = Σ out_deg(u)²
/// den2 = Σ in_deg(v)²
///
/// num = num1 − num2 * num3 / m
/// den = sqrt(den1 − num2² / m) * sqrt(den2 − num3² / m)
/// r   = num / den       (None if den == 0)
/// ```
///
/// Returns `None` for graphs with no edges or where either variance
/// term collapses (regular in-degrees and/or regular out-degrees —
/// matches upstream NaN). Undirected graphs are accepted and route
/// to the undirected formula via [`assortativity_degree`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity_degree_directed};
///
/// // Directed 3-cycle 0→1→2→0: every vertex has out-degree 1 and
/// // in-degree 1. Both variance terms vanish → None.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(assortativity_degree_directed(&g).unwrap(), None);
/// ```
pub fn assortativity_degree_directed(graph: &Graph) -> IgraphResult<Option<f64>> {
    if !graph.is_directed() {
        // Undirected graphs use the symmetric formula; defer to the
        // canonical undirected entry.
        return assortativity_degree(graph);
    }
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }

    let n = graph.vcount();
    let n_us = n as usize;
    let mut out_deg = vec![0.0_f64; n_us];
    let mut in_deg = vec![0.0_f64; n_us];

    let m_u32 = u32::try_from(m).map_err(|_| {
        crate::core::IgraphError::Internal("ecount overflows u32 for assortativity")
    })?;
    for e in 0..m_u32 {
        let (src, tgt) = graph.edge(e as EdgeId)?;
        out_deg[src as usize] += 1.0;
        in_deg[tgt as usize] += 1.0;
    }

    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut num3 = 0.0_f64;
    let mut den1 = 0.0_f64;
    let mut den2 = 0.0_f64;

    for e in 0..m_u32 {
        let (src, tgt) = graph.edge(e as EdgeId)?;
        let from_value = out_deg[src as usize];
        let to_value = in_deg[tgt as usize];
        num1 += from_value * to_value;
        num2 += from_value;
        num3 += to_value;
        den1 += from_value * from_value;
        den2 += to_value * to_value;
    }

    #[allow(clippy::cast_precision_loss)]
    let total = m as f64;
    let num = num1 - num2 * num3 / total;
    let var_from = den1 - num2 * num2 / total;
    let var_to = den2 - num3 * num3 / total;
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

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() < tol,
            "expected {b} ± {tol}, got {a} (diff {})",
            (a - b).abs()
        );
    }

    #[test]
    fn empty_graph_is_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    #[test]
    fn isolated_vertices_no_edges_is_none() {
        let g = Graph::with_vertices(5);
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    #[test]
    fn regular_graph_returns_none() {
        // 4-cycle: every vertex has degree 2.
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    #[test]
    fn k4_is_regular_returns_none() {
        // K4: every vertex has degree 3.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    #[test]
    fn path_3_is_perfectly_disassortative() {
        // 0-1-2: deg [1, 2, 1]. By the formula, r = -1.0.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = assortativity_degree(&g).unwrap().unwrap();
        assert_close(r, -1.0, 1e-12);
    }

    #[test]
    fn star_is_perfectly_disassortative() {
        // Star centre deg=3, 3 leaves with deg=1 each.
        // All 3 edges: deg(centre)=3, deg(leaf)=1.
        // num1 = 3*1 + 3*1 + 3*1 = 9, /m=9/3 = 3
        // num2 = (3+1)*3 / (2*3) = 12/6 = 2; squared = 4
        // den1 = (9+1)*3 / (2*3) = 30/6 = 5
        // r = (3 - 4) / (5 - 4) = -1.0
        let mut g = Graph::with_vertices(4);
        for v in 1..4 {
            g.add_edge(0, v).unwrap();
        }
        let r = assortativity_degree(&g).unwrap().unwrap();
        assert_close(r, -1.0, 1e-12);
    }

    #[test]
    fn two_disjoint_edges_is_assortative_or_regular() {
        // Two parallel edges 0-1 and 2-3: every vertex has degree 1
        // (regular; r is undefined / None).
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    #[test]
    fn directed_graph_routes_to_directed_formula() {
        // PR-006c extension: `assortativity_degree(directed_g)` no
        // longer returns `Unsupported` — it routes to
        // `assortativity_degree_directed`. A single edge has too few
        // samples to compute Pearson (variance == 0 on both sides),
        // so result is None.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(assortativity_degree(&g).unwrap(), None);
    }

    // ----- ALGO-PR-006c: directed assortativity -----

    #[test]
    fn directed_3_cycle_is_regular_returns_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(assortativity_degree_directed(&g).unwrap(), None);
    }

    #[test]
    fn directed_path_three_disassortative() {
        // 0 → 1 → 2:
        //   out_deg = [1, 1, 0], in_deg = [0, 1, 1]
        //   Edge (0,1): from=1, to=1. Edge (1,2): from=1, to=1.
        //   num1 = 1*1 + 1*1 = 2
        //   num2 = 1 + 1 = 2 (sum of out_degs over edges)
        //   num3 = 1 + 1 = 2 (sum of in_degs over edges)
        //   den1 = 1 + 1 = 2; den2 = 1 + 1 = 2; m = 2
        //   num = 2 - 2*2/2 = 2 - 2 = 0
        //   var_from = 2 - 4/2 = 0; var_to = 2 - 4/2 = 0
        //   → den is 0 → None
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(assortativity_degree_directed(&g).unwrap(), None);
    }

    #[test]
    fn directed_chain_with_branch_is_well_defined() {
        // 0→1, 1→2, 0→2: out_deg=[2, 1, 0], in_deg=[0, 1, 2].
        //   Edges (0,1): out(0)=2, in(1)=1
        //         (0,2): out(0)=2, in(2)=2
        //         (1,2): out(1)=1, in(2)=2
        //   num1 = 2*1 + 2*2 + 1*2 = 8
        //   num2 = 2 + 2 + 1 = 5
        //   num3 = 1 + 2 + 2 = 5
        //   den1 = 4 + 4 + 1 = 9
        //   den2 = 1 + 4 + 4 = 9
        //   m = 3
        //   num = 8 - 5*5/3 = 8 - 25/3 ≈ -0.333
        //   var_from = 9 - 25/3 ≈ 0.667; var_to = same ≈ 0.667
        //   den = sqrt(0.667)² ≈ 0.667
        //   r ≈ -0.5
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let r = assortativity_degree_directed(&g).unwrap().unwrap();
        assert_close(r, -0.5, 1e-12);
    }

    #[test]
    fn directed_empty_graph_returns_none() {
        let g = Graph::new(0, true).unwrap();
        assert_eq!(assortativity_degree_directed(&g).unwrap(), None);
    }

    #[test]
    fn directed_undirected_graph_routes_to_undirected_formula() {
        // assortativity_degree_directed on an undirected graph should
        // delegate to assortativity_degree (matches python-igraph's
        // behaviour where the `directed` arg is ignored on undirected).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let a = assortativity_degree(&g).unwrap();
        let b = assortativity_degree_directed(&g).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn diamond_k4_minus_edge() {
        // Edges (0,1)(0,2)(1,2)(1,3)(2,3): deg=[2, 3, 3, 2].
        // m = 5
        // num1 = 2*3 + 2*3 + 3*3 + 3*2 + 3*2 = 6+6+9+6+6 = 33; /5 = 6.6
        // num2 = (2+3)+(2+3)+(3+3)+(3+2)+(3+2) = 5+5+6+5+5 = 26;  / (2*5) = 2.6; ^2 = 6.76
        // den1 = 4+9 + 4+9 + 9+9 + 9+4 + 9+4 = 13+13+18+13+13 = 70; /(2*5) = 7.0
        // r = (6.6 - 6.76) / (7.0 - 6.76) = -0.16 / 0.24 = -0.66666...
        let mut g = Graph::with_vertices(4);
        for &(u, v) in &[(0u32, 1), (0, 2), (1, 2), (1, 3), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let r = assortativity_degree(&g).unwrap().unwrap();
        assert_close(r, -2.0 / 3.0, 1e-12);
    }

    #[test]
    fn two_triangles_joined_by_bridge_matches_python_igraph() {
        // {0,1,2} triangle, {3,4,5} triangle, plus edge 2-3.
        // deg = [2, 2, 3, 3, 2, 2]. python-igraph 0.11.9 reports
        // assortativity_degree() = -0.16666666666666424 (slightly
        // disassortative — the 6 inner triangle edges connect deg-2 to
        // deg-3 vertices, and the lone bridge connects deg-3 to deg-3).
        let mut g = Graph::with_vertices(6);
        for &(u, v) in &[(0u32, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
            g.add_edge(u, v).unwrap();
        }
        let r = assortativity_degree(&g).unwrap().unwrap();
        assert_close(r, -0.166_666_666_666_664_24, 1e-12);
    }
}
