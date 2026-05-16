//! Eulerian path / cycle existence (ALGO-CC-040).
//!
//! Counterpart of `igraph_is_eulerian()` from
//! `references/igraph/src/paths/eulerian.c:333` (and its undirected /
//! directed helpers above). Determines whether the graph admits an
//! Eulerian path (visits every edge exactly once) and/or cycle (closed
//! Eulerian path).
//!
//! Algorithm (per upstream):
//! - Trivial: empty edge set or `n <= 1` → both true.
//! - Weak-connectivity check on the non-singleton part: at most one
//!   weak component may contain edges.
//! - Singleton accounting: at most one singleton-with-self-loops
//!   vertex may exist, and it cannot coexist with non-singleton edges.
//! - Undirected: count odd-degree vertices (self-loops count twice).
//!   `odd > 2` → neither; `odd == 2` → path only; `odd == 0` → both.
//! - Directed: each vertex must have `in_degree == out_degree` for a
//!   cycle; for a path one vertex may have `out - in == 1` and one
//!   `in - out == 1`. Larger imbalances → neither.
//!
//! Phase-1 minimal slice ships only the *existence* test. Concrete
//! path/cycle construction (Hierholzer) ships in CC-041 / CC-042.

use crate::core::{Graph, IgraphResult, VertexId};

/// Whether an Eulerian path or cycle exists in a graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EulerianClassification {
    /// `true` iff there exists a walk that traverses every edge exactly once.
    pub has_path: bool,
    /// `true` iff there exists a *closed* walk that traverses every edge
    /// exactly once. Implies `has_path`.
    pub has_cycle: bool,
}

/// Decide whether `graph` has an Eulerian path and/or cycle.
///
/// Counterpart of `igraph_is_eulerian()` from
/// `references/igraph/src/paths/eulerian.c:333`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_eulerian};
///
/// // Triangle (3-cycle): every vertex has even degree → both path & cycle.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let r = is_eulerian(&g).unwrap();
/// assert!(r.has_path && r.has_cycle);
///
/// // Path 0-1-2: vertices 0, 2 have odd degree → path only, no cycle.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// let r = is_eulerian(&g).unwrap();
/// assert!(r.has_path && !r.has_cycle);
/// ```
pub fn is_eulerian(graph: &Graph) -> IgraphResult<EulerianClassification> {
    let n = graph.vcount();
    let m = graph.ecount();
    if m == 0 || n <= 1 {
        return Ok(EulerianClassification {
            has_path: true,
            has_cycle: true,
        });
    }
    if graph.is_directed() {
        is_eulerian_directed(graph)
    } else {
        is_eulerian_undirected(graph)
    }
}

const NONE: EulerianClassification = EulerianClassification {
    has_path: false,
    has_cycle: false,
};

/// Connectivity precondition shared by the directed and undirected
/// variants: the graph may have at most one weak component of size > 1.
/// (Singletons — vertices with no incident edges or only self-loops —
/// don't affect Eulerianness on the connectivity axis.)
fn at_most_one_nonsingleton_component(graph: &Graph) -> IgraphResult<bool> {
    let cc = crate::algorithms::connectivity::connected_components(graph)?;
    let n = graph.vcount() as usize;
    let mut sizes = vec![0u32; cc.count as usize];
    for &m in &cc.membership {
        sizes[m as usize] = sizes[m as usize].saturating_add(1);
        let _ = n; // silence unused-binding warnings on smaller graphs
    }
    let big = sizes.into_iter().filter(|&s| s > 1).count();
    Ok(big <= 1)
}

/// Number of self-loops incident to `v` (counts each loop edge once).
fn self_loop_count(graph: &Graph, v: VertexId) -> IgraphResult<usize> {
    // For undirected graphs `neighbors(v)` reports each self-loop twice
    // (LOOPS_TWICE convention). Halve the count to recover edge multiplicity.
    let raw = graph.neighbors(v)?.iter().filter(|&&u| u == v).count();
    if graph.is_directed() {
        Ok(raw)
    } else {
        Ok(raw / 2)
    }
}

fn is_eulerian_undirected(graph: &Graph) -> IgraphResult<EulerianClassification> {
    if !at_most_one_nonsingleton_component(graph)? {
        return Ok(NONE);
    }

    let n = graph.vcount();

    let mut odd: u32 = 0;
    let mut singleton_self_loop: u32 = 0; // upstream `es`
    let mut has_nonsingleton_edge: u32 = 0; // upstream `ens`

    for v in 0..n {
        let total_deg = graph.degree(v)?; // LOOPS_TWICE
        if total_deg == 0 {
            continue;
        }
        let loops = self_loop_count(graph, v)?;
        // upstream IGRAPH_NO_LOOPS degree:
        //   undirected NO_LOOPS = LOOPS_TWICE - 2 * loop_count
        let nonloop_deg = total_deg - 2 * loops;
        if nonloop_deg == 0 {
            // singleton with self-loop(s)
            singleton_self_loop = singleton_self_loop.saturating_add(1);
        } else {
            has_nonsingleton_edge = 1;
            // Self-loops counted twice (LOOPS_TWICE), so total_deg's
            // parity already mirrors upstream's odd-degree check.
            if total_deg % 2 != 0 {
                odd = odd.saturating_add(1);
            }
        }
        if singleton_self_loop + has_nonsingleton_edge > 1 {
            return Ok(NONE);
        }
    }

    let (has_path, has_cycle) = match odd.cmp(&2) {
        std::cmp::Ordering::Greater => (false, false),
        std::cmp::Ordering::Equal => (true, false),
        std::cmp::Ordering::Less => (true, true),
    };
    Ok(EulerianClassification {
        has_path,
        has_cycle,
    })
}

fn is_eulerian_directed(graph: &Graph) -> IgraphResult<EulerianClassification> {
    if !at_most_one_nonsingleton_component(graph)? {
        return Ok(NONE);
    }
    let n = graph.vcount();

    let mut incoming_excess: u32 = 0;
    let mut outgoing_excess: u32 = 0;
    let mut singleton_self_loop: u32 = 0;
    let mut has_nonsingleton_edge: u32 = 0;

    for v in 0..n {
        let out_deg = graph.out_neighbors_vec(v)?.len();
        let in_deg = graph.in_neighbors_vec(v)?.len();
        if out_deg + in_deg == 0 {
            continue;
        }
        let loops = self_loop_count(graph, v)?;
        // upstream's `nonsingleton` is degree(IGRAPH_ALL, NO_LOOPS).
        // For directed, NO_LOOPS_ALL = (out_deg + in_deg) - 2 * loops.
        let nonloop_deg = out_deg + in_deg - 2 * loops;
        if nonloop_deg == 0 {
            singleton_self_loop = singleton_self_loop.saturating_add(1);
        } else {
            has_nonsingleton_edge = 1;
        }
        if singleton_self_loop + has_nonsingleton_edge > 1 {
            return Ok(NONE);
        }

        // Per-vertex in/out balance (self-loops contribute equally to both
        // — they cancel automatically).
        if in_deg == out_deg {
            continue;
        }
        if in_deg > out_deg {
            let diff = u32::try_from(in_deg - out_deg)
                .map_err(|_| crate::core::IgraphError::Internal("degree overflow"))?;
            incoming_excess = incoming_excess.saturating_add(diff);
        } else {
            let diff = u32::try_from(out_deg - in_deg)
                .map_err(|_| crate::core::IgraphError::Internal("degree overflow"))?;
            outgoing_excess = outgoing_excess.saturating_add(diff);
        }
        if incoming_excess > 1 || outgoing_excess > 1 {
            return Ok(NONE);
        }
    }

    let has_cycle = incoming_excess == 0 && outgoing_excess == 0;
    // upstream sets has_path = true at this point unconditionally:
    // any imbalance survived above must be the (1, 1) "path only" case.
    Ok(EulerianClassification {
        has_path: true,
        has_cycle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(graph: &Graph) -> EulerianClassification {
        is_eulerian(graph).unwrap()
    }

    #[test]
    fn empty_graph_is_eulerian() {
        let g = Graph::with_vertices(0);
        assert_eq!(
            classify(&g),
            EulerianClassification {
                has_path: true,
                has_cycle: true
            }
        );
    }

    #[test]
    fn single_isolated_vertex_is_eulerian() {
        let g = Graph::with_vertices(1);
        assert_eq!(
            classify(&g),
            EulerianClassification {
                has_path: true,
                has_cycle: true
            }
        );
    }

    #[test]
    fn isolated_vertices_no_edges_are_eulerian() {
        let g = Graph::with_vertices(5);
        assert_eq!(
            classify(&g),
            EulerianClassification {
                has_path: true,
                has_cycle: true
            }
        );
    }

    #[test]
    fn undirected_path_has_path_no_cycle() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = classify(&g);
        assert!(r.has_path);
        assert!(!r.has_cycle);
    }

    #[test]
    fn undirected_triangle_has_cycle_and_path() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = classify(&g);
        assert!(r.has_path);
        assert!(r.has_cycle);
    }

    #[test]
    fn undirected_disconnected_components_are_not_eulerian() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let r = classify(&g);
        assert!(!r.has_path);
        assert!(!r.has_cycle);
    }

    #[test]
    fn undirected_too_many_odd_vertices_is_not_eulerian() {
        // K4: every vertex has degree 3 → 4 odd vertices → not Eulerian.
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        let r = classify(&g);
        assert!(!r.has_path);
        assert!(!r.has_cycle);
    }

    #[test]
    fn undirected_self_loop_on_otherwise_eulerian_graph() {
        // Self-loop adds 2 to degree → keeps parities, so triangle + self-loop
        // is still Eulerian (path AND cycle).
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = classify(&g);
        assert!(r.has_path);
        assert!(r.has_cycle);
    }

    #[test]
    fn directed_cycle_has_cycle() {
        // 0 -> 1 -> 2 -> 0
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let r = classify(&g);
        assert!(r.has_path);
        assert!(r.has_cycle);
    }

    #[test]
    fn directed_path_has_path_no_cycle() {
        // 0 -> 1 -> 2: out_excess(0)=1, in_excess(2)=1 → path only.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let r = classify(&g);
        assert!(r.has_path);
        assert!(!r.has_cycle);
    }

    #[test]
    fn directed_too_imbalanced_is_not_eulerian() {
        // Vertex 0 has out_degree 2 in_degree 0; not Eulerian.
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        let r = classify(&g);
        assert!(!r.has_path);
        assert!(!r.has_cycle);
    }
}
