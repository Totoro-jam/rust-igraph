//! Cycle detection (ALGO-CY-001).
//!
//! Finds a single cycle in a graph using iterative DFS.
//! Counterpart of `igraph_find_cycle`.

use crate::core::{Graph, IgraphResult, VertexId};

/// Result of a cycle search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleResult {
    /// Vertex IDs forming the cycle (in order). Empty if no cycle exists.
    pub vertices: Vec<VertexId>,
    /// Edge IDs forming the cycle (in order). Empty if no cycle exists.
    pub edges: Vec<u32>,
}

/// Mode for following edges in directed graphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleMode {
    /// Follow outgoing edges.
    Out,
    /// Follow incoming edges.
    In,
    /// Ignore edge directions.
    All,
}

/// Finds a single cycle in the graph, if one exists.
///
/// Uses iterative DFS to find a back-edge, then extracts the cycle
/// from the DFS path. Returns empty vectors if the graph is acyclic.
///
/// For undirected graphs, `mode` is ignored and treated as `All`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, find_cycle, CycleMode};
///
/// // Directed cycle: 0→1→2→0
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
///
/// let result = find_cycle(&g, CycleMode::Out).unwrap();
/// // Cycle 0→1→2→0: vertices=[0,1,2,0], edges=[e0,e1,e2]
/// assert_eq!(result.vertices.len(), 4);
/// assert_eq!(result.edges.len(), 3);
/// assert_eq!(result.vertices.first(), result.vertices.last());
///
/// // DAG: no cycle
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
///
/// let result = find_cycle(&g, CycleMode::Out).unwrap();
/// assert!(result.vertices.is_empty());
/// ```
pub fn find_cycle(graph: &Graph, mode: CycleMode) -> IgraphResult<CycleResult> {
    let n = graph.vcount();
    let ecount = graph.ecount();

    if ecount == 0 {
        return Ok(CycleResult {
            vertices: Vec::new(),
            edges: Vec::new(),
        });
    }

    let effective_mode = if graph.is_directed() {
        mode
    } else {
        CycleMode::All
    };

    let inc = build_incidence(graph, effective_mode)?;

    // DFS state
    // seen: 0 = unseen, 1 = ancestor (on current path), 2 = fully explored
    let mut seen: Vec<u8> = vec![0; n as usize];
    let mut vpath: Vec<VertexId> = Vec::new();
    let mut epath: Vec<u32> = Vec::new();

    // Stack stores frames: either a vertex+edge to explore or a sentinel (-1)
    // to pop the path on backtrack.
    // We use i64 to encode: negative = sentinel, non-negative = vertex/edge pair
    let mut stack: Vec<StackEntry> = Vec::new();

    for start in 0..n {
        if seen[start as usize] != 0 {
            continue;
        }

        stack.push(StackEntry::Visit {
            vertex: start,
            edge: u32::MAX,
        });

        while let Some(entry) = stack.pop() {
            match entry {
                StackEntry::Backtrack => {
                    if let Some(v) = vpath.pop() {
                        seen[v as usize] = 2;
                    }
                    epath.pop();
                }
                StackEntry::Visit {
                    vertex: va,
                    edge: ea,
                } => {
                    if seen[va as usize] == 1 {
                        // Found a cycle — va is an ancestor on the current path
                        let cycle = extract_cycle(&vpath, &epath, va, ea);
                        return Ok(cycle);
                    }
                    if seen[va as usize] == 2 {
                        continue;
                    }

                    // Push va onto the path
                    seen[va as usize] = 1;
                    vpath.push(va);
                    epath.push(ea);

                    // Push backtrack sentinel
                    stack.push(StackEntry::Backtrack);

                    // Push incident edges
                    for &(nb, eid) in &inc[va as usize] {
                        if eid == ea {
                            continue;
                        }
                        if seen[nb as usize] == 2 {
                            continue;
                        }
                        stack.push(StackEntry::Visit {
                            vertex: nb,
                            edge: eid,
                        });
                    }
                }
            }
        }
    }

    Ok(CycleResult {
        vertices: Vec::new(),
        edges: Vec::new(),
    })
}

#[derive(Debug)]
enum StackEntry {
    Backtrack,
    Visit { vertex: VertexId, edge: u32 },
}

fn extract_cycle(
    vpath: &[VertexId],
    epath: &[u32],
    target: VertexId,
    closing_edge: u32,
) -> CycleResult {
    // Find where `target` appears in the path
    let start_idx = vpath.iter().position(|&v| v == target).unwrap_or(0);

    let mut vertices: Vec<VertexId> = vpath[start_idx..].to_vec();
    vertices.push(target);

    let mut edges: Vec<u32> = epath[(start_idx + 1)..].to_vec();
    edges.push(closing_edge);

    CycleResult { vertices, edges }
}

/// Build incidence list: for each vertex, list of (neighbor, `edge_id`) pairs.
fn build_incidence(graph: &Graph, mode: CycleMode) -> IgraphResult<Vec<Vec<(VertexId, u32)>>> {
    let n = graph.vcount() as usize;
    let ecount = graph.ecount();
    let directed = graph.is_directed();
    let mut inc: Vec<Vec<(VertexId, u32)>> = vec![Vec::new(); n];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let eid_u32 = eid as u32;
        let (src, tgt) = graph.edge(eid_u32)?;

        if !directed || mode == CycleMode::All {
            inc[src as usize].push((tgt, eid_u32));
            if src != tgt {
                inc[tgt as usize].push((src, eid_u32));
            }
        } else if mode == CycleMode::Out {
            inc[src as usize].push((tgt, eid_u32));
        } else {
            // CycleMode::In
            inc[tgt as usize].push((src, eid_u32));
        }
    }

    Ok(inc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = Graph::with_vertices(0);
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(r.vertices.is_empty());
        assert!(r.edges.is_empty());
    }

    #[test]
    fn test_no_edges() {
        let g = Graph::with_vertices(5);
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(r.vertices.is_empty());
    }

    #[test]
    fn test_tree_no_cycle() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(r.vertices.is_empty());
    }

    #[test]
    fn test_triangle_undirected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(!r.vertices.is_empty());
        // Cycle should have 3 vertices (first == last) and 3 edges
        assert_eq!(r.vertices.len(), r.edges.len() + 1);
        assert_eq!(*r.vertices.first().unwrap(), *r.vertices.last().unwrap());
    }

    #[test]
    fn test_directed_cycle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = find_cycle(&g, CycleMode::Out).unwrap();
        assert!(!r.vertices.is_empty());
        assert_eq!(r.vertices.first(), r.vertices.last());
    }

    #[test]
    fn test_dag_no_cycle() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = find_cycle(&g, CycleMode::Out).unwrap();
        assert!(r.vertices.is_empty());
    }

    #[test]
    fn test_directed_cycle_in_mode() {
        // 0←1←2←0 when followed in reverse
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = find_cycle(&g, CycleMode::In).unwrap();
        assert!(!r.vertices.is_empty());
        assert_eq!(r.vertices.first(), r.vertices.last());
    }

    #[test]
    fn test_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 1).unwrap();
        // Self-loop: the edge goes from 1 to 1. DFS from 0 visits 1 via edge 0,
        // then finds the self-loop edge 1 leading back to 1 which is on the path.
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(!r.vertices.is_empty());
    }

    #[test]
    fn test_undirected_4_cycle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(!r.vertices.is_empty());
        assert_eq!(r.vertices.first(), r.vertices.last());
        // A 4-cycle should be found
        assert!(r.vertices.len() >= 4);
    }

    #[test]
    fn test_disconnected_one_has_cycle() {
        let mut g = Graph::with_vertices(6);
        // Component 1: tree (0-1-2)
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        // Component 2: triangle (3-4-5)
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(3, 5).unwrap();
        let r = find_cycle(&g, CycleMode::All).unwrap();
        assert!(!r.vertices.is_empty());
    }

    #[test]
    fn test_cycle_edges_valid() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(1, 2).unwrap(); // eid 1
        g.add_edge(2, 3).unwrap(); // eid 2
        g.add_edge(3, 1).unwrap(); // eid 3
        let r = find_cycle(&g, CycleMode::Out).unwrap();
        assert!(!r.vertices.is_empty());
        // All edge IDs should be valid
        for &eid in &r.edges {
            assert!(graph_has_edge(&g, eid));
        }
    }

    fn graph_has_edge(g: &Graph, eid: u32) -> bool {
        g.edge(eid).is_ok()
    }
}
