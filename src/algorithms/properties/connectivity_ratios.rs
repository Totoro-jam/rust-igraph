//! Connectivity-based ratio indices (ALGO-TR-102).
//!
//! Ratios capturing connectivity structure:
//!
//! - **Component ratio** — number of components / number of vertices
//! - **Largest component fraction** — largest component size / n
//! - **Giant component gap** — (largest - 2nd largest) / n
//! - **Vertex connectivity ratio** — vertex connectivity / (n - 1)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphResult};

/// Compute the component ratio.
///
/// `num_components / n` — fraction of vertices that are component
/// representatives. Equals 1.0 when every vertex is isolated,
/// 1/n when the graph is connected. Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, component_ratio};
///
/// // K_3: 1 component, 3 vertices → 1/3
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((component_ratio(&g).unwrap() - 1.0/3.0).abs() < 1e-10);
/// ```
pub fn component_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let comp_sizes = component_sizes(graph)?;
    Ok(comp_sizes.len() as f64 / n as f64)
}

/// Compute the largest component fraction.
///
/// `max_component_size / n` — how much of the graph is in the
/// largest connected component. Equals 1.0 for connected graphs.
/// Returns 0.0 for empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, largest_component_fraction};
///
/// // K_3: fully connected → 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((largest_component_fraction(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn largest_component_fraction(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let comp_sizes = component_sizes(graph)?;
    let max_size = comp_sizes.iter().copied().max().unwrap_or(0);
    Ok(max_size as f64 / n as f64)
}

/// Compute the giant component gap.
///
/// `(largest - second_largest) / n` — the dominance of the largest
/// component. Close to 1.0 means a single giant component dominates;
/// close to 0 means two similarly-sized components exist.
/// Returns 0.0 for graphs with fewer than 2 components or empty graphs.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, giant_component_gap};
///
/// // K_3: single component → gap = (3 - 0) / 3 = 1.0
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// assert!((giant_component_gap(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn giant_component_gap(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(0.0);
    }

    let mut comp_sizes = component_sizes(graph)?;
    comp_sizes.sort_unstable_by(|a, b| b.cmp(a));

    let largest = comp_sizes[0];
    let second = if comp_sizes.len() > 1 {
        comp_sizes[1]
    } else {
        0
    };

    Ok((largest - second) as f64 / n as f64)
}

/// Compute the vertex connectivity ratio.
///
/// `kappa / (n - 1)` where `kappa` is the vertex connectivity
/// (minimum number of vertices whose removal disconnects the graph).
/// Ranges from 0 (disconnected or tree) to 1 (complete graph).
/// Returns 0.0 for graphs with fewer than 2 vertices.
///
/// For efficiency this uses an approximation: the minimum degree
/// as an upper bound on vertex connectivity, since computing exact
/// vertex connectivity is expensive. For simple undirected graphs,
/// `kappa <= delta` (minimum degree) always holds, and for many
/// graph families equality holds.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, vertex_connectivity_ratio};
///
/// // K_4: min_degree=3, n-1=3 → 3/3 = 1.0
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(0,3),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// assert!((vertex_connectivity_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
/// ```
pub fn vertex_connectivity_ratio(graph: &Graph) -> IgraphResult<f64> {
    let n = graph.vcount() as usize;
    if n < 2 {
        return Ok(0.0);
    }

    let comp_sizes = component_sizes(graph)?;
    if comp_sizes.len() > 1 {
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

fn component_sizes(graph: &Graph) -> IgraphResult<Vec<usize>> {
    let n = graph.vcount() as usize;
    let mut visited = vec![false; n];
    let mut sizes = Vec::new();
    let mut queue = std::collections::VecDeque::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut size = 1_usize;
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            let neighbors = graph.neighbors(v as u32)?;
            for &u in &neighbors {
                let ui = u as usize;
                if !visited[ui] {
                    visited[ui] = true;
                    size += 1;
                    queue.push_back(ui);
                }
            }
        }
        sizes.push(size);
    }

    Ok(sizes)
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

    fn disconnected_2_2() -> Graph {
        Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap()
    }

    fn disconnected_3_1() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(4)).unwrap()
    }

    // --- component_ratio ---

    #[test]
    fn cr_empty() {
        let g = Graph::with_vertices(0);
        assert!(component_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cr_single() {
        let g = Graph::with_vertices(1);
        assert!((component_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_isolated() {
        let g = Graph::with_vertices(5);
        assert!((component_ratio(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cr_k3() {
        assert!((component_ratio(&k3()).unwrap() - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cr_k4() {
        assert!((component_ratio(&k4()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn cr_disconnected_2_2() {
        // 2 components, 4 vertices → 0.5
        assert!((component_ratio(&disconnected_2_2()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cr_disconnected_3_1() {
        // path(0-1-2) + isolated(3) → 2 components, 4 vertices → 0.5
        assert!((component_ratio(&disconnected_3_1()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cr_single_edge() {
        assert!((component_ratio(&single_edge()).unwrap() - 0.5).abs() < 1e-10);
    }

    // --- largest_component_fraction ---

    #[test]
    fn lcf_empty() {
        let g = Graph::with_vertices(0);
        assert!(largest_component_fraction(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn lcf_single() {
        let g = Graph::with_vertices(1);
        assert!((largest_component_fraction(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lcf_isolated() {
        // 5 isolated vertices → largest = 1 → 1/5 = 0.2
        let g = Graph::with_vertices(5);
        assert!((largest_component_fraction(&g).unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn lcf_k3() {
        assert!((largest_component_fraction(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn lcf_disconnected_2_2() {
        // Two K_2's → largest = 2 → 2/4 = 0.5
        assert!((largest_component_fraction(&disconnected_2_2()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn lcf_disconnected_3_1() {
        // path(3) + isolated(1) → largest = 3 → 3/4 = 0.75
        assert!((largest_component_fraction(&disconnected_3_1()).unwrap() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn lcf_star5() {
        assert!((largest_component_fraction(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- giant_component_gap ---

    #[test]
    fn gcg_empty() {
        let g = Graph::with_vertices(0);
        assert!(giant_component_gap(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gcg_single() {
        let g = Graph::with_vertices(1);
        assert!((giant_component_gap(&g).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gcg_k3() {
        // Single component → gap = (3 - 0) / 3 = 1.0
        assert!((giant_component_gap(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gcg_disconnected_2_2() {
        // Two equal components → gap = (2 - 2) / 4 = 0.0
        assert!(giant_component_gap(&disconnected_2_2()).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gcg_disconnected_3_1() {
        // 3 + 1 → gap = (3 - 1) / 4 = 0.5
        assert!((giant_component_gap(&disconnected_3_1()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn gcg_isolated() {
        // 5 isolated → all size 1 → gap = (1 - 1) / 5 = 0.0
        let g = Graph::with_vertices(5);
        assert!(giant_component_gap(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn gcg_star5() {
        // Single component → gap = (5 - 0) / 5 = 1.0
        assert!((giant_component_gap(&star5()).unwrap() - 1.0).abs() < 1e-10);
    }

    // --- vertex_connectivity_ratio ---

    #[test]
    fn vcr_empty() {
        let g = Graph::with_vertices(0);
        assert!(vertex_connectivity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_single() {
        let g = Graph::with_vertices(1);
        assert!(vertex_connectivity_ratio(&g).unwrap().abs() < 1e-10);
    }

    #[test]
    fn vcr_single_edge() {
        // min_deg=1, n-1=1 → 1.0
        assert!((vertex_connectivity_ratio(&single_edge()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_k3() {
        // min_deg=2, n-1=2 → 1.0
        assert!((vertex_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_k4() {
        // min_deg=3, n-1=3 → 1.0
        assert!((vertex_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_cycle4() {
        // min_deg=2, n-1=3 → 2/3
        assert!((vertex_connectivity_ratio(&cycle4()).unwrap() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn vcr_star5() {
        // min_deg=1, n-1=4 → 0.25
        assert!((vertex_connectivity_ratio(&star5()).unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn vcr_path3() {
        // min_deg=1, n-1=2 → 0.5
        assert!((vertex_connectivity_ratio(&path3()).unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn vcr_disconnected() {
        // Disconnected → 0.0
        assert!(
            vertex_connectivity_ratio(&disconnected_2_2())
                .unwrap()
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn vcr_isolated() {
        // Isolated vertices → disconnected → 0.0
        let g = Graph::with_vertices(5);
        assert!(vertex_connectivity_ratio(&g).unwrap().abs() < 1e-10);
    }

    // --- cross-consistency ---

    #[test]
    fn cr_in_01() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            disconnected_2_2(),
        ] {
            let r = component_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn lcf_in_01() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            disconnected_2_2(),
        ] {
            let r = largest_component_fraction(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn gcg_in_01() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            disconnected_2_2(),
        ] {
            let r = giant_component_gap(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn vcr_in_01() {
        for g in &[
            single_edge(),
            path3(),
            k3(),
            k4(),
            cycle4(),
            star5(),
            disconnected_2_2(),
        ] {
            let r = vertex_connectivity_ratio(g).unwrap();
            assert!(r >= -1e-10);
            assert!(r <= 1.0 + 1e-10);
        }
    }

    #[test]
    fn connected_graphs_lcf_one() {
        for g in &[single_edge(), path3(), k3(), k4(), cycle4(), star5()] {
            assert!((largest_component_fraction(g).unwrap() - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn complete_graphs_vcr_one() {
        assert!((vertex_connectivity_ratio(&k3()).unwrap() - 1.0).abs() < 1e-10);
        assert!((vertex_connectivity_ratio(&k4()).unwrap() - 1.0).abs() < 1e-10);
    }
}
