//! Flow-based ratio indices (ALGO-TR-118).
//!
//! Measures derived from network flow and connectivity concepts:
//!
//! - **Max-flow efficiency** — average max-flow between all pairs /
//!   maximum possible flow (related to edge connectivity)
//! - **Bottleneck ratio** — minimum edge betweenness / maximum edge
//!   betweenness, measuring flow bottleneck concentration
//! - **Flow hierarchy** — fraction of edges that are in a minimum
//!   spanning tree (backbone edges)

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

/// Compute the max-flow efficiency ratio.
///
/// For each pair of vertices, the max-flow equals the edge connectivity
/// between them. We approximate this using the minimum degree of the
/// two endpoints (an upper bound on the local edge connectivity).
/// Returns the average over all pairs divided by the global minimum
/// degree. Values near 1 indicate uniform connectivity; values < 1
/// indicate some pairs have weaker connections. Returns 0.0 for
/// disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, max_flow_efficiency};
///
/// // K_4: all pairs have connectivity 3, min_deg=3 → ratio=1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((max_flow_efficiency(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn max_flow_efficiency(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let mut degrees = Vec::with_capacity(n);
    let mut min_deg = usize::MAX;
    for v in 0..n {
        let d = graph.degree(v as u32)?;
        degrees.push(d);
        if d < min_deg {
            min_deg = d;
        }
    }

    if min_deg == 0 {
        return Ok(0.0);
    }

    let mut sum_min_deg = 0_u64;
    let mut pairs = 0_u64;
    for v in 0..n {
        for u in (v + 1)..n {
            sum_min_deg += degrees[v].min(degrees[u]) as u64;
            pairs += 1;
        }
    }

    if pairs == 0 {
        return Ok(0.0);
    }

    let avg_min_deg = sum_min_deg as f64 / pairs as f64;
    Ok(avg_min_deg / min_deg as f64)
}

/// Compute the bottleneck ratio.
///
/// `min_edge_betweenness / max_edge_betweenness` — measures how
/// concentrated flow bottlenecks are. Values near 1 indicate all edges
/// carry similar load (uniform flow); values near 0 indicate a few
/// edges carry most of the flow. Uses shortest-path betweenness.
/// Returns 0.0 for trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bottleneck_ratio};
///
/// // K_3: all edge betweennesses equal → ratio = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((bottleneck_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn bottleneck_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let betweenness = edge_betweenness(graph, n)?;
    if betweenness.is_empty() {
        return Ok(0.0);
    }

    let mut min_b = f64::MAX;
    let mut max_b = 0.0_f64;
    for &b in &betweenness {
        if b < min_b {
            min_b = b;
        }
        if b > max_b {
            max_b = b;
        }
    }

    if max_b < 1e-30 {
        return Ok(0.0);
    }

    Ok(min_b / max_b)
}

/// Compute the flow hierarchy ratio.
///
/// Fraction of edges that would be in a minimum spanning tree (assuming
/// unit weights, this equals (n-1)/m for connected graphs). Measures
/// how tree-like the graph is. Values near 1 indicate a tree (all edges
/// are bridges); values near 0 indicate a densely connected graph.
/// Returns 0.0 for disconnected or trivial graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, flow_hierarchy_ratio};
///
/// // Tree (path 0-1-2-3): all edges in MST → (n-1)/m = 3/3 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// assert!((flow_hierarchy_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn flow_hierarchy_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    // Check connectivity via BFS from vertex 0
    let mut visited = vec![false; n];
    visited[0] = true;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(0_usize);
    let mut visit_count = 1_usize;

    while let Some(v) = queue.pop_front() {
        let nbrs = graph.neighbors(v as u32)?;
        for &u in &nbrs {
            let ui = u as usize;
            if !visited[ui] {
                visited[ui] = true;
                visit_count += 1;
                queue.push_back(ui);
            }
        }
    }

    if visit_count < n {
        return Ok(0.0);
    }

    // For a connected graph, MST has n-1 edges
    Ok((n - 1) as f64 / m as f64)
}

/// Compute edge betweenness for all edges via BFS from every vertex.
fn edge_betweenness(graph: &Graph, n: usize) -> IgraphResult<Vec<f64>> {
    // Store betweenness per edge using (min(u,v), max(u,v)) as key
    let mut bet_map: std::collections::HashMap<(u32, u32), f64> = std::collections::HashMap::new();

    for s in 0..n {
        // BFS
        let mut dist = vec![u32::MAX; n];
        let mut sigma = vec![0_u64; n]; // number of shortest paths
        let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut order = Vec::new();

        dist[s] = 0;
        sigma[s] = 1;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            order.push(v);
            let nbrs = graph.neighbors(v as u32)?;
            for &u in &nbrs {
                let ui = u as usize;
                if dist[ui] == u32::MAX {
                    dist[ui] = dist[v] + 1;
                    queue.push_back(ui);
                }
                if dist[ui] == dist[v] + 1 {
                    sigma[ui] += sigma[v];
                    pred[ui].push(v);
                }
            }
        }

        // Back-propagation
        let mut delta = vec![0.0_f64; n];
        for &w in order.iter().rev() {
            for &v in &pred[w] {
                let coeff = (sigma[v] as f64 / sigma[w] as f64) * (1.0 + delta[w]);
                let e = (v.min(w) as u32, v.max(w) as u32);
                *bet_map.entry(e).or_insert(0.0) += coeff;
                delta[v] += coeff;
            }
        }
    }

    Ok(bet_map.into_values().collect())
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

    fn paw() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- max_flow_efficiency ---

    #[test]
    fn mfe_empty() {
        assert!(max_flow_efficiency(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mfe_single() {
        assert!(max_flow_efficiency(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn mfe_k3() {
        // All degrees 2, min_deg=2, all pair-min = 2 → avg=2, 2/2=1.0
        assert!((max_flow_efficiency(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mfe_k4() {
        assert!((max_flow_efficiency(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mfe_cycle4() {
        assert!((max_flow_efficiency(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mfe_star5() {
        // Degrees: 4,1,1,1,1; min_deg=1
        // Pairs: center-leaf: min=1; leaf-leaf: min=1
        // All pairs have min=1, avg=1, 1/1=1.0
        assert!((max_flow_efficiency(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mfe_ge_1() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = max_flow_efficiency(g).unwrap();
            assert!(r >= 1.0 - 1e-10);
        }
    }

    // --- bottleneck_ratio ---

    #[test]
    fn br_empty() {
        assert!(bottleneck_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_single() {
        assert!(bottleneck_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_k3() {
        // All edges have equal betweenness → ratio = 1.0
        assert!((bottleneck_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_k4() {
        assert!((bottleneck_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_cycle4() {
        // Symmetric → all equal → 1.0
        assert!((bottleneck_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = bottleneck_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- flow_hierarchy_ratio ---

    #[test]
    fn fhr_empty() {
        assert!(flow_hierarchy_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fhr_single() {
        assert!(flow_hierarchy_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn fhr_path3() {
        // Tree: (n-1)/m = 2/2 = 1.0
        assert!((flow_hierarchy_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fhr_path4() {
        assert!((flow_hierarchy_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fhr_star5() {
        // Tree: (n-1)/m = 4/4 = 1.0
        assert!((flow_hierarchy_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fhr_k3() {
        // (3-1)/3 = 2/3
        assert!((flow_hierarchy_ratio(&k3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn fhr_k4() {
        // (4-1)/6 = 3/6 = 0.5
        assert!((flow_hierarchy_ratio(&k4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn fhr_cycle4() {
        // (4-1)/4 = 3/4 = 0.75
        assert!((flow_hierarchy_ratio(&cycle4()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn fhr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = flow_hierarchy_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn regular_unit_bottleneck() {
        assert!((bottleneck_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((bottleneck_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((bottleneck_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trees_unit_hierarchy() {
        assert!((flow_hierarchy_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((flow_hierarchy_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((flow_hierarchy_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }
}
