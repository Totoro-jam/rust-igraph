//! Fundamental cycle basis (ALGO-CY-003).
//!
//! Computes a fundamental cycle basis associated with a BFS tree.
//! Each non-tree edge creates exactly one fundamental cycle when
//! combined with the unique tree path between its endpoints.
//!
//! Edge directions are ignored. Multi-edges and self-loops are
//! supported.
//!
//! Counterpart of `igraph_fundamental_cycles`.

use std::collections::VecDeque;

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute the fundamental cycle basis of a graph.
///
/// Returns a list of cycles, where each cycle is a `Vec<EdgeId>`
/// listing the edge IDs forming that cycle. The cycle basis is
/// associated with a BFS tree rooted at each connected component.
///
/// If `start_vid` is `Some(v)`, only the component containing `v`
/// is processed. If `None`, all components are processed.
///
/// If `bfs_cutoff` is `Some(k)`, only cycles of length at most
/// `2*k + 1` are returned. If `None`, all fundamental cycles are
/// found.
///
/// Edge directions are ignored (the graph is treated as undirected).
///
/// # Errors
///
/// Returns an error if `start_vid` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fundamental_cycles};
///
/// // Triangle: one fundamental cycle of length 3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let cycles = fundamental_cycles(&g, None, None).unwrap();
/// assert_eq!(cycles.len(), 1);
/// assert_eq!(cycles[0].len(), 3);
///
/// // Tree: no cycles.
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// let cycles = fundamental_cycles(&g, None, None).unwrap();
/// assert!(cycles.is_empty());
/// ```
pub fn fundamental_cycles(
    graph: &Graph,
    start_vid: Option<VertexId>,
    bfs_cutoff: Option<u32>,
) -> IgraphResult<Vec<Vec<EdgeId>>> {
    let n = graph.vcount();

    if let Some(v) = start_vid {
        if v >= n {
            return Err(IgraphError::InvalidArgument(format!(
                "start vertex {v} out of range (vcount = {n})"
            )));
        }
    }

    let adj = build_incident_all(graph)?;

    let mut visited: Vec<u8> = vec![0; n as usize];
    let mut result: Vec<Vec<EdgeId>> = Vec::new();

    if let Some(v) = start_vid {
        fundamental_cycles_bfs(graph, &adj, &mut result, v, bfs_cutoff, &mut visited, 0)?;
    } else {
        for v in 0..n {
            if visited[v as usize] == 0 {
                fundamental_cycles_bfs(graph, &adj, &mut result, v, bfs_cutoff, &mut visited, 0)?;
            }
        }
    }

    Ok(result)
}

fn build_incident_all(graph: &Graph) -> IgraphResult<Vec<Vec<(EdgeId, VertexId)>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<(EdgeId, VertexId)>> = vec![Vec::new(); n];

    for eid in 0..graph.ecount() {
        #[allow(clippy::cast_possible_truncation)]
        let eid32 = eid as EdgeId;
        let (from, to) = graph.edge(eid32)?;
        if from == to {
            adj[from as usize].push((eid32, to));
        } else {
            adj[from as usize].push((eid32, to));
            adj[to as usize].push((eid32, from));
        }
    }

    Ok(adj)
}

fn fundamental_cycles_bfs(
    graph: &Graph,
    adj: &[Vec<(EdgeId, VertexId)>],
    result: &mut Vec<Vec<EdgeId>>,
    start: VertexId,
    bfs_cutoff: Option<u32>,
    visited: &mut [u8],
    _mark: u8,
) -> IgraphResult<()> {
    let n = graph.vcount() as usize;
    let mut pred_edge: Vec<i64> = vec![-1; n];
    let mut queue: VecDeque<(VertexId, u32)> = VecDeque::new();

    visited[start as usize] = 1; // queued
    queue.push_back((start, 0));

    while let Some((v, vdist)) = queue.pop_front() {
        for &(eid, u) in &adj[v as usize] {
            if i64::from(eid) == pred_edge[v as usize] {
                continue;
            }

            let u_state = visited[u as usize];

            if u_state == 2 {
                continue;
            }
            if u_state == 1 {
                let cycle = trace_cycle(graph, pred_edge.as_slice(), eid, u, v)?;
                result.push(cycle);
            } else if bfs_cutoff.is_none() || vdist < bfs_cutoff.unwrap_or(0) {
                visited[u as usize] = 1;
                pred_edge[u as usize] = i64::from(eid);
                queue.push_back((u, vdist.saturating_add(1)));
            }
        }

        visited[v as usize] = 2; // processed
    }

    Ok(())
}

fn trace_cycle(
    graph: &Graph,
    pred_edge: &[i64],
    closing_edge: EdgeId,
    u: VertexId,
    v: VertexId,
) -> IgraphResult<Vec<EdgeId>> {
    let mut u_back: Vec<EdgeId> = Vec::new();
    let mut v_back: Vec<EdgeId> = vec![closing_edge];

    let mut up = u;
    let mut vp = v;

    loop {
        if up == vp {
            break;
        }

        let u_pred = pred_edge[up as usize];
        if u_pred < 0 {
            break;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let u_eid = u_pred as EdgeId;
        u_back.push(u_eid);
        up = graph.edge_other(u_eid, up)?;

        if up == vp {
            break;
        }

        let v_pred = pred_edge[vp as usize];
        if v_pred < 0 {
            break;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v_eid = v_pred as EdgeId;
        v_back.push(v_eid);
        vp = graph.edge_other(v_eid, vp)?;
    }

    // Build cycle: v_back + reversed u_back
    let mut cycle = v_back;
    cycle.extend(u_back.into_iter().rev());
    Ok(cycle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn tree_no_cycles() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn single_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn k4_three_cycles() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(0, 2).unwrap(); // 1
        g.add_edge(0, 3).unwrap(); // 2
        g.add_edge(1, 2).unwrap(); // 3
        g.add_edge(1, 3).unwrap(); // 4
        g.add_edge(2, 3).unwrap(); // 5
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        // K4: 6 edges - 4 vertices + 1 = 3 fundamental cycles
        assert_eq!(cycles.len(), 3);
    }

    #[test]
    fn two_components() {
        let mut g = Graph::with_vertices(6);
        // Triangle 0-1-2
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Triangle 3-4-5
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn start_vertex_single_component() {
        let mut g = Graph::with_vertices(6);
        // Triangle 0-1-2
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        // Triangle 3-4-5
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let cycles = fundamental_cycles(&g, Some(0), None).unwrap();
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn start_vertex_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(fundamental_cycles(&g, Some(5), None).is_err());
    }

    #[test]
    fn cycle_rank_formula() {
        // For a connected graph: cycle_rank = |E| - |V| + 1
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap(); // creates one cycle
        g.add_edge(0, 2).unwrap(); // creates another
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        // rank = 6 - 5 + 1 = 2
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn self_loop() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 1); // self-loop is a cycle of 1 edge
    }

    #[test]
    fn multi_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel edge
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2); // two edges form a cycle
    }

    #[test]
    fn directed_graph_treated_as_undirected() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn bfs_cutoff_limits_cycles() {
        // Create a cycle of length 5 and a cycle of length 3
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap(); // cycle of length 5
        g.add_edge(0, 2).unwrap(); // creates a shorter cycle through 0-1-2-0

        // With cutoff = 1, only cycles of length 2*1+1=3 or shorter
        let cycles = fundamental_cycles(&g, None, Some(1)).unwrap();
        for cycle in &cycles {
            assert!(cycle.len() <= 3);
        }
    }

    #[test]
    fn isolated_vertices() {
        let g = Graph::with_vertices(10);
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let cycles = fundamental_cycles(&g, None, None).unwrap();
        assert!(cycles.is_empty());
    }
}
