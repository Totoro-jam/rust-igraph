//! Graph reversal operator (ALGO-OP-008).
//!
//! Reverses all edge directions in a directed graph.

use crate::core::{Graph, IgraphResult};

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
}
