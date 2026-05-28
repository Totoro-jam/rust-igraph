//! Enumerate all simple cycles — Johnson's algorithm (ALGO-CY-003).
//!
//! Counterpart of `igraph_simple_cycles()` from
//! `references/igraph/src/cycles/simple_cycles.c`.
//!
//! A simple cycle is a closed path without repeated vertices.
//! Uses Johnson's algorithm (1975) which systematically enumerates
//! all elementary circuits of a directed graph. For undirected graphs,
//! the algorithm treats them as symmetric directed graphs with
//! duplicate-cycle suppression.
//!
//! Reference: Johnson DB, "Finding all the elementary circuits of a
//! directed graph." SIAM J. Comput. 4(1):77–84, 1975.

use crate::core::graph::EdgeId;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Mode for following edges during cycle search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleCycleMode {
    /// Follow outgoing edges (directed graphs only).
    Out,
    /// Follow incoming edges (directed graphs only).
    In,
    /// Ignore edge directions.
    All,
}

/// A single simple cycle found by Johnson's algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCycle {
    /// Vertex IDs forming the cycle (ordered, not repeated).
    pub vertices: Vec<VertexId>,
    /// Edge IDs forming the cycle (ordered).
    pub edges: Vec<EdgeId>,
}

/// Finds all simple cycles in the graph using Johnson's algorithm.
///
/// Returns cycles with length in `[min_length, max_length]`.
/// If `max_results` is `Some(n)`, at most `n` cycles are returned.
///
/// For undirected graphs, `mode` is ignored. Each undirected cycle is
/// reported once (the canonical orientation has the first edge id
/// smaller than the closing edge id).
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `min_length < 1`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, simple_cycles, SimpleCycleMode};
///
/// // Directed triangle: 0→1→2→0
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
///
/// let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
/// assert_eq!(cycles.len(), 1);
/// assert_eq!(cycles[0].vertices, vec![0, 1, 2]);
/// assert_eq!(cycles[0].edges, vec![0, 1, 2]);
/// ```
pub fn simple_cycles(
    graph: &Graph,
    mode: SimpleCycleMode,
    min_length: u32,
    max_length: Option<u32>,
    max_results: Option<usize>,
) -> IgraphResult<Vec<SimpleCycle>> {
    if min_length < 1 {
        return Err(IgraphError::InvalidArgument(
            "simple_cycles: min_length must be >= 1".to_string(),
        ));
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(Vec::new());
    }

    let max_len = max_length.unwrap_or(n);
    if max_len == 0 {
        return Ok(Vec::new());
    }

    let directed = graph.is_directed() && mode != SimpleCycleMode::All;

    // Build adjacency + incidence lists according to mode.
    let (adj, inc) = build_adj_inc(graph, mode)?;

    // Mutable copies that get progressively shrunk as vertices are processed.
    let n_usize = n as usize;
    let mut adj_work: Vec<Vec<VertexId>> = adj.clone();
    let mut inc_work: Vec<Vec<EdgeId>> = inc.clone();

    let mut blocked: Vec<bool> = vec![false; n_usize];
    let mut b_lists: Vec<Vec<VertexId>> = vec![Vec::new(); n_usize];

    let mut vertex_stack: Vec<VertexId> = Vec::new();
    let mut edge_stack: Vec<EdgeId> = Vec::new();

    let mut results: Vec<SimpleCycle> = Vec::new();
    let mut stop = false;

    for s in 0..n {
        if stop {
            break;
        }

        // Reset blocked and B for vertices >= s.
        for i in (s as usize)..n_usize {
            blocked[i] = false;
            b_lists[i].clear();
        }

        // Skip vertices that can't start a cycle (degree < 2 in undirected sense,
        // and already visited in a prior component).
        if adj_work[s as usize].is_empty() {
            continue;
        }

        // Search for cycles starting from s.
        johnson_circuit(
            &adj_work,
            &inc_work,
            s,
            s,
            directed,
            min_length,
            max_len,
            max_results,
            &mut blocked,
            &mut b_lists,
            &mut vertex_stack,
            &mut edge_stack,
            &mut results,
            &mut stop,
        );

        // Remove vertex s from the graph (Johnson's L3 step).
        for i in 0..n_usize {
            if let Some(pos) = adj_work[i].iter().position(|&v| v == s) {
                adj_work[i].remove(pos);
                inc_work[i].remove(pos);
            }
        }
        adj_work[s as usize].clear();
        inc_work[s as usize].clear();
    }

    Ok(results)
}

/// Adjacency and incidence lists (parallel vectors).
type AdjInc = (Vec<Vec<VertexId>>, Vec<Vec<EdgeId>>);

/// Build sorted adjacency and incidence lists for the given mode.
fn build_adj_inc(graph: &Graph, mode: SimpleCycleMode) -> IgraphResult<AdjInc> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<VertexId>> = vec![Vec::new(); n];
    let mut inc: Vec<Vec<EdgeId>> = vec![Vec::new(); n];

    let m = graph.ecount();
    for eid in 0..m {
        let eid32 = u32::try_from(eid)
            .map_err(|_| IgraphError::InvalidArgument("edge count exceeds u32".to_string()))?;
        let (from, to) = graph.edge(eid32)?;

        if graph.is_directed() {
            match mode {
                SimpleCycleMode::Out => {
                    adj[from as usize].push(to);
                    inc[from as usize].push(eid32);
                }
                SimpleCycleMode::In => {
                    adj[to as usize].push(from);
                    inc[to as usize].push(eid32);
                }
                SimpleCycleMode::All => {
                    adj[from as usize].push(to);
                    inc[from as usize].push(eid32);
                    adj[to as usize].push(from);
                    inc[to as usize].push(eid32);
                }
            }
        } else {
            adj[from as usize].push(to);
            inc[from as usize].push(eid32);
            adj[to as usize].push(from);
            inc[to as usize].push(eid32);
        }
    }

    // Sort each adjacency list by neighbor id (keep inc in sync).
    for v in 0..n {
        let mut pairs: Vec<(VertexId, EdgeId)> =
            adj[v].iter().copied().zip(inc[v].iter().copied()).collect();
        pairs.sort_unstable();
        adj[v] = pairs.iter().map(|&(a, _)| a).collect();
        inc[v] = pairs.iter().map(|&(_, e)| e).collect();
    }

    Ok((adj, inc))
}

/// Johnson's CIRCUIT procedure (iterative).
#[allow(clippy::too_many_arguments)]
fn johnson_circuit(
    adj: &[Vec<VertexId>],
    inc: &[Vec<EdgeId>],
    s: VertexId,
    start_v: VertexId,
    directed: bool,
    min_length: u32,
    max_length: u32,
    max_results: Option<usize>,
    blocked: &mut [bool],
    b_lists: &mut [Vec<VertexId>],
    vertex_stack: &mut Vec<VertexId>,
    edge_stack: &mut Vec<EdgeId>,
    results: &mut Vec<SimpleCycle>,
    stop: &mut bool,
) {
    // Iterative DFS using an explicit frame stack.
    // Each frame tracks: which vertex we're at, where we are in its neighbor list,
    // and whether we found a cycle below.
    let mut frame_v: Vec<VertexId> = Vec::new();
    let mut frame_idx: Vec<usize> = Vec::new();
    let mut frame_found: Vec<bool> = Vec::new();
    let mut frame_length_stop: Vec<bool> = Vec::new();

    // Push the initial frame for the start vertex.
    vertex_stack.push(start_v);
    blocked[start_v as usize] = true;
    frame_v.push(start_v);
    frame_idx.push(0);
    frame_found.push(false);
    frame_length_stop.push(false);

    loop {
        if *stop || frame_v.is_empty() {
            break;
        }

        let depth = frame_v.len() - 1;
        let v = frame_v[depth];
        let v_idx = v as usize;
        let neighbors = &adj[v_idx];
        let incidents = &inc[v_idx];

        let mut advanced = false;

        while frame_idx[depth] < neighbors.len() {
            let i = frame_idx[depth];
            frame_idx[depth] += 1;

            let w = neighbors[i];
            let we = incidents[i];

            if w == s {
                frame_found[depth] = true;

                // For undirected graphs, suppress duplicates.
                if !directed {
                    if edge_stack.len() == 1 && edge_stack[0] == we {
                        continue;
                    }
                    if !edge_stack.is_empty() && edge_stack[0] > we {
                        continue;
                    }
                }

                let cycle_len = vertex_stack.len();
                let min_l = min_length as usize;
                let max_l = max_length as usize;
                if cycle_len >= min_l && cycle_len <= max_l {
                    let mut cycle_edges = edge_stack.clone();
                    cycle_edges.push(we);
                    results.push(SimpleCycle {
                        vertices: vertex_stack.clone(),
                        edges: cycle_edges,
                    });

                    if let Some(max) = max_results {
                        if results.len() >= max {
                            *stop = true;
                            break;
                        }
                    }
                }
            } else if !blocked[w as usize] {
                let can_recurse = vertex_stack.len() < max_length as usize;
                if can_recurse {
                    vertex_stack.push(w);
                    edge_stack.push(we);
                    blocked[w as usize] = true;

                    frame_v.push(w);
                    frame_idx.push(0);
                    frame_found.push(false);
                    frame_length_stop.push(false);
                    advanced = true;
                    break;
                }
                frame_length_stop[depth] = true;
            }
        }

        if *stop {
            break;
        }

        if !advanced {
            let found = frame_found[depth];
            let length_stop = frame_length_stop[depth];
            let cur_v = frame_v[depth];

            frame_v.pop();
            frame_idx.pop();
            frame_found.pop();
            frame_length_stop.pop();

            if found || length_stop {
                unblock(cur_v, blocked, b_lists);
            } else {
                for &w in &adj[cur_v as usize] {
                    let w_b = &mut b_lists[w as usize];
                    if !w_b.contains(&cur_v) {
                        w_b.push(cur_v);
                    }
                }
            }

            vertex_stack.pop();
            edge_stack.pop();

            // Propagate "found" to parent frame.
            if !frame_v.is_empty() && (found || length_stop) {
                let parent = frame_v.len() - 1;
                frame_found[parent] = true;
            }
        }
    }

    vertex_stack.clear();
    edge_stack.clear();
}

/// Johnson's UNBLOCK procedure (iterative).
fn unblock(u: VertexId, blocked: &mut [bool], b_lists: &mut [Vec<VertexId>]) {
    let mut stack: Vec<VertexId> = vec![u];

    while let Some(v) = stack.pop() {
        if blocked[v as usize] {
            blocked[v as usize] = false;
            while let Some(w) = b_lists[v as usize].pop() {
                if blocked[w as usize] {
                    stack.push(w);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn no_edges() {
        let g = Graph::with_vertices(5);
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn directed_triangle() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].vertices, vec![0, 1, 2]);
        assert_eq!(cycles[0].edges, vec![0, 1, 2]);
    }

    #[test]
    fn directed_two_cycles() {
        // 0→1→0 and 0→1→2→0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // 0
        g.add_edge(1, 0).unwrap(); // 1
        g.add_edge(1, 2).unwrap(); // 2
        g.add_edge(2, 0).unwrap(); // 3
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn undirected_triangle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, None, None).unwrap();
        // One triangle
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].vertices.len(), 3);
    }

    #[test]
    fn undirected_square() {
        // 0-1-2-3-0: one 4-cycle
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].vertices.len(), 4);
    }

    #[test]
    fn undirected_complete_4() {
        // K4 has 7 simple cycles: 4 triangles + 3 four-cycles
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 7);
    }

    #[test]
    fn min_length_filter() {
        // K4: filter out triangles (length 3), only keep 4-cycles
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 4, None, None).unwrap();
        assert_eq!(cycles.len(), 3);
        for c in &cycles {
            assert_eq!(c.vertices.len(), 4);
        }
    }

    #[test]
    fn max_length_filter() {
        // K4: only triangles (max_length=3)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, Some(3), None).unwrap();
        assert_eq!(cycles.len(), 4);
        for c in &cycles {
            assert_eq!(c.vertices.len(), 3);
        }
    }

    #[test]
    fn max_results() {
        // K4 has 7 cycles, limit to 3
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, None, Some(3)).unwrap();
        assert_eq!(cycles.len(), 3);
    }

    #[test]
    fn self_loop_directed() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].vertices, vec![0]);
        assert_eq!(cycles[0].edges, vec![0]);
    }

    #[test]
    fn directed_in_mode() {
        // 0→1→2→0: in IN mode, the reversed direction has a cycle 0←2←1←0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::In, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 1);
        // In reverse: 0←2←1←0, so vertices should be [0, 2, 1]
        assert_eq!(cycles[0].vertices.len(), 3);
    }

    #[test]
    fn dag_no_cycles() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn tree_no_cycles() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::All, 1, None, None).unwrap();
        assert!(cycles.is_empty());
    }

    #[test]
    fn directed_k3_all_edges() {
        // Complete directed graph on 3 vertices: 6 directed edges, 8 cycles
        // (3 two-cycles + 2 three-cycles... actually: 3 pairs of opposite edges =
        //  3 two-cycles, plus 2 directed triangles = 5 total? Let's count.)
        // Actually K3 directed has edges: 0→1, 1→0, 0→2, 2→0, 1→2, 2→1
        // Two-cycles: {0,1}, {0,2}, {1,2} = 3
        // Three-cycles: 0→1→2→0 and 0→2→1→0 = 2
        // Total = 5
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        let cycles = simple_cycles(&g, SimpleCycleMode::Out, 1, None, None).unwrap();
        assert_eq!(cycles.len(), 5);
    }
}
