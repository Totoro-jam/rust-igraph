//! Weighted biadjacency matrix extraction (ALGO-PR-046).
//!
//! Counterpart of `igraph_get_biadjacency()` with weights in
//! `references/igraph/src/misc/bipartite.c:874-956`.
//!
//! Extracts the weighted biadjacency matrix from a bipartite graph
//! given a per-vertex type assignment and per-edge weight vector.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`get_biadjacency_weighted`].
#[derive(Debug, Clone)]
pub struct GetBiadjacencyWeightedResult {
    /// Row-major `n1 × n2` weighted biadjacency matrix.
    /// `matrix[i][j]` is the sum of weights of edges between
    /// row-vertex `i` and column-vertex `j`.
    pub matrix: Vec<Vec<f64>>,
    /// Vertex IDs corresponding to rows (partition with `types[v] == false`).
    pub row_ids: Vec<VertexId>,
    /// Vertex IDs corresponding to columns (partition with `types[v] == true`).
    pub col_ids: Vec<VertexId>,
}

/// Extract a weighted biadjacency matrix from a bipartite graph.
///
/// Given a graph, a per-vertex type vector, and per-edge weights,
/// returns the `n1 × n2` biadjacency matrix where entry `(i, j)` is
/// the sum of weights of edges between row-vertex `i` and
/// column-vertex `j`.
///
/// Edges within the same partition are silently ignored.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `types` — per-vertex boolean: `false` = row, `true` = column.
/// * `weights` — per-edge weight (must have length `graph.ecount()`).
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — if `types.len() != vcount` or
///   `weights.len() != ecount`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, get_biadjacency_weighted};
///
/// let mut g = Graph::new(4, false).unwrap();
/// // rows: {0, 1}, cols: {2, 3}
/// g.add_edges(vec![(0, 2), (0, 3), (1, 2)]).unwrap();
/// let types = vec![false, false, true, true];
/// let weights = vec![1.5, 2.0, 3.0];
/// let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
/// assert_eq!(r.matrix[0][0], 1.5); // edge (0,2) weight
/// assert_eq!(r.matrix[0][1], 2.0); // edge (0,3) weight
/// assert_eq!(r.matrix[1][0], 3.0); // edge (1,2) weight
/// assert_eq!(r.matrix[1][1], 0.0); // no edge (1,3)
/// ```
pub fn get_biadjacency_weighted(
    graph: &Graph,
    types: &[bool],
    weights: &[f64],
) -> IgraphResult<GetBiadjacencyWeightedResult> {
    let vcount = graph.vcount();
    let ecount = graph.ecount();

    if types.len() != vcount as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "get_biadjacency_weighted: types length ({}) != vcount ({})",
            types.len(),
            vcount
        )));
    }

    if weights.len() != ecount {
        return Err(IgraphError::InvalidArgument(format!(
            "get_biadjacency_weighted: weights length ({}) != ecount ({})",
            weights.len(),
            ecount
        )));
    }

    let mut n1: usize = 0;
    let mut n2: usize = 0;
    let mut row_ids: Vec<VertexId> = Vec::new();
    let mut col_ids: Vec<VertexId> = Vec::new();
    let mut vertex_to_idx: Vec<u32> = vec![0; vcount as usize];

    for v in 0..vcount {
        if types[v as usize] {
            #[allow(clippy::cast_possible_truncation)]
            let idx = n2 as u32;
            vertex_to_idx[v as usize] = idx;
            col_ids.push(v);
            n2 += 1;
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let idx = n1 as u32;
            vertex_to_idx[v as usize] = idx;
            row_ids.push(v);
            n1 += 1;
        }
    }

    let mut matrix = vec![vec![0.0_f64; n2]; n1];

    for (eid, &w) in weights.iter().enumerate().take(ecount) {
        #[allow(clippy::cast_possible_truncation)]
        let e = eid as u32;
        let (src, tgt) = graph.edge(e)?;

        let src_type = types[src as usize];
        let tgt_type = types[tgt as usize];

        if src_type == tgt_type {
            continue;
        }

        let (row_v, col_v) = if src_type { (tgt, src) } else { (src, tgt) };

        let ri = vertex_to_idx[row_v as usize] as usize;
        let ci = vertex_to_idx[col_v as usize] as usize;
        matrix[ri][ci] += w;
    }

    Ok(GetBiadjacencyWeightedResult {
        matrix,
        row_ids,
        col_ids,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let r = get_biadjacency_weighted(&g, &[], &[]).unwrap();
        assert!(r.matrix.is_empty());
    }

    #[test]
    fn types_mismatch() {
        let g = Graph::new(3, false).unwrap();
        let result = get_biadjacency_weighted(&g, &[false, true], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn weights_mismatch() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        let result = get_biadjacency_weighted(&g, &[false, true], &[1.0, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn simple_weighted() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 2), (0, 3), (1, 2)]).unwrap();
        let types = vec![false, false, true, true];
        let weights = vec![1.5, 2.0, 3.0];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        assert_eq!(r.matrix[0][0], 1.5);
        assert_eq!(r.matrix[0][1], 2.0);
        assert_eq!(r.matrix[1][0], 3.0);
        assert_eq!(r.matrix[1][1], 0.0);
    }

    #[test]
    fn parallel_edges_sum_weights() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 1)]).unwrap();
        let types = vec![false, true];
        let weights = vec![1.5, 2.5];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        assert_eq!(r.matrix[0][0], 4.0);
    }

    #[test]
    fn ignores_same_partition() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (0, 2)]).unwrap();
        let types = vec![false, false, true];
        let weights = vec![5.0, 3.0];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        // Edge (0,1) is same partition → ignored
        assert_eq!(r.matrix[0][0], 3.0); // only edge (0,2) with weight 3.0
        assert_eq!(r.matrix[1][0], 0.0);
    }

    #[test]
    fn negative_weights() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        let types = vec![false, true];
        let weights = vec![-2.5];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        assert_eq!(r.matrix[0][0], -2.5);
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 2), (2, 1)]).unwrap();
        let types = vec![false, false, true];
        let weights = vec![1.0, 2.0];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        assert_eq!(r.matrix[0][0], 1.0); // edge (0,2)
        assert_eq!(r.matrix[1][0], 2.0); // edge (2,1): col→row counts
    }

    #[test]
    fn row_col_ids() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(1, 0), (3, 2)]).unwrap();
        let types = vec![true, false, true, false];
        let weights = vec![1.0, 2.0];
        let r = get_biadjacency_weighted(&g, &types, &weights).unwrap();
        assert_eq!(r.row_ids, vec![1, 3]);
        assert_eq!(r.col_ids, vec![0, 2]);
        assert_eq!(r.matrix[0][0], 1.0);
        assert_eq!(r.matrix[1][1], 2.0);
    }
}
