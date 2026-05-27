//! Mutual (reciprocal) edge detection (ALGO-PR-038).
//!
//! For directed graphs, an edge (A, B) is *mutual* if (B, A) also
//! exists. For undirected graphs, all edges are mutual by definition.
//!
//! Counterpart of `igraph_is_mutual` and `igraph_has_mutual` from
//! `references/igraph/src/properties/multiplicity.c`.

use crate::core::{Graph, IgraphResult, VertexId};

/// Check whether each edge in a directed graph is mutual (reciprocated).
///
/// An edge (A, B) is mutual if the reverse edge (B, A) also exists.
/// Self-loops are considered mutual if `loops` is true.
///
/// For undirected graphs, all edges are considered mutual.
///
/// Returns a boolean vector with one entry per edge.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_mutual};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 0).unwrap(); // edge 1
/// g.add_edge(1, 2).unwrap(); // edge 2
/// let mutual = is_mutual(&g, true).unwrap();
/// assert_eq!(mutual, vec![true, true, false]);
/// ```
pub fn is_mutual(graph: &Graph, loops: bool) -> IgraphResult<Vec<bool>> {
    let ecount = graph.ecount();

    if !graph.is_directed() {
        return Ok(vec![true; ecount]);
    }

    let n = graph.vcount() as usize;
    let mut out_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        out_adj[from as usize].push(to);
    }
    for adj in &mut out_adj {
        adj.sort_unstable();
    }

    let mut result = Vec::with_capacity(ecount);

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        if from == to {
            result.push(loops);
        } else {
            // Check if reverse edge exists: is `from` in out-neighbors of `to`?
            let has_reverse = out_adj[to as usize].binary_search(&from).is_ok();
            result.push(has_reverse);
        }
    }

    Ok(result)
}

/// Check whether a directed graph has any mutual (reciprocal) edges.
///
/// For undirected graphs, returns true if there is at least one edge
/// (all edges are mutual by definition).
///
/// Self-loops are considered mutual if `loops` is true.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, has_mutual};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(!has_mutual(&g, false).unwrap());
///
/// g.add_edge(1, 0).unwrap();
/// assert!(has_mutual(&g, false).unwrap());
/// ```
pub fn has_mutual(graph: &Graph, loops: bool) -> IgraphResult<bool> {
    let ecount = graph.ecount();

    if !graph.is_directed() {
        return Ok(ecount > 0);
    }

    let n = graph.vcount() as usize;
    let mut out_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        out_adj[from as usize].push(to);
    }
    for adj in &mut out_adj {
        adj.sort_unstable();
    }

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        if from == to {
            if loops {
                return Ok(true);
            }
        } else if out_adj[to as usize].binary_search(&from).is_ok() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Count the number of mutual edge pairs in a directed graph.
///
/// Each pair of reciprocal edges (A→B and B→A) is counted once.
/// Self-loops are counted if `loops` is true.
///
/// For undirected graphs, returns the number of edges (all are mutual).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, count_mutual};
///
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 0).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 1).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert_eq!(count_mutual(&g, false).unwrap(), 2);
/// ```
pub fn count_mutual(graph: &Graph, loops: bool) -> IgraphResult<usize> {
    let ecount = graph.ecount();

    if !graph.is_directed() {
        return Ok(ecount);
    }

    let n = graph.vcount() as usize;
    let mut out_adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        out_adj[from as usize].push(to);
    }
    for adj in &mut out_adj {
        adj.sort_unstable();
    }

    let mut count: usize = 0;

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;

        if from == to {
            if loops {
                count += 1;
            }
        } else if from < to && out_adj[to as usize].binary_search(&from).is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_mutual_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = is_mutual(&g, true).unwrap();
        assert_eq!(result, vec![true, true]);
    }

    #[test]
    fn is_mutual_directed_reciprocal() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = is_mutual(&g, true).unwrap();
        assert_eq!(result, vec![true, true, false]);
    }

    #[test]
    fn is_mutual_self_loop_true() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let result = is_mutual(&g, true).unwrap();
        assert_eq!(result, vec![true, false]);
    }

    #[test]
    fn is_mutual_self_loop_false() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        let result = is_mutual(&g, false).unwrap();
        assert_eq!(result, vec![false, false]);
    }

    #[test]
    fn is_mutual_empty() {
        let g = Graph::new(3, true).unwrap();
        let result = is_mutual(&g, true).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn has_mutual_no_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!has_mutual(&g, false).unwrap());
    }

    #[test]
    fn has_mutual_with_mutual() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert!(has_mutual(&g, false).unwrap());
    }

    #[test]
    fn has_mutual_undirected_nonempty() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(has_mutual(&g, false).unwrap());
    }

    #[test]
    fn has_mutual_undirected_empty() {
        let g = Graph::with_vertices(3);
        assert!(!has_mutual(&g, false).unwrap());
    }

    #[test]
    fn has_mutual_self_loop_counted() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        assert!(has_mutual(&g, true).unwrap());
        assert!(!has_mutual(&g, false).unwrap());
    }

    #[test]
    fn count_mutual_basic() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(count_mutual(&g, false).unwrap(), 2);
    }

    #[test]
    fn count_mutual_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_mutual(&g, false).unwrap(), 0);
    }

    #[test]
    fn count_mutual_all() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(count_mutual(&g, false).unwrap(), 3);
    }

    #[test]
    fn count_mutual_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert_eq!(count_mutual(&g, false).unwrap(), 2);
    }

    #[test]
    fn count_mutual_with_self_loops() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        assert_eq!(count_mutual(&g, true).unwrap(), 2); // self-loop + (0,1)/(1,0) pair
        assert_eq!(count_mutual(&g, false).unwrap(), 1); // only (0,1)/(1,0)
    }
}
