//! Link prediction heuristic scores (ALGO-TR-013).
//!
//! Classical topology-based link prediction features that score vertex pairs
//! by their structural proximity. These are widely used as baselines in link
//! prediction benchmarks and as edge features in GNN models.
//!
//! Implements:
//! - Common Neighbors (CN)
//! - Adamic-Adar Index (AA)
//! - Resource Allocation Index (RA)
//! - Preferential Attachment (PA)
//! - Jaccard Coefficient

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute Common Neighbors score for given vertex pairs.
///
/// For each pair `(u, v)`, returns `|N(u) ∩ N(v)|` — the number of
/// shared neighbors.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, link_pred_common_neighbors};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let scores = link_pred_common_neighbors(&g, &[(0, 3)]).unwrap();
/// // Vertices 0 and 3 share neighbors 1 and 2
/// assert_eq!(scores[0], 2.0);
/// ```
pub fn link_pred_common_neighbors(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    validate_pairs(graph, pairs)?;
    let mut scores = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        let nu = neighbor_set(graph, u)?;
        let nv = neighbor_set(graph, v)?;
        let cn = count_intersection(&nu, &nv);
        scores.push(cn as f64);
    }

    Ok(scores)
}

/// Compute Adamic-Adar Index for given vertex pairs.
///
/// For each pair `(u, v)`, returns `Σ_{w ∈ N(u) ∩ N(v)} 1/log(deg(w))`.
/// Weights common neighbors by the inverse log of their degree, giving
/// more importance to less-connected shared neighbors.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, link_pred_adamic_adar};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let scores = link_pred_adamic_adar(&g, &[(0, 3)]).unwrap();
/// // Common neighbors of 0,3 are {1,2}; deg(1)=3, deg(2)=3
/// // AA = 1/log(3) + 1/log(3)
/// let expected = 2.0 / 3.0_f64.ln();
/// assert!((scores[0] - expected).abs() < 1e-10);
/// ```
pub fn link_pred_adamic_adar(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    validate_pairs(graph, pairs)?;
    let nv = graph.vcount() as usize;

    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    let mut scores = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        let nu = neighbor_set(graph, u)?;
        let nv_set = neighbor_set(graph, v)?;
        let mut score = 0.0;
        for &w in &nu {
            if nv_set.contains(&w) {
                let deg = degrees[w as usize];
                if deg > 1 {
                    score += 1.0 / (deg as f64).ln();
                }
            }
        }
        scores.push(score);
    }

    Ok(scores)
}

/// Compute Resource Allocation Index for given vertex pairs.
///
/// For each pair `(u, v)`, returns `Σ_{w ∈ N(u) ∩ N(v)} 1/deg(w)`.
/// Similar to Adamic-Adar but uses inverse degree instead of inverse
/// log-degree, penalizing high-degree common neighbors more strongly.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, link_pred_resource_allocation};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let scores = link_pred_resource_allocation(&g, &[(0, 3)]).unwrap();
/// // Common neighbors of 0,3 are {1,2}; deg(1)=3, deg(2)=3
/// // RA = 1/3 + 1/3 = 2/3
/// assert!((scores[0] - 2.0 / 3.0).abs() < 1e-10);
/// ```
pub fn link_pred_resource_allocation(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    validate_pairs(graph, pairs)?;
    let nv = graph.vcount() as usize;

    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    let mut scores = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        let nu = neighbor_set(graph, u)?;
        let nv_set = neighbor_set(graph, v)?;
        let mut score = 0.0;
        for &w in &nu {
            if nv_set.contains(&w) {
                let deg = degrees[w as usize];
                if deg > 0 {
                    score += 1.0 / deg as f64;
                }
            }
        }
        scores.push(score);
    }

    Ok(scores)
}

/// Compute Preferential Attachment score for given vertex pairs.
///
/// For each pair `(u, v)`, returns `deg(u) × deg(v)`. Based on the
/// preferential attachment model where high-degree nodes are more
/// likely to form new connections.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, link_pred_preferential_attachment};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let scores = link_pred_preferential_attachment(&g, &[(0, 3)]).unwrap();
/// // deg(0)=2, deg(3)=2 → PA = 4
/// assert_eq!(scores[0], 4.0);
/// ```
pub fn link_pred_preferential_attachment(
    graph: &Graph,
    pairs: &[(VertexId, VertexId)],
) -> IgraphResult<Vec<f64>> {
    validate_pairs(graph, pairs)?;

    let mut scores = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        let du = graph.degree(u)?;
        let dv = graph.degree(v)?;
        scores.push((du * dv) as f64);
    }

    Ok(scores)
}

/// Compute Jaccard Coefficient for given vertex pairs.
///
/// For each pair `(u, v)`, returns `|N(u) ∩ N(v)| / |N(u) ∪ N(v)|`.
/// Returns 0.0 if both vertices have no neighbors.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, link_pred_jaccard};
///
/// let g = Graph::from_edges(
///     &[(0,1),(0,2),(1,2),(1,3),(2,3)], false, Some(4)
/// ).unwrap();
/// let scores = link_pred_jaccard(&g, &[(0, 3)]).unwrap();
/// // N(0)={1,2}, N(3)={1,2}. Intersection={1,2}, Union={1,2}
/// // Jaccard = 2/2 = 1.0
/// assert!((scores[0] - 1.0).abs() < 1e-10);
/// ```
pub fn link_pred_jaccard(graph: &Graph, pairs: &[(VertexId, VertexId)]) -> IgraphResult<Vec<f64>> {
    validate_pairs(graph, pairs)?;
    let mut scores = Vec::with_capacity(pairs.len());

    for &(u, v) in pairs {
        let nu = neighbor_set(graph, u)?;
        let nv = neighbor_set(graph, v)?;
        let intersection = count_intersection(&nu, &nv);
        let union_size = nu.len() + nv.len() - intersection;
        let score = if union_size == 0 {
            0.0
        } else {
            intersection as f64 / union_size as f64
        };
        scores.push(score);
    }

    Ok(scores)
}

// --- Internal helpers ---

fn validate_pairs(graph: &Graph, pairs: &[(VertexId, VertexId)]) -> IgraphResult<()> {
    let n = graph.vcount();
    for &(u, v) in pairs {
        if u >= n {
            return Err(IgraphError::VertexOutOfRange { id: u, n });
        }
        if v >= n {
            return Err(IgraphError::VertexOutOfRange { id: v, n });
        }
    }
    Ok(())
}

fn neighbor_set(graph: &Graph, v: VertexId) -> IgraphResult<Vec<VertexId>> {
    graph.neighbors(v)
}

fn count_intersection(a: &[VertexId], b: &[VertexId]) -> usize {
    // For small neighbor lists, linear scan is fine
    let mut count = 0;
    for &x in a {
        if b.contains(&x) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diamond() -> Graph {
        // 0-1, 0-2, 1-2, 1-3, 2-3
        Graph::from_edges(&[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], false, Some(4)).unwrap()
    }

    fn star5() -> Graph {
        // 0 connected to 1,2,3,4
        Graph::from_edges(&[(0, 1), (0, 2), (0, 3), (0, 4)], false, Some(5)).unwrap()
    }

    // --- common neighbors ---

    #[test]
    fn cn_basic() {
        let g = diamond();
        let scores = link_pred_common_neighbors(&g, &[(0, 3)]).unwrap();
        assert!((scores[0] - 2.0).abs() < 1e-10); // {1, 2}
    }

    #[test]
    fn cn_adjacent_pair() {
        let g = diamond();
        let scores = link_pred_common_neighbors(&g, &[(0, 1)]).unwrap();
        assert!((scores[0] - 1.0).abs() < 1e-10); // {2}
    }

    #[test]
    fn cn_no_common() {
        let g = star5();
        let scores = link_pred_common_neighbors(&g, &[(1, 2)]).unwrap();
        assert!((scores[0] - 1.0).abs() < 1e-10); // {0} is the common neighbor
    }

    #[test]
    fn cn_disconnected_pair() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let scores = link_pred_common_neighbors(&g, &[(0, 2)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }

    #[test]
    fn cn_multiple_pairs() {
        let g = diamond();
        let scores = link_pred_common_neighbors(&g, &[(0, 3), (0, 1), (1, 3)]).unwrap();
        assert_eq!(scores.len(), 3);
    }

    #[test]
    fn cn_empty_pairs() {
        let g = diamond();
        let scores = link_pred_common_neighbors(&g, &[]).unwrap();
        assert!(scores.is_empty());
    }

    #[test]
    fn cn_invalid_vertex() {
        let g = diamond();
        assert!(link_pred_common_neighbors(&g, &[(0, 10)]).is_err());
    }

    // --- adamic adar ---

    #[test]
    fn aa_basic() {
        let g = diamond();
        let scores = link_pred_adamic_adar(&g, &[(0, 3)]).unwrap();
        // Common neighbors {1,2}; deg(1)=3, deg(2)=3
        let expected = 2.0 / 3.0_f64.ln();
        assert!((scores[0] - expected).abs() < 1e-10);
    }

    #[test]
    fn aa_no_common() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let scores = link_pred_adamic_adar(&g, &[(0, 2)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }

    #[test]
    fn aa_degree_one_neighbor() {
        // If common neighbor has degree 1, log(1) = 0 → skipped
        let g = Graph::from_edges(&[(0, 2), (1, 2)], false, Some(3)).unwrap();
        let scores = link_pred_adamic_adar(&g, &[(0, 1)]).unwrap();
        // Common neighbor is 2 with degree 2: 1/log(2)
        let expected = 1.0 / 2.0_f64.ln();
        assert!((scores[0] - expected).abs() < 1e-10);
    }

    // --- resource allocation ---

    #[test]
    fn ra_basic() {
        let g = diamond();
        let scores = link_pred_resource_allocation(&g, &[(0, 3)]).unwrap();
        // Common neighbors {1,2}; deg(1)=3, deg(2)=3
        let expected = 2.0 / 3.0;
        assert!((scores[0] - expected).abs() < 1e-10);
    }

    #[test]
    fn ra_no_common() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let scores = link_pred_resource_allocation(&g, &[(0, 2)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }

    // --- preferential attachment ---

    #[test]
    fn pa_basic() {
        let g = diamond();
        let scores = link_pred_preferential_attachment(&g, &[(0, 3)]).unwrap();
        // deg(0)=2, deg(3)=2
        assert!((scores[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn pa_star_center() {
        let g = star5();
        let scores = link_pred_preferential_attachment(&g, &[(1, 2)]).unwrap();
        // deg(1)=1, deg(2)=1
        assert!((scores[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pa_isolated() {
        let g = Graph::with_vertices(3);
        let scores = link_pred_preferential_attachment(&g, &[(0, 1)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }

    // --- jaccard ---

    #[test]
    fn jaccard_perfect_overlap() {
        let g = diamond();
        let scores = link_pred_jaccard(&g, &[(0, 3)]).unwrap();
        // N(0)={1,2}, N(3)={1,2}. Jaccard = 2/2 = 1.0
        assert!((scores[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let g = diamond();
        let scores = link_pred_jaccard(&g, &[(0, 1)]).unwrap();
        // N(0)={1,2}, N(1)={0,2,3}. Intersection={2}, Union={0,1,2,3}
        // Jaccard = 1/4 = 0.25
        assert!((scores[0] - 0.25).abs() < 1e-10);
    }

    #[test]
    fn jaccard_no_overlap() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let scores = link_pred_jaccard(&g, &[(0, 2)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }

    #[test]
    fn jaccard_both_isolated() {
        let g = Graph::with_vertices(3);
        let scores = link_pred_jaccard(&g, &[(0, 1)]).unwrap();
        assert!(scores[0].abs() < 1e-10);
    }
}
