//! Biadjacency matrix extraction (ALGO-PR-043).
//!
//! Counterpart of `igraph_get_biadjacency()` in
//! `references/igraph/src/misc/bipartite.c:874-956`.
//!
//! Extracts the biadjacency matrix from a bipartite graph given
//! a per-vertex type assignment.

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`get_biadjacency_matrix`].
#[derive(Debug, Clone)]
pub struct GetBiadjacencyResult {
    /// Row-major `n1 × n2` biadjacency matrix.
    /// `matrix[i][j]` is the number of edges between row-vertex `i`
    /// and column-vertex `j`.
    pub matrix: Vec<Vec<f64>>,
    /// Vertex IDs corresponding to rows (partition with `types[v] == false`).
    pub row_ids: Vec<VertexId>,
    /// Vertex IDs corresponding to columns (partition with `types[v] == true`).
    pub col_ids: Vec<VertexId>,
}

/// Extract the biadjacency matrix from a bipartite graph.
///
/// Given a graph and a per-vertex type vector (`false` = row partition,
/// `true` = column partition), returns the `n1 × n2` biadjacency
/// matrix where entry `(i, j)` counts the number of edges between
/// row-vertex `i` and column-vertex `j`.
///
/// Edges within the same partition are silently ignored (as in the C
/// implementation).
///
/// # Parameters
///
/// * `graph` — the input graph (directed or undirected).
/// * `types` — per-vertex boolean: `false` = row (partition 1),
///   `true` = column (partition 2). Must have length equal to `graph.vcount()`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — if `types.len() != graph.vcount()`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{biadjacency, get_biadjacency_matrix, BipartiteMode};
///
/// let matrix: &[&[f64]] = &[
///     &[1.0, 0.0, 2.0],
///     &[0.0, 1.0, 0.0],
/// ];
/// let r = biadjacency(matrix, false, BipartiteMode::All, true).unwrap();
/// let extracted = get_biadjacency_matrix(&r.graph, &r.types).unwrap();
/// assert_eq!(extracted.matrix.len(), 2); // 2 rows
/// assert_eq!(extracted.matrix[0].len(), 3); // 3 columns
/// assert_eq!(extracted.matrix[0][0], 1.0);
/// assert_eq!(extracted.matrix[0][2], 2.0);
/// assert_eq!(extracted.matrix[1][1], 1.0);
/// ```
pub fn get_biadjacency_matrix(graph: &Graph, types: &[bool]) -> IgraphResult<GetBiadjacencyResult> {
    let vcount = graph.vcount();

    if types.len() != vcount as usize {
        return Err(IgraphError::InvalidArgument(format!(
            "get_biadjacency_matrix: types length ({}) != vcount ({})",
            types.len(),
            vcount
        )));
    }

    // Count partition sizes and build mapping from vertex ID → row/col index.
    let mut n1: usize = 0;
    let mut n2: usize = 0;
    let mut row_ids: Vec<VertexId> = Vec::new();
    let mut col_ids: Vec<VertexId> = Vec::new();
    // Maps vertex ID → index within its partition.
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

    // Initialize n1 × n2 matrix of zeros.
    let mut matrix = vec![vec![0.0_f64; n2]; n1];

    // Iterate over all edges.
    let ecount = graph.ecount();
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let e = eid as u32;
        let (src, tgt) = graph.edge(e)?;

        let src_type = types[src as usize];
        let tgt_type = types[tgt as usize];

        // Skip edges within the same partition (matches C behaviour).
        if src_type == tgt_type {
            continue;
        }

        // Determine which is the row vertex and which is the column vertex.
        let (row_v, col_v) = if src_type { (tgt, src) } else { (src, tgt) };

        let ri = vertex_to_idx[row_v as usize] as usize;
        let ci = vertex_to_idx[col_v as usize] as usize;
        matrix[ri][ci] += 1.0;
    }

    Ok(GetBiadjacencyResult {
        matrix,
        row_ids,
        col_ids,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::algorithms::constructors::biadjacency::biadjacency;
    use crate::algorithms::games::bipartite::BipartiteMode;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let r = get_biadjacency_matrix(&g, &[]).unwrap();
        assert!(r.matrix.is_empty());
        assert!(r.row_ids.is_empty());
        assert!(r.col_ids.is_empty());
    }

    #[test]
    fn single_vertex_row() {
        let g = Graph::new(1, false).unwrap();
        let r = get_biadjacency_matrix(&g, &[false]).unwrap();
        assert_eq!(r.matrix.len(), 1);
        assert!(r.matrix[0].is_empty()); // 1×0
        assert_eq!(r.row_ids, vec![0]);
        assert!(r.col_ids.is_empty());
    }

    #[test]
    fn single_vertex_col() {
        let g = Graph::new(1, false).unwrap();
        let r = get_biadjacency_matrix(&g, &[true]).unwrap();
        assert!(r.matrix.is_empty()); // 0×1
        assert!(r.row_ids.is_empty());
        assert_eq!(r.col_ids, vec![0]);
    }

    #[test]
    fn types_length_mismatch() {
        let g = Graph::new(3, false).unwrap();
        let result = get_biadjacency_matrix(&g, &[false, true]);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_simple() {
        let matrix: &[&[f64]] = &[&[1.0, 0.0, 1.0], &[0.0, 1.0, 0.0]];
        let built = biadjacency(matrix, false, BipartiteMode::All, false).unwrap();
        let extracted = get_biadjacency_matrix(&built.graph, &built.types).unwrap();

        assert_eq!(extracted.matrix.len(), 2);
        assert_eq!(extracted.matrix[0].len(), 3);
        assert_eq!(extracted.matrix[0][0], 1.0);
        assert_eq!(extracted.matrix[0][1], 0.0);
        assert_eq!(extracted.matrix[0][2], 1.0);
        assert_eq!(extracted.matrix[1][0], 0.0);
        assert_eq!(extracted.matrix[1][1], 1.0);
        assert_eq!(extracted.matrix[1][2], 0.0);
    }

    #[test]
    fn roundtrip_with_multiple() {
        let matrix: &[&[f64]] = &[&[1.0, 0.0, 2.0], &[0.0, 1.0, 0.0]];
        let built = biadjacency(matrix, false, BipartiteMode::All, true).unwrap();
        let extracted = get_biadjacency_matrix(&built.graph, &built.types).unwrap();

        assert_eq!(extracted.matrix[0][0], 1.0);
        assert_eq!(extracted.matrix[0][2], 2.0);
        assert_eq!(extracted.matrix[1][1], 1.0);
    }

    #[test]
    fn directed_out() {
        let matrix: &[&[f64]] = &[&[1.0, 1.0]];
        let built = biadjacency(matrix, true, BipartiteMode::Out, false).unwrap();
        let extracted = get_biadjacency_matrix(&built.graph, &built.types).unwrap();
        assert_eq!(extracted.matrix[0][0], 1.0);
        assert_eq!(extracted.matrix[0][1], 1.0);
    }

    #[test]
    fn directed_in() {
        let matrix: &[&[f64]] = &[&[1.0, 1.0]];
        let built = biadjacency(matrix, true, BipartiteMode::In, false).unwrap();
        let extracted = get_biadjacency_matrix(&built.graph, &built.types).unwrap();
        // In mode: arcs from col→row, but biadjacency counts edges crossing partitions
        assert_eq!(extracted.matrix[0][0], 1.0);
        assert_eq!(extracted.matrix[0][1], 1.0);
    }

    #[test]
    fn directed_all_mutual() {
        let matrix: &[&[f64]] = &[&[1.0]];
        let built = biadjacency(matrix, true, BipartiteMode::All, false).unwrap();
        // All mode creates 2 arcs (mutual), extraction counts both
        let extracted = get_biadjacency_matrix(&built.graph, &built.types).unwrap();
        assert_eq!(extracted.matrix[0][0], 2.0);
    }

    #[test]
    fn ignores_same_partition_edges() {
        // Manually create a graph with an intra-partition edge
        let mut g = Graph::new(4, false).unwrap();
        // vertices 0,1 = rows (false); vertices 2,3 = cols (true)
        g.add_edges(vec![(0, 2), (0, 1), (1, 3)]).unwrap();
        let types = vec![false, false, true, true];
        let r = get_biadjacency_matrix(&g, &types).unwrap();
        // Edge (0,1) is same-partition → ignored
        assert_eq!(r.matrix[0][0], 1.0); // edge (0,2)
        assert_eq!(r.matrix[0][1], 0.0);
        assert_eq!(r.matrix[1][0], 0.0);
        assert_eq!(r.matrix[1][1], 1.0); // edge (1,3)
    }

    #[test]
    fn row_col_ids_ordering() {
        // Types: [true, false, true, false] → rows are {1,3}, cols are {0,2}
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(1, 0), (3, 2)]).unwrap();
        let types = vec![true, false, true, false];
        let r = get_biadjacency_matrix(&g, &types).unwrap();
        assert_eq!(r.row_ids, vec![1, 3]);
        assert_eq!(r.col_ids, vec![0, 2]);
        assert_eq!(r.matrix.len(), 2); // 2 rows
        assert_eq!(r.matrix[0].len(), 2); // 2 cols
        assert_eq!(r.matrix[0][0], 1.0); // row 1, col 0
        assert_eq!(r.matrix[1][1], 1.0); // row 3, col 2
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(4, false).unwrap();
        let types = vec![false, false, true, true];
        let r = get_biadjacency_matrix(&g, &types).unwrap();
        assert_eq!(r.matrix, vec![vec![0.0, 0.0], vec![0.0, 0.0]]);
    }

    #[test]
    fn self_loop_ignored() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 0), (0, 2)]).unwrap();
        let types = vec![false, false, true];
        let r = get_biadjacency_matrix(&g, &types).unwrap();
        // Self-loop (0,0) is same-partition → ignored
        assert_eq!(r.matrix[0][0], 1.0); // edge (0,2)
        assert_eq!(r.matrix[1][0], 0.0);
    }
}
