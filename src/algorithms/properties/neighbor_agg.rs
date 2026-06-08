//! Neighborhood aggregation operators (ALGO-TR-020).
//!
//! Core primitives for message-passing / GNN-style computation on graphs.
//! Each operator computes, for every vertex, an aggregate of a signal over
//! its neighbors:
//!
//! - **Mean aggregation**: `agg(v) = (1/deg(v)) · Σ_{u∈N(v)} f(u)`
//! - **Sum aggregation**: `agg(v) = Σ_{u∈N(v)} f(u)`
//! - **Max aggregation**: `agg(v) = max_{u∈N(v)} f(u)`
//! - **Min aggregation**: `agg(v) = min_{u∈N(v)} f(u)`
//! - **Attention-weighted aggregation**: `agg(v) = Σ_{u∈N(v)} α(v,u) · f(u)`
//!   where `α(v,u)` is normalized via softmax over `N(v)`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::needless_range_loop
)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Aggregation mode for neighborhood operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggMode {
    /// Mean over neighbors: `Σ f(u) / deg(v)`.
    Mean,
    /// Sum over neighbors: `Σ f(u)`.
    Sum,
    /// Maximum over neighbors.
    Max,
    /// Minimum over neighbors.
    Min,
}

/// Aggregate a signal over each vertex's neighborhood.
///
/// For each vertex `v`, computes an aggregate of `signal[u]` over all
/// neighbors `u ∈ N(v)` using the specified mode. Isolated vertices
/// receive `0.0` for `Mean`/`Sum`, `f64::NEG_INFINITY` for `Max`,
/// and `f64::INFINITY` for `Min`.
///
/// # Parameters
///
/// - `graph` — The input graph (undirected).
/// - `signal` — Input signal of length `vcount`.
/// - `mode` — Aggregation mode (`Mean`, `Sum`, `Max`, `Min`).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, AggMode, neighbor_aggregate};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let signal = vec![1.0, 2.0, 3.0];
/// let mean_agg = neighbor_aggregate(&g, &signal, AggMode::Mean).unwrap();
/// // Vertex 0: neighbors {1,2}, mean = (2+3)/2 = 2.5
/// assert!((mean_agg[0] - 2.5).abs() < 1e-10);
/// let sum_agg = neighbor_aggregate(&g, &signal, AggMode::Sum).unwrap();
/// // Vertex 0: sum = 2+3 = 5
/// assert!((sum_agg[0] - 5.0).abs() < 1e-10);
/// ```
pub fn neighbor_aggregate(graph: &Graph, signal: &[f64], mode: AggMode) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {nv}",
            signal.len()
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "neighbor_aggregate is defined for undirected graphs only".to_string(),
        ));
    }

    let mut result = match mode {
        AggMode::Mean | AggMode::Sum => vec![0.0_f64; nv],
        AggMode::Max => vec![f64::NEG_INFINITY; nv],
        AggMode::Min => vec![f64::INFINITY; nv],
    };

    for (u, v) in graph.edges() {
        let ui = u as usize;
        let vi = v as usize;

        match mode {
            AggMode::Mean | AggMode::Sum => {
                result[ui] += signal[vi];
                result[vi] += signal[ui];
            }
            AggMode::Max => {
                if signal[vi] > result[ui] {
                    result[ui] = signal[vi];
                }
                if signal[ui] > result[vi] {
                    result[vi] = signal[ui];
                }
            }
            AggMode::Min => {
                if signal[vi] < result[ui] {
                    result[ui] = signal[vi];
                }
                if signal[ui] < result[vi] {
                    result[vi] = signal[ui];
                }
            }
        }
    }

    if mode == AggMode::Mean {
        for v in 0..nv {
            let deg = graph.degree(v as VertexId)?;
            if deg > 0 {
                result[v] /= deg as f64;
            }
        }
    }

    // Fix isolated vertices for Max/Min
    if matches!(mode, AggMode::Max | AggMode::Min) {
        for v in 0..nv {
            let deg = graph.degree(v as VertexId)?;
            if deg == 0 {
                result[v] = 0.0;
            }
        }
    }

    Ok(result)
}

/// Aggregate a signal with per-edge attention weights.
///
/// For each vertex `v`, computes
/// `agg(v) = Σ_{u∈N(v)} softmax(attn(v,u)) · signal(u)`
/// where `softmax` normalizes `attn` scores across `N(v)`.
///
/// # Parameters
///
/// - `graph` — Undirected graph.
/// - `signal` — Input signal of length `vcount`.
/// - `attention` — Raw attention scores, one per edge. Length must equal
///   `ecount`. For edge `(u,v)`, the score is used in both directions.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, attention_aggregate};
///
/// let g = Graph::from_edges(&[(0,1),(0,2)], false, Some(3)).unwrap();
/// let signal = vec![0.0, 1.0, 2.0];
/// // Equal attention → equivalent to mean
/// let agg = attention_aggregate(&g, &signal, &[0.0, 0.0]).unwrap();
/// assert!((agg[0] - 1.5).abs() < 1e-10); // mean(1, 2)
/// ```
pub fn attention_aggregate(
    graph: &Graph,
    signal: &[f64],
    attention: &[f64],
) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {nv}",
            signal.len()
        )));
    }

    if attention.len() != ne {
        return Err(IgraphError::InvalidArgument(format!(
            "attention length {} does not match ecount {ne}",
            attention.len()
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "attention_aggregate is defined for undirected graphs only".to_string(),
        ));
    }

    // Build per-vertex neighbor lists with attention scores
    let mut neighbor_scores: Vec<Vec<(usize, f64, f64)>> = vec![Vec::new(); nv];
    for (eid, (u, v)) in graph.edges().enumerate() {
        let ui = u as usize;
        let vi = v as usize;
        let attn = attention[eid];
        neighbor_scores[ui].push((vi, attn, signal[vi]));
        neighbor_scores[vi].push((ui, attn, signal[ui]));
    }

    // Softmax + weighted sum per vertex
    let mut result = vec![0.0_f64; nv];
    for v in 0..nv {
        let neighbors = &neighbor_scores[v];
        if neighbors.is_empty() {
            continue;
        }

        // Stable softmax: subtract max for numerical stability
        let max_attn = neighbors
            .iter()
            .map(|&(_, a, _)| a)
            .fold(f64::NEG_INFINITY, f64::max);

        let mut sum_exp = 0.0_f64;
        let exps: Vec<f64> = neighbors
            .iter()
            .map(|&(_, a, _)| {
                let e = (a - max_attn).exp();
                sum_exp += e;
                e
            })
            .collect();

        if sum_exp > 0.0 {
            for (i, &(_, _, sig)) in neighbors.iter().enumerate() {
                result[v] += (exps[i] / sum_exp) * sig;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    // --- neighbor_aggregate Mean tests ---

    #[test]
    fn mean_triangle() {
        let g = triangle();
        let s = vec![1.0, 2.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Mean).unwrap();
        assert!((r[0] - 2.5).abs() < 1e-10); // (2+3)/2
        assert!((r[1] - 2.0).abs() < 1e-10); // (1+3)/2
        assert!((r[2] - 1.5).abs() < 1e-10); // (1+2)/2
    }

    #[test]
    fn mean_isolated() {
        let g = Graph::with_vertices(3);
        let s = vec![1.0, 2.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Mean).unwrap();
        for &v in &r {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn mean_star() {
        let g = star4();
        let s = vec![0.0, 1.0, 2.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Mean).unwrap();
        assert!((r[0] - 2.0).abs() < 1e-10); // (1+2+3)/3
        assert!((r[1] - 0.0).abs() < 1e-10); // only neighbor is 0
        assert!((r[2] - 0.0).abs() < 1e-10);
        assert!((r[3] - 0.0).abs() < 1e-10);
    }

    // --- neighbor_aggregate Sum tests ---

    #[test]
    fn sum_triangle() {
        let g = triangle();
        let s = vec![1.0, 2.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Sum).unwrap();
        assert!((r[0] - 5.0).abs() < 1e-10); // 2+3
        assert!((r[1] - 4.0).abs() < 1e-10); // 1+3
        assert!((r[2] - 3.0).abs() < 1e-10); // 1+2
    }

    #[test]
    fn sum_path() {
        let g = path4();
        let s = vec![1.0, 2.0, 3.0, 4.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Sum).unwrap();
        assert!((r[0] - 2.0).abs() < 1e-10); // only neighbor 1
        assert!((r[1] - 4.0).abs() < 1e-10); // 1+3
        assert!((r[2] - 6.0).abs() < 1e-10); // 2+4
        assert!((r[3] - 3.0).abs() < 1e-10); // only neighbor 2
    }

    // --- neighbor_aggregate Max tests ---

    #[test]
    fn max_triangle() {
        let g = triangle();
        let s = vec![1.0, 5.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Max).unwrap();
        assert!((r[0] - 5.0).abs() < 1e-10); // max(5, 3)
        assert!((r[1] - 3.0).abs() < 1e-10); // max(1, 3)
        assert!((r[2] - 5.0).abs() < 1e-10); // max(1, 5)
    }

    #[test]
    fn max_isolated() {
        let g = Graph::with_vertices(2);
        let s = vec![10.0, 20.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Max).unwrap();
        assert!(r[0].abs() < 1e-10);
        assert!(r[1].abs() < 1e-10);
    }

    // --- neighbor_aggregate Min tests ---

    #[test]
    fn min_triangle() {
        let g = triangle();
        let s = vec![1.0, 5.0, 3.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Min).unwrap();
        assert!((r[0] - 3.0).abs() < 1e-10); // min(5, 3)
        assert!((r[1] - 1.0).abs() < 1e-10); // min(1, 3)
        assert!((r[2] - 1.0).abs() < 1e-10); // min(1, 5)
    }

    #[test]
    fn min_isolated() {
        let g = Graph::with_vertices(2);
        let s = vec![10.0, 20.0];
        let r = neighbor_aggregate(&g, &s, AggMode::Min).unwrap();
        assert!(r[0].abs() < 1e-10);
        assert!(r[1].abs() < 1e-10);
    }

    // --- error tests ---

    #[test]
    fn agg_invalid_signal() {
        let g = triangle();
        assert!(neighbor_aggregate(&g, &[1.0], AggMode::Mean).is_err());
    }

    #[test]
    fn agg_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(neighbor_aggregate(&g, &[1.0, 2.0], AggMode::Sum).is_err());
    }

    // --- attention_aggregate tests ---

    #[test]
    fn attn_equal_weights() {
        let g = Graph::from_edges(&[(0, 1), (0, 2)], false, Some(3)).unwrap();
        let s = vec![0.0, 1.0, 2.0];
        let r = attention_aggregate(&g, &s, &[0.0, 0.0]).unwrap();
        // Equal attention → mean
        assert!((r[0] - 1.5).abs() < 1e-10);
    }

    #[test]
    fn attn_dominant_weight() {
        let g = Graph::from_edges(&[(0, 1), (0, 2)], false, Some(3)).unwrap();
        let s = vec![0.0, 1.0, 2.0];
        // Very high attention to edge 0-1, low to 0-2
        let r = attention_aggregate(&g, &s, &[100.0, 0.0]).unwrap();
        // Vertex 0 should weight neighbor 1 much more
        assert!((r[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn attn_isolated() {
        let g = Graph::with_vertices(2);
        let s = vec![1.0, 2.0];
        let r = attention_aggregate(&g, &s, &[]).unwrap();
        assert!(r[0].abs() < 1e-10);
        assert!(r[1].abs() < 1e-10);
    }

    #[test]
    fn attn_invalid_signal() {
        let g = triangle();
        assert!(attention_aggregate(&g, &[1.0], &[0.0; 3]).is_err());
    }

    #[test]
    fn attn_invalid_attention() {
        let g = triangle();
        assert!(attention_aggregate(&g, &[0.0; 3], &[1.0]).is_err());
    }

    #[test]
    fn attn_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(attention_aggregate(&g, &[1.0, 2.0], &[0.0]).is_err());
    }

    // --- consistency tests ---

    #[test]
    fn sum_is_mean_times_degree() {
        let g = triangle();
        let s = vec![1.0, 2.0, 3.0];
        let mean = neighbor_aggregate(&g, &s, AggMode::Mean).unwrap();
        let sum = neighbor_aggregate(&g, &s, AggMode::Sum).unwrap();
        for v in 0..3 {
            let deg = g.degree(v as VertexId).unwrap() as f64;
            assert!((sum[v] - mean[v] * deg).abs() < 1e-10);
        }
    }

    #[test]
    fn constant_signal_mean_equals_constant() {
        let g = star4();
        let c = 7.0;
        let s = vec![c; 4];
        let r = neighbor_aggregate(&g, &s, AggMode::Mean).unwrap();
        for v in 0..4 {
            if g.degree(v as VertexId).unwrap() > 0 {
                assert!((r[v] - c).abs() < 1e-10);
            }
        }
    }
}
