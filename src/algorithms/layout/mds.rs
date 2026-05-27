//! Multidimensional Scaling layout (ALGO-LO-010).
//!
//! Classical MDS (Torgerson 1952): place vertices so that pairwise
//! Euclidean distances approximate given dissimilarities. Uses
//! double centering + eigendecomposition of the Gram matrix.
//!
//! Reference: Cox & Cox, "Multidimensional Scaling" (1994), Chapman & Hall.

use crate::algorithms::paths::floyd_warshall::floyd_warshall_distances;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the MDS (Multidimensional Scaling) layout.
///
/// Places vertices so that Euclidean distances in 2D approximate the
/// graph-theoretic distances. Uses classical (metric) MDS via double
/// centering of the squared distance matrix and eigendecomposition.
///
/// For disconnected graphs, each component is laid out independently
/// and the resulting layouts are placed side by side.
///
/// # Arguments
///
/// * `graph` — input graph (edge directions ignored for distances).
/// * `dist` — optional distance matrix (row-major, n×n). If `None`,
///   shortest-path distances are computed automatically.
///
/// Returns `[x, y]` positions for each vertex.
///
/// # Errors
///
/// Returns `InvalidArgument` if the distance matrix dimensions don't
/// match the vertex count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_mds};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
///
/// let pos = layout_mds(&g, None).unwrap();
/// assert_eq!(pos.len(), 5);
/// assert!(pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_mds(graph: &Graph, dist: Option<&[Vec<f64>]>) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![[0.0, 0.0]]);
    }

    if let Some(d) = dist {
        if d.len() != n {
            return Err(IgraphError::InvalidArgument(format!(
                "distance matrix rows ({}) must equal vertex count ({})",
                d.len(),
                n
            )));
        }
        for row in d {
            if row.len() != n {
                return Err(IgraphError::InvalidArgument(format!(
                    "distance matrix columns ({}) must equal vertex count ({})",
                    row.len(),
                    n
                )));
            }
        }
    }

    // Get distance matrix
    let dist_matrix = if let Some(d) = dist {
        d.to_vec()
    } else {
        // Use Floyd-Warshall for all-pairs unweighted shortest paths
        let fw = floyd_warshall_distances(graph, None)?;
        fw.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|v| v.unwrap_or(f64::INFINITY))
                    .collect()
            })
            .collect()
    };

    // Check for disconnected graph (infinite distances)
    let connected = dist_matrix
        .iter()
        .all(|row| row.iter().all(|&v| v.is_finite()));

    if connected {
        mds_single(n, &dist_matrix)
    } else {
        mds_disconnected(n, &dist_matrix)
    }
}

fn mds_single(n: usize, dist: &[Vec<f64>]) -> IgraphResult<Vec<[f64; 2]>> {
    if n == 1 {
        return Ok(vec![[0.0, 0.0]]);
    }
    if n == 2 {
        return Ok(vec![[0.0, 0.0], [dist[0][1].max(1.0), 0.0]]);
    }

    // Square the distances
    let mut b: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            b[i][j] = dist[i][j] * dist[i][j];
        }
    }

    // Double centering: B = -0.5 * (D² - row_mean - col_mean + grand_mean)
    let mut row_means = vec![0.0_f64; n];
    for i in 0..n {
        let sum: f64 = b[i].iter().sum();
        row_means[i] = sum / n as f64;
    }

    let grand_mean: f64 = row_means.iter().sum::<f64>() / n as f64;

    for i in 0..n {
        for j in 0..n {
            b[i][j] = -0.5 * (b[i][j] - row_means[i] - row_means[j] + grand_mean);
        }
    }

    // Extract top 2 eigenvectors via power iteration
    let (eigenvalues, eigenvectors) = top_k_eigen_symmetric(&b, 2);

    // Scale eigenvectors by sqrt(|eigenvalue|)
    let mut pos = vec![[0.0_f64; 2]; n];
    for dim in 0..2 {
        let scale = eigenvalues[dim].abs().sqrt();
        for i in 0..n {
            pos[i][dim] = scale * eigenvectors[dim][i];
        }
    }

    Ok(pos)
}

fn mds_disconnected(n: usize, dist: &[Vec<f64>]) -> IgraphResult<Vec<[f64; 2]>> {
    // Find connected components via BFS
    let mut component = vec![usize::MAX; n];
    let mut comp_id = 0;
    let mut components: Vec<Vec<usize>> = Vec::new();

    for start in 0..n {
        if component[start] != usize::MAX {
            continue;
        }
        let mut members = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        component[start] = comp_id;
        while let Some(v) = queue.pop_front() {
            members.push(v);
            for j in 0..n {
                if component[j] == usize::MAX && dist[v][j].is_finite() && dist[v][j] > 0.0 {
                    component[j] = comp_id;
                    queue.push_back(j);
                }
            }
        }
        components.push(members);
        comp_id += 1;
    }

    // Lay out each component separately
    let mut pos = vec![[0.0_f64; 2]; n];
    let mut x_offset = 0.0_f64;

    for comp in &components {
        let cn = comp.len();
        if cn == 1 {
            pos[comp[0]] = [x_offset, 0.0];
            x_offset += 2.0;
            continue;
        }

        // Extract sub-distance-matrix
        let mut sub_dist = vec![vec![0.0_f64; cn]; cn];
        for (li, &vi) in comp.iter().enumerate() {
            for (lj, &vj) in comp.iter().enumerate() {
                sub_dist[li][lj] = dist[vi][vj];
            }
        }

        let sub_pos = mds_single(cn, &sub_dist)?;

        // Find bounding box of sub_pos
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        for p in &sub_pos {
            if p[0] < min_x {
                min_x = p[0];
            }
            if p[0] > max_x {
                max_x = p[0];
            }
        }

        // Place component
        for (li, &vi) in comp.iter().enumerate() {
            pos[vi][0] = sub_pos[li][0] - min_x + x_offset;
            pos[vi][1] = sub_pos[li][1];
        }

        x_offset += (max_x - min_x) + 2.0;
    }

    Ok(pos)
}

/// Extract top-k eigenpairs of a symmetric matrix using power iteration
/// with deflation. Returns (eigenvalues, eigenvectors) sorted by
/// decreasing eigenvalue magnitude.
fn top_k_eigen_symmetric(matrix: &[Vec<f64>], k: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    let k = k.min(n);
    let max_iter = 300;
    let tol = 1e-10;

    let mut eigenvalues = Vec::with_capacity(k);
    let mut eigenvectors: Vec<Vec<f64>> = Vec::with_capacity(k);

    // Work on a mutable copy for deflation
    let mut work: Vec<Vec<f64>> = matrix.to_vec();

    let mut rng = SplitMix64::new(31415);

    for _ev in 0..k {
        // Random initial vector
        let mut v: Vec<f64> = (0..n).map(|_| rng.next_uniform() - 0.5).collect();
        normalize(&mut v);

        let mut eigenvalue = 0.0_f64;

        for _iter in 0..max_iter {
            // Matrix-vector multiply: w = A * v
            let mut w = vec![0.0_f64; n];
            for i in 0..n {
                let mut sum = 0.0_f64;
                for j in 0..n {
                    sum += work[i][j] * v[j];
                }
                w[i] = sum;
            }

            // Rayleigh quotient
            let new_eigenvalue: f64 = v.iter().zip(w.iter()).map(|(a, b)| a * b).sum();

            normalize(&mut w);

            // Check convergence
            let diff: f64 = v.iter().zip(w.iter()).map(|(a, b)| (a - b) * (a - b)).sum();
            v = w;
            eigenvalue = new_eigenvalue;

            if diff < tol {
                break;
            }
        }

        // Deflate: A = A - lambda * v * v^T
        for i in 0..n {
            for j in 0..n {
                work[i][j] -= eigenvalue * v[i] * v[j];
            }
        }

        eigenvalues.push(eigenvalue);
        eigenvectors.push(v);
    }

    (eigenvalues, eigenvectors)
}

fn normalize(v: &mut [f64]) {
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Internal RNG
// ═══════════════════════════════════════════════════════════════════

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mds_empty() {
        let g = Graph::with_vertices(0);
        let pos = layout_mds(&g, None).unwrap();
        assert!(pos.is_empty());
    }

    #[test]
    fn test_mds_single() {
        let g = Graph::with_vertices(1);
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 1);
        assert!(pos[0][0].abs() < 1e-10 && pos[0][1].abs() < 1e-10);
    }

    #[test]
    fn test_mds_two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 2);
        assert!(pos[0][0].is_finite() && pos[1][0].is_finite());
        // Two vertices should be separated
        let dx = pos[0][0] - pos[1][0];
        let dy = pos[0][1] - pos[1][1];
        assert!((dx * dx + dy * dy).sqrt() > 0.1);
    }

    #[test]
    fn test_mds_path() {
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 5);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_mds_cycle() {
        let mut g = Graph::with_vertices(6);
        for i in 0..6 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 6);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_mds_with_dist_matrix() {
        let g = Graph::with_vertices(3);
        let dist = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 1.0],
            vec![2.0, 1.0, 0.0],
        ];
        let pos = layout_mds(&g, Some(&dist)).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_mds_wrong_dist_size() {
        let g = Graph::with_vertices(3);
        let dist = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        assert!(layout_mds(&g, Some(&dist)).is_err());
    }

    #[test]
    fn test_mds_disconnected() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
        // Components should be separated
        let comp0_max_x = pos[0][0].max(pos[1][0]);
        let comp1_min_x = pos[2][0].min(pos[3][0]);
        assert!(comp1_min_x > comp0_max_x - 0.1 || comp0_max_x > comp1_min_x - 0.1);
    }

    #[test]
    fn test_mds_deterministic() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let pos1 = layout_mds(&g, None).unwrap();
        let pos2 = layout_mds(&g, None).unwrap();
        for i in 0..4 {
            assert!((pos1[i][0] - pos2[i][0]).abs() < 1e-8);
            assert!((pos1[i][1] - pos2[i][1]).abs() < 1e-8);
        }
    }

    #[test]
    fn test_mds_distance_preservation() {
        // For a path graph, MDS should roughly preserve relative distances
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let pos = layout_mds(&g, None).unwrap();

        // Distance 0-1 should be less than distance 0-3
        let d01 = ((pos[0][0] - pos[1][0]).powi(2) + (pos[0][1] - pos[1][1]).powi(2)).sqrt();
        let d03 = ((pos[0][0] - pos[3][0]).powi(2) + (pos[0][1] - pos[3][1]).powi(2)).sqrt();
        assert!(d01 < d03, "d01={d01} should be < d03={d03}");
    }

    #[test]
    fn test_mds_complete_graph() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        let pos = layout_mds(&g, None).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }
}
