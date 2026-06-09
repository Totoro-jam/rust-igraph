//! Resilience-based ratio indices (ALGO-TR-106).
//!
//! Ratios capturing network resilience and vulnerability:
//!
//! - **Vertex connectivity ratio** — `min_degree / (n-1)` proxy for connected,
//!   0 for disconnected
//! - **Edge connectivity ratio** — `min_degree / (n-1)` edge analog
//! - **Diameter vulnerability** — max increase in diameter when removing
//!   one vertex, normalized
//! - **Neighbor degree disparity** — mean `knn(v) / degree(v)` over
//!   vertices with `degree >= 1`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the vertex connectivity ratio.
///
/// For connected graphs: `min_degree / (n-1)`. For disconnected
/// graphs: 0.0. This is an upper bound proxy for the actual vertex
/// connectivity κ(G) / (n-1). Returns 0.0 for trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_conn_ratio};
///
/// // K_4: min_degree=3, n-1=3 → 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((vertex_conn_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn vertex_conn_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    if !is_connected_bfs(graph)? {
        return Ok(0.0);
    }

    let mut min_deg = usize::MAX;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < min_deg {
            min_deg = d;
        }
    }

    Ok(min_deg as f64 / (n - 1) as f64)
}

/// Compute the edge connectivity ratio.
///
/// For connected graphs: `min_degree / (n-1)`. This is the same
/// formula as vertex connectivity ratio since both use `min_degree`
/// as a proxy (Whitney's theorem: κ(G) ≤ κ'(G) ≤ δ(G)). Returns
/// 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_conn_ratio};
///
/// // Cycle_4: min_degree=2, n-1=3 → 2/3
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,0)], false, Some(4)
/// ).unwrap();
/// assert!((edge_conn_ratio(&g).unwrap() - 2.0/3.0).abs() < 1e-10);
/// ```
pub fn edge_conn_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    if !is_connected_bfs(graph)? {
        return Ok(0.0);
    }

    let mut min_deg = usize::MAX;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        if d < min_deg {
            min_deg = d;
        }
    }

    Ok(min_deg as f64 / (n - 1) as f64)
}

/// Compute the diameter vulnerability.
///
/// For each vertex v, compute the diameter of G - v (graph with v
/// removed). The vulnerability is the maximum increase in diameter
/// over all removals, normalized by the original diameter. Returns
/// 0.0 for disconnected, trivial graphs, or graphs with zero diameter.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, diameter_vulnerability};
///
/// // K_3: removing any vertex leaves K_2, diameter unchanged (1→1) → 0.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!(diameter_vulnerability(&g).unwrap().abs() < 1e-10);
/// ```
pub fn diameter_vulnerability(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let orig_diam = compute_diameter(graph)?;
    if orig_diam == 0 || orig_diam == u32::MAX {
        return Ok(0.0);
    }

    let mut max_increase = 0_i64;

    for removed in 0..n {
        let sub_diam = compute_diameter_without(graph, removed)?;
        if sub_diam == u32::MAX {
            // subgraph disconnected — treat as infinite increase
            // but we can cap it reasonably
            let increase = i64::from(orig_diam);
            if increase > max_increase {
                max_increase = increase;
            }
        } else {
            let increase = i64::from(sub_diam) - i64::from(orig_diam);
            if increase > max_increase {
                max_increase = increase;
            }
        }
    }

    if max_increase <= 0 {
        return Ok(0.0);
    }

    Ok(max_increase as f64 / f64::from(orig_diam))
}

/// Compute the average neighbor degree ratio.
///
/// Mean of `knn(v) / degree(v)` over all vertices with `degree >= 1`,
/// where `knn(v)` is the average degree of v's neighbors. This
/// measures the tendency for high-degree vertices to connect to
/// other high-degree vertices. Returns 0.0 for graphs with no
/// edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, neighbor_degree_disparity};
///
/// // K_3: each vertex has degree 2, neighbors have degree 2 → knn/d = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((neighbor_degree_disparity(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn neighbor_degree_disparity(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut sum = 0.0_f64;
    let mut count = 0_usize;

    for v in 0..n {
        let dv = degrees[v];
        if dv == 0 {
            continue;
        }

        let neighbors = graph.neighbors(v as u32)?;
        let knn: f64 = neighbors
            .iter()
            .map(|&u| degrees[u as usize] as f64)
            .sum::<f64>()
            / dv as f64;
        sum += knn / dv as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64)
}

fn is_connected_bfs(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(true);
    }

    let mut visited = vec![false; n];
    visited[0] = true;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0_usize);
    let mut seen = 1_usize;

    while let Some(v) = queue.pop_front() {
        let neighbors = graph.neighbors(v as u32)?;
        for &u in &neighbors {
            let ui = u as usize;
            if !visited[ui] {
                visited[ui] = true;
                seen += 1;
                queue.push_back(ui);
            }
        }
    }

    Ok(seen == n)
}

fn compute_diameter(graph: &Graph) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0);
    }

    let mut max_d = 0_u32;
    for s in 0..n {
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(v) = queue.pop_front() {
            let d = dist[v];
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if dist[ui] == u32::MAX {
                    dist[ui] = d + 1;
                    queue.push_back(ui);
                }
            }
        }
        for u in 0..n {
            if dist[u] == u32::MAX {
                return Ok(u32::MAX);
            }
            if dist[u] > max_d {
                max_d = dist[u];
            }
        }
    }

    Ok(max_d)
}

fn compute_diameter_without(graph: &Graph, removed: usize) -> IgraphResult<u32> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0);
    }

    let mut max_d = 0_u32;
    for s in 0..n {
        if s == removed {
            continue;
        }
        let mut dist = vec![u32::MAX; n];
        dist[s] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        while let Some(v) = queue.pop_front() {
            let d = dist[v];
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if ui == removed {
                    continue;
                }
                if dist[ui] == u32::MAX {
                    dist[ui] = d + 1;
                    queue.push_back(ui);
                }
            }
        }
        for u in 0..n {
            if u == removed {
                continue;
            }
            if dist[u] == u32::MAX {
                return Ok(u32::MAX);
            }
            if dist[u] > max_d {
                max_d = dist[u];
            }
        }
    }

    Ok(max_d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_edge() -> Graph {
        Graph::from_edges(&[(0, 1)], false, Some(2)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
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

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- vertex_conn_ratio ---

    #[test]
    fn vcr_empty() {
        let g = Graph::with_vertices(0);
        assert!(vertex_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_single() {
        let g = Graph::with_vertices(1);
        assert!(vertex_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_k3() {
        // min_deg=2, n-1=2 → 1.0
        assert!((vertex_conn_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_k4() {
        // min_deg=3, n-1=3 → 1.0
        assert!((vertex_conn_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_cycle4() {
        // min_deg=2, n-1=3 → 2/3
        assert!((vertex_conn_ratio(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_star5() {
        // min_deg=1, n-1=4 → 1/4
        assert!((vertex_conn_ratio(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn vcr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(vertex_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = vertex_conn_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- edge_conn_ratio ---

    #[test]
    fn ecr_empty() {
        let g = Graph::with_vertices(0);
        assert!(edge_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_single() {
        let g = Graph::with_vertices(1);
        assert!(edge_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ecr_k3() {
        assert!((edge_conn_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_k4() {
        assert!((edge_conn_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_cycle4() {
        assert!((edge_conn_ratio(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ecr_star5() {
        assert!((edge_conn_ratio(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn ecr_disconnected() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        assert!(edge_conn_ratio(&g).unwrap().abs() < 1e-10);
    }

    // --- diameter_vulnerability ---

    #[test]
    fn dv_empty() {
        let g = Graph::with_vertices(0);
        assert!(diameter_vulnerability(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_single() {
        let g = Graph::with_vertices(1);
        assert!(diameter_vulnerability(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_two() {
        assert!(diameter_vulnerability(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_k3() {
        // Removing any vertex: diameter stays 1 → 0
        assert!(diameter_vulnerability(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_k4() {
        // Removing any vertex: K_3 → diameter 1 (same as K_4) → 0
        assert!(diameter_vulnerability(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_path3() {
        // Original diameter=2. Remove v1: disconnected → vulnerability = 2/2 = 1.0
        assert!((diameter_vulnerability(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dv_cycle4() {
        // diameter=2. Remove any vertex: path of 3, diameter=2 → no increase → 0
        assert!(diameter_vulnerability(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn dv_star5() {
        // diameter=2. Remove center: 4 isolated vertices → disconnected → increase = 2/2 = 1.0
        assert!((diameter_vulnerability(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn dv_nonneg() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(diameter_vulnerability(g).unwrap() >= -1e-10);
        }
    }

    // --- neighbor_degree_disparity ---

    #[test]
    fn andr_empty() {
        let g = Graph::with_vertices(0);
        assert!(neighbor_degree_disparity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn andr_single() {
        let g = Graph::with_vertices(1);
        assert!(neighbor_degree_disparity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn andr_no_edges() {
        let g = Graph::with_vertices(5);
        assert!(neighbor_degree_disparity(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn andr_k3() {
        // Each vertex: degree=2, neighbors have degree=2 → knn/d = 2/2 = 1.0
        assert!((neighbor_degree_disparity(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_k4() {
        // Each vertex: degree=3, neighbors have degree=3 → knn/d = 3/3 = 1.0
        assert!((neighbor_degree_disparity(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_cycle4() {
        // Each vertex: degree=2, neighbors have degree=2 → knn/d = 2/2 = 1.0
        assert!((neighbor_degree_disparity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_star5() {
        // Center: d=4, knn=4*1/4=1 → knn/d = 1/4 = 0.25
        // Leaf: d=1, knn=4/1=4 → knn/d = 4/1 = 4.0
        // avg = (0.25 + 4*4.0) / 5 = 16.25/5 = 3.25
        let r = neighbor_degree_disparity(&star5()).unwrap();
        assert!((r - 3.25).abs() < 1e-10);
    }

    #[test]
    fn andr_single_edge() {
        // Both have d=1, knn=1 → knn/d = 1.0
        assert!((neighbor_degree_disparity(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn andr_positive() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            assert!(neighbor_degree_disparity(g).unwrap() > 0.0);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn complete_max_connectivity() {
        assert!((vertex_conn_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((vertex_conn_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_conn_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((edge_conn_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn complete_zero_vulnerability() {
        assert!(diameter_vulnerability(&k3()).unwrap().abs() < 1e-10);
        assert!(diameter_vulnerability(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn regular_unit_neighbor_ratio() {
        assert!((neighbor_degree_disparity(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((neighbor_degree_disparity(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((neighbor_degree_disparity(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
