//! Forest predicate (ALGO-PR-024).
//!
//! Counterpart of `igraph_is_forest()` from
//! `references/igraph/src/properties/trees.c:520`. Returns
//! `Some(roots)` iff `graph` is a forest under `mode`, otherwise
//! `None`. Unlike [`crate::is_tree`], the null graph (`vcount == 0`)
//! **is** considered a forest by convention (with an empty root
//! list).
//!
//! For directed graphs the [`DijkstraMode`] argument selects the
//! orientation:
//! - [`DijkstraMode::Out`]: out-forest — every tree component is an
//!   out-arborescence; roots are vertices with in-degree `0`.
//! - [`DijkstraMode::In`]: in-forest — every tree component is an
//!   in-arborescence; roots are vertices with out-degree `0`.
//! - [`DijkstraMode::All`]: edge directions ignored; roots are
//!   the canonical (lowest-id) vertex of each connected component.
//!
//! For undirected graphs the mode argument is ignored (matches
//! upstream).
//!
//! Time complexity: `O(V + E)`.
//!
//! Note: `is_forest` and [`crate::is_acyclic`] differ for directed
//! graphs. `is_acyclic` only forbids *directed* cycles, whereas
//! `is_forest(_, Out)` additionally forbids in-degree `> 1` (i.e.
//! a vertex receiving from two parents — an *undirected* cycle).
//! Example: `0 → 2 ← 1` is acyclic but is **not** an out-forest
//! because vertex 2 has in-degree 2.

use crate::algorithms::paths::dijkstra::DijkstraMode;
use crate::core::Graph;
use crate::core::cache::CachedProperty;
use crate::core::error::IgraphResult;
use crate::core::graph::{EdgeId, VertexId};

/// Returns `Some(roots)` iff `graph` is a forest under `mode`,
/// otherwise `None`. The null graph is a forest with empty roots.
///
/// For undirected graphs the mode argument is ignored. For directed
/// graphs:
/// - [`DijkstraMode::Out`]: roots are the vertices with in-degree
///   `0` (one per tree component).
/// - [`DijkstraMode::In`]: roots are the vertices with out-degree
///   `0` (one per tree component).
/// - [`DijkstraMode::All`]: orientation ignored; the canonical root
///   of each component is the lowest-indexed unvisited vertex
///   reached during a left-to-right scan.
///
/// Counterpart of `igraph_is_forest(_, _, _, mode)` from
/// `references/igraph/src/properties/trees.c:520`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_forest, DijkstraMode};
///
/// // Two disjoint undirected edges: 0-1 and 2-3 form a forest.
/// let mut g = Graph::with_vertices(4);
/// g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
/// let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
/// assert_eq!(roots, vec![0, 2]);
///
/// // Triangle 0-1-2-0 has a cycle ⇒ not a forest.
/// let mut g = Graph::with_vertices(3);
/// g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
/// assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
///
/// // Two disjoint out-arborescences 0→1 and 2→3.
/// let mut g = Graph::new(4, true).unwrap();
/// g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
/// let roots = is_forest(&g, DijkstraMode::Out).unwrap().unwrap();
/// assert_eq!(roots, vec![0, 2]);
///
/// // 0→2←1: not an out-forest (in-degree 2 at vertex 2),
/// // but is an undirected forest (treated as 0-2-1 path).
/// let mut g = Graph::new(3, true).unwrap();
/// g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
/// assert!(is_forest(&g, DijkstraMode::Out).unwrap().is_none());
/// assert!(is_forest(&g, DijkstraMode::All).unwrap().is_some());
/// ```
pub fn is_forest(graph: &Graph, mode: DijkstraMode) -> IgraphResult<Option<Vec<VertexId>>> {
    let n = graph.vcount();
    let m = graph.ecount();

    // Null graph: forest with empty roots.
    if n == 0 {
        if matches!(
            (graph.is_directed(), mode),
            (false, _) | (true, DijkstraMode::All)
        ) {
            graph.cache_set(CachedProperty::IsForest, true);
        }
        return Ok(Some(Vec::new()));
    }

    // No edges: every vertex is its own tree, all are roots.
    if m == 0 {
        if matches!(
            (graph.is_directed(), mode),
            (false, _) | (true, DijkstraMode::All)
        ) {
            graph.cache_set(CachedProperty::IsForest, true);
        }
        return Ok(Some((0..n).collect()));
    }

    let directed = graph.is_directed();
    let effective_mode = if directed { mode } else { DijkstraMode::All };
    let cache_eligible = matches!(effective_mode, DijkstraMode::All);

    if cache_eligible {
        if let Some(false) = graph.cache_get(CachedProperty::IsForest) {
            return Ok(None);
        }
    }

    // Forest has at most n-1 edges.
    if m as u64 > u64::from(n).saturating_sub(1) {
        if cache_eligible {
            graph.cache_set(CachedProperty::IsForest, false);
        }
        return Ok(None);
    }

    let n_us = n as usize;
    let mut visited = vec![false; n_us];
    let mut visited_count: u32 = 0;

    let roots_opt = collect_roots_and_traverse(
        graph,
        n,
        effective_mode,
        directed,
        &mut visited,
        &mut visited_count,
    )?;

    let Some(roots) = roots_opt else {
        if cache_eligible {
            graph.cache_set(CachedProperty::IsForest, false);
        }
        return Ok(None);
    };

    // All vertices must be reachable from some root.
    if visited_count == n {
        if cache_eligible {
            graph.cache_set(CachedProperty::IsForest, true);
        }
        Ok(Some(roots))
    } else {
        if cache_eligible {
            graph.cache_set(CachedProperty::IsForest, false);
        }
        Ok(None)
    }
}

/// Iterate vertices in id order, picking roots per `mode` and
/// running an `is_forest_visitor` DFS from each. Returns `None`
/// the moment a cycle, an arity violation, or a self-loop is
/// detected.
fn collect_roots_and_traverse(
    graph: &Graph,
    n: VertexId,
    mode: DijkstraMode,
    directed: bool,
    visited: &mut [bool],
    visited_count: &mut u32,
) -> IgraphResult<Option<Vec<VertexId>>> {
    let mut roots: Vec<VertexId> = Vec::new();

    match mode {
        DijkstraMode::All => {
            for v in 0..n {
                if !visited[v as usize] {
                    roots.push(v);
                    if !is_forest_visitor(graph, v, mode, directed, visited, visited_count)? {
                        return Ok(None);
                    }
                }
            }
        }
        DijkstraMode::Out | DijkstraMode::In => {
            // For an out-tree, every vertex has in-degree ≤ 1
            // (roots: 0). For an in-tree, every vertex has
            // out-degree ≤ 1 (roots: 0). Counter-direction degree.
            let use_in_degree = matches!(mode, DijkstraMode::Out);
            for v in 0..n {
                let d = counter_degree(graph, v, use_in_degree)?;
                if d > 1 {
                    return Ok(None);
                }
                if d == 0 {
                    roots.push(v);
                    if !is_forest_visitor(graph, v, mode, directed, visited, visited_count)? {
                        return Ok(None);
                    }
                }
            }
        }
    }

    Ok(Some(roots))
}

/// Counter-direction degree (in-degree if `use_in_degree`, else
/// out-degree). Counts every incidence including parallel edges
/// and self-loops once per stored entry; this matches upstream's
/// `igraph_degree(_, IGRAPH_REVERSE_MODE(mode), IGRAPH_LOOPS)`
/// behaviour.
fn counter_degree(graph: &Graph, v: VertexId, use_in_degree: bool) -> IgraphResult<usize> {
    if use_in_degree {
        Ok(graph.incident_in(v)?.len())
    } else {
        Ok(graph.incident(v)?.len())
    }
}

/// DFS from `root` in the given `mode`. Returns `false` if a cycle
/// (or self-loop in undirected) is detected; `true` otherwise. On
/// the way it sets `visited[]` and increments `visited_count`.
///
/// Mirrors `igraph_i_is_forest_visitor` from
/// `references/igraph/src/properties/trees.c:402`.
fn is_forest_visitor(
    graph: &Graph,
    root: VertexId,
    mode: DijkstraMode,
    directed: bool,
    visited: &mut [bool],
    visited_count: &mut u32,
) -> IgraphResult<bool> {
    let mut stack: Vec<VertexId> = Vec::new();
    stack.push(root);

    while let Some(u) = stack.pop() {
        if visited[u as usize] {
            // We popped a vertex that was already marked: we
            // re-entered it through a back edge (non-tree edge).
            // That's a cycle.
            return Ok(false);
        }
        visited[u as usize] = true;
        *visited_count = visited_count.saturating_add(1);

        for eid in incident_edges(graph, u, mode, directed)? {
            let nei = graph.edge_other(eid, u)?;
            match mode {
                DijkstraMode::All => {
                    if !visited[nei as usize] {
                        stack.push(nei);
                    } else if nei == u {
                        // self-loop in the undirected case
                        return Ok(false);
                    }
                    // else: skip the parent edge.
                }
                DijkstraMode::Out | DijkstraMode::In => {
                    // Push every neighbour; cycle is detected by
                    // popping a vertex that's already visited.
                    stack.push(nei);
                }
            }
        }
    }

    Ok(true)
}

/// Edges incident on `u` in the requested orientation. (Re-uses
/// the same dispatch table as `is_tree`.)
fn incident_edges(
    graph: &Graph,
    u: VertexId,
    mode: DijkstraMode,
    directed: bool,
) -> IgraphResult<Vec<EdgeId>> {
    match mode {
        DijkstraMode::Out => graph.incident(u),
        DijkstraMode::In => graph.incident_in(u),
        DijkstraMode::All => {
            if directed {
                let mut out = graph.incident(u)?;
                out.extend(graph.incident_in(u)?);
                Ok(out)
            } else {
                graph.incident(u)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------- Null / no-edge --------

    #[test]
    fn null_graph_is_forest_with_empty_roots() {
        let g = Graph::with_vertices(0);
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert!(roots.is_empty());
    }

    #[test]
    fn no_edges_undirected_every_vertex_is_a_root() {
        let g = Graph::with_vertices(4);
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert_eq!(roots, vec![0, 1, 2, 3]);
    }

    #[test]
    fn no_edges_directed_out_every_vertex_is_a_root() {
        let g = Graph::new(3, true).unwrap();
        let roots = is_forest(&g, DijkstraMode::Out).unwrap().unwrap();
        assert_eq!(roots, vec![0, 1, 2]);
    }

    // -------- Undirected --------

    #[test]
    fn undirected_path_is_forest_one_root() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert_eq!(roots, vec![0]);
    }

    #[test]
    fn undirected_two_components_each_a_tree() {
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (2, 3), (3, 4)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert_eq!(roots, vec![0, 2]);
    }

    #[test]
    fn undirected_triangle_is_not_a_forest() {
        let mut g = Graph::with_vertices(3);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    #[test]
    fn undirected_self_loop_breaks_forest() {
        // (0,0) self-loop on vertex 0; rest is forest.
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        // ecount=2, vcount=3 ⇒ ecount ≤ vcount-1 holds. But the
        // self-loop is detected during the DFS visitor.
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    #[test]
    fn undirected_parallel_edges_break_forest() {
        // Two parallel edges 0-1 form a 2-cycle.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        // ecount=2 > vcount-1=1 ⇒ failed by the cardinality bound.
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    // -------- Directed: OUT --------

    #[test]
    fn directed_out_two_arborescences_are_a_forest() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (2, 3)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::Out).unwrap().unwrap();
        assert_eq!(roots, vec![0, 2]);
    }

    #[test]
    fn directed_out_v_pattern_is_not_an_out_forest() {
        // 0→2, 1→2: vertex 2 has in-degree 2.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        assert!(is_forest(&g, DijkstraMode::Out).unwrap().is_none());
    }

    #[test]
    fn directed_in_two_anti_arborescences_are_a_forest() {
        // 1→0, 3→2: every edge points to a sink.
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(1u32, 0u32), (3, 2)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::In).unwrap().unwrap();
        assert_eq!(roots, vec![0, 2]);
    }

    #[test]
    fn directed_cycle_is_not_a_forest_in_any_mode() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 0)]).unwrap();
        assert!(is_forest(&g, DijkstraMode::Out).unwrap().is_none());
        assert!(is_forest(&g, DijkstraMode::In).unwrap().is_none());
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    #[test]
    fn directed_all_mode_treats_as_undirected() {
        // 0→2←1 — not an out-forest, but undirected it's a path.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edges(vec![(0u32, 2u32), (1, 2)]).unwrap();
        assert!(is_forest(&g, DijkstraMode::Out).unwrap().is_none());
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert_eq!(roots, vec![0]);
    }

    // -------- Edge-cardinality fast path --------

    #[test]
    fn too_many_edges_is_not_a_forest() {
        // 5 vertices, 5 edges ⇒ ecount > vcount-1 ⇒ not forest.
        let mut g = Graph::with_vertices(5);
        g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 4), (4, 0)])
            .unwrap();
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    #[test]
    fn correct_edge_count_but_disconnected_with_cycle_is_not_a_forest() {
        // vcount=4, ecount=3=vcount-1. Edges (0,1),(1,0),(2,3) ⇒
        // multi-edge between 0 and 1 forms a cycle, vertex 3
        // remains reachable from 2.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_forest(&g, DijkstraMode::All).unwrap().is_none());
    }

    // -------- Single tree is also a forest --------

    #[test]
    fn single_tree_is_also_a_forest() {
        let mut g = Graph::with_vertices(4);
        g.add_edges(vec![(0u32, 1u32), (0, 2), (0, 3)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::All).unwrap().unwrap();
        assert_eq!(roots, vec![0]);
    }

    #[test]
    fn single_out_arborescence_is_also_an_out_forest() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3)]).unwrap();
        let roots = is_forest(&g, DijkstraMode::Out).unwrap().unwrap();
        assert_eq!(roots, vec![0]);
    }
}
