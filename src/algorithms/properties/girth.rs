//! Girth — length of the shortest cycle (ALGO-PR-001).
//!
//! Counterpart of `igraph_girth()` from
//! `references/igraph/src/properties/girth.c:73-220`.
//!
//! Algorithm: BFS from each vertex. When the BFS frontier touches an
//! already-visited vertex, it has discovered a cycle whose length is
//! `actlevel + neilevel - 1` (with the convention that the root is at
//! level 1). The minimum across all starts is the girth.
//!
//! Implementation details that match upstream:
//! - Self-loops and parallel edges are ignored (cycles of length 1 or 2
//!   are not counted). We pre-build a *simple* neighbour list per vertex.
//! - Once any cycle is found, BFS is capped at `stoplevel` so subsequent
//!   starts can short-circuit. The first triangle (girth = 3) ends the
//!   outer loop entirely (no shorter cycle exists in a simple graph).
//! - Phase-1 minimal slice returns only the girth value (`Option<u32>`).
//!   The cycle-vertex reconstruction (upstream's optional `cycle` output)
//!   ships in PR-001b.

use std::collections::VecDeque;

use crate::core::{Graph, IgraphResult, VertexId};

/// Shortest cycle length in `graph`. Returns `None` for acyclic graphs.
///
/// Self-loops and parallel edges are ignored — only "real" cycles of
/// length ≥ 3 are considered. Counterpart of `igraph_girth(_, &girth, NULL)`
/// where `IGRAPH_INFINITY` for acyclic maps to `None`.
///
/// Edge directions are ignored (`IGRAPH_ALL` semantics).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, girth};
///
/// // Triangle: girth = 3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// assert_eq!(girth(&g).unwrap(), Some(3));
///
/// // Tree: no cycles.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// assert_eq!(girth(&g).unwrap(), None);
/// ```
pub fn girth(graph: &Graph) -> IgraphResult<Option<u32>> {
    let n = graph.vcount();
    if n < 3 {
        return Ok(None);
    }

    // Pre-build simple (no-loop, no-multi) neighbour lists once.
    // Equivalent to upstream's
    // `igraph_lazy_adjlist_init(_, _, IGRAPH_ALL, NO_LOOPS, NO_MULTIPLE)`.
    let n_us = n as usize;
    let mut adj: Vec<Vec<VertexId>> = Vec::with_capacity(n_us);
    for v in 0..n {
        let raw = graph.neighbors(v)?;
        let mut simple: Vec<VertexId> = raw.into_iter().filter(|&u| u != v).collect();
        simple.sort_unstable();
        simple.dedup();
        adj.push(simple);
    }

    // upstream uses `IGRAPH_INTEGER_MAX` as the "no cycle yet" sentinel.
    // We use `u32::MAX` for the same purpose.
    let mut mincirc: u32 = u32::MAX;
    let mut stoplevel: u32 = (n + 1).max(2);
    let mut triangle = false;

    let mut level: Vec<u32> = vec![0; n_us];
    let mut q: VecDeque<VertexId> = VecDeque::new();

    let mut node = 0u32;
    while !triangle && node < n {
        if node == 1 && mincirc == u32::MAX {
            // Per upstream: if the graph is connected and we found no cycle
            // starting from vertex 0, it's a tree → no cycles anywhere.
            let cc = crate::algorithms::connectivity::connected_components(graph)?;
            if cc.count == 1 {
                break;
            }
        }

        // Reset BFS state for this start vertex.
        q.clear();
        level.fill(0);
        q.push_back(node);
        level[node as usize] = 1;

        while let Some(actnode) = q.pop_front() {
            let actlevel = level[actnode as usize];
            if actlevel >= stoplevel {
                break;
            }
            let neilist = &adj[actnode as usize];
            for &nei in neilist {
                let neilevel = level[nei as usize];
                if neilevel != 0 {
                    if neilevel == actlevel - 1 {
                        // Tree edge back to BFS parent — not a cycle.
                        continue;
                    }
                    // Found a cycle of length `actlevel + neilevel - 1`.
                    stoplevel = neilevel;
                    let candidate = actlevel + neilevel - 1;
                    if candidate < mincirc {
                        mincirc = candidate;
                        if neilevel == 2 {
                            // Triangle — minimum possible in a simple graph.
                            triangle = true;
                        }
                    }
                    if neilevel == actlevel {
                        break;
                    }
                } else {
                    q.push_back(nei);
                    level[nei as usize] = actlevel + 1;
                }
            }
        }

        node += 1;
    }

    if mincirc == u32::MAX {
        Ok(None)
    } else {
        Ok(Some(mincirc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_has_no_girth() {
        let g = Graph::with_vertices(0);
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn singleton_has_no_girth() {
        let g = Graph::with_vertices(1);
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn isolated_vertices_have_no_girth() {
        let g = Graph::with_vertices(5);
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn tree_has_no_girth() {
        // Path 0-1-2-3-4: a tree.
        let mut g = Graph::with_vertices(5);
        for i in 0..4 {
            g.add_edge(i, i + 1).unwrap();
        }
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn triangle_girth_is_3() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(girth(&g).unwrap(), Some(3));
    }

    #[test]
    fn square_4_cycle_has_girth_4() {
        let mut g = Graph::with_vertices(4);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        assert_eq!(girth(&g).unwrap(), Some(4));
    }

    #[test]
    fn pentagon_has_girth_5() {
        let mut g = Graph::with_vertices(5);
        for i in 0..5u32 {
            g.add_edge(i, (i + 1) % 5).unwrap();
        }
        assert_eq!(girth(&g).unwrap(), Some(5));
    }

    #[test]
    fn k4_has_girth_3() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert_eq!(girth(&g).unwrap(), Some(3));
    }

    #[test]
    fn self_loop_does_not_count_as_cycle() {
        // Self-loop on 0 plus a 0-1 edge: no cycle of length >= 3.
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn parallel_edges_do_not_count_as_cycle() {
        // Two parallel edges between 0 and 1: no cycle of length >= 3
        // (multi-edges form a 2-cycle which we ignore per upstream).
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 1).unwrap();
        assert_eq!(girth(&g).unwrap(), None);
    }

    #[test]
    fn two_components_picks_smaller_girth() {
        // Component A: 0-1-2-3-0 (4-cycle, girth 4).
        // Component B: 4-5-6-4 (3-cycle, girth 3).
        // Overall girth: 3 (min of the two).
        let mut g = Graph::with_vertices(7);
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(6, 4).unwrap();
        assert_eq!(girth(&g).unwrap(), Some(3));
    }

    #[test]
    fn pendant_does_not_change_cycle_girth() {
        // Triangle + pendant edge to a fourth vertex.
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(girth(&g).unwrap(), Some(3));
    }
}
