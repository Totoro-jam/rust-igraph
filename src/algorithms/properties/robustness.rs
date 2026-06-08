//! Graph robustness and resilience metrics (ALGO-TR-027).
//!
//! Measures how resilient a graph is to node/edge removal, building on
//! the existing flow-based connectivity functions.
//!
//! - **Vertex resilience**: `κ(G) / n` — fraction of vertices whose
//!   removal is needed to disconnect the graph.
//! - **Edge resilience**: `λ(G) / m` — fraction of edges whose removal
//!   is needed to disconnect the graph.
//! - **Toughness**: `min_{S} |S| / ω(G-S)` where `ω` is the number
//!   of connected components after removing vertex set `S`. Measures
//!   how many vertices must be removed per component created.
//! - **Integrity**: `min_{S} (|S| + m(G-S))` where `m(G-S)` is the
//!   order of the largest component after removing `S`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};
use std::collections::VecDeque;

/// Compute vertex resilience `κ(G) / n`.
///
/// The fraction of vertices that must be removed to disconnect the
/// graph. Higher values indicate more robust graphs. Uses the
/// flow-based `vertex_connectivity` from the flow module internally.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_resilience};
///
/// // K_4: κ = 3, n = 4, resilience = 0.75
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let r = vertex_resilience(&g).unwrap();
/// assert!((r - 0.75).abs() < 0.01);
/// ```
pub fn vertex_resilience(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }
    let kappa = crate::algorithms::flow::vertex_connectivity::vertex_connectivity(graph, true)?;
    Ok(kappa.max(0) as f64 / n as f64)
}

/// Compute edge resilience `λ(G) / m`.
///
/// The fraction of edges that must be removed to disconnect the graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_resilience};
///
/// // Path 0-1-2: λ = 1, m = 2, resilience = 0.5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let r = edge_resilience(&g).unwrap();
/// assert!((r - 0.5).abs() < 0.01);
/// ```
pub fn edge_resilience(graph: &Graph) -> IgraphResult<f64> {
    let m = graph.ecount();
    if m == 0 {
        return Ok(0.0);
    }
    let lambda = crate::algorithms::flow::edge_connectivity::edge_connectivity(graph, true)?;
    Ok(lambda.max(0) as f64 / m as f64)
}

/// Count connected components after removing a set of vertices.
fn count_components_after_removal(graph: &Graph, removed: &[bool]) -> (usize, usize) {
    let n = graph.vcount() as usize;
    let mut visited = vec![false; n];
    let mut num_components = 0_usize;
    let mut largest_component = 0_usize;

    for start in 0..n {
        if visited[start] || removed[start] {
            continue;
        }
        num_components += 1;
        let mut size = 0_usize;
        let mut queue = VecDeque::new();
        queue.push_back(start as u32);
        visited[start] = true;

        while let Some(u) = queue.pop_front() {
            size += 1;
            if let Ok(nbrs) = graph.neighbors(u) {
                for &v in &nbrs {
                    let vi = v as usize;
                    if !visited[vi] && !removed[vi] {
                        visited[vi] = true;
                        queue.push_back(v);
                    }
                }
            }
        }

        if size > largest_component {
            largest_component = size;
        }
    }

    (num_components, largest_component)
}

/// Compute the toughness of a graph.
///
/// `τ(G) = min_{S ⊂ V, ω(G-S)>1} |S| / ω(G-S)`
///
/// where `ω(G-S)` is the number of connected components of `G` after
/// removing vertex set `S`. A graph is `t`-tough if `τ(G) ≥ t`.
///
/// Returns `f64::INFINITY` for complete graphs (no vertex set
/// disconnects them into multiple components). Returns `0.0` for
/// disconnected graphs.
///
/// Brute-force — suitable for graphs with ≤ ~20 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_toughness};
///
/// // Path 0-1-2: removing {1} → 2 components → τ = 1/2
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let t = graph_toughness(&g).unwrap();
/// assert!((t - 0.5).abs() < 0.01);
/// ```
pub fn graph_toughness(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "graph_toughness is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n <= 1 {
        return Ok(0.0);
    }

    let (initial_comp, _) = count_components_after_removal(graph, &vec![false; n]);
    if initial_comp > 1 {
        return Ok(0.0);
    }

    let mut min_toughness = f64::INFINITY;
    let mut removed = vec![false; n];

    toughness_search(graph, n, 0, 0, &mut removed, &mut min_toughness);

    Ok(min_toughness)
}

fn toughness_search(
    graph: &Graph,
    n: usize,
    start: usize,
    removed_count: usize,
    removed: &mut Vec<bool>,
    min_toughness: &mut f64,
) {
    if removed_count > 0 {
        let (comp, _) = count_components_after_removal(graph, removed);
        if comp > 1 {
            let t = removed_count as f64 / comp as f64;
            if t < *min_toughness {
                *min_toughness = t;
            }
        }
    }

    let active_remaining = (start..n).filter(|&i| !removed[i]).count();
    if active_remaining <= 1 {
        return;
    }

    for i in start..n {
        if removed[i] {
            continue;
        }
        removed[i] = true;
        toughness_search(graph, n, i + 1, removed_count + 1, removed, min_toughness);
        removed[i] = false;
    }
}

/// Compute the integrity of a graph.
///
/// `I(G) = min_{S ⊂ V} (|S| + m(G-S))`
///
/// where `m(G-S)` is the order (vertex count) of the largest connected
/// component of `G - S`. Lower integrity means the graph is more
/// vulnerable.
///
/// Brute-force — suitable for graphs with ≤ ~20 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, graph_integrity};
///
/// // K_3: optimal is S=∅ → I = 0 + 3 = 3;
/// //       or S={0} → I = 1 + 2 = 3. Min = 3.
/// // Actually for S={0,1} → I = 2 + 1 = 3. Always 3 for K_3.
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let i = graph_integrity(&g).unwrap();
/// assert!((i - 3.0).abs() < 0.01);
/// ```
pub fn graph_integrity(graph: &Graph) -> IgraphResult<f64> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "graph_integrity is defined for undirected graphs only".into(),
        ));
    }
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut min_integrity = n as f64;
    let mut removed = vec![false; n];

    integrity_search(graph, n, 0, 0, &mut removed, &mut min_integrity);

    Ok(min_integrity)
}

fn integrity_search(
    graph: &Graph,
    n: usize,
    start: usize,
    removed_count: usize,
    removed: &mut Vec<bool>,
    min_integrity: &mut f64,
) {
    let (_, largest) = count_components_after_removal(graph, removed);
    let val = removed_count as f64 + largest as f64;
    if val < *min_integrity {
        *min_integrity = val;
    }

    if removed_count as f64 >= *min_integrity {
        return;
    }

    for i in start..n {
        if removed[i] {
            continue;
        }
        removed[i] = true;
        integrity_search(graph, n, i + 1, removed_count + 1, removed, min_integrity);
        removed[i] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn star4() -> Graph {
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3)], false, Some(4)).unwrap()
    }

    // --- vertex_resilience ---

    #[test]
    fn vr_k4() {
        let g = k4();
        let r = vertex_resilience(&g).unwrap();
        assert!((r - 0.75).abs() < 0.01);
    }

    #[test]
    fn vr_path() {
        let g = path4();
        let r = vertex_resilience(&g).unwrap();
        assert!((r - 0.25).abs() < 0.01);
    }

    #[test]
    fn vr_empty() {
        let g = Graph::with_vertices(0);
        let r = vertex_resilience(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn vr_single() {
        let g = Graph::with_vertices(1);
        let r = vertex_resilience(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    // --- edge_resilience ---

    #[test]
    fn er_path3() {
        let g = path3();
        let r = edge_resilience(&g).unwrap();
        assert!((r - 0.5).abs() < 0.01);
    }

    #[test]
    fn er_k3() {
        let g = k3();
        let r = edge_resilience(&g).unwrap();
        // λ = 2, m = 3, resilience = 2/3
        assert!((r - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn er_empty() {
        let g = Graph::with_vertices(1);
        let r = edge_resilience(&g).unwrap();
        assert!(r.abs() < 1e-10);
    }

    // --- graph_toughness ---

    #[test]
    fn gt_path3() {
        let g = path3();
        let t = graph_toughness(&g).unwrap();
        // Removing {1} → 2 comp → τ = 1/2
        assert!((t - 0.5).abs() < 0.01);
    }

    #[test]
    fn gt_k3() {
        let g = k3();
        let t = graph_toughness(&g).unwrap();
        // K_3: must remove 2 to disconnect → 2 leaves 1 vertex (1 comp)
        // No S creates > 1 component while |S| < n-1
        // Remove {0}: K_2 remains (1 comp). Remove {0,1}: 1 vertex (1 comp).
        // Never > 1 component → toughness = ∞
        assert!(t.is_infinite());
    }

    #[test]
    fn gt_k4() {
        let g = k4();
        let t = graph_toughness(&g).unwrap();
        assert!(t.is_infinite());
    }

    #[test]
    fn gt_cycle4() {
        let g = cycle4();
        let t = graph_toughness(&g).unwrap();
        // Removing 2 non-adjacent vertices → 2 components → τ = 2/2 = 1
        // Removing 2 adjacent vertices → 2 isolated + path of 0 → 1 comp
        // Wait: C_4 = 0-1-2-3-0. Remove {0,2}: 1,3 isolated → 2 comp → 2/2=1
        // Remove {1}: 0-3-2 still connected → 1 comp
        // So minimum is 1.0
        assert!((t - 1.0).abs() < 0.01);
    }

    #[test]
    fn gt_star4() {
        let g = star4();
        let t = graph_toughness(&g).unwrap();
        // Remove {0} (center) → 3 isolated vertices → τ = 1/3
        assert!((t - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn gt_disconnected() {
        let g = Graph::with_vertices(3);
        let t = graph_toughness(&g).unwrap();
        assert!(t.abs() < 1e-10);
    }

    #[test]
    fn gt_single() {
        let g = Graph::with_vertices(1);
        let t = graph_toughness(&g).unwrap();
        assert!(t.abs() < 1e-10);
    }

    #[test]
    fn gt_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(graph_toughness(&g).is_err());
    }

    // --- graph_integrity ---

    #[test]
    fn gi_k3() {
        let g = k3();
        let i = graph_integrity(&g).unwrap();
        // S=∅: 0+3=3, S={v}: 1+2=3, S={v,w}: 2+1=3 → I=3
        assert!((i - 3.0).abs() < 0.01);
    }

    #[test]
    fn gi_path3() {
        let g = path3();
        let i = graph_integrity(&g).unwrap();
        // S=∅: 0+3=3, S={1}: 1+1=2, S={0}: 1+2=3 → I=2
        assert!((i - 2.0).abs() < 0.01);
    }

    #[test]
    fn gi_path4() {
        let g = path4();
        let i = graph_integrity(&g).unwrap();
        // S={1}: 1+2=3, S={1,2}: 2+1=3, S={2}: 1+2=3 → I=3
        // Actually: S={1,2}: leaves {0} and {3} → largest=1 → 2+1=3
        // S={1}: leaves {0} and {2,3} → largest=2 → 1+2=3
        assert!((i - 3.0).abs() < 0.01);
    }

    #[test]
    fn gi_star4() {
        let g = star4();
        let i = graph_integrity(&g).unwrap();
        // S={0}: 1 + 1 = 2 (three isolated vertices, largest = 1)
        assert!((i - 2.0).abs() < 0.01);
    }

    #[test]
    fn gi_empty() {
        let g = Graph::with_vertices(0);
        let i = graph_integrity(&g).unwrap();
        assert!(i.abs() < 1e-10);
    }

    #[test]
    fn gi_single() {
        let g = Graph::with_vertices(1);
        let i = graph_integrity(&g).unwrap();
        assert!((i - 1.0).abs() < 0.01);
    }

    #[test]
    fn gi_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(graph_integrity(&g).is_err());
    }

    // --- cross-consistency ---

    #[test]
    fn resilience_bounded() {
        let g = cycle4();
        let vr = vertex_resilience(&g).unwrap();
        let er = edge_resilience(&g).unwrap();
        assert!((0.0..=1.0).contains(&vr));
        assert!((0.0..=1.0).contains(&er));
    }

    #[test]
    fn complete_graph_is_infinitely_tough() {
        for n in 3_u32..=5 {
            let mut edges = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    edges.push((u, v));
                }
            }
            let g = Graph::from_edges(&edges, false, Some(n)).unwrap();
            let t = graph_toughness(&g).unwrap();
            assert!(t.is_infinite(), "K_{n} should have infinite toughness");
        }
    }
}
