//! Graph signal diffusion operators (ALGO-TR-011).
//!
//! Core primitives for propagating signals on graphs, widely used in GNNs
//! (GCN, APPNP, GPR-GNN), graph semi-supervised learning, and graph signal
//! processing.
//!
//! - **Heat kernel diffusion**: approximates `exp(-t·L)·signal` via truncated
//!   Taylor series, where `L = I - D⁻¹A` is the normalized Laplacian.
//! - **PPR diffusion**: computes the Personalized `PageRank` propagation
//!   `α·(I - (1-α)·D⁻¹A)⁻¹·signal` via power iteration.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Diffuse a signal on the graph using the heat kernel.
///
/// Approximates `exp(-t·L)·signal` where `L = I - D⁻¹A` is the random-walk
/// normalized Laplacian. Uses a truncated Taylor expansion:
/// `exp(-t·L) ≈ Σ_{k=0}^{K} (-t)^k / k! · L^k`
///
/// Equivalently this iterates: `s_{k+1} = D⁻¹A · s_k` and accumulates
/// the weighted sum `Σ (t^k · e^{-t} / k!) · s_k` (Poisson weights).
///
/// # Parameters
///
/// - `graph` — Undirected graph.
/// - `signal` — Input signal of length `vcount`.
/// - `t` — Diffusion time (heat parameter, t > 0). Larger = more smoothing.
/// - `max_terms` — Maximum number of Taylor terms (default: 20).
///
/// # Returns
///
/// The diffused signal as `Vec<f64>` of length `vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, heat_kernel_diffuse};
///
/// // Path graph: signal on first vertex spreads to neighbors
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let signal = vec![1.0, 0.0, 0.0, 0.0];
/// let diffused = heat_kernel_diffuse(&g, &signal, 1.0, 20).unwrap();
/// // Signal should have spread: vertex 0 < 1.0, vertex 1 > 0.0
/// assert!(diffused[0] < 1.0);
/// assert!(diffused[1] > 0.0);
/// // Total signal is approximately preserved (heat kernel is doubly stochastic
/// // on regular graphs, approximately preserved on irregular ones)
/// ```
pub fn heat_kernel_diffuse(
    graph: &Graph,
    signal: &[f64],
    t: f64,
    max_terms: usize,
) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {}",
            signal.len(),
            nv
        )));
    }

    if t <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "diffusion time t must be positive".to_string(),
        ));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "heat_kernel_diffuse is defined for undirected graphs only".to_string(),
        ));
    }

    let degrees = compute_degrees(graph, nv)?;

    // Poisson-weighted accumulation: result = Σ_{k=0}^{K} w_k · s_k
    // where w_k = t^k · e^{-t} / k! and s_k = (D⁻¹A)^k · signal
    let mut current = signal.to_vec();
    let mut result = vec![0.0_f64; nv];

    let mut poisson_weight = (-t).exp(); // w_0 = e^{-t}
    add_scaled(&mut result, &current, poisson_weight);

    for k in 1..=max_terms {
        current = apply_transition(graph, &current, &degrees, nv)?;
        poisson_weight *= t / k as f64;
        add_scaled(&mut result, &current, poisson_weight);

        if poisson_weight.abs() < 1e-15 {
            break;
        }
    }

    Ok(result)
}

/// Diffuse a signal using Personalized `PageRank` propagation.
///
/// Computes the APPNP-style propagation:
/// `output = α·signal + (1-α)·D⁻¹A·output` (solved by power iteration).
///
/// This converges to `α·(I - (1-α)·D⁻¹A)⁻¹·signal`, the Personalized
/// `PageRank` vector with `signal` as the teleportation distribution.
///
/// # Parameters
///
/// - `graph` — Undirected graph.
/// - `signal` — Input signal of length `vcount`.
/// - `alpha` — Teleportation probability (0 < alpha ≤ 1). Typical: 0.1–0.2.
/// - `max_iter` — Maximum power iterations (default: 50).
/// - `tol` — Convergence tolerance on L∞ change (default: 1e-6).
///
/// # Returns
///
/// The diffused signal as `Vec<f64>` of length `vcount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, ppr_diffuse};
///
/// // Complete graph: PPR with uniform signal stays uniform
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let signal = vec![0.25, 0.25, 0.25, 0.25];
/// let diffused = ppr_diffuse(&g, &signal, 0.15, 50, 1e-6).unwrap();
/// for &v in &diffused {
///     assert!((v - 0.25).abs() < 1e-6);
/// }
/// ```
pub fn ppr_diffuse(
    graph: &Graph,
    signal: &[f64],
    alpha: f64,
    max_iter: usize,
    tol: f64,
) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {}",
            signal.len(),
            nv
        )));
    }

    if alpha <= 0.0 || alpha > 1.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "alpha must be in (0.0, 1.0], got {alpha}"
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "ppr_diffuse is defined for undirected graphs only".to_string(),
        ));
    }

    let degrees = compute_degrees(graph, nv)?;
    let one_minus_alpha = 1.0 - alpha;

    // Power iteration: z_{k+1} = α·signal + (1-α)·D⁻¹A·z_k
    let mut z = signal.to_vec();

    for _ in 0..max_iter {
        let propagated = apply_transition(graph, &z, &degrees, nv)?;

        let mut max_diff = 0.0_f64;
        for i in 0..nv {
            let new_val = alpha * signal[i] + one_minus_alpha * propagated[i];
            let diff = (new_val - z[i]).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            z[i] = new_val;
        }

        if max_diff < tol {
            break;
        }
    }

    Ok(z)
}

/// Diffuse a signal using symmetric normalized propagation.
///
/// Computes `D^{-1/2} A D^{-1/2} · signal`, the GCN-style propagation
/// operator (Kipf & Welling, 2017). Can be iterated `k` times for
/// multi-hop smoothing.
///
/// # Parameters
///
/// - `graph` — Undirected graph.
/// - `signal` — Input signal of length `vcount`.
/// - `k` — Number of propagation steps.
///
/// # Returns
///
/// The propagated signal after `k` steps.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, symmetric_diffuse};
///
/// // Triangle: uniform signal is an eigenvector, stays unchanged
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let signal = vec![1.0, 1.0, 1.0];
/// let out = symmetric_diffuse(&g, &signal, 1).unwrap();
/// for &v in &out {
///     assert!((v - 1.0).abs() < 1e-10);
/// }
/// ```
pub fn symmetric_diffuse(graph: &Graph, signal: &[f64], k: usize) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;

    if signal.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "signal length {} does not match vcount {}",
            signal.len(),
            nv
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "symmetric_diffuse is defined for undirected graphs only".to_string(),
        ));
    }

    let degrees = compute_degrees(graph, nv)?;

    // Precompute D^{-1/2}
    let inv_sqrt_deg: Vec<f64> = degrees
        .iter()
        .map(|&d| if d == 0 { 0.0 } else { 1.0 / (d as f64).sqrt() })
        .collect();

    let mut current = signal.to_vec();

    for _ in 0..k {
        // Apply D^{-1/2} A D^{-1/2}:
        // For vertex v: result[v] = Σ_{u∈N(v)} inv_sqrt_deg[v] * inv_sqrt_deg[u] * current[u]
        let mut next = vec![0.0_f64; nv];

        for v in 0..nv {
            if degrees[v] == 0 {
                continue;
            }
            let neighbors = graph.neighbors(v as VertexId)?;
            let mut sum = 0.0;
            for &u in &neighbors {
                let u_idx = u as usize;
                sum += inv_sqrt_deg[u_idx] * current[u_idx];
            }
            next[v] = inv_sqrt_deg[v] * sum;
        }

        current = next;
    }

    Ok(current)
}

// --- Internal helpers ---

fn compute_degrees(graph: &Graph, nv: usize) -> IgraphResult<Vec<usize>> {
    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }
    Ok(degrees)
}

/// Apply the random walk transition matrix D⁻¹A to a vector.
fn apply_transition(
    graph: &Graph,
    signal: &[f64],
    degrees: &[usize],
    nv: usize,
) -> IgraphResult<Vec<f64>> {
    let mut result = vec![0.0_f64; nv];

    for v in 0..nv {
        if signal[v] == 0.0 {
            continue;
        }
        let deg = degrees[v];
        if deg == 0 {
            result[v] += signal[v];
            continue;
        }
        let weight = signal[v] / deg as f64;
        let neighbors = graph.neighbors(v as VertexId)?;
        for &u in &neighbors {
            result[u as usize] += weight;
        }
    }

    Ok(result)
}

fn add_scaled(target: &mut [f64], source: &[f64], scale: f64) {
    for (t, &s) in target.iter_mut().zip(source.iter()) {
        *t += s * scale;
    }
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

    fn complete4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    // --- heat_kernel_diffuse tests ---

    #[test]
    fn heat_zero_signal() {
        let g = path4();
        let signal = vec![0.0; 4];
        let result = heat_kernel_diffuse(&g, &signal, 1.0, 20).unwrap();
        for &v in &result {
            assert!((v).abs() < 1e-15);
        }
    }

    #[test]
    fn heat_uniform_signal_regular() {
        let g = triangle();
        let signal = vec![1.0, 1.0, 1.0];
        let result = heat_kernel_diffuse(&g, &signal, 2.0, 30).unwrap();
        for &v in &result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn heat_signal_spreads() {
        let g = path4();
        let signal = vec![1.0, 0.0, 0.0, 0.0];
        let result = heat_kernel_diffuse(&g, &signal, 1.0, 20).unwrap();
        assert!(result[0] < 1.0);
        assert!(result[1] > 0.0);
        assert!(result[2] > 0.0);
    }

    #[test]
    fn heat_nonnegative() {
        let g = path4();
        let signal = vec![1.0, 0.0, 0.0, 0.0];
        let result = heat_kernel_diffuse(&g, &signal, 0.5, 20).unwrap();
        for &v in &result {
            assert!(v >= -1e-15);
        }
    }

    #[test]
    fn heat_invalid_signal_length() {
        let g = path4();
        assert!(heat_kernel_diffuse(&g, &[1.0, 2.0], 1.0, 20).is_err());
    }

    #[test]
    fn heat_invalid_t() {
        let g = path4();
        assert!(heat_kernel_diffuse(&g, &[0.0; 4], 0.0, 20).is_err());
        assert!(heat_kernel_diffuse(&g, &[0.0; 4], -1.0, 20).is_err());
    }

    #[test]
    fn heat_directed_error() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        assert!(heat_kernel_diffuse(&g, &[1.0; 3], 1.0, 20).is_err());
    }

    #[test]
    fn heat_isolated_vertex() {
        let g = Graph::with_vertices(3);
        let signal = vec![1.0, 2.0, 3.0];
        let result = heat_kernel_diffuse(&g, &signal, 1.0, 20).unwrap();
        for i in 0..3 {
            assert!((result[i] - signal[i]).abs() < 1e-10);
        }
    }

    // --- ppr_diffuse tests ---

    #[test]
    fn ppr_uniform_on_regular() {
        let g = complete4();
        let signal = vec![0.25, 0.25, 0.25, 0.25];
        let result = ppr_diffuse(&g, &signal, 0.15, 50, 1e-10).unwrap();
        for &v in &result {
            assert!((v - 0.25).abs() < 1e-8);
        }
    }

    #[test]
    fn ppr_signal_spreads() {
        let g = path4();
        let signal = vec![1.0, 0.0, 0.0, 0.0];
        let result = ppr_diffuse(&g, &signal, 0.15, 50, 1e-10).unwrap();
        // Signal concentrated at vertex 0 spreads to neighbors
        assert!(result[0] > 0.0);
        assert!(result[1] > 0.0);
        // Distant vertices get less signal
        assert!(result[1] > result[3]);
        assert!(result[2] > result[3]);
    }

    #[test]
    fn ppr_alpha_one_is_identity() {
        let g = path4();
        let signal = vec![1.0, 2.0, 3.0, 4.0];
        let result = ppr_diffuse(&g, &signal, 1.0, 50, 1e-10).unwrap();
        for i in 0..4 {
            assert!((result[i] - signal[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn ppr_nonnegative() {
        let g = path4();
        let signal = vec![1.0, 0.0, 0.0, 0.0];
        let result = ppr_diffuse(&g, &signal, 0.2, 50, 1e-10).unwrap();
        for &v in &result {
            assert!(v >= -1e-15);
        }
    }

    #[test]
    fn ppr_invalid_alpha() {
        let g = path4();
        assert!(ppr_diffuse(&g, &[0.0; 4], 0.0, 50, 1e-6).is_err());
        assert!(ppr_diffuse(&g, &[0.0; 4], 1.5, 50, 1e-6).is_err());
        assert!(ppr_diffuse(&g, &[0.0; 4], -0.1, 50, 1e-6).is_err());
    }

    #[test]
    fn ppr_invalid_signal_length() {
        let g = path4();
        assert!(ppr_diffuse(&g, &[1.0], 0.15, 50, 1e-6).is_err());
    }

    #[test]
    fn ppr_directed_error() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        assert!(ppr_diffuse(&g, &[1.0; 3], 0.15, 50, 1e-6).is_err());
    }

    #[test]
    fn ppr_isolated_vertex() {
        let g = Graph::with_vertices(3);
        let signal = vec![1.0, 2.0, 3.0];
        let result = ppr_diffuse(&g, &signal, 0.15, 50, 1e-10).unwrap();
        for i in 0..3 {
            assert!((result[i] - signal[i]).abs() < 1e-10);
        }
    }

    // --- symmetric_diffuse tests ---

    #[test]
    fn symmetric_uniform_regular() {
        let g = triangle();
        let signal = vec![1.0, 1.0, 1.0];
        let result = symmetric_diffuse(&g, &signal, 3).unwrap();
        for &v in &result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn symmetric_zero_steps() {
        let g = path4();
        let signal = vec![1.0, 2.0, 3.0, 4.0];
        let result = symmetric_diffuse(&g, &signal, 0).unwrap();
        for i in 0..4 {
            assert!((result[i] - signal[i]).abs() < 1e-15);
        }
    }

    #[test]
    fn symmetric_signal_smooths() {
        let g = path4();
        let signal = vec![4.0, 0.0, 0.0, 0.0];
        let result = symmetric_diffuse(&g, &signal, 1).unwrap();
        // After one step, signal should have spread
        assert!(result[0] < 4.0);
        assert!(result[1] > 0.0);
    }

    #[test]
    fn symmetric_invalid_signal_length() {
        let g = path4();
        assert!(symmetric_diffuse(&g, &[1.0], 1).is_err());
    }

    #[test]
    fn symmetric_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(symmetric_diffuse(&g, &[1.0; 2], 1).is_err());
    }

    #[test]
    fn symmetric_isolated_vertex() {
        let g = Graph::with_vertices(2);
        let signal = vec![5.0, 3.0];
        let result = symmetric_diffuse(&g, &signal, 5).unwrap();
        // Isolated vertices: signal stays (no neighbors)
        assert!((result[0]).abs() < 1e-15);
        assert!((result[1]).abs() < 1e-15);
    }
}
