//! Value-based assortativity coefficient (ALGO-PR-067).
//!
//! Counterpart of the generic `igraph_assortativity()` from
//! `references/igraph/src/misc/mixing.c`. Where [`super::assortativity`]
//! (`assortativity_degree`) hard-wires each vertex's "value" to its
//! degree, this entry takes an arbitrary numeric value per vertex and
//! Pearson-correlates the endpoint values over the edge list.
//!
//! It backs python-igraph's `Graph.assortativity(types1, types2, ...)`
//! and rigraph's `assortativity(graph, values, values.in, ...)`.
//!
//! Formula (matches upstream's float ordering at `mixing.c`):
//!
//! Undirected (each edge `(u, v)` with optional weight `w`, default 1):
//! ```text
//! W    = Σ w                     (= m when unweighted)
//! num1 = Σ w · value[u] · value[v]
//! num2 = Σ w · (value[u] + value[v])
//! den1 = Σ w · (value[u]² + value[v]²)      (only when normalized)
//! num1 /= W;  den1 /= 2W;  num2 /= 2W;  num2 = num2²
//! r = normalized ? (num1 − num2) / (den1 − num2) : (num1 − num2)
//! ```
//!
//! Directed (source value from `values`, target value from `values_in`,
//! which defaults to `values`):
//! ```text
//! num1 = Σ w · value[u] · value_in[v]
//! num2 = Σ w · value[u]
//! num3 = Σ w · value_in[v]
//! den1 = Σ w · value[u]²            (only when normalized)
//! den2 = Σ w · value_in[v]²         (only when normalized)
//! num = num1 − num2 · num3 / W
//! den = sqrt(den1 − num2²/W) · sqrt(den2 − num3²/W)
//! r = normalized ? num / den : num / W
//! ```

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Value-based assortativity coefficient of `graph`.
///
/// - `values`: numeric value for each vertex (length must equal the
///   vertex count). For directed graphs these are the **source** values.
/// - `values_in`: optional target values for directed graphs (length must
///   equal the vertex count). When `None`, `values` is reused for both
///   endpoints. Ignored for undirected graphs.
/// - `weights`: optional per-edge weights (length must equal the edge
///   count). When `None`, every edge contributes weight 1.
/// - `directed`: respect edge orientation; only takes effect when the
///   graph itself is directed.
/// - `normalized`: when `true`, return the standard normalized
///   (Pearson-style) coefficient in `[-1, 1]`; when `false`, return the
///   unnormalized covariance-style value.
///
/// Returns `None` when the coefficient is undefined (no edges, zero total
/// weight, or a vanishing variance denominator) — mirroring upstream's
/// `IGRAPH_NAN`.
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if any input vector length
/// does not match the corresponding count.
///
/// # References
///
/// M. E. J. Newman: Mixing patterns in networks,
/// Phys. Rev. E 67, 026126 (2003).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, assortativity};
///
/// // Path 0-1-2 with values mirroring the degrees [1, 2, 1] reproduces
/// // the degree-assortativity result of -1.0 (perfectly disassortative).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let values = [1.0, 2.0, 1.0];
/// let r = assortativity(&g, &values, None, None, false, true).unwrap();
/// assert!((r.unwrap() - (-1.0)).abs() < 1e-12);
/// ```
pub fn assortativity(
    graph: &Graph,
    values: &[f64],
    values_in: Option<&[f64]>,
    weights: Option<&[f64]>,
    directed: bool,
    normalized: bool,
) -> IgraphResult<Option<f64>> {
    let n = graph.vcount() as usize;
    let m = graph.ecount();

    if values.len() != n {
        return Err(IgraphError::InvalidArgument(format!(
            "assortativity: values length ({}) does not match vertex count ({n})",
            values.len()
        )));
    }
    if let Some(vin) = values_in {
        if vin.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "assortativity: values_in length ({}) does not match vertex count ({n})",
                vin.len()
            )));
        }
    }
    if let Some(w) = weights {
        if w.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "assortativity: weights length ({}) does not match edge count ({m})",
                w.len()
            )));
        }
    }

    let directed = directed && graph.is_directed();

    let m_u32 = u32::try_from(m)
        .map_err(|_| IgraphError::Internal("ecount overflows u32 for assortativity"))?;

    #[allow(clippy::cast_precision_loss)]
    let total_weight = match weights {
        Some(w) => w.iter().sum::<f64>(),
        None => m as f64,
    };

    let result = if directed {
        assortativity_directed(
            graph,
            values,
            values_in.unwrap_or(values),
            weights,
            normalized,
            total_weight,
            m_u32,
        )?
    } else {
        assortativity_undirected(graph, values, weights, normalized, total_weight, m_u32)?
    };

    Ok(if result.is_finite() {
        Some(result)
    } else {
        None
    })
}

#[allow(clippy::too_many_arguments)]
fn assortativity_undirected(
    graph: &Graph,
    values: &[f64],
    weights: Option<&[f64]>,
    normalized: bool,
    total_weight: f64,
    m_u32: u32,
) -> IgraphResult<f64> {
    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut den1 = 0.0_f64;

    for e in 0..m_u32 {
        let (from, to) = graph.edge(e as EdgeId)?;
        let from_value = values[from as usize];
        let to_value = values[to as usize];
        let w = weights.map_or(1.0, |ws| ws[e as usize]);

        num1 += w * from_value * to_value;
        num2 += w * (from_value + to_value);
        if normalized {
            den1 += w * (from_value * from_value + to_value * to_value);
        }
    }

    num1 /= total_weight;
    if normalized {
        den1 /= total_weight * 2.0;
    }
    num2 /= total_weight * 2.0;
    num2 *= num2;

    Ok(if normalized {
        (num1 - num2) / (den1 - num2)
    } else {
        num1 - num2
    })
}

#[allow(clippy::too_many_arguments)]
fn assortativity_directed(
    graph: &Graph,
    values: &[f64],
    values_in: &[f64],
    weights: Option<&[f64]>,
    normalized: bool,
    total_weight: f64,
    m_u32: u32,
) -> IgraphResult<f64> {
    let mut num1 = 0.0_f64;
    let mut num2 = 0.0_f64;
    let mut num3 = 0.0_f64;
    let mut den1 = 0.0_f64;
    let mut den2 = 0.0_f64;

    for e in 0..m_u32 {
        let (from, to) = graph.edge(e as EdgeId)?;
        let from_value = values[from as usize];
        let to_value = values_in[to as usize];
        let w = weights.map_or(1.0, |ws| ws[e as usize]);

        num1 += w * from_value * to_value;
        num2 += w * from_value;
        num3 += w * to_value;
        if normalized {
            den1 += w * (from_value * from_value);
            den2 += w * (to_value * to_value);
        }
    }

    let num = num1 - num2 * num3 / total_weight;
    Ok(if normalized {
        let den =
            (den1 - num2 * num2 / total_weight).sqrt() * (den2 - num3 * num3 / total_weight).sqrt();
        num / den
    } else {
        num / total_weight
    })
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
    fn no_edges_is_none() {
        let g = Graph::with_vertices(3);
        let values = [1.0, 2.0, 3.0];
        assert_eq!(
            assortativity(&g, &values, None, None, false, true).unwrap(),
            None
        );
    }

    #[test]
    fn wrong_values_length_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let values = [1.0, 2.0];
        assert!(assortativity(&g, &values, None, None, false, true).is_err());
    }

    #[test]
    fn wrong_weights_length_errors() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let values = [1.0, 2.0, 1.0];
        let weights = [1.0];
        assert!(assortativity(&g, &values, None, Some(&weights), false, true).is_err());
    }

    #[test]
    fn path_with_degree_values_matches_degree_assortativity() {
        // 0-1-2 with values = degrees [1,2,1] → r = -1.0 (matches
        // assortativity_degree on the same path).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let values = [1.0, 2.0, 1.0];
        let r = assortativity(&g, &values, None, None, false, true)
            .unwrap()
            .unwrap();
        assert_close(r, -1.0, 1e-12);
    }

    #[test]
    fn constant_values_zero_variance_is_none() {
        // All vertices share the same value → variance denominator
        // vanishes → undefined → None (normalized).
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let values = [5.0, 5.0, 5.0, 5.0];
        assert_eq!(
            assortativity(&g, &values, None, None, false, true).unwrap(),
            None
        );
    }

    #[test]
    fn perfectly_assortative_two_components() {
        // Two edges 0-1 (values 1,1) and 2-3 (values 9,9): like values
        // always paired → r = +1.0.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let values = [1.0, 1.0, 9.0, 9.0];
        let r = assortativity(&g, &values, None, None, false, true)
            .unwrap()
            .unwrap();
        assert_close(r, 1.0, 1e-12);
    }

    #[test]
    fn unnormalized_is_covariance_like() {
        // Path 0-1-2 with values [1,2,1], unnormalized.
        // num1 = 1*2 + 2*1 = 4; /m=4/2 = 2
        // num2 = (1+2)+(2+1) = 6; /(2*2) = 1.5; ^2 = 2.25
        // r = 2 - 2.25 = -0.25
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let values = [1.0, 2.0, 1.0];
        let r = assortativity(&g, &values, None, None, false, false)
            .unwrap()
            .unwrap();
        assert_close(r, -0.25, 1e-12);
    }

    #[test]
    fn weights_reweight_edges() {
        // Path 0-1-2 with values [1,2,3]. Unweighted normalized r:
        //   num1 = 1*2 + 2*3 = 8; /2 = 4
        //   num2 = (1+2)+(2+3)=8; /(4) = 2; ^2 = 4
        //   den1 = (1+4)+(4+9)=18; /(4) = 4.5
        //   r = (4-4)/(4.5-4) = 0
        // With weights [3,1] the first edge dominates → different value.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let values = [1.0, 2.0, 3.0];
        let r_unw = assortativity(&g, &values, None, None, false, true)
            .unwrap()
            .unwrap();
        assert_close(r_unw, 0.0, 1e-12);
        let weights = [3.0, 1.0];
        let r_w = assortativity(&g, &values, None, Some(&weights), false, true)
            .unwrap()
            .unwrap();
        // With heavier weight on the (1,2) — wait edges are (0,1),(1,2);
        // weight 3 on (0,1). Just assert it differs and is finite.
        assert!(r_w.is_finite());
        assert!((r_w - r_unw).abs() > 1e-9);
    }

    #[test]
    fn directed_uses_source_and_target_values() {
        // 0→1, 0→2, 1→2 with source values = out-degree [2,1,0] and
        // target values_in = in-degree [0,1,2] reproduces the directed
        // degree assortativity result of -0.5.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let out_values = [2.0, 1.0, 0.0];
        let in_values = [0.0, 1.0, 2.0];
        let r = assortativity(&g, &out_values, Some(&in_values), None, true, true)
            .unwrap()
            .unwrap();
        assert_close(r, -0.5, 1e-12);
    }

    #[test]
    fn directed_flag_ignored_on_undirected_graph() {
        // directed=true on an undirected graph should behave like the
        // undirected formula (values_in ignored).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let values = [1.0, 2.0, 1.0];
        let a = assortativity(&g, &values, None, None, false, true).unwrap();
        let b = assortativity(&g, &values, None, None, true, true).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn directed_without_values_in_reuses_values() {
        // Passing values_in=None for a directed graph reuses `values`
        // for both endpoints.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let values = [1.0, 2.0, 3.0, 4.0];
        let r = assortativity(&g, &values, None, None, true, true).unwrap();
        assert!(r.is_some());
    }
}
