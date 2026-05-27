//! Joint degree matrix (ALGO-PR-050).
//!
//! Counterpart of `igraph_joint_degree_matrix()` from
//! `references/igraph/src/misc/mixing.c:528`.
//!
//! Entry `(i-1, j-1)` of the matrix counts edges (or sums edge weights)
//! between degree-`i` and degree-`j` vertices.
//! For undirected graphs the matrix is symmetric.
//! For directed graphs rows correspond to out-degree, columns to in-degree.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Compute the joint degree matrix of a graph.
///
/// Returns a matrix where entry `(d1-1, d2-1)` counts (or sums weights of)
/// edges between vertices of degree `d1` and degree `d2`.
///
/// - For undirected graphs: both endpoints contribute their total degree.
///   The matrix is symmetric. Each edge is counted once (but contributes
///   to both `[d_u-1][d_v-1]` and `[d_v-1][d_u-1]` unless `d_u == d_v`).
/// - For directed graphs: row index is source out-degree, column index is
///   target in-degree. Each edge is counted exactly once.
///
/// `weights`: optional edge weights (length must equal `ecount()`).
///
/// Returns an empty matrix for graphs with no edges.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, joint_degree_matrix};
///
/// // Triangle: all vertices have degree 2. JDM[1][1] = 3 (three edges).
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let jdm = joint_degree_matrix(&g, None).unwrap();
/// // max_degree = 2, so matrix is 2×2. Only [1][1] is nonzero.
/// assert_eq!(jdm.len(), 2);
/// assert!((jdm[1][1] - 3.0).abs() < 1e-10);
/// assert!((jdm[0][0]).abs() < 1e-10);
/// ```
pub fn joint_degree_matrix(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<Vec<f64>>> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if let Some(w) = weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "joint_degree_matrix: weights length ({}) does not match edge count ({ecount})",
                w.len()
            )));
        }
    }

    if ecount == 0 {
        return Ok(Vec::new());
    }

    let n_usize = n as usize;
    let directed = graph.is_directed();

    let m_u32 =
        u32::try_from(ecount).map_err(|_| IgraphError::Internal("ecount exceeds u32::MAX"))?;

    if directed {
        // Compute out-degree and in-degree for each vertex.
        let mut out_deg = vec![0u32; n_usize];
        let mut in_deg = vec![0u32; n_usize];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            out_deg[from as usize] += 1;
            in_deg[to as usize] += 1;
        }

        let max_out = out_deg.iter().copied().max().unwrap_or(0) as usize;
        let max_in = in_deg.iter().copied().max().unwrap_or(0) as usize;

        if max_out == 0 || max_in == 0 {
            return Ok(Vec::new());
        }

        let mut jdm = vec![vec![0.0_f64; max_in]; max_out];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            let d_out = out_deg[from as usize] as usize;
            let d_in = in_deg[to as usize] as usize;
            let w = weights.map_or(1.0, |ws| ws[eid as usize]);
            jdm[d_out - 1][d_in - 1] += w;
        }

        Ok(jdm)
    } else {
        // Compute degree for each vertex.
        let mut deg = vec![0u32; n_usize];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            deg[from as usize] += 1;
            deg[to as usize] += 1;
        }

        let max_deg = deg.iter().copied().max().unwrap_or(0) as usize;

        if max_deg == 0 {
            return Ok(Vec::new());
        }

        let mut jdm = vec![vec![0.0_f64; max_deg]; max_deg];

        for eid in 0..m_u32 {
            let (from, to) = graph.edge(eid)?;
            let d1 = deg[from as usize] as usize;
            let d2 = deg[to as usize] as usize;
            let w = weights.map_or(1.0, |ws| ws[eid as usize]);
            jdm[d1 - 1][d2 - 1] += w;
            if d1 != d2 {
                jdm[d2 - 1][d1 - 1] += w;
            }
        }

        Ok(jdm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert!(jdm.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert!(jdm.is_empty());
    }

    #[test]
    fn triangle_undirected() {
        // All degree-2. JDM is 2×2, only [1][1] = 3 (three edges between d=2 vertices).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 2);
        assert_eq!(jdm[0].len(), 2);
        assert!(close(jdm[0][0], 0.0));
        assert!(close(jdm[0][1], 0.0));
        assert!(close(jdm[1][0], 0.0));
        assert!(close(jdm[1][1], 3.0));
    }

    #[test]
    fn path_3_undirected() {
        // 0-1-2: deg = [1, 2, 1].
        // Edge (0,1): d=1 and d=2 → jdm[0][1] += 1, jdm[1][0] += 1.
        // Edge (1,2): d=2 and d=1 → jdm[1][0] += 1, jdm[0][1] += 1.
        // Result: jdm[0][1] = 2, jdm[1][0] = 2. Max degree = 2.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 2);
        assert!(close(jdm[0][0], 0.0));
        assert!(close(jdm[0][1], 2.0));
        assert!(close(jdm[1][0], 2.0));
        assert!(close(jdm[1][1], 0.0));
    }

    #[test]
    fn star_undirected() {
        // Star: 0 connected to 1,2,3. deg(0)=3, deg(1)=deg(2)=deg(3)=1.
        // Edges: (0,1), (0,2), (0,3). Each connects d=3 with d=1.
        // jdm[2][0] += 3, jdm[0][2] += 3. Matrix is 3×3.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 3);
        assert!(close(jdm[0][2], 3.0));
        assert!(close(jdm[2][0], 3.0));
        assert!(close(jdm[0][0], 0.0));
        assert!(close(jdm[1][1], 0.0));
        assert!(close(jdm[2][2], 0.0));
    }

    #[test]
    fn directed_simple() {
        // 0→1→2. out_deg = [1,1,0], in_deg = [0,1,1].
        // Edge (0→1): out_deg[0]=1, in_deg[1]=1 → jdm[0][0] += 1.
        // Edge (1→2): out_deg[1]=1, in_deg[2]=1 → jdm[0][0] += 1.
        // max_out = 1, max_in = 1. Matrix is 1×1: jdm[0][0] = 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 1);
        assert_eq!(jdm[0].len(), 1);
        assert!(close(jdm[0][0], 2.0));
    }

    #[test]
    fn directed_star() {
        // 0→1, 0→2, 0→3. out_deg = [3,0,0,0], in_deg = [0,1,1,1].
        // Edge (0→1): out_deg[0]=3, in_deg[1]=1 → jdm[2][0] += 1.
        // Edge (0→2): out_deg[0]=3, in_deg[2]=1 → jdm[2][0] += 1.
        // Edge (0→3): out_deg[0]=3, in_deg[3]=1 → jdm[2][0] += 1.
        // max_out = 3, max_in = 1. Matrix is 3×1: jdm[2][0] = 3.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 3);
        assert_eq!(jdm[0].len(), 1);
        assert!(close(jdm[2][0], 3.0));
        assert!(close(jdm[0][0], 0.0));
        assert!(close(jdm[1][0], 0.0));
    }

    #[test]
    fn weighted_undirected() {
        // 0-1 (w=3), 1-2 (w=2). deg = [1, 2, 1].
        // Edge 0-1: d1=1, d2=2 → jdm[0][1] += 3, jdm[1][0] += 3.
        // Edge 1-2: d1=2, d2=1 → jdm[1][0] += 2, jdm[0][1] += 2.
        // jdm[0][1] = 5, jdm[1][0] = 5.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let jdm = joint_degree_matrix(&g, Some(&[3.0, 2.0])).unwrap();
        assert!(close(jdm[0][1], 5.0));
        assert!(close(jdm[1][0], 5.0));
        assert!(close(jdm[0][0], 0.0));
        assert!(close(jdm[1][1], 0.0));
    }

    #[test]
    fn self_loop_undirected() {
        // 0-0 (self-loop), 0-1. deg = [3, 1] (self-loop adds 2 to degree of 0).
        // Edge (0,0): d1=3, d2=3 (same vertex) → jdm[2][2] += 1. d1==d2 so no double.
        // Edge (0,1): d1=3, d2=1 → jdm[2][0] += 1, jdm[0][2] += 1.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 3);
        assert!(close(jdm[2][2], 1.0));
        assert!(close(jdm[2][0], 1.0));
        assert!(close(jdm[0][2], 1.0));
    }

    #[test]
    fn total_equals_edge_count_undirected() {
        // For unweighted undirected: sum of JDM diagonal = edges within same-degree,
        // sum of upper triangle = edges between different-degree.
        // Total sum should equal ecount (each edge counted once in lower+diagonal,
        // but symmetric so total = 2*off_diag + diag... actually let's just
        // check sum of upper-triangle + diagonal = ecount.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap(); // cycle
        let jdm = joint_degree_matrix(&g, None).unwrap();
        // All vertices have degree 2. So jdm[1][1] = 5.
        assert!(close(jdm[1][1], 5.0));
        // Total of matrix (symmetric, same-degree so just diagonal): 5.
        let total: f64 = jdm.iter().flat_map(|r| r.iter()).sum();
        // For same-degree case, total = ecount (no double counting).
        assert!(close(total, 5.0));
    }

    #[test]
    fn weights_mismatch_errors() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(joint_degree_matrix(&g, Some(&[1.0, 2.0])).is_err());
    }

    #[test]
    fn k4_undirected() {
        // K4: all degree 3. 6 edges. JDM[2][2] = 6.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let jdm = joint_degree_matrix(&g, None).unwrap();
        assert_eq!(jdm.len(), 3);
        assert!(close(jdm[2][2], 6.0));
        assert!(close(jdm[0][0], 0.0));
    }
}
