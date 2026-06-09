//! Subgraph ratio indices (ALGO-TR-097).
//!
//! Simple substructure density measures:
//!
//! - **Pendant edge ratio** — fraction of edges incident to degree-1 vertices
//! - **Bridge ratio** — fraction of edges that are bridges (via Tarjan)
//! - **Triangle participation** — fraction of vertices in at least one triangle
//! - **Isolated vertex ratio** — fraction of vertices with degree 0

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the pendant edge ratio.
///
/// Fraction of edges that are incident to at least one degree-1
/// vertex. A pendant edge connects a leaf to the rest of the graph.
/// Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pendant_edge_ratio};
///
/// // Star S_5: all 4 edges are pendant → 1.0
/// let g = Graph::from_edges(&[(0,1),(0,2),(0,3),(0,4)], false, Some(5)).unwrap();
/// assert!((pendant_edge_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn pendant_edge_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let n = graph.vcount() as usize;
    let mut degrees = Vec::with_capacity(n);
    for v in 0..n {
        degrees.push(graph.degree(v as u32)?);
    }

    let mut pendant_count = 0_usize;
    for (u, v) in graph.edges() {
        if degrees[u as usize] == 1 || degrees[v as usize] == 1 {
            pendant_count += 1;
        }
    }

    Ok(pendant_count as f64 / m as f64)
}

/// Compute the bridge ratio of the graph.
///
/// Fraction of edges that are bridges (whose removal disconnects
/// the graph). Uses Tarjan's bridge-finding algorithm with DFS.
/// Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bridge_ratio};
///
/// // Path 0-1-2: both edges are bridges → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((bridge_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn bridge_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let bridge_count = count_bridges(graph)?;
    Ok(bridge_count as f64 / m as f64)
}

/// Compute the triangle participation ratio.
///
/// Fraction of vertices that participate in at least one triangle.
/// Returns 0.0 for graphs with fewer than 3 vertices or no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, triangle_participation};
///
/// // K_3: all 3 vertices in a triangle → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((triangle_participation(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn triangle_participation(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 {
        return Ok(0.0);
    }

    let mut in_triangle = vec![false; n];

    for v in 0..n {
        if in_triangle[v] {
            continue;
        }
        let vid = v as u32;
        let neighbors = graph.neighbors(vid)?;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let a = neighbors[i];
                let b = neighbors[j];
                if graph.has_edge(a, b) {
                    in_triangle[v] = true;
                    in_triangle[a as usize] = true;
                    in_triangle[b as usize] = true;
                }
            }
        }
    }

    let count = in_triangle.iter().filter(|&&x| x).count();
    Ok(count as f64 / n as f64)
}

/// Compute the isolated vertex ratio.
///
/// Fraction of vertices with degree 0. Returns 0.0 for empty
/// graphs (no vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, isolated_vertex_ratio};
///
/// // 5 isolated vertices → 1.0
/// let g = Graph::with_vertices(5);
/// assert!((isolated_vertex_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn isolated_vertex_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut isolated = 0_usize;
    for v in 0..n {
        if graph.degree(v as u32)? == 0 {
            isolated += 1;
        }
    }

    Ok(isolated as f64 / n as f64)
}

fn count_bridges(graph: &Graph) -> IgraphResult<usize> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut disc = vec![0_u32; n];
    let mut low = vec![0_u32; n];
    let mut visited = vec![false; n];
    let mut timer = 1_u32;
    let mut bridge_count = 0_usize;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut stack: Vec<(u32, i64, usize)> = vec![(start as u32, -1, 0)];
        visited[start] = true;
        disc[start] = timer;
        low[start] = timer;
        timer += 1;

        while let Some(&mut (v, parent, ref mut ni)) = stack.last_mut() {
            let neighbors = graph.neighbors(v)?;
            if *ni < neighbors.len() {
                let u = neighbors[*ni];
                *ni += 1;
                if i64::from(u) == parent {
                    continue;
                }
                let ui = u as usize;
                if visited[ui] {
                    if disc[ui] < low[v as usize] {
                        low[v as usize] = disc[ui];
                    }
                } else {
                    visited[ui] = true;
                    disc[ui] = timer;
                    low[ui] = timer;
                    timer += 1;
                    stack.push((u, i64::from(v), 0));
                }
            } else {
                let vi = v as usize;
                if parent >= 0 {
                    let pi = parent as usize;
                    if low[vi] < low[pi] {
                        low[pi] = low[vi];
                    }
                    if low[vi] > disc[pi] {
                        bridge_count += 1;
                    }
                }
                stack.pop();
            }
        }
    }

    Ok(bridge_count)
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

    // --- pendant_edge_ratio ---

    #[test]
    fn per_empty() {
        let g = Graph::with_vertices(0);
        assert!(pendant_edge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn per_isolated() {
        let g = Graph::with_vertices(5);
        assert!(pendant_edge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn per_single_edge() {
        // Both endpoints have d=1 → 1 pendant edge / 1 edge = 1.0
        assert!((pendant_edge_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn per_path3() {
        // Edges: (0,1) d(0)=1→pendant, (1,2) d(2)=1→pendant → 2/2 = 1.0
        assert!((pendant_edge_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn per_k3() {
        // All d=2, no pendants → 0
        assert!(pendant_edge_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn per_k4() {
        // All d=3, no pendants → 0
        assert!(pendant_edge_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn per_cycle4() {
        // All d=2, no pendants → 0
        assert!(pendant_edge_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn per_star5() {
        // All edges are pendant (leaves have d=1) → 4/4 = 1.0
        assert!((pendant_edge_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn per_paw() {
        // Edge (2,3): d(3)=1 → pendant. Others: no d=1 endpoints
        // 1 pendant / 4 edges = 0.25
        assert!((pendant_edge_ratio(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- bridge_ratio ---

    #[test]
    fn br_empty() {
        let g = Graph::with_vertices(0);
        assert!(bridge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_isolated() {
        let g = Graph::with_vertices(5);
        assert!(bridge_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_single_edge() {
        // 1 bridge / 1 edge = 1.0
        assert!((bridge_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_path3() {
        // Both edges are bridges → 2/2 = 1.0
        assert!((bridge_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_k3() {
        // No bridges (biconnected) → 0
        assert!(bridge_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_k4() {
        // No bridges → 0
        assert!(bridge_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_cycle4() {
        // No bridges → 0
        assert!(bridge_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_star5() {
        // All 4 edges are bridges → 4/4 = 1.0
        assert!((bridge_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_paw() {
        // Edge (2,3) is a bridge, triangle edges are not → 1/4 = 0.25
        assert!((bridge_ratio(&paw()).unwrap() - 0.25).abs() < 1e-10);
    }

    // --- triangle_participation ---

    #[test]
    fn tp_empty() {
        let g = Graph::with_vertices(0);
        assert!(triangle_participation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tp_two() {
        let g = Graph::with_vertices(2);
        assert!(triangle_participation(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tp_path3() {
        // No triangles → 0
        assert!(triangle_participation(&path3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tp_k3() {
        // All in triangle → 3/3 = 1.0
        assert!((triangle_participation(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tp_k4() {
        // All in triangles → 4/4 = 1.0
        assert!((triangle_participation(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn tp_cycle4() {
        // No triangles → 0
        assert!(triangle_participation(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tp_star5() {
        // No triangles → 0
        assert!(triangle_participation(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn tp_paw() {
        // Triangle: {0,1,2} → 3 vertices in triangle, v3 not
        // 3/4 = 0.75
        assert!((triangle_participation(&paw()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn tp_single_edge() {
        // n < 3 → 0
        assert!(triangle_participation(&single_edge()).unwrap().abs() < 1e-10);
    }

    // --- isolated_vertex_ratio ---

    #[test]
    fn ivr_empty() {
        let g = Graph::with_vertices(0);
        assert!(isolated_vertex_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ivr_all_isolated() {
        let g = Graph::with_vertices(5);
        assert!((isolated_vertex_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ivr_single_edge() {
        // 0 isolated / 2 = 0
        assert!(isolated_vertex_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ivr_k3() {
        assert!(isolated_vertex_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ivr_with_isolates() {
        // 5 vertices, 1 edge (0-1), 3 isolated → 3/5 = 0.6
        let g = Graph::from_edges(&[(0, 1)], false, Some(5)).unwrap();
        assert!((isolated_vertex_ratio(&g).unwrap() - 0.6).abs() < 1e-10);
    }

    #[test]
    fn ivr_star5() {
        assert!(isolated_vertex_ratio(&star5()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ivr_paw() {
        assert!(isolated_vertex_ratio(&paw()).unwrap().abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn per_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = pendant_edge_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn br_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = bridge_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn tp_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = triangle_participation(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn ivr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), star5(), paw()] {
            let r = isolated_vertex_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn trees_all_bridges() {
        // In a tree, every edge is a bridge
        assert!((bridge_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((bridge_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pendant_implies_bridge_for_trees() {
        // In a tree, pendant ratio = bridge ratio = 1.0 only if all edges are pendant
        // path3: both pendant and bridge
        // star5: all pendant and all bridge
        let path_p = pendant_edge_ratio(&path3()).unwrap();
        let path_b = bridge_ratio(&path3()).unwrap();
        assert!((path_p - path_b).abs() < 1e-10);

        let star_p = pendant_edge_ratio(&star5()).unwrap();
        let star_b = bridge_ratio(&star5()).unwrap();
        assert!((star_p - star_b).abs() < 1e-10);
    }
}
