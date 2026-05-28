//! Bipartite graph from a weighted biadjacency matrix (ALGO-CN-038).
//!
//! Counterpart of `igraph_weighted_biadjacency()` in
//! `references/igraph/src/misc/bipartite.c:770-830`.
//!
//! Creates a bipartite graph from an `n1 × n2` weighted matrix where
//! non-zero entries become edges whose weight is the entry value.

use crate::algorithms::games::bipartite::BipartiteMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`weighted_biadjacency`].
#[derive(Debug, Clone)]
pub struct WeightedBiadjacencyResult {
    /// The constructed bipartite graph on `n1 + n2` vertices.
    pub graph: Graph,
    /// Per-vertex type: `false` = partition 1 (rows), `true` = partition 2 (columns).
    pub types: Vec<bool>,
    /// Per-edge weight corresponding to the matrix entry that produced it.
    pub weights: Vec<f64>,
}

/// Build a bipartite graph from a weighted biadjacency matrix.
///
/// The matrix has `n1` rows and `n2` columns. The resulting graph has
/// `n1 + n2` vertices: vertices `[0, n1)` correspond to rows (partition 1)
/// and vertices `[n1, n1+n2)` correspond to columns (partition 2).
///
/// Each non-zero entry in the matrix produces one edge (or two mutual
/// edges when `directed = true` and `mode = All`) whose weight equals
/// the matrix entry value.
///
/// # Parameters
///
/// * `matrix` — row-major `n1 × n2` weighted biadjacency matrix (all rows
///   must have equal length `n2`).
/// * `directed` — whether the resulting graph is directed.
/// * `mode` — edge direction policy (only meaningful when `directed = true`):
///   - [`BipartiteMode::Out`] — arcs from rows to columns.
///   - [`BipartiteMode::In`] — arcs from columns to rows.
///   - [`BipartiteMode::All`] — mutual arcs in both directions.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — ragged matrix or vertex count overflow.
///
/// # Examples
///
/// ```
/// use rust_igraph::{weighted_biadjacency, BipartiteMode};
///
/// let matrix: &[&[f64]] = &[
///     &[1.5, 0.0],
///     &[0.0, 2.5],
/// ];
/// let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
/// assert_eq!(r.graph.vcount(), 4);
/// assert_eq!(r.graph.ecount(), 2);
/// assert_eq!(r.weights, vec![1.5, 2.5]);
/// assert_eq!(r.types, vec![false, false, true, true]);
/// ```
pub fn weighted_biadjacency(
    matrix: &[&[f64]],
    directed: bool,
    mode: BipartiteMode,
) -> IgraphResult<WeightedBiadjacencyResult> {
    let n1 = matrix.len();
    let n2 = if n1 == 0 { 0 } else { matrix[0].len() };

    for (row_idx, row) in matrix.iter().enumerate() {
        if row.len() != n2 {
            return Err(IgraphError::InvalidArgument(format!(
                "weighted_biadjacency: row {row_idx} has length {} but expected {n2}",
                row.len()
            )));
        }
    }

    let total = n1.checked_add(n2).ok_or_else(|| {
        IgraphError::InvalidArgument(
            "weighted_biadjacency: total vertex count overflows usize".to_string(),
        )
    })?;
    let total_u32 = u32::try_from(total).map_err(|_| {
        IgraphError::InvalidArgument(
            "weighted_biadjacency: total vertex count exceeds u32::MAX".to_string(),
        )
    })?;
    let n1_u32 = u32::try_from(n1).map_err(|_| {
        IgraphError::InvalidArgument("weighted_biadjacency: n1 exceeds u32::MAX".to_string())
    })?;
    let n2_u32 = u32::try_from(n2).map_err(|_| {
        IgraphError::InvalidArgument("weighted_biadjacency: n2 exceeds u32::MAX".to_string())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();
    let mut weights: Vec<f64> = Vec::new();

    for j in 0..n2_u32 {
        for i in 0..n1_u32 {
            let weight = matrix[i as usize][j as usize];
            if weight == 0.0 {
                continue;
            }

            let (from, to) = if mode == BipartiteMode::In {
                (n1_u32 + j, i)
            } else {
                (i, n1_u32 + j)
            };

            if mode != BipartiteMode::All || !directed {
                edges.push((from, to));
                weights.push(weight);
            } else {
                edges.push((from, to));
                weights.push(weight);
                edges.push((to, from));
                weights.push(weight);
            }
        }
    }

    let mut graph = Graph::new(total_u32, directed)?;
    if !edges.is_empty() {
        graph.add_edges(edges)?;
    }

    let mut types = vec![false; total];
    for t in types.iter_mut().skip(n1) {
        *t = true;
    }

    Ok(WeightedBiadjacencyResult {
        graph,
        types,
        weights,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix() {
        let matrix: &[&[f64]] = &[];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.weights.is_empty());
        assert!(r.types.is_empty());
    }

    #[test]
    fn single_nonzero() {
        let matrix: &[&[f64]] = &[&[3.5]];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.weights, vec![3.5]);
        assert_eq!(r.types, vec![false, true]);
    }

    #[test]
    fn single_zero() {
        let matrix: &[&[f64]] = &[&[0.0]];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.weights.is_empty());
    }

    #[test]
    fn two_by_two() {
        let matrix: &[&[f64]] = &[&[1.5, 0.0], &[0.0, 2.5]];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 4);
        assert_eq!(r.graph.ecount(), 2);
        assert_eq!(r.weights, vec![1.5, 2.5]);
        assert_eq!(r.types, vec![false, false, true, true]);
    }

    #[test]
    fn negative_weight() {
        let matrix: &[&[f64]] = &[&[-2.0]];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.weights, vec![-2.0]);
    }

    #[test]
    fn directed_out() {
        let matrix: &[&[f64]] = &[&[1.0, 2.0]];
        let r = weighted_biadjacency(matrix, true, BipartiteMode::Out).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 2);
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = r.graph.edge(eid as u32).unwrap();
            assert!(!r.types[s as usize], "source should be row");
            assert!(r.types[t as usize], "target should be col");
        }
        assert_eq!(r.weights, vec![1.0, 2.0]);
    }

    #[test]
    fn directed_in() {
        let matrix: &[&[f64]] = &[&[1.0, 2.0]];
        let r = weighted_biadjacency(matrix, true, BipartiteMode::In).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 2);
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = r.graph.edge(eid as u32).unwrap();
            assert!(r.types[s as usize], "source should be col");
            assert!(!r.types[t as usize], "target should be row");
        }
    }

    #[test]
    fn directed_all_mutual() {
        let matrix: &[&[f64]] = &[&[5.0]];
        let r = weighted_biadjacency(matrix, true, BipartiteMode::All).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 2);
        assert_eq!(r.weights, vec![5.0, 5.0]);
    }

    #[test]
    fn undirected_ignores_mode() {
        let matrix: &[&[f64]] = &[&[1.0]];
        let r_out = weighted_biadjacency(matrix, false, BipartiteMode::Out).unwrap();
        let r_all = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r_out.graph.ecount(), 1);
        assert_eq!(r_all.graph.ecount(), 1);
    }

    #[test]
    fn ragged_matrix_rejected() {
        let matrix: &[&[f64]] = &[&[1.0, 2.0], &[3.0]];
        let result = weighted_biadjacency(matrix, false, BipartiteMode::All);
        assert!(result.is_err());
    }

    #[test]
    fn larger_matrix() {
        let matrix: &[&[f64]] = &[&[1.0, 0.0, 3.0], &[0.0, 2.0, 0.0], &[4.0, 0.0, 0.0]];
        let r = weighted_biadjacency(matrix, false, BipartiteMode::All).unwrap();
        assert_eq!(r.graph.vcount(), 6); // 3 + 3
        assert_eq!(r.graph.ecount(), 4); // 4 non-zero entries
        assert_eq!(r.weights.len(), 4);
        assert_eq!(r.types, vec![false, false, false, true, true, true]);
    }
}
