//! Bipartite graph from biadjacency matrix (ALGO-CN-037).
//!
//! Counterpart of `igraph_biadjacency()` in
//! `references/igraph/src/misc/bipartite.c:634-737`.
//!
//! A *biadjacency matrix* (also called incidence matrix of a bipartite
//! graph) is an `n1 × n2` matrix where rows correspond to vertices in
//! partition 1 and columns to partition 2. Non-zero entries indicate
//! edges between the corresponding vertices.

use crate::algorithms::games::bipartite::BipartiteMode;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`biadjacency`].
#[derive(Debug, Clone)]
pub struct BiadjacencyResult {
    /// The constructed bipartite graph on `n1 + n2` vertices.
    pub graph: Graph,
    /// Per-vertex type: `false` = partition 1 (rows), `true` = partition 2 (columns).
    pub types: Vec<bool>,
}

/// Build a bipartite graph from a biadjacency matrix.
///
/// The matrix has `n1` rows and `n2` columns. The resulting graph has
/// `n1 + n2` vertices: vertices `[0, n1)` correspond to rows (partition 1)
/// and vertices `[n1, n1+n2)` correspond to columns (partition 2).
///
/// # Parameters
///
/// * `matrix` — row-major `n1 × n2` biadjacency matrix (all rows must have
///   equal length `n2`).
/// * `directed` — whether the resulting graph is directed.
/// * `mode` — edge direction policy (only meaningful when `directed = true`):
///   - [`BipartiteMode::Out`] — arcs from rows to columns.
///   - [`BipartiteMode::In`] — arcs from columns to rows.
///   - [`BipartiteMode::All`] — mutual arcs in both directions.
/// * `multiple` — if `true`, matrix values are interpreted as integer
///   multiplicities (must be non-negative); if `false`, any non-zero
///   value produces a single edge.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — ragged matrix, negative entry
///   when `multiple = true`, or vertex count overflow.
///
/// # Examples
///
/// ```
/// use rust_igraph::{biadjacency, BipartiteMode};
///
/// // 2×3 matrix → graph with 5 vertices.
/// let matrix: &[&[f64]] = &[
///     &[1.0, 0.0, 2.0],
///     &[0.0, 1.0, 0.0],
/// ];
/// let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
/// assert_eq!(r.graph.vcount(), 5);
/// assert_eq!(r.graph.ecount(), 3); // three non-zero entries
/// assert_eq!(r.types, vec![false, false, true, true, true]);
///
/// // With multiple=true, entry 2.0 produces 2 edges.
/// let m = biadjacency(matrix, false, BipartiteMode::All, true).unwrap();
/// assert_eq!(m.graph.ecount(), 4); // 1 + 2 + 1
/// ```
pub fn biadjacency(
    matrix: &[&[f64]],
    directed: bool,
    mode: BipartiteMode,
    multiple: bool,
) -> IgraphResult<BiadjacencyResult> {
    let n1 = matrix.len();
    let n2 = if n1 == 0 { 0 } else { matrix[0].len() };

    for (row_idx, row) in matrix.iter().enumerate() {
        if row.len() != n2 {
            return Err(IgraphError::InvalidArgument(format!(
                "biadjacency: row {row_idx} has length {} but expected {n2}",
                row.len()
            )));
        }
    }

    let total = n1.checked_add(n2).ok_or_else(|| {
        IgraphError::InvalidArgument("biadjacency: total vertex count overflows usize".to_string())
    })?;
    let total_u32 = u32::try_from(total).map_err(|_| {
        IgraphError::InvalidArgument("biadjacency: total vertex count exceeds u32::MAX".to_string())
    })?;
    // Safe: n1 <= total which fits in u32.
    let n1_u32 = u32::try_from(n1).map_err(|_| {
        IgraphError::InvalidArgument("biadjacency: n1 exceeds u32::MAX".to_string())
    })?;

    let n2_u32 = u32::try_from(n2).map_err(|_| {
        IgraphError::InvalidArgument("biadjacency: n2 exceeds u32::MAX".to_string())
    })?;

    let mut edges: Vec<(VertexId, VertexId)> = Vec::new();

    if multiple {
        if n1 > 0 && n2 > 0 {
            for row in matrix {
                for &val in *row {
                    if val < 0.0 {
                        return Err(IgraphError::InvalidArgument(format!(
                            "biadjacency: matrix elements must be non-negative when multiple=true, found {val}"
                        )));
                    }
                }
            }
        }

        for j in 0..n2_u32 {
            for i in 0..n1_u32 {
                #[allow(clippy::cast_possible_truncation)]
                let elem = matrix[i as usize][j as usize] as i64;
                if elem == 0 {
                    continue;
                }

                let (from, to) = if mode == BipartiteMode::In {
                    (n1_u32 + j, i)
                } else {
                    (i, n1_u32 + j)
                };

                if mode != BipartiteMode::All || !directed {
                    for _ in 0..elem {
                        edges.push((from, to));
                    }
                } else {
                    for _ in 0..elem {
                        edges.push((from, to));
                        edges.push((to, from));
                    }
                }
            }
        }
    } else {
        for j in 0..n2_u32 {
            for i in 0..n1_u32 {
                if matrix[i as usize][j as usize] != 0.0 {
                    let (from, to) = if mode == BipartiteMode::In {
                        (n1_u32 + j, i)
                    } else {
                        (i, n1_u32 + j)
                    };

                    if mode != BipartiteMode::All || !directed {
                        edges.push((from, to));
                    } else {
                        edges.push((from, to));
                        edges.push((to, from));
                    }
                }
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

    Ok(BiadjacencyResult { graph, types })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix() {
        let matrix: &[&[f64]] = &[];
        let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        assert_eq!(r.graph.vcount(), 0);
        assert_eq!(r.graph.ecount(), 0);
        assert!(r.types.is_empty());
    }

    #[test]
    fn single_entry_nonzero() {
        let matrix: &[&[f64]] = &[&[1.0]];
        let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 1);
        assert_eq!(r.types, vec![false, true]);
    }

    #[test]
    fn single_entry_zero() {
        let matrix: &[&[f64]] = &[&[0.0]];
        let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        assert_eq!(r.graph.vcount(), 2);
        assert_eq!(r.graph.ecount(), 0);
    }

    #[test]
    fn two_by_three_no_multiple() {
        let matrix: &[&[f64]] = &[&[1.0, 0.0, 3.0], &[0.0, 1.0, 0.0]];
        let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        assert_eq!(r.graph.vcount(), 5);
        assert_eq!(r.graph.ecount(), 3);
        assert_eq!(r.types, vec![false, false, true, true, true]);
    }

    #[test]
    fn two_by_three_with_multiple() {
        let matrix: &[&[f64]] = &[&[1.0, 0.0, 2.0], &[0.0, 1.0, 0.0]];
        let r = biadjacency(matrix, false, BipartiteMode::All, true).unwrap();
        assert_eq!(r.graph.vcount(), 5);
        assert_eq!(r.graph.ecount(), 4); // 1 + 2 + 1
    }

    #[test]
    fn directed_out() {
        let matrix: &[&[f64]] = &[&[1.0, 1.0], &[1.0, 0.0]];
        let r = biadjacency(matrix, true, BipartiteMode::Out, false).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 3);
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = r.graph.edge(eid as u32).unwrap();
            assert!(!r.types[s as usize], "source should be row (partition 1)");
            assert!(r.types[t as usize], "target should be col (partition 2)");
        }
    }

    #[test]
    fn directed_in() {
        let matrix: &[&[f64]] = &[&[1.0, 1.0], &[1.0, 0.0]];
        let r = biadjacency(matrix, true, BipartiteMode::In, false).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 3);
        for eid in 0..r.graph.ecount() {
            #[allow(clippy::cast_possible_truncation)]
            let (s, t) = r.graph.edge(eid as u32).unwrap();
            assert!(r.types[s as usize], "source should be col (partition 2)");
            assert!(!r.types[t as usize], "target should be row (partition 1)");
        }
    }

    #[test]
    fn directed_all_mutual() {
        let matrix: &[&[f64]] = &[&[1.0]];
        let r = biadjacency(matrix, true, BipartiteMode::All, false).unwrap();
        assert!(r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 2); // mutual
    }

    #[test]
    fn multiple_with_mutual() {
        let matrix: &[&[f64]] = &[&[2.0]];
        let r = biadjacency(matrix, true, BipartiteMode::All, true).unwrap();
        assert_eq!(r.graph.ecount(), 4); // 2 × 2 mutual
    }

    #[test]
    fn rejects_negative_in_multiple_mode() {
        let matrix: &[&[f64]] = &[&[-1.0]];
        let result = biadjacency(matrix, false, BipartiteMode::All, true);
        assert!(result.is_err());
    }

    #[test]
    fn negative_allowed_in_non_multiple_mode() {
        let matrix: &[&[f64]] = &[&[-1.0]];
        let r = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        assert_eq!(r.graph.ecount(), 1); // non-zero → edge
    }

    #[test]
    fn ragged_matrix_rejected() {
        let matrix: &[&[f64]] = &[&[1.0, 2.0], &[3.0]];
        let result = biadjacency(matrix, false, BipartiteMode::All, false);
        assert!(result.is_err());
    }

    #[test]
    fn undirected_ignores_mode_out() {
        let matrix: &[&[f64]] = &[&[1.0]];
        let r = biadjacency(matrix, false, BipartiteMode::Out, false).unwrap();
        assert!(!r.graph.is_directed());
        assert_eq!(r.graph.ecount(), 1);
    }
}
