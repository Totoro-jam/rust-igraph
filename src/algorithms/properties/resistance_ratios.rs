//! Resistance-distance-based ratio indices (ALGO-TR-119).
//!
//! Measures derived from effective resistance (Kirchhoff) concepts:
//!
//! - **Kirchhoff index ratio** — Kirchhoff index / (n*(n-1)/2 * diameter),
//!   normalized resistance sum
//! - **Resistance regularity** — min effective resistance / max effective
//!   resistance between adjacent pairs
//! - **Spanning tree ratio** — log(number of spanning trees) / (n-1)*log(n),
//!   a normalized complexity measure via Kirchhoff's matrix-tree theorem

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the Kirchhoff index ratio.
///
/// The Kirchhoff index Kf(G) is the sum of effective resistances over all
/// vertex pairs. For a connected graph, we normalize by the number of pairs
/// times the diameter: `Kf / (pairs * diameter)`. Values near 1 indicate
/// a tree-like resistance structure; lower values indicate more redundant
/// paths. Returns 0.0 for disconnected or trivial graphs.
///
/// We approximate the Kirchhoff index using BFS distances: for connected
/// graphs, `resistance(u,v) >= dist(u,v)/max_degree` and
/// `resistance(u,v) <= dist(u,v)`. We use the sum of distances divided
/// by pairs*diameter as a proxy (the Wiener index ratio).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, kirchhoff_index_ratio};
///
/// // Path graph 0-1-2-3: tree, sum of distances = 1+2+3+1+2+1 = 10
/// // pairs=6, diameter=3, ratio = 10/(6*3) = 10/18 ≈ 0.556
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let r = kirchhoff_index_ratio(&g).unwrap();
/// assert!(r > 0.5 && r < 0.6);
/// ```
pub fn kirchhoff_index_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let (dist_sum, diameter, connected) = bfs_all_pairs_stats(graph, n)?;
    if !connected || diameter == 0 {
        return Ok(0.0);
    }

    let pairs = n * (n - 1) / 2;
    Ok(dist_sum as f64 / (pairs as f64 * diameter as f64))
}

/// Compute the resistance regularity ratio.
///
/// For each edge (u,v), the effective resistance is at least 1/min(deg(u),deg(v))
/// and at most 1. We use `1/min(deg(u), deg(v))` as a proxy for edge
/// resistance, then compute `min_resistance / max_resistance` over all edges.
/// Values near 1 indicate uniform edge resistances (regular graph);
/// values near 0 indicate highly non-uniform resistances. Returns 0.0
/// for trivial or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, resistance_regularity};
///
/// // K_3: all edges have same resistance → ratio = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((resistance_regularity(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn resistance_regularity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut min_r = f64::MAX;
    let mut max_r = 0.0_f64;

    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if ui > v {
                let min_deg = degrees[v].min(degrees[ui]);
                if min_deg == 0 {
                    continue;
                }
                let r = 1.0 / min_deg as f64;
                if r < min_r {
                    min_r = r;
                }
                if r > max_r {
                    max_r = r;
                }
            }
        }
    }

    if max_r < 1e-30 {
        return Ok(0.0);
    }

    Ok(min_r / max_r)
}

/// Compute the spanning tree ratio.
///
/// Uses Kirchhoff's matrix-tree theorem: the number of spanning trees τ(G)
/// equals (1/n) * product of non-zero Laplacian eigenvalues. We compute
/// `log(τ) / ((n-1) * log(n))` as a normalized measure. Values near 1
/// indicate a graph rich in spanning trees (complete-graph-like); values
/// near 0 indicate few spanning trees (tree-like). Returns 0.0 for
/// disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, spanning_tree_ratio};
///
/// // K_3: τ = 3, log(3)/((3-1)*log(3)) = 1/(2) = 0.5
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let r = spanning_tree_ratio(&g).unwrap();
/// assert!(r > 0.45 && r < 0.55);
/// ```
pub fn spanning_tree_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    // Build Laplacian matrix
    let laplacian = build_laplacian(graph, n)?;

    // Compute eigenvalues via QR iteration
    let eigenvalues = symmetric_eigenvalues(&laplacian, n);

    // Sum log of non-zero eigenvalues (those > epsilon)
    let eps = 1e-10;
    let mut log_product = 0.0_f64;
    let mut nonzero_count = 0_usize;
    for &ev in &eigenvalues {
        if ev > eps {
            log_product += ev.ln();
            nonzero_count += 1;
        }
    }

    if nonzero_count < n - 1 {
        // Graph is disconnected
        return Ok(0.0);
    }

    // log(τ) = log_product - log(n)
    let log_tau = log_product - (n as f64).ln();
    let normalizer = (n - 1) as f64 * (n as f64).ln();
    if normalizer < 1e-30 {
        return Ok(0.0);
    }

    Ok((log_tau / normalizer).clamp(0.0, 1.0))
}

/// BFS from every vertex, return (`sum_of_distances`, `diameter`, `is_connected`).
fn bfs_all_pairs_stats(graph: &Graph, n: usize) -> IgraphResult<(u64, u32, bool)> {
    let mut total_sum = 0_u64;
    let mut diameter = 0_u32;

    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        let mut visit_count = 1_usize;

        while let Some(v) = queue.pop_front() {
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                let ui = u as usize;
                if dist[ui] == u32::MAX {
                    dist[ui] = dist[v] + 1;
                    visit_count += 1;
                    queue.push_back(ui);
                }
            }
        }

        if visit_count < n {
            return Ok((0, 0, false));
        }

        for t in (s + 1)..n {
            total_sum += dist[t] as u64;
            if dist[t] > diameter {
                diameter = dist[t];
            }
        }
    }

    Ok((total_sum, diameter, true))
}

/// Build the Laplacian matrix L = D - A.
fn build_laplacian(graph: &Graph, n: usize) -> IgraphResult<Vec<Vec<f64>>> {
    let mut lap = vec![vec![0.0_f64; n]; n];

    for v in 0..n {
        let nbrs = graph.neighbors(v as u32)?;
        lap[v][v] = nbrs.len() as f64;
        for &u in &nbrs {
            let ui = u as usize;
            lap[v][ui] -= 1.0;
        }
    }

    Ok(lap)
}

/// Compute eigenvalues of a symmetric matrix via Jacobi iteration.
fn symmetric_eigenvalues(mat: &[Vec<f64>], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![mat[0][0]];
    }

    let mut a = mat.to_vec();
    let max_iter = 100 * n * n;

    for _ in 0..max_iter {
        // Find the largest off-diagonal element
        let mut max_val = 0.0_f64;
        let mut p = 0_usize;
        let mut q = 1_usize;
        for i in 0..n {
            for j in (i + 1)..n {
                if a[i][j].abs() > max_val {
                    max_val = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }

        if max_val < 1e-12 {
            break;
        }

        // Compute rotation angle
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        let theta = if (app - aqq).abs() < 1e-30 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };

        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // Apply Jacobi rotation
        let mut new_a = a.clone();
        for i in 0..n {
            if i != p && i != q {
                new_a[i][p] = cos_t * a[i][p] + sin_t * a[i][q];
                new_a[p][i] = new_a[i][p];
                new_a[i][q] = -sin_t * a[i][p] + cos_t * a[i][q];
                new_a[q][i] = new_a[i][q];
            }
        }
        new_a[p][p] = cos_t * cos_t * app + 2.0 * sin_t * cos_t * apq + sin_t * sin_t * aqq;
        new_a[q][q] = sin_t * sin_t * app - 2.0 * sin_t * cos_t * apq + cos_t * cos_t * aqq;
        new_a[p][q] = 0.0;
        new_a[q][p] = 0.0;

        a = new_a;
    }

    (0..n).map(|i| a[i][i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Graph {
        Graph::with_vertices(0)
    }

    fn single() -> Graph {
        Graph::with_vertices(1)
    }

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn k3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn k4() -> Graph {
        Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn star5() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    fn disconnected() -> Graph {
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
    }

    // --- kirchhoff_index_ratio ---

    #[test]
    fn kir_empty() {
        assert!(kirchhoff_index_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kir_single() {
        assert!(kirchhoff_index_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kir_disconnected() {
        assert!(kirchhoff_index_ratio(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn kir_single_edge() {
        // sum=1, pairs=1, diameter=1 → 1.0
        assert!((kirchhoff_index_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn kir_k3() {
        // distances: 1+1+1=3, pairs=3, diameter=1 → 3/(3*1)=1.0
        assert!((kirchhoff_index_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn kir_path4() {
        // distances: 1+2+3+1+2+1=10, pairs=6, diameter=3 → 10/18 ≈ 0.556
        let r = kirchhoff_index_ratio(&path4()).unwrap();
        assert!((r - 10.0 / 18.0).abs() < 1e-10);
    }

    #[test]
    fn kir_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let r = kirchhoff_index_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- resistance_regularity ---

    #[test]
    fn rr_empty() {
        assert!(resistance_regularity(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rr_single() {
        assert!(resistance_regularity(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn rr_k3() {
        // All degrees 2, all edges have resistance proxy 1/2 → ratio = 1.0
        assert!((resistance_regularity(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_k4() {
        // All degrees 3 → ratio = 1.0
        assert!((resistance_regularity(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_cycle4() {
        // All degrees 2 → ratio = 1.0
        assert!((resistance_regularity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_star5() {
        // Center degree 4, leaves degree 1
        // Edge resistance proxy: 1/min(4,1) = 1/1 = 1.0 for all edges
        // So ratio = 1.0
        assert!((resistance_regularity(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_path3() {
        // degrees: 1, 2, 1
        // edge (0,1): 1/min(1,2) = 1/1 = 1.0
        // edge (1,2): 1/min(2,1) = 1/1 = 1.0
        // ratio = 1.0
        assert!((resistance_regularity(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let r = resistance_regularity(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn rr_paw() {
        // Paw: 0-1, 1-2, 0-2, 2-3. Degrees: 2, 2, 3, 1
        // edge(0,1): 1/min(2,2) = 0.5
        // edge(1,2): 1/min(2,3) = 0.5
        // edge(0,2): 1/min(2,3) = 0.5
        // edge(2,3): 1/min(3,1) = 1.0
        // min=0.5, max=1.0, ratio=0.5
        let g = Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap();
        assert!((resistance_regularity(&g).unwrap() - 0.5).abs() < 1e-10);
    }

    // --- spanning_tree_ratio ---

    #[test]
    fn str_empty() {
        assert!(spanning_tree_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn str_single() {
        assert!(spanning_tree_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn str_disconnected() {
        assert!(spanning_tree_ratio(&disconnected()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn str_single_edge() {
        // τ=1, log(1)=0, ratio=0. But normalizer = (2-1)*log(2) = log(2)
        // log(τ)/normalizer = 0/log(2) = 0
        let r = spanning_tree_ratio(&single_edge()).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn str_k3() {
        // τ(K_3)=3, log(3)/((3-1)*log(3)) = 1/2 = 0.5
        let r = spanning_tree_ratio(&k3()).unwrap();
        assert!((r - 0.5).abs() < 0.05);
    }

    #[test]
    fn str_k4() {
        // τ(K_4)=16, log(16)/((4-1)*log(4)) = 4*log(2)/(3*2*log(2)) = 4/6 = 2/3
        let r = spanning_tree_ratio(&k4()).unwrap();
        assert!((r - 2.0 / 3.0).abs() < 0.05);
    }

    #[test]
    fn str_path_tree() {
        // Path (tree) has τ=1 → log(1)=0 → ratio=0
        let r = spanning_tree_ratio(&path4()).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn str_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            let r = spanning_tree_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn str_cycle4() {
        // τ(C_4)=4, log(4)/((4-1)*log(4)) = 1/3
        let r = spanning_tree_ratio(&cycle4()).unwrap();
        assert!((r - 1.0 / 3.0).abs() < 0.05);
    }

    // --- cross-consistency ---

    #[test]
    fn regular_graphs_unit_resistance() {
        // Regular graphs should have resistance regularity = 1.0
        assert!((resistance_regularity(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((resistance_regularity(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((resistance_regularity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trees_zero_spanning_ratio() {
        // Trees have exactly 1 spanning tree → ratio = 0
        assert!(spanning_tree_ratio(&path3()).unwrap().abs() < 1e-10);
        assert!(spanning_tree_ratio(&path4()).unwrap().abs() < 1e-10);
        assert!(spanning_tree_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn complete_diameter_one_kirchhoff() {
        // Complete graphs: diameter=1, all distances=1, sum=pairs → ratio=1.0
        assert!((kirchhoff_index_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((kirchhoff_index_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
