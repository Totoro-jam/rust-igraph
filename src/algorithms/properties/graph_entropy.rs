//! Graph structural entropy and complexity measures (ALGO-TR-016).
//!
//! Information-theoretic descriptors of graph structure, used as
//! graph-level features in graph classification, complexity analysis,
//! and for characterizing random graph ensembles.
//!
//! - **Degree entropy**: Shannon entropy of the degree distribution
//! - **Edge entropy**: entropy based on edge endpoint degree product
//! - **Von Neumann entropy**: quantum graph entropy from normalized Laplacian eigenvalues
//!   (approximated via trace formula)
//! - **Structural information content**: log of the number of automorphisms
//!   approximation via degree sequence partition

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Shannon entropy of the degree distribution.
///
/// `H_degree = -Σ_k p(k) log2(p(k))`
///
/// where `p(k)` is the fraction of vertices with degree `k`.
/// Higher entropy indicates a more heterogeneous degree distribution.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_entropy};
///
/// // Regular graph (cycle): all degrees equal → entropy = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// let h = degree_entropy(&g).unwrap();
/// assert!(h.abs() < 1e-10);
/// ```
pub fn degree_entropy(graph: &Graph) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    if nv == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(nv);
    let mut max_deg = 0usize;
    for v in 0..nv {
        let d = graph.degree(v as VertexId)?;
        if d > max_deg {
            max_deg = d;
        }
        degrees.push(d);
    }

    // Build degree histogram
    let mut hist = vec![0usize; max_deg + 1];
    for &d in &degrees {
        hist[d] += 1;
    }

    // Compute Shannon entropy
    let n_f64 = nv as f64;
    let mut entropy = 0.0;
    for &count in &hist {
        if count > 0 {
            let p = count as f64 / n_f64;
            entropy -= p * p.log2();
        }
    }

    Ok(entropy)
}

/// Shannon entropy of the edge-degree distribution.
///
/// `H_edge = -Σ_e p(e) log2(p(e))`
///
/// where `p(e)` is the normalized weight of edge `e` defined as
/// `1 / (deg(u) * deg(v))` divided by the sum of all such weights.
/// Captures how "uniformly" connectivity is distributed across edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_entropy};
///
/// // Star graph: heterogeneous edge weights → lower entropy than regular
/// let star = Graph::from_edges(&[(0,1),(0,2),(0,3)], false, Some(4)).unwrap();
/// let h = edge_entropy(&star).unwrap();
/// assert!(h > 0.0);
/// ```
pub fn edge_entropy(graph: &Graph) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    let ne = graph.ecount();
    if ne == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    // Compute unnormalized weights
    let mut weights = Vec::with_capacity(ne);
    let mut total_weight = 0.0;
    for (u, v) in graph.edges() {
        let du = degrees[u as usize];
        let dv = degrees[v as usize];
        let w = if du > 0 && dv > 0 {
            1.0 / (du as f64 * dv as f64)
        } else {
            0.0
        };
        weights.push(w);
        total_weight += w;
    }

    if total_weight <= 0.0 {
        return Ok(0.0);
    }

    // Compute entropy of normalized distribution
    let mut entropy = 0.0;
    for &w in &weights {
        if w > 0.0 {
            let p = w / total_weight;
            entropy -= p * p.log2();
        }
    }

    Ok(entropy)
}

/// Approximate Von Neumann entropy of the graph.
///
/// The Von Neumann entropy is `S = -Σ_i λ_i log2(λ_i)` where `λ_i` are
/// the eigenvalues of the density matrix `ρ = L / trace(L)` with `L` being
/// the combinatorial Laplacian.
///
/// Since `trace(L) = 2|E|` and eigenvalues sum to `trace(L)`,
/// we use the quadratic approximation:
/// `S ≈ 1 - (1/|V|) - (1/(2|E|)²) Σ_v deg(v)²`  (in bits)
///
/// This avoids expensive eigendecomposition while capturing the essential
/// structural complexity.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, von_neumann_entropy};
///
/// // Complete graph has high structural complexity
/// let k4 = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let h = von_neumann_entropy(&k4).unwrap();
/// assert!(h > 0.0);
/// ```
pub fn von_neumann_entropy(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "von_neumann_entropy is defined for undirected graphs only".to_string(),
        ));
    }

    let nv = graph.vcount() as usize;
    let ne = graph.ecount();

    if nv == 0 || ne == 0 {
        return Ok(0.0);
    }

    let mut sum_deg_sq = 0.0_f64;
    for v in 0..nv {
        let d = graph.degree(v as VertexId)? as f64;
        sum_deg_sq += d * d;
    }

    let two_m = 2.0 * ne as f64;

    // Normalized Laplacian eigenvalue sum of squares:
    // Σ λ_i² = Σ_v (1 + deg(v)²/(2m)) for normalized Laplacian
    // For the density matrix ρ = L_norm / n:
    // S ≈ 1 - 1/n - (1/(4m²)) * Σ_v deg(v)²
    let entropy = 1.0 - 1.0 / nv as f64 - sum_deg_sq / (two_m * two_m);

    // Clamp to non-negative (approximation can go slightly negative for sparse graphs)
    Ok(entropy.max(0.0))
}

/// Structural information content based on degree sequence.
///
/// `I = log2(|V|!) - Σ_k log2(n_k!)`
///
/// where `n_k` is the number of vertices with degree `k`.
/// This approximates the logarithm of the automorphism group size
/// from the degree partition alone.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_structural_info};
///
/// // Regular graph: all vertices equivalent → I = 0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// let info = degree_structural_info(&g).unwrap();
/// assert!(info.abs() < 1e-10);
/// ```
pub fn degree_structural_info(graph: &Graph) -> IgraphResult<f64> {
    let nv = graph.vcount() as usize;
    if nv <= 1 {
        return Ok(0.0);
    }

    let mut max_deg = 0usize;
    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        let d = graph.degree(v as VertexId)?;
        if d > max_deg {
            max_deg = d;
        }
        degrees.push(d);
    }

    // Build degree histogram
    let mut hist = vec![0usize; max_deg + 1];
    for &d in &degrees {
        hist[d] += 1;
    }

    // I = log2(n!) - Σ_k log2(n_k!)
    let log_n_fact = log2_factorial(nv);
    let mut sum_log_nk_fact = 0.0;
    for &count in &hist {
        if count > 1 {
            sum_log_nk_fact += log2_factorial(count);
        }
    }

    Ok(log_n_fact - sum_log_nk_fact)
}

// --- Internal helpers ---

fn log2_factorial(n: usize) -> f64 {
    // Stirling for large n, exact sum for small n
    if n <= 1 {
        return 0.0;
    }
    if n <= 20 {
        let mut result = 0.0;
        for i in 2..=n {
            result += (i as f64).log2();
        }
        return result;
    }
    // Stirling's approximation: log2(n!) ≈ n*log2(n) - n*log2(e) + 0.5*log2(2πn)
    let nf = n as f64;
    nf * nf.log2() - nf * std::f64::consts::E.log2()
        + 0.5 * (2.0 * std::f64::consts::PI * nf).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    fn complete4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    // --- degree_entropy tests ---

    #[test]
    fn degree_entropy_regular() {
        let g = cycle4();
        let h = degree_entropy(&g).unwrap();
        // All degrees equal → single bin → entropy = 0
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn degree_entropy_star() {
        let g = star4();
        let h = degree_entropy(&g).unwrap();
        // Degrees: [3, 1, 1, 1] → p(1)=3/4, p(3)=1/4
        let expected = -(3.0 / 4.0) * (3.0_f64 / 4.0).log2() - (1.0 / 4.0) * (1.0_f64 / 4.0).log2();
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn degree_entropy_path() {
        let g = path4();
        let h = degree_entropy(&g).unwrap();
        // Degrees: [1, 2, 2, 1] → p(1)=2/4, p(2)=2/4
        let expected = -2.0 * (0.5 * 0.5_f64.log2());
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn degree_entropy_empty() {
        let g = Graph::with_vertices(0);
        let h = degree_entropy(&g).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn degree_entropy_isolated() {
        let g = Graph::with_vertices(5);
        let h = degree_entropy(&g).unwrap();
        // All degree 0 → single bin → entropy = 0
        assert!(h.abs() < 1e-10);
    }

    // --- edge_entropy tests ---

    #[test]
    fn edge_entropy_regular() {
        let g = cycle4();
        let h = edge_entropy(&g).unwrap();
        // All edges have same weight → uniform → entropy = log2(4)
        let expected = 4.0_f64.log2();
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn edge_entropy_empty() {
        let g = Graph::with_vertices(3);
        let h = edge_entropy(&g).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn edge_entropy_positive() {
        let g = star4();
        let h = edge_entropy(&g).unwrap();
        assert!(h > 0.0);
    }

    // --- von_neumann_entropy tests ---

    #[test]
    fn vne_positive_for_complex() {
        let g = complete4();
        let h = von_neumann_entropy(&g).unwrap();
        assert!(h > 0.0);
    }

    #[test]
    fn vne_empty() {
        let g = Graph::with_vertices(3);
        let h = von_neumann_entropy(&g).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn vne_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(von_neumann_entropy(&g).is_err());
    }

    #[test]
    fn vne_non_negative() {
        let g = path4();
        let h = von_neumann_entropy(&g).unwrap();
        assert!(h >= 0.0);
    }

    // --- degree_structural_info tests ---

    #[test]
    fn dsi_regular_zero() {
        let g = cycle4();
        let info = degree_structural_info(&g).unwrap();
        // All vertices same degree → info = 0
        assert!(info.abs() < 1e-10);
    }

    #[test]
    fn dsi_star_positive() {
        let g = star4();
        let info = degree_structural_info(&g).unwrap();
        // Degrees [3,1,1,1]: log2(4!) - log2(3!) - log2(1!)
        let expected = log2_factorial(4) - log2_factorial(3);
        assert!((info - expected).abs() < 1e-10);
    }

    #[test]
    fn dsi_all_different() {
        // Path of 4: degrees [1,2,2,1] → log2(4!) - log2(2!) - log2(2!)
        let g = path4();
        let info = degree_structural_info(&g).unwrap();
        let expected = log2_factorial(4) - 2.0 * log2_factorial(2);
        assert!((info - expected).abs() < 1e-10);
    }

    #[test]
    fn dsi_single_vertex() {
        let g = Graph::with_vertices(1);
        let info = degree_structural_info(&g).unwrap();
        assert!(info.abs() < 1e-10);
    }

    #[test]
    fn dsi_empty() {
        let g = Graph::with_vertices(0);
        let info = degree_structural_info(&g).unwrap();
        assert!(info.abs() < 1e-10);
    }
}
