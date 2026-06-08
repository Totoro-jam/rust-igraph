//! Label spreading for semi-supervised node classification (ALGO-TR-012).
//!
//! Given a graph where some vertices have known labels and others are
//! unlabeled, propagates labels through the graph structure to predict
//! labels for unlabeled vertices. Implements the iterative label spreading
//! algorithm (Zhou et al., 2004) which balances between smoothness over
//! the graph and fitting the initial labels.
//!
//! Used as a baseline for graph semi-supervised learning and as a
//! post-processing step in GNN pipelines.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of label spreading prediction.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelSpreadResult {
    /// Predicted label for each vertex (class with max probability).
    pub labels: Vec<u32>,
    /// Confidence matrix: `confidence[v][c]` = probability of vertex v
    /// belonging to class c.
    pub confidence: Vec<Vec<f64>>,
}

/// Predict labels for unlabeled vertices using label spreading.
///
/// Iteratively propagates label information from labeled vertices to
/// their neighbors via the graph structure. At each step:
/// `Y_{t+1} = α · S · Y_t + (1-α) · Y_0`
///
/// where `S = D^{-1/2} A D^{-1/2}` is the symmetric normalized adjacency,
/// `Y_0` is the initial label matrix, and `α` controls the balance between
/// propagation and clamping to initial labels.
///
/// # Parameters
///
/// - `graph` — Undirected graph.
/// - `labels` — Label for each vertex: `Some(class_id)` for labeled vertices,
///   `None` for unlabeled vertices to predict.
/// - `alpha` — Propagation strength (0 < alpha < 1). Higher = more propagation.
///   Typical: 0.2–0.8.
/// - `max_iter` — Maximum iterations.
/// - `tol` — Convergence tolerance on max label probability change.
///
/// # Returns
///
/// A [`LabelSpreadResult`] with predicted labels and confidence scores.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, label_spread};
///
/// // Path 0-1-2-3: label vertex 0 as class 0, vertex 3 as class 1
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3)], false, Some(4)).unwrap();
/// let labels = vec![Some(0), None, None, Some(1)];
/// let result = label_spread(&g, &labels, 0.5, 50, 1e-6).unwrap();
/// // Vertex 1 should be closer to class 0, vertex 2 closer to class 1
/// assert_eq!(result.labels[0], 0);
/// assert_eq!(result.labels[3], 1);
/// assert_eq!(result.labels[1], 0);
/// assert_eq!(result.labels[2], 1);
/// ```
pub fn label_spread(
    graph: &Graph,
    labels: &[Option<u32>],
    alpha: f64,
    max_iter: usize,
    tol: f64,
) -> IgraphResult<LabelSpreadResult> {
    let nv = graph.vcount() as usize;

    if labels.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "labels length {} does not match vcount {}",
            labels.len(),
            nv
        )));
    }

    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(IgraphError::InvalidArgument(format!(
            "alpha must be in (0.0, 1.0), got {alpha}"
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "label_spread is defined for undirected graphs only".to_string(),
        ));
    }

    // Determine number of classes
    let num_classes = labels.iter().filter_map(|l| *l).max().map_or(0, |m| m + 1) as usize;

    if num_classes == 0 {
        return Err(IgraphError::InvalidArgument(
            "at least one labeled vertex is required".to_string(),
        ));
    }

    // Compute degrees and D^{-1/2}
    let mut degrees = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }
    let inv_sqrt_deg: Vec<f64> = degrees
        .iter()
        .map(|&d| if d == 0 { 0.0 } else { 1.0 / (d as f64).sqrt() })
        .collect();

    // Initialize Y_0: one-hot for labeled, uniform for unlabeled
    let mut y_init: Vec<Vec<f64>> = Vec::with_capacity(nv);
    for label in labels {
        let mut row = vec![0.0; num_classes];
        if let Some(c) = label {
            let c_idx = *c as usize;
            if c_idx < num_classes {
                row[c_idx] = 1.0;
            }
        } else {
            let uniform = 1.0 / num_classes as f64;
            row.fill(uniform);
        }
        y_init.push(row);
    }

    let one_minus_alpha = 1.0 - alpha;
    let mut y_current = y_init.clone();

    // Iterate: Y_{t+1} = α · S · Y_t + (1-α) · Y_0
    for _ in 0..max_iter {
        let mut y_next: Vec<Vec<f64>> = vec![vec![0.0; num_classes]; nv];
        let mut max_diff = 0.0_f64;

        // Apply S = D^{-1/2} A D^{-1/2} to y_current
        for v in 0..nv {
            if degrees[v] == 0 {
                // Isolated: keep initial label
                for c in 0..num_classes {
                    y_next[v][c] = y_init[v][c];
                }
                continue;
            }

            let neighbors = graph.neighbors(v as VertexId)?;
            for c in 0..num_classes {
                let mut propagated = 0.0;
                for &u in &neighbors {
                    let u_idx = u as usize;
                    propagated += inv_sqrt_deg[u_idx] * y_current[u_idx][c];
                }
                propagated *= inv_sqrt_deg[v];

                let new_val = alpha * propagated + one_minus_alpha * y_init[v][c];
                let diff = (new_val - y_current[v][c]).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
                y_next[v][c] = new_val;
            }
        }

        y_current = y_next;

        if max_diff < tol {
            break;
        }
    }

    // Extract predictions
    let mut predicted_labels = Vec::with_capacity(nv);
    for row in &y_current {
        let mut best_class = 0u32;
        let mut best_prob = f64::NEG_INFINITY;
        for (c, &prob) in row.iter().enumerate() {
            if prob > best_prob {
                best_prob = prob;
                best_class = c as u32;
            }
        }
        predicted_labels.push(best_class);
    }

    Ok(LabelSpreadResult {
        labels: predicted_labels,
        confidence: y_current,
    })
}

/// Predict labels using simple majority voting from labeled neighbors.
///
/// A non-iterative baseline: each unlabeled vertex adopts the most
/// common label among its labeled neighbors. If no labeled neighbor
/// exists, assigns label 0 (or keeps the existing label if provided).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, label_propagate_predict};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2),(2,3)], false, Some(4)).unwrap();
/// let labels = vec![Some(0), Some(0), None, Some(1)];
/// let predicted = label_propagate_predict(&g, &labels).unwrap();
/// // Vertex 2 has neighbors: 0(class 0), 1(class 0), 3(class 1) → majority = 0
/// assert_eq!(predicted[2], 0);
/// ```
pub fn label_propagate_predict(graph: &Graph, labels: &[Option<u32>]) -> IgraphResult<Vec<u32>> {
    let nv = graph.vcount() as usize;

    if labels.len() != nv {
        return Err(IgraphError::InvalidArgument(format!(
            "labels length {} does not match vcount {}",
            labels.len(),
            nv
        )));
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "label_propagate_predict is defined for undirected graphs only".to_string(),
        ));
    }

    let num_classes = labels.iter().filter_map(|l| *l).max().map_or(0, |m| m + 1) as usize;

    let mut result: Vec<u32> = Vec::with_capacity(nv);

    for (v, label) in labels.iter().enumerate() {
        if let Some(c) = label {
            result.push(*c);
        } else {
            // Count labeled neighbors by class
            let neighbors = graph.neighbors(v as VertexId)?;
            let mut counts = vec![0u32; num_classes.max(1)];
            for &u in &neighbors {
                if let Some(c) = labels[u as usize] {
                    if (c as usize) < counts.len() {
                        counts[c as usize] += 1;
                    }
                }
            }

            let best_class = counts
                .iter()
                .enumerate()
                .max_by_key(|(_, cnt)| *cnt)
                .map_or(0, |(c, _)| c as u32);
            result.push(best_class);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(4)).unwrap()
    }

    fn triangle_with_tail() -> Graph {
        // 0-1-2-0, 2-3
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2), (2, 3)], false, Some(4)).unwrap()
    }

    // --- label_spread tests ---

    #[test]
    fn spread_basic_path() {
        let g = path4();
        let labels = vec![Some(0), None, None, Some(1)];
        let result = label_spread(&g, &labels, 0.5, 100, 1e-8).unwrap();
        assert_eq!(result.labels[0], 0);
        assert_eq!(result.labels[3], 1);
        // Middle vertices: vertex 1 closer to 0, vertex 2 closer to 1
        assert_eq!(result.labels[1], 0);
        assert_eq!(result.labels[2], 1);
    }

    #[test]
    fn spread_all_labeled() {
        let g = path4();
        let labels = vec![Some(0), Some(1), Some(0), Some(1)];
        let result = label_spread(&g, &labels, 0.3, 50, 1e-6).unwrap();
        // With strong clamping, labels should stay close to initial
        assert_eq!(result.labels[0], 0);
        assert_eq!(result.labels[1], 1);
        assert_eq!(result.labels[2], 0);
        assert_eq!(result.labels[3], 1);
    }

    #[test]
    fn spread_single_class() {
        let g = path4();
        let labels = vec![Some(0), None, None, Some(0)];
        let result = label_spread(&g, &labels, 0.5, 50, 1e-6).unwrap();
        for &l in &result.labels {
            assert_eq!(l, 0);
        }
    }

    #[test]
    fn spread_confidence_sums_reasonable() {
        let g = path4();
        let labels = vec![Some(0), None, None, Some(1)];
        let result = label_spread(&g, &labels, 0.5, 50, 1e-6).unwrap();
        for row in &result.confidence {
            let sum: f64 = row.iter().sum();
            // Should be approximately 1 for labeled vertices
            assert!(sum > 0.0);
            for &p in row {
                assert!(p >= 0.0);
            }
        }
    }

    #[test]
    fn spread_invalid_alpha() {
        let g = path4();
        let labels = vec![Some(0), None, None, Some(1)];
        assert!(label_spread(&g, &labels, 0.0, 50, 1e-6).is_err());
        assert!(label_spread(&g, &labels, 1.0, 50, 1e-6).is_err());
        assert!(label_spread(&g, &labels, -0.5, 50, 1e-6).is_err());
    }

    #[test]
    fn spread_invalid_labels_length() {
        let g = path4();
        assert!(label_spread(&g, &[Some(0)], 0.5, 50, 1e-6).is_err());
    }

    #[test]
    fn spread_no_labeled_vertices() {
        let g = path4();
        let labels = vec![None, None, None, None];
        assert!(label_spread(&g, &labels, 0.5, 50, 1e-6).is_err());
    }

    #[test]
    fn spread_directed_error() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        let labels = vec![Some(0), None, Some(1)];
        assert!(label_spread(&g, &labels, 0.5, 50, 1e-6).is_err());
    }

    #[test]
    fn spread_isolated_vertex() {
        let mut labels = vec![Some(0), None, None, Some(1), None];
        // Vertex 4 is isolated
        let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 3)], false, Some(5)).unwrap();
        labels.push(None);
        labels.truncate(5);
        let result = label_spread(&g, &labels, 0.5, 50, 1e-6).unwrap();
        assert_eq!(result.labels.len(), 5);
    }

    #[test]
    fn spread_multiclass() {
        let g =
            Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], false, Some(6)).unwrap();
        let labels = vec![Some(0), None, Some(1), None, Some(2), None];
        let result = label_spread(&g, &labels, 0.5, 100, 1e-8).unwrap();
        assert_eq!(result.labels[0], 0);
        assert_eq!(result.labels[2], 1);
        assert_eq!(result.labels[4], 2);
        assert_eq!(result.confidence[0].len(), 3);
    }

    // --- label_propagate_predict tests ---

    #[test]
    fn predict_majority_vote() {
        let g = triangle_with_tail();
        let labels = vec![Some(0), Some(0), None, Some(1)];
        let predicted = label_propagate_predict(&g, &labels).unwrap();
        // Vertex 2: neighbors are 0(class 0), 1(class 0), 3(class 1) → majority = 0
        assert_eq!(predicted[2], 0);
    }

    #[test]
    fn predict_all_labeled() {
        let g = path4();
        let labels = vec![Some(0), Some(1), Some(0), Some(1)];
        let predicted = label_propagate_predict(&g, &labels).unwrap();
        assert_eq!(predicted, vec![0, 1, 0, 1]);
    }

    #[test]
    fn predict_no_labeled_neighbors() {
        let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
        let labels = vec![Some(0), None, None, Some(1)];
        let predicted = label_propagate_predict(&g, &labels).unwrap();
        // Vertex 1: only neighbor is vertex 0 (class 0) → class 0
        assert_eq!(predicted[1], 0);
        // Vertex 2: only neighbor is vertex 3 (class 1) → class 1
        assert_eq!(predicted[2], 1);
    }

    #[test]
    fn predict_invalid_length() {
        let g = path4();
        assert!(label_propagate_predict(&g, &[Some(0)]).is_err());
    }

    #[test]
    fn predict_directed_error() {
        let g = Graph::from_edges(&[(0, 1)], true, Some(2)).unwrap();
        assert!(label_propagate_predict(&g, &[Some(0), None]).is_err());
    }
}
