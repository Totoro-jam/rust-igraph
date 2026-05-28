//! Joint degree distribution (ALGO-PR-045).
//!
//! Counterpart of `igraph_joint_degree_distribution()` in
//! `references/igraph/src/misc/mixing.c:852-927`.
//!
//! Computes `P_ij`: the (possibly normalized) count of connected ordered
//! vertex pairs having degrees i and j.

use crate::algorithms::properties::degree::{DegreeMode, degree_sequence};
use crate::core::{Graph, IgraphResult};

/// Compute the joint degree distribution matrix.
///
/// Returns a matrix `P` where `P[i][j]` counts (or gives the probability of)
/// a randomly chosen ordered pair of connected vertices having degrees `i`
/// and `j`.
///
/// For undirected graphs, `from_mode` and `to_mode` are forced to
/// [`DegreeMode::All`] and each edge contributes to both `P[d(u)][d(v)]`
/// and `P[d(v)][d(u)]`.
///
/// For directed graphs with `directed_neighbors = true`, only `u → v` arcs
/// are counted (incrementing `P[deg_from(u)][deg_to(v)]`). With
/// `directed_neighbors = false`, each edge also contributes in reverse.
///
/// # Parameters
///
/// * `graph` — the input graph.
/// * `from_mode` — degree mode for sources (Out/In/All). Ignored for
///   undirected graphs.
/// * `to_mode` — degree mode for targets. Ignored for undirected graphs.
/// * `directed_neighbors` — whether to treat connections as directed.
///   Ignored for undirected graphs.
/// * `normalized` — if `true`, scale matrix so entries sum to 1.0.
/// * `max_from_degree` — if `Some(k)`, clip row count to `k+1`.
/// * `max_to_degree` — if `Some(k)`, clip column count to `k+1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, joint_degree_distribution, DegreeMode};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// // Path graph: degrees are [1, 2, 2, 1]
/// let p = joint_degree_distribution(
///     &g, DegreeMode::All, DegreeMode::All, false, false, None, None,
/// ).unwrap();
/// // P[1][2] counts edges connecting degree-1 and degree-2 vertices
/// assert!(p[1][2] > 0.0);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn joint_degree_distribution(
    graph: &Graph,
    from_mode: DegreeMode,
    to_mode: DegreeMode,
    directed_neighbors: bool,
    normalized: bool,
    max_from_degree: Option<u32>,
    max_to_degree: Option<u32>,
) -> IgraphResult<Vec<Vec<f64>>> {
    let vcount = graph.vcount();
    let ecount = graph.ecount();

    // For undirected graphs, force modes.
    let (fm, tm, dir_neigh) = if graph.is_directed() {
        (from_mode, to_mode, directed_neighbors)
    } else {
        (DegreeMode::All, DegreeMode::All, false)
    };

    // Compute degree vectors.
    let deg_from = degree_sequence(graph, fm)?;
    let deg_to = if std::mem::discriminant(&fm) == std::mem::discriminant(&tm) {
        deg_from.clone()
    } else {
        degree_sequence(graph, tm)?
    };

    // Determine matrix dimensions.
    let nrow = if let Some(max) = max_from_degree {
        max as usize + 1
    } else if vcount == 0 {
        0
    } else {
        *deg_from.iter().max().unwrap_or(&0) as usize + 1
    };

    let ncol = if let Some(max) = max_to_degree {
        max as usize + 1
    } else if vcount == 0 {
        0
    } else {
        *deg_to.iter().max().unwrap_or(&0) as usize + 1
    };

    let mut matrix = vec![vec![0.0_f64; ncol]; nrow];
    let mut sum = 0.0_f64;

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;

        let from_type = deg_from[src as usize] as usize;
        let to_type = deg_to[tgt as usize] as usize;

        if from_type < nrow && to_type < ncol {
            matrix[from_type][to_type] += 1.0;
            sum += 1.0;
        }

        if !dir_neigh {
            let rev_from = deg_from[tgt as usize] as usize;
            let rev_to = deg_to[src as usize] as usize;
            if rev_from < nrow && rev_to < ncol {
                matrix[rev_from][rev_to] += 1.0;
                sum += 1.0;
            }
        }
    }

    if normalized && ecount > 0 && sum > 0.0 {
        let inv = 1.0 / sum;
        for row in &mut matrix {
            for val in row.iter_mut() {
                *val *= inv;
            }
        }
    }

    Ok(matrix)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, false).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::new(5, false).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // Max degree is 0, so matrix is 1×1 with value 0.
        assert_eq!(p.len(), 1);
        assert_eq!(p[0][0], 0.0);
    }

    #[test]
    fn single_edge_undirected() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // Both vertices have degree 1. Undirected → counts both directions.
        assert_eq!(p.len(), 2); // rows 0..1
        assert_eq!(p[1][1], 2.0); // (deg1→deg1) counted twice
    }

    #[test]
    fn single_edge_normalized() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            true,
            None,
            None,
        )
        .unwrap();
        assert_eq!(p[1][1], 1.0); // normalized: all mass at (1,1)
    }

    #[test]
    fn path_graph() {
        // 0-1-2-3: degrees [1,2,2,1]
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // Edge (0,1): forward p[1][2]+=1, reverse p[2][1]+=1
        // Edge (1,2): forward p[2][2]+=1, reverse p[2][2]+=1
        // Edge (2,3): forward p[2][1]+=1, reverse p[1][2]+=1
        assert_eq!(p[1][2], 2.0);
        assert_eq!(p[2][1], 2.0);
        assert_eq!(p[2][2], 2.0);
    }

    #[test]
    fn directed_out_in() {
        // 0→1→2: out-degrees [1,1,0], in-degrees [0,1,1]
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0, 1), (1, 2)]).unwrap();
        let p =
            joint_degree_distribution(&g, DegreeMode::Out, DegreeMode::In, true, false, None, None)
                .unwrap();
        // Edge 0→1: from_deg=out(0)=1, to_deg=in(1)=1 → p[1][1] += 1
        // Edge 1→2: from_deg=out(1)=1, to_deg=in(2)=1 → p[1][1] += 1
        assert_eq!(p[1][1], 2.0);
    }

    #[test]
    fn max_degree_clips() {
        let mut g = Graph::new(4, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 3)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            Some(1),
            Some(1),
        )
        .unwrap();
        // Matrix is 2×2 (indices 0..=1)
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].len(), 2);
        // Only p[1][1] from edges between deg-1 vertices: edges (0,1) and (2,3)
        // where both endpoints have deg≤1... actually vertex 1 has deg 2 which is clipped
        // edge (0,1): from_deg=1, to_deg=2 → 2 >= nrow=2, skipped in forward
        //            reverse: from_deg=2, to_deg=1 → 2 >= nrow=2, skipped
        // edge (1,2): both deg=2, both >= nrow=2, skipped both ways
        // edge (2,3): from_deg=2, to_deg=1 → 2 >= nrow=2, skipped forward
        //            reverse: from_deg=1, to_deg=2 → 2 >= ncol=2, skipped
        // So p[1][1] = 0 — only deg-1-to-deg-1 direct connections exist (none in this graph)
        assert_eq!(p[1][1], 0.0);
    }

    #[test]
    fn triangle_undirected() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // All vertices have degree 2. Each of 3 edges counted twice → p[2][2] = 6
        assert_eq!(p[2][2], 6.0);
    }

    #[test]
    fn triangle_normalized() {
        let mut g = Graph::new(3, false).unwrap();
        g.add_edges(vec![(0, 1), (1, 2), (2, 0)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            true,
            None,
            None,
        )
        .unwrap();
        assert_eq!(p[2][2], 1.0);
    }

    #[test]
    fn directed_not_directed_neighbors() {
        // 0→1: with directed_neighbors=false, counts both directions
        let mut g = Graph::new(2, true).unwrap();
        g.add_edges(vec![(0, 1)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::Out,
            DegreeMode::In,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // out-degrees: [1, 0], in-degrees: [0, 1]
        // Forward: p[out(0)=1][in(1)=1] += 1 → p[1][1] = 1
        // Reverse: p[out(1)=0][in(0)=0] += 1 → p[0][0] = 1
        assert_eq!(p[1][1], 1.0);
        assert_eq!(p[0][0], 1.0);
    }

    #[test]
    fn self_loop_undirected() {
        let mut g = Graph::new(2, false).unwrap();
        g.add_edges(vec![(0, 0), (0, 1)]).unwrap();
        let p = joint_degree_distribution(
            &g,
            DegreeMode::All,
            DegreeMode::All,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        // degrees: vertex 0 has degree 3 (loop=2 + edge=1), vertex 1 has degree 1
        // edge (0,0): both endpoints are vertex 0 with deg 3
        //   forward: p[3][3] += 1, reverse: p[3][3] += 1 → 2
        // edge (0,1): deg 3 and deg 1
        //   forward: p[3][1] += 1, reverse: p[1][3] += 1
        assert_eq!(p[3][3], 2.0);
        assert_eq!(p[3][1], 1.0);
        assert_eq!(p[1][3], 1.0);
    }
}
