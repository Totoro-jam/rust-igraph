//! Residual and reverse-residual graphs (ALGO-OP-030).
//!
//! Counterpart of `igraph_residual_graph()` and
//! `igraph_reverse_residual_graph()` from
//! `references/igraph/src/flow/st-cuts.c:148-283`.
//!
//! The residual graph contains edges with remaining capacity
//! (capacity − flow > 0). The reverse-residual graph contains both
//! forward edges with positive flow and backward edges with remaining
//! capacity — used in BFS-based min-cut algorithms.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Result of the residual graph computation.
#[derive(Debug, Clone)]
pub struct ResidualGraphResult {
    /// The residual directed graph (same vertex count, fewer edges).
    pub graph: Graph,
    /// Residual capacity of each edge in the new graph.
    pub capacity: Vec<f64>,
}

/// Compute the residual graph given capacity and flow vectors.
///
/// For each edge `e` where `capacity[e] - flow[e] > 0`, the residual
/// graph contains the same edge with residual capacity
/// `capacity[e] - flow[e]`.
///
/// # Errors
///
/// Returns an error if `capacity` or `flow` length differs from the
/// edge count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, residual_graph};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 2).unwrap(); // edge 1
/// let capacity = vec![10.0, 5.0];
/// let flow = vec![7.0, 5.0];
/// let result = residual_graph(&g, &capacity, &flow).unwrap();
/// // Edge 0 has residual 3.0, edge 1 is saturated (removed)
/// assert_eq!(result.graph.vcount(), 3);
/// assert_eq!(result.graph.ecount(), 1);
/// assert_eq!(result.capacity, vec![3.0]);
/// ```
pub fn residual_graph(
    graph: &Graph,
    capacity: &[f64],
    flow: &[f64],
) -> IgraphResult<ResidualGraphResult> {
    let m = graph.ecount();

    if capacity.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "residual_graph: capacity length {} != edge count {m}",
            capacity.len()
        )));
    }
    if flow.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "residual_graph: flow length {} != edge count {m}",
            flow.len()
        )));
    }

    let n = graph.vcount();
    let mut new_graph = Graph::new(n, true)?;
    let mut res_capacity: Vec<f64> = Vec::new();

    for eid in 0..m {
        let residual = capacity[eid] - flow[eid];
        if residual > 0.0 {
            let eid32 = u32::try_from(eid).map_err(|_| {
                IgraphError::InvalidArgument("residual_graph: edge id overflow".into())
            })?;
            let (from, to) = graph.edge(eid32)?;
            new_graph.add_edge(from, to)?;
            res_capacity.push(residual);
        }
    }

    Ok(ResidualGraphResult {
        graph: new_graph,
        capacity: res_capacity,
    })
}

/// Compute the reverse-residual graph given capacity and flow vectors.
///
/// For each edge `e` in the original graph:
/// - If `flow[e] > 0`: adds forward edge `(from, to)`
/// - If `flow[e] < capacity[e]`: adds backward edge `(to, from)`
///
/// If `capacity` is `None`, all edges are assumed to have capacity 1.0.
///
/// # Errors
///
/// Returns an error if vector lengths don't match the edge count.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, reverse_residual_graph};
///
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap(); // edge 0
/// g.add_edge(1, 2).unwrap(); // edge 1
/// let capacity = vec![10.0, 5.0];
/// let flow = vec![7.0, 5.0];
/// let result = reverse_residual_graph(&g, Some(&capacity), &flow).unwrap();
/// // Edge 0: flow>0 → forward (0,1); residual>0 → backward (1,0) = 2 edges
/// // Edge 1: flow>0 → forward (1,2); saturated → no backward = 1 edge
/// assert_eq!(result.vcount(), 3);
/// assert_eq!(result.ecount(), 3);
/// ```
pub fn reverse_residual_graph(
    graph: &Graph,
    capacity: Option<&[f64]>,
    flow: &[f64],
) -> IgraphResult<Graph> {
    let m = graph.ecount();

    if let Some(cap) = capacity {
        if cap.len() != m {
            return Err(IgraphError::InvalidArgument(format!(
                "reverse_residual_graph: capacity length {} != edge count {m}",
                cap.len()
            )));
        }
    }
    if flow.len() != m {
        return Err(IgraphError::InvalidArgument(format!(
            "reverse_residual_graph: flow length {} != edge count {m}",
            flow.len()
        )));
    }

    let n = graph.vcount();
    let mut new_graph = Graph::new(n, true)?;

    for eid in 0..m {
        let cap = capacity.map_or(1.0, |c| c[eid]);
        let eid32 = u32::try_from(eid).map_err(|_| {
            IgraphError::InvalidArgument("reverse_residual_graph: edge id overflow".into())
        })?;
        let (from, to) = graph.edge(eid32)?;

        if flow[eid] > 0.0 {
            new_graph.add_edge(from, to)?;
        }
        if flow[eid] < cap {
            new_graph.add_edge(to, from)?;
        }
    }

    Ok(new_graph)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn residual_empty() {
        let g = Graph::new(0, true).unwrap();
        let result = residual_graph(&g, &[], &[]).unwrap();
        assert_eq!(result.graph.vcount(), 0);
        assert_eq!(result.graph.ecount(), 0);
    }

    #[test]
    fn residual_no_flow() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let cap = vec![10.0, 5.0];
        let flow = vec![0.0, 0.0];
        let result = residual_graph(&g, &cap, &flow).unwrap();
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.capacity, vec![10.0, 5.0]);
    }

    #[test]
    fn residual_saturated() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![5.0];
        let flow = vec![5.0];
        let result = residual_graph(&g, &cap, &flow).unwrap();
        assert_eq!(result.graph.ecount(), 0);
        assert!(result.capacity.is_empty());
    }

    #[test]
    fn residual_partial_flow() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        let cap = vec![10.0, 10.0, 5.0];
        let flow = vec![7.0, 10.0, 3.0];
        let result = residual_graph(&g, &cap, &flow).unwrap();
        // Edge 0: residual 3, Edge 1: saturated, Edge 2: residual 2
        assert_eq!(result.graph.ecount(), 2);
        assert_eq!(result.capacity, vec![3.0, 2.0]);
    }

    #[test]
    fn residual_invalid_capacity_len() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(residual_graph(&g, &[1.0, 2.0], &[0.0]).is_err());
    }

    #[test]
    fn residual_invalid_flow_len() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(residual_graph(&g, &[1.0], &[0.0, 0.0]).is_err());
    }

    #[test]
    fn reverse_residual_empty() {
        let g = Graph::new(0, true).unwrap();
        let result = reverse_residual_graph(&g, Some(&[]), &[]).unwrap();
        assert_eq!(result.vcount(), 0);
        assert_eq!(result.ecount(), 0);
    }

    #[test]
    fn reverse_residual_no_flow() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![5.0];
        let flow = vec![0.0];
        let result = reverse_residual_graph(&g, Some(&cap), &flow).unwrap();
        // No forward (flow=0), one backward (flow<cap)
        assert_eq!(result.ecount(), 1);
        assert_eq!(result.edge(0).unwrap(), (1, 0));
    }

    #[test]
    fn reverse_residual_full_flow() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![5.0];
        let flow = vec![5.0];
        let result = reverse_residual_graph(&g, Some(&cap), &flow).unwrap();
        // Forward only (flow>0, flow==cap so no backward)
        assert_eq!(result.ecount(), 1);
        assert_eq!(result.edge(0).unwrap(), (0, 1));
    }

    #[test]
    fn reverse_residual_partial() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let cap = vec![10.0];
        let flow = vec![7.0];
        let result = reverse_residual_graph(&g, Some(&cap), &flow).unwrap();
        // Both forward (flow>0) and backward (flow<cap)
        assert_eq!(result.ecount(), 2);
        assert_eq!(result.edge(0).unwrap(), (0, 1));
        assert_eq!(result.edge(1).unwrap(), (1, 0));
    }

    #[test]
    fn reverse_residual_no_capacity() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let flow = vec![0.5];
        let result = reverse_residual_graph(&g, None, &flow).unwrap();
        // cap=1.0 by default: flow>0 → forward, flow<1.0 → backward
        assert_eq!(result.ecount(), 2);
    }

    #[test]
    fn reverse_residual_invalid_len() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(reverse_residual_graph(&g, Some(&[1.0, 2.0]), &[0.0]).is_err());
        assert!(reverse_residual_graph(&g, Some(&[1.0]), &[0.0, 0.0]).is_err());
    }
}
