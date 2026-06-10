//! Bridge and articulation-point ratio indices (ALGO-TR-111).
//!
//! Measures of structural vulnerability via bridges and cut vertices:
//!
//! - **Bridge ratio** — fraction of edges that are bridges
//! - **Articulation ratio** — fraction of vertices that are cut vertices
//! - **Biconnected ratio** — fraction of edges in the largest
//!   biconnected component
//! - **Leaf ratio** — fraction of vertices with degree 1 (pendant vertices)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the bridge ratio.
///
/// Fraction of edges that are bridges (whose removal disconnects the
/// graph). Uses Tarjan's bridge-finding algorithm in O(V+E). Trees
/// have bridge ratio 1.0; biconnected graphs have 0.0. Returns 0.0
/// for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, bridge_edge_ratio};
///
/// // Path 0-1-2: both edges are bridges → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((bridge_edge_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn bridge_edge_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let bridges = count_bridges(graph)?;
    Ok(bridges as f64 / m as f64)
}

/// Compute the articulation ratio.
///
/// Fraction of vertices that are articulation points (cut vertices
/// whose removal disconnects the graph). Uses a DFS-based algorithm
/// in O(V+E). Returns 0.0 for graphs with fewer than 3 vertices or
/// no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, articulation_ratio};
///
/// // Path 0-1-2: vertex 1 is the only cut vertex → 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// assert!((articulation_ratio(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn articulation_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 3 || graph.ecount() == 0 {
        return Ok(0.0);
    }

    let cut_vertices = count_articulation_points(graph)?;
    Ok(cut_vertices as f64 / n as f64)
}

/// Compute the biconnected ratio.
///
/// Fraction of edges belonging to the largest biconnected component.
/// A biconnected component is a maximal subgraph with no cut vertices.
/// Higher values indicate the graph is dominated by a single robust
/// block. Returns 0.0 for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, biconnected_ratio};
///
/// // K_4: entire graph is biconnected → 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((biconnected_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn biconnected_ratio(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }

    let max_block_edges = largest_block_edge_count(graph)?;
    Ok(max_block_edges as f64 / m as f64)
}

/// Compute the leaf ratio.
///
/// Fraction of vertices with degree exactly 1 (pendant/leaf vertices).
/// These are the most vulnerable vertices — removing their single edge
/// isolates them. Returns 0.0 for empty or edgeless graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, leaf_ratio};
///
/// // Star_5: 4 leaves out of 5 vertices → 4/5
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(0,4)], false, Some(5)
/// ).unwrap();
/// assert!((leaf_ratio(&g).unwrap() - 4.0/5.0).abs() < 1e-10);
/// ```
pub fn leaf_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut leaves = 0_u64;
    for v in 0..n {
        if graph.degree(v as u32)? == 1 {
            leaves += 1;
        }
    }

    Ok(leaves as f64 / n as f64)
}

/// Count bridges using iterative Tarjan's algorithm.
fn count_bridges(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut disc = vec![0_u32; n];
    let mut low = vec![0_u32; n];
    let mut visited = vec![false; n];
    let mut timer = 1_u32;
    let mut bridges = 0_u64;

    for start in 0..n {
        if visited[start] {
            continue;
        }

        // Iterative DFS with stack: (vertex, parent, neighbor_index)
        let mut stack: Vec<(usize, usize, usize)> = Vec::new();
        visited[start] = true;
        disc[start] = timer;
        low[start] = timer;
        timer += 1;
        stack.push((start, usize::MAX, 0));

        while let Some((v, parent, idx)) = stack.last_mut() {
            let v = *v;
            let parent = *parent;
            let nbrs = graph.neighbors(v as u32)?;

            if *idx < nbrs.len() {
                let u = nbrs[*idx] as usize;
                *idx += 1;

                if !visited[u] {
                    visited[u] = true;
                    disc[u] = timer;
                    low[u] = timer;
                    timer += 1;
                    stack.push((u, v, 0));
                } else if u != parent && disc[u] < low[v] {
                    let len = stack.len();
                    let cv = stack[len - 1].0;
                    if disc[u] < low[cv] {
                        low[cv] = disc[u];
                    }
                }
            } else {
                // All neighbors processed, backtrack
                let cv = v;
                stack.pop();
                if let Some(top) = stack.last_mut() {
                    let pv = top.0;
                    if low[cv] < low[pv] {
                        low[pv] = low[cv];
                    }
                    if low[cv] > disc[pv] {
                        bridges += 1;
                    }
                }
            }
        }
    }

    Ok(bridges)
}

/// Count articulation points using iterative Tarjan's algorithm.
fn count_articulation_points(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut disc = vec![0_u32; n];
    let mut low = vec![0_u32; n];
    let mut visited = vec![false; n];
    let mut is_cut = vec![false; n];
    let mut timer = 1_u32;

    for start in 0..n {
        if visited[start] {
            continue;
        }

        visited[start] = true;
        disc[start] = timer;
        low[start] = timer;
        timer += 1;

        // For the root, count children in DFS tree
        let mut root_children = 0_u32;
        let mut stack: Vec<(usize, usize, usize)> = Vec::new();
        stack.push((start, usize::MAX, 0));

        while let Some((v, parent, idx)) = stack.last_mut() {
            let v = *v;
            let parent = *parent;
            let nbrs = graph.neighbors(v as u32)?;

            if *idx < nbrs.len() {
                let u = nbrs[*idx] as usize;
                *idx += 1;

                if !visited[u] {
                    visited[u] = true;
                    disc[u] = timer;
                    low[u] = timer;
                    timer += 1;

                    if v == start {
                        root_children += 1;
                    }

                    stack.push((u, v, 0));
                } else if u != parent {
                    let len = stack.len();
                    let cv = stack[len - 1].0;
                    if disc[u] < low[cv] {
                        low[cv] = disc[u];
                    }
                }
            } else {
                let cv = v;
                stack.pop();
                if let Some(top) = stack.last_mut() {
                    let pv = top.0;
                    if low[cv] < low[pv] {
                        low[pv] = low[cv];
                    }
                    // Non-root: pv is cut vertex if low[cv] >= disc[pv]
                    if pv != start && low[cv] >= disc[pv] {
                        is_cut[pv] = true;
                    }
                }
            }
        }

        // Root is cut vertex if it has >1 children in DFS tree
        if root_children > 1 {
            is_cut[start] = true;
        }
    }

    Ok(is_cut.iter().filter(|&&c| c).count() as u64)
}

/// Find edge count of the largest biconnected component.
fn largest_block_edge_count(graph: &Graph) -> IgraphResult<u64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0);
    }

    let mut disc = vec![0_u32; n];
    let mut low = vec![0_u32; n];
    let mut visited = vec![false; n];
    let mut timer = 1_u32;
    let mut max_block = 0_u64;

    // Edge stack for biconnected component decomposition
    let mut edge_stack: Vec<(usize, usize)> = Vec::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }

        visited[start] = true;
        disc[start] = timer;
        low[start] = timer;
        timer += 1;

        let mut stack: Vec<(usize, usize, usize)> = Vec::new();
        stack.push((start, usize::MAX, 0));

        while let Some((v, parent, idx)) = stack.last_mut() {
            let v = *v;
            let parent = *parent;
            let nbrs = graph.neighbors(v as u32)?;

            if *idx < nbrs.len() {
                let u = nbrs[*idx] as usize;
                *idx += 1;

                if !visited[u] {
                    visited[u] = true;
                    disc[u] = timer;
                    low[u] = timer;
                    timer += 1;
                    edge_stack.push((v, u));
                    stack.push((u, v, 0));
                } else if u != parent && disc[u] < disc[v] {
                    edge_stack.push((v, u));
                    let len = stack.len();
                    let cv = stack[len - 1].0;
                    if disc[u] < low[cv] {
                        low[cv] = disc[u];
                    }
                }
            } else {
                let cv = v;
                stack.pop();
                if let Some(top) = stack.last_mut() {
                    let pv = top.0;
                    if low[cv] < low[pv] {
                        low[pv] = low[cv];
                    }
                    // Extract biconnected component
                    if low[cv] >= disc[pv] {
                        let mut block_edges = 0_u64;
                        while let Some(&(a, b)) = edge_stack.last() {
                            edge_stack.pop();
                            block_edges += 1;
                            if (a == pv && b == cv) || (a == cv && b == pv) {
                                break;
                            }
                        }
                        if block_edges > max_block {
                            max_block = block_edges;
                        }
                    }
                }
            }
        }
    }

    Ok(max_block)
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

    fn bowtie() -> Graph {
        // Two triangles sharing vertex 2: {0,1,2} and {2,3,4}
        Graph::from_edges(
            &[(0, 1), (1, 2), (0, 2), (2, 3), (3, 4), (2, 4)],
            false,
            Some(5),
        )
        .unwrap()
    }

    // --- bridge_ratio ---

    #[test]
    fn br_empty() {
        assert!(bridge_edge_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_single() {
        assert!(bridge_edge_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_single_edge() {
        // The single edge is a bridge → 1.0
        assert!((bridge_edge_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_path3() {
        // Both edges are bridges → 1.0
        assert!((bridge_edge_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_path4() {
        // All 3 edges are bridges → 1.0
        assert!((bridge_edge_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_k3() {
        // No bridges in a cycle → 0.0
        assert!(bridge_edge_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_k4() {
        assert!(bridge_edge_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_cycle4() {
        assert!(bridge_edge_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_star5() {
        // All 4 edges are bridges → 1.0
        assert!((bridge_edge_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn br_paw() {
        // Edge (2,3) is a bridge, edges in triangle are not → 1/4
        assert!((bridge_edge_ratio(&paw()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn br_bowtie() {
        // No bridges (each edge is in a triangle) → 0.0
        assert!(bridge_edge_ratio(&bowtie()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn br_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = bridge_edge_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- articulation_ratio ---

    #[test]
    fn ar_empty() {
        assert!(articulation_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_single() {
        assert!(articulation_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_single_edge() {
        // n=2, < 3 → 0.0
        assert!(articulation_ratio(&single_edge()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_path3() {
        // Vertex 1 is cut vertex → 1/3
        assert!((articulation_ratio(&path3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn ar_path4() {
        // Vertices 1,2 are cut vertices → 2/4 = 0.5
        assert!((articulation_ratio(&path4()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn ar_k3() {
        // No cut vertices → 0.0
        assert!(articulation_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_k4() {
        assert!(articulation_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_cycle4() {
        assert!(articulation_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn ar_star5() {
        // Center (vertex 0) is cut vertex → 1/5
        assert!((articulation_ratio(&star5()).unwrap() - 1.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn ar_paw() {
        // Vertex 2 is cut vertex (removing it disconnects 3) → 1/4
        assert!((articulation_ratio(&paw()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn ar_bowtie() {
        // Vertex 2 is the only cut vertex → 1/5
        assert!((articulation_ratio(&bowtie()).unwrap() - 1.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn ar_in_01() {
        for g in &[path3(), k3(), k4(), cycle4(), star5(), paw(), bowtie()] {
            let r = articulation_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- biconnected_ratio ---

    #[test]
    fn bcr_empty() {
        assert!(biconnected_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn bcr_single_edge() {
        // Single edge is its own block → 1/1 = 1.0
        assert!((biconnected_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bcr_path3() {
        // Two blocks of 1 edge each → max=1, total=2 → 0.5
        assert!((biconnected_ratio(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn bcr_k3() {
        // Entire graph is one block → 1.0
        assert!((biconnected_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bcr_k4() {
        assert!((biconnected_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bcr_cycle4() {
        assert!((biconnected_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bcr_paw() {
        // Two blocks: triangle {0,1,2} (3 edges) and bridge {2,3} (1 edge)
        // Max = 3, total = 4 → 3/4
        assert!((biconnected_ratio(&paw()).unwrap() - 3.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn bcr_bowtie() {
        // Two blocks: {0,1,2} (3 edges) and {2,3,4} (3 edges) → max=3, total=6 → 0.5
        assert!((biconnected_ratio(&bowtie()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn bcr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = biconnected_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- leaf_ratio ---

    #[test]
    fn lr_empty() {
        assert!(leaf_ratio(&empty()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_single() {
        // degree 0, not a leaf
        assert!(leaf_ratio(&single()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_single_edge() {
        // Both vertices have degree 1 → 1.0
        assert!((leaf_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lr_path3() {
        // Vertices 0,2 have degree 1 → 2/3
        assert!((leaf_ratio(&path3()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn lr_k3() {
        // All degree 2 → 0.0
        assert!(leaf_ratio(&k3()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_k4() {
        assert!(leaf_ratio(&k4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_cycle4() {
        assert!(leaf_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lr_star5() {
        // 4 leaves out of 5 → 4/5
        assert!((leaf_ratio(&star5()).unwrap() - 4.0 / 5.0).abs() < 1e-10);
    }

    #[test]
    fn lr_paw() {
        // Vertex 3 has degree 1 → 1/4
        assert!((leaf_ratio(&paw()).unwrap() - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn lr_in_01() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5(), paw()] {
            let r = leaf_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    // --- cross-consistency ---

    #[test]
    fn tree_all_bridges() {
        // Trees: all edges are bridges
        assert!((bridge_edge_ratio(&path3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((bridge_edge_ratio(&star5()).unwrap() - 1.0).abs() < 1e-10);
        assert!((bridge_edge_ratio(&path4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn biconnected_no_bridges() {
        // Biconnected graphs: no bridges
        assert!(bridge_edge_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(bridge_edge_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(bridge_edge_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn biconnected_no_cut_vertices() {
        assert!(articulation_ratio(&k3()).unwrap().abs() < 1e-10);
        assert!(articulation_ratio(&k4()).unwrap().abs() < 1e-10);
        assert!(articulation_ratio(&cycle4()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn biconnected_full_block() {
        // Biconnected: entire graph is one block → ratio = 1.0
        assert!((biconnected_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((biconnected_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
        assert!((biconnected_ratio(&cycle4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
