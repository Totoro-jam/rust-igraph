//! Clique and independent set tests (ALGO-PR-040).
//!
//! Tests whether a given set of vertices forms a clique (all pairs
//! adjacent) or an independent set (no pairs adjacent).
//!
//! Counterpart of `igraph_is_clique` and `igraph_is_independent_vertex_set`
//! from `references/igraph/src/properties/complete.c`.

use crate::core::{Graph, IgraphResult, VertexId};

/// Check whether a set of vertices forms a clique.
///
/// A clique is a set of vertices in which every pair is adjacent.
/// An empty set and a singleton set are always considered cliques.
///
/// For directed graphs, if `directed` is true, requires edges in both
/// directions (u→v and v→u) for every pair. If false, an edge in
/// either direction suffices.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_clique};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_clique(&g, &[0, 1, 2], false).unwrap());
/// assert!(!is_clique(&g, &[0, 1, 2, 3], false).unwrap());
/// ```
pub fn is_clique(graph: &Graph, vertices: &[VertexId], directed: bool) -> IgraphResult<bool> {
    let directed_eff = directed && graph.is_directed();
    check_subset(graph, vertices, directed_eff, false)
}

/// Check whether a set of vertices forms an independent set.
///
/// An independent set is a set of vertices in which no pair is adjacent.
/// An empty set and a singleton set are always considered independent sets.
///
/// Edge directions are ignored — any edge between two vertices in the
/// set disqualifies it.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, is_independent_vertex_set};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(2, 3).unwrap();
/// assert!(is_independent_vertex_set(&g, &[0, 2]).unwrap());
/// assert!(!is_independent_vertex_set(&g, &[0, 1]).unwrap());
/// ```
pub fn is_independent_vertex_set(graph: &Graph, vertices: &[VertexId]) -> IgraphResult<bool> {
    check_subset(graph, vertices, false, true)
}

fn check_subset(
    graph: &Graph,
    vertices: &[VertexId],
    directed: bool,
    independent_set: bool,
) -> IgraphResult<bool> {
    let n = vertices.len();
    if n <= 1 {
        return Ok(true);
    }

    let vcount = graph.vcount();
    for &v in vertices {
        if v >= vcount {
            return Err(crate::core::IgraphError::InvalidArgument(format!(
                "vertex {v} out of range (vcount = {vcount})"
            )));
        }
    }

    let ecount = graph.ecount();
    let mut out_adj: Vec<Vec<VertexId>> = vec![Vec::new(); vcount as usize];
    let mut in_adj: Vec<Vec<VertexId>> = vec![Vec::new(); vcount as usize];

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        out_adj[from as usize].push(to);
        if graph.is_directed() {
            in_adj[to as usize].push(from);
        }
    }

    for adj in &mut out_adj {
        adj.sort_unstable();
    }
    if graph.is_directed() {
        for adj in &mut in_adj {
            adj.sort_unstable();
        }
    }

    for (i, &u) in vertices.iter().enumerate() {
        let j_start = if directed { 0 } else { i + 1 };
        for &v in &vertices[j_start..] {
            if u == v {
                continue;
            }

            let has_edge = if directed {
                out_adj[u as usize].binary_search(&v).is_ok()
            } else {
                out_adj[u as usize].binary_search(&v).is_ok()
                    || out_adj[v as usize].binary_search(&u).is_ok()
                    || (graph.is_directed()
                        && (in_adj[u as usize].binary_search(&v).is_ok()
                            || in_adj[v as usize].binary_search(&u).is_ok()))
            };

            if independent_set {
                if has_edge {
                    return Ok(false);
                }
            } else if !has_edge {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clique_empty_set() {
        let g = Graph::with_vertices(5);
        assert!(is_clique(&g, &[], false).unwrap());
    }

    #[test]
    fn clique_singleton() {
        let g = Graph::with_vertices(5);
        assert!(is_clique(&g, &[2], false).unwrap());
    }

    #[test]
    fn clique_pair_connected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(is_clique(&g, &[0, 1], false).unwrap());
    }

    #[test]
    fn clique_pair_not_connected() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        assert!(!is_clique(&g, &[0, 2], false).unwrap());
    }

    #[test]
    fn clique_triangle() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_clique(&g, &[0, 1, 2], false).unwrap());
        assert!(!is_clique(&g, &[0, 1, 2, 3], false).unwrap());
    }

    #[test]
    fn clique_directed_both_directions() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        // directed=true: need both 0→2 and 2→0, but 2→0 missing
        assert!(!is_clique(&g, &[0, 1, 2], true).unwrap());
        // directed=false: any direction suffices (0→2 and 1→2 exist)
        assert!(is_clique(&g, &[0, 1, 2], false).unwrap());
    }

    #[test]
    fn clique_directed_full() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 0).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 1).unwrap();
        assert!(is_clique(&g, &[0, 1, 2], true).unwrap());
    }

    #[test]
    fn clique_with_self_loop() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 0).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        assert!(is_clique(&g, &[0, 1, 2], false).unwrap());
    }

    #[test]
    fn clique_vertex_out_of_range() {
        let g = Graph::with_vertices(3);
        assert!(is_clique(&g, &[0, 5], false).is_err());
    }

    #[test]
    fn independent_empty_set() {
        let g = Graph::with_vertices(5);
        assert!(is_independent_vertex_set(&g, &[]).unwrap());
    }

    #[test]
    fn independent_singleton() {
        let g = Graph::with_vertices(5);
        assert!(is_independent_vertex_set(&g, &[3]).unwrap());
    }

    #[test]
    fn independent_no_edges_between() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(is_independent_vertex_set(&g, &[0, 2]).unwrap());
        assert!(is_independent_vertex_set(&g, &[1, 3]).unwrap());
        assert!(is_independent_vertex_set(&g, &[0, 3]).unwrap());
    }

    #[test]
    fn independent_has_edge() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(!is_independent_vertex_set(&g, &[0, 1]).unwrap());
        assert!(!is_independent_vertex_set(&g, &[0, 1, 2]).unwrap());
    }

    #[test]
    fn independent_directed_ignores_direction() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap(); // only 0→1, not 1→0
        assert!(!is_independent_vertex_set(&g, &[0, 1]).unwrap());
    }

    #[test]
    fn independent_all_isolated() {
        let g = Graph::with_vertices(5);
        assert!(is_independent_vertex_set(&g, &[0, 1, 2, 3, 4]).unwrap());
    }
}
