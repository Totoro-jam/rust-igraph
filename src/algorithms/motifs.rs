//! Motif census (ALGO-MO-001).
//!
//! Dyad census for directed graphs: counts mutual, asymmetric, and null
//! dyads. Counterpart of `igraph_dyad_census`.

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of a dyad census on a directed graph.
#[derive(Debug, Clone, PartialEq)]
pub struct DyadCensus {
    /// Number of mutual (reciprocated) dyads.
    pub mutual: f64,
    /// Number of asymmetric (one-way) dyads.
    pub asymmetric: f64,
    /// Number of null (no edge) dyads.
    pub null: f64,
}

/// Counts the mutual, asymmetric, and null dyads in a directed graph.
///
/// A *dyad* is an unordered pair of vertices. In a directed graph:
/// - **Mutual**: both `(u,v)` and `(v,u)` edges exist.
/// - **Asymmetric**: exactly one of `(u,v)` or `(v,u)` exists.
/// - **Null**: neither edge exists.
///
/// Self-loops and multi-edges are ignored (only simple edges counted).
/// For undirected graphs, all edges are treated as mutual.
///
/// Uses `f64` to avoid overflow for large graphs where `n*(n-1)/2`
/// may exceed `u32::MAX`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, dyad_census};
///
/// let mut g = Graph::new(4, true).unwrap();
/// // Mutual: 0↔1
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// // Asymmetric: 2→3
/// g.add_edge(2, 3).unwrap();
///
/// let dc = dyad_census(&g).unwrap();
/// assert!((dc.mutual - 1.0).abs() < 1e-10);
/// assert!((dc.asymmetric - 1.0).abs() < 1e-10);
/// assert!((dc.null - 4.0).abs() < 1e-10);
/// ```
pub fn dyad_census(graph: &Graph) -> IgraphResult<DyadCensus> {
    let n = graph.vcount();

    if n < 2 {
        return Ok(DyadCensus {
            mutual: 0.0,
            asymmetric: 0.0,
            null: 0.0,
        });
    }

    if !graph.is_directed() {
        return dyad_census_undirected(graph);
    }

    let (in_adj, out_adj) = build_directed_adj(graph)?;
    let mut rec: f64 = 0.0;
    let mut nonrec: f64 = 0.0;

    for v in 0..n {
        let in_neis = &in_adj[v as usize];
        let out_neis = &out_adj[v as usize];

        let mut ip = 0;
        let mut op = 0;

        while ip < in_neis.len() && op < out_neis.len() {
            match in_neis[ip].cmp(&out_neis[op]) {
                std::cmp::Ordering::Less => {
                    nonrec += 1.0;
                    ip += 1;
                }
                std::cmp::Ordering::Greater => {
                    nonrec += 1.0;
                    op += 1;
                }
                std::cmp::Ordering::Equal => {
                    rec += 1.0;
                    ip += 1;
                    op += 1;
                }
            }
        }
        #[allow(clippy::cast_precision_loss)]
        {
            nonrec += (in_neis.len() - ip) as f64 + (out_neis.len() - op) as f64;
        }
    }

    let mutual = rec / 2.0;
    let asymmetric = nonrec / 2.0;
    #[allow(clippy::cast_precision_loss)]
    let total_dyads = 0.5 * (f64::from(n)) * ((f64::from(n)) - 1.0);
    let null = total_dyads - mutual - asymmetric;

    Ok(DyadCensus {
        mutual,
        asymmetric,
        null: if null == 0.0 { 0.0 } else { null },
    })
}

fn dyad_census_undirected(graph: &Graph) -> IgraphResult<DyadCensus> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    // Count simple edges (no self-loops, no multi-edges)
    let mut edge_set = std::collections::HashSet::new();
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            let key = if src < tgt { (src, tgt) } else { (tgt, src) };
            edge_set.insert(key);
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let mutual = edge_set.len() as f64;
    #[allow(clippy::cast_precision_loss)]
    let total_dyads = 0.5 * (f64::from(n)) * ((f64::from(n)) - 1.0);
    let null = total_dyads - mutual;

    Ok(DyadCensus {
        mutual,
        asymmetric: 0.0,
        null: if null == 0.0 { 0.0 } else { null },
    })
}

type AdjPair = (Vec<Vec<VertexId>>, Vec<Vec<VertexId>>);

/// Build sorted simple in-neighbor and out-neighbor lists for directed graph.
fn build_directed_adj(graph: &Graph) -> IgraphResult<AdjPair> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let mut in_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let mut out_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (src, tgt) = graph.edge(eid as u32)?;
        if src != tgt {
            out_adj[src as usize].push(tgt);
            in_adj[tgt as usize].push(src);
        }
    }

    for neighbors in &mut in_adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }
    for neighbors in &mut out_adj {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    Ok((in_adj, out_adj))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_single_vertex() {
        let g = Graph::new(1, true).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_two_vertices_no_edges() {
        let g = Graph::new(2, true).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_vertices_one_edge() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric - 1.0).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_two_vertices_mutual() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual - 1.0).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_three_vertices_cycle() {
        // 0→1, 1→2, 2→0: three asymmetric dyads
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric - 3.0).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_complete_directed_mutual() {
        // K4 directed with all edges mutual: 6 mutual dyads
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            for j in 0..4 {
                if i != j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual - 6.0).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_mixed() {
        // 4 vertices:
        // mutual: 0↔1
        // asymmetric: 2→3
        // null: (0,2), (0,3), (1,2), (1,3) = 4
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual - 1.0).abs() < 1e-10);
        assert!((dc.asymmetric - 1.0).abs() < 1e-10);
        assert!((dc.null - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_self_loops_ignored() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        let dc = dyad_census(&g).unwrap();
        // Only 0→1 counts as asymmetric
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric - 1.0).abs() < 1e-10);
        assert!((dc.null - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_multi_edges_deduplicated() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        let dc = dyad_census(&g).unwrap();
        // Multiple edges same direction still count as one asymmetric dyad
        assert!((dc.mutual).abs() < 1e-10);
        assert!((dc.asymmetric - 1.0).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_undirected_all_mutual() {
        // Undirected: all edges counted as mutual
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual - 3.0).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_undirected_complete() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            for j in (i + 1)..5 {
                g.add_edge(i, j).unwrap();
            }
        }
        let dc = dyad_census(&g).unwrap();
        assert!((dc.mutual - 10.0).abs() < 1e-10);
        assert!((dc.asymmetric).abs() < 1e-10);
        assert!((dc.null).abs() < 1e-10);
    }

    #[test]
    fn test_counts_sum_to_total() {
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        let dc = dyad_census(&g).unwrap();
        let total = dc.mutual + dc.asymmetric + dc.null;
        // n*(n-1)/2 = 5*4/2 = 10
        assert!((total - 10.0).abs() < 1e-10);
    }
}
