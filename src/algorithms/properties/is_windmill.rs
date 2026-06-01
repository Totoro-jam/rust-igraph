//! Windmill graph predicate (ALGO-PR-092).
//!
//! A windmill graph `Wd(k, n)` is formed by joining n copies of the
//! complete graph `K_k` at a single shared vertex (the "hub"). It has
//! 1 + n*(k-1) vertices and n*k*(k-1)/2 edges.
//!
//! Special cases: `Wd(3, n)` is the friendship graph (n triangles
//! sharing a vertex). `Wd(2, n)` is a star `K_{1,n}`.
//!
//! For directed graphs, the function returns `false`.

use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is a windmill graph.
///
/// A windmill graph has a single hub vertex connected to n copies of
/// `K_k` (complete graphs of the same size k ≥ 2). The hub is the
/// only vertex shared across copies.
///
/// Returns `false` for directed graphs and non-simple graphs.
/// Returns `true` for `K_1` (trivially `Wd(k, 0)` for any k).
///
/// On success returns `Some((k, n))` where k is the clique size and
/// n is the number of cliques. Returns `None` if not a windmill.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_windmill};
///
/// // Friendship graph: 2 triangles sharing vertex 0
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(3, 4).unwrap();
/// assert_eq!(is_windmill(&g).unwrap(), Some((3, 2)));
/// ```
pub fn is_windmill(graph: &Graph) -> IgraphResult<Option<(u32, u32)>> {
    if graph.is_directed() {
        return Ok(None);
    }

    let n = graph.vcount();
    if n == 0 {
        return Ok(None);
    }
    if n == 1 {
        return Ok(Some((2, 0)));
    }

    if !is_simple(graph)? {
        return Ok(None);
    }

    // Find the hub: vertex with maximum degree
    let mut hub = 0u32;
    let mut hub_deg = graph.degree(0)?;
    for v in 1..n {
        let d = graph.degree(v)?;
        if d > hub_deg {
            hub_deg = d;
            hub = v;
        }
    }

    // The hub must be connected to all other vertices
    if hub_deg != (n - 1) as usize {
        return Ok(None);
    }

    // All non-hub vertices must have the same degree
    let mut non_hub_deg = 0usize;
    for v in 0..n {
        if v == hub {
            continue;
        }
        let d = graph.degree(v)?;
        if non_hub_deg == 0 {
            non_hub_deg = d;
        } else if d != non_hub_deg {
            return Ok(None);
        }
    }

    // In Wd(k, num_copies): non-hub vertices have degree k-1
    // (connected to k-2 other vertices in their clique + the hub)
    let k_minus_1 = non_hub_deg;
    if k_minus_1 == 0 {
        return Ok(None);
    }
    let k = u32::try_from(k_minus_1 + 1).unwrap_or(u32::MAX);

    // Number of copies
    let non_hub_count = (n - 1) as usize;
    if non_hub_count % k_minus_1 != 0 {
        return Ok(None);
    }
    let num_copies = u32::try_from(non_hub_count / k_minus_1).unwrap_or(u32::MAX);

    // Verify edge count: num_copies * k*(k-1)/2
    let k64 = u64::from(k);
    let expected_edges =
        u64::from(num_copies).saturating_mul(k64.saturating_mul(k64.saturating_sub(1)) / 2);
    if graph.ecount() as u64 != expected_edges {
        return Ok(None);
    }

    // Verify structure: each non-hub vertex's neighbors (excluding hub)
    // must form a clique of size k-2 among themselves, and all must be
    // neighbors of the hub (which they are since hub connects to everyone).
    // Additionally, no two non-hub vertices from different cliques should
    // be connected.
    // Assign cliques: pick an unvisited non-hub vertex, its non-hub
    // neighbors form its clique mates.
    let mut visited = vec![false; n as usize];
    visited[hub as usize] = true;

    for v in 0..n {
        if visited[v as usize] {
            continue;
        }

        let nbrs = graph.neighbors(v)?;
        let clique_mates: Vec<u32> = nbrs.iter().filter(|&&w| w != hub).copied().collect();

        if clique_mates.len() != k_minus_1 - 1 {
            return Ok(None);
        }

        // All clique mates must be unvisited (belong to same clique)
        for &mate in &clique_mates {
            if visited[mate as usize] {
                return Ok(None);
            }
        }

        // Verify clique mates are connected to each other
        for (i, &u) in clique_mates.iter().enumerate() {
            let u_nbrs = graph.neighbors(u)?;
            for &w in &clique_mates[i + 1..] {
                if !u_nbrs.contains(&w) {
                    return Ok(None);
                }
            }
        }

        // Mark all clique members as visited
        visited[v as usize] = true;
        for &mate in &clique_mates {
            visited[mate as usize] = true;
        }
    }

    Ok(Some((k, num_copies)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert_eq!(is_windmill(&g).unwrap(), None);
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert_eq!(is_windmill(&g).unwrap(), Some((2, 0)));
    }

    #[test]
    fn single_triangle_wd31() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), Some((3, 1)));
    }

    #[test]
    fn friendship_wd32() {
        // 2 triangles sharing vertex 0
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), Some((3, 2)));
    }

    #[test]
    fn friendship_wd33() {
        // 3 triangles sharing vertex 0
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(0, 6).unwrap();
        g.add_edge(5, 6).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), Some((3, 3)));
    }

    #[test]
    fn wd42() {
        // 2 copies of K4 sharing vertex 0
        // Copy 1: {0,1,2,3}, Copy 2: {0,4,5,6}
        let mut g = Graph::with_vertices(7);
        // Copy 1
        for i in 0..4u32 {
            for j in (i + 1)..4 {
                g.add_edge(i, j).unwrap();
            }
        }
        // Copy 2
        for &i in &[0u32, 4, 5, 6] {
            for &j in &[0u32, 4, 5, 6] {
                if i < j {
                    g.add_edge(i, j).unwrap();
                }
            }
        }
        assert_eq!(is_windmill(&g).unwrap(), Some((4, 2)));
    }

    #[test]
    fn star_is_wd2n() {
        // K_{1,3} = Wd(2, 3)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), Some((2, 3)));
    }

    #[test]
    fn path_not_windmill() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), None);
    }

    #[test]
    fn cycle_c4_not_windmill() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), None);
    }

    #[test]
    fn k4_is_wd41() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), Some((4, 1)));
    }

    #[test]
    fn directed_returns_none() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert_eq!(is_windmill(&g).unwrap(), None);
    }

    #[test]
    fn butterfly_not_windmill() {
        // Butterfly: two triangles sharing an edge, not a vertex
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(3, 1).unwrap();
        // This is actually NOT a butterfly (bowtie). Let me verify:
        // Vertices: 0,1,2,3. Edges: 0-1, 1-2, 2-0, 0-3, 3-1
        // Degrees: 0→3, 1→3, 2→2, 3→2
        // Two vertices have max degree 3 (not n-1=3, which is correct)
        // But they both share degree n-1=3, so hub candidate is ambiguous
        // Hub=0: non-hub neighbors all of degree? 1→3, 2→2, 3→2. Not uniform.
        assert_eq!(is_windmill(&g).unwrap(), None);
    }
}
