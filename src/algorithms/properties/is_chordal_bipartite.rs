//! Chordal bipartite graph predicate (ALGO-PR-111).
//!
//! A bipartite graph is chordal bipartite if every cycle of length ≥ 6
//! has a chord. Equivalently, a bipartite graph with no induced `C_6`
//! or longer. This is NOT the same as being both chordal and bipartite
//! (which would force a forest).
//!
//! Non-bipartite and directed graphs return `false`.

use crate::algorithms::connectivity::{ConnectednessMode, is_connected};
use crate::algorithms::properties::is_bipartite::is_bipartite;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is chordal bipartite.
///
/// A bipartite graph is chordal bipartite if it has no induced cycle
/// of length ≥ 6. Returns `false` for non-bipartite or directed graphs.
///
/// Uses a perfect elimination ordering (PEO) approach on the bipartite
/// adjacency: repeatedly finds a vertex whose neighborhood forms a
/// "bisimplicial" structure and removes it.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_chordal_bipartite};
///
/// // Complete bipartite `K_{2,3}` is chordal bipartite
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(0, 3).unwrap();
/// g.add_edge(0, 4).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(1, 4).unwrap();
/// assert!(is_chordal_bipartite(&g).unwrap());
///
/// // `C_6` is bipartite but NOT chordal bipartite
/// let mut g = Graph::with_vertices(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 5).unwrap();
/// g.add_edge(5, 0).unwrap();
/// assert!(!is_chordal_bipartite(&g).unwrap());
/// ```
pub fn is_chordal_bipartite(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount();
    if n <= 4 {
        // Any bipartite graph on ≤ 4 vertices is chordal bipartite
        // (need ≥ 6 vertices for C_6).
        return is_bipartite(graph).map(|r| r.is_bipartite);
    }

    let bip = is_bipartite(graph)?;
    if !bip.is_bipartite {
        return Ok(false);
    }

    // For disconnected graphs, each component must be chordal bipartite.
    if !is_connected(graph, ConnectednessMode::Weak)? {
        return check_components_chordal_bipartite(graph);
    }

    check_single_component(graph, n)
}

/// Check each connected component independently.
fn check_components_chordal_bipartite(graph: &Graph) -> IgraphResult<bool> {
    let n = graph.vcount();
    let n_usize = n as usize;
    let mut visited = vec![false; n_usize];
    let mut nbrs_cache: Vec<Vec<u32>> = Vec::with_capacity(n_usize);
    for v in 0..n {
        nbrs_cache.push(graph.neighbors(v)?);
    }

    for start in 0..n_usize {
        if visited[start] {
            continue;
        }
        // BFS to find component
        let mut component = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        visited[start] = true;
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            component.push(u);
            for &w in &nbrs_cache[u] {
                let wi = w as usize;
                if !visited[wi] {
                    visited[wi] = true;
                    queue.push_back(wi);
                }
            }
        }

        if component.len() >= 6 && !check_component_chordal_bipartite(&nbrs_cache, &component) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Check a single connected component for chordal bipartiteness.
fn check_component_chordal_bipartite(nbrs_cache: &[Vec<u32>], component: &[usize]) -> bool {
    // Build local adjacency for the component using a vertex map.
    let max_id = component.iter().copied().max().unwrap_or(0);
    let mut global_to_local = vec![usize::MAX; max_id + 1];
    for (i, &v) in component.iter().enumerate() {
        global_to_local[v] = i;
    }

    let cn = component.len();
    let mut adj = vec![vec![false; cn]; cn];
    for &v in component {
        let li = global_to_local[v];
        for &w in &nbrs_cache[v] {
            let wi = w as usize;
            if wi <= max_id && global_to_local[wi] != usize::MAX {
                adj[li][global_to_local[wi]] = true;
            }
        }
    }

    is_chordal_bipartite_from_adj(&adj, cn)
}

/// Check a single connected graph using adjacency matrix.
fn check_single_component(graph: &Graph, n: u32) -> IgraphResult<bool> {
    let n_usize = n as usize;
    let mut adj = vec![vec![false; n_usize]; n_usize];
    for v in 0..n {
        let nbrs = graph.neighbors(v)?;
        for &w in &nbrs {
            adj[v as usize][w as usize] = true;
        }
    }
    Ok(is_chordal_bipartite_from_adj(&adj, n_usize))
}

/// Core algorithm: check chordal bipartiteness via bisimplicial
/// elimination. A bipartite graph is chordal bipartite iff it has a
/// perfect elimination ordering where each eliminated vertex is
/// bisimplicial (its neighbors form a complete bipartite subgraph
/// with their common neighborhood).
///
/// Simpler approach: check that no induced `C_6` exists. Since
/// checking all `C_{2k}` for k≥3 is expensive, we use the
/// bisimplicial elimination characterization.
fn is_chordal_bipartite_from_adj(adj: &[Vec<bool>], n: usize) -> bool {
    // A vertex v is bisimplicial if: for every pair of neighbors
    // u, w of v, every common neighbor of u and w (other than v)
    // that is on the same side as v is also a neighbor of v.
    // Equivalently: N(u) ∩ N(w) ⊆ N(v) ∪ {v} for all u,w ∈ N(v)
    // on one side, and the neighborhoods of all N(v) restricted to
    // v's side are nested (form a chain under inclusion).

    // Practical approach: iteratively remove bisimplicial vertices.
    // If we can remove all vertices, it's chordal bipartite.

    let mut removed = vec![false; n];
    let mut remaining = n;

    while remaining > 0 {
        let mut found = false;
        for v in 0..n {
            if removed[v] {
                continue;
            }

            if is_bisimplicial(adj, v, &removed, n) {
                removed[v] = true;
                remaining -= 1;
                found = true;
                break;
            }
        }

        if !found {
            return false;
        }
    }

    true
}

/// Check if vertex v is bisimplicial in the remaining graph.
/// In a bipartite graph, v is bisimplicial if the subgraph induced by
/// N(v) and (N(N(v)) \ {v}) is complete bipartite: every second-neighbor
/// of v must be adjacent to ALL vertices in N(v).
fn is_bisimplicial(adj: &[Vec<bool>], v: usize, removed: &[bool], n: usize) -> bool {
    let nbrs_v: Vec<usize> = (0..n).filter(|&u| !removed[u] && adj[v][u]).collect();

    if nbrs_v.is_empty() {
        return true;
    }

    // For each neighbor u of v, every active neighbor x of u (x ≠ v)
    // must be adjacent to ALL of N(v).
    for &u in &nbrs_v {
        for x in 0..n {
            if removed[x] || x == v || !adj[u][x] {
                continue;
            }
            // x is a second-neighbor of v via u — must connect to all of N(v)
            for &w in &nbrs_v {
                if !adj[x][w] {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn c4_chordal_bipartite() {
        // `C_4` is bipartite and chordal bipartite (no `C_6`)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn c6_not_chordal_bipartite() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn complete_bipartite_k23() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn complete_bipartite_k33() {
        let mut g = Graph::with_vertices(6);
        for i in 0..3u32 {
            for j in 3..6u32 {
                g.add_edge(i, j).unwrap();
            }
        }
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn tree_chordal_bipartite() {
        // Trees are bipartite and chordal bipartite (no cycles at all)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn triangle_not_bipartite() {
        // Triangle is not bipartite → not chordal bipartite
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(!is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn path_chordal_bipartite() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn star_chordal_bipartite() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(0, 3).unwrap();
        g.add_edge(0, 4).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn c8_not_chordal_bipartite() {
        let mut g = Graph::with_vertices(8);
        for i in 0..8u32 {
            g.add_edge(i, (i + 1) % 8).unwrap();
        }
        assert!(!is_chordal_bipartite(&g).unwrap());
    }

    #[test]
    fn disconnected_bipartite() {
        // Two `C_4` components → chordal bipartite
        let mut g = Graph::with_vertices(8);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 6).unwrap();
        g.add_edge(6, 7).unwrap();
        g.add_edge(7, 4).unwrap();
        assert!(is_chordal_bipartite(&g).unwrap());
    }
}
