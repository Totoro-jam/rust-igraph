//! Wheel graph predicate (ALGO-PR-085).
//!
//! A wheel graph `W_n` (n ≥ 4 vertices) consists of a single hub
//! vertex adjacent to all other vertices, which themselves form a
//! cycle. Equivalently: one vertex of degree n-1, all other vertices
//! of degree 3, exactly 2(n-1) edges, and the graph is simple and
//! undirected.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a wheel graph.
///
/// A wheel graph has n ≥ 4 vertices: one hub of degree n-1 connected
/// to all others, and the remaining n-1 vertices form a cycle (each
/// has degree 3).
///
/// Returns `false` for directed graphs, non-simple graphs, and graphs
/// with fewer than 4 vertices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_wheel};
///
/// // W4: hub 0 + triangle 1-2-3
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 1).unwrap();
/// assert!(is_wheel(&g).unwrap());
///
/// // K4 is NOT a wheel (all vertices have degree 3, no unique hub)
/// // Actually K4 IS W4 — every vertex can serve as the hub.
/// // But a cycle C4 is NOT a wheel:
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
/// assert!(!is_wheel(&g).unwrap());
/// ```
pub fn is_wheel(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n < 4 {
        return Ok(false);
    }

    if !is_simple(graph)? {
        return Ok(false);
    }

    // A wheel on n vertices has exactly 2(n-1) edges
    let n_usize = n as usize;
    let expected_edges = 2 * (n_usize - 1);
    if graph.ecount() != expected_edges {
        return Ok(false);
    }

    // Find the hub: exactly one vertex with degree n-1
    let mut hub: Option<u32> = None;
    for v in 0..n {
        let deg = graph.degree(v)?;
        if deg == n_usize - 1 {
            if hub.is_some() {
                // Multiple hubs → check if it's still a valid wheel
                // (K4 has all vertices with degree 3 = n-1, and K4 IS W4)
                // We allow multiple candidate hubs; we just need ONE that works.
                // We'll verify below.
            }
            if hub.is_none() {
                hub = Some(v);
            }
        }
    }

    let Some(hub) = hub else {
        return Ok(false);
    };

    // Verify: all non-hub vertices have degree 3
    for v in 0..n {
        if v == hub {
            continue;
        }
        if graph.degree(v)? != 3 {
            return Ok(false);
        }
    }

    // Verify: the non-hub vertices form a cycle.
    // Each non-hub vertex has 3 neighbors: the hub + 2 rim neighbors.
    // Walk the rim: start from any non-hub, follow rim edges (non-hub neighbors).
    let rim_start = u32::from(hub == 0);
    let rim_size = n - 1;

    let mut visited = vec![false; n as usize];
    visited[hub as usize] = true;
    visited[rim_start as usize] = true;

    // First step: rim_start has exactly 2 rim neighbors; pick the first.
    let first_nbrs = graph.neighbors(rim_start)?;
    let first_rim: Vec<u32> = first_nbrs.into_iter().filter(|&w| w != hub).collect();
    if first_rim.len() != 2 {
        return Ok(false);
    }

    let mut current = first_rim[0];
    visited[current as usize] = true;
    let mut count: u32 = 2;

    while count < rim_size {
        let nbrs = graph.neighbors(current)?;
        let rim_nbrs: Vec<u32> = nbrs
            .into_iter()
            .filter(|&w| w != hub && !visited[w as usize])
            .collect();

        if rim_nbrs.len() != 1 {
            return Ok(false);
        }

        current = rim_nbrs[0];
        visited[current as usize] = true;
        count = count.saturating_add(1);
    }

    // All rim vertices visited; the last must connect back to rim_start.
    let last_nbrs = graph.neighbors(current)?;
    Ok(last_nbrs.contains(&rim_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn three_vertices_too_small() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn w4_hub_0() {
        // Hub 0 connected to 1,2,3; rim: 1-2-3-1
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 1).unwrap();
        assert!(is_wheel(&g).unwrap());
    }

    #[test]
    fn w5() {
        // Hub 0 connected to 1,2,3,4; rim: 1-2-3-4-1
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 1).unwrap();
        assert!(is_wheel(&g).unwrap());
    }

    #[test]
    fn w6() {
        // Hub 0, rim: 1-2-3-4-5-1
        let mut g = Graph::with_vertices(6);
        for i in 1..6u32 {
            g.add_edge(0, i).unwrap();
        }
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 1).unwrap();
        assert!(is_wheel(&g).unwrap());
    }

    #[test]
    fn hub_not_vertex_0() {
        // Hub is vertex 3, rim: 0-1-2-0
        let mut g = Graph::with_vertices(4);
        g.add_edge(3, 0).unwrap();
        g.add_edge(3, 1).unwrap();
        g.add_edge(3, 2).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_wheel(&g).unwrap());
    }

    #[test]
    fn c4_not_wheel() {
        // C4: no hub vertex with degree n-1
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn star_not_wheel() {
        // Star K_{1,4}: hub has degree 4, leaves have degree 1 (not 3)
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(4, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 1).unwrap();
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn k4_is_wheel() {
        // K4 is isomorphic to W4 (any vertex can be the hub)
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_wheel(&g).unwrap());
    }

    #[test]
    fn k5_not_wheel() {
        // K5: 10 edges, but W5 has 8 edges → wrong edge count
        let mut g = Graph::with_vertices(5);
        for u in 0..5u32 {
            for v in (u + 1)..5 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn broken_rim_not_wheel() {
        // Hub 0 + rim 1,2,3,4 but rim has wrong connectivity
        // (1-2, 2-3, 3-4, but 4-1 missing, 1-3 added instead)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(1, 3).unwrap();
        assert!(!is_wheel(&g).unwrap());
    }

    #[test]
    fn petersen_not_wheel() {
        // Petersen: 3-regular, 10 vertices, no vertex has degree 9
        let mut g = Graph::with_vertices(10);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(5, 7).unwrap();
        g.add_edge(7, 9).unwrap();
        g.add_edge(9, 6).unwrap();
        g.add_edge(6, 8).unwrap();
        g.add_edge(8, 5).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(1, 6).unwrap();
        g.add_edge(2, 7).unwrap();
        g.add_edge(3, 8).unwrap();
        g.add_edge(4, 9).unwrap();
        assert!(!is_wheel(&g).unwrap());
    }
}
