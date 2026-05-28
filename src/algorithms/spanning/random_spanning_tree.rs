//! Random spanning tree via loop-erased random walk (ALGO-RST-001).
//!
//! Counterpart of `igraph_random_spanning_tree()` from
//! `references/igraph/src/misc/spanning_trees.c`.
//!
//! Uniformly samples spanning trees of a connected graph (or spanning
//! forests of a disconnected graph) using Wilson's algorithm (loop-erased
//! random walk). Edge directions are ignored.

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult};

/// Uniformly sample a random spanning tree (or forest) of a graph.
///
/// Uses loop-erased random walk (Wilson's algorithm). Edge directions
/// are ignored. Multi-edges are supported and affect sampling frequency.
///
/// If `start_vertex` is `Some(v)`, only the component containing `v`
/// is spanned; the result has `component_size - 1` edges.
/// If `start_vertex` is `None`, a random spanning forest of all
/// components is returned.
///
/// Returns a vector of edge IDs forming the spanning tree/forest.
///
/// # Errors
///
/// - `InvalidArgument` if `start_vertex` is out of range.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, random_spanning_tree};
///
/// // Triangle: any spanning tree has exactly 2 edges.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(0, 2).unwrap();
/// let tree = random_spanning_tree(&g, Some(0), 42).unwrap();
/// assert_eq!(tree.len(), 2);
/// ```
pub fn random_spanning_tree(
    graph: &Graph,
    start_vertex: Option<u32>,
    seed: u64,
) -> IgraphResult<Vec<u32>> {
    let vcount = graph.vcount();

    if let Some(v) = start_vertex {
        if v >= vcount {
            return Err(IgraphError::InvalidArgument(format!(
                "random_spanning_tree: vertex {v} out of range (vcount={vcount})"
            )));
        }
    }

    if vcount == 0 {
        return Ok(Vec::new());
    }

    let adj = build_incidence(graph)?;
    let mut rng = SplitMix64::new(seed);
    let mut visited = vec![false; vcount as usize];
    let mut result: Vec<u32> = Vec::new();

    if let Some(vid) = start_vertex {
        let comp_size = count_component(graph, vid, &adj)?;
        lerw(
            graph,
            &adj,
            vid,
            comp_size,
            &mut visited,
            &mut rng,
            &mut result,
        )?;
    } else {
        let components = find_components(vcount, &adj);
        for (root, comp_size) in components {
            lerw(
                graph,
                &adj,
                root,
                comp_size,
                &mut visited,
                &mut rng,
                &mut result,
            )?;
        }
    }

    Ok(result)
}

/// For each vertex, store a list of `(edge_id, other_vertex)` pairs,
/// treating the graph as undirected.
fn build_incidence(graph: &Graph) -> IgraphResult<Vec<Vec<(u32, u32)>>> {
    let vcount = graph.vcount();
    let ecount = graph.ecount();
    let mut inc: Vec<Vec<(u32, u32)>> = vec![Vec::new(); vcount as usize];

    for eid in 0..ecount {
        let eid_u32 = u32::try_from(eid).map_err(|_| IgraphError::Internal("edge id overflow"))?;
        let (from, to) = graph.edge(eid_u32)?;
        inc[from as usize].push((eid_u32, to));
        inc[to as usize].push((eid_u32, from));
    }

    Ok(inc)
}

/// Count vertices reachable from `start` treating graph as undirected.
fn count_component(graph: &Graph, start: u32, adj: &[Vec<(u32, u32)>]) -> IgraphResult<u32> {
    let vcount = graph.vcount();
    let mut visited = vec![false; vcount as usize];
    let mut queue = std::collections::VecDeque::new();
    visited[start as usize] = true;
    queue.push_back(start);
    let mut count: u32 = 1;

    while let Some(v) = queue.pop_front() {
        for &(_, nb) in &adj[v as usize] {
            if !visited[nb as usize] {
                visited[nb as usize] = true;
                count = count
                    .checked_add(1)
                    .ok_or(IgraphError::Internal("component size overflow"))?;
                queue.push_back(nb);
            }
        }
    }

    Ok(count)
}

/// Find one representative vertex and size for each connected component.
fn find_components(vcount: u32, adj: &[Vec<(u32, u32)>]) -> Vec<(u32, u32)> {
    let mut visited = vec![false; vcount as usize];
    let mut components: Vec<(u32, u32)> = Vec::new();

    for v in 0..vcount {
        if visited[v as usize] {
            continue;
        }
        let mut queue = std::collections::VecDeque::new();
        visited[v as usize] = true;
        queue.push_back(v);
        let mut size: u32 = 1;

        while let Some(u) = queue.pop_front() {
            for &(_, nb) in &adj[u as usize] {
                if !visited[nb as usize] {
                    visited[nb as usize] = true;
                    size = size.saturating_add(1);
                    queue.push_back(nb);
                }
            }
        }

        components.push((v, size));
    }

    components
}

/// Loop-erased random walk from `start` until all `comp_size` vertices
/// in the component are visited.
fn lerw(
    graph: &Graph,
    adj: &[Vec<(u32, u32)>],
    start: u32,
    comp_size: u32,
    visited: &mut [bool],
    rng: &mut SplitMix64,
    result: &mut Vec<u32>,
) -> IgraphResult<()> {
    let _ = graph;
    visited[start as usize] = true;
    let mut visited_count: u32 = 1;
    let mut current = start;

    while visited_count < comp_size {
        let edges = &adj[current as usize];
        if edges.is_empty() {
            break;
        }

        let idx = rng.gen_index(edges.len());
        let (eid, next) = edges[idx];

        if !visited[next as usize] {
            result.push(eid);
            visited[next as usize] = true;
            visited_count = visited_count
                .checked_add(1)
                .ok_or(IgraphError::Internal("visited count overflow"))?;
        }

        current = next;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_undirected(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_vertices(n);
        for &(u, v) in edges {
            g.add_edge(u, v).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let t = random_spanning_tree(&g, None, 0).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let t = random_spanning_tree(&g, Some(0), 0).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn single_edge() {
        let g = make_undirected(2, &[(0, 1)]);
        let t = random_spanning_tree(&g, Some(0), 0).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0], 0);
    }

    #[test]
    fn triangle() {
        let g = make_undirected(3, &[(0, 1), (1, 2), (0, 2)]);
        let t = random_spanning_tree(&g, Some(0), 42).unwrap();
        assert_eq!(t.len(), 2);
        // All edges should be valid edge IDs
        for &eid in &t {
            assert!(eid < 3);
        }
    }

    #[test]
    fn k4_complete() {
        let g = make_undirected(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let t = random_spanning_tree(&g, Some(0), 123).unwrap();
        assert_eq!(t.len(), 3); // spanning tree of K4 has 3 edges
    }

    #[test]
    fn path_graph() {
        // Path 0-1-2-3: only one spanning tree (the path itself)
        let g = make_undirected(4, &[(0, 1), (1, 2), (2, 3)]);
        let t = random_spanning_tree(&g, Some(0), 0).unwrap();
        assert_eq!(t.len(), 3);
        let mut sorted = t.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2]);
    }

    #[test]
    fn spanning_forest_disconnected() {
        // Two triangles: 0-1-2 and 3-4-5
        let g = make_undirected(6, &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)]);
        let t = random_spanning_tree(&g, None, 42).unwrap();
        // Forest should have 4 edges (2 per component)
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn start_vertex_component_only() {
        // Two triangles: 0-1-2 and 3-4-5
        let g = make_undirected(6, &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)]);
        // Only span the component containing vertex 0
        let t = random_spanning_tree(&g, Some(0), 42).unwrap();
        assert_eq!(t.len(), 2);
        // All edge IDs should be from the first component (0, 1, or 2)
        for &eid in &t {
            assert!(eid < 3);
        }
    }

    #[test]
    fn invalid_vertex_error() {
        let g = Graph::with_vertices(3);
        let err = random_spanning_tree(&g, Some(5), 0).unwrap_err();
        assert!(matches!(err, IgraphError::InvalidArgument(_)));
    }

    #[test]
    fn deterministic_with_same_seed() {
        let g = make_undirected(5, &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (2, 3), (3, 4)]);
        let t1 = random_spanning_tree(&g, Some(0), 999).unwrap();
        let t2 = random_spanning_tree(&g, Some(0), 999).unwrap();
        assert_eq!(t1, t2);
    }

    #[test]
    fn different_seeds_may_differ() {
        let g = make_undirected(
            5,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 3),
                (2, 3),
                (2, 4),
                (3, 4),
            ],
        );
        let mut different = false;
        for s in 0..20 {
            let t1 = random_spanning_tree(&g, Some(0), s).unwrap();
            let t2 = random_spanning_tree(&g, Some(0), s + 100).unwrap();
            if t1 != t2 {
                different = true;
                break;
            }
        }
        assert!(
            different,
            "with enough seeds, different trees should appear"
        );
    }

    #[test]
    fn result_forms_spanning_tree() {
        // Verify the result is a valid spanning tree: n-1 edges, connects all vertices
        let g = make_undirected(6, &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)]);
        let t = random_spanning_tree(&g, Some(0), 77).unwrap();
        assert_eq!(t.len(), 5); // 6 vertices - 1

        // Build adjacency of tree edges and verify connectivity
        let mut tree_adj: Vec<Vec<u32>> = vec![Vec::new(); 6];
        for &eid in &t {
            let (from, to) = g.edge(eid).unwrap();
            tree_adj[from as usize].push(to);
            tree_adj[to as usize].push(from);
        }

        // BFS to check connectivity
        let mut vis = [false; 6];
        let mut queue = std::collections::VecDeque::new();
        vis[0] = true;
        queue.push_back(0u32);
        let mut count = 1;
        while let Some(v) = queue.pop_front() {
            for &nb in &tree_adj[v as usize] {
                if !vis[nb as usize] {
                    vis[nb as usize] = true;
                    count += 1;
                    queue.push_back(nb);
                }
            }
        }
        assert_eq!(count, 6);
    }

    #[test]
    fn directed_graph_works() {
        // Edge directions are ignored
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let t = random_spanning_tree(&g, Some(0), 42).unwrap();
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn isolated_vertices_forest() {
        // 5 isolated vertices: forest has 0 edges
        let g = Graph::with_vertices(5);
        let t = random_spanning_tree(&g, None, 0).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn multi_edge_graph() {
        // Multi-edges: 0-1 appears twice, 1-2 once
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap(); // eid 0
        g.add_edge(0, 1).unwrap(); // eid 1
        g.add_edge(1, 2).unwrap(); // eid 2
        let t = random_spanning_tree(&g, Some(0), 42).unwrap();
        assert_eq!(t.len(), 2);
        // edge 2 (1-2) must always be in the tree
        assert!(t.contains(&2));
    }
}
