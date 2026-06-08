//! Graph homophily metrics (ALGO-TR-015).
//!
//! Measures the tendency of edges to connect vertices with the same label.
//! These metrics are central to understanding when message-passing GNNs
//! will succeed (high homophily) vs. struggle (low homophily / heterophily).
//!
//! Implements three standard variants from the GNN literature:
//! - Edge homophily ratio (Zhu et al., 2020)
//! - Node homophily ratio (Pei et al., 2020)
//! - Class homophily ratio (Lim et al., 2021)

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Edge homophily ratio: fraction of edges connecting same-label vertices.
///
/// `h_edge = |{(u,v) ∈ E : y_u = y_v}| / |E|`
///
/// Range: [0, 1]. Higher values indicate stronger homophily.
/// This is the simplest and most common homophily metric in GNN papers.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_homophily};
///
/// // Triangle with uniform labels: all edges connect same label
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let h = edge_homophily(&g, &[0, 0, 0]).unwrap();
/// assert!((h - 1.0).abs() < 1e-10);
/// ```
pub fn edge_homophily(graph: &Graph, labels: &[u32]) -> IgraphResult<f64> {
    validate_labels(graph, labels)?;

    let ne = graph.ecount();
    if ne == 0 {
        return Ok(0.0);
    }

    let mut same_count = 0usize;
    for (u, v) in graph.edges() {
        if labels[u as usize] == labels[v as usize] {
            same_count += 1;
        }
    }

    Ok(same_count as f64 / ne as f64)
}

/// Node homophily ratio: average proportion of same-label neighbors.
///
/// `h_node = (1/|V|) Σ_v |{u ∈ N(v) : y_u = y_v}| / |N(v)|`
///
/// Isolated vertices (degree 0) are excluded from the average.
/// Range: [0, 1]. This per-node definition better captures local structure
/// than the edge-level metric.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, node_homophily};
///
/// // Path 0-1-2 with labels [0, 0, 1]:
/// // Node 0: 1 neighbor (1, same label) → 1.0
/// // Node 1: 2 neighbors (0=same, 2=diff) → 0.5
/// // Node 2: 1 neighbor (1, diff label) → 0.0
/// // Average = (1.0 + 0.5 + 0.0) / 3 = 0.5
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let h = node_homophily(&g, &[0, 0, 1]).unwrap();
/// assert!((h - 0.5).abs() < 1e-10);
/// ```
pub fn node_homophily(graph: &Graph, labels: &[u32]) -> IgraphResult<f64> {
    validate_labels(graph, labels)?;

    let nv = graph.vcount() as usize;
    let mut sum = 0.0;
    let mut count = 0usize;

    for v in 0..nv {
        let neighbors = graph.neighbors(v as VertexId)?;
        let deg = neighbors.len();
        if deg == 0 {
            continue;
        }
        let same = neighbors
            .iter()
            .filter(|&&u| labels[u as usize] == labels[v])
            .count();
        sum += same as f64 / deg as f64;
        count += 1;
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(sum / count as f64)
}

/// Class-balanced homophily ratio (adjusted for class imbalance).
///
/// `h_class = (1/C) Σ_k [ h_k - |C_k|/|V| ] / (1 - |C_k|/|V|)`
///
/// where `h_k` is the proportion of edges from class-k vertices that
/// connect to other class-k vertices, and `|C_k|/|V|` is the class
/// proportion (baseline under random connectivity).
///
/// Range: approximately [-1, 1]. Values near 0 indicate behavior
/// consistent with random label assignment. Negative values indicate
/// heterophily beyond random expectation.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, class_homophily};
///
/// // Complete bipartite K_{2,2} with labels [0,0,1,1]
/// // All edges cross classes → strong heterophily
/// let g = Graph::from_edges(&[(0,2),(0,3),(1,2),(1,3)], false, Some(4)).unwrap();
/// let h = class_homophily(&g, &[0, 0, 1, 1]).unwrap();
/// assert!(h < 0.0);
/// ```
pub fn class_homophily(graph: &Graph, labels: &[u32]) -> IgraphResult<f64> {
    validate_labels(graph, labels)?;

    let nv = graph.vcount() as usize;
    if nv == 0 {
        return Ok(0.0);
    }

    let num_classes = labels.iter().max().map_or(0, |&m| m + 1) as usize;
    if num_classes == 0 {
        return Ok(0.0);
    }

    // Count class sizes
    let mut class_size = vec![0usize; num_classes];
    for &l in labels {
        class_size[l as usize] += 1;
    }

    // For each class k, count edges from class-k vertices to same class
    let mut intra_edges = vec![0usize; num_classes];
    let mut total_from_class = vec![0usize; num_classes];

    for v in 0..nv {
        let label_v = labels[v] as usize;
        let neighbors = graph.neighbors(v as VertexId)?;
        total_from_class[label_v] += neighbors.len();
        for &u in &neighbors {
            if labels[u as usize] == labels[v] {
                intra_edges[label_v] += 1;
            }
        }
    }

    // For undirected graphs, each edge is counted twice in the above
    // but the ratio remains the same since both numerator and denominator
    // are doubled.

    let mut sum = 0.0;
    let mut valid_classes = 0usize;

    for k in 0..num_classes {
        let proportion = class_size[k] as f64 / nv as f64;
        if (1.0 - proportion).abs() < 1e-15 || total_from_class[k] == 0 {
            continue;
        }
        let h_k = intra_edges[k] as f64 / total_from_class[k] as f64;
        sum += (h_k - proportion) / (1.0 - proportion);
        valid_classes += 1;
    }

    if valid_classes == 0 {
        return Ok(0.0);
    }

    Ok(sum / valid_classes as f64)
}

/// Heterophily ratio: `1 - edge_homophily`.
///
/// Convenience function returning the complement of edge homophily.
/// Higher values indicate edges predominantly connect different-label vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, edge_heterophily};
///
/// let g = Graph::from_edges(&[(0,1),(1,2)], false, Some(3)).unwrap();
/// let h = edge_heterophily(&g, &[0, 1, 0]).unwrap();
/// // Both edges cross labels → heterophily = 1.0
/// assert!((h - 1.0).abs() < 1e-10);
/// ```
pub fn edge_heterophily(graph: &Graph, labels: &[u32]) -> IgraphResult<f64> {
    Ok(1.0 - edge_homophily(graph, labels)?)
}

// --- Internal helpers ---

fn validate_labels(graph: &Graph, labels: &[u32]) -> IgraphResult<()> {
    let nv = graph.vcount() as usize;
    if labels.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "labels length {} does not match vcount {}",
            labels.len(),
            nv
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn bipartite_k22() -> Graph {
        Graph::from_edges(&[(0, 2), (0, 3), (1, 2), (1, 3)], false, Some(4)).unwrap()
    }

    // --- edge_homophily tests ---

    #[test]
    fn edge_homo_all_same() {
        let g = triangle();
        let h = edge_homophily(&g, &[0, 0, 0]).unwrap();
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn edge_homo_all_different() {
        let g = triangle();
        let h = edge_homophily(&g, &[0, 1, 2]).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn edge_homo_mixed() {
        let g = path3();
        // Labels [0, 0, 1]: edge(0,1)=same, edge(1,2)=diff → 0.5
        let h = edge_homophily(&g, &[0, 0, 1]).unwrap();
        assert!((h - 0.5).abs() < 1e-10);
    }

    #[test]
    fn edge_homo_empty_graph() {
        let g = Graph::with_vertices(3);
        let h = edge_homophily(&g, &[0, 1, 2]).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn edge_homo_invalid_labels() {
        let g = triangle();
        assert!(edge_homophily(&g, &[0, 1]).is_err());
    }

    // --- node_homophily tests ---

    #[test]
    fn node_homo_path() {
        let g = path3();
        // Labels [0, 0, 1]:
        // Node 0: N={1}, same=1 → 1.0
        // Node 1: N={0,2}, same=1 → 0.5
        // Node 2: N={1}, same=0 → 0.0
        // Average = (1.0 + 0.5 + 0.0) / 3 = 0.5
        let h = node_homophily(&g, &[0, 0, 1]).unwrap();
        assert!((h - 0.5).abs() < 1e-10);
    }

    #[test]
    fn node_homo_all_same() {
        let g = triangle();
        let h = node_homophily(&g, &[0, 0, 0]).unwrap();
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn node_homo_all_different() {
        let g = triangle();
        let h = node_homophily(&g, &[0, 1, 2]).unwrap();
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn node_homo_isolated() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(3)).unwrap();
        // Vertex 2 is isolated, excluded from average
        let h = node_homophily(&g, &[0, 0, 1]).unwrap();
        // Node 0: N={1}, same=1 → 1.0
        // Node 1: N={0}, same=1 → 1.0
        // Average = (1.0 + 1.0) / 2 = 1.0
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn node_homo_empty() {
        let g = Graph::with_vertices(3);
        let h = node_homophily(&g, &[0, 1, 2]).unwrap();
        assert!(h.abs() < 1e-10);
    }

    // --- class_homophily tests ---

    #[test]
    fn class_homo_perfect_homophily() {
        // Two components, each with same label
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let h = class_homophily(&g, &[0, 0, 1, 1]).unwrap();
        // Each class has h_k=1.0, proportion=0.5
        // adjusted = (1.0 - 0.5) / (1.0 - 0.5) = 1.0
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn class_homo_perfect_heterophily() {
        let g = bipartite_k22();
        let h = class_homophily(&g, &[0, 0, 1, 1]).unwrap();
        // Each class has h_k=0.0, proportion=0.5
        // adjusted = (0.0 - 0.5) / (1.0 - 0.5) = -1.0
        assert!((h - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn class_homo_random_like() {
        // Diamond graph where labels match class proportions
        let g = Graph::from_edges(
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            false,
            Some(4),
        )
        .unwrap();
        // K4 with labels [0,0,1,1]: 6 edges, intra={0-1, 2-3}=2, cross=4
        let h = class_homophily(&g, &[0, 0, 1, 1]).unwrap();
        // h_k = 2/6 = 1/3 for each class (each has 6 total edges from its vertices,
        // actually: vertex 0 has 3 edges, vertex 1 has 3 edges → total_from_class[0]=6,
        // intra[0]=2 (0→1 and 1→0) → h_0 = 2/6 = 1/3
        // proportion = 0.5, adjusted = (1/3 - 0.5) / (1 - 0.5) = -1/3
        assert!((h - (-1.0 / 3.0)).abs() < 1e-10);
    }

    #[test]
    fn class_homo_single_class() {
        let g = triangle();
        let h = class_homophily(&g, &[0, 0, 0]).unwrap();
        // Only one class with proportion=1.0 → skipped → 0.0
        assert!(h.abs() < 1e-10);
    }

    #[test]
    fn class_homo_empty() {
        let g = Graph::with_vertices(0);
        let h = class_homophily(&g, &[]).unwrap();
        assert!(h.abs() < 1e-10);
    }

    // --- edge_heterophily tests ---

    #[test]
    fn heterophily_complement() {
        let g = path3();
        let homo = edge_homophily(&g, &[0, 1, 0]).unwrap();
        let hetero = edge_heterophily(&g, &[0, 1, 0]).unwrap();
        assert!((homo + hetero - 1.0).abs() < 1e-10);
    }

    #[test]
    fn heterophily_all_cross() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap();
        let h = edge_heterophily(&g, &[0, 1, 0]).unwrap();
        // Both edges cross: heterophily = 1.0
        assert!((h - 1.0).abs() < 1e-10);
    }
}
