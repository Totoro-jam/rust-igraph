//! Random Walk Positional Encoding (ALGO-TR-010).
//!
//! Computes the diagonal entries of powers of the random walk transition
//! matrix `P = D⁻¹A`, where `D` is the degree matrix and `A` is the
//! adjacency matrix. For each vertex `v`, the k-th entry is the probability
//! that a random walk of length `k` starting at `v` returns to `v`.
//!
//! Used as positional encodings in Graph Transformers (e.g., `GraphGPS`,
//! `SAN`, `SignNet`) as a structural characterization of each node's local
//! topology.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute Random Walk Positional Encoding for all vertices.
///
/// For each vertex, returns a vector of `k_steps` values where entry `i`
/// is `P(walk of length i+1 returns to start)`. This is the diagonal of
/// `(D⁻¹A)^(i+1)`.
///
/// # Parameters
///
/// - `graph` — The input graph (undirected).
/// - `k_steps` — Number of walk lengths to compute (1 through `k_steps`).
///
/// # Returns
///
/// A `Vec<Vec<f64>>` of shape `[vcount][k_steps]`. Entry `[v][k]` is the
/// return probability for vertex `v` at walk length `k+1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rwpe};
///
/// // Triangle: each vertex has degree 2, connected to the other two.
/// // At step 1: from any vertex, probability of return is 0 (must leave).
/// // At step 2: from any vertex, each neighbor has prob 0.5 to go back.
/// //   P(return at step 2) = 0.5 * 0.5 + 0.5 * 0.5 = 0.5
/// let g = Graph::from_edges(&[(0,1),(1,2),(0,2)], false, Some(3)).unwrap();
/// let pe = rwpe(&g, 2).unwrap();
/// assert_eq!(pe.len(), 3);
/// assert!((pe[0][0] - 0.0).abs() < 1e-10); // step 1: no self-loop
/// assert!((pe[0][1] - 0.5).abs() < 1e-10); // step 2: return prob = 0.5
/// ```
pub fn rwpe(graph: &Graph, k_steps: usize) -> IgraphResult<Vec<Vec<f64>>> {
    let nv = graph.vcount() as usize;

    if k_steps == 0 {
        return Ok(vec![Vec::new(); nv]);
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "rwpe is defined for undirected graphs only".to_string(),
        ));
    }

    // Compute degrees and transition probabilities
    let mut degrees: Vec<usize> = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    // For each vertex, compute return probabilities via explicit
    // probability propagation (sparse matrix-vector product).
    let mut result: Vec<Vec<f64>> = Vec::with_capacity(nv);

    for src in 0..nv {
        let mut pe = Vec::with_capacity(k_steps);

        // prob[v] = probability that the walk is currently at vertex v
        let mut prob: Vec<f64> = vec![0.0; nv];
        prob[src] = 1.0;

        for _ in 0..k_steps {
            let mut next_prob: Vec<f64> = vec![0.0; nv];

            for v in 0..nv {
                if prob[v] == 0.0 {
                    continue;
                }
                let deg = degrees[v];
                if deg == 0 {
                    // Isolated vertex: walk stays in place (absorbing)
                    next_prob[v] += prob[v];
                    continue;
                }
                let transition = prob[v] / deg as f64;
                let neighbors = graph.neighbors(v as VertexId)?;
                for &nei in &neighbors {
                    next_prob[nei as usize] += transition;
                }
            }

            pe.push(next_prob[src]);
            prob = next_prob;
        }

        result.push(pe);
    }

    Ok(result)
}

/// Compute RWPE only for specified vertices (more efficient for batches).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, rwpe_vertices};
///
/// let g = Graph::from_edges(&[(0,1),(1,2),(2,3),(3,0)], false, Some(4)).unwrap();
/// let pe = rwpe_vertices(&g, &[0, 2], 3).unwrap();
/// assert_eq!(pe.len(), 2);
/// assert_eq!(pe[0].len(), 3);
/// ```
pub fn rwpe_vertices(
    graph: &Graph,
    vertices: &[VertexId],
    k_steps: usize,
) -> IgraphResult<Vec<Vec<f64>>> {
    let nv = graph.vcount() as usize;

    if k_steps == 0 {
        return Ok(vec![Vec::new(); vertices.len()]);
    }

    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "rwpe is defined for undirected graphs only".to_string(),
        ));
    }

    for &v in vertices {
        if v >= graph.vcount() {
            return Err(IgraphError::VertexOutOfRange {
                id: v,
                n: graph.vcount(),
            });
        }
    }

    let mut degrees: Vec<usize> = Vec::with_capacity(nv);
    for v in 0..nv {
        degrees.push(graph.degree(v as VertexId)?);
    }

    let mut result: Vec<Vec<f64>> = Vec::with_capacity(vertices.len());

    for &src in vertices {
        let mut pe = Vec::with_capacity(k_steps);
        let mut prob: Vec<f64> = vec![0.0; nv];
        prob[src as usize] = 1.0;

        for _ in 0..k_steps {
            let mut next_prob: Vec<f64> = vec![0.0; nv];

            for v in 0..nv {
                if prob[v] == 0.0 {
                    continue;
                }
                let deg = degrees[v];
                if deg == 0 {
                    next_prob[v] += prob[v];
                    continue;
                }
                let transition = prob[v] / deg as f64;
                let neighbors = graph.neighbors(v as VertexId)?;
                for &nei in &neighbors {
                    next_prob[nei as usize] += transition;
                }
            }

            pe.push(next_prob[src as usize]);
            prob = next_prob;
        }

        result.push(pe);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (0, 2)], false, Some(3)).unwrap()
    }

    fn cycle4() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 0)], false, Some(4)).unwrap()
    }

    fn path3() -> Graph {
        Graph::from_edges(&[(0, 1), (1, 2)], false, Some(3)).unwrap()
    }

    #[test]
    fn triangle_step1_no_return() {
        let g = triangle();
        let pe = rwpe(&g, 1).unwrap();
        for row in &pe {
            assert!((row[0] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn triangle_step2_return() {
        let g = triangle();
        let pe = rwpe(&g, 2).unwrap();
        for row in &pe {
            assert!((row[1] - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn cycle4_return_probs() {
        let g = cycle4();
        let pe = rwpe(&g, 4).unwrap();
        // In 4-cycle, each vertex has degree 2.
        // Step 1: cannot return (no self-loops)
        assert!((pe[0][0] - 0.0).abs() < 1e-10);
        // Step 2: from vertex 0, go to 1 or 3 (prob 0.5 each).
        //   From 1: prob 0.5 to go back to 0. From 3: prob 0.5 to go to 0.
        //   Return = 0.5*0.5 + 0.5*0.5 = 0.5
        assert!((pe[0][1] - 0.5).abs() < 1e-10);
        // Step 3: cannot return (odd length, bipartite cycle)
        assert!((pe[0][2] - 0.0).abs() < 1e-10);
        // Step 4: should be positive
        assert!(pe[0][3] > 0.0);
    }

    #[test]
    fn isolated_vertex() {
        let g = Graph::with_vertices(3);
        let pe = rwpe(&g, 3).unwrap();
        for row in &pe {
            for &val in row {
                assert!((val - 1.0).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn path_endpoints_differ() {
        let g = path3(); // 0-1-2
        let pe = rwpe(&g, 3).unwrap();
        // Vertex 0 (degree 1) vs vertex 1 (degree 2) should differ
        assert_ne!(pe[0], pe[1]);
        // Endpoints should be symmetric
        assert_eq!(pe[0], pe[2]);
    }

    #[test]
    fn zero_steps() {
        let g = triangle();
        let pe = rwpe(&g, 0).unwrap();
        assert_eq!(pe.len(), 3);
        for row in &pe {
            assert!(row.is_empty());
        }
    }

    #[test]
    fn directed_error() {
        let g = Graph::from_edges(&[(0, 1), (1, 2)], true, Some(3)).unwrap();
        assert!(rwpe(&g, 2).is_err());
    }

    #[test]
    fn rwpe_vertices_subset() {
        let g = cycle4();
        let full = rwpe(&g, 3).unwrap();
        let subset = rwpe_vertices(&g, &[0, 2], 3).unwrap();
        assert_eq!(subset.len(), 2);
        assert_eq!(subset[0], full[0]);
        assert_eq!(subset[1], full[2]);
    }

    #[test]
    fn rwpe_vertices_invalid() {
        let g = cycle4();
        assert!(rwpe_vertices(&g, &[10], 2).is_err());
    }

    #[test]
    fn probabilities_bounded() {
        let g = cycle4();
        let pe = rwpe(&g, 10).unwrap();
        for row in &pe {
            for &val in row {
                assert!(val >= 0.0);
                assert!(val <= 1.0);
            }
        }
    }

    #[test]
    fn all_vertices_same_in_regular_graph() {
        let g = cycle4();
        let pe = rwpe(&g, 5).unwrap();
        for row in pe.iter().skip(1) {
            for (k, &val) in row.iter().enumerate() {
                assert!((pe[0][k] - val).abs() < 1e-10);
            }
        }
    }
}
