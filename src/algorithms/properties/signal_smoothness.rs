//! Graph signal smoothness metrics (ALGO-TR-019).
//!
//! Measures how smoothly a real-valued signal varies over a graph,
//! central to graph signal processing (GSP), GNN over-smoothing
//! analysis, and semi-supervised learning.
//!
//! - **Dirichlet energy**: `Σ_{(u,v)∈E} w(u,v) · (f(u) - f(v))²`
//! - **Total variation**: `Σ_{(u,v)∈E} w(u,v) · |f(u) - f(v)|`
//! - **Normalized Dirichlet energy**: Dirichlet energy divided by
//!   `Σ_v deg(v) · f(v)²` (the "Rayleigh quotient" form)
//! - **Smoothness ratio**: `1 - Dirichlet / (2 · Σ_{(u,v)} w · (f(u)² + f(v)²))`

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the Dirichlet energy of a signal on a graph.
///
/// `E_D(f) = Σ_{(u,v)∈E} w(u,v) · (f(u) - f(v))²`
///
/// Measures the total squared variation of the signal across edges.
/// A constant signal has Dirichlet energy zero; a signal that varies
/// wildly across edges has high energy.
///
/// # Parameters
///
/// - `graph` — The input graph (directed or undirected).
/// - `signal` — Signal values, one per vertex.
/// - `weights` — Optional edge weights. If `None`, each edge has weight 1.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dirichlet_energy};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// // Constant signal → energy = 0
/// let e = dirichlet_energy(&g, &[5.0, 5.0, 5.0, 5.0], None).unwrap();
/// assert!(e.abs() < 1e-10);
/// // Step signal → energy > 0
/// let e2 = dirichlet_energy(&g, &[0.0, 0.0, 1.0, 1.0], None).unwrap();
/// assert!((e2 - 1.0).abs() < 1e-10);
/// ```
pub fn dirichlet_energy(
    graph: &Graph,
    signal: &[f64],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    validate_inputs(graph, signal, weights)?;

    let mut energy = 0.0_f64;
    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let diff = signal[u as usize] - signal[v as usize];
        energy += w * diff * diff;
    }

    Ok(energy)
}

/// Compute the total variation of a signal on a graph.
///
/// `TV(f) = Σ_{(u,v)∈E} w(u,v) · |f(u) - f(v)|`
///
/// The L1 analog of Dirichlet energy, less sensitive to outlier edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, total_variation};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let tv = total_variation(&g, &[0.0, 1.0, 3.0], None).unwrap();
/// assert!((tv - 3.0).abs() < 1e-10); // |0-1| + |1-3| = 1 + 2
/// ```
pub fn total_variation(
    graph: &Graph,
    signal: &[f64],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    validate_inputs(graph, signal, weights)?;

    let mut tv = 0.0_f64;
    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        tv += w * (signal[u as usize] - signal[v as usize]).abs();
    }

    Ok(tv)
}

/// Compute the normalized Dirichlet energy (Rayleigh quotient form).
///
/// `E_norm = Σ_{(u,v)∈E} w(u,v)·(f(u)-f(v))² / Σ_v deg(v)·f(v)²`
///
/// This equals the Rayleigh quotient `f^T L f / f^T D f` of the graph
/// Laplacian, bounded in `[0, 2]` for undirected graphs. Values near 0
/// indicate smooth signals; values near 2 indicate maximally varying signals.
///
/// Returns `0.0` if the denominator is zero (all-zero signal or isolated
/// vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, normalized_dirichlet_energy};
///
/// // Constant non-zero signal → 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let e = normalized_dirichlet_energy(&g, &[3.0, 3.0, 3.0], None).unwrap();
/// assert!(e.abs() < 1e-10);
/// ```
pub fn normalized_dirichlet_energy(
    graph: &Graph,
    signal: &[f64],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    validate_inputs(graph, signal, weights)?;

    let nv = graph.vcount() as usize;

    let mut numerator = 0.0_f64;
    let mut deg_weighted = vec![0.0_f64; nv];

    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let diff = signal[u as usize] - signal[v as usize];
        numerator += w * diff * diff;
        deg_weighted[u as usize] += w;
        deg_weighted[v as usize] += w;
    }

    let mut denominator = 0.0_f64;
    for v in 0..nv {
        denominator += deg_weighted[v] * signal[v] * signal[v];
    }

    if denominator.abs() < 1e-300 {
        return Ok(0.0);
    }

    Ok(numerator / denominator)
}

/// Compute the smoothness ratio of a signal on a graph.
///
/// `SR = 1 - E_D / (2 · Σ_{(u,v)} w · (f(u)² + f(v)²))`
///
/// Bounded in `[0, 1]` for non-negative weights. A value of 1 means the
/// signal is perfectly smooth (constant on each connected component);
/// a value of 0 means maximally rough.
///
/// Returns `1.0` if the denominator is zero (no edges or all-zero signal).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, smoothness_ratio};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// // Constant signal → perfectly smooth
/// let sr = smoothness_ratio(&g, &[2.0, 2.0, 2.0], None).unwrap();
/// assert!((sr - 1.0).abs() < 1e-10);
/// ```
pub fn smoothness_ratio(
    graph: &Graph,
    signal: &[f64],
    weights: Option<&[f64]>,
) -> IgraphResult<f64> {
    validate_inputs(graph, signal, weights)?;

    let mut dirichlet = 0.0_f64;
    let mut sum_sq = 0.0_f64;

    for (eid, (u, v)) in graph.edges().enumerate() {
        let w = weights.map_or(1.0, |ws| ws[eid]);
        let fu = signal[u as usize];
        let fv = signal[v as usize];
        dirichlet += w * (fu - fv) * (fu - fv);
        sum_sq += w * (fu * fu + fv * fv);
    }

    let denom = 2.0 * sum_sq;
    if denom.abs() < 1e-300 {
        return Ok(1.0);
    }

    Ok(1.0 - dirichlet / denom)
}

// --- Internal helpers ---

fn validate_inputs(graph: &Graph, signal: &[f64], weights: Option<&[f64]>) -> IgraphResult<()> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {nv}",
            signal.len()
        )));
    }

    if let Some(w) = weights {
        if w.len() != ne {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length {} does not match ecount {ne}",
                w.len()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    // --- dirichlet_energy tests ---

    #[test]
    fn de_constant_signal() {
        let g = path4();
        let e = dirichlet_energy(&g, &[3.0, 3.0, 3.0, 3.0], None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn de_step_signal() {
        let g = path4();
        // Signal: [0, 0, 1, 1] → only edge 1-2 contributes: (0-1)² = 1
        let e = dirichlet_energy(&g, &[0.0, 0.0, 1.0, 1.0], None).unwrap();
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn de_linear_signal() {
        let g = path4();
        // Signal: [0, 1, 2, 3] → each edge contributes 1² = 1, total = 3
        let e = dirichlet_energy(&g, &[0.0, 1.0, 2.0, 3.0], None).unwrap();
        assert!((e - 3.0).abs() < 1e-10);
    }

    #[test]
    fn de_weighted() {
        let g = path4();
        let w = vec![2.0, 1.0, 3.0];
        // Signal: [0, 1, 0, 1]
        // Edge 0-1: 2*(0-1)² = 2, Edge 1-2: 1*(1-0)² = 1, Edge 2-3: 3*(0-1)² = 3
        let e = dirichlet_energy(&g, &[0.0, 1.0, 0.0, 1.0], Some(&w)).unwrap();
        assert!((e - 6.0).abs() < 1e-10);
    }

    #[test]
    fn de_empty_graph() {
        let g = Graph::with_vertices(3);
        let e = dirichlet_energy(&g, &[1.0, 2.0, 3.0], None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn de_invalid_signal() {
        let g = path4();
        assert!(dirichlet_energy(&g, &[1.0], None).is_err());
    }

    #[test]
    fn de_invalid_weights() {
        let g = path4();
        assert!(dirichlet_energy(&g, &[0.0; 4], Some(&[1.0])).is_err());
    }

    #[test]
    fn de_directed() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        let e = dirichlet_energy(&g, &[0.0, 1.0, 3.0], None).unwrap();
        // Edge 0→1: (0-1)²=1, Edge 1→2: (1-3)²=4
        assert!((e - 5.0).abs() < 1e-10);
    }

    // --- total_variation tests ---

    #[test]
    fn tv_constant() {
        let g = triangle();
        let tv = total_variation(&g, &[5.0, 5.0, 5.0], None).unwrap();
        assert!(tv.abs() < 1e-10);
    }

    #[test]
    fn tv_step() {
        let g = path4();
        // [0, 0, 1, 1]: only |0-1| = 1
        let tv = total_variation(&g, &[0.0, 0.0, 1.0, 1.0], None).unwrap();
        assert!((tv - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tv_linear() {
        let g = path4();
        // [0, 1, 2, 3]: each |diff| = 1, total = 3
        let tv = total_variation(&g, &[0.0, 1.0, 2.0, 3.0], None).unwrap();
        assert!((tv - 3.0).abs() < 1e-10);
    }

    #[test]
    fn tv_nonneg() {
        let g = cycle4();
        let tv = total_variation(&g, &[1.0, -1.0, 2.0, -2.0], None).unwrap();
        assert!(tv >= 0.0);
    }

    // --- normalized_dirichlet_energy tests ---

    #[test]
    fn nde_constant() {
        let g = triangle();
        let e = normalized_dirichlet_energy(&g, &[3.0, 3.0, 3.0], None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn nde_zero_signal() {
        let g = triangle();
        let e = normalized_dirichlet_energy(&g, &[0.0, 0.0, 0.0], None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn nde_bounded() {
        let g = cycle4();
        // Alternating signal [1, -1, 1, -1] should give high energy
        let e = normalized_dirichlet_energy(&g, &[1.0, -1.0, 1.0, -1.0], None).unwrap();
        assert!(e >= -1e-10);
        assert!(e <= 2.0 + 1e-10);
    }

    #[test]
    fn nde_empty_graph() {
        let g = Graph::with_vertices(3);
        let e = normalized_dirichlet_energy(&g, &[1.0, 2.0, 3.0], None).unwrap();
        assert!(e.abs() < 1e-10);
    }

    // --- smoothness_ratio tests ---

    #[test]
    fn sr_constant() {
        let g = triangle();
        let sr = smoothness_ratio(&g, &[2.0, 2.0, 2.0], None).unwrap();
        assert!((sr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sr_bounded() {
        let g = cycle4();
        let sr = smoothness_ratio(&g, &[1.0, -1.0, 1.0, -1.0], None).unwrap();
        assert!(sr >= -1e-10);
        assert!(sr <= 1.0 + 1e-10);
    }

    #[test]
    fn sr_zero_signal() {
        let g = triangle();
        let sr = smoothness_ratio(&g, &[0.0, 0.0, 0.0], None).unwrap();
        assert!((sr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sr_empty_graph() {
        let g = Graph::with_vertices(3);
        let sr = smoothness_ratio(&g, &[1.0, 2.0, 3.0], None).unwrap();
        assert!((sr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sr_alternating_on_path() {
        let g = path4();
        // [1, -1, 1, -1]: maximally rough on path
        let sr = smoothness_ratio(&g, &[1.0, -1.0, 1.0, -1.0], None).unwrap();
        // Dirichlet = 3 * 4 = 12, sum_sq = 3 * 2 = 6, denom = 12
        // sr = 1 - 12/12 = 0
        assert!(sr.abs() < 1e-10);
    }

    // --- cross-consistency tests ---

    #[test]
    fn de_equals_tv_squared_for_unit_step() {
        let g = path4();
        let signal = [0.0, 0.0, 1.0, 1.0];
        let de = dirichlet_energy(&g, &signal, None).unwrap();
        let tv = total_variation(&g, &signal, None).unwrap();
        // For a signal with diffs in {0, ±1}, DE = TV
        assert!((de - tv).abs() < 1e-10);
    }

    #[test]
    fn de_geq_zero() {
        let g = cycle4();
        for signal in &[
            vec![1.0, 2.0, 3.0, 4.0],
            vec![-1.0, 0.5, -0.3, 2.0],
            vec![0.0, 0.0, 0.0, 0.0],
        ] {
            let de = dirichlet_energy(&g, signal, None).unwrap();
            assert!(de >= -1e-15);
        }
    }
}
