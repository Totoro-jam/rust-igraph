//! Split graph predicate (ALGO-PR-072).
//!
//! A split graph is a graph whose vertices can be partitioned into a
//! clique and an independent set. Recognition is polynomial via the
//! Hammer-Simeone theorem (1977): sort the degree sequence in
//! non-increasing order, find `m = max{i : d_i >= i-1}`, then the
//! graph is split iff `sum(d_1..d_m) = m(m-1) + sum(d_{m+1}..d_n)`.
//!
//! This characterization applies to simple undirected graphs. For
//! non-simple graphs (with self-loops or parallel edges) or directed
//! graphs, the function returns `false`.

use crate::algorithms::properties::degree::{DegreeMode, degree_sequence};
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a split graph.
///
/// A split graph can partition its vertices into a clique C and an
/// independent set I. Uses the Hammer-Simeone degree-sequence
/// characterization for O(V + E) recognition.
///
/// Returns `false` for directed graphs, multigraphs, or graphs with
/// self-loops (the characterization requires simple undirected graphs).
///
/// An empty graph (0 vertices) is a split graph (vacuously).
/// An edgeless graph is a split graph (C = empty, I = all vertices).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_split_graph};
///
/// // Path P3 (0-1-2): split as C={1}, I={0,2}
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_split_graph(&g).unwrap());
///
/// // Cycle C5 is NOT a split graph
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_split_graph(&g).unwrap());
/// ```
pub fn is_split_graph(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount() as usize;
    if n == 0 {
        return Ok(true);
    }

    if !is_simple(graph)? {
        return Ok(false);
    }

    let mut degrees = degree_sequence(graph, DegreeMode::All)?;
    degrees.sort_unstable_by(|a, b| b.cmp(a));

    let m = find_m(&degrees);
    if m == 0 {
        return Ok(true);
    }

    let sum_top: u64 = degrees[..m].iter().map(|&d| u64::from(d)).sum();
    let sum_bottom: u64 = degrees[m..].iter().map(|&d| u64::from(d)).sum();
    let m64 = m as u64;
    let target = m64
        .saturating_mul(m64.saturating_sub(1))
        .saturating_add(sum_bottom);

    Ok(sum_top == target)
}

fn find_m(sorted_desc: &[u32]) -> usize {
    let mut m = 0usize;
    for (i, &d) in sorted_desc.iter().enumerate() {
        let idx = u32::try_from(i).unwrap_or(u32::MAX);
        if d >= idx {
            m = i.saturating_add(1);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn edgeless_graph() {
        let g = Graph::with_vertices(5);
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn path_3_is_split() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn triangle_is_split() {
        // K3: C={0,1,2}, I={}
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn complete_k4_is_split() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn star_is_split() {
        // Star: C={center}, I={leaves}
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn cycle_4_is_split() {
        // C4 is a split graph: C={0,2}, I={1,3} (if 0-2 are adjacent)
        // Wait: C4 = 0-1-2-3-0. Vertices {0,2} are NOT adjacent.
        // So C4 is NOT split because neither {0,2} nor any 2-subset
        // forms a clique that leaves an IS.
        // Actually C4 IS a split graph: C={0,1}, I={2,3}
        // But 0-1 is an edge, so C={0,1} is a clique.
        // I={2,3}: 2-3 is an edge! So not an IS.
        // C={1,2}: 1-2 is an edge. I={0,3}: 3-0 is an edge. Not IS.
        // C4 has no split partition → it's NOT split.
        // Verify with Hammer-Simeone: degrees [2,2,2,2], m=2
        // (d_1=2>=0, d_2=2>=1, d_3=2>=2? yes), so m=3?
        // Actually m = max{i : d_i >= i-1} where i is 1-indexed:
        // i=1: d_1=2>=0 yes; i=2: d_2=2>=1 yes; i=3: d_3=2>=2 yes;
        // i=4: d_4=2>=3? no. So m=3.
        // sum_top = 2+2+2 = 6
        // target = 3*2 + 2 = 8 ≠ 6 → NOT split. Correct!
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn cycle_5_not_split() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn k4_minus_edge_is_split() {
        // K4 minus one edge: e.g., all K4 edges except (2,3)
        // degrees: [3,3,2,2], sorted desc [3,3,2,2]
        // m: i=1: 3>=0 yes; i=2: 3>=1 yes; i=3: 2>=2 yes; i=4: 2>=3 no. m=3
        // sum_top = 3+3+2 = 8, target = 3*2 + 2 = 8 → split!
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        // no edge (2,3)
        assert!(is_split_graph(&g).unwrap());
    }

    #[test]
    fn bipartite_k23_not_split() {
        // K_{2,3}: degrees [3,3,2,2,2]
        // m: i=1: 3>=0; i=2: 3>=1; i=3: 2>=2; i=4: 2>=3? no. m=3
        // sum_top = 3+3+2 = 8, target = 3*2 + 2+2 = 10 ≠ 8 → not split
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn self_loop_returns_false() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn parallel_edges_returns_false() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_split_graph(&g).unwrap());
    }

    #[test]
    fn threshold_graph_is_split() {
        // Threshold graph: start with K1, repeatedly add isolated or
        // dominating vertex. E.g., {0}, add 1 dominating: K2.
        // Add 2 isolated, add 3 dominating to {0,1,2}: 3-0, 3-1, 3-2.
        // degrees: [3,2,1,2], sorted [3,2,2,1]
        // m: i=1: 3>=0; i=2: 2>=1; i=3: 2>=2; i=4: 1>=3? no. m=3
        // sum_top=7, target=3*2+1=7 → split!
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 1).unwrap();
        g.add_edge(3, 2).unwrap();
        assert!(is_split_graph(&g).unwrap());
    }
}
