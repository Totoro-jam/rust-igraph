//! Maximum cut algorithms (ALGO-FL-002).
//!
//! A cut of a graph partitions the vertices into two disjoint sets
//! (S, V\S). The cut value is the number (or total weight) of edges
//! crossing the partition. Finding a maximum cut is NP-hard; we
//! provide a greedy 0.5-approximation.
//!
//! The greedy algorithm processes vertices in order, assigning each
//! to the side that maximizes the number of crossing edges. This
//! guarantees a cut of at least |E|/2.

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of [`maximum_cut`].
#[derive(Debug, Clone)]
pub struct MaxCutResult {
    /// The partition: `partition[v]` is `true` if vertex `v` is in
    /// set S, `false` if in set V\S.
    pub partition: Vec<bool>,
    /// Number of edges crossing the cut.
    pub cut_value: usize,
}

/// Compute the cut value for a given vertex partition.
///
/// Returns the number of edges with exactly one endpoint in each
/// side of the partition. For directed graphs, counts edges
/// crossing in either direction.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cut_value};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(0, 3).unwrap();
///
/// // Partition {0,1} vs {2,3}: edges 1-2 and 0-3 cross → value = 2
/// let partition = vec![true, true, false, false];
/// assert_eq!(cut_value(&g, &partition).unwrap(), 2);
/// ```
#[allow(clippy::cast_possible_truncation)]
pub fn cut_value(graph: &Graph, partition: &[bool]) -> IgraphResult<usize> {
    let ecount = graph.ecount();
    let mut value = 0usize;

    for eid in 0..ecount as u32 {
        let (u, v) = graph.edge(eid)?;
        if partition[u as usize] != partition[v as usize] {
            value = value.saturating_add(1);
        }
    }

    Ok(value)
}

/// Compute an approximate maximum cut using a greedy heuristic.
///
/// Processes vertices in order. Each vertex is assigned to the side
/// (S or V\S) that maximizes the number of edges crossing the cut.
/// The result is guaranteed to have a cut value of at least |E|/2.
///
/// Self-loops never cross a cut (both endpoints are the same vertex).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, maximum_cut, cut_value};
///
/// // Complete graph K4: max cut is 4 (partition {0,2} vs {1,3})
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// let result = maximum_cut(&g).unwrap();
/// assert!(result.cut_value >= 3); // at least |E|/2 = 3
/// assert_eq!(result.cut_value, cut_value(&g, &result.partition).unwrap());
/// ```
#[allow(clippy::cast_possible_truncation)]
pub fn maximum_cut(graph: &Graph) -> IgraphResult<MaxCutResult> {
    let n = graph.vcount();
    let directed = graph.is_directed();

    if n == 0 {
        return Ok(MaxCutResult {
            partition: Vec::new(),
            cut_value: 0,
        });
    }

    let mut partition = vec![false; n as usize];

    // Precompute adjacency: for each vertex, its neighbors (both directions for directed)
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n as usize);
    for v in 0..n {
        let mut neighbors = graph.neighbors(v)?;
        if directed {
            let in_n = graph.in_neighbors_vec(v)?;
            for w in in_n {
                if !neighbors.contains(&w) {
                    neighbors.push(w);
                }
            }
        }
        adj.push(neighbors);
    }

    // Greedy: for each vertex, count how many already-placed neighbors
    // are in S vs V\S. Place vertex on the side with fewer neighbors
    // (to maximize crossing edges).
    for v in 0..n {
        let mut count_in_s = 0u32;
        let mut count_in_complement = 0u32;

        for &w in &adj[v as usize] {
            if w == v || w >= v {
                continue;
            }
            if partition[w as usize] {
                count_in_s = count_in_s.saturating_add(1);
            } else {
                count_in_complement = count_in_complement.saturating_add(1);
            }
        }

        partition[v as usize] = count_in_complement > count_in_s;
    }

    let value = cut_value(graph, &partition)?;

    Ok(MaxCutResult {
        partition,
        cut_value: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let r = maximum_cut(&g).unwrap();
        assert!(r.partition.is_empty());
        assert_eq!(r.cut_value, 0);
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let r = maximum_cut(&g).unwrap();
        assert_eq!(r.cut_value, 0);
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert_eq!(r.cut_value, 1);
        assert_ne!(r.partition[0], r.partition[1]);
    }

    #[test]
    fn path_4() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert_eq!(r.cut_value, cut_value(&g, &r.partition).unwrap());
        assert!(r.cut_value >= 1); // at least |E|/2 = 1
    }

    #[test]
    fn triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 1); // at least |E|/2 = 1
        assert_eq!(r.cut_value, 2); // optimal for triangle
    }

    #[test]
    fn complete_k4() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 3); // at least |E|/2 = 3
        assert_eq!(r.cut_value, cut_value(&g, &r.partition).unwrap());
    }

    #[test]
    fn bipartite_k22() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 2); // at least |E|/2 = 2
        // Bipartite optimal: all 4 edges cross
        assert_eq!(r.cut_value, 4);
    }

    #[test]
    fn star_graph() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 2); // at least |E|/2 = 2
        assert_eq!(r.cut_value, 4); // star: all 4 edges cross
    }

    #[test]
    fn self_loop_ignored() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert_eq!(r.cut_value, 1); // self-loop doesn't cross
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 1);
        assert_eq!(r.cut_value, cut_value(&g, &r.partition).unwrap());
    }

    #[test]
    fn cut_value_all_same_side() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let partition = vec![true, true, true];
        assert_eq!(cut_value(&g, &partition).unwrap(), 0);
    }

    #[test]
    fn cut_value_alternating() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let partition = vec![true, false, true, false];
        assert_eq!(cut_value(&g, &partition).unwrap(), 3);
    }

    #[test]
    fn guarantee_half_edges() {
        // Cycle of 6: |E| = 6, guarantee ≥ 3
        let mut g = Graph::with_vertices(6);
        for i in 0..6u32 {
            g.add_edge(i, (i + 1) % 6).unwrap();
        }
        let r = maximum_cut(&g).unwrap();
        assert!(r.cut_value >= 3);
    }

    #[test]
    fn parallel_edges() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let r = maximum_cut(&g).unwrap();
        assert_eq!(r.cut_value, 3); // all parallel edges cross
    }
}
