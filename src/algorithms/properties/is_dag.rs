//! DAG predicate (ALGO-PR-020).
//!
//! Counterpart of `igraph_is_dag()` from
//! `references/igraph/src/properties/dag.c:151`. Returns `true` iff
//! `graph` is a directed acyclic graph. Undirected graphs are
//! always `false` (a DAG must be directed by definition — matches
//! upstream).
//!
//! Algorithm: Kahn's topological-sort. Repeatedly peel off
//! zero-in-degree vertices until either the graph is empty (DAG)
//! or every remaining vertex has at least one incoming edge
//! (a cycle exists). Self-loops are detected as an early
//! short-circuit because they contribute a vertex with
//! `in-degree >= 1` that can never be peeled.
//!
//! Time complexity: `O(V + E)`.

use std::collections::VecDeque;

use crate::core::Graph;
use crate::core::cache::CachedProperty;
use crate::core::graph::VertexId;

/// Returns `true` iff `graph` is a directed acyclic graph.
///
/// Undirected graphs return `false` unconditionally (matches
/// upstream — DAGs are directed by definition). Empty graphs and
/// graphs with isolated vertices but no edges return `true`.
///
/// Algorithm: Kahn's topological-sort peel. Start with vertices
/// whose in-degree is 0 in a queue; pop one, "remove" it by
/// decrementing the in-degree of its out-neighbours. If any
/// neighbour reaches 0, queue it. A self-loop is short-circuited
/// to `false` because the loop's tail can never have its in-degree
/// reduced past itself.
///
/// Counterpart of `igraph_is_dag` from
/// `references/igraph/src/properties/dag.c:151`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_dag};
///
/// // Linear chain 0 → 1 → 2 — a DAG.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert!(is_dag(&g));
///
/// // Cycle 0 → 1 → 2 → 0 — not a DAG.
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert!(!is_dag(&g));
/// ```
#[must_use]
pub fn is_dag(graph: &Graph) -> bool {
    if !graph.is_directed() {
        return false;
    }
    if let Some(v) = graph.cache_get(CachedProperty::IsDag) {
        return v;
    }

    let n = graph.vcount();
    let n_us = n as usize;

    // In-degree of every vertex. `incident_in(v)` returns the in-edges
    // on a directed graph; its length is the in-degree.
    let mut in_deg: Vec<u32> = Vec::with_capacity(n_us);
    for v in 0..n {
        let deg = u32::try_from(
            graph
                .incident_in(v)
                .expect("incident_in on valid vertex")
                .len(),
        )
        .unwrap_or(u32::MAX);
        in_deg.push(deg);
    }

    // Queue of vertices with current in-degree 0.
    let mut sources: VecDeque<VertexId> = VecDeque::new();
    for v in 0..n {
        if in_deg[v as usize] == 0 {
            sources.push_back(v);
        }
    }

    let mut peeled: u32 = 0;
    while let Some(v) = sources.pop_front() {
        peeled = peeled.saturating_add(1);

        // Walk out-edges; for each neighbour, drop its in-degree.
        let out_eids = graph.incident(v).expect("incident on valid vertex");
        for eid in out_eids {
            let nei = graph.edge_other(eid, v).expect("edge_other on valid edge");
            if nei == v {
                // Self-loop: vertex v depends on itself; it can
                // never be peeled away. The graph has a cycle.
                return false;
            }
            let nei_idx = nei as usize;
            in_deg[nei_idx] = in_deg[nei_idx].saturating_sub(1);
            if in_deg[nei_idx] == 0 {
                sources.push_back(nei);
            }
        }
    }

    let result = peeled == n;
    graph.cache_set(CachedProperty::IsDag, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Graph;

    #[test]
    fn empty_directed_graph_is_dag() {
        let g = Graph::new(0, true).unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn isolated_vertices_only_is_dag() {
        let g = Graph::new(5, true).unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn linear_chain_is_dag() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn binary_tree_is_dag() {
        // 0 → 1, 0 → 2, 1 → 3, 1 → 4.
        let mut g = Graph::new(5, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (1, 4)])
            .unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn three_cycle_is_not_dag() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(!is_dag(&g));
    }

    #[test]
    fn self_loop_is_not_dag() {
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(!is_dag(&g));
    }

    #[test]
    fn undirected_graph_is_not_dag_even_if_acyclic() {
        // Undirected path 0-1-2 — acyclic as a graph but the
        // upstream contract is "DAGs are directed".
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
        assert!(!is_dag(&g));
    }

    #[test]
    fn dag_with_multiple_roots() {
        // Two disjoint DAG branches feeding into a sink.
        // 0 → 2, 1 → 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn dag_with_parallel_edges_still_dag() {
        // Parallel edges 0→1 don't introduce a cycle.
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert!(is_dag(&g));
    }

    #[test]
    fn cycle_with_extra_tail_branches_not_dag() {
        // Cycle 0→1→2→0 plus a tail 3→0. The tail can be peeled
        // but the cycle remains, so the graph is not a DAG.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0), (3, 0)])
            .unwrap();
        assert!(!is_dag(&g));
    }

    #[test]
    fn back_edge_to_root_is_not_dag() {
        // 0 → 1 → 2 → 0 (3-cycle) — different framing of the
        // three_cycle case, double-check the back edge is detected.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(!is_dag(&g));
    }

    #[test]
    fn two_disjoint_dags_is_dag() {
        // Two unrelated 2-vertex DAGs in the same graph.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
        assert!(is_dag(&g));
    }
}
