//! Even-Tarjan reduction (ALGO-OP-029).
//!
//! Counterpart of `igraph_even_tarjan_reduction()` from
//! `references/igraph/src/flow/st-cuts.c:87-148`.
//!
//! Creates a directed graph with twice as many vertices where each
//! vertex `i` is split into `i'` and `i''`, connected by an edge.
//! Used for vertex-connectivity via edge-connectivity reduction.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of the Even-Tarjan reduction.
#[derive(Debug, Clone)]
pub struct EvenTarjanResult {
    /// The reduced directed graph with `2 * vcount` vertices.
    pub graph: Graph,
    /// Edge capacities: `1.0` for vertex-split edges, `f64::INFINITY`
    /// for edges derived from original edges.
    pub capacity: Vec<f64>,
}

/// Compute the Even-Tarjan reduction of a graph.
///
/// For each vertex `i` in the original graph (with `n` vertices), the
/// reduced graph contains vertices `i' = i` and `i'' = i + n`, connected
/// by a directed edge `i' → i''` with capacity 1.
///
/// For each directed edge `(from, to)` in the original graph, two edges
/// are added: `from'' → to'` and `to'' → from'`, both with infinite
/// capacity.
///
/// This reduction converts vertex-connectivity problems into
/// edge-connectivity problems on the reduced graph.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, even_tarjan_reduction};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let result = even_tarjan_reduction(&g).unwrap();
/// // 2*3 = 6 vertices
/// assert_eq!(result.graph.vcount(), 6);
/// // 3 vertex-split edges + 2*2 = 7 edges
/// assert_eq!(result.graph.ecount(), 7);
/// assert_eq!(result.capacity.len(), 7);
/// // First 3 capacities are 1.0 (vertex splits)
/// assert_eq!(result.capacity[0], 1.0);
/// assert_eq!(result.capacity[1], 1.0);
/// assert_eq!(result.capacity[2], 1.0);
/// // Remaining are infinity
/// assert_eq!(result.capacity[3], f64::INFINITY);
/// ```
pub fn even_tarjan_reduction(graph: &Graph) -> IgraphResult<EvenTarjanResult> {
    let n = graph.vcount();
    let m = graph.ecount();

    let new_n = n.checked_mul(2).ok_or_else(|| {
        IgraphError::InvalidArgument("even_tarjan_reduction: vertex count overflow".into())
    })?;

    let edges_from_vertices = n as usize;
    let edges_from_edges = m.checked_mul(2).ok_or_else(|| {
        IgraphError::InvalidArgument("even_tarjan_reduction: edge count overflow".into())
    })?;
    let total_edges = edges_from_vertices
        .checked_add(edges_from_edges)
        .ok_or_else(|| {
            IgraphError::InvalidArgument("even_tarjan_reduction: total edge count overflow".into())
        })?;

    let mut new_graph = Graph::new(new_n, true)?;
    let mut capacity: Vec<f64> = Vec::with_capacity(total_edges);

    // For each vertex i: add edge i' → i'' (capacity 1)
    for i in 0..n {
        new_graph.add_edge(i, i + n)?;
        capacity.push(1.0);
    }

    // For each original edge (from, to): add from'' → to' and to'' → from'
    // (both with infinite capacity)
    for eid in 0..m {
        let eid32 = u32::try_from(eid).map_err(|_| {
            IgraphError::InvalidArgument("even_tarjan_reduction: edge id overflow".into())
        })?;
        let (from, to) = graph.edge(eid32)?;

        new_graph.add_edge(from + n, to)?;
        capacity.push(f64::INFINITY);

        new_graph.add_edge(to + n, from)?;
        capacity.push(f64::INFINITY);
    }

    Ok(EvenTarjanResult {
        graph: new_graph,
        capacity,
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
        assert!(result.capacity.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, true).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        assert_eq!(result.graph.vcount(), 2);
        assert_eq!(result.graph.ecount(), 1);
        assert_eq!(result.capacity, vec![1.0]);
        let (from, to) = result.graph.edge(0).unwrap();
        assert_eq!((from, to), (0, 1));
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        // 4 vertices: 0', 1', 0'', 1''
        assert_eq!(result.graph.vcount(), 4);
        // 2 vertex-split + 2 from the edge = 4
        assert_eq!(result.graph.ecount(), 4);
        assert_eq!(
            result.capacity,
            vec![1.0, 1.0, f64::INFINITY, f64::INFINITY]
        );

        // Vertex splits: 0→2 (0'→0''), 1→3 (1'→1'')
        assert_eq!(result.graph.edge(0).unwrap(), (0, 2));
        assert_eq!(result.graph.edge(1).unwrap(), (1, 3));
        // Edge (0,1): 0''→1' (2→1), 1''→0' (3→0)
        assert_eq!(result.graph.edge(2).unwrap(), (2, 1));
        assert_eq!(result.graph.edge(3).unwrap(), (3, 0));
    }

    #[test]
    fn triangle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        assert_eq!(result.graph.vcount(), 6);
        // 3 vertex-split + 3*2 = 9
        assert_eq!(result.graph.ecount(), 9);
        assert_eq!(result.capacity.len(), 9);
        // First 3 are vertex splits
        for i in 0..3 {
            assert_eq!(result.capacity[i], 1.0);
        }
        // Rest are infinite
        for i in 3..9 {
            assert_eq!(result.capacity[i], f64::INFINITY);
        }
    }

    #[test]
    fn undirected_graph() {
        // Should still work — treats edges as directed
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        assert_eq!(result.graph.vcount(), 6);
        // 3 vertex-split + 2*2 = 7
        assert_eq!(result.graph.ecount(), 7);
        assert!(result.graph.is_directed());
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        assert_eq!(result.graph.vcount(), 4);
        // 2 vertex-split + 2*2 = 6
        assert_eq!(result.graph.ecount(), 6);
    }

    #[test]
    fn capacity_values() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let result = even_tarjan_reduction(&g).unwrap();
        // 4 vertex-split (cap 1) + 4 edge-derived (cap inf)
        assert_eq!(result.capacity.len(), 8);
        let ones: Vec<f64> = result
            .capacity
            .iter()
            .filter(|&&c| c == 1.0)
            .copied()
            .collect();
        let infs: Vec<f64> = result
            .capacity
            .iter()
            .filter(|c| c.is_infinite())
            .copied()
            .collect();
        assert_eq!(ones.len(), 4);
        assert_eq!(infs.len(), 4);
    }
}
