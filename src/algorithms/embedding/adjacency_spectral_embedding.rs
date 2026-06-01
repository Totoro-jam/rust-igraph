//! Adjacency spectral embedding (ALGO-EM-002).
//!
//! Computes an `no`-dimensional Euclidean representation of the graph
//! based on the spectral decomposition of its adjacency matrix.
//!
//! For undirected graphs, decomposes A = U D U^T (eigendecomposition)
//! and returns X = U^no (the first `no` eigenvectors). If `scaled`,
//! multiplies each column by `sqrt(|λ_i|)`.
//!
//! Counterpart of `igraph_adjacency_spectral_embedding()` from
//! `references/igraph/src/misc/embedding.c:872`.

use crate::algorithms::community::lanczos::{EigenWhich, lanczos_top_k};
use crate::core::{Graph, IgraphError, IgraphResult};

/// Which eigenvalues to select for spectral embedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralWhich {
    /// Largest algebraic eigenvalues.
    LargestAlgebraic,
    /// Smallest algebraic eigenvalues.
    SmallestAlgebraic,
    /// Largest magnitude eigenvalues.
    LargestMagnitude,
}

/// Result of adjacency spectral embedding.
#[derive(Debug, Clone)]
pub struct AdjacencySpectralEmbeddingResult {
    /// Eigenvalues (or singular values) in the order selected by `which`.
    pub eigenvalues: Vec<f64>,
    /// Embedding matrix: `embedding[v]` is the `no`-dimensional embedding
    /// of vertex `v`. Each inner vector has length `no`.
    pub embedding: Vec<Vec<f64>>,
}

/// Compute the adjacency spectral embedding of an undirected graph.
///
/// Decomposes the (optionally augmented) adjacency matrix
/// A + diag(cvec) into its top eigenpairs and returns the
/// eigenvectors as vertex embeddings.
///
/// `no`: embedding dimension (number of eigenpairs to compute).
///
/// `weights`: optional edge weights. Must have length `ecount` if
/// provided.
///
/// `which`: which eigenvalues to use. [`SpectralWhich::LargestAlgebraic`]
/// is the default in igraph.
///
/// `scaled`: if true, multiply each eigenvector column by
/// sqrt(|eigenvalue|), giving X = U D^(1/2).
///
/// `cvec`: optional diagonal augmentation vector. If `Some(&[c])` (a
/// single element), `c` is added to every diagonal entry. If
/// `Some(&[c0, c1, ..., cn-1])` (length = vcount), vertex `i` gets
/// `c_i` added to its diagonal. Default is zero augmentation.
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, adjacency_spectral_embedding, SpectralWhich};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 3).unwrap();
///
/// let r = adjacency_spectral_embedding(&g, 2, None, SpectralWhich::LargestAlgebraic,
///                                       false, None).unwrap();
/// assert_eq!(r.eigenvalues.len(), 2);
/// assert_eq!(r.embedding.len(), 4);
/// assert_eq!(r.embedding[0].len(), 2);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn adjacency_spectral_embedding(
    graph: &Graph,
    no: usize,
    weights: Option<&[f64]>,
    which: SpectralWhich,
    scaled: bool,
    cvec: Option<&[f64]>,
) -> IgraphResult<AdjacencySpectralEmbeddingResult> {
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
        }
    }

    if let Some(cv) = cvec {
        if cv.len() != 1 && cv.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "cvec length ({}) must be 1 or vcount ({n})",
                cv.len()
            )));
        }
    }

    if n == 0 {
        return Ok(AdjacencySpectralEmbeddingResult {
            eigenvalues: Vec::new(),
            embedding: Vec::new(),
        });
    }

    if graph.ecount() == 0 {
        return Ok(AdjacencySpectralEmbeddingResult {
            eigenvalues: vec![0.0; no],
            embedding: vec![vec![0.0; no]; n],
        });
    }

    // Build adjacency list (undirected)
    let adj = build_adjacency(graph);

    // Matrix-vector product: y = (A + diag(cvec)) x
    let matvec = |x: &[f64], y: &mut [f64]| {
        adjacency_matvec(&adj, weights, cvec, n, x, y);
    };

    let eigen_which = match which {
        SpectralWhich::LargestAlgebraic => EigenWhich::LargestAlgebraic,
        SpectralWhich::SmallestAlgebraic => EigenWhich::SmallestAlgebraic,
        SpectralWhich::LargestMagnitude => EigenWhich::LargestMagnitude,
    };

    let max_iter = n.saturating_mul(50).max(1000);
    let result = lanczos_top_k(n, &matvec, no, eigen_which, max_iter);

    let actual_no = result.eigenvalues.len();

    // Zapsmall eigenvalues
    let mut eigenvalues = result.eigenvalues;
    zapsmall_vec(&mut eigenvalues);

    // Build embedding: embedding[v][d] = eigenvector_d[v]
    let mut embedding = vec![vec![0.0; actual_no]; n];
    for (d, evec) in result.eigenvectors.iter().enumerate().take(actual_no) {
        for (v, row) in embedding.iter_mut().enumerate() {
            row[d] = evec[v];
        }
    }

    // Zapsmall the embedding
    for row in &mut embedding {
        zapsmall_vec(row);
    }

    // Scale: X = U D^{1/2} (multiply column d by sqrt(|λ_d|))
    if scaled {
        for (d, &eval) in eigenvalues.iter().enumerate().take(actual_no) {
            let scale = eval.abs().sqrt();
            for row in &mut embedding {
                row[d] *= scale;
            }
        }
    }

    Ok(AdjacencySpectralEmbeddingResult {
        eigenvalues,
        embedding,
    })
}

// ─── Internal helpers ────────────────────────────────────────────

type AdjList = Vec<Vec<(usize, usize)>>; // adj[v] = [(neighbor, edge_id), ...]

fn build_adjacency(graph: &Graph) -> AdjList {
    let n = graph.vcount() as usize;
    let mut adj: AdjList = vec![Vec::new(); n];
    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as u32;
        let s = graph.edge_source(eid32).unwrap() as usize;
        let t = graph.edge_target(eid32).unwrap() as usize;
        adj[s].push((t, eid));
        if s != t {
            adj[t].push((s, eid));
        }
    }
    adj
}

fn adjacency_matvec(
    adj: &AdjList,
    weights: Option<&[f64]>,
    cvec: Option<&[f64]>,
    n: usize,
    x: &[f64],
    y: &mut [f64],
) {
    for i in 0..n {
        y[i] = 0.0;
        for &(nei, eid) in &adj[i] {
            let w = match weights {
                Some(wt) => wt[eid],
                None => 1.0,
            };
            y[i] += w * x[nei];
        }
        // Diagonal augmentation
        if let Some(cv) = cvec {
            let c = if cv.len() == 1 { cv[0] } else { cv[i] };
            y[i] += c * x[i];
        }
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

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r =
            adjacency_spectral_embedding(&g, 1, None, SpectralWhich::LargestAlgebraic, false, None);
        assert!(r.is_err()); // no > n
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(4);
        let r =
            adjacency_spectral_embedding(&g, 2, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        assert_eq!(r.eigenvalues.len(), 2);
        assert_eq!(r.embedding.len(), 4);
        for &e in &r.eigenvalues {
            assert!(
                (e).abs() < 1e-10,
                "eigenvalue should be 0 for edgeless graph"
            );
        }
    }

    #[test]
    fn path_graph_2d() {
        // Path P4: adjacency eigenvalues are ±(1+√5)/2 and ±(√5-1)/2
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let r =
            adjacency_spectral_embedding(&g, 2, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        assert_eq!(r.eigenvalues.len(), 2);
        // Eigenvalues should be positive (largest algebraic of a path)
        assert!(r.eigenvalues[0] > 0.0, "got {}", r.eigenvalues[0]);
        assert!(r.eigenvalues[1] > 0.0, "got {}", r.eigenvalues[1]);
        // First eigenvalue should be the golden ratio (1+√5)/2 ≈ 1.618
        #[allow(unknown_lints, clippy::manual_midpoint)]
        let golden = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!(
            (r.eigenvalues[0] - golden).abs() < 1e-4,
            "expected {golden}, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn complete_graph_k4() {
        // K4: adjacency eigenvalues are 3 (×1) and -1 (×3)
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let r =
            adjacency_spectral_embedding(&g, 1, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        assert!(
            (r.eigenvalues[0] - 3.0).abs() < 1e-4,
            "expected 3.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn scaled_embedding() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let r_unscaled =
            adjacency_spectral_embedding(&g, 1, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        let r_scaled =
            adjacency_spectral_embedding(&g, 1, None, SpectralWhich::LargestAlgebraic, true, None)
                .unwrap();

        let scale = r_unscaled.eigenvalues[0].abs().sqrt();
        for v in 0..4 {
            let expected = r_unscaled.embedding[v][0] * scale;
            assert!(
                (r_scaled.embedding[v][0] - expected).abs() < 1e-8,
                "vertex {v}: expected {expected}, got {}",
                r_scaled.embedding[v][0]
            );
        }
    }

    #[test]
    fn cvec_scalar_augmentation() {
        // Adding c to diagonal shifts all eigenvalues by c
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let r_plain =
            adjacency_spectral_embedding(&g, 1, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        let r_shifted = adjacency_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::LargestAlgebraic,
            false,
            Some(&[2.0]),
        )
        .unwrap();
        // λ_shifted = λ_plain + 2.0
        assert!(
            (r_shifted.eigenvalues[0] - r_plain.eigenvalues[0] - 2.0).abs() < 1e-4,
            "expected shift of 2.0, got {} vs {}",
            r_shifted.eigenvalues[0],
            r_plain.eigenvalues[0]
        );
    }

    #[test]
    fn smallest_algebraic() {
        // K4: eigenvalues 3, -1, -1, -1. Smallest algebraic = -1
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let r = adjacency_spectral_embedding(
            &g,
            1,
            None,
            SpectralWhich::SmallestAlgebraic,
            false,
            None,
        )
        .unwrap();
        assert!(
            (r.eigenvalues[0] - (-1.0)).abs() < 1e-4,
            "expected -1.0, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn weighted_graph() {
        // Star graph S3 with weights: center=0, leaves=1,2,3
        // A = [[0,w,w,w],[w,0,0,0],[w,0,0,0],[w,0,0,0]]
        // Eigenvalues: ±w√3, 0, 0
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let weights = vec![2.0, 2.0, 2.0];
        let r = adjacency_spectral_embedding(
            &g,
            1,
            Some(&weights),
            SpectralWhich::LargestAlgebraic,
            false,
            None,
        )
        .unwrap();
        let expected = 2.0 * 3.0_f64.sqrt();
        assert!(
            (r.eigenvalues[0] - expected).abs() < 1e-4,
            "expected {expected}, got {}",
            r.eigenvalues[0]
        );
    }

    #[test]
    fn invalid_no() {
        let g = Graph::with_vertices(3);
        let r =
            adjacency_spectral_embedding(&g, 5, None, SpectralWhich::LargestAlgebraic, false, None);
        assert!(r.is_err());
    }

    #[test]
    fn embedding_dimension() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        let r =
            adjacency_spectral_embedding(&g, 3, None, SpectralWhich::LargestAlgebraic, false, None)
                .unwrap();
        assert_eq!(r.eigenvalues.len(), 3);
        assert_eq!(r.embedding.len(), 6);
        for row in &r.embedding {
            assert_eq!(row.len(), 3);
        }
    }
}
