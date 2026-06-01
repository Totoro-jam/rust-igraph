//! Distance-hereditary graph predicate (ALGO-PR-079).
//!
//! A connected graph is distance-hereditary if for every pair of
//! vertices, the distance between them in every connected induced
//! subgraph containing both equals the distance in the full graph.
//!
//! Equivalently, a connected graph is distance-hereditary iff it can
//! be reduced to a single vertex by repeatedly:
//! 1. Removing a pendant vertex (degree 1), or
//! 2. Removing a "false twin" (two non-adjacent vertices with
//!    identical open neighborhoods), or
//! 3. Removing a "true twin" (two adjacent vertices with identical
//!    closed neighborhoods, i.e., same open neighborhood plus each
//!    other).
//!
//! For disconnected graphs, each component must be distance-hereditary.
//! For directed graphs, the function returns `false`.

use crate::algorithms::connectivity::components::connected_components;
use crate::core::{Graph, IgraphResult};

/// Check whether a graph is distance-hereditary.
///
/// Uses a pruning algorithm: repeatedly remove pendant vertices
/// (degree 1) and twin vertices until the graph is empty or no more
/// removals are possible. O(V^2) in the worst case.
///
/// Returns `false` for directed graphs.
///
/// An empty graph (0 vertices) is distance-hereditary (vacuously).
/// A single vertex is distance-hereditary.
/// Any tree is distance-hereditary.
/// Any block graph is distance-hereditary.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_distance_hereditary};
///
/// // Tree: distance-hereditary
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_distance_hereditary(&g).unwrap());
///
/// // C5 is NOT distance-hereditary
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
/// assert!(!is_distance_hereditary(&g).unwrap());
/// ```
pub fn is_distance_hereditary(graph: &Graph) -> IgraphResult<bool> {
    if graph.is_directed() {
        return Ok(false);
    }

    let n = graph.vcount() as usize;
    if n <= 2 {
        return Ok(true);
    }

    let cc = connected_components(graph)?;
    for comp_id in 0..cc.count {
        let comp_verts: Vec<u32> = (0..graph.vcount())
            .filter(|&v| cc.membership[v as usize] == comp_id)
            .collect();
        if !is_component_distance_hereditary(graph, &comp_verts)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn is_component_distance_hereditary(graph: &Graph, vertices: &[u32]) -> IgraphResult<bool> {
    if vertices.len() <= 2 {
        return Ok(true);
    }

    let n = graph.vcount() as usize;
    let mut removed = vec![false; n];
    let mut remaining = vertices.len();

    loop {
        if remaining <= 2 {
            return Ok(true);
        }

        let mut progress = false;

        for &v in vertices {
            if removed[v as usize] {
                continue;
            }

            let nbrs = graph.neighbors(v)?;
            let active_nbrs: Vec<u32> =
                nbrs.into_iter().filter(|&w| !removed[w as usize]).collect();
            let deg = active_nbrs.len();

            if deg == 0 {
                removed[v as usize] = true;
                remaining = remaining.saturating_sub(1);
                progress = true;
                continue;
            }

            if deg == 1 {
                removed[v as usize] = true;
                remaining = remaining.saturating_sub(1);
                progress = true;
                continue;
            }

            if find_and_remove_twin(graph, v, &active_nbrs, &mut removed, n)? {
                remaining = remaining.saturating_sub(1);
                progress = true;
            }
        }

        if !progress {
            return Ok(false);
        }
    }
}

fn find_and_remove_twin(
    graph: &Graph,
    v: u32,
    v_nbrs: &[u32],
    removed: &mut [bool],
    _n: usize,
) -> IgraphResult<bool> {
    let mut v_nbr_set: Vec<u32> = v_nbrs.to_vec();
    v_nbr_set.sort_unstable();

    for &w in v_nbrs {
        if removed[w as usize] || w <= v {
            continue;
        }

        let w_all_nbrs = graph.neighbors(w)?;
        let mut w_nbrs: Vec<u32> = w_all_nbrs
            .into_iter()
            .filter(|&u| !removed[u as usize])
            .collect();
        w_nbrs.sort_unstable();

        // True twins: v and w are adjacent, and N(v)\{w} == N(w)\{v}
        let v_without_w: Vec<u32> = v_nbr_set.iter().copied().filter(|&x| x != w).collect();
        let w_without_v: Vec<u32> = w_nbrs.iter().copied().filter(|&x| x != v).collect();
        if v_without_w == w_without_v {
            removed[v as usize] = true;
            return Ok(true);
        }
    }

    // Check false twins: v and w are non-adjacent, N(v) == N(w)
    for &candidate in v_nbrs {
        if removed[candidate as usize] {
            continue;
        }
        let candidate_nbrs = graph.neighbors(candidate)?;
        for &w in &candidate_nbrs {
            if removed[w as usize] || w == v || w <= v {
                continue;
            }
            if v_nbr_set.binary_search(&w).is_ok() {
                continue;
            }

            let w_all_nbrs = graph.neighbors(w)?;
            let mut w_nbrs: Vec<u32> = w_all_nbrs
                .into_iter()
                .filter(|&u| !removed[u as usize])
                .collect();
            w_nbrs.sort_unstable();

            if v_nbr_set == w_nbrs {
                removed[v as usize] = true;
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn single_edge() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn path_is_dh() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn tree_is_dh() {
        let mut g = Graph::with_vertices(7);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(1, 4).unwrap();
        g.add_edge(2, 5).unwrap();
        g.add_edge(2, 6).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn triangle_is_dh() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn k4_is_dh() {
        let mut g = Graph::with_vertices(4);
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(u, v).unwrap();
            }
        }
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn star_is_dh() {
        let mut g = Graph::with_vertices(5);
        for i in 1..5u32 {
            g.add_edge(0, i).unwrap();
        }
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn cycle_4_is_dh() {
        // C4: actually IS distance-hereditary (no induced C5+)
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn cycle_5_not_dh() {
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 0).unwrap();
        assert!(!is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn cycle_6_not_dh() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        assert!(!is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn house_graph_not_dh() {
        // House: C4 (0-1-2-3) + top edge (0-2)... no.
        // House: square 0-1-2-3-0 + triangle roof 0-4-3
        // Wait — house is C5 with chord. Actually house = P5 vertices
        // with edges: 0-1, 1-2, 2-3, 3-4, 0-4, 0-3
        // Better: standard house = 5 vertices, square bottom + triangle top
        // 0-1, 1-2, 2-3, 3-0 (square), 2-4, 3-4 (triangle roof)
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 0).unwrap();
        g.add_edge(2, 4).unwrap();
        g.add_edge(3, 4).unwrap();
        assert!(!is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn edgeless_is_dh() {
        let g = Graph::with_vertices(5);
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn disconnected_trees_dh() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn directed_returns_false() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(!is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn two_triangles_sharing_vertex() {
        // Block graph → distance-hereditary
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 2).unwrap();
        assert!(is_distance_hereditary(&g).unwrap());
    }

    #[test]
    fn gem_not_dh() {
        // Gem = fan graph: P4 (0-1-2-3) + vertex 4 adjacent to all of P4
        let mut g = Graph::with_vertices(5);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(4, 0).unwrap();
        g.add_edge(4, 1).unwrap();
        g.add_edge(4, 2).unwrap();
        g.add_edge(4, 3).unwrap();
        assert!(!is_distance_hereditary(&g).unwrap());
    }
}
