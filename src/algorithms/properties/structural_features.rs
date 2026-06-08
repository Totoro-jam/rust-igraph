//! Per-vertex structural feature vectors (ALGO-TR-017).
//!
//! Assembles multi-dimensional structural descriptors for each vertex,
//! combining degree statistics, local clustering, and neighborhood
//! topology into feature matrices suitable for GNN input or classical
//! ML on graphs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::needless_range_loop
)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Per-vertex structural feature vector.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuralFeatures {
    /// Number of vertices.
    pub num_vertices: usize,
    /// Number of features per vertex.
    pub num_features: usize,
    /// Feature matrix in row-major order: `features[v * num_features + f]`.
    pub features: Vec<f64>,
    /// Feature names in column order.
    pub feature_names: Vec<&'static str>,
}

impl StructuralFeatures {
    /// Get feature vector for vertex `v`.
    pub fn vertex_features(&self, v: usize) -> &[f64] {
        let start = v * self.num_features;
        &self.features[start..start + self.num_features]
    }
}

/// Compute structural feature vectors for all vertices.
///
/// Each vertex gets a feature vector containing:
/// 0. `degree` — vertex degree
/// 1. `log_degree` — log2(1 + degree)
/// 2. `clustering` — local clustering coefficient
/// 3. `avg_neighbor_degree` — mean degree of neighbors
/// 4. `min_neighbor_degree` — minimum degree among neighbors
/// 5. `max_neighbor_degree` — maximum degree among neighbors
/// 6. `triangles` — number of triangles containing this vertex
/// 7. `square_clustering` — fraction of possible squares closed
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, structural_feature_vectors};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2),(2,3)], false, Some(4)).unwrap();
/// let sf = structural_feature_vectors(&g).unwrap();
/// assert_eq!(sf.num_vertices, 4);
/// assert_eq!(sf.num_features, 8);
/// // Vertex 0 has degree 2
/// assert!((sf.vertex_features(0)[0] - 2.0).abs() < 1e-10);
/// ```
pub fn structural_feature_vectors(graph: &Graph) -> IgraphResult<StructuralFeatures> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "structural_feature_vectors is defined for undirected graphs only".to_string(),
        ));
    }

    let nv = graph.vcount() as usize;
    let num_features = 8;

    // Pre-compute degrees and neighbor lists
    let mut degrees = Vec::with_capacity(nv);
    let mut neighbors_cache: Vec<Vec<VertexId>> = Vec::with_capacity(nv);
    for v in 0..nv {
        let vid = v as VertexId;
        degrees.push(graph.degree(vid)?);
        neighbors_cache.push(graph.neighbors(vid)?);
    }

    let mut features = vec![0.0_f64; nv * num_features];

    for v in 0..nv {
        let offset = v * num_features;
        let deg = degrees[v];
        let neighbors = &neighbors_cache[v];

        // Feature 0: degree
        features[offset] = deg as f64;

        // Feature 1: log_degree
        features[offset + 1] = (1.0 + deg as f64).log2();

        // Feature 2: local clustering coefficient
        features[offset + 2] = local_clustering(&neighbors_cache, v);

        // Feature 3-5: neighbor degree statistics
        if deg > 0 {
            let mut sum_nd = 0usize;
            let mut min_nd = usize::MAX;
            let mut max_nd = 0usize;
            for &u in neighbors {
                let nd = degrees[u as usize];
                sum_nd += nd;
                if nd < min_nd {
                    min_nd = nd;
                }
                if nd > max_nd {
                    max_nd = nd;
                }
            }
            features[offset + 3] = sum_nd as f64 / deg as f64;
            features[offset + 4] = min_nd as f64;
            features[offset + 5] = max_nd as f64;
        }

        // Feature 6: triangles
        let tri = count_vertex_triangles(&neighbors_cache, v);
        features[offset + 6] = tri as f64;

        // Feature 7: square clustering (fraction of possible squares closed)
        features[offset + 7] = square_clustering_coeff(&neighbors_cache, &degrees, v);
    }

    Ok(StructuralFeatures {
        num_vertices: nv,
        num_features,
        features,
        feature_names: vec![
            "degree",
            "log_degree",
            "clustering",
            "avg_neighbor_degree",
            "min_neighbor_degree",
            "max_neighbor_degree",
            "triangles",
            "square_clustering",
        ],
    })
}

/// Compute degree profile for each vertex: [deg, deg², log(deg+1)].
///
/// A lightweight alternative to the full feature vector when only
/// degree-based features are needed.
///
/// Returns a flat `Vec<f64>` of length `3 * vcount`, row-major.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_profile};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let prof = degree_profile(&g).unwrap();
/// // Vertex 0: deg=2, deg²=4, log(3)
/// assert!((prof[0] - 2.0).abs() < 1e-10);
/// assert!((prof[1] - 4.0).abs() < 1e-10);
/// ```
pub fn degree_profile(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let nv = graph.vcount() as usize;
    let mut result = Vec::with_capacity(nv * 3);

    for v in 0..nv {
        let d = graph.degree(v as VertexId)? as f64;
        result.push(d);
        result.push(d * d);
        result.push((d + 1.0).log2());
    }

    Ok(result)
}

// --- Internal helpers ---

fn local_clustering(neighbors_cache: &[Vec<VertexId>], v: usize) -> f64 {
    let neighbors = &neighbors_cache[v];
    let deg = neighbors.len();
    if deg < 2 {
        return 0.0;
    }

    let mut triangles = 0usize;
    for i in 0..deg {
        let ni = neighbors[i] as usize;
        for j in (i + 1)..deg {
            let nj = neighbors[j];
            if neighbors_cache[ni].contains(&nj) {
                triangles += 1;
            }
        }
    }

    let possible = deg * (deg - 1) / 2;
    triangles as f64 / possible as f64
}

fn count_vertex_triangles(neighbors_cache: &[Vec<VertexId>], v: usize) -> usize {
    let neighbors = &neighbors_cache[v];
    let deg = neighbors.len();
    let mut count = 0;

    for i in 0..deg {
        let ni = neighbors[i] as usize;
        for j in (i + 1)..deg {
            let nj = neighbors[j];
            if neighbors_cache[ni].contains(&nj) {
                count += 1;
            }
        }
    }

    count
}

fn square_clustering_coeff(neighbors_cache: &[Vec<VertexId>], degrees: &[usize], v: usize) -> f64 {
    // Square clustering: for vertex v, count pairs of neighbors (u, w)
    // that have a common neighbor other than v (forming a 4-cycle through v).
    let neighbors = &neighbors_cache[v];
    let deg = neighbors.len();
    if deg < 2 {
        return 0.0;
    }

    let mut squares = 0usize;
    for i in 0..deg {
        let u = neighbors[i] as usize;
        for j in (i + 1)..deg {
            let w = neighbors[j];
            // Count common neighbors of u and w, excluding v
            for &x in &neighbors_cache[u] {
                if x != v as u32 && x != w && neighbors_cache[w as usize].contains(&x) {
                    squares += 1;
                    break; // Only count once per pair
                }
            }
        }
    }

    // Maximum possible squares: C(deg, 2) pairs, each could form a square
    let possible = deg * (deg - 1) / 2;
    if possible == 0 {
        return 0.0;
    }

    let _ = degrees;
    squares as f64 / possible as f64
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

    fn square_graph() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    // --- structural_feature_vectors tests ---

    #[test]
    fn sf_triangle_features() {
        let g = triangle();
        let sf = structural_feature_vectors(&g).unwrap();
        assert_eq!(sf.num_vertices, 3);
        assert_eq!(sf.num_features, 8);
        // All vertices in K3 have degree 2
        for v in 0..3 {
            let f = sf.vertex_features(v);
            assert!((f[0] - 2.0).abs() < 1e-10); // degree
            assert!((f[2] - 1.0).abs() < 1e-10); // clustering = 1 in K3
            assert!((f[3] - 2.0).abs() < 1e-10); // avg_neighbor_deg = 2
            assert!((f[6] - 1.0).abs() < 1e-10); // 1 triangle per vertex
        }
    }

    #[test]
    fn sf_path_features() {
        let g = path4();
        let sf = structural_feature_vectors(&g).unwrap();
        // Vertex 0: deg=1, clustering=0, triangles=0
        let f0 = sf.vertex_features(0);
        assert!((f0[0] - 1.0).abs() < 1e-10);
        assert!(f0[2].abs() < 1e-10);
        assert!(f0[6].abs() < 1e-10);

        // Vertex 1: deg=2, clustering=0
        let f1 = sf.vertex_features(1);
        assert!((f1[0] - 2.0).abs() < 1e-10);
        assert!(f1[2].abs() < 1e-10);
    }

    #[test]
    fn sf_empty_graph() {
        let g = Graph::with_vertices(3);
        let sf = structural_feature_vectors(&g).unwrap();
        for v in 0..3 {
            let f = sf.vertex_features(v);
            assert!(f[0].abs() < 1e-10); // degree 0
        }
    }

    #[test]
    fn sf_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(structural_feature_vectors(&g).is_err());
    }

    #[test]
    fn sf_feature_names() {
        let g = triangle();
        let sf = structural_feature_vectors(&g).unwrap();
        assert_eq!(sf.feature_names.len(), 8);
        assert_eq!(sf.feature_names[0], "degree");
        assert_eq!(sf.feature_names[7], "square_clustering");
    }

    #[test]
    fn sf_square_clustering() {
        let g = square_graph();
        let sf = structural_feature_vectors(&g).unwrap();
        // In C4, each vertex has 2 neighbors forming 1 square
        // square_clustering for each vertex: pair (u,w) has common neighbor → 1.0
        for v in 0..4 {
            let f = sf.vertex_features(v);
            assert!((f[7] - 1.0).abs() < 1e-10);
        }
    }

    // --- degree_profile tests ---

    #[test]
    fn dp_triangle() {
        let g = triangle();
        let prof = degree_profile(&g).unwrap();
        assert_eq!(prof.len(), 9); // 3 vertices × 3 features
        // Vertex 0: deg=2, deg²=4, log2(3)
        assert!((prof[0] - 2.0).abs() < 1e-10);
        assert!((prof[1] - 4.0).abs() < 1e-10);
        assert!((prof[2] - 3.0_f64.log2()).abs() < 1e-10);
    }

    #[test]
    fn dp_empty() {
        let g = Graph::with_vertices(2);
        let prof = degree_profile(&g).unwrap();
        assert_eq!(prof.len(), 6);
        // All zeros for degree, degree²
        assert!(prof[0].abs() < 1e-10);
        assert!(prof[1].abs() < 1e-10);
        // log2(0+1) = 0
        assert!(prof[2].abs() < 1e-10);
    }

    #[test]
    fn dp_star() {
        let g = Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap();
        let prof = degree_profile(&g).unwrap();
        // Vertex 0: deg=3, deg²=9
        assert!((prof[0] - 3.0).abs() < 1e-10);
        assert!((prof[1] - 9.0).abs() < 1e-10);
    }
}
