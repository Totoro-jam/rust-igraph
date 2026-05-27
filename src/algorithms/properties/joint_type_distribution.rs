//! Joint type distribution / mixing matrix (ALGO-PR-047).
//!
//! Computes the mixing matrix for a graph given vertex type labels.
//! Entry (i,j) counts (or gives the probability of) edges going from
//! type i to type j.
//!
//! Counterpart of `igraph_joint_type_distribution` from
//! `references/igraph/src/misc/mixing.c`.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the joint type distribution (mixing matrix).
///
/// Given vertex type labels, produces a matrix where entry (i,j) is the
/// count (or probability, if `normalized`) of edges from type-i to type-j
/// vertices.
///
/// - `from_types`: type label for source endpoint of each vertex (length = vcount).
/// - `to_types`: type label for target endpoint. If `None`, uses `from_types`
///   for both endpoints.
/// - `directed`: if true and graph is directed, count each edge once
///   (from→to); otherwise count each edge in both directions.
/// - `normalized`: if true, divide by total weight so entries sum to 1.
/// - `weights`: optional edge weights (length must equal ecount).
///
/// Returns a row-major matrix as `Vec<Vec<f64>>` with dimensions
/// `(max_from_type+1) x (max_to_type+1)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, joint_type_distribution};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap(); // type 0 - type 0
/// g.add_edge(2, 3).unwrap(); // type 1 - type 1
/// let types = vec![0u32, 0, 1, 1];
/// let m = joint_type_distribution(&g, &types, None, false, true, None).unwrap();
/// // Normalized undirected: [[0.5, 0], [0, 0.5]]
/// assert!((m[0][0] - 0.5).abs() < 1e-10);
/// assert!((m[1][1] - 0.5).abs() < 1e-10);
/// assert!((m[0][1]).abs() < 1e-10);
/// ```
pub fn joint_type_distribution(
    graph: &Graph,
    from_types: &[u32],
    to_types: Option<&[u32]>,
    directed: bool,
    normalized: bool,
    weights: Option<&[f64]>,
) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount();
    let ecount = graph.ecount();
    let same_types = to_types.is_none();
    let to_types = to_types.unwrap_or(from_types);

    #[allow(clippy::cast_possible_truncation)]
    let n_usize = n as usize;

    if from_types.len() != n_usize {
        return Err(IgraphError::InvalidArgument(format!(
            "joint_type_distribution: from_types length ({}) does not match vertex count ({n})",
            from_types.len()
        )));
    }
    if to_types.len() != n_usize {
        return Err(IgraphError::InvalidArgument(format!(
            "joint_type_distribution: to_types length ({}) does not match vertex count ({n})",
            to_types.len()
        )));
    }
    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "joint_type_distribution: weights length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    if n == 0 {
        return Ok(Vec::new());
    }

    let nrow = from_types.iter().copied().max().unwrap_or(0) as usize + 1;
    let ncol = if same_types {
        nrow
    } else {
        to_types.iter().copied().max().unwrap_or(0) as usize + 1
    };

    let directed = directed && graph.is_directed();

    let mut matrix = vec![vec![0.0_f64; ncol]; nrow];
    let mut sum = 0.0_f64;

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        let from_type = from_types[from as usize] as usize;
        let to_type = to_types[to as usize] as usize;
        let w = weights.map_or(1.0, |ws| ws[eid]);

        matrix[from_type][to_type] += w;
        sum += w;

        if !directed {
            matrix[to_type][from_type] += w;
            sum += w;
        }
    }

    if normalized && ecount > 0 {
        for row in &mut matrix {
            for val in row.iter_mut() {
                *val /= sum;
            }
        }
    }

    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let m = joint_type_distribution(&g, &[], None, false, false, None).unwrap();
        assert!(m.is_empty());
    }

    #[test]
    fn no_edges_undirected() {
        let g = Graph::with_vertices(3);
        let types = vec![0, 1, 2];
        let m = joint_type_distribution(&g, &types, None, false, false, None).unwrap();
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].len(), 3);
        for row in &m {
            for &v in row {
                assert!(approx_eq(v, 0.0));
            }
        }
    }

    #[test]
    fn perfect_assortative_undirected_normalized() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let types = vec![0, 0, 1, 1];
        let m = joint_type_distribution(&g, &types, None, false, true, None).unwrap();
        assert!(approx_eq(m[0][0], 0.5));
        assert!(approx_eq(m[1][1], 0.5));
        assert!(approx_eq(m[0][1], 0.0));
        assert!(approx_eq(m[1][0], 0.0));
    }

    #[test]
    fn cross_type_undirected_unnormalized() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap(); // type 0 → type 1
        g.add_edge(1, 3).unwrap(); // type 0 → type 1
        let types = vec![0, 0, 1, 1];
        let m = joint_type_distribution(&g, &types, None, false, false, None).unwrap();
        // Each edge counted in both directions
        assert!(approx_eq(m[0][1], 2.0));
        assert!(approx_eq(m[1][0], 2.0));
        assert!(approx_eq(m[0][0], 0.0));
        assert!(approx_eq(m[1][1], 0.0));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // type 0 → type 0
        g.add_edge(0, 2).unwrap(); // type 0 → type 1
        let types = vec![0, 0, 1];
        let m = joint_type_distribution(&g, &types, None, true, false, None).unwrap();
        assert!(approx_eq(m[0][0], 1.0));
        assert!(approx_eq(m[0][1], 1.0));
        assert!(approx_eq(m[1][0], 0.0));
        assert!(approx_eq(m[1][1], 0.0));
    }

    #[test]
    fn directed_normalized() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let types = vec![0, 0, 1];
        let m = joint_type_distribution(&g, &types, None, true, true, None).unwrap();
        assert!(approx_eq(m[0][0], 0.5));
        assert!(approx_eq(m[0][1], 0.5));
    }

    #[test]
    fn weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // type 0 - type 0
        g.add_edge(1, 2).unwrap(); // type 0 - type 1
        let types = vec![0, 0, 1];
        let weights = vec![3.0, 2.0];
        let m = joint_type_distribution(&g, &types, None, false, false, Some(&weights)).unwrap();
        // edge(0,1): matrix[0][0]+=3, undirected: matrix[0][0]+=3 → 6
        // edge(1,2): matrix[0][1]+=2, undirected: matrix[1][0]+=2
        assert!(approx_eq(m[0][0], 6.0));
        assert!(approx_eq(m[0][1], 2.0));
        assert!(approx_eq(m[1][0], 2.0));
        assert!(approx_eq(m[1][1], 0.0));
    }

    #[test]
    fn different_from_to_types() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let from_types = vec![0, 1, 0];
        let to_types = vec![0, 0, 1];
        let m =
            joint_type_distribution(&g, &from_types, Some(&to_types), true, false, None).unwrap();
        // edge(0,1): from_types[0]=0, to_types[1]=0 → m[0][0]+=1
        // edge(1,2): from_types[1]=1, to_types[2]=1 → m[1][1]+=1
        assert!(approx_eq(m[0][0], 1.0));
        assert!(approx_eq(m[1][1], 1.0));
        assert!(approx_eq(m[0][1], 0.0));
        assert!(approx_eq(m[1][0], 0.0));
    }

    #[test]
    fn types_mismatch_error() {
        let g = Graph::with_vertices(3);
        let types = vec![0, 1]; // wrong length
        assert!(joint_type_distribution(&g, &types, None, false, false, None).is_err());
    }

    #[test]
    fn weights_mismatch_error() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let types = vec![0, 1];
        assert!(
            joint_type_distribution(&g, &types, None, false, false, Some(&[1.0, 2.0])).is_err()
        );
    }
}
