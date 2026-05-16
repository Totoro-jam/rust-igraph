//! Strongly connected components (ALGO-CC-002).
//!
//! Counterpart of `igraph_connected_components(_, _, _, _, IGRAPH_STRONG)` from
//! `references/igraph/src/connectivity/components.c:203-386`.
//!
//! Algorithm: iterative two-pass DFS (Kosaraju), matching upstream igraph
//! line-for-line so that membership labels agree with `python-igraph`'s
//! `Graph.connected_components(mode='strong')` exactly (verified by
//! `tests/oracle.rs::strongly_connected_components_*`).
//!
//! Pass 1 — forward DFS over **out**-neighbours from every unvisited vertex.
//! When all out-edges of a vertex are exhausted, push it to a post-order
//! stack `out`.
//!
//! Pass 2 — pop `out` from the back. Each unmarked grandfather seeds a new
//! component; from there, do an iterative LIFO walk over **in**-neighbours
//! to gather every vertex in the same SCC.
//!
//! Undirected graphs delegate to weak [`connected_components`] — the strong
//! components of an undirected graph are the same as its weak components.
//!
//! Result shape is identical to weak components ([`ConnectedComponents`]):
//! `membership` length `vcount`, dense ids `0..count` assigned in
//! grandfather-pop order.
//!
//! Self-loops and parallel edges are handled identically to upstream
//! (`IGRAPH_LOOPS_ONCE, IGRAPH_MULTIPLE` adjlist flags) — the
//! `next_nei`/`visited` flag prevents re-pushing.

use crate::algorithms::connectivity::components::{ConnectedComponents, connected_components};
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Compute the strongly connected components of `graph`.
///
/// Counterpart of `igraph_connected_components(_, _, _, _, IGRAPH_STRONG)`.
/// On an undirected graph, returns the same partition as
/// [`connected_components`].
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, strongly_connected_components};
///
/// // Two disjoint 3-cycles in a directed graph: 0→1→2→0 and 3→4→5→3.
/// let mut g = Graph::new(6, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 3).unwrap();
///
/// let scc = strongly_connected_components(&g).unwrap();
/// assert_eq!(scc.count, 2);
/// assert_eq!(scc.membership[0], scc.membership[1]);
/// assert_eq!(scc.membership[1], scc.membership[2]);
/// assert_eq!(scc.membership[3], scc.membership[4]);
/// assert_ne!(scc.membership[0], scc.membership[3]);
/// ```
pub fn strongly_connected_components(graph: &Graph) -> IgraphResult<ConnectedComponents> {
    let n = graph.vcount();
    if n == 0 {
        return Ok(ConnectedComponents {
            membership: Vec::new(),
            count: 0,
        });
    }

    if !graph.is_directed() {
        return connected_components(graph);
    }

    let n_us = n as usize;

    // Cache out-neighbour lists once. Same memory cost as upstream's
    // `igraph_adjlist_init(graph, &adjlist, IGRAPH_OUT, ...)`.
    let mut out_adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        out_adj.push(graph.out_neighbors_vec(v)?);
    }

    // Pass 1: iterative forward DFS, post-order stack.
    //
    // `next_nei[v]` is upstream's eponymous counter:
    //   0           — vertex never pushed onto the DFS stack;
    //   1..=len     — DFS frame is live; next out-neighbour to consider
    //                 is `out_adj[v][next_nei[v] - 1]`;
    //   len + 1     — vertex has been popped (post-order recorded).
    let mut next_nei: Vec<usize> = vec![0; n_us];
    let mut out: Vec<VertexId> = Vec::with_capacity(n_us);
    let mut stack: Vec<VertexId> = Vec::new();

    for start in 0..n {
        if next_nei[start as usize] > out_adj[start as usize].len() {
            continue;
        }
        stack.push(start);
        while let Some(&act) = stack.last() {
            let act_us = act as usize;
            let nbrs_len = out_adj[act_us].len();
            if next_nei[act_us] == 0 {
                // first visit
                next_nei[act_us] = 1;
            } else if next_nei[act_us] <= nbrs_len {
                let nb = out_adj[act_us][next_nei[act_us] - 1];
                if next_nei[nb as usize] == 0 {
                    stack.push(nb);
                }
                next_nei[act_us] += 1;
            } else {
                // all out-edges processed; record post-order
                out.push(act);
                stack.pop();
            }
        }
    }

    // Cache in-neighbour lists for pass 2.
    let mut in_adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        in_adj.push(graph.in_neighbors_vec(v)?);
    }

    // Pass 2: pop `out` from the back; each unmarked grandfather seeds a
    // new SCC. Visited vertices are flagged by `visited[v] = 1`.
    let mut visited: Vec<u8> = vec![0; n_us];
    let mut membership = vec![u32::MAX; n_us];
    let mut count: u32 = 0;
    let mut walk: Vec<VertexId> = Vec::new();

    while let Some(grandfather) = out.pop() {
        if visited[grandfather as usize] != 0 {
            continue;
        }
        visited[grandfather as usize] = 1;
        let comp_id = count;
        count = count
            .checked_add(1)
            .ok_or(IgraphError::Internal("too many components"))?;
        membership[grandfather as usize] = comp_id;

        walk.clear();
        walk.push(grandfather);
        while let Some(v) = walk.pop() {
            for &nb in &in_adj[v as usize] {
                if visited[nb as usize] == 0 {
                    visited[nb as usize] = 1;
                    membership[nb as usize] = comp_id;
                    walk.push(nb);
                }
            }
        }
    }

    Ok(ConnectedComponents { membership, count })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::new(0, true).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 0);
        assert!(scc.membership.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::new(1, true).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 1);
        assert_eq!(scc.membership, vec![0]);
    }

    #[test]
    fn n_isolated_vertices_yields_n_components() {
        let g = Graph::new(5, true).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 5);
        // Order in which Kosaraju pops grandfathers from `out`. With no
        // edges, pass-1 push/pop is `0,1,2,3,4` so `out = [0,1,2,3,4]`
        // and pop order is `4,3,2,1,0`, giving labels `[4,3,2,1,0]`.
        assert_eq!(scc.membership, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn directed_path_each_vertex_alone() {
        // 0 -> 1 -> 2 -> 3: every vertex its own SCC.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 4);
        // Distinct labels for every vertex.
        let mut sorted = scc.membership.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
    }

    #[test]
    fn directed_3_cycle_is_one_component() {
        // 0 -> 1 -> 2 -> 0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 1);
        assert_eq!(scc.membership, vec![0, 0, 0]);
    }

    #[test]
    fn cycle_with_outgoing_chain() {
        // 0 -> 1 -> 2 -> 0; 2 -> 3; 3 -> 4
        // SCCs: {0,1,2}, {3}, {4}.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 3);
        // Labels must agree on the cycle.
        assert_eq!(scc.membership[0], scc.membership[1]);
        assert_eq!(scc.membership[1], scc.membership[2]);
        // Tail vertices are singletons distinct from each other and from the cycle.
        assert_ne!(scc.membership[3], scc.membership[0]);
        assert_ne!(scc.membership[4], scc.membership[0]);
        assert_ne!(scc.membership[3], scc.membership[4]);
    }

    #[test]
    fn two_disjoint_cycles() {
        // 0 -> 1 -> 2 -> 0; 3 -> 4 -> 5 -> 3
        let mut g = Graph::new(6, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 3).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 2);
        assert_eq!(scc.membership[0], scc.membership[1]);
        assert_eq!(scc.membership[1], scc.membership[2]);
        assert_eq!(scc.membership[3], scc.membership[4]);
        assert_eq!(scc.membership[4], scc.membership[5]);
        assert_ne!(scc.membership[0], scc.membership[3]);
    }

    #[test]
    fn self_loop_does_not_split_component() {
        // 0 -> 0; 0 -> 1; 1 -> 0 → SCC {0,1}.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 1);
        assert_eq!(scc.membership, vec![0, 0]);
    }

    #[test]
    fn parallel_edges_do_not_split_component() {
        // Three parallel edges 0 -> 1 plus a return 1 -> 0 → SCC {0,1}.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 1);
        assert_eq!(scc.membership, vec![0, 0]);
    }

    #[test]
    fn undirected_graph_delegates_to_weak() {
        // Undirected 5-vertex with two components → SCC == WCC.
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        let wcc = connected_components(&g).unwrap();
        assert_eq!(scc.membership, wcc.membership);
        assert_eq!(scc.count, wcc.count);
    }

    #[test]
    fn membership_ids_are_dense() {
        // 0 -> 1 -> 2 -> 0; 3 -> 4 (chain) → 3 components, ids 0..3.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(3, 4).unwrap();
        let scc = strongly_connected_components(&g).unwrap();
        assert_eq!(scc.count, 3);
        let mut seen: Vec<u32> = scc.membership.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen, vec![0, 1, 2]);
    }
}
