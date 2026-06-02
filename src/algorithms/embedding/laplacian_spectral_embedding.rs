//! Laplacian spectral embedding (ALGO-EM-003).
//!
//! Computes an `no`-dimensional Euclidean representation of the graph
//! based on the spectral decomposition of one of several Laplacian
//! matrices.
//!
//! Three Laplacian types are supported for undirected graphs:
//!
//! - **D − A** (`LaplacianType::DA`): the combinatorial Laplacian.
//! - **D⁻¹ᐟ² A D⁻¹ᐟ²** (`LaplacianType::DAD`): the normalized
//!   adjacency (related to the symmetric normalised Laplacian by
//!   `L_sym = I − DAD`).
//! - **I − D⁻¹ᐟ² A D⁻¹ᐟ²** (`LaplacianType::IDAD`): the symmetric
//!   normalized Laplacian.
//!
//! Counterpart of `igraph_laplacian_spectral_embedding()` from
//! `references/igraph/src/misc/embedding.c:1071`.

use crate::algorithms::community::lanczos::{EigenWhich, lanczos_top_k};
use crate::algorithms::embedding::adjacency_spectral_embedding::SpectralWhich;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Which Laplacian variant to use for spectral embedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaplacianType {
    /// D − A: the combinatorial (unnormalized) Laplacian.
    DA,
    /// D⁻¹ᐟ² A D⁻¹ᐟ²: the normalized adjacency matrix.
    DAD,
    /// I − D⁻¹ᐟ² A D⁻¹ᐟ²: the symmetric normalized Laplacian.
    IDAD,
}

/// Result of Laplacian spectral embedding.
#[derive(Debug, Clone)]
pub struct LaplacianSpectralEmbeddingResult {
    /// Eigenvalues in the order selected by `which`.
    pub eigenvalues: Vec<f64>,
    /// Embedding matrix: `embedding[v]` is the `no`-dimensional embedding
    /// of vertex `v`. Each inner vector has length `no`.
    pub embedding: Vec<Vec<f64>>,
}

/// Compute the Laplacian spectral embedding of an undirected graph.
///
/// Decomposes a Laplacian matrix of the graph into its top eigenpairs
/// and returns the eigenvectors as vertex embeddings.
///
/// `no`: embedding dimension (number of eigenpairs to compute).
///
/// `weights`: optional edge weights. Must have length `ecount` if
/// provided. All weights must be non-negative.
///
/// `which`: which eigenvalues to use. See [`SpectralWhich`].
///
/// `lap_type`: which Laplacian variant. See [`LaplacianType`].
///
/// `scaled`: if true, multiply each eigenvector column by
/// `sqrt(|eigenvalue|)`, giving `X = U |D|^{1/2}`.
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// # Errors
///
/// Returns an error if `no` is 0 or exceeds `vcount`, if `weights`
/// has the wrong length or contains non-finite values, or if any
/// weight is negative (for `DAD`/`IDAD` types, which require
/// non-negative weights for the square-root degree normalization).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, laplacian_spectral_embedding, SpectralWhich, LaplacianType};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 3).unwrap();
///
/// let r = laplacian_spectral_embedding(&g, 2, None, SpectralWhich::LargestAlgebraic,
///                                       LaplacianType::DA, false).unwrap();
/// assert_eq!(r.eigenvalues.len(), 2);
/// assert_eq!(r.embedding.len(), 4);
/// assert_eq!(r.embedding[0].len(), 2);
/// ```
pub fn laplacian_spectral_embedding(
    graph: &Graph,
    no: usize,
    weights: Option<&[f64]>,
    which: SpectralWhich,
    lap_type: LaplacianType,
    scaled: bool,
) -> IgraphResult<LaplacianSpectralEmbeddingResult> {
    let n = graph.vcount() as usize;

    if no == 0 {
        return Err(IgraphError::InvalidArgument(
            "no must be at least 1".to_string(),
        ));
    }
    if no > n {
        return Err(IgraphError::InvalidArgument(format!(
            "no ({no}) exceeds vertex count ({n})"
        )));
    }

    if let Some(w) = weights {
        if w.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(format!(
                "weights length ({}) differs from edge count ({})",
                w.len(),
                graph.ecount()
            )));
        }
        for &wv in w {
            if !wv.is_finite() {
                return Err(IgraphError::InvalidArgument(
                    "edge weights must be finite".to_string(),
                ));
            }
            if wv < 0.0 {
                return Err(IgraphError::InvalidArgument(
                    "edge weights must be non-negative for Laplacian embedding".to_string(),
                ));
            }
        }
    }

    if n == 0 {
        return Ok(LaplacianSpectralEmbeddingResult {
            eigenvalues: Vec::new(),
            embedding: Vec::new(),
        });
    }

    if graph.ecount() == 0 {
        return Ok(LaplacianSpectralEmbeddingResult {
            eigenvalues: vec![0.0; no],
            embedding: vec![vec![0.0; no]; n],
        });
    }

    let adj = build_adjacency(graph)?;
    let deg = compute_degree(&adj, weights, n);

    let eigen_which = match which {
        SpectralWhich::LargestAlgebraic => EigenWhich::LargestAlgebraic,
        SpectralWhich::SmallestAlgebraic => EigenWhich::SmallestAlgebraic,
        SpectralWhich::LargestMagnitude => EigenWhich::LargestMagnitude,
    };
    let max_iter = n.saturating_mul(50).max(1000);

    let result = match lap_type {
        LaplacianType::DA => {
            let mv = |x: &[f64], y: &mut [f64]| matvec_da(&adj, weights, &deg, n, x, y);
            lanczos_top_k(n, &mv, no, eigen_which, max_iter)
        }
        LaplacianType::DAD => {
            let isd = inv_sqrt_degree(&deg);
            let mv = |x: &[f64], y: &mut [f64]| matvec_dad(&adj, weights, &isd, n, x, y);
            lanczos_top_k(n, &mv, no, eigen_which, max_iter)
        }
        LaplacianType::IDAD => {
            let isd = inv_sqrt_degree(&deg);
            let mv = |x: &[f64], y: &mut [f64]| matvec_idad(&adj, weights, &isd, n, x, y);
            lanczos_top_k(n, &mv, no, eigen_which, max_iter)
        }
    };

    Ok(build_result(result, n, scaled))
}

// ─── Internal helpers ────────────────────────────────────────────

use crate::algorithms::community::lanczos::LanczosTopKResult;

fn build_result(
    result: LanczosTopKResult,
    n: usize,
    scaled: bool,
) -> LaplacianSpectralEmbeddingResult {
    let actual_no = result.eigenvalues.len();

    let mut eigenvalues = result.eigenvalues;
    zapsmall_vec(&mut eigenvalues);

    let mut embedding = vec![vec![0.0; actual_no]; n];
    for (d, evec) in result.eigenvectors.iter().enumerate().take(actual_no) {
        for (v, row) in embedding.iter_mut().enumerate() {
            row[d] = evec[v];
        }
    }

    for row in &mut embedding {
        zapsmall_vec(row);
    }

    if scaled {
        for (d, &eval) in eigenvalues.iter().enumerate().take(actual_no) {
            let scale = eval.abs().sqrt();
            for row in &mut embedding {
                row[d] *= scale;
            }
        }
    }

    LaplacianSpectralEmbeddingResult {
        eigenvalues,
        embedding,
    }
}

type AdjList = Vec<Vec<(usize, usize)>>; // adj[v] = [(neighbor, edge_id), ...]

fn build_adjacency(graph: &Graph) -> IgraphResult<AdjList> {
    let n = graph.vcount() as usize;
    let mut adj: AdjList = vec![Vec::new(); n];
    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as u32;
        let s = graph.edge_source(eid32)? as usize;
        let t = graph.edge_target(eid32)? as usize;
        adj[s].push((t, eid));
        if s != t {
            adj[t].push((s, eid));
        }
    }
    Ok(adj)
}

fn compute_degree(adj: &AdjList, weights: Option<&[f64]>, n: usize) -> Vec<f64> {
    let mut deg = vec![0.0; n];
    for (i, neighbors) in adj.iter().enumerate().take(n) {
        for &(_nei, eid) in neighbors {
            let w = match weights {
                Some(wt) => wt[eid],
                None => 1.0,
            };
            deg[i] += w;
        }
    }
    deg
}

fn inv_sqrt_degree(deg: &[f64]) -> Vec<f64> {
    deg.iter()
        .map(|&d| if d > 0.0 { 1.0 / d.sqrt() } else { 0.0 })
        .collect()
}

/// D − A: y = D·x − A·x
fn matvec_da(
    adj: &AdjList,
    weights: Option<&[f64]>,
    deg: &[f64],
    n: usize,
    x: &[f64],
    y: &mut [f64],
) {
    for i in 0..n {
        y[i] = deg[i] * x[i];
        for &(nei, eid) in &adj[i] {
            let w = match weights {
                Some(wt) => wt[eid],
                None => 1.0,
            };
            y[i] -= w * x[nei];
        }
    }
}

/// D⁻¹ᐟ² A D⁻¹ᐟ²: y = D^{-1/2} (A (D^{-1/2} x))
fn matvec_dad(
    adj: &AdjList,
    weights: Option<&[f64]>,
    inv_sqrt_deg: &[f64],
    n: usize,
    x: &[f64],
    y: &mut [f64],
) {
    // Step 1: z = D^{-1/2} x
    // Step 2: t = A z
    // Step 3: y = D^{-1/2} t
    for i in 0..n {
        let mut t_i = 0.0;
        for &(nei, eid) in &adj[i] {
            let w = match weights {
                Some(wt) => wt[eid],
                None => 1.0,
            };
            t_i += w * inv_sqrt_deg[nei] * x[nei];
        }
        y[i] = inv_sqrt_deg[i] * t_i;
    }
}

/// I − D⁻¹ᐟ² A D⁻¹ᐟ²: y = x − D^{-1/2} A D^{-1/2} x
fn matvec_idad(
    adj: &AdjList,
    weights: Option<&[f64]>,
    inv_sqrt_deg: &[f64],
    n: usize,
    x: &[f64],
    y: &mut [f64],
) {
    matvec_dad(adj, weights, inv_sqrt_deg, n, x, y);
    for i in 0..n {
        y[i] = x[i] - y[i];
    }
}

fn zapsmall_vec(v: &mut [f64]) {
    if v.is_empty() {
        return;
    }
    let max_abs = v.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    if max_abs == 0.0 {
        return;
    }
    let threshold = max_abs * f64::EPSILON.sqrt();
    for x in v {
        if x.abs() < threshold {
            *x = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cycle(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).unwrap();
        }
        g
    }

    fn make_complete(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n {
            for j in (i + 1)..n {
                g.add_edge(i, j).unwrap();
            }
        }
        g
    }

    #[test]
    fn empty_graph_err() {
        let g = Graph::with_vertices(0);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        );
        assert!(r.is_err());
    }

    #[test]
    fn no_edges_all_zero() {
        let g = Graph::with_vertices(4);
        let r = laplacian_spectral_embedding(
            &g,
            2,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert_eq!(r.eigenvalues.len(), 2);
        for &e in &r.eigenvalues {
            assert!(e.abs() < 1e-10);
        }
    }

    #[test]
    fn da_complete_k4() {
        // K4: L = D - A. D = 3I. Eigenvalues of L: 0, 4, 4, 4.
        let g = make_complete(4);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - 4.0).abs() < 1e-4,
            "expected 4.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn da_smallest_is_zero() {
        // The smallest eigenvalue of the combinatorial Laplacian of a
        // connected graph is always 0.
        let g = make_complete(4);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::SmallestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert!(
            r.eigenvalues[0].abs() < 1e-4,
            "expected ~0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn da_path_p4() {
        // Path P4: L eigenvalues are 0, 2-√2, 2, 2+√2.
        // Largest = 2 + √2 ≈ 3.414
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        let expected = 2.0 + 2.0_f64.sqrt();
        assert!(
            (r.eigenvalues[0] - expected).abs() < 1e-3,
            "expected {expected}, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn dad_complete_k4() {
        // K4: D = 3I, A has eigenvalues 3,-1,-1,-1.
        // D^{-1/2} A D^{-1/2} = (1/3) A, eigenvalues 1, -1/3, -1/3, -1/3.
        let g = make_complete(4);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DAD,
            false,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - 1.0).abs() < 1e-4,
            "expected 1.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn idad_complete_k4() {
        // I - DAD eigenvalues = 1 - DAD eigenvalues = 0, 4/3, 4/3, 4/3.
        // Largest = 4/3.
        let g = make_complete(4);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::IDAD,
            false,
        )
        .unwrap();
        let expected = 4.0 / 3.0;
        assert!(
            (r.eigenvalues[0] - expected).abs() < 1e-4,
            "expected {expected}, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn idad_smallest_is_zero() {
        // Smallest eigenvalue of L_sym = I - DAD for connected graph is 0.
        let g = make_cycle(6);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::SmallestAlgebraic,
            LaplacianType::IDAD,
            false,
        )
        .unwrap();
        assert!(
            r.eigenvalues[0].abs() < 1e-4,
            "expected ~0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn da_cycle_c6() {
        // C6: L eigenvalues are 2 - 2cos(2πk/6) for k=0..5.
        // = 0, 1, 3, 4, 3, 1. Largest = 4.
        let g = make_cycle(6);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - 4.0).abs() < 1e-3,
            "expected 4.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn scaled_embedding() {
        let g = make_complete(4);
        let r_plain = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        let r_scaled = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            true,
        )
        .unwrap();

        let scale = r_plain.eigenvalues[0].abs().sqrt();
        for v in 0..4 {
            let expected = r_plain.embedding[v][0] * scale;
            assert!(
                (r_scaled.embedding[v][0] - expected).abs() < 1e-8,
                "vertex {v}: expected {expected}, got {}",
                r_scaled.embedding[v][0]
            );
        }
    }

    #[test]
    fn weighted_da() {
        // Star S3 with weights 2: center=0, leaves=1,2,3.
        // D = diag(6,2,2,2), A has edges with weight 2.
        // L = D - A. Largest eigenvalue of L(star) with weight w:
        // eigenvalues are 0, w*sqrt(3), ... For unweighted star:
        // L eigenvalues are 0,1,1,n. For weighted star S3 with w=2:
        // D=diag(6,2,2,2), eigenvalues of L: 0, 2, 2, 8.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let weights = vec![2.0, 2.0, 2.0];
        let r = laplacian_spectral_embedding(
            &g,
            1,
            Some(&weights),
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - 8.0).abs() < 1e-3,
            "expected 8.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn embedding_dimension() {
        let g = make_cycle(6);
        let r = laplacian_spectral_embedding(
            &g,
            3,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        )
        .unwrap();
        assert_eq!(r.eigenvalues.len(), 3);
        assert_eq!(r.embedding.len(), 6);
        for row in &r.embedding {
            assert_eq!(row.len(), 3);
        }
    }

    #[test]
    fn invalid_no() {
        let g = Graph::with_vertices(3);
        let r = laplacian_spectral_embedding(
            &g,
            5,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        );
        assert!(r.is_err());
    }

    #[test]
    fn negative_weight_rejected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = laplacian_spectral_embedding(
            &g,
            1,
            Some(&[1.0, -1.0]),
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DA,
            false,
        );
        assert!(r.is_err());
    }

    #[test]
    fn dad_regular_graph() {
        // For a d-regular graph, D = dI, so D^{-1/2} A D^{-1/2} = A/d.
        // C4 is 2-regular. A eigenvalues: 2, 0, -2, 0.
        // DAD eigenvalues: 1, 0, -1, 0. Largest = 1.
        let g = make_cycle(4);
        let r = laplacian_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            LaplacianType::DAD,
            false,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - 1.0).abs() < 1e-4,
            "expected 1.0, got {}",
            r.eigenvalues[0]
        );
    }
}
