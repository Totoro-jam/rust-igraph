//! Graph reversal operators (ALGO-OP-008, ALGO-OP-025).
//!
//! Reverses edge directions — all edges or a specified subset.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Returns a new graph with all edge directions reversed.
///
/// For directed graphs, every edge `(u, v)` becomes `(v, u)`.
/// For undirected graphs, returns a structural copy (edges unchanged).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reverse};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// let rev = reverse(&g).unwrap();
/// assert!(rev.is_directed());
/// assert_eq!(rev.ecount(), 2);
/// // Edge 0→1 becomes 1→0, edge 1→2 becomes 2→1
/// ```
pub fn reverse(graph: &Graph) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let ecount = graph.ecount();

    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(ecount);

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;
        if directed {
            edges.push((tgt, src));
        } else {
            edges.push((src, tgt));
        }
    }

    let mut result = Graph::new(n, directed)?;
    result.add_edges(edges)?;
    Ok(result)
}

/// Returns a new graph with only the specified edges reversed.
///
/// For directed graphs, each edge in `eids` has its direction flipped
/// (`(u, v)` becomes `(v, u)`). Edges not listed remain unchanged.
/// For undirected graphs, this is a no-op (returns a structural copy).
///
/// Duplicate edge IDs in `eids` are silently ignored (double-reversing
/// a single edge still results in one reversal).
///
/// # Errors
///
/// Returns `InvalidArgument` if any edge ID in `eids` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reverse_edges};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap(); // eid 0
/// g.add_edge(1, 2).unwrap(); // eid 1
///
/// // Reverse only edge 0
/// let rev = reverse_edges(&g, &[0]).unwrap();
/// assert_eq!(rev.edge(0).unwrap(), (1, 0)); // reversed
/// assert_eq!(rev.edge(1).unwrap(), (1, 2)); // unchanged
/// ```
pub fn reverse_edges(graph: &Graph, eids: &[u32]) -> IgraphResult<Graph> {
    let n = graph.vcount();
    let directed = graph.is_directed();
    let ecount = graph.ecount();

    if !directed {
        // No-op for undirected graphs — just copy
        return reverse_edges_copy_undirected(graph, n, ecount);
    }

    // Build a set of edges to reverse
    let mut to_reverse: Vec<bool> = vec![false; ecount];
    for &eid in eids {
        if eid as usize >= ecount {
            return Err(IgraphError::InvalidArgument(format!(
                "reverse_edges: edge ID {eid} out of range (graph has {ecount} edges)"
            )));
        }
        to_reverse[eid as usize] = true;
    }

    let mut result = Graph::new(n, true)?;
    for (eid_usize, &rev) in to_reverse.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (src, tgt) = graph.edge(eid)?;
        if rev {
            result.add_edge(tgt, src)?;
        } else {
            result.add_edge(src, tgt)?;
        }
    }

    Ok(result)
}

fn reverse_edges_copy_undirected(graph: &Graph, n: u32, ecount: usize) -> IgraphResult<Graph> {
    let mut result = Graph::with_vertices(n);
    for eid_usize in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid = eid_usize as u32;
        let (src, tgt) = graph.edge(eid)?;
        result.add_edge(src, tgt)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_directed() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();

        let rev = reverse(&g).unwrap();
        assert!(rev.is_directed());
        assert_eq!(rev.vcount(), 4);
        assert_eq!(rev.ecount(), 3);

        // Check edges are reversed
        let (s, t) = rev.edge(0).unwrap();
        assert_eq!((s, t), (1, 0));
        let (s, t) = rev.edge(1).unwrap();
        assert_eq!((s, t), (2, 1));
        let (s, t) = rev.edge(2).unwrap();
        assert_eq!((s, t), (3, 2));
    }

    #[test]
    fn test_reverse_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rev = reverse(&g).unwrap();
        assert!(!rev.is_directed());
        assert_eq!(rev.vcount(), 3);
        assert_eq!(rev.ecount(), 2);
    }

    #[test]
    fn test_reverse_empty() {
        let g = Graph::new(5, true).unwrap();
        let rev = reverse(&g).unwrap();
        assert_eq!(rev.vcount(), 5);
        assert_eq!(rev.ecount(), 0);
    }

    #[test]
    fn test_reverse_self_loop() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();

        let rev = reverse(&g).unwrap();
        let (s, t) = rev.edge(0).unwrap();
        assert_eq!((s, t), (0, 0)); // self-loop unchanged
        let (s, t) = rev.edge(1).unwrap();
        assert_eq!((s, t), (1, 0)); // reversed
    }

    #[test]
    fn test_reverse_involution() {
        // Reversing twice gives back the original
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 0).unwrap();

        let rev2 = reverse(&reverse(&g).unwrap()).unwrap();
        assert_eq!(rev2.ecount(), 3);
        for eid in 0..3u32 {
            assert_eq!(g.edge(eid).unwrap(), rev2.edge(eid).unwrap());
        }
    }

    #[test]
    fn test_reverse_preserves_multi_edges() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();

        let rev = reverse(&g).unwrap();
        assert_eq!(rev.ecount(), 2);
        assert_eq!(rev.edge(0).unwrap(), (1, 0));
        assert_eq!(rev.edge(1).unwrap(), (1, 0));
    }

    // ---- reverse_edges tests ----

    #[test]
    fn test_reverse_edges_subset() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1
        g.add_edge(2, 3).unwrap(); // 2

        let rev = reverse_edges(&g, &[0, 2]).unwrap();
        assert_eq!(rev.edge(0).unwrap(), (1, 0)); // reversed
        assert_eq!(rev.edge(1).unwrap(), (1, 2)); // unchanged
        assert_eq!(rev.edge(2).unwrap(), (3, 2)); // reversed
    }

    #[test]
    fn test_reverse_edges_empty_set() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rev = reverse_edges(&g, &[]).unwrap();
        assert_eq!(rev.edge(0).unwrap(), (0, 1));
        assert_eq!(rev.edge(1).unwrap(), (1, 2));
    }

    #[test]
    fn test_reverse_edges_all() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rev = reverse_edges(&g, &[0, 1]).unwrap();
        assert_eq!(rev.edge(0).unwrap(), (1, 0));
        assert_eq!(rev.edge(1).unwrap(), (2, 1));
    }

    #[test]
    fn test_reverse_edges_undirected_noop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let rev = reverse_edges(&g, &[0]).unwrap();
        assert!(!rev.is_directed());
        assert_eq!(rev.ecount(), 2);
    }

    #[test]
    fn test_reverse_edges_out_of_range() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();

        let err = reverse_edges(&g, &[5]).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn test_reverse_edges_duplicates() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 2).unwrap(); // 1

        // Duplicate IDs should still result in a single reversal
        let rev = reverse_edges(&g, &[0, 0, 0]).unwrap();
        assert_eq!(rev.edge(0).unwrap(), (1, 0));
        assert_eq!(rev.edge(1).unwrap(), (1, 2));
    }

    #[test]
    fn test_reverse_edges_self_loop() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();

        let rev = reverse_edges(&g, &[0, 1]).unwrap();
        assert_eq!(rev.edge(0).unwrap(), (0, 0)); // self-loop unchanged
        assert_eq!(rev.edge(1).unwrap(), (1, 0)); // reversed
    }
}
