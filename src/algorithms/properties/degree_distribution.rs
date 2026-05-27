//! Degree distribution histogram (ALGO-PR-052).
//!
//! For each degree value `d` from 0 to `max_degree`, counts how many
//! vertices have exactly that degree. Supports directed graphs via
//! in-degree / out-degree / total-degree selection.

use crate::algorithms::properties::degree::DegreeMode;
use crate::core::{Graph, IgraphResult};

/// Compute the degree distribution histogram of a graph.
///
/// Returns a `Vec<u32>` of length `max_degree + 1` where entry `i`
/// is the number of vertices with degree `i`.
///
/// - `mode`: which degree to use (`Out`, `In`, or `All`). For
///   undirected graphs, all modes are equivalent.
/// - `loops`: if `true`, self-loops add 2 to the degree of their
///   endpoint (matching igraph convention for undirected). If `false`,
///   self-loops are still counted once per endpoint in directed mode.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, degree_distribution};
/// use rust_igraph::DegreeMode;
///
/// // Triangle (K3): all vertices have degree 2.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let hist = degree_distribution(&g, DegreeMode::All).unwrap();
/// // hist = [0, 0, 3] — zero vertices with degree 0 or 1, three with degree 2.
/// assert_eq!(hist, vec![0, 0, 3]);
/// ```
pub fn degree_distribution(graph: &Graph, mode: DegreeMode) -> IgraphResult<Vec<u32>> {
    let n = graph.vcount();

    if n == 0 {
        return Ok(Vec::new());
    }

    let n_usize = n as usize;
    let ecount = graph.ecount();
    let directed = graph.is_directed();

    let mut deg = vec![0u32; n_usize];

    if ecount == 0 {
        return Ok(vec![n; 1]);
    }

    let m_u32 = u32::try_from(ecount)
        .map_err(|_| crate::core::IgraphError::Internal("ecount exceeds u32::MAX"))?;

    for eid in 0..m_u32 {
        let (from, to) = graph.edge(eid)?;
        let f = from as usize;
        let t = to as usize;

        if directed {
            match mode {
                DegreeMode::Out => {
                    deg[f] += 1;
                }
                DegreeMode::In => {
                    deg[t] += 1;
                }
                DegreeMode::All => {
                    deg[f] += 1;
                    deg[t] += 1;
                }
            }
        } else if from == to {
            deg[f] += 2;
        } else {
            deg[f] += 1;
            deg[t] += 1;
        }
    }

    let max_deg = deg.iter().copied().max().unwrap_or(0) as usize;
    let mut hist = vec![0u32; max_deg + 1];

    for &d in &deg {
        hist[d as usize] += 1;
    }

    Ok(hist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        assert!(h.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        assert_eq!(h, vec![5]);
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        assert_eq!(h, vec![0, 0, 3]);
    }

    #[test]
    fn star_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        assert_eq!(h, vec![0, 4, 0, 0, 1]);
    }

    #[test]
    fn path_5() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        // deg = [1, 2, 2, 2, 1] → hist[0]=0, hist[1]=2, hist[2]=3
        assert_eq!(h, vec![0, 2, 3]);
    }

    #[test]
    fn directed_out_degree() {
        // 0→1, 0→2, 1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = degree_distribution(&g, DegreeMode::Out).unwrap();
        // out-deg = [2, 1, 0] → hist[0]=1, hist[1]=1, hist[2]=1
        assert_eq!(h, vec![1, 1, 1]);
    }

    #[test]
    fn directed_in_degree() {
        // 0→1, 0→2, 1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = degree_distribution(&g, DegreeMode::In).unwrap();
        // in-deg = [0, 1, 2] → hist[0]=1, hist[1]=1, hist[2]=1
        assert_eq!(h, vec![1, 1, 1]);
    }

    #[test]
    fn directed_all_degree() {
        // 0→1, 0→2, 1→2
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        // total-deg = [2, 2, 2] → hist[0]=0, hist[1]=0, hist[2]=3
        assert_eq!(h, vec![0, 0, 3]);
    }

    #[test]
    fn self_loop_undirected() {
        // 0-0, 0-1. Self-loop adds 2 to degree of 0.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        // deg(0) = 2 + 1 = 3, deg(1) = 1 → hist = [0, 1, 0, 1]
        assert_eq!(h, vec![0, 1, 0, 1]);
    }

    #[test]
    fn k4_regular() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        // All degree 3 → hist = [0, 0, 0, 4]
        assert_eq!(h, vec![0, 0, 0, 4]);
    }

    #[test]
    fn isolated_plus_edges() {
        // 5 vertices: 0-1, 2-3, 4 isolated
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let h = degree_distribution(&g, DegreeMode::All).unwrap();
        // deg = [1, 1, 1, 1, 0] → hist[0]=1, hist[1]=4
        assert_eq!(h, vec![1, 4]);
    }
}
