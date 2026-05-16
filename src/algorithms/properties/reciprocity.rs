//! Reciprocity (ALGO-PR-004).
//!
//! Counterpart of `igraph_reciprocity()` from
//! `references/igraph/src/properties/basic_properties.c:325-406`.
//!
//! For directed graphs, reciprocity is the proportion of mutual
//! connections — formally `1 - (sum_ij |A_ij - A_ji|) / (2 sum_ij A_ij)`.
//! Equivalent to (number of edges with a reverse counterpart) / (total
//! edges). For undirected graphs it is 1.0 by definition. For graphs
//! with no edges, the value is undefined (`None` here, matching upstream's
//! `IGRAPH_NAN`).
//!
//! Phase-1 minimal slice: default mode (`IGRAPH_RECIPROCITY_DEFAULT`)
//! and `ignore_loops = false`. The `IGRAPH_RECIPROCITY_RATIO` mode and
//! `ignore_loops = true` ship as PR-004b when needed.

use crate::core::{Graph, IgraphResult};

/// Reciprocity of `graph`. Returns `None` for graphs with no edges
/// (matches upstream's `IGRAPH_NAN`).
///
/// Counterpart of `igraph_reciprocity(_, _, /*ignore_loops=*/false,
/// IGRAPH_RECIPROCITY_DEFAULT)`. For undirected graphs returns
/// `Some(1.0)` unconditionally.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reciprocity};
///
/// // Directed mutual pair: 0 -> 1, 1 -> 0. Both edges have a reverse → 1.0.
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
///
/// // One-way: 0 -> 1 only. No reverse → 0.0.
/// let mut g = Graph::new(2, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
/// ```
pub fn reciprocity(graph: &Graph) -> IgraphResult<Option<f64>> {
    let n = graph.vcount();
    let m = graph.ecount();
    if m == 0 {
        return Ok(None);
    }
    if !graph.is_directed() {
        return Ok(Some(1.0));
    }

    // Mirrors upstream's two-pointer merge over sorted in-neighbours and
    // out-neighbours per vertex.
    let mut rec: u64 = 0;

    for v in 0..n {
        let outneis = graph.out_neighbors_vec(v)?;
        let inneis = graph.in_neighbors_vec(v)?;

        let mut ip = 0usize;
        let mut op = 0usize;
        while ip < inneis.len() && op < outneis.len() {
            match inneis[ip].cmp(&outneis[op]) {
                std::cmp::Ordering::Less => ip += 1,
                std::cmp::Ordering::Greater => op += 1,
                std::cmp::Ordering::Equal => {
                    // Loop edge or genuine mutual?
                    if inneis[ip] == v {
                        // Self-loop: counted as mutual (matches upstream's
                        // ignore_loops=false branch).
                        rec += 1;
                    } else {
                        rec += 1;
                    }
                    ip += 1;
                    op += 1;
                }
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let result = (rec as f64) / (m as f64);
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_returns_none() {
        let g = Graph::with_vertices(0);
        assert_eq!(reciprocity(&g).unwrap(), None);
    }

    #[test]
    fn isolated_vertices_return_none() {
        let g = Graph::with_vertices(5);
        assert_eq!(reciprocity(&g).unwrap(), None);
    }

    #[test]
    fn undirected_graph_is_always_1() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn directed_one_way_edge_has_zero() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn directed_mutual_pair_has_one() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }

    #[test]
    fn directed_partial_reciprocity() {
        // 0 -> 1, 1 -> 0 (mutual), 0 -> 2 (one-way). 3 edges, 2 reciprocal → 2/3.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        let two_thirds = 2.0_f64 / 3.0;
        assert_eq!(reciprocity(&g).unwrap(), Some(two_thirds));
    }

    #[test]
    fn directed_3_cycle_no_reciprocity() {
        // 0 -> 1 -> 2 -> 0: each edge has no reverse → 0.0.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(0.0));
    }

    #[test]
    fn directed_self_loop_is_counted_as_mutual() {
        // 0 -> 0 self-loop: 1 edge, mutual → 1.0.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert_eq!(reciprocity(&g).unwrap(), Some(1.0));
    }
}
